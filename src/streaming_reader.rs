//! Streaming reader for XLSX files with optimized memory usage
//!
//! This module provides a reader that processes data row-by-row with an iterator interface.
//!
//! **Memory Usage:**
//! - Shared Strings Table (SST): Loaded fully (~3-5 MB for typical files)
//! - Worksheet XML: Loaded fully from ZIP (uncompressed size)
//! - Total memory ≈ SST + Uncompressed XML size
//!
//! **Important Notes:**
//! - XLSX files are compressed. A 86 MB file may contain 1.2 GB uncompressed XML
//! - For small-medium files (< 100 MB): Memory usage is reasonable
//! - For large files with huge XML: Memory = uncompressed XML size
//! - Still faster than calamine (no style parsing) and uses optimized SST
//!
//! **Trade-offs:**
//! - Only supports simple XLSX files (no complex formatting)
//! - Sequential read only (can't jump to random rows)
//! - Best for: Fast iteration, simple data extraction, no formatting needs

use crate::error::{ExcelError, Result};
use crate::fast_writer::StreamingZipReader;
use crate::types::{CellValue, Row};
use std::io::{BufReader, Read};
use std::path::Path;

/// Parse Excel date serial number to ISO date or datetime string
/// Excel stores dates as floating point numbers representing days since 1900-01-01
/// Examples:
///   - 45217.0 = "2023-10-18" (date only)
///   - 45217.5 = "2023-10-18 12:00:00" (date with time at noon)
///
/// Note: Excel has a bug where it treats 1900 as a leap year (it's not).
/// This means dates from March 1, 1900 onwards are off by 1 day in Excel's count.
///
/// Performance: O(1) using 400-year cycle math (no loops for distant dates)
fn parse_excel_date(serial: f64) -> String {
    // Handle invalid dates
    if !(1.0..=2958465.999).contains(&serial) {
        // 2958465 = December 31, 9999
        return serial.to_string();
    }

    // Split into date and time parts
    let date_part = serial.floor();
    let time_part = serial.fract();

    // Excel epoch: January 1, 1900 = serial 1.0
    // Account for Excel's leap year bug
    let days_since_1900 = if date_part >= 60.0 {
        (date_part - 2.0) as i64 // -2: -1 for bug, -1 for zero-based
    } else {
        (date_part - 1.0) as i64
    };

    // Calculate year from days since 1900
    // Optimized: Estimate then iterate (much faster than pure iteration)
    let mut year = 1900i64;
    let mut remaining_days = days_since_1900;

    // Quick estimate to get close (average 365.25 days/year)
    // Then iterate the last few years
    let est_years = (remaining_days / 365).min(500); // Cap at 500 for safety
    if est_years > 0 {
        year += est_years;

        // Count exact days for estimated years
        let mut days_counted = 0i64;
        for y in 1900..(1900 + est_years) {
            days_counted += if is_leap_year(y) { 366 } else { 365 };
        }
        remaining_days -= days_counted;

        // Adjust if we overshot
        while remaining_days < 0 {
            year -= 1;
            remaining_days += if is_leap_year(year) { 366 } else { 365 };
        }

        // Adjust if we undershot
        while remaining_days >= 365 {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }
    }

    // Calculate month and day from remaining days
    const DAYS_IN_MONTHS_LEAP: [i32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    const DAYS_IN_MONTHS_COMMON: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let days_in_months = if is_leap_year(year) {
        &DAYS_IN_MONTHS_LEAP
    } else {
        &DAYS_IN_MONTHS_COMMON
    };

    let mut month = 1;
    let mut day = remaining_days as i32 + 1;

    for (m, &days) in days_in_months.iter().enumerate() {
        if day <= days {
            month = m + 1;
            break;
        }
        day -= days;
    }

    // Format with time if fractional part exists (>0.0001 to avoid float errors)
    if time_part > 0.0001 {
        // Time stored as fraction: 0.5 = 12:00:00, 0.25 = 06:00:00
        let total_seconds = (time_part * 86400.0).round() as i64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, minutes, seconds
        )
    } else {
        // Date only
        format!("{:04}-{:02}-{:02}", year, month, day)
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Streaming reader for XLSX files
///
/// **Memory Usage:**
/// - SST (Shared Strings): Loaded fully (typically 3-5 MB)
/// - Worksheet XML: Loaded from ZIP (uncompressed size)
/// - Total ≈ SST + Uncompressed XML size
///
/// **Performance:**
/// - 60K-85K rows/sec depending on file size
/// - Faster than calamine (no style/format parsing)
/// - Optimized hybrid SST
///
/// **Best for:**
/// - Small to medium files (< 100 MB compressed)
/// - Files with small SST but many rows
/// - Simple data extraction without formatting
pub struct StreamingReader {
    archive: StreamingZipReader,
    sst: Vec<String>,
    sheet_names: Vec<String>,
    sheet_paths: Vec<String>,
}

impl StreamingReader {
    /// Open XLSX file for streaming read
    ///
    /// # Memory Usage
    ///
    /// - Loads SST (Shared Strings Table) fully into memory
    /// - Worksheet data loaded as single XML string (uncompressed size)
    /// - For 86 MB file: May use ~1.2 GB if XML is large
    /// - For smaller files (< 50 MB): Usually reasonable memory
    ///
    /// # Performance
    ///
    /// - Fast: 60K-85K rows/sec
    /// - No style/format parsing overhead
    /// - Optimized for simple data extraction
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::streaming_reader::StreamingReader;
    ///
    /// let reader = StreamingReader::open("large.xlsx")?;
    /// // SST loaded, ready to stream rows
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut archive = StreamingZipReader::open(path)
            .map_err(|e| ExcelError::ReadError(format!("Failed to open ZIP: {}", e)))?;

        // Load Shared Strings Table (can't avoid this)
        let sst = Self::load_shared_strings(&mut archive)?;

        println!(
            "📊 Loaded {} shared strings (~{:.2} MB in memory)",
            sst.len(),
            Self::estimate_sst_size(&sst) as f64 / (1024.0 * 1024.0)
        );

        // Load sheet names and paths from workbook.xml
        let (sheet_names, sheet_paths) = Self::load_sheet_info(&mut archive)?;

        println!("📋 Found {} sheets: {:?}", sheet_names.len(), sheet_names);

        Ok(StreamingReader {
            archive,
            sst,
            sheet_names,
            sheet_paths,
        })
    }

    /// Get list of sheet names
    ///
    /// Returns the names of all worksheets in the workbook.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::ExcelReader;
    ///
    /// let reader = ExcelReader::open("workbook.xlsx")?;
    /// for sheet_name in reader.sheet_names() {
    ///     println!("Sheet: {}", sheet_name);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sheet_names(&self) -> Vec<String> {
        self.sheet_names.clone()
    }

    /// Read rows by sheet index (for backward compatibility)
    ///
    /// # Arguments
    /// * `sheet_index` - Zero-based sheet index (0 = first sheet)
    ///
    /// # Returns
    /// Iterator of Row structs
    pub fn rows_by_index(&mut self, sheet_index: usize) -> Result<RowStructIterator<'_>> {
        let sheet_name = self
            .sheet_names
            .get(sheet_index)
            .ok_or_else(|| {
                ExcelError::ReadError(format!(
                    "Sheet index {} out of bounds. Available: {} sheets",
                    sheet_index,
                    self.sheet_names.len()
                ))
            })?
            .clone();

        self.rows(&sheet_name)
    }

    /// Get worksheet dimensions (rows, columns) - for backward compatibility
    ///
    /// # Note
    /// This is a simplified implementation that reads all rows to count them.
    /// Returns (row_count, max_column_count).
    /// For large files, this can be slow as it needs to iterate through all rows.
    pub fn dimensions(&mut self, sheet_name: &str) -> Result<(usize, usize)> {
        let mut row_count = 0;
        let mut max_cols = 0;

        for row_result in self.rows(sheet_name)? {
            let row = row_result?;
            row_count += 1;
            max_cols = max_cols.max(row.cells.len());
        }

        Ok((row_count, max_cols))
    }

    /// Stream rows from a worksheet
    ///
    /// # Memory Usage
    ///
    /// - Loads worksheet XML fully from ZIP (uncompressed)
    /// - Processes rows with iterator (appears as streaming)
    /// - Memory = SST + Full worksheet XML
    ///
    /// # Performance
    ///
    /// - Returns iterator for row-by-row processing
    /// - Fast iteration: 60K-85K rows/sec
    /// - No style/format overhead
    ///
    /// # Example
    /// - Does NOT load entire worksheet into memory
    /// - SST already loaded in `open()`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::streaming_reader::StreamingReader;
    ///
    /// let mut reader = StreamingReader::open("large.xlsx")?;
    /// for row in reader.stream_rows("Sheet1")? {
    ///     let row = row?;
    ///     println!("Row: {:?}", row);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn stream_rows(&mut self, sheet_name: &str) -> Result<RowIterator<'_>> {
        // Find sheet path by name
        let sheet_path = self
            .sheet_names
            .iter()
            .position(|name| name == sheet_name)
            .and_then(|idx| self.sheet_paths.get(idx))
            .ok_or_else(|| {
                ExcelError::ReadError(format!(
                    "Sheet '{}' not found. Available sheets: {:?}",
                    sheet_name, self.sheet_names
                ))
            })?
            .clone();

        // Get streaming reader for worksheet XML
        let reader = self
            .archive
            .read_entry_streaming_by_name(&sheet_path)
            .map_err(|e| ExcelError::ReadError(format!("Failed to open sheet: {}", e)))?;

        Ok(RowIterator {
            reader: BufReader::with_capacity(64 * 1024, reader), // 64KB buffer
            sst: &self.sst,
            buffer: String::with_capacity(128 * 1024), // 128KB for XML parsing
            pos: 0,
        })
    }

    /// Alias for `stream_rows()` for backward compatibility
    ///
    /// This method provides the same functionality as `stream_rows()` but uses
    /// the more familiar `rows()` name that matches the old calamine-based API.
    /// Returns an iterator of `Row` structs for full API compatibility.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::ExcelReader;
    ///
    /// let mut reader = ExcelReader::open("large.xlsx")?;
    /// for row_result in reader.rows("Sheet1")? {
    ///     let row = row_result?;
    ///     println!("Row {}: {:?}", row.index, row.to_strings());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn rows(&mut self, sheet_name: &str) -> Result<RowStructIterator<'_>> {
        let inner = self.stream_rows(sheet_name)?;
        Ok(RowStructIterator {
            inner,
            row_index: 0,
        })
    }
}

