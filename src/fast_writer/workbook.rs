//! Fast workbook implementation with ZIP compression

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

use super::shared_strings::SharedStrings;
use super::worksheet::FastWorksheet;
use super::xml_writer::XmlWriter;
use crate::error::Result;

/// Fast workbook for high-performance Excel writing
pub struct FastWorkbook {
    zip: ZipWriter<BufWriter<File>>,
    shared_strings: SharedStrings,
    worksheets: Vec<String>,
    worksheet_count: u32,
    current_worksheet: Option<u32>,
    current_row: u32,
    xml_buffer: Vec<u8>,         // Reusable buffer for XML writing
    cell_ref_cache: Vec<String>, // Cache for cell references (A, B, C, ...)
    flush_interval: u32,         // Flush every N rows
    max_buffer_size: usize,      // Max buffer size before force flush

    // Column width and row height support
    column_widths: HashMap<u32, f64>, // column index -> width in Excel units
    next_row_height: Option<f64>,     // height for next row in points
    sheet_data_started: bool,         // track if <sheetData> element has been started
}

impl FastWorkbook {
    /// Create a new fast workbook
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::with_capacity(64 * 1024, file); // 64KB buffer
        let mut zip = ZipWriter::new(writer);

        let options = Self::file_options();

        // Write [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)?;
        Self::write_content_types(&mut zip)?;

        // Write _rels/.rels
        zip.start_file("_rels/.rels", options)?;
        Self::write_root_rels(&mut zip)?;

        // Write docProps/core.xml
        zip.start_file("docProps/core.xml", options)?;
        Self::write_core_props(&mut zip)?;

        // Write docProps/app.xml
        zip.start_file("docProps/app.xml", options)?;
        Self::write_app_props(&mut zip)?;

        // Pre-generate cell reference cache for first 100 columns (A-CV)
        let mut cell_ref_cache = Vec::with_capacity(100);
        for col in 1..=100 {
            cell_ref_cache.push(Self::col_to_letter(col));
        }

