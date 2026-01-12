//! GCS Excel writer with TRUE direct streaming support (no temp files!)
//!
//! This module provides streaming Excel generation directly to Google Cloud Storage
//! using s-zip's cloud support. NO local disk space required!

use crate::error::{ExcelError, Result};
use crate::types::{CellStyle, CellValue};

#[cfg(feature = "cloud-gcs")]
use s_zip::cloud::GCSZipWriter;
#[cfg(feature = "cloud-gcs")]
use s_zip::AsyncStreamingZipWriter;

/// GCS Excel writer that streams directly to Google Cloud Storage (no temp files!)
///
/// # Example
///
/// ```no_run
/// use excelstream::cloud::GCSExcelWriter;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut writer = GCSExcelWriter::builder()
///         .bucket("my-reports")
///         .object("monthly/2024-12.xlsx")
///         .build()
///         .await?;
///
///     writer.write_header_bold(&["Month", "Sales", "Profit"]).await?;
///     writer.write_row(&["January", "50000", "12000"]).await?;
///     writer.write_row(&["February", "55000", "15000"]).await?;
///
///     writer.save().await?;
///     println!("Report uploaded to GCS!");
///     Ok(())
/// }
/// ```
pub struct GCSExcelWriter {
    zip_writer: Option<AsyncStreamingZipWriter<GCSZipWriter>>,
    current_row: u32,
    max_col: u32,
    xml_buffer: Vec<u8>,
    worksheet_count: u32,
    worksheets: Vec<String>,
    in_worksheet: bool,
}

impl GCSExcelWriter {
    /// Create a new GCS Excel writer builder
    pub fn builder() -> GCSExcelWriterBuilder {
        GCSExcelWriterBuilder::default()
    }

    async fn ensure_worksheet(&mut self) -> Result<()> {
        if !self.in_worksheet {
            self.add_worksheet("Sheet1").await?;
        }
        Ok(())
    }