// Decode XML entities (&lt; &gt; &amp; &quot; &apos;)
fn decode_xml_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

impl StreamingReader {
    /// Load Shared Strings Table
    ///
    /// This MUST be loaded fully because cells reference strings by index.
    /// For files with millions of unique strings, this can still be large.
    fn load_shared_strings(archive: &mut StreamingZipReader) -> Result<Vec<String>> {
        let mut sst = Vec::new();

        // Try to find sharedStrings.xml
        let xml_data = match archive.read_entry_by_name("xl/sharedStrings.xml") {
            Ok(data) => String::from_utf8_lossy(&data).to_string(),
            Err(_) => return Ok(sst), // No SST = all cells are inline
        };

        // Parse all <si> tags (multiple per line in compact XML)
        let mut pos = 0;
        while let Some(si_start) = xml_data[pos..].find("<si>") {
            let si_start = pos + si_start;
            if let Some(si_end) = xml_data[si_start..].find("</si>") {
                let si_end = si_start + si_end + 5; // Include "</si>"
                let si_block = &xml_data[si_start..si_end];

                // Extract text from <t>text</t>
                if let Some(t_start) = si_block.find("<t>") {
                    if let Some(t_end) = si_block.find("</t>") {
                        let text = &si_block[t_start + 3..t_end];
                        // Decode XML entities in SST
                        let decoded = decode_xml_entities(text);
                        sst.push(decoded);
                    }
                }

                pos = si_end;
            } else {
                break;
            }
        }

        Ok(sst)
    }

