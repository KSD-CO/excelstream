//! S3 Excel writer with TRUE direct streaming support (no temp files!)
//!
//! This module provides streaming Excel generation directly to Amazon S3
//! using s-zip's cloud support. NO local disk space required!

use crate::error::{ExcelError, Result};
use crate::types::{CellStyle, CellValue};

#[cfg(feature = "cloud-s3")]
use aws_sdk_s3::Client;
#[cfg(feature = "cloud-s3")]
use s_zip::cloud::S3ZipWriter;
#[cfg(feature = "cloud-s3")]
use s_zip::AsyncStreamingZipWriter;

/// S3 Excel writer that streams directly to Amazon S3 (no temp files!)
///
/// # Example
///
/// ```no_run
/// use excelstream::cloud::S3ExcelWriter;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut writer = S3ExcelWriter::builder()
///         .bucket("my-reports")
///         .key("monthly/2024-12.xlsx")
///         .region("us-east-1")
///         .build()
///         .await?;
///
///     writer.write_header_bold(&["Month", "Sales", "Profit"]).await?;
///     writer.write_row(&["January", "50000", "12000"]).await?;
///     writer.write_row(&["February", "55000", "15000"]).await?;
///
///     writer.save().await?;
///     println!("Report uploaded to S3!");
///     Ok(())
/// }
/// ```
pub struct S3ExcelWriter {
    zip_writer: Option<AsyncStreamingZipWriter<S3ZipWriter>>,
    current_row: u32,
    max_col: u32,
    xml_buffer: Vec<u8>,
    worksheet_count: u32,
    worksheets: Vec<String>,
    in_worksheet: bool,
}

impl std::fmt::Debug for S3ExcelWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3ExcelWriter")
            .field("current_row", &self.current_row)
            .field("max_col", &self.max_col)
            .field("worksheet_count", &self.worksheet_count)
            .field("worksheets", &self.worksheets)
            .field("in_worksheet", &self.in_worksheet)
            .field("has_zip_writer", &self.zip_writer.is_some())
            .finish()
    }
}

impl S3ExcelWriter {
    /// Create a new S3 Excel writer builder
    pub fn builder() -> S3ExcelWriterBuilder {
        S3ExcelWriterBuilder::default()
    }

    async fn ensure_worksheet(&mut self) -> Result<()> {
        if !self.in_worksheet {
            self.add_worksheet("Sheet1").await?;
        }
        Ok(())
    }

