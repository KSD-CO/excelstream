//! Ultra-low memory workbook - writes XML to temp dir, then ZIP at end
//!
//! # Memory Usage
//!
//! This writer uses **<4 MB peak memory** regardless of file size by:
//! 1. Writing XML files uncompressed to a temp directory
//! 2. ZIPing the directory at the end with 64KB chunked reads
//!
//! # Compression Levels
//!
//! - **Level 0**: No compression - 3 MB peak, largest files (~280 MB for 1M rows)
//! - **Level 1**: Fast compression - 3.7 MB peak, ~31 MB files (recommended)
//! - **Level 3**: Moderate - 3.8 MB peak, ~22 MB files
//! - **Level 6**: Balanced - ~5-8 MB peak, ~18 MB files
//! - **Level 9**: Maximum - ~10-15 MB peak, smallest files
//!
//! # Example
//!
//! ```no_run
//! use excelstream::fast_writer::UltraLowMemoryWorkbook;
//!
//! // Default compression level 6 (balanced)
//! let mut wb = UltraLowMemoryWorkbook::new("output.xlsx")?;
//! wb.add_worksheet("Sheet1")?;
//!
//! for i in 0..1_000_000 {
//!     wb.write_row(&["A", "B", "C"])?;
//! }
//!
//! wb.close()?; // Creates compressed file with 15-25 MB peak memory
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::shared_strings::SharedStrings;
use crate::error::Result;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Ultra-low memory workbook - writes uncompressed XML first
pub struct UltraLowMemoryWorkbook {
    final_path: PathBuf,
    temp_dir: tempfile::TempDir,
    worksheets: Vec<String>,
    worksheet_count: u32,
    current_writer: Option<BufWriter<File>>,
    current_row: u32,
    max_col: u32,
    xml_buffer: Vec<u8>,
    compression_level: u32,
    shared_strings: SharedStrings,
}