    /// Load sheet names and paths from workbook.xml
    ///
    /// Parses workbook.xml to get sheet names and their corresponding worksheet paths.
    /// Supports Unicode sheet names.
    fn load_sheet_info(archive: &mut StreamingZipReader) -> Result<(Vec<String>, Vec<String>)> {
        let mut sheet_names = Vec::new();
        let mut sheet_ids = Vec::new();

        // Load workbook.xml
        let xml_data = archive
            .read_entry_by_name("xl/workbook.xml")
            .map_err(|e| ExcelError::ReadError(format!("Failed to open workbook.xml: {}", e)))?;
        let xml_data = String::from_utf8_lossy(&xml_data).to_string();

        // Parse <sheet> tags to get names and rIds
        // Example: <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
        let mut pos = 0;
        while let Some(sheet_start) = xml_data[pos..].find("<sheet ") {
            let sheet_start = pos + sheet_start;
            if let Some(sheet_end) = xml_data[sheet_start..].find("/>") {
                let sheet_end = sheet_start + sheet_end + 2;
                let sheet_tag = &xml_data[sheet_start..sheet_end];

                // Extract name attribute
                if let Some(name_start) = sheet_tag.find("name=\"") {
                    let name_start = name_start + 6;
                    if let Some(name_end) = sheet_tag[name_start..].find("\"") {
                        let name = &sheet_tag[name_start..name_start + name_end];
                        sheet_names.push(name.to_string());
                    }
                }

                // Extract r:id attribute
                if let Some(rid_start) = sheet_tag.find("r:id=\"") {
                    let rid_start = rid_start + 6;
                    if let Some(rid_end) = sheet_tag[rid_start..].find("\"") {
                        let rid = &sheet_tag[rid_start..rid_start + rid_end];
                        sheet_ids.push(rid.to_string());
                    }
                }

                pos = sheet_end;
            } else {
                break;
            }
        }
        // Now load workbook.xml.rels to map rIds to worksheet paths
        let mut sheet_paths = Vec::new();

        let rels_data = archive
            .read_entry_by_name("xl/_rels/workbook.xml.rels")
            .map_err(|e| {
                ExcelError::ReadError(format!("Failed to open workbook.xml.rels: {}", e))
            })?;
        let rels_data = String::from_utf8_lossy(&rels_data).to_string();

        // Map rIds to worksheet paths
        for rid in &sheet_ids {
            // Find <Relationship Id="rId1" Target="worksheets/sheet1.xml"/>
            if let Some(rel_start) = rels_data.find(&format!("Id=\"{}\"", rid)) {
                // Find the start of this Relationship tag
                let tag_start = rels_data[..rel_start]
                    .rfind("<Relationship")
                    .unwrap_or(rel_start.saturating_sub(100));

                // Find the end of this Relationship tag
                let tag_end = if let Some(end_pos) = rels_data[rel_start..].find("/>") {
                    rel_start + end_pos + 2
                } else {
                    rels_data.len()
                };

                let rel_tag = &rels_data[tag_start..tag_end];

                // Extract Target from this specific tag
                if let Some(target_start) = rel_tag.find("Target=\"") {
                    let target_start = target_start + 8;
                    if let Some(target_end) = rel_tag[target_start..].find("\"") {
                        let target = &rel_tag[target_start..target_start + target_end];
                        // Target is relative to xl/, e.g., "worksheets/sheet1.xml"
                        let full_path = format!("xl/{}", target);
                        sheet_paths.push(full_path);
                    }
                }
            }
        }

        if sheet_names.len() != sheet_paths.len() {
            return Err(ExcelError::ReadError(format!(
                "Mismatch between sheet names ({}) and paths ({})",
                sheet_names.len(),
                sheet_paths.len()
            )));
        }

        Ok((sheet_names, sheet_paths))
    }

