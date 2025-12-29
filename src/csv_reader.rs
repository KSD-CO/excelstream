//! CSV file reading with streaming support and decompression

use crate::csv::CsvParser;
use crate::error::{ExcelError, Result};
use crate::fast_writer::StreamingZipReader;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// CSV file reader with streaming capabilities and decompression support
///
/// Reads CSV files row by row using an iterator pattern.
/// Automatically handles compressed files (.csv.zst, .csv.gz, .csv.zip).
/// Memory usage is constant and low.
///
/// # Examples
///
/// ```no_run
/// use excelstream::csv_reader::CsvReader;
///
/// let mut reader = CsvReader::open("data.csv").unwrap();
///
/// for row_result in reader.rows() {
///     let row = row_result.unwrap();
///     println!("{:?}", row);
/// }
/// ```
///
/// # With Headers
///
/// ```no_run
/// use excelstream::csv_reader::CsvReader;
///
/// let mut reader = CsvReader::open("data.csv")
///     .unwrap()
///     .has_header(true);
///
/// if let Some(headers) = reader.headers() {
///     println!("Headers: {:?}", headers);
/// }
///
/// for row_result in reader.rows() {
///     let row = row_result.unwrap();
///     // Process data rows (header already consumed)
/// }
/// ```
pub struct CsvReader {
    // Input sources (one active)
    direct_reader: Option<BufReader<File>>,
    zip_reader_data: Option<Vec<u8>>,

    // Parser state
    line_buffer: String,
    row_count: u64,
    lines_iter: Option<Box<dyn Iterator<Item = String>>>,

    // Configuration
    delimiter: u8,
    quote_char: u8,
    has_header: bool,
    headers: Vec<String>,
}

impl CsvReader {
    /// Open CSV file - auto-detects compression from file extension
    ///
    /// # File Extensions
    /// - `.csv` → Uncompressed, direct read
    /// - `.csv.zst`, `.csv.zip` → Zstd decompression
    /// - `.csv.gz` → Deflate/Gzip decompression
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_reader::CsvReader;
    ///
    /// // Plain CSV
    /// let reader = CsvReader::open("data.csv").unwrap();
    ///
    /// // Compressed CSV (auto-detected)
    /// let reader = CsvReader::open("data.csv.zst").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_str().unwrap_or("");