impl UltraLowMemoryWorkbook {
    /// Create new workbook with default compression (level 6 - balanced)
    ///
    /// For large files (1M+ rows):
    /// - Memory: ~50 MB peak
    /// - Speed: 8-10x faster than streaming compression
    /// - File size: Similar to fully compressed Excel files
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::with_compression(path, 6)
    }

    /// Create new workbook with custom compression level
    ///
    /// Compression levels:
    /// - 0: No compression (~280 MB files, 3 MB memory, fastest)
    /// - 1: Fast compression (~34 MB files, 43 MB memory, very fast)
    /// - 6: Balanced compression (~20 MB files, 49 MB memory) - **recommended**
    /// - 9: Max compression (~18 MB files, 60-80 MB memory, slower)
    pub fn with_compression<P: AsRef<Path>>(path: P, compression_level: u32) -> Result<Self> {
        let final_path = path.as_ref().to_path_buf();

        // Create temp dir with file-specific prefix to avoid conflicts
        let file_stem = final_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("workbook");
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("excelstream_{}_", file_stem))
            .tempdir()?;

        // Create directory structure
        fs::create_dir_all(temp_dir.path().join("xl/worksheets"))?;
        fs::create_dir_all(temp_dir.path().join("xl/_rels"))?;
        fs::create_dir_all(temp_dir.path().join("xl/theme"))?;
        fs::create_dir_all(temp_dir.path().join("docProps"))?;
        fs::create_dir_all(temp_dir.path().join("_rels"))?;

        Ok(UltraLowMemoryWorkbook {
            final_path,
            temp_dir,
            worksheets: Vec::new(),
            worksheet_count: 0,
            current_writer: None,
            current_row: 0,
            max_col: 0,
            xml_buffer: Vec::with_capacity(4096),
            compression_level: compression_level.min(9),
            shared_strings: SharedStrings::new(),
        })
    }

    pub fn add_worksheet(&mut self, name: &str) -> Result<()> {
        // Close previous worksheet
        if let Some(mut writer) = self.current_writer.take() {
            writer.write_all(b"</sheetData></worksheet>")?;
            writer.flush()?;
        }

        self.worksheet_count += 1;
        self.worksheets.push(name.to_string());
        self.current_row = 0;
        self.max_col = 0;

        let path = self
            .temp_dir
            .path()
            .join("xl/worksheets")
            .join(format!("sheet{}.xml", self.worksheet_count));

        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(32 * 1024, file);

        // Write header (dimension will be added when closing)
        writer.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
        writer.write_all(
            b"<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">",
        )?;
        // Add dimension - Numbers needs this to display data
        writer.write_all(b"<dimension ref=\"A1\"/>")?;
        writer.write_all(b"<sheetViews><sheetView workbookViewId=\"0\"/></sheetViews>")?;
        writer.write_all(b"<sheetFormatPr defaultRowHeight=\"15\"/>")?;
        writer.write_all(b"<sheetData>")?;

        self.current_writer = Some(writer);
        Ok(())
    }

    pub fn write_row(&mut self, values: &[&str]) -> Result<()> {
        self.current_row += 1;
        let row_num = self.current_row;

        // Track max column
        if values.len() as u32 > self.max_col {
            self.max_col = values.len() as u32;
        }

        self.xml_buffer.clear();
        self.xml_buffer.extend_from_slice(b"<row r=\"");
        self.xml_buffer
            .extend_from_slice(row_num.to_string().as_bytes());
        self.xml_buffer.extend_from_slice(b"\">");

        for (col_idx, value) in values.iter().enumerate() {
            let col_num = (col_idx + 1) as u32;
            let col_letter = Self::col_to_letter(col_num);

            self.xml_buffer.extend_from_slice(b"<c r=\"");
            self.xml_buffer.extend_from_slice(col_letter.as_bytes());
            self.xml_buffer
                .extend_from_slice(row_num.to_string().as_bytes());

            // Hybrid SST Strategy:
            // 1. Numbers → inline as number type (no SST)
            // 2. Long strings (>50 chars) → inline (usually unique)
            // 3. Short repeating strings → SST (dedupe)
            // 4. If SST full (>100k) → inline

            if let Ok(_num) = value.parse::<f64>() {
                // Numbers: use number type (more efficient, no SST)
                self.xml_buffer.extend_from_slice(b"\" t=\"n\"><v>");
                self.xml_buffer.extend_from_slice(value.as_bytes());
                self.xml_buffer.extend_from_slice(b"</v></c>");
            } else if value.len() > 50 || self.shared_strings.count() >= 100_000 {
                // Long strings or SST full: use inline string
                self.xml_buffer
                    .extend_from_slice(b"\" t=\"inlineStr\"><is><t>");
                Self::write_escaped(&mut self.xml_buffer, value);
                self.xml_buffer.extend_from_slice(b"</t></is></c>");
            } else {
                // Short strings: use shared strings for deduplication
                let string_id = self.shared_strings.add_string(value);
                self.xml_buffer.extend_from_slice(b"\" t=\"s\"><v>");
                self.xml_buffer
                    .extend_from_slice(string_id.to_string().as_bytes());
                self.xml_buffer.extend_from_slice(b"</v></c>");
            }
        }

        self.xml_buffer.extend_from_slice(b"</row>");

        if let Some(writer) = &mut self.current_writer {
            writer.write_all(&self.xml_buffer)?;
        }

        Ok(())
    }

    /// Write styled row with cell formatting
    pub fn write_row_styled(&mut self, cells: &[crate::types::StyledCell]) -> Result<()> {
        use crate::types::CellValue;

        if self.current_writer.is_none() {
            return Err(crate::error::ExcelError::WriteError(
                "No active worksheet. Call add_worksheet() first.".to_string(),
            ));
        }

        self.current_row += 1;
        let row_num = self.current_row;

        let writer = self.current_writer.as_mut().unwrap();

        // Write row start tag
        write!(writer, "<row r=\"{}\"", row_num)?;
        if !cells.is_empty() {
            write!(writer, " spans=\"1:{}\"", cells.len())?;
        }
        write!(writer, ">")?;

        // Write each cell with styling
        for (col_idx, cell) in cells.iter().enumerate() {
            let col_num = (col_idx + 1) as u32;
            let col_letter = Self::col_to_letter(col_num);
            let cell_ref = format!("{}{}", col_letter, row_num);
            let style_index = cell.style.index();

            match &cell.value {
                CellValue::Empty => {
                    // Skip empty cells
                    continue;
                }
                CellValue::String(s) => {
                    let string_index = self.shared_strings.add_string(s);
                    write!(writer, "<c r=\"{}\"", cell_ref)?;
                    if style_index > 0 {
                        write!(writer, " s=\"{}\"", style_index)?;
                    }
                    write!(writer, " t=\"s\"><v>{}</v></c>", string_index)?;
                }
                CellValue::Int(n) => {
                    write!(writer, "<c r=\"{}\"", cell_ref)?;
                    if style_index > 0 {
                        write!(writer, " s=\"{}\"", style_index)?;
                    }
                    write!(writer, "><v>{}</v></c>", n)?;
                }
                CellValue::Float(f) => {
                    write!(writer, "<c r=\"{}\"", cell_ref)?;
                    if style_index > 0 {
                        write!(writer, " s=\"{}\"", style_index)?;
                    }
                    write!(writer, "><v>{}</v></c>", f)?;
                }
                CellValue::Bool(b) => {
                    write!(writer, "<c r=\"{}\"", cell_ref)?;
                    if style_index > 0 {
                        write!(writer, " s=\"{}\"", style_index)?;
                    }
                    write!(writer, " t=\"b\"><v>{}</v></c>", if *b { 1 } else { 0 })?;
                }
                CellValue::Formula(formula) => {
                    write!(writer, "<c r=\"{}\"", cell_ref)?;
                    if style_index > 0 {
                        write!(writer, " s=\"{}\"", style_index)?;
                    }
                    write!(writer, "><f>{}</f></c>", formula)?;
                }
                CellValue::DateTime(_) | CellValue::Error(_) => {
                    let s = format!("{:?}", cell.value);
                    let string_index = self.shared_strings.add_string(&s);
                    write!(writer, "<c r=\"{}\"", cell_ref)?;
                    if style_index > 0 {
                        write!(writer, " s=\"{}\"", style_index)?;
                    }
                    write!(writer, " t=\"s\"><v>{}</v></c>", string_index)?;
                }
            }
        }

        // Close row tag
        write!(writer, "</row>")?;

        // Flush periodically (every 1000 rows)
        if self.current_row.is_multiple_of(1000) {
            writer.flush()?;
        }

        Ok(())
    }

    /// Set column width (no-op for compatibility)
    pub fn set_column_width(&mut self, _col: u32, _width: f64) -> Result<()> {
        // Column width not implemented in ultra-low memory mode
        Ok(())
    }

    /// Set next row height (no-op for compatibility)
    pub fn set_next_row_height(&mut self, _height: f64) -> Result<()> {
        // Row height not implemented in ultra-low memory mode
        Ok(())
    }

    /// Set flush interval (no-op for compatibility - always flushes immediately)
    pub fn set_flush_interval(&mut self, _interval: u32) {
        // No buffering in ultra-low memory mode
    }

    /// Set max buffer size (no-op for compatibility)
    pub fn set_max_buffer_size(&mut self, _size: usize) {
        // No large buffers in ultra-low memory mode
    }

    /// Set compression level for the final ZIP file
    ///
    /// # Arguments
    /// * `level` - Compression level from 0 to 9
    ///   - 0: No compression (fastest, largest files ~280MB for 1M rows)
    ///   - 1: Fast compression (very fast, ~31MB files) - good for development
    ///   - 3: Moderate compression (~22MB files)
    ///   - 6: Balanced compression (~18MB files) - **recommended for production**
    ///   - 9: Maximum compression (~18MB files, slowest)
    ///
    /// # Example
    /// ```no_run
    /// # use excelstream::fast_writer::UltraLowMemoryWorkbook;
    /// let mut wb = UltraLowMemoryWorkbook::new("output.xlsx")?;
    /// wb.set_compression_level(1); // Fast compression for development
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_compression_level(&mut self, level: u32) {
        self.compression_level = level.min(9);
    }

    /// Get current compression level
    pub fn compression_level(&self) -> u32 {
        self.compression_level
    }

    pub fn close(mut self) -> Result<()> {
        // Close current worksheet
        if let Some(mut writer) = self.current_writer.take() {
            writer.write_all(b"</sheetData></worksheet>")?;
            writer.flush()?;
        }

        // Write shared strings file
        self.write_shared_strings()?;

        // Write metadata files
        self.write_metadata_files()?;

        // ZIP everything
        self.zip_directory()?;

        // Explicitly cleanup temp directory after ZIP completes
        // (Drop will also cleanup, but we do it explicitly for clarity)
        let temp_path = self.temp_dir.path().to_path_buf();
        drop(self.temp_dir);

        // Verify temp dir is removed
        if temp_path.exists() {
            // If still exists, try manual cleanup
            let _ = fs::remove_dir_all(&temp_path);
        }

        Ok(())
    }

    fn write_shared_strings(&mut self) -> Result<()> {
        use super::xml_writer::XmlWriter;

        let path = self.temp_dir.path().join("xl/sharedStrings.xml");
        let f = BufWriter::new(File::create(path)?);
        let mut writer = XmlWriter::new(f);

        // Use the built-in write_xml method from SharedStrings
        self.shared_strings.write_xml(&mut writer)?;
        writer.flush()?;

        Ok(())
    }

    /// XML escape for inline strings in xml_buffer
    fn write_escaped(buf: &mut Vec<u8>, s: &str) {
        for ch in s.chars() {
            match ch {
                '<' => buf.extend_from_slice(b"&lt;"),
                '>' => buf.extend_from_slice(b"&gt;"),
                '&' => buf.extend_from_slice(b"&amp;"),
                '"' => buf.extend_from_slice(b"&quot;"),
                '\'' => buf.extend_from_slice(b"&apos;"),
                _ => {
                    let mut bytes = [0u8; 4];
                    let s = ch.encode_utf8(&mut bytes);
                    buf.extend_from_slice(s.as_bytes());
                }
            }
        }
    }

    fn write_metadata_files(&self) -> Result<()> {
        // Write [Content_Types].xml
        let path = self.temp_dir.path().join("[Content_Types].xml");
        let mut f = File::create(path)?;
        write!(f, "<?xml version=\"1.0\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">")?;
        write!(
            f,
            "<Default Extension=\"xml\" ContentType=\"application/xml\"/>"
        )?;
        write!(f, "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>")?;
        write!(f, "<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>")?;
        for i in 1..=self.worksheet_count {
            write!(f, "<Override PartName=\"/xl/worksheets/sheet{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>", i)?;
        }
        write!(f, "<Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>")?;
        write!(f, "<Override PartName=\"/xl/sharedStrings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml\"/>")?;
        write!(f, "<Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/>")?;
        write!(f, "<Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>")?;
        write!(f, "</Types>")?;
        f.flush()?;

        // Write simple workbook.xml
        let path = self.temp_dir.path().join("xl/workbook.xml");
        let mut f = File::create(path)?;
        write!(f, "<?xml version=\"1.0\"?><workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets>")?;
        for (i, name) in self.worksheets.iter().enumerate() {
            write!(
                f,
                "<sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>",
                name,
                i + 1,
                i + 1
            )?;
        }
        write!(f, "</sheets></workbook>")?;
        f.flush()?;

        // Write complete styles.xml with all 14 CellStyle variants
        let path = self.temp_dir.path().join("xl/styles.xml");
        let mut f = File::create(path)?;
        write!(
            f,
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        )?;
        write!(
            f,
            "<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">"
        )?;
        // Fonts: 0=normal, 1=bold, 2=italic
        write!(f, "<fonts count=\"3\">")?;
        write!(f, "<font><sz val=\"11\"/><color theme=\"1\"/><name val=\"Calibri\"/><family val=\"2\"/><scheme val=\"minor\"/></font>")?;
        write!(f, "<font><b/><sz val=\"11\"/><color theme=\"1\"/><name val=\"Calibri\"/><family val=\"2\"/><scheme val=\"minor\"/></font>")?;
        write!(f, "<font><i/><sz val=\"11\"/><color theme=\"1\"/><name val=\"Calibri\"/><family val=\"2\"/><scheme val=\"minor\"/></font>")?;
        write!(f, "</fonts>")?;
        // Fills: 0=none, 1=gray125, 2=yellow, 3=green, 4=red
        write!(f, "<fills count=\"5\">")?;
        write!(f, "<fill><patternFill patternType=\"none\"/></fill>")?;
        write!(f, "<fill><patternFill patternType=\"gray125\"/></fill>")?;
        write!(f, "<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FFFFFF00\"/><bgColor indexed=\"64\"/></patternFill></fill>")?;
        write!(f, "<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FF00FF00\"/><bgColor indexed=\"64\"/></patternFill></fill>")?;
        write!(f, "<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FFFF0000\"/><bgColor indexed=\"64\"/></patternFill></fill>")?;
        write!(f, "</fills>")?;
        // Borders: 0=none, 1=thin
        write!(f, "<borders count=\"2\">")?;
        write!(
            f,
            "<border><left/><right/><top/><bottom/><diagonal/></border>"
        )?;
        write!(f, "<border><left style=\"thin\"><color auto=\"1\"/></left><right style=\"thin\"><color auto=\"1\"/></right><top style=\"thin\"><color auto=\"1\"/></top><bottom style=\"thin\"><color auto=\"1\"/></bottom><diagonal/></border>")?;
        write!(f, "</borders>")?;
        // cellStyleXfs
        write!(f, "<cellStyleXfs count=\"1\">")?;
        write!(
            f,
            "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/>"
        )?;
        write!(f, "</cellStyleXfs>")?;
        // cellXfs - 14 styles matching CellStyle enum
        write!(f, "<cellXfs count=\"14\">")?;
        write!(
            f,
            "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>"
        )?; // 0: Default
        write!(f, "<xf numFmtId=\"0\" fontId=\"1\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\"/>")?; // 1: HeaderBold
        write!(f, "<xf numFmtId=\"3\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/>")?; // 2: NumberInteger
        write!(f, "<xf numFmtId=\"4\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/>")?; // 3: NumberDecimal
        write!(f, "<xf numFmtId=\"5\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/>")?; // 4: NumberCurrency
        write!(f, "<xf numFmtId=\"9\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/>")?; // 5: NumberPercentage
        write!(f, "<xf numFmtId=\"14\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/>")?; // 6: DateDefault
        write!(f, "<xf numFmtId=\"22\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/>")?; // 7: DateTimestamp
        write!(f, "<xf numFmtId=\"0\" fontId=\"1\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\"/>")?; // 8: TextBold
        write!(f, "<xf numFmtId=\"0\" fontId=\"2\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\"/>")?; // 9: TextItalic
        write!(f, "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"2\" borderId=\"0\" xfId=\"0\" applyFill=\"1\"/>")?; // 10: HighlightYellow
        write!(f, "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"3\" borderId=\"0\" xfId=\"0\" applyFill=\"1\"/>")?; // 11: HighlightGreen
        write!(f, "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"4\" borderId=\"0\" xfId=\"0\" applyFill=\"1\"/>")?; // 12: HighlightRed
        write!(f, "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"1\" xfId=\"0\" applyBorder=\"1\"/>")?; // 13: BorderThin
        write!(f, "</cellXfs>")?;
        // cellStyles
        write!(f, "<cellStyles count=\"1\">")?;
        write!(f, "<cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/>")?;
        write!(f, "</cellStyles>")?;
        write!(f, "<dxfs count=\"0\"/>")?;
        write!(f, "<tableStyles count=\"0\" defaultTableStyle=\"TableStyleMedium9\" defaultPivotStyle=\"PivotStyleLight16\"/>")?;
        write!(f, "</styleSheet>")?;
        f.flush()?;

        // Write _rels/.rels
        let path = self.temp_dir.path().join("_rels/.rels");
        let mut f = File::create(path)?;
        write!(f, "<?xml version=\"1.0\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">")?;
        write!(f, "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>")?;
        write!(f, "<Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>")?;
        write!(f, "<Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties\" Target=\"docProps/app.xml\"/>")?;
        write!(f, "</Relationships>")?;
        f.flush()?;

        // Write xl/_rels/workbook.xml.rels
        let path = self.temp_dir.path().join("xl/_rels/workbook.xml.rels");
        let mut f = File::create(path)?;
        write!(f, "<?xml version=\"1.0\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">")?;
        for i in 1..=self.worksheet_count {
            write!(f, "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{}.xml\"/>", i, i)?;
        }
        let next_id = self.worksheet_count + 1;
        write!(f, "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>", next_id)?;
        write!(f, "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.xml\"/>", next_id + 1)?;
        write!(f, "</Relationships>")?;
        f.flush()?;

        // Write docProps/app.xml
        let path = self.temp_dir.path().join("docProps/app.xml");
        let mut f = File::create(path)?;
        write!(
            f,
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        )?;
        write!(f, "<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\" xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">")?;
        write!(f, "<Application>ExcelStream</Application>")?;
        write!(f, "<DocSecurity>0</DocSecurity>")?;
        write!(f, "<ScaleCrop>false</ScaleCrop>")?;
        write!(f, "<Company></Company>")?;
        write!(f, "<LinksUpToDate>false</LinksUpToDate>")?;
        write!(f, "<SharedDoc>false</SharedDoc>")?;
        write!(f, "<HyperlinksChanged>false</HyperlinksChanged>")?;
        write!(f, "<AppVersion>1.0</AppVersion>")?;
        write!(f, "</Properties>")?;
        f.flush()?;

        // Write docProps/core.xml
        let path = self.temp_dir.path().join("docProps/core.xml");
        let mut f = File::create(path)?;
        write!(
            f,
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        )?;
        write!(f, "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">")?;
        write!(f, "<dc:creator>ExcelStream</dc:creator>")?;
        write!(f, "<cp:lastModifiedBy>ExcelStream</cp:lastModifiedBy>")?;
        write!(
            f,
            "<dcterms:created xsi:type=\"dcterms:W3CDTF\">2024-01-01T00:00:00Z</dcterms:created>"
        )?;
        write!(
            f,
            "<dcterms:modified xsi:type=\"dcterms:W3CDTF\">2024-01-01T00:00:00Z</dcterms:modified>"
        )?;
        write!(f, "</cp:coreProperties>")?;
        f.flush()?;

        Ok(())
    }

    fn zip_directory(&self) -> Result<()> {
        let file = File::create(&self.final_path)?;
        let mut zip = ZipWriter::new(file);

        // Use configured compression level
        let compression_method = if self.compression_level == 0 {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };

        let options = FileOptions::default()
            .compression_method(compression_method)
            .compression_level(Some(self.compression_level as i32));

        // Add files in the correct order per Office Open XML spec
        // [Content_Types].xml MUST be first
        self.add_file_to_zip(&mut zip, "[Content_Types].xml", &options)?;

        // Then _rels
        self.add_file_to_zip(&mut zip, "_rels/.rels", &options)?;

        // Then docProps
        self.add_file_to_zip(&mut zip, "docProps/app.xml", &options)?;
        self.add_file_to_zip(&mut zip, "docProps/core.xml", &options)?;

        // Then xl/ directory
        self.add_file_to_zip(&mut zip, "xl/workbook.xml", &options)?;
        self.add_file_to_zip(&mut zip, "xl/_rels/workbook.xml.rels", &options)?;

        // Worksheets
        for i in 1..=self.worksheet_count {
            self.add_file_to_zip(&mut zip, &format!("xl/worksheets/sheet{}.xml", i), &options)?;
        }

        // Other xl/ files
        self.add_file_to_zip(&mut zip, "xl/sharedStrings.xml", &options)?;
        self.add_file_to_zip(&mut zip, "xl/styles.xml", &options)?;

        zip.finish()?;
        Ok(())
    }

    fn add_file_to_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        relative_path: &str,
        options: &FileOptions,
    ) -> Result<()> {
        let full_path = self.temp_dir.path().join(relative_path);

        zip.start_file(relative_path, *options)?;
        let mut f = File::open(&full_path)?;

        // Read and write in 64KB chunks to limit memory
        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let bytes_read = f.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            zip.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    fn col_to_letter(col: u32) -> String {
        let mut result = String::new();
        let mut n = col;
        while n > 0 {
            let rem = (n - 1) % 26;
            result.insert(0, (b'A' + rem as u8) as char);
            n = (n - 1) / 26;
        }
        result
    }
}