    /// Add a new worksheet
    async fn add_worksheet(&mut self, name: &str) -> Result<()> {
        // Finish previous worksheet if any
        if self.in_worksheet {
            self.finish_current_worksheet().await?;
        }

        self.worksheet_count += 1;
        self.worksheets.push(name.to_string());
        self.current_row = 0;
        self.max_col = 0;

        // Start new worksheet entry in ZIP
        let entry_name = format!("xl/worksheets/sheet{}.xml", self.worksheet_count);
        self.zip_writer
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Writer not initialized".to_string()))?
            .start_entry(&entry_name)
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        // Write worksheet XML header
        let header = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheetData>"#;

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(header.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        self.in_worksheet = true;
        Ok(())
    }

    async fn finish_current_worksheet(&mut self) -> Result<()> {
        if !self.in_worksheet {
            return Ok(());
        }

        // Close sheetData and worksheet tags
        let footer = "</sheetData></worksheet>";
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(footer.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        self.in_worksheet = false;
        Ok(())
    }

    /// Write a header row with bold formatting
    pub async fn write_header_bold<I, S>(&mut self, headers: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.ensure_worksheet().await?;

        let cells: Vec<_> = headers
            .into_iter()
            .map(|h| {
                crate::types::StyledCell::new(
                    CellValue::String(h.as_ref().to_string()),
                    CellStyle::HeaderBold,
                )
            })
            .collect();

        self.write_row_styled(&cells).await
    }

    /// Write a data row (strings)
    pub async fn write_row<I, S>(&mut self, row: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.ensure_worksheet().await?;

        self.current_row += 1;
        let values: Vec<String> = row.into_iter().map(|s| s.as_ref().to_string()).collect();
        self.max_col = self.max_col.max(values.len() as u32);

        // Build row XML in buffer
        self.xml_buffer.clear();
        self.xml_buffer.extend_from_slice(b"<row r=\"");
        self.xml_buffer
            .extend_from_slice(self.current_row.to_string().as_bytes());
        self.xml_buffer.extend_from_slice(b"\">");

        for (col_idx, value) in values.iter().enumerate() {
            let col_letter = Self::column_letter(col_idx as u32 + 1);
            self.xml_buffer.extend_from_slice(b"<c r=\"");
            self.xml_buffer.extend_from_slice(col_letter.as_bytes());
            self.xml_buffer
                .extend_from_slice(self.current_row.to_string().as_bytes());

            if value.is_empty() {
                self.xml_buffer.extend_from_slice(b"\"/>");
            } else {
                self.xml_buffer
                    .extend_from_slice(b"\" t=\"inlineStr\"><is><t>");
                Self::write_escaped(&mut self.xml_buffer, value.as_str());
                self.xml_buffer.extend_from_slice(b"</t></is></c>");
            }
        }

        self.xml_buffer.extend_from_slice(b"</row>");

        // Stream to GCS immediately
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(&self.xml_buffer)
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    /// Write a data row with typed values
    pub async fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        let styled_cells: Vec<_> = cells
            .iter()
            .map(|v| crate::types::StyledCell::new(v.clone(), CellStyle::Default))
            .collect();

        self.write_row_styled(&styled_cells).await
    }

    /// Write a row with styled cells
    async fn write_row_styled(&mut self, cells: &[crate::types::StyledCell]) -> Result<()> {
        self.ensure_worksheet().await?;

        self.current_row += 1;
        self.max_col = self.max_col.max(cells.len() as u32);

        // Build row XML in buffer
        self.xml_buffer.clear();
        self.xml_buffer.extend_from_slice(b"<row r=\"");
        self.xml_buffer
            .extend_from_slice(self.current_row.to_string().as_bytes());
        self.xml_buffer.extend_from_slice(b"\">");

        for (col_idx, styled_cell) in cells.iter().enumerate() {
            let col_letter = Self::column_letter(col_idx as u32 + 1);
            let value = &styled_cell.value;
            let style_id = styled_cell.style.index();

            self.xml_buffer.extend_from_slice(b"<c r=\"");
            self.xml_buffer.extend_from_slice(col_letter.as_bytes());
            self.xml_buffer
                .extend_from_slice(self.current_row.to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\"");

            // Add style attribute if not default
            if style_id > 0 {
                self.xml_buffer.extend_from_slice(b" s=\"");
                self.xml_buffer
                    .extend_from_slice(style_id.to_string().as_bytes());
                self.xml_buffer.extend_from_slice(b"\"");
            }

            // Write cell value based on type
            match value {
                CellValue::Empty => {
                    self.xml_buffer.extend_from_slice(b"/>");
                }
                CellValue::Int(i) => {
                    self.xml_buffer.extend_from_slice(b" t=\"n\"><v>");
                    self.xml_buffer.extend_from_slice(i.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Float(f) => {
                    self.xml_buffer.extend_from_slice(b" t=\"n\"><v>");
                    self.xml_buffer.extend_from_slice(f.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Bool(b) => {
                    self.xml_buffer.extend_from_slice(b" t=\"b\"><v>");
                    self.xml_buffer
                        .extend_from_slice(if *b { b"1" } else { b"0" });
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::String(s) => {
                    self.xml_buffer
                        .extend_from_slice(b" t=\"inlineStr\"><is><t>");
                    Self::write_escaped(&mut self.xml_buffer, s);
                    self.xml_buffer.extend_from_slice(b"</t></is></c>");
                }
                CellValue::Formula(f) => {
                    self.xml_buffer.extend_from_slice(b"><f>");
                    Self::write_escaped(&mut self.xml_buffer, f);
                    self.xml_buffer.extend_from_slice(b"</f></c>");
                }
                CellValue::DateTime(dt) => {
                    self.xml_buffer.extend_from_slice(b" t=\"n\"><v>");
                    self.xml_buffer.extend_from_slice(dt.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Error(e) => {
                    self.xml_buffer.extend_from_slice(b" t=\"e\"><v>");
                    Self::write_escaped(&mut self.xml_buffer, e);
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
            }
        }

        self.xml_buffer.extend_from_slice(b"</row>");

        // Stream to GCS immediately
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(&self.xml_buffer)
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    /// Save and upload Excel file to GCS (streaming, no temp files!)
    pub async fn save(mut self) -> Result<()> {
        // Finish current worksheet
        self.finish_current_worksheet().await?;

        // Write all required Excel files
        self.write_content_types().await?;
        self.write_rels().await?;
        self.write_workbook().await?;
        self.write_workbook_rels().await?;
        self.write_styles().await?;

        // Finish ZIP - this completes the GCS upload
        let zip_writer = self
            .zip_writer
            .take()
            .ok_or_else(|| ExcelError::InvalidState("Writer not initialized".to_string()))?;

        zip_writer
            .finish()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    async fn write_content_types(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("[Content_Types].xml")
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#,
        );

        for i in 1..=self.worksheet_count {
            xml.push_str(&format!(
                r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
                i
            ));
        }

        xml.push_str("</Types>");

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    async fn write_rels(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("_rels/.rels")
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    async fn write_workbook(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/workbook.xml")
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>"#,
        );

        for (idx, name) in self.worksheets.iter().enumerate() {
            let sheet_id = idx + 1;
            xml.push_str(&format!(
                r#"<sheet name="{}" sheetId="{}" r:id="rId{}"/>"#,
                name, sheet_id, sheet_id
            ));
        }

        xml.push_str("</sheets></workbook>");

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    async fn write_workbook_rels(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/_rels/workbook.xml.rels")
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );

        for i in 1..=self.worksheet_count {
            xml.push_str(&format!(
                r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
                i, i
            ));
        }

        xml.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
            self.worksheet_count + 1
        ));

        xml.push_str("</Relationships>");

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    async fn write_styles(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/styles.xml")
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="2">
<font><sz val="11"/><name val="Calibri"/></font>
<font><b/><sz val="11"/><name val="Calibri"/></font>
</fonts>
<fills count="1"><fill><patternFill patternType="none"/></fill></fills>
<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
<cellXfs count="2">
<xf numFmtId="0" fontId="0" fillId="0" borderId="0"/>
<xf numFmtId="0" fontId="1" fillId="0" borderId="0" applyFont="1"/>
</cellXfs>
</styleSheet>"#;

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    fn column_letter(col: u32) -> String {
        let mut result = String::new();
        let mut n = col;
        while n > 0 {
            n -= 1;
            result.insert(0, (b'A' + (n % 26) as u8) as char);
            n /= 26;
        }
        result
    }

    fn write_escaped(buffer: &mut Vec<u8>, text: &str) {
        for ch in text.chars() {
            match ch {
                '<' => buffer.extend_from_slice(b"&lt;"),
                '>' => buffer.extend_from_slice(b"&gt;"),
                '&' => buffer.extend_from_slice(b"&amp;"),
                '"' => buffer.extend_from_slice(b"&quot;"),
                '\'' => buffer.extend_from_slice(b"&apos;"),
                _ => {
                    let mut buf = [0; 4];
                    buffer.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
            }
        }
    }
}

/// Builder for GCSExcelWriter
#[derive(Default)]
pub struct GCSExcelWriterBuilder {
    bucket: Option<String>,
    object: Option<String>,
}

impl GCSExcelWriterBuilder {
    /// Set the GCS bucket name
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Set the GCS object name (file path)
    pub fn object(mut self, object: impl Into<String>) -> Self {
        self.object = Some(object.into());
        self
    }

    /// Build the GCSExcelWriter
    #[cfg(feature = "cloud-gcs")]
    pub async fn build(self) -> Result<GCSExcelWriter> {
        use google_cloud_storage::client::{Client, ClientConfig};

        let bucket = self
            .bucket
            .ok_or_else(|| ExcelError::InvalidState("Bucket name required".to_string()))?;
        let object = self
            .object
            .ok_or_else(|| ExcelError::InvalidState("Object name required".to_string()))?;

        // Create GCS client
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;
        let client = Client::new(config);

        // Create GCS writer - streams directly to GCS!
        let gcs_writer = GCSZipWriter::new(client, &object, &bucket)
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        // Wrap in AsyncStreamingZipWriter
        let zip_writer = AsyncStreamingZipWriter::from_writer(gcs_writer);

        Ok(GCSExcelWriter {
            zip_writer: Some(zip_writer),
            current_row: 0,
            max_col: 0,
            xml_buffer: Vec::with_capacity(4096),
            worksheet_count: 0,
            worksheets: Vec::new(),
            in_worksheet: false,
        })
    }

    #[cfg(not(feature = "cloud-gcs"))]
    pub async fn build(self) -> Result<GCSExcelWriter> {
        Err(ExcelError::InvalidState(
            "cloud-gcs feature not enabled".to_string(),
        ))
    }
}
