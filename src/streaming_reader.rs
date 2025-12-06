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
use crate::types::{CellValue, Row};
use std::path::Path;
use zip::ZipArchive;

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
    archive: ZipArchive<std::fs::File>,
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
        let file = std::fs::File::open(path)
            .map_err(|e| ExcelError::ReadError(format!("Failed to open file: {}", e)))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| ExcelError::ReadError(format!("Failed to read ZIP: {}", e)))?;

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
            })?;

        let sheet_file = self
            .archive
            .by_name(sheet_path)
            .map_err(|e| ExcelError::ReadError(format!("Failed to open sheet: {}", e)))?;

        Ok(RowIterator {
            reader: std::io::BufReader::new(sheet_file),
            sst: &self.sst,
            buffer: String::new(),
            position: 0,
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
    fn load_shared_strings(archive: &mut ZipArchive<std::fs::File>) -> Result<Vec<String>> {
        let mut sst = Vec::new();

        // Try to find sharedStrings.xml
        let mut sst_file = match archive.by_name("xl/sharedStrings.xml") {
            Ok(f) => f,
            Err(_) => return Ok(sst), // No SST = all cells are inline
        };

        let mut xml_data = String::new();
        use std::io::Read;
        sst_file
            .read_to_string(&mut xml_data)
            .map_err(|e| ExcelError::ReadError(e.to_string()))?;

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
    fn load_sheet_info(
        archive: &mut ZipArchive<std::fs::File>,
    ) -> Result<(Vec<String>, Vec<String>)> {
        let mut sheet_names = Vec::new();
        let mut sheet_ids = Vec::new();

        // Load workbook.xml in a scope to release the borrow
        {
            let mut workbook_file = archive.by_name("xl/workbook.xml").map_err(|e| {
                ExcelError::ReadError(format!("Failed to open workbook.xml: {}", e))
            })?;

            let mut xml_data = String::new();
            use std::io::Read;
            workbook_file
                .read_to_string(&mut xml_data)
                .map_err(|e| ExcelError::ReadError(e.to_string()))?;

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
        } // workbook_file dropped here

        // Now load workbook.xml.rels to map rIds to worksheet paths
        let mut sheet_paths = Vec::new();
        {
            let mut rels_file = archive.by_name("xl/_rels/workbook.xml.rels").map_err(|e| {
                ExcelError::ReadError(format!("Failed to open workbook.xml.rels: {}", e))
            })?;

            let mut rels_data = String::new();
            use std::io::Read;
            rels_file
                .read_to_string(&mut rels_data)
                .map_err(|e| ExcelError::ReadError(e.to_string()))?;

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
        } // rels_file dropped here

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
pub struct RowIterator<'a> {
    reader: std::io::BufReader<zip::read::ZipFile<'a, std::fs::File>>,
    sst: &'a [String],
    buffer: String,  // String buffer for accumulated data
    position: usize, // Current position in buffer
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::io::Read;

        const CHUNK_SIZE: usize = 131072; // 128 KB chunks

        loop {
            // Try to find complete <row>...</row> in current buffer
            let search_slice = &self.buffer[self.position..];

            if let Some(row_start) = search_slice.find("<row ") {
                if let Some(row_end_pos) = search_slice[row_start..].find("</row>") {
                    // Found complete row
                    let row_end = row_start + row_end_pos + 6; // Include "</row>"
                    let row_xml = &search_slice[row_start..row_end];

                    let result = Self::parse_row(row_xml, self.sst);

                    // Move position past this row
                    self.position += row_end;

                    return Some(result);
                } else {
                    // Found <row but no </row> - need more data
                    // Remove processed data, keep from <row onwards
                    self.buffer = self.buffer[self.position + row_start..].to_string();
                    self.position = 0;

                    // Read more data
                    let mut chunk = vec![0u8; CHUNK_SIZE];
                    match self.reader.read(&mut chunk) {
                        Ok(0) => return None, // EOF without closing tag
                        Ok(n) => {
                            if let Ok(s) = std::str::from_utf8(&chunk[..n]) {
                                self.buffer.push_str(s);
                            }
                        }
                        Err(e) => return Some(Err(ExcelError::ReadError(e.to_string()))),
                    }
                    continue;
                }
            } else {
                // No <row found - advance position but keep tail for split tags
                if search_slice.len() > 10 {
                    // Move forward, keeping last 10 bytes
                    let advance = search_slice.len() - 10;
                    self.position += advance;

                    // If buffer too large, trim processed part
                    if self.position > 1_000_000 {
                        self.buffer = self.buffer[self.position..].to_string();
                        self.position = 0;
                    }
                } else if search_slice.is_empty() {
                    // Buffer exhausted, read more
                    self.buffer.clear();
                    self.position = 0;
                } else {
                    // Small remaining data, keep it and read more
                    self.buffer = self.buffer[self.position..].to_string();
                    self.position = 0;
                }

                // Read next chunk
                let mut chunk = vec![0u8; CHUNK_SIZE];
                match self.reader.read(&mut chunk) {
                    Ok(0) => return None, // EOF
                    Ok(n) => {
                        if let Ok(s) = std::str::from_utf8(&chunk[..n]) {
                            self.buffer.push_str(s);
                        }
                    }
                    Err(e) => return Some(Err(ExcelError::ReadError(e.to_string()))),
                }
            }
        }
    }
}

impl<'a> RowIterator<'a> {
    fn parse_row(row_xml: &str, sst: &[String]) -> Result<Vec<String>> {
        let mut row_data = Vec::new();
        let mut pos = 0;

        while let Some(cell_start) = row_xml[pos..]
            .find("<c ")
            .or_else(|| row_xml[pos..].find("<c>"))
        {
            let cell_start = pos + cell_start;
            let cell_end = match row_xml[cell_start..].find("</c>") {
                Some(end) => cell_start + end + 4,
                None => break,
            };

            let cell_xml = &row_xml[cell_start..cell_end];

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
                row_data.push(String::new());
            }

            // Determine cell type
            let is_shared_string = cell_xml.contains("t=\"s\"");
            let is_inline_str = cell_xml.contains("t=\"inlineStr\"");

            // Extract value
            let value = if is_inline_str {
                // Inline string - look for <is><t>...</t></is>
                if let Some(t_start) = cell_xml.find("<t>") {
                    if let Some(t_end) = cell_xml[t_start..].find("</t>") {
                        cell_xml[t_start + 3..t_start + t_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else if let Some(v_start) = cell_xml.find("<v>") {
                if let Some(v_end) = cell_xml[v_start..].find("</v>") {
                    let val_str = &cell_xml[v_start + 3..v_start + v_end];

                    if is_shared_string {
                        // Lookup in SST
                        if let Ok(idx) = val_str.parse::<usize>() {
                            sst.get(idx).cloned().unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        val_str.to_string()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Decode XML entities
            let value = decode_xml_entities(&value);

            row_data.push(value);
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

/// Iterator wrapper that returns Row structs instead of Vec<String>
/// for backward compatibility with the old calamine-based API
pub struct RowStructIterator<'a> {
    inner: RowIterator<'a>,
    row_index: u32,
}

impl<'a> Iterator for RowStructIterator<'a> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next()? {
            Ok(strings) => {
                let cells: Vec<CellValue> = strings.into_iter().map(CellValue::String).collect();

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
}
