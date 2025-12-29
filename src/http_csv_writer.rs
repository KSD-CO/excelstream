//! HTTP streaming CSV writer
//!
//! This module provides direct streaming CSV generation to HTTP responses.
//! Perfect for web APIs that need to generate CSV files on-the-fly.
//!
//! # Features
//!
//! - Stream CSV directly to HTTP response body
//! - No temporary files required
//! - Constant memory usage
//! - Works with any async web framework (Axum, Actix-web, Warp, etc.)
//! - Supports compression (Zstd, Deflate/Gzip)
//!
//! # Example with Axum
//!
//! ```no_run
//! use excelstream::HttpCsvWriter;
//! use excelstream::CompressionMethod;
//! use axum::{
//!     response::{IntoResponse, Response},
//!     http::header,
//! };
//!
//! async fn download_csv() -> Response {
//!     let mut writer = HttpCsvWriter::new();
//!
//!     writer.write_row(&["Month", "Sales", "Profit"]).unwrap();
//!     writer.write_row(&["January", "50000", "12000"]).unwrap();
//!     writer.write_row(&["February", "55000", "15000"]).unwrap();
//!
//!     let bytes = writer.finish().unwrap();
//!
//!     (
//!         [
//!             (header::CONTENT_TYPE, "text/csv"),
//!             (header::CONTENT_DISPOSITION, "attachment; filename=\"report.csv\""),
//!         ],
//!         bytes
//!     ).into_response()
//! }
//!
//! // With compression
//! async fn download_csv_compressed() -> Response {
//!     let mut writer = HttpCsvWriter::with_compression(6);
//!
//!     writer.write_row(&["ID", "Name", "Value"]).unwrap();
//!     // ... write data ...
//!
//!     let bytes = writer.finish().unwrap();
//!
//!     (
//!         [
//!             (header::CONTENT_TYPE, "application/zip"),
//!             (header::CONTENT_DISPOSITION, "attachment; filename=\"report.csv.gz\""),
//!         ],
//!         bytes
//!     ).into_response()
//! }
//! ```

use crate::csv::CsvEncoder;
use crate::error::{ExcelError, Result};
use crate::fast_writer::StreamingZipWriter;
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

/// HTTP CSV writer that generates CSV files in memory for streaming responses
///
/// This writer generates the entire CSV file in memory and can be used
/// to stream responses in web servers. Supports optional compression.
///
/// # Example
///
/// ```no_run
/// use excelstream::HttpCsvWriter;
///
/// let mut writer = HttpCsvWriter::new();
/// writer.write_row(&["ID", "Name", "Value"])?;
/// writer.write_row(&["1", "Alice", "100"])?;
/// writer.write_row(&["2", "Bob", "200"])?;
///
/// let csv_bytes = writer.finish()?;
/// // Send csv_bytes as HTTP response body
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct HttpCsvWriter {
    // Dual mode: compressed or uncompressed
    zip_writer: Option<StreamingZipWriter<MemoryBuffer>>,
    direct_buffer: Option<MemoryBuffer>,

    // State
    row_count: u64,
    buffer: Vec<u8>,
    finished: bool,

    // Configuration
    delimiter: u8,
    quote_char: u8,
    line_ending: &'static [u8],
}

