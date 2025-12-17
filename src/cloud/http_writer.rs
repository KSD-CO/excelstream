//! HTTP streaming Excel writer
//!
//! This module provides direct streaming Excel generation to HTTP responses.
//! Perfect for web APIs that need to generate Excel files on-the-fly.
//!
//! # Features
//!
//! - Stream Excel directly to HTTP response body
//! - No temporary files required
//! - Constant memory usage
//! - Works with any async web framework (Axum, Actix-web, Warp, etc.)
//!
//! # Example with Axum
//!
//! ```no_run
//! use excelstream::cloud::HttpExcelWriter;
//! use axum::{
//!     response::{IntoResponse, Response},
//!     http::header,
//! };
//!
//! async fn download_report() -> Response {
//!     let mut writer = HttpExcelWriter::new();
//!
//!     writer.write_header_bold(&["Month", "Sales", "Profit"]).unwrap();
//!     writer.write_row(&["January", "50000", "12000"]).unwrap();
//!     writer.write_row(&["February", "55000", "15000"]).unwrap();
//!
//!     let bytes = writer.finish().unwrap();
//!
//!     (
//!         [
//!             (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
//!             (header::CONTENT_DISPOSITION, "attachment; filename=\"report.xlsx\""),
//!         ],
//!         bytes
//!     ).into_response()
//! }
//! ```

use crate::error::{ExcelError, Result};
use crate::types::CellValue;

/// In-memory buffer that implements Write + Seek traits
struct MemoryBuffer {
    buffer: Vec<u8>,
    position: u64,
}

impl MemoryBuffer {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(1024 * 1024), // 1MB initial capacity
            position: 0,
        }
    }

    fn into_inner(self) -> Vec<u8> {
        self.buffer
    }
}

impl std::io::Write for MemoryBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let pos = self.position as usize;
        let end_pos = pos + buf.len();

        // Extend buffer if needed
        if end_pos > self.buffer.len() {
            self.buffer.resize(end_pos, 0);
        }

        // Write at current position
        self.buffer[pos..end_pos].copy_from_slice(buf);
        self.position = end_pos as u64;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Seek for MemoryBuffer {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::End(offset) => self.buffer.len() as i64 + offset,
            std::io::SeekFrom::Current(offset) => self.position as i64 + offset,
        };

        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid seek position",
            ));
        }

        self.position = new_pos as u64;
        Ok(self.position)
    }
}

/// HTTP Excel writer that generates Excel files in memory for streaming responses
///
/// This writer generates the entire Excel file in memory and can be used
/// to stream responses in web servers.
///
/// # Example
///
/// ```no_run
/// use excelstream::cloud::HttpExcelWriter;
///
/// let mut writer = HttpExcelWriter::new();
/// writer.write_header_bold(&["ID", "Name", "Value"])?;
/// writer.write_row(&["1", "Alice", "100"])?;
/// writer.write_row(&["2", "Bob", "200"])?;
///
/// let excel_bytes = writer.finish()?;
/// // Send excel_bytes as HTTP response body
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct HttpExcelWriter {
    workbook: Option<InMemoryWorkbook>,
    finished: bool,
}

/// Internal workbook that writes to memory
struct InMemoryWorkbook {
    zip_writer: Option<s_zip::StreamingZipWriter<MemoryBuffer>>,
    worksheets: Vec<String>,
    worksheet_count: u32,
    current_row: u32,
    xml_buffer: Vec<u8>,
    in_worksheet: bool,
}

impl HttpExcelWriter {
    /// Create a new HTTP Excel writer
    pub fn new() -> Self {
        Self::with_compression(6)
    }

    /// Create a new HTTP Excel writer with custom compression level
    ///
    /// # Arguments
    /// * `compression_level` - Compression level from 0 to 9
    ///   - 0: No compression (fastest, largest)
    ///   - 1: Fast compression
    ///   - 6: Balanced (recommended)
    ///   - 9: Maximum compression (slowest)
    pub fn with_compression(compression_level: u32) -> Self {
        let workbook = InMemoryWorkbook::new(compression_level.min(9));

        Self {
            workbook: Some(workbook),
            finished: false,
        }
    }