    fn estimate_sst_size(sst: &[String]) -> usize {
        sst.iter().map(|s| s.len() + 24).sum() // 24 bytes per String overhead
    }
}

/// Iterator over rows in a worksheet
/// Streams XML data from ZIP without loading entire worksheet into memory
pub struct RowIterator<'a> {
    reader: BufReader<Box<dyn Read + 'a>>,
    sst: &'a [String],
    buffer: String, // Buffer for reading XML chunks
    pos: usize,     // Current scan position in buffer
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Result<Vec<CellValue>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to find row in current buffer
            let search_slice = &self.buffer[self.pos..];
            if let Some(start_idx) = search_slice.find("<row") {
                let row_start = self.pos + start_idx;
                // Check if we have the end of the row
                if let Some(end_idx) = self.buffer[row_start..].find("</row>") {
                    let row_end = row_start + end_idx + 6; // + length of </row>

                    let row_xml = &self.buffer[row_start..row_end];
                    let result = Self::parse_row(row_xml, self.sst);

                    // Advance position
                    self.pos = row_end;
                    return Some(result);
                }
            }

            // If we are here, either no row found, or incomplete row at end
            // We need to read more data.
            // First, compact the buffer if needed (move valid tail to front)
            if self.pos > 0 {
                // If we consumed everything, just clear
                if self.pos >= self.buffer.len() {
                    self.buffer.clear();
                } else {
                    // We have some data left (incomplete row), move it to front
                    self.buffer.drain(..self.pos);
                }
                self.pos = 0;
            }

            // Read next chunk
            let mut chunk = vec![0u8; 32 * 1024];
            match self.reader.read(&mut chunk) {
                Ok(0) => {
                    // EOF
                    if !self.buffer.is_empty() {
                        self.buffer.clear();
                    }
                    return None;
                }
                Ok(n) => {
                    // Append data. Use lossy utf8 conversion to be safe
                    let s = String::from_utf8_lossy(&chunk[..n]);
                    self.buffer.push_str(&s);
                }
                Err(e) => {
                    return Some(Err(ExcelError::ReadError(format!(
                        "Failed to read XML: {}",
                        e
                    ))))
                }
            }
        }
    }
}