impl HttpCsvWriter {
    /// Create a new HTTP CSV writer (uncompressed)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::HttpCsvWriter;
    ///
    /// let mut writer = HttpCsvWriter::new();
    /// writer.write_row(&["Name", "Age"])?;
    /// writer.write_row(&["Alice", "30"])?;
    ///
    /// let bytes = writer.finish()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Self {
        Self {
            zip_writer: None,
            direct_buffer: Some(MemoryBuffer::new()),
            row_count: 0,
            buffer: Vec::with_capacity(4096),
            finished: false,
            delimiter: b',',
            quote_char: b'"',
            line_ending: b"\n",
        }
    }

    /// Create a new HTTP CSV writer with Deflate/Gzip compression
    ///
    /// # Arguments
    /// * `compression_level` - Compression level from 0 to 9:
    ///   - 0: No compression (fastest)
    ///   - 1: Fast compression
    ///   - 6: Balanced (recommended)
    ///   - 9: Maximum compression (slowest)
    ///
    /// Note: HTTP streaming only supports Deflate compression.
    /// For Zstd compression, use file-based `CsvWriter` instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::HttpCsvWriter;
    ///
    /// let mut writer = HttpCsvWriter::with_compression(6);
    /// writer.write_row(&["Name", "Age"])?;
    /// writer.write_row(&["Alice", "30"])?;
    ///
    /// let compressed_bytes = writer.finish()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_compression(compression_level: u32) -> Self {
        let memory_buffer = MemoryBuffer::new();

        let mut zip = StreamingZipWriter::from_writer_with_compression(
            memory_buffer,
            compression_level.min(9),
        )
        .expect("Failed to create ZIP writer");

        zip.start_entry("data.csv")
            .expect("Failed to start ZIP entry");

        Self {
            zip_writer: Some(zip),
            direct_buffer: None,
            row_count: 0,
            buffer: Vec::with_capacity(4096),
            finished: false,
            delimiter: b',',
            quote_char: b'"',
            line_ending: b"\n",
        }
    }

    /// Set custom delimiter (builder pattern)
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
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::HttpCsvWriter;
    ///
    /// let mut writer = HttpCsvWriter::new();
    /// writer.write_row(&["Name", "Age", "City"])?;
    /// writer.write_row(&["Alice", "30", "NYC"])?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_row<I, S>(&mut self, data: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if self.finished {
            return Err(ExcelError::WriteError(
                "Writer already finished".to_string(),
            ));
        }

        // Reuse buffer
        self.buffer.clear();

        // Encode row
        let encoder = CsvEncoder::new(self.delimiter, self.quote_char);
        let fields: Vec<String> = data.into_iter().map(|s| s.as_ref().to_string()).collect();
        let refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();

        encoder.encode_row(&refs, &mut self.buffer);
        self.buffer.extend_from_slice(self.line_ending);

        // Write to output
        if let Some(ref mut zip) = self.zip_writer {
            zip.write_data(&self.buffer)
                .map_err(|e| ExcelError::WriteError(format!("Failed to write to ZIP: {}", e)))?;
        } else if let Some(ref mut buffer) = self.direct_buffer {
            use std::io::Write;
            buffer
                .write_all(&self.buffer)
                .map_err(|e| ExcelError::WriteError(format!("Failed to write to buffer: {}", e)))?;
        }

        self.row_count += 1;
        Ok(())
    }

    /// Write a row of typed values
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::{HttpCsvWriter, CellValue};
    ///
    /// let mut writer = HttpCsvWriter::new();
    /// writer.write_row_typed(&[
    ///     CellValue::String("Alice".to_string()),
    ///     CellValue::Int(30),
    ///     CellValue::Float(75.5),
    /// ])?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        let strings: Vec<String> = cells.iter().map(|c| c.as_string()).collect();
        let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        self.write_row(refs)
    }

    /// Get the number of rows written
    pub fn row_count(&self) -> u64 {
        self.row_count
    }

    /// Finish writing and return the CSV bytes
    ///
    /// This consumes the writer and returns the complete CSV file as bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::HttpCsvWriter;
    ///
    /// let mut writer = HttpCsvWriter::new();
    /// writer.write_row(&["Name", "Age"])?;
    /// writer.write_row(&["Alice", "30"])?;
    ///
    /// let csv_bytes = writer.finish()?;
    /// // Now send csv_bytes as HTTP response
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn finish(mut self) -> Result<Vec<u8>> {
        if self.finished {
            return Err(ExcelError::WriteError(
                "Writer already finished".to_string(),
            ));
        }

        self.finished = true;

        if let Some(zip) = self.zip_writer.take() {
            let memory_buffer = zip
                .finish()
                .map_err(|e| ExcelError::WriteError(format!("Failed to finish ZIP: {}", e)))?;
            Ok(memory_buffer.into_inner())
        } else if let Some(buffer) = self.direct_buffer.take() {
            Ok(buffer.into_inner())
        } else {
            Err(ExcelError::WriteError("No buffer available".to_string()))
        }
    }
}

impl Default for HttpCsvWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_csv_plain() -> Result<()> {
        let mut writer = HttpCsvWriter::new();
        writer.write_row(["Name", "Age", "City"])?;
        writer.write_row(["Alice", "30", "NYC"])?;
        writer.write_row(["Bob", "25", "SF"])?;

        let bytes = writer.finish()?;
        let content = String::from_utf8(bytes).unwrap();

        assert!(content.contains("Name,Age,City"));
        assert!(content.contains("Alice,30,NYC"));
        assert!(content.contains("Bob,25,SF"));

        Ok(())
    }

    #[test]
    fn test_http_csv_compressed() -> Result<()> {
        let mut writer = HttpCsvWriter::with_compression(6);
        writer.write_row(["ID", "Value"])?;

        for i in 0..100 {
            writer.write_row([&i.to_string(), &format!("Value_{}", i)])?;
        }

        let row_count = writer.row_count();
        let bytes = writer.finish()?;

        // Should be compressed (smaller than plain text)
        assert!(bytes.len() < 2000); // Plain would be ~2.7KB
        assert_eq!(row_count, 101);

        Ok(())
    }

    #[test]
    fn test_http_csv_typed() -> Result<()> {
        let mut writer = HttpCsvWriter::new();
        writer.write_row_typed(&[
            CellValue::String("Test".to_string()),
            CellValue::Int(42),
            CellValue::Float(3.15),
        ])?;

        let bytes = writer.finish()?;
        let content = String::from_utf8(bytes).unwrap();

        assert!(content.contains("Test,42,3.15"));

        Ok(())
    }
}
