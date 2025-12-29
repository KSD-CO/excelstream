//! CSV file writing with streaming support and compression

use crate::csv::{CompressionMethod, CsvEncoder};
use crate::error::{ExcelError, Result};
use crate::fast_writer::StreamingZipWriter;
use crate::types::CellValue;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// CSV file writer with streaming capabilities and compression support
///
/// Writes CSV files row by row, streaming data directly to disk or compressed ZIP.
/// Memory usage is constant (~5MB or less) regardless of dataset size.
///
/// # Examples
///
/// ```no_run
/// use excelstream::csv_writer::CsvWriter;
///
/// let mut writer = CsvWriter::new("output.csv").unwrap();
/// writer.write_row(&["Name", "Age", "City"]).unwrap();
/// writer.write_row(&["Alice", "30", "NYC"]).unwrap();
/// writer.save().unwrap();
/// ```
///
/// # Compression
///
/// Auto-detects compression from file extension:
/// - `.csv` → Uncompressed
/// - `.csv.zst` or `.csv.zip` → Zstd compression (level 3)
/// - `.csv.gz` → Deflate/Gzip compression (level 6)
///
/// ```no_run
/// use excelstream::csv_writer::CsvWriter;
/// use excelstream::csv::CompressionMethod;
///
/// // Auto-detect from extension
/// let mut writer = CsvWriter::new("data.csv.zst").unwrap();
///
/// // Or explicit compression
/// let mut writer = CsvWriter::with_compression(
///     "data.csv.zst",
///     CompressionMethod::Zstd,
///     3
/// ).unwrap();
/// ```
pub struct CsvWriter {
    // Dual-mode output
    zip_writer: Option<StreamingZipWriter<File>>,
    direct_writer: Option<BufWriter<File>>,

    // State
    row_count: u64,
    buffer: Vec<u8>,

    // Configuration
    delimiter: u8,
    quote_char: u8,
    line_ending: &'static [u8],
}

impl CsvWriter {
    /// Create a new CSV writer - auto-detects compression from file extension
    ///
    /// # File Extensions
    /// - `.csv` → Uncompressed
    /// - `.csv.zst` or `.csv.zip` → Zstd compression (level 3)
    /// - `.csv.gz` → Deflate compression (level 6)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    ///
    /// // Plain CSV
    /// let mut writer = CsvWriter::new("data.csv").unwrap();
    ///
    /// // Zstd compressed
    /// let mut writer = CsvWriter::new("data.csv.zst").unwrap();
    ///
    /// // Gzip compressed
    /// let mut writer = CsvWriter::new("data.csv.gz").unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_str().unwrap_or("");