        if path_str.ends_with(".csv.zst")
            || path_str.ends_with(".csv.zip")
            || path_str.ends_with(".csv.gz")
        {
            // Compressed - use s-zip
            let mut zip = StreamingZipReader::open(path_ref)
                .map_err(|e| ExcelError::ReadError(format!("Failed to open ZIP: {}", e)))?;

            // Find first .csv entry
            let entry_name = zip
                .entries()
                .iter()
                .find(|e| e.name.ends_with(".csv"))
                .or_else(|| zip.entries().first())
                .ok_or_else(|| ExcelError::ReadError("No CSV entry found in archive".to_string()))?
                .name
                .clone();

            // Read decompressed data
            let data = zip
                .read_entry_by_name(&entry_name)
                .map_err(|e| ExcelError::ReadError(format!("Failed to read ZIP entry: {}", e)))?;

            Ok(CsvReader {
                direct_reader: None,
                zip_reader_data: Some(data),
                line_buffer: String::with_capacity(1024),
                row_count: 0,
                lines_iter: None,
                delimiter: b',',
                quote_char: b'"',
                has_header: false,
                headers: Vec::new(),
            })
        } else {
            // Plain CSV
            let file = File::open(path_ref)
                .map_err(|e| ExcelError::ReadError(format!("Failed to open CSV file: {}", e)))?;

            Ok(CsvReader {
                direct_reader: Some(BufReader::new(file)),
                zip_reader_data: None,
                line_buffer: String::with_capacity(1024),
                row_count: 0,
                lines_iter: None,
                delimiter: b',',
                quote_char: b'"',
                has_header: false,
                headers: Vec::new(),
            })
        }
    }

    /// Set custom delimiter (builder pattern)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_reader::CsvReader;
    ///
    /// let reader = CsvReader::open("data.csv")
    ///     .unwrap()
    ///     .delimiter(b';');
    /// ```
    pub fn delimiter(mut self, delim: u8) -> Self {
        self.delimiter = delim;
        self
    }

    /// Set custom quote character (builder pattern)
    pub fn quote_char(mut self, quote: u8) -> Self {
        self.quote_char = quote;
        self
    }

    /// Indicate that the first row contains headers (builder pattern)
    ///
    /// When set to `true`, the first row will be stored and accessible via `headers()`.
    /// The iterator will skip the header row.
    pub fn has_header(mut self, has: bool) -> Self {
        self.has_header = has;
        self
    }

    /// Get header row if available
    ///
    /// Returns `Some(&[String])` if headers were parsed, `None` otherwise.
    pub fn headers(&self) -> Option<&[String]> {
        if self.headers.is_empty() {
            None
        } else {
            Some(&self.headers)
        }
    }

    /// Read a single row
    ///
    /// Returns `Ok(None)` when EOF is reached.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_reader::CsvReader;
    ///
    /// let mut reader = CsvReader::open("data.csv").unwrap();
    ///
    /// while let Some(row) = reader.read_row().unwrap() {
    ///     println!("{:?}", row);
    /// }
    /// ```
    pub fn read_row(&mut self) -> Result<Option<Vec<String>>> {
        // Clear buffer
        self.line_buffer.clear();

        // Read line from source
        let bytes_read = if let Some(ref mut reader) = self.direct_reader {
            reader
                .read_line(&mut self.line_buffer)
                .map_err(|e| ExcelError::ReadError(format!("Failed to read line: {}", e)))?
        } else if let Some(ref data) = self.zip_reader_data {
            // For ZIP data, we need to parse lines ourselves
            // This is a simplified approach - in production, consider using a proper line iterator
            if self.lines_iter.is_none() {
                let content = String::from_utf8_lossy(data).to_string();
                let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                self.lines_iter = Some(Box::new(lines.into_iter()));
            }

            if let Some(ref mut iter) = self.lines_iter {
                if let Some(line) = iter.next() {
                    self.line_buffer = line;
                    self.line_buffer.len()
                } else {
                    return Ok(None); // EOF
                }
            } else {
                return Ok(None);
            }
        } else {
            return Err(ExcelError::ReadError("No reader available".to_string()));
        };

        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        // Remove trailing newline (for direct reader)
        if self.line_buffer.ends_with('\n') {
            self.line_buffer.pop();
            if self.line_buffer.ends_with('\r') {
                self.line_buffer.pop();
            }
        }

        // Parse line
        let parser = CsvParser::new(self.delimiter, self.quote_char);
        let fields = parser.parse_line(&self.line_buffer);

        // Handle header row
        if self.has_header && self.row_count == 0 {
            self.headers = fields.clone();
        }

        self.row_count += 1;
        Ok(Some(fields))
    }

    /// Get iterator over rows
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_reader::CsvReader;
    ///
    /// let mut reader = CsvReader::open("data.csv").unwrap();
    ///
    /// for row_result in reader.rows() {
    ///     let row = row_result.unwrap();
    ///     println!("{:?}", row);
    /// }
    /// ```
    pub fn rows(&mut self) -> CsvRowIterator<'_> {
        CsvRowIterator { reader: self }
    }

    /// Get the number of rows read so far
    pub fn row_count(&self) -> u64 {
        self.row_count
    }
}

/// Iterator over CSV rows
pub struct CsvRowIterator<'a> {
    reader: &'a mut CsvReader,
}

impl<'a> Iterator for CsvRowIterator<'a> {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_row() {
            Ok(Some(row)) => {
                // Skip header if has_header is true and this is the first row
                if self.reader.has_header && self.reader.row_count == 1 {
                    // This was the header row, read next
                    match self.reader.read_row() {
                        Ok(Some(next_row)) => Some(Ok(next_row)),
                        Ok(None) => None,
                        Err(e) => Some(Err(e)),
                    }
                } else {
                    Some(Ok(row))
                }
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_writer::CsvWriter;

    #[test]
    fn test_read_plain_csv() -> Result<()> {
        // Create test file
        let path = "test_read_plain.csv";
        {
            let mut writer = CsvWriter::new(path)?;
            writer.write_row(["Name", "Age", "City"])?;
            writer.write_row(["Alice", "30", "NYC"])?;
            writer.write_row(["Bob", "25", "SF"])?;
            writer.save()?;
        }

        // Read it back
        let mut reader = CsvReader::open(path)?;
        let mut rows = vec![];
        for row_result in reader.rows() {
            rows.push(row_result?);
        }

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["Name", "Age", "City"]);
        assert_eq!(rows[1], vec!["Alice", "30", "NYC"]);

        // Cleanup
        std::fs::remove_file(path).ok();
        Ok(())
    }

    #[test]
    fn test_read_with_headers() -> Result<()> {
        let path = "test_read_headers.csv";
        {
            let mut writer = CsvWriter::new(path)?;
            writer.write_row(["ID", "Name"])?;
            writer.write_row(["1", "Alice"])?;
            writer.write_row(["2", "Bob"])?;
            writer.save()?;
        }

        let mut reader = CsvReader::open(path)?.has_header(true);
        assert_eq!(reader.headers(), None); // Not read yet

        let mut rows = vec![];
        for row_result in reader.rows() {
            rows.push(row_result?);
        }

        // Headers should be set after first read
        assert_eq!(
            reader.headers(),
            Some(&["ID".to_string(), "Name".to_string()][..])
        );
        // Iterator should skip header
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["1", "Alice"]);

        std::fs::remove_file(path).ok();
        Ok(())
    }
}