    /// Add a new worksheet
    async fn add_worksheet(&mut self, name: &str) -> Result<()> {
        if self.in_worksheet {
            self.finish_current_worksheet().await?;
        }

        self.worksheet_count += 1;
        self.worksheets.push(name.to_string());
        self.current_row = 0;
        self.max_col = 0;

        let entry_name = format!("xl/worksheets/sheet{}.xml", self.worksheet_count);
        self.zip_writer
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Writer not initialized".to_string()))?
            .start_entry(&entry_name)
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

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

            if style_id > 0 {
                self.xml_buffer.extend_from_slice(b" s=\"");
                self.xml_buffer
                    .extend_from_slice(style_id.to_string().as_bytes());
                self.xml_buffer.extend_from_slice(b"\"");
            }

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

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(&self.xml_buffer)
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    /// Save and upload Excel file to S3 (streaming, no temp files!)
    pub async fn save(mut self) -> Result<()> {
        self.finish_current_worksheet().await?;

        self.write_content_types().await?;
        self.write_rels().await?;
        self.write_workbook().await?;
        self.write_workbook_rels().await?;
        self.write_styles().await?;

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

/// Builder for S3ExcelWriter
///
/// Supports AWS S3 and S3-compatible services (MinIO, Cloudflare R2, DigitalOcean Spaces, etc.)
pub struct S3ExcelWriterBuilder {
    bucket: Option<String>,
    key: Option<String>,
    region: Option<String>,
    endpoint_url: Option<String>,
    force_path_style: bool,
}

impl Default for S3ExcelWriterBuilder {
    fn default() -> Self {
        Self {
            bucket: None,
            key: None,
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
            force_path_style: false,
        }
    }
}

impl S3ExcelWriterBuilder {
    /// Set the S3 bucket name
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Set the S3 object key (file path)
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the AWS region
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set custom endpoint URL for S3-compatible services
    ///
    /// # Supported Services
    ///
    /// - MinIO: `http://localhost:9000`
    /// - Cloudflare R2: `https://<account_id>.r2.cloudflarestorage.com`
    /// - DigitalOcean Spaces: `https://nyc3.digitaloceanspaces.com`
    pub fn endpoint_url(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint_url = Some(endpoint.into());
        self
    }

    /// Force path-style addressing (required for MinIO and some S3-compatible services)
    ///
    /// When enabled, uses `http://endpoint/bucket/key` instead of `http://bucket.endpoint/key`
    pub fn force_path_style(mut self, force: bool) -> Self {
        self.force_path_style = force;
        self
    }

    /// Build the S3ExcelWriter
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use excelstream::cloud::S3ExcelWriter;
    ///
    /// // AWS S3
    /// let writer = S3ExcelWriter::builder()
    ///     .bucket("my-bucket")
    ///     .key("report.xlsx")
    ///     .region("us-east-1")
    ///     .build()
    ///     .await?;
    ///
    /// // MinIO
    /// let writer = S3ExcelWriter::builder()
    ///     .endpoint_url("http://localhost:9000")
    ///     .bucket("my-bucket")
    ///     .key("report.xlsx")
    ///     .region("us-east-1")
    ///     .force_path_style(true)
    ///     .build()
    ///     .await?;
    /// ```
    #[cfg(feature = "cloud-s3")]
    pub async fn build(self) -> Result<S3ExcelWriter> {
        let bucket = self
            .bucket
            .ok_or_else(|| ExcelError::InvalidState("Bucket name required".to_string()))?;
        let key = self
            .key
            .ok_or_else(|| ExcelError::InvalidState("Object key required".to_string()))?;
        let region = self.region.unwrap_or_else(|| "us-east-1".to_string());

        let mut builder = S3ZipWriter::builder()
            .region(&region)
            .bucket(&bucket)
            .key(&key);

        if let Some(endpoint) = &self.endpoint_url {
            builder = builder.endpoint_url(endpoint);
        }

        if self.force_path_style {
            builder = builder.force_path_style(true);
        }

        let s3_writer = builder
            .build()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Self::create_writer_from_s3_writer(s3_writer)
    }

    #[cfg(not(feature = "cloud-s3"))]
    pub async fn build(self) -> Result<S3ExcelWriter> {
        Err(ExcelError::InvalidState(
            "cloud-s3 feature not enabled".to_string(),
        ))
    }

    #[cfg(feature = "cloud-s3")]
    pub async fn build_with_client(self, client: Client) -> Result<S3ExcelWriter> {
        let bucket = self
            .bucket
            .ok_or_else(|| ExcelError::InvalidState("Bucket name required".to_string()))?;
        let key = self
            .key
            .ok_or_else(|| ExcelError::InvalidState("Object key required".to_string()))?;
        let region = self.region.unwrap_or_else(|| "us-east-1".to_string());

        let s3_writer = S3ZipWriter::builder()
            .client(client)
            .region(&region)
            .bucket(&bucket)
            .key(&key)
            .build()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Self::create_writer_from_s3_writer(s3_writer)
    }

    #[cfg(not(feature = "cloud-s3"))]
    pub async fn build_with_client(self, _client: Client) -> Result<S3ExcelWriter> {
        Err(ExcelError::InvalidState(
            "cloud-s3 feature not enabled".to_string(),
        ))
    }

    #[cfg(feature = "cloud-s3")]
    fn create_writer_from_s3_writer(s3_writer: S3ZipWriter) -> Result<S3ExcelWriter> {
        let zip_writer = AsyncStreamingZipWriter::from_writer(s3_writer);

        Ok(S3ExcelWriter {
            zip_writer: Some(zip_writer),
            current_row: 0,
            max_col: 0,
            xml_buffer: Vec::with_capacity(4096),
            worksheet_count: 0,
            worksheets: Vec::new(),
            in_worksheet: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_validation_missing_bucket() {
        let builder = S3ExcelWriterBuilder::default().key("test.xlsx");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(builder.build());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Bucket name required"));
    }

    #[test]
    fn test_builder_validation_missing_key() {
        let builder = S3ExcelWriterBuilder::default().bucket("test-bucket");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(builder.build());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Object key required"));
    }

    #[test]
    fn test_default_region() {
        let builder = S3ExcelWriterBuilder::default();
        assert_eq!(builder.region, Some("us-east-1".to_string()));
    }

    #[test]
    fn test_builder_methods() {
        let builder = S3ExcelWriterBuilder::default()
            .bucket("my-bucket")
            .key("path/to/file.xlsx")
            .region("ap-southeast-1")
            .endpoint_url("http://localhost:9000")
            .force_path_style(true);

        assert_eq!(builder.bucket, Some("my-bucket".to_string()));
        assert_eq!(builder.key, Some("path/to/file.xlsx".to_string()));
        assert_eq!(builder.region, Some("ap-southeast-1".to_string()));
        assert_eq!(
            builder.endpoint_url,
            Some("http://localhost:9000".to_string())
        );
        assert!(builder.force_path_style);
    }

    #[cfg(feature = "cloud-s3")]
    #[tokio::test]
    async fn test_build_with_client() {
        use aws_config::BehaviorVersion;
        use aws_sdk_s3::config::Region;

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .load()
            .await;
        let client = Client::new(&config);

        let result = S3ExcelWriterBuilder::default()
            .bucket("test-bucket")
            .key("test.xlsx")
            .build_with_client(client)
            .await;

        assert!(result.is_ok());
    }

    #[cfg(feature = "cloud-s3")]
    #[tokio::test]
    async fn test_build_with_client_missing_bucket() {
        use aws_config::BehaviorVersion;
        use aws_sdk_s3::config::Region;

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .load()
            .await;
        let client = Client::new(&config);

        let result = S3ExcelWriterBuilder::default()
            .key("test.xlsx")
            .build_with_client(client)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Bucket name required"));
    }

    #[cfg(feature = "cloud-s3")]
    #[tokio::test]
    async fn test_build_with_client_missing_key() {
        use aws_config::BehaviorVersion;
        use aws_sdk_s3::config::Region;

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .load()
            .await;
        let client = Client::new(&config);

        let result = S3ExcelWriterBuilder::default()
            .bucket("test-bucket")
            .build_with_client(client)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Object key required"));
    }
}