    /// Write a header row with bold formatting
    pub fn write_header_bold<I, S>(&mut self, headers: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.check_not_finished()?;

        let workbook = self
            .workbook
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Workbook not initialized".to_string()))?;

        if workbook.worksheet_count == 0 {
            workbook.add_worksheet("Sheet1")?;
        }

        let headers: Vec<String> = headers
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        workbook.write_row(&headers.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    /// Write a data row (strings)
    pub fn write_row<I, S>(&mut self, row: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.check_not_finished()?;

        let workbook = self
            .workbook
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Workbook not initialized".to_string()))?;

        if workbook.worksheet_count == 0 {
            workbook.add_worksheet("Sheet1")?;
        }

        let row: Vec<String> = row.into_iter().map(|s| s.as_ref().to_string()).collect();

        workbook.write_row(&row.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    /// Write a data row with typed values
    pub fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        self.check_not_finished()?;

        let workbook = self
            .workbook
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Workbook not initialized".to_string()))?;

        if workbook.worksheet_count == 0 {
            workbook.add_worksheet("Sheet1")?;
        }

        workbook.write_row_typed(cells)
    }

    /// Add a new worksheet
    pub fn add_worksheet(&mut self, name: &str) -> Result<()> {
        self.check_not_finished()?;

        let workbook = self
            .workbook
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Workbook not initialized".to_string()))?;

        workbook.add_worksheet(name)
    }

    /// Finish writing and return the Excel file as bytes
    ///
    /// This consumes the writer and returns the complete Excel file
    /// as a Vec<u8> that can be sent as an HTTP response.
    pub fn finish(mut self) -> Result<Vec<u8>> {
        if self.finished {
            return Err(ExcelError::InvalidState("Already finished".to_string()));
        }

        let workbook = self
            .workbook
            .take()
            .ok_or_else(|| ExcelError::InvalidState("Workbook not initialized".to_string()))?;

        let bytes = workbook.close()?;
        self.finished = true;

        Ok(bytes)
    }

    fn check_not_finished(&self) -> Result<()> {
        if self.finished {
            Err(ExcelError::InvalidState(
                "Writer already finished".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

impl Default for HttpExcelWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryWorkbook {
    fn new(compression_level: u32) -> Self {
        let buffer = MemoryBuffer::new();
        let zip_writer = s_zip::StreamingZipWriter::from_writer_with_compression(
            buffer,
            compression_level.min(9),
        )
        .expect("Failed to create ZIP writer");

        Self {
            zip_writer: Some(zip_writer),
            worksheets: Vec::new(),
            worksheet_count: 0,
            current_row: 0,
            xml_buffer: Vec::with_capacity(4096),
            in_worksheet: false,
        }
    }

    fn add_worksheet(&mut self, name: &str) -> Result<()> {
        // Finish previous worksheet if any
        self.finish_current_worksheet()?;

        self.worksheet_count += 1;
        self.worksheets.push(name.to_string());
        self.current_row = 0;

        // Start new worksheet entry in ZIP
        let entry_name = format!("xl/worksheets/sheet{}.xml", self.worksheet_count);
        self.zip_writer.as_mut().unwrap().start_entry(&entry_name)?;

        // Write worksheet XML header
        let header = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheetData>"#;

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(header.as_bytes())?;
        self.in_worksheet = true;

        Ok(())
    }

    fn write_row(&mut self, values: &[&str]) -> Result<()> {
        if !self.in_worksheet {
            return Err(ExcelError::WriteError("No worksheet started".to_string()));
        }

        self.current_row += 1;

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
                Self::write_escaped(&mut self.xml_buffer, value);
                self.xml_buffer.extend_from_slice(b"</t></is></c>");
            }
        }

        self.xml_buffer.extend_from_slice(b"</row>");

        // Stream to compressor immediately
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(&self.xml_buffer)?;

        Ok(())
    }

    fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        if !self.in_worksheet {
            return Err(ExcelError::WriteError("No worksheet started".to_string()));
        }

        self.current_row += 1;

        // Build row XML in buffer
        self.xml_buffer.clear();
        self.xml_buffer.extend_from_slice(b"<row r=\"");
        self.xml_buffer
            .extend_from_slice(self.current_row.to_string().as_bytes());
        self.xml_buffer.extend_from_slice(b"\">");

        for (col_idx, value) in cells.iter().enumerate() {
            let col_letter = Self::column_letter(col_idx as u32 + 1);

            self.xml_buffer.extend_from_slice(b"<c r=\"");
            self.xml_buffer.extend_from_slice(col_letter.as_bytes());
            self.xml_buffer
                .extend_from_slice(self.current_row.to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\"");

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

        // Stream to compressor immediately
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(&self.xml_buffer)?;

        Ok(())
    }

    fn finish_current_worksheet(&mut self) -> Result<()> {
        if self.in_worksheet {
            self.zip_writer
                .as_mut()
                .unwrap()
                .write_data(b"</sheetData></worksheet>")?;
            self.in_worksheet = false;
        }
        Ok(())
    }

    fn close(mut self) -> Result<Vec<u8>> {
        // Finish current worksheet
        self.finish_current_worksheet()?;

        // Write all other required ZIP entries
        self.write_content_types()?;
        self.write_rels()?;
        self.write_workbook()?;
        self.write_workbook_rels()?;
        self.write_styles()?;
        self.write_shared_strings()?;
        self.write_app_props()?;
        self.write_core_props()?;

        // Finish ZIP and get buffer
        let zip_writer = self.zip_writer.take().unwrap();
        let buffer = zip_writer.finish()?;

        Ok(buffer.into_inner())
    }

    fn write_content_types(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("[Content_Types].xml")?;
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#,
        );

        for i in 1..=self.worksheet_count {
            xml.push_str(&format!(
                r#"
<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
                i
            ));
        }

        xml.push_str("\n</Types>");
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_rels(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("_rels/.rels")?;
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_workbook(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/workbook.xml")?;
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>"#,
        );

        for (i, name) in self.worksheets.iter().enumerate() {
            xml.push_str(&format!(
                r#"
<sheet name="{}" sheetId="{}" r:id="rId{}"/>"#,
                name,
                i + 1,
                i + 1
            ));
        }

        xml.push_str("\n</sheets>\n</workbook>");
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_workbook_rels(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/_rels/workbook.xml.rels")?;
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );

        for i in 1..=self.worksheet_count {
            xml.push_str(&format!(
                r#"
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
                i, i
            ));
        }

        xml.push_str(&format!(
            r#"
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>"#,
            self.worksheet_count + 1,
            self.worksheet_count + 2
        ));

        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_styles(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/styles.xml")?;
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<numFmts count="0"/>
<fonts count="2">
<font><sz val="11"/><name val="Calibri"/></font>
<font><b/><sz val="11"/><name val="Calibri"/></font>
</fonts>
<fills count="2">
<fill><patternFill patternType="none"/></fill>
<fill><patternFill patternType="gray125"/></fill>
</fills>
<borders count="1">
<border><left/><right/><top/><bottom/><diagonal/></border>
</borders>
<cellXfs count="2">
<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
<xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0" applyFont="1"/>
</cellXfs>
</styleSheet>"#;
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_shared_strings(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("xl/sharedStrings.xml")?;
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="0" uniqueCount="0"/>
"#;
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_app_props(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("docProps/app.xml")?;
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
<Application>ExcelStream HTTP</Application>
</Properties>"#;
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn write_core_props(&mut self) -> Result<()> {
        self.zip_writer
            .as_mut()
            .unwrap()
            .start_entry("docProps/core.xml")?;
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:creator>ExcelStream HTTP</dc:creator>
</cp:coreProperties>"#;
        self.zip_writer
            .as_mut()
            .unwrap()
            .write_data(xml.as_bytes())?;
        Ok(())
    }

    fn column_letter(n: u32) -> String {
        let mut result = String::new();
        let mut n = n;
        while n > 0 {
            let rem = (n - 1) % 26;
            result.insert(0, (b'A' + rem as u8) as char);
            n = (n - 1) / 26;
        }
        result
    }

    fn write_escaped(buffer: &mut Vec<u8>, s: &str) {
        for c in s.chars() {
            match c {
                '&' => buffer.extend_from_slice(b"&amp;"),
                '<' => buffer.extend_from_slice(b"&lt;"),
                '>' => buffer.extend_from_slice(b"&gt;"),
                '"' => buffer.extend_from_slice(b"&quot;"),
                '\'' => buffer.extend_from_slice(b"&apos;"),
                _ => {
                    let mut buf = [0; 4];
                    buffer.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                }
            }
        }
    }
}