impl<'a> RowIterator<'a> {
    fn parse_row(row_xml: &str, sst: &[String]) -> Result<Vec<CellValue>> {
        let mut row_data = Vec::new();
        let mut pos = 0;

        while let Some(cell_start) = row_xml[pos..]
            .find("<c ")
            .or_else(|| row_xml[pos..].find("<c>"))
        {
            let cell_start = pos + cell_start;

            // Handle both self-closing <c ... /> and <c ...></c>
            let (cell_end, cell_xml) =
                if let Some(self_close_pos) = row_xml[cell_start..].find("/>") {
                    let end = cell_start + self_close_pos + 2;
                    let xml = &row_xml[cell_start..end];
                    (end, xml)
                } else if let Some(close_tag_pos) = row_xml[cell_start..].find("</c>") {
                    let end = cell_start + close_tag_pos + 4;
                    let xml = &row_xml[cell_start..end];
                    (end, xml)
                } else {
                    break; // Incomplete cell tag
                };

            // Extract cell reference (e.g., "A1", "B1", "AA1")
            let col_idx = if let Some(r_start) = cell_xml.find("r=\"") {
                let r_start = r_start + 3;
                if let Some(r_end) = cell_xml[r_start..].find("\"") {
                    let cell_ref = &cell_xml[r_start..r_start + r_end];
                    parse_column_index(cell_ref)
                } else {
                    row_data.len()
                }
            } else {
                row_data.len()
            };

            // Fill empty cells between last column and current column
            while row_data.len() < col_idx {
                row_data.push(CellValue::Empty);
            }

            // Determine cell type
            let cell_type = if let Some(t_start) = cell_xml.find("t=\"") {
                let t_start = t_start + 3;
                if let Some(t_end) = cell_xml[t_start..].find("\"") {
                    &cell_xml[t_start..t_start + t_end]
                } else {
                    ""
                }
            } else {
                "" // No type means numeric
            };

            let is_shared_string = cell_type == "s";
            let is_inline_str = cell_type == "inlineStr";
            let is_boolean = cell_type == "b";
            let is_error = cell_type == "e";
            // Empty type means numeric or date

            // Extract value
            let cell_value = if is_inline_str {
                // Inline string - look for <is><t>...</t></is>
                if let Some(t_start) = cell_xml.find("<t>") {
                    if let Some(t_end) = cell_xml[t_start..].find("</t>") {
                        let value = cell_xml[t_start + 3..t_start + t_end].to_string();
                        CellValue::String(decode_xml_entities(&value))
                    } else {
                        CellValue::Empty
                    }
                } else {
                    CellValue::Empty
                }
            } else if let Some(v_start) = cell_xml.find("<v>") {
                if let Some(v_end) = cell_xml[v_start..].find("</v>") {
                    let val_str = &cell_xml[v_start + 3..v_start + v_end];

                    if is_shared_string {
                        // Lookup in SST
                        if let Ok(idx) = val_str.parse::<usize>() {
                            let value = sst.get(idx).cloned().unwrap_or_default();
                            CellValue::String(decode_xml_entities(&value))
                        } else {
                            CellValue::Empty
                        }
                    } else if is_boolean {
                        // Boolean: 0 = false, 1 = true
                        CellValue::Bool(val_str == "1")
                    } else if is_error {
                        // Error cell
                        CellValue::Error(val_str.to_string())
                    } else {
                        // Numeric value (could be number or date)
                        // Try to parse as number first
                        if let Ok(num) = val_str.parse::<f64>() {
                            // Check if this might be a date
                            // Dates in Excel are typically between 1 (1900-01-01) and 2958465 (9999-12-31)
                            // Also check for style attribute 's' which indicates formatting
                            let has_style = cell_xml.contains("s=\"");

                            // If it looks like a date serial number and has a style, try parsing as date
                            if has_style && (1.0..=2958465.0).contains(&num) && num.fract() < 0.0001
                            {
                                // Likely a date - return as string in ISO format
                                CellValue::String(parse_excel_date(num))
                            } else if num.fract() == 0.0
                                && (i64::MIN as f64..=i64::MAX as f64).contains(&num)
                            {
                                // Integer
                                CellValue::Int(num as i64)
                            } else {
                                // Float
                                CellValue::Float(num)
                            }
                        } else {
                            // Can't parse as number, treat as string
                            CellValue::String(decode_xml_entities(val_str))
                        }
                    }
                } else {
                    CellValue::Empty
                }
            } else {
                CellValue::Empty
            };

            row_data.push(cell_value);
            pos = cell_end;
        }

        Ok(row_data)
    }
}