        if path_str.ends_with(".csv.zst") || path_str.ends_with(".csv.zip") {
            Self::with_compression(path_ref, CompressionMethod::Zstd, 3)
        } else if path_str.ends_with(".csv.gz") {
            Self::with_compression(path_ref, CompressionMethod::Deflate, 6)
        } else {
            // Plain CSV - direct file write
            let file = File::create(path_ref)
                .map_err(|e| ExcelError::WriteError(format!("Failed to create CSV file: {}", e)))?;

            Ok(CsvWriter {
                zip_writer: None,
                direct_writer: Some(BufWriter::new(file)),
                row_count: 0,
                buffer: Vec::with_capacity(4096),
                delimiter: b',',
                quote_char: b'"',
                line_ending: b"\n",
            })
        }
    }

    /// Create a writer with explicit compression method and level
    ///
    /// # Arguments
    /// * `path` - Output file path
    /// * `method` - Compression method (Zstd or Deflate)
    /// * `level` - Compression level:
    ///   - Zstd: 1-21 (recommend 3 for balanced)
    ///   - Deflate: 0-9 (recommend 6 for balanced)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    /// use excelstream::csv::CompressionMethod;
    ///
    /// // Maximum Zstd compression
    /// let mut writer = CsvWriter::with_compression(
    ///     "data.csv.zst",
    ///     CompressionMethod::Zstd,
    ///     9
    /// ).unwrap();
    /// ```
    pub fn with_compression<P: AsRef<Path>>(
        path: P,
        method: CompressionMethod,
        level: u32,
    ) -> Result<Self> {
        let path_ref = path.as_ref();

        // Create ZIP with single CSV entry
        let mut zip = StreamingZipWriter::with_method(path_ref, method, level)
            .map_err(|e| ExcelError::WriteError(format!("Failed to create ZIP writer: {}", e)))?;

        // Entry name: extract from path or use "data.csv"
        let entry_name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| {
                // Remove .zip/.zst/.gz extension if present
                let clean = s
                    .trim_end_matches(".csv")
                    .trim_end_matches(".zst")
                    .trim_end_matches(".gz");
                format!("{}.csv", clean)
            })
            .unwrap_or_else(|| "data.csv".to_string());

        zip.start_entry(&entry_name)
            .map_err(|e| ExcelError::WriteError(format!("Failed to start ZIP entry: {}", e)))?;

        Ok(CsvWriter {
            zip_writer: Some(zip),
            direct_writer: None,
            row_count: 0,
            buffer: Vec::with_capacity(4096),
            delimiter: b',',
            quote_char: b'"',
            line_ending: b"\n",
        })
    }

    /// Set custom delimiter (builder pattern)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    ///
    /// let mut writer = CsvWriter::new("data.csv")
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

    /// Write a row of strings
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    ///
    /// let mut writer = CsvWriter::new("data.csv").unwrap();
    /// writer.write_row(&["Name", "Age", "City"]).unwrap();
    /// writer.write_row(&["Alice", "30", "NYC"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_row<I, S>(&mut self, data: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Reuse buffer
        self.buffer.clear();

        // Encode row using CSV encoder
        let encoder = CsvEncoder::new(self.delimiter, self.quote_char);
        let fields: Vec<String> = data.into_iter().map(|s| s.as_ref().to_string()).collect();
        let refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();

        encoder.encode_row(&refs, &mut self.buffer);
        self.buffer.extend_from_slice(self.line_ending);

        // Write to output
        if let Some(ref mut zip) = self.zip_writer {
            zip.write_data(&self.buffer)
                .map_err(|e| ExcelError::WriteError(format!("Failed to write to ZIP: {}", e)))?;
        } else if let Some(ref mut writer) = self.direct_writer {
            writer
                .write_all(&self.buffer)
                .map_err(|e| ExcelError::WriteError(format!("Failed to write to file: {}", e)))?;
        }

        self.row_count += 1;
        Ok(())
    }

    /// Write a row of typed values
    ///
    /// Converts CellValue types to strings before writing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    /// use excelstream::types::CellValue;
    ///
    /// let mut writer = CsvWriter::new("data.csv").unwrap();
    /// writer.write_row_typed(&[
    ///     CellValue::String("Alice".to_string()),
    ///     CellValue::Int(30),
    ///     CellValue::Float(75.5),
    /// ]).unwrap();
    /// ```
    pub fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        let strings: Vec<String> = cells.iter().map(|c| c.as_string()).collect();
        let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        self.write_row(refs)
    }

    /// Write multiple rows at once
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    ///
    /// let mut writer = CsvWriter::new("data.csv").unwrap();
    /// let rows = vec![
    ///     vec!["Alice", "30"],
    ///     vec!["Bob", "25"],
    /// ];
    /// writer.write_rows_batch(rows).unwrap();
    /// ```
    pub fn write_rows_batch<I, R, S>(&mut self, rows: I) -> Result<()>
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for row_data in rows {
            self.write_row(row_data)?;
        }
        Ok(())
    }

    /// Get the number of rows written
    pub fn row_count(&self) -> u64 {
        self.row_count
    }

    /// Finalize and save the CSV file
    ///
    /// This must be called to properly close the file.
    /// Consumes the writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::csv_writer::CsvWriter;
    ///
    /// let mut writer = CsvWriter::new("data.csv").unwrap();
    /// writer.write_row(&["Name", "Age"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn save(mut self) -> Result<()> {
        if let Some(zip) = self.zip_writer.take() {
            zip.finish()
                .map_err(|e| ExcelError::WriteError(format!("Failed to finish ZIP: {}", e)))?;
        } else if let Some(mut writer) = self.direct_writer.take() {
            writer
                .flush()
                .map_err(|e| ExcelError::WriteError(format!("Failed to flush file: {}", e)))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_plain_csv() -> Result<()> {
        let path = "test_output.csv";
        {
            let mut writer = CsvWriter::new(path)?;
            writer.write_row(["Name", "Age", "City"])?;
            writer.write_row(["Alice", "30", "NYC"])?;
            writer.save()?;
        }

        // Read and verify
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        assert!(content.contains("Name,Age,City"));
        assert!(content.contains("Alice,30,NYC"));

        // Cleanup
        std::fs::remove_file(path).ok();
        Ok(())
    }

    #[test]
    fn test_typed_values() -> Result<()> {
        let path = "test_typed.csv";
        {
            let mut writer = CsvWriter::new(path)?;
            writer.write_row_typed(&[
                CellValue::String("Test".to_string()),
                CellValue::Int(42),
                CellValue::Float(3.15),
            ])?;
            writer.save()?;
        }

        // Read and verify
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        assert!(content.contains("Test,42,3.15"));

        // Cleanup
        std::fs::remove_file(path).ok();
        Ok(())
    }

    #[test]
    fn test_edge_cases() -> Result<()> {
        let path = "test_edge.csv";
        {
            let mut writer = CsvWriter::new(path)?;
            writer.write_row(["a,b", r#"Say "Hi""#, "Line1\nLine2"])?;
            writer.save()?;
        }

        // Read and verify
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        assert!(content.contains(r#""a,b""#));
        assert!(content.contains(r#""Say ""Hi""""#));

        // Cleanup
        std::fs::remove_file(path).ok();
        Ok(())
    }
}