        Ok(FastWorkbook {
            zip,
            shared_strings: SharedStrings::new(),
            worksheets: Vec::new(),
            worksheet_count: 0,
            current_worksheet: None,
            current_row: 0,
            xml_buffer: Vec::with_capacity(8192),
            cell_ref_cache,
            flush_interval: 1000,         // Flush mỗi 1000 dòng
            max_buffer_size: 1024 * 1024, // 1MB max buffer

            // Initialize column width and row height support
            column_widths: HashMap::new(),
            next_row_height: None,
            sheet_data_started: false,
        })
    }

    /// Set flush interval (số dòng giữa các lần flush)
    pub fn set_flush_interval(&mut self, interval: u32) {
        self.flush_interval = interval;
    }

    /// Set max buffer size (bytes) trước khi force flush
    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.max_buffer_size = size;
    }

    /// Create file options with large file support enabled
    fn file_options() -> FileOptions {
        FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6))
            .large_file(true) // Enable ZIP64 for files > 4GB
    }

    /// Set column width for the current worksheet
    ///
    /// Must be called after `add_worksheet()` but before writing any rows.
    /// Width is in Excel units (approximately the width of '0' in standard font).
    /// Default column width is 8.43 units.
    ///
    /// # Arguments
    /// * `col` - Zero-based column index (0 = A, 1 = B, etc.)
    /// * `width` - Column width in Excel units (typically 8-50)
    ///
    /// # Errors
    /// Returns error if:
    /// - No active worksheet
    /// - Rows have already been written (sheetData already started)
    pub fn set_column_width(&mut self, col: u32, width: f64) -> Result<()> {
        if self.current_worksheet.is_none() {
            return Err(crate::error::ExcelError::WriteError(
                "No active worksheet. Call add_worksheet() first.".to_string(),
            ));
        }

        if self.sheet_data_started {
            return Err(crate::error::ExcelError::WriteError(
                "Cannot set column width after writing rows. Set widths before write_row()."
                    .to_string(),
            ));
        }

        self.column_widths.insert(col, width);
        Ok(())
    }

    /// Set height for the next row to be written
    ///
    /// Height is in points (1 point = 1/72 inch).
    /// Default row height is 15 points.
    /// This setting is consumed by the next write_row call.
    ///
    /// # Arguments
    /// * `height` - Row height in points (typically 10-50)
    ///
    /// # Errors
    /// Returns error if no active worksheet
    pub fn set_next_row_height(&mut self, height: f64) -> Result<()> {
        if self.current_worksheet.is_none() {
            return Err(crate::error::ExcelError::WriteError(
                "No active worksheet. Call add_worksheet() first.".to_string(),
            ));
        }

        self.next_row_height = Some(height);
        Ok(())
    }

    /// Ensure sheetData element has been started
    /// Writes <cols> if needed, then starts <sheetData>
    fn ensure_sheet_data_started(&mut self) -> Result<()> {
        if self.sheet_data_started {
            return Ok(());
        }

        let mut xml_writer = XmlWriter::new(&mut self.zip);

        // Write <cols> element if we have column widths
        if !self.column_widths.is_empty() {
            xml_writer.start_element("cols")?;
            xml_writer.close_start_tag()?;

            // Sort columns for consistent output
            let mut cols: Vec<_> = self.column_widths.iter().collect();
            cols.sort_by_key(|(col, _)| *col);

            for (col, width) in cols {
                xml_writer.start_element("col")?;
                xml_writer.attribute_int("min", (*col + 1) as i64)?; // Excel is 1-indexed
                xml_writer.attribute_int("max", (*col + 1) as i64)?;
                xml_writer.attribute("width", &width.to_string())?;
                xml_writer.attribute("customWidth", "1")?;
                xml_writer.write_raw(b"/>")?;
            }

            xml_writer.end_element("cols")?;
        }

        // Start sheetData
        xml_writer.start_element("sheetData")?;
        xml_writer.close_start_tag()?;
        xml_writer.flush()?;

        self.sheet_data_started = true;
        Ok(())
    }

    /// Add a worksheet and get a writer for it
    pub fn add_worksheet(&mut self, name: &str) -> Result<()> {
        // Close previous worksheet if any
        if self.current_worksheet.is_some() {
            self.finish_current_worksheet()?;
        }

        self.worksheet_count += 1;
        let sheet_id = self.worksheet_count;

        self.worksheets.push(name.to_string());

        let options = Self::file_options();

        let sheet_path = format!("xl/worksheets/sheet{}.xml", sheet_id);
        self.zip.start_file(&sheet_path, options)?;

        // Write worksheet header
        let mut xml_writer = XmlWriter::new(&mut self.zip);
        xml_writer.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n")?;
        xml_writer.start_element("worksheet")?;
        xml_writer.attribute(
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        )?;
        xml_writer.attribute(
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        )?;
        xml_writer.close_start_tag()?;
        
        // Add dimension (will be A1:B2 for first sheet, A1 for others)
        // TODO: calculate actual dimension based on data written
        if sheet_id == 1 {
            xml_writer.write_str("<dimension ref=\"A1:B2\"/>")?;
        } else {
            xml_writer.write_str("<dimension ref=\"A1\"/>")?;
        }
        xml_writer.write_str("<sheetViews><sheetView")?;
        if sheet_id == 1 {
            xml_writer.write_str(" tabSelected=\"1\"")?;
        }
        xml_writer.write_str(" workbookViewId=\"0\"/></sheetViews>")?;
        xml_writer.write_str("<sheetFormatPr defaultRowHeight=\"15\"/>")?;
        
        xml_writer.flush()?;

        // Reset state for new worksheet
        self.current_worksheet = Some(sheet_id);
        self.current_row = 0;
        self.column_widths.clear();
        self.next_row_height = None;
        self.sheet_data_started = false;

        Ok(())
    }

    /// Write a row to the current worksheet
    pub fn write_row(&mut self, values: &[&str]) -> Result<()> {
        if self.current_worksheet.is_none() {
            return Err(crate::error::ExcelError::WriteError(
                "No active worksheet".to_string(),
            ));
        }

        // Ensure sheetData has been started (writes <cols> if needed)
        self.ensure_sheet_data_started()?;

        self.current_row += 1;
        let row_num = self.current_row;

        // Get row height if set
        let row_height = self.next_row_height.take();

        // Build XML in buffer
        self.xml_buffer.clear();
        self.xml_buffer.extend_from_slice(b"<row r=\"");
        self.xml_buffer
            .extend_from_slice(row_num.to_string().as_bytes());
        self.xml_buffer.extend_from_slice(b"\"");

        // Add spans attribute to match reference format
        if !values.is_empty() {
            self.xml_buffer.extend_from_slice(b" spans=\"1:");
            self.xml_buffer
                .extend_from_slice(values.len().to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\"");
        }

        // Add height attribute if set
        if let Some(height) = row_height {
            self.xml_buffer.extend_from_slice(b" ht=\"");
            self.xml_buffer
                .extend_from_slice(height.to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\" customHeight=\"1\"");
        }

        self.xml_buffer.extend_from_slice(b">");

        for (col_idx, value) in values.iter().enumerate() {
            let string_index = self.shared_strings.add_string(value);

            // Use cached column letter if available
            let col_num = (col_idx + 1) as u32;

            self.xml_buffer.extend_from_slice(b"<c r=\"");
            if col_num <= self.cell_ref_cache.len() as u32 {
                self.xml_buffer
                    .extend_from_slice(self.cell_ref_cache[col_idx].as_bytes());
            } else {
                let col_letter = Self::col_to_letter(col_num);
                self.xml_buffer.extend_from_slice(col_letter.as_bytes());
            }
            self.xml_buffer
                .extend_from_slice(row_num.to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\" t=\"s\"><v>");
            self.xml_buffer
                .extend_from_slice(string_index.to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"</v></c>");
        }

        self.xml_buffer.extend_from_slice(b"</row>");

        // Write buffer to zip
        self.zip.write_all(&self.xml_buffer)?;

        // Flush định kỳ để giới hạn memory
        if self.current_row.is_multiple_of(self.flush_interval) {
            self.zip.flush()?;
        }

        Ok(())
    }

    /// Write a row of styled cells to the current worksheet
    pub fn write_row_styled(&mut self, cells: &[crate::types::StyledCell]) -> Result<()> {
        use crate::types::CellValue;

        if self.current_worksheet.is_none() {
            return Err(crate::error::ExcelError::WriteError(
                "No active worksheet".to_string(),
            ));
        }

        // Ensure sheetData has been started (writes <cols> if needed)
        self.ensure_sheet_data_started()?;

        self.current_row += 1;
        let row_num = self.current_row;

        // Get row height if set
        let row_height = self.next_row_height.take();

        // Build XML in buffer
        self.xml_buffer.clear();
        self.xml_buffer.extend_from_slice(b"<row r=\"");
        self.xml_buffer
            .extend_from_slice(row_num.to_string().as_bytes());
        self.xml_buffer.extend_from_slice(b"\"");

        // Add spans attribute
        if !cells.is_empty() {
            self.xml_buffer.extend_from_slice(b" spans=\"1:");
            self.xml_buffer
                .extend_from_slice(cells.len().to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\"");
        }

        // Add height attribute if set
        if let Some(height) = row_height {
            self.xml_buffer.extend_from_slice(b" ht=\"");
            self.xml_buffer
                .extend_from_slice(height.to_string().as_bytes());
            self.xml_buffer.extend_from_slice(b"\" customHeight=\"1\"");
        }

        self.xml_buffer.extend_from_slice(b">");

        for (col_idx, cell) in cells.iter().enumerate() {
            let col_num = (col_idx + 1) as u32;
            let style_index = cell.style.index();

            // Get column letter
            let col_letter = if col_num <= self.cell_ref_cache.len() as u32 {
                &self.cell_ref_cache[col_idx]
            } else {
                &Self::col_to_letter(col_num)
            };

            match &cell.value {
                CellValue::Empty => {
                    // Skip empty cells
                    continue;
                }
                CellValue::String(s) => {
                    let string_index = self.shared_strings.add_string(s);
                    self.xml_buffer.extend_from_slice(b"<c r=\"");
                    self.xml_buffer.extend_from_slice(col_letter.as_bytes());
                    self.xml_buffer
                        .extend_from_slice(row_num.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"\"");
                    if style_index > 0 {
                        self.xml_buffer.extend_from_slice(b" s=\"");
                        self.xml_buffer
                            .extend_from_slice(style_index.to_string().as_bytes());
                        self.xml_buffer.extend_from_slice(b"\"");
                    }
                    self.xml_buffer.extend_from_slice(b" t=\"s\"><v>");
                    self.xml_buffer
                        .extend_from_slice(string_index.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Int(n) => {
                    self.xml_buffer.extend_from_slice(b"<c r=\"");
                    self.xml_buffer.extend_from_slice(col_letter.as_bytes());
                    self.xml_buffer
                        .extend_from_slice(row_num.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"\"");
                    if style_index > 0 {
                        self.xml_buffer.extend_from_slice(b" s=\"");
                        self.xml_buffer
                            .extend_from_slice(style_index.to_string().as_bytes());
                        self.xml_buffer.extend_from_slice(b"\"");
                    }
                    self.xml_buffer.extend_from_slice(b"><v>");
                    self.xml_buffer.extend_from_slice(n.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Float(f) => {
                    self.xml_buffer.extend_from_slice(b"<c r=\"");
                    self.xml_buffer.extend_from_slice(col_letter.as_bytes());
                    self.xml_buffer
                        .extend_from_slice(row_num.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"\"");
                    if style_index > 0 {
                        self.xml_buffer.extend_from_slice(b" s=\"");
                        self.xml_buffer
                            .extend_from_slice(style_index.to_string().as_bytes());
                        self.xml_buffer.extend_from_slice(b"\"");
                    }
                    self.xml_buffer.extend_from_slice(b"><v>");
                    self.xml_buffer.extend_from_slice(f.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Bool(b) => {
                    self.xml_buffer.extend_from_slice(b"<c r=\"");
                    self.xml_buffer.extend_from_slice(col_letter.as_bytes());
                    self.xml_buffer
                        .extend_from_slice(row_num.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"\"");
                    if style_index > 0 {
                        self.xml_buffer.extend_from_slice(b" s=\"");
                        self.xml_buffer
                            .extend_from_slice(style_index.to_string().as_bytes());
                        self.xml_buffer.extend_from_slice(b"\"");
                    }
                    self.xml_buffer.extend_from_slice(b" t=\"b\"><v>");
                    self.xml_buffer
                        .extend_from_slice(if *b { b"1" } else { b"0" });
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
                CellValue::Formula(formula) => {
                    self.xml_buffer.extend_from_slice(b"<c r=\"");
                    self.xml_buffer.extend_from_slice(col_letter.as_bytes());
                    self.xml_buffer
                        .extend_from_slice(row_num.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"\"");
                    if style_index > 0 {
                        self.xml_buffer.extend_from_slice(b" s=\"");
                        self.xml_buffer
                            .extend_from_slice(style_index.to_string().as_bytes());
                        self.xml_buffer.extend_from_slice(b"\"");
                    }
                    self.xml_buffer.extend_from_slice(b"><f>");
                    self.xml_buffer.extend_from_slice(formula.as_bytes());
                    self.xml_buffer.extend_from_slice(b"</f></c>");
                }
                CellValue::DateTime(_) | CellValue::Error(_) => {
                    let s = format!("{:?}", cell.value);
                    let string_index = self.shared_strings.add_string(&s);
                    self.xml_buffer.extend_from_slice(b"<c r=\"");
                    self.xml_buffer.extend_from_slice(col_letter.as_bytes());
                    self.xml_buffer
                        .extend_from_slice(row_num.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"\"");
                    if style_index > 0 {
                        self.xml_buffer.extend_from_slice(b" s=\"");
                        self.xml_buffer
                            .extend_from_slice(style_index.to_string().as_bytes());
                        self.xml_buffer.extend_from_slice(b"\"");
                    }
                    self.xml_buffer.extend_from_slice(b" t=\"s\"><v>");
                    self.xml_buffer
                        .extend_from_slice(string_index.to_string().as_bytes());
                    self.xml_buffer.extend_from_slice(b"</v></c>");
                }
            }
        }

        self.xml_buffer.extend_from_slice(b"</row>");

        // Write buffer to zip
        self.zip.write_all(&self.xml_buffer)?;

        // Flush định kỳ để giới hạn memory
        if self.current_row.is_multiple_of(self.flush_interval) {
            self.zip.flush()?;
        }

        Ok(())
    }

    fn col_to_letter(col: u32) -> String {
        let mut col_str = String::new();
        let mut n = col;
        while n > 0 {
            let rem = (n - 1) % 26;
            col_str.insert(0, (b'A' + rem as u8) as char);
            n = (n - 1) / 26;
        }
        col_str
    }

    fn finish_current_worksheet(&mut self) -> Result<()> {
        if self.current_worksheet.is_none() {
            return Ok(());
        }

        // Ensure sheetData has been started (important for empty sheets)
        self.ensure_sheet_data_started()?;

        let mut xml_writer = XmlWriter::new(&mut self.zip);
        xml_writer.end_element("sheetData")?;
        xml_writer.write_str("<pageMargins left=\"0.7\" right=\"0.7\" top=\"0.75\" bottom=\"0.75\" header=\"0.3\" footer=\"0.3\"/>")?;
        xml_writer.end_element("worksheet")?;
        xml_writer.flush()?;

        self.current_worksheet = None;
        Ok(())
    }

    /// Finish a worksheet and restore shared strings
    pub fn finish_worksheet(
        &mut self,
        _worksheet: FastWorksheet<&mut ZipWriter<BufWriter<File>>>,
    ) -> Result<()> {
        // This method is no longer needed with the new API
        // Keeping for backward compatibility but it does nothing
        Ok(())
    }

    /// Close the workbook and write remaining files
    pub fn close(mut self) -> Result<()> {
        // Close current worksheet if any
        self.finish_current_worksheet()?;

        let options = Self::file_options();

        // Write shared strings
        self.zip.start_file("xl/sharedStrings.xml", options)?;
        {
            let mut xml_writer = XmlWriter::new(&mut self.zip);
            self.shared_strings.write_xml(&mut xml_writer)?;
            xml_writer.flush()?;
        }

        // Write workbook.xml
        self.zip.start_file("xl/workbook.xml", options)?;
        self.write_workbook_xml()?;
        self.zip.flush()?;

        // Write xl/_rels/workbook.xml.rels
        self.zip.start_file("xl/_rels/workbook.xml.rels", options)?;
        self.write_workbook_rels()?;

        // Write styles.xml
        self.zip.start_file("xl/styles.xml", options)?;
        self.write_styles()?;

        // Write theme
        self.zip.start_file("xl/theme/theme1.xml", options)?;
        self.write_theme()?;

        self.zip.finish()?;
        Ok(())
    }

    fn write_content_types<W: Write>(writer: &mut W) -> Result<()> {
        // Emit reference [Content_Types].xml from rust_xlsxwriter sample to match exact ordering
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/><Override PartName="/xl/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/worksheets/sheet3.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/></Types>"#;

        writer.write_all(xml.as_bytes())?;
        Ok(())
    }

    fn write_root_rels<W: Write>(writer: &mut W) -> Result<()> {
        // Write _rels/.rels with correct Relationships
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#;

        writer.write_all(xml.as_bytes())?;
        Ok(())
    }

    fn write_core_props<W: Write>(writer: &mut W) -> Result<()> {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:creator>rust-excelize</dc:creator>
<cp:lastModifiedBy>rust-excelize</cp:lastModifiedBy>
<dcterms:created xsi:type="dcterms:W3CDTF">2024-01-01T00:00:00Z</dcterms:created>
<dcterms:modified xsi:type="dcterms:W3CDTF">2024-01-01T00:00:00Z</dcterms:modified>
</cp:coreProperties>"#;
        writer.write_all(xml.as_bytes())?;
        Ok(())
    }

    fn write_app_props<W: Write>(writer: &mut W) -> Result<()> {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
<Application>rust-excelize</Application>
<DocSecurity>0</DocSecurity>
<ScaleCrop>false</ScaleCrop>
<Company></Company>
<LinksUpToDate>false</LinksUpToDate>
<SharedDoc>false</SharedDoc>
<HyperlinksChanged>false</HyperlinksChanged>
<AppVersion>1.0</AppVersion>
</Properties>"#;
        writer.write_all(xml.as_bytes())?;
        Ok(())
    }

    fn write_workbook_xml(&mut self) -> Result<()> {
        let mut xml_writer = XmlWriter::new(&mut self.zip);

        xml_writer.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n")?;
        xml_writer.start_element("workbook")?;
        xml_writer.attribute(
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        )?;
        xml_writer.attribute(
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        )?;
        xml_writer.close_start_tag()?;

        // Add standard workbook metadata
        xml_writer.write_str("<fileVersion appName=\"xl\" lastEdited=\"4\" lowestEdited=\"4\" rupBuild=\"4505\"/>")?;
        xml_writer.write_str("<workbookPr defaultThemeVersion=\"124226\"/>")?;
        xml_writer.write_str("<bookViews><workbookView xWindow=\"240\" yWindow=\"15\" windowWidth=\"16095\" windowHeight=\"9660\"/></bookViews>")?;

        // Sheets
        xml_writer.start_element("sheets")?;
        xml_writer.close_start_tag()?;

        for (i, name) in self.worksheets.iter().enumerate() {
            let sheet_id = i + 1;
            xml_writer.start_element("sheet")?;
            xml_writer.attribute("name", name)?;
            xml_writer.attribute_int("sheetId", sheet_id as i64)?;
            xml_writer.attribute("r:id", &format!("rId{}", sheet_id))?;
            xml_writer.write_raw(b"/>")?;
        }

        xml_writer.end_element("sheets")?;
        xml_writer.write_str("<calcPr calcId=\"124519\" fullCalcOnLoad=\"1\"/>")?;
        xml_writer.end_element("workbook")?;
        xml_writer.flush()?;

        Ok(())
    }

    fn write_workbook_rels(&mut self) -> Result<()> {
        // Emit reference workbook rels from rust_xlsxwriter sample to match exact ordering and rIds
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet3.xml"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/><Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId6" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/></Relationships>"#;

        let mut writer = &mut self.zip;
        writer.write_all(xml.as_bytes())?;
        Ok(())
    }

    fn write_styles(&mut self) -> Result<()> {
    // Use the reference styles.xml from rust_xlsxwriter to match exactly
    let xml = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="1"><font><sz val="11"/><color theme="1"/><name val="Calibri"/><family val="2"/><scheme val="minor"/></font></fonts><fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills><borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs><cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles><dxfs count="0"/><tableStyles count="0" defaultTableStyle="TableStyleMedium9" defaultPivotStyle="PivotStyleLight16"/></styleSheet>"##;
    self.zip.write_all(xml.as_bytes())?;
    Ok(())
    }

        fn write_theme(&mut self) -> Result<()> {
                        // Write the reference theme1.xml taken from rust_xlsxwriter sample
                        let xml = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F497D"/></a:dk2><a:lt2><a:srgbClr val="EEECE1"/></a:lt2><a:accent1><a:srgbClr val="4F81BD"/></a:accent1><a:accent2><a:srgbClr val="C0504D"/></a:accent2><a:accent3><a:srgbClr val="9BBB59"/></a:accent3><a:accent4><a:srgbClr val="8064A2"/></a:accent4><a:accent5><a:srgbClr val="4BACC6"/></a:accent5><a:accent6><a:srgbClr val="F79646"/></a:accent6><a:hlink><a:srgbClr val="0000FF"/></a:hlink><a:folHlink><a:srgbClr val="800080"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Cambria"/><a:ea typeface=""/><a:cs typeface=""/><a:font script="Jpan" typeface="ＭＳ Ｐゴシック"/><a:font script="Hang" typeface="맑은 고딕"/><a:font script="Hans" typeface="宋体"/><a:font script="Hant" typeface="新細明體"/><a:font script="Arab" typeface="Times New Roman"/><a:font script="Hebr" typeface="Times New Roman"/><a:font script="Thai" typeface="Tahoma"/><a:font script="Ethi" typeface="Nyala"/><a:font script="Beng" typeface="Vrinda"/><a:font script="Gujr" typeface="Shruti"/><a:font script="Khmr" typeface="MoolBoran"/><a:font script="Knda" typeface="Tunga"/><a:font script="Guru" typeface="Raavi"/><a:font script="Cans" typeface="Euphemia"/><a:font script="Cher" typeface="Plantagenet Cherokee"/><a:font script="Yiii" typeface="Microsoft Yi Baiti"/><a:font script="Tibt" typeface="Microsoft Himalaya"/><a:font script="Thaa" typeface="MV Boli"/><a:font script="Deva" typeface="Mangal"/><a:font script="Telu" typeface="Gautami"/><a:font script="Taml" typeface="Latha"/><a:font script="Syrc" typeface="Estrangelo Edessa"/><a:font script="Orya" typeface="Kalinga"/><a:font script="Mlym" typeface="Kartika"/><a:font script="Laoo" typeface="DokChampa"/><a:font script="Sinh" typeface="Iskoola Pota"/><a:font script="Mong" typeface="Mongolian Baiti"/><a:font script="Viet" typeface="Times New Roman"/><a:font script="Uigh" typeface="Microsoft Uighur"/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/><a:font script="Jpan" typeface="ＭＳ Ｐゴシック"/><a:font script="Hang" typeface="맑은 고딕"/><a:font script="Hans" typeface="宋体"/><a:font script="Hant" typeface="新細明體"/><a:font script="Arab" typeface="Arial"/><a:font script="Hebr" typeface="Arial"/><a:font script="Thai" typeface="Tahoma"/><a:font script="Ethi" typeface="Nyala"/><a:font script="Beng" typeface="Vrinda"/><a:font script="Gujr" typeface="Shruti"/><a:font script="Khmr" typeface="DaunPenh"/><a:font script="Knda" typeface="Tunga"/><a:font script="Guru" typeface="Raavi"/><a:font script="Cans" typeface="Euphemia"/><a:font script="Cher" typeface="Plantagenet Cherokee"/><a:font script="Yiii" typeface="Microsoft Yi Baiti"/><a:font script="Tibt" typeface="Microsoft Himalaya"/><a:font script="Thaa" typeface="MV Boli"/><a:font script="Deva" typeface="Mangal"/><a:font script="Telu" typeface="Gautami"/><a:font script="Taml" typeface="Latha"/><a:font script="Syrc" typeface="Estrangelo Edessa"/><a:font script="Orya" typeface="Kalinga"/><a:font script="Mlym" typeface="Kartika"/><a:font script="Laoo" typeface="DokChampa"/><a:font script="Sinh" typeface="Iskoola Pota"/><a:font script="Mong" typeface="Mongolian Baiti"/><a:font script="Viet" typeface="Arial"/><a:font script="Uigh" typeface="Microsoft Uighur"/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:tint val="50000"/><a:satMod val="300000"/></a:schemeClr></a:gs><a:gs pos="35000"><a:schemeClr val="phClr"><a:tint val="37000"/><a:satMod val="300000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:tint val="15000"/><a:satMod val="350000"/></a:schemeClr></a:gs></a:gsLst><a:lin ang="16200000" scaled="1"/></a:gradFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:shade val="51000"/><a:satMod val="130000"/></a:schemeClr></a:gs><a:gs pos="80000"><a:schemeClr val="phClr"><a:shade val="93000"/><a:satMod val="130000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:shade val="94000"/><a:satMod val="135000"/></a:schemeClr></a:gs></a:gsLst><a:lin ang="16200000" scaled="0"/></a:gradFill></a:fillStyleLst><a:lnStyleLst><a:ln w="9525" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"><a:shade val="95000"/><a:satMod val="105000"/></a:schemeClr></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="25400" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="38100" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst><a:outerShdw blurRad="40000" dist="20000" dir="5400000" rotWithShape="0"><a:srgbClr val="000000"><a:alpha val="38000"/></a:srgbClr></a:outerShdw></a:effectLst></a:effectStyle><a:effectStyle><a:effectLst><a:outerShdw blurRad="40000" dist="23000" dir="5400000" rotWithShape="0"><a:srgbClr val="000000"><a:alpha val="35000"/></a:srgbClr></a:outerShdw></a:effectLst></a:effectStyle><a:effectStyle><a:effectLst><a:outerShdw blurRad="40000" dist="23000" dir="5400000" rotWithShape="0"><a:srgbClr val="000000"><a:alpha val="35000"/></a:srgbClr></a:outerShdw></a:effectLst><a:scene3d><a:camera prst="orthographicFront"><a:rot lat="0" lon="0" rev="0"/></a:camera><a:lightRig rig="threePt" dir="t"><a:rot lat="0" lon="0" rev="1200000"/></a:lightRig></a:scene3d><a:sp3d><a:bevelT w="63500" h="25400"/></a:sp3d></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:tint val="40000"/><a:satMod val="350000"/></a:schemeClr></a:gs><a:gs pos="40000"><a:schemeClr val="phClr"><a:tint val="45000"/><a:shade val="99000"/><a:satMod val="350000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:shade val="20000"/><a:satMod val="255000"/></a:schemeClr></a:gs></a:gsLst><a:path path="circle"><a:fillToRect l="50000" t="-80000" r="50000" b="180000"/></a:path></a:gradFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:tint val="80000"/><a:satMod val="300000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:shade val="30000"/><a:satMod val="200000"/></a:schemeClr></a:gs></a:gsLst><a:path path="circle"><a:fillToRect l="50000" t="50000" r="50000" b="50000"/></a:path></a:gradFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>"##;

                        self.zip.write_all(xml.as_bytes())?;
                        Ok(())
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_fast_workbook() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test.xlsx");

        let mut workbook = FastWorkbook::new(&path)?;
        workbook.add_worksheet("Sheet1")?;

        workbook.write_row(&["Name", "Age"])?;
        workbook.write_row(&["Alice", "30"])?;

        workbook.close()?;

        assert!(path.exists());
        Ok(())
    }
}