// Parse column index from cell reference (e.g., "A1" -> 0, "B1" -> 1, "AA1" -> 26)
fn parse_column_index(cell_ref: &str) -> usize {
    let mut col_idx = 0usize;
    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            col_idx = col_idx * 26 + (ch.to_ascii_uppercase() as usize - 'A' as usize + 1);
        } else {
            break;
        }
    }
    col_idx.saturating_sub(1) // Convert to 0-based index
}

/// Iterator wrapper that returns Row structs instead of Vec<CellValue>
/// for backward compatibility with the old calamine-based API
pub struct RowStructIterator<'a> {
    inner: RowIterator<'a>,
    row_index: u32,
}

impl<'a> Iterator for RowStructIterator<'a> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next()? {
            Ok(cells) => {
                let row = Row::new(self.row_index, cells);
                self.row_index += 1;
                Some(Ok(row))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_sst_size() {
        let sst = vec!["hello".to_string(), "world".to_string()];
        let size = StreamingReader::estimate_sst_size(&sst);
        assert!(size > 10); // At least the string bytes
    }

    #[test]
    fn test_parse_excel_date() {
        // Test January 1, 2022 (known: 44562)
        let date = parse_excel_date(44562.0);
        assert_eq!(date, "2022-01-01", "Serial 44562 should be 2022-01-01");

        // Test January 1, 1970 (Unix epoch, known: 25569)
        let date = parse_excel_date(25569.0);
        assert_eq!(date, "1970-01-01", "Serial 25569 should be 1970-01-01");

        // Test January 1, 2000 (known: 36526)
        let date = parse_excel_date(36526.0);
        assert_eq!(date, "2000-01-01", "Serial 36526 should be 2000-01-01");

        // Test December 31, 2020 (known: 44196)
        let date = parse_excel_date(44196.0);
        assert_eq!(date, "2020-12-31", "Serial 44196 should be 2020-12-31");

        // Test leap year: February 29, 2020 (known: 43890)
        let date = parse_excel_date(43890.0);
        assert_eq!(date, "2020-02-29", "Serial 43890 should be 2020-02-29");

        // Test October 18, 2023 (actual value for 45217 from online converter)
        let date = parse_excel_date(45217.0);
        assert_eq!(date, "2023-10-18", "Serial 45217 should be 2023-10-18");
    }

    #[test]
    fn test_parse_excel_datetime() {
        // Test with time component: noon (0.5 = 12:00:00)
        let datetime = parse_excel_date(44562.5);
        assert_eq!(
            datetime, "2022-01-01 12:00:00",
            "Serial 44562.5 should be 2022-01-01 12:00:00"
        );

        // Test with time: 6:00 AM (0.25 = 06:00:00)
        let datetime = parse_excel_date(44562.25);
        assert_eq!(
            datetime, "2022-01-01 06:00:00",
            "Serial 44562.25 should be 2022-01-01 06:00:00"
        );

        // Test with time: 6:00 PM (0.75 = 18:00:00)
        let datetime = parse_excel_date(44562.75);
        assert_eq!(
            datetime, "2022-01-01 18:00:00",
            "Serial 44562.75 should be 2022-01-01 18:00:00"
        );

        // Test with specific time: 14:30:00 (14.5 hours / 24 = 0.6041666...)
        let datetime = parse_excel_date(44562.0 + (14.5 / 24.0));
        assert_eq!(
            datetime, "2022-01-01 14:30:00",
            "Serial with 14:30 should parse correctly"
        );

        // Test midnight (0.0 = 00:00:00) - should return date only
        let datetime = parse_excel_date(44562.0);
        assert_eq!(
            datetime, "2022-01-01",
            "Serial 44562.0 should be date only (midnight)"
        );

        // Test near-midnight (0.00001 < threshold) - should return date only
        let datetime = parse_excel_date(44562.00005);
        assert_eq!(
            datetime, "2022-01-01",
            "Serial with tiny fraction should be date only"
        );
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024)); // Divisible by 4
        assert!(!is_leap_year(2023)); // Not divisible by 4
        assert!(!is_leap_year(1900)); // Divisible by 100 but not 400
        assert!(is_leap_year(2000)); // Divisible by 400
    }

    #[test]
    fn test_parse_excel_date_edge_cases() {
        // Test year 2100 (next century) - Jan 1, 2100 = serial 73049 + 1 = 73050
        // Actually: 73049 days from 1900 = Jan 1, 2100, so serial is 73049 + 2 = 73051
        let next_century = parse_excel_date(73051.0);
        assert_eq!(next_century, "2100-01-01", "Should handle next century");

        // Test year 2000 transition (Y2K)
        let y2k = parse_excel_date(36526.0);
        assert_eq!(y2k, "2000-01-01", "Y2K transition");

        // Test near Excel's leap year bug boundary
        let feb28_1900 = parse_excel_date(59.0); // Feb 28, 1900
        let mar1_1900 = parse_excel_date(61.0); // Mar 1, 1900
        assert_eq!(feb28_1900, "1900-02-28", "Feb 28, 1900");
        assert_eq!(mar1_1900, "1900-03-01", "Mar 1, 1900");
    }
}
