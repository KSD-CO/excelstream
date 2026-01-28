//! Excel file writing with streaming support
//!
//! **Breaking Change in v0.2.0:** ExcelWriter now uses streaming with constant memory usage.
//! Data is written directly to disk as you call write_row(), not kept in memory.

use crate::error::Result;
use crate::fast_writer::UltraLowMemoryWorkbook;
use crate::types::{CellStyle, CellValue};
use std::path::Path;

/// Excel file writer with streaming capabilities
///
/// **V0.2.0 Breaking Change:** Now uses streaming underneath.
/// Data is written directly to disk with constant memory usage.
///
/// Writes Excel files row by row, streaming data directly to a ZIP file.
/// Memory usage is constant (~80MB) regardless of dataset size.
///
/// # Examples
///
/// ```no_run
/// use excelstream::writer::ExcelWriter;
///
/// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
///
/// // Write millions of rows with constant memory usage
/// for i in 0..1_000_000 {
///     writer.write_row(&["Name", "Age", "Email"]).unwrap();
/// }
///
/// writer.save().unwrap();
/// ```
pub struct ExcelWriter {
    inner: UltraLowMemoryWorkbook,
    current_sheet_name: String,
    current_row: u32,
}

impl ExcelWriter {
    /// Create a new Excel writer with streaming support
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row(&["Name", "Age"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut inner = UltraLowMemoryWorkbook::new(path)?;
        inner.add_worksheet("Sheet1")?;

        Ok(ExcelWriter {
            inner,
            current_sheet_name: "Sheet1".to_string(),
            current_row: 0,
        })
    }

    /// Create a new Excel writer with custom compression level
    ///
    /// # Arguments
    /// * `path` - Output file path
    /// * `compression_level` - Compression level from 0 to 9
    ///   - 0: No compression (fastest, largest files)
    ///   - 1: Fast compression (very fast, ~31MB for 1M rows) - good for development
    ///   - 6: Balanced compression (~18MB for 1M rows) - **recommended for production**
    ///   - 9: Maximum compression (~18MB for 1M rows, slowest)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// // Fast compression for development
    /// let mut writer = ExcelWriter::with_compression("output.xlsx", 1).unwrap();
    /// writer.write_row(&["Name", "Age"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn with_compression<P: AsRef<Path>>(path: P, compression_level: u32) -> Result<Self> {
        let mut inner = UltraLowMemoryWorkbook::with_compression(path, compression_level)?;
        inner.add_worksheet("Sheet1")?;

        Ok(ExcelWriter {
            inner,
            current_sheet_name: "Sheet1".to_string(),
            current_row: 0,
        })
    }

    /// Set compression level for the output file
    ///
    /// # Arguments
    /// * `level` - Compression level from 0 to 9 (see [`with_compression`](#method.with_compression))
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.set_compression_level(1); // Fast compression
    /// writer.write_row(&["Name", "Age"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn set_compression_level(&mut self, level: u32) {
        self.inner.set_compression_level(level);
    }

    /// Get current compression level
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let writer = ExcelWriter::new("output.xlsx").unwrap();
    /// println!("Compression level: {}", writer.compression_level());
    /// ```
    pub fn compression_level(&self) -> u32 {
        self.inner.compression_level()
    }

    /// Write a row of data (streaming to disk)
    ///
    /// Data is written directly to the ZIP file and flushed periodically.
    /// Memory usage remains constant regardless of how many rows you write.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row(&["Alice", "30", "New York"]).unwrap();
    /// writer.write_row(&["Bob", "25", "San Francisco"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_row<I, S>(&mut self, data: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.inner.write_row(data)?;
        self.current_row += 1;
        Ok(())
    }

    /// Write multiple rows at once (batch operation)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    ///
    /// let rows = vec![
    ///     vec!["Alice", "30", "NYC"],
    ///     vec!["Bob", "25", "SF"],
    ///     vec!["Carol", "35", "LA"],
    /// ];
    ///
    /// writer.write_rows_batch(&rows).unwrap();
    /// writer.save().unwrap();
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

    /// Write multiple typed rows at once (batch operation)
    pub fn write_rows_typed_batch(&mut self, rows: &[Vec<CellValue>]) -> Result<()> {
        for row_cells in rows {
            self.write_row_typed(row_cells)?;
        }
        Ok(())
    }

    /// Write a row with typed cell values
    ///
    /// Converts typed values to strings for writing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    /// use excelstream::types::CellValue;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row_typed(&[
    ///     CellValue::String("Alice".to_string()),
    ///     CellValue::Int(30),
    ///     CellValue::Float(1234.56),
    /// ]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        use crate::types::StyledCell;

        // Convert CellValue to StyledCell with default style to preserve types
        let styled_cells: Vec<StyledCell> = cells
            .iter()
            .map(|cell| StyledCell::new(cell.clone(), CellStyle::Default))
            .collect();

        self.inner.write_row_styled(&styled_cells)?;
        self.current_row += 1;
        Ok(())
    }

    /// Write a row with styled cells
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    /// use excelstream::types::{CellValue, CellStyle};
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row_styled(&[
    ///     (CellValue::String("Total".to_string()), CellStyle::HeaderBold),
    ///     (CellValue::Float(1234.56), CellStyle::NumberCurrency),
    ///     (CellValue::Int(95), CellStyle::NumberPercentage),
    /// ]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_row_styled(&mut self, cells: &[(CellValue, CellStyle)]) -> Result<()> {
        use crate::types::StyledCell;

        let styled_cells: Vec<StyledCell> = cells
            .iter()
            .map(|(value, style)| StyledCell::new(value.clone(), *style))
            .collect();

        self.inner.write_row_styled(&styled_cells)?;
        self.current_row += 1;
        Ok(())
    }

    /// Write a row with all cells using the same style
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    /// use excelstream::types::{CellValue, CellStyle};
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row_with_style(&[
    ///     CellValue::Int(100),
    ///     CellValue::Int(200),
    ///     CellValue::Int(300),
    /// ], CellStyle::NumberInteger).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_row_with_style(&mut self, values: &[CellValue], style: CellStyle) -> Result<()> {
        let cells: Vec<_> = values.iter().map(|v| (v.clone(), style)).collect();
        self.write_row_styled(&cells)
    }

    /// Write header row with bold formatting
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_header_bold(&["ID", "Name", "Email"]).unwrap();
    /// writer.write_row(&["1", "Alice", "alice@example.com"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_header_bold<I, S>(&mut self, headers: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        use crate::types::CellStyle;

        let cells: Vec<_> = headers
            .into_iter()
            .map(|h| {
                (
                    CellValue::String(h.as_ref().to_string()),
                    CellStyle::HeaderBold,
                )
            })
            .collect();
        self.write_row_styled(&cells)
    }

    /// Write header row (without bold - for backward compatibility)
    ///
    /// **Note:** For bold headers, use `write_header_bold()` instead.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_header(&["ID", "Name", "Email"]).unwrap();
    /// writer.write_row(&["1", "Alice", "alice@example.com"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn write_header<I, S>(&mut self, headers: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.write_row(headers)
    }

    /// Add a new sheet and switch to it
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row(&["Data on Sheet1"]).unwrap();
    ///
    /// writer.add_sheet("Sheet2").unwrap();
    /// writer.write_row(&["Data on Sheet2"]).unwrap();
    ///
    /// writer.save().unwrap();
    /// ```
    pub fn add_sheet(&mut self, name: &str) -> Result<()> {
        self.inner.add_worksheet(name)?;
        self.current_sheet_name = name.to_string();
        self.current_row = 0;
        Ok(())
    }

    /// Set column width for the current worksheet
    ///
    /// Width is in Excel units (default is 8.43).
    /// One unit is approximately the width of one character in the default font.
    ///
    /// **IMPORTANT:** Must be called BEFORE writing any rows.
    /// Column widths cannot be set after rows have been written.
    ///
    /// # Arguments
    /// * `col` - Column index (0-based: 0=A, 1=B, 2=C, etc.)
    /// * `width` - Column width in Excel units (typically 8-50)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    ///
    /// // Set column widths BEFORE writing rows
    /// writer.set_column_width(0, 20.0).unwrap();  // Column A = 20 units wide
    /// writer.set_column_width(1, 15.0).unwrap();  // Column B = 15 units wide
    /// writer.set_column_width(2, 30.0).unwrap();  // Column C = 30 units wide
    ///
    /// // Now write rows
    /// writer.write_header_bold(&["Name", "Age", "Email"]).unwrap();
    /// writer.write_row(&["Alice", "30", "alice@example.com"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn set_column_width(&mut self, col: u32, width: f64) -> Result<()> {
        self.inner.set_column_width(col, width)
    }

    /// Set height for the next row to be written
    ///
    /// Height is in points (1 point = 1/72 inch).
    /// Default row height is 15 points.
    ///
    /// This setting is consumed by the next write_row call.
    /// To set height for multiple rows, call this before each write_row.
    ///
    /// # Arguments
    /// * `height` - Row height in points (typically 10-50)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    ///
    /// // Set height for header row
    /// writer.set_next_row_height(25.0).unwrap();
    /// writer.write_header_bold(&["Name", "Age", "Email"]).unwrap();
    ///
    /// // Regular row (default height)
    /// writer.write_row(&["Alice", "30", "alice@example.com"]).unwrap();
    ///
    /// // Tall row
    /// writer.set_next_row_height(40.0).unwrap();
    /// writer.write_row(&["Bob", "25", "bob@example.com"]).unwrap();
    ///
    /// writer.save().unwrap();
    /// ```
    pub fn set_next_row_height(&mut self, height: f64) -> Result<()> {
        self.inner.set_next_row_height(height)
    }

    /// Protect the current worksheet with options
    ///
    /// Protects the worksheet from editing. Users can still view and select cells
    /// unless restricted. Call this after `add_sheet()` and before writing data.
    ///
    /// # Arguments
    /// * `options` - Protection settings including password and permissions
    ///
    /// # Example
    /// ```no_run
    /// use excelstream::{ExcelWriter, ProtectionOptions};
    ///
    /// let mut writer = ExcelWriter::new("protected.xlsx").unwrap();
    ///
    /// // Default sheet is already created, protect it
    /// let protection = ProtectionOptions::new()
    ///     .with_password("secret123")
    ///     .allow_select_locked_cells(true)
    ///     .allow_select_unlocked_cells(true);
    ///
    /// writer.protect_sheet(protection).unwrap();
    /// writer.write_row(&["Protected", "Data"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn protect_sheet(&mut self, options: crate::types::ProtectionOptions) -> Result<()> {
        self.inner.protect_sheet(options)
    }

    /// Set flush interval (rows between disk flushes)
    ///
    /// Default is 1000 rows. Lower values use less memory but slower.
    /// Higher values are faster but use more memory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.set_flush_interval(500); // Flush every 500 rows
    /// ```
    pub fn set_flush_interval(&mut self, interval: u32) {
        self.inner.set_flush_interval(interval);
    }

    /// Set maximum buffer size before forcing a flush
    ///
    /// Default is 1MB. This ensures memory usage stays bounded.
    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.inner.set_max_buffer_size(size);
    }

    /// Save and finalize the workbook
    ///
    /// This closes the ZIP file and ensures all data is written to disk.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.write_row(&["Data"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn save(self) -> Result<()> {
        self.inner.close()
    }

    /// Get current row number (0-based)
    pub fn current_row(&self) -> u32 {
        self.current_row
    }
}

/// Builder for creating configured Excel writers
pub struct ExcelWriterBuilder {
    path: String,
    default_sheet_name: Option<String>,
    flush_interval: Option<u32>,
    max_buffer_size: Option<usize>,
}

impl ExcelWriterBuilder {
    /// Create a new builder
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ExcelWriterBuilder {
            path: path.as_ref().to_string_lossy().to_string(),
            default_sheet_name: None,
            flush_interval: None,
            max_buffer_size: None,
        }
    }

    /// Set the default sheet name
    pub fn with_sheet_name(mut self, name: &str) -> Self {
        self.default_sheet_name = Some(name.to_string());
        self
    }

    /// Set flush interval (rows between disk flushes)
    pub fn with_flush_interval(mut self, interval: u32) -> Self {
        self.flush_interval = Some(interval);
        self
    }

    /// Set maximum buffer size
    pub fn with_max_buffer_size(mut self, size: usize) -> Self {
        self.max_buffer_size = Some(size);
        self
    }

    /// Build the writer
    pub fn build(self) -> Result<ExcelWriter> {
        let mut inner = UltraLowMemoryWorkbook::new(&self.path)?;

        let sheet_name = self
            .default_sheet_name
            .unwrap_or_else(|| "Sheet1".to_string());
        inner.add_worksheet(&sheet_name)?;

        let mut writer = ExcelWriter {
            inner,
            current_row: 0,
            current_sheet_name: sheet_name,
        };

        if let Some(interval) = self.flush_interval {
            writer.set_flush_interval(interval);
        }

        if let Some(size) = self.max_buffer_size {
            writer.set_max_buffer_size(size);
        }

        Ok(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_writer_creation() {
        let temp = NamedTempFile::new().unwrap();
        let writer = ExcelWriter::new(temp.path());
        assert!(writer.is_ok());

        // Should be able to save immediately
        let writer = writer.unwrap();
        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_write_row() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        assert!(writer.write_row(["A", "B", "C"]).is_ok());
        assert!(writer.write_row(["1", "2", "3"]).is_ok());
        assert_eq!(writer.current_row(), 2);

        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_write_row_typed() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        use crate::types::CellValue;
        let row = vec![
            CellValue::String("Text".to_string()),
            CellValue::Int(42),
            CellValue::Float(1234.56),
            CellValue::Bool(true),
        ];

        assert!(writer.write_row_typed(&row).is_ok());
        assert_eq!(writer.current_row(), 1);

        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_builder() {
        let temp = NamedTempFile::new().unwrap();
        let writer = ExcelWriterBuilder::new(temp.path())
            .with_sheet_name("CustomSheet")
            .with_flush_interval(500)
            .with_max_buffer_size(512 * 1024)
            .build();

        assert!(writer.is_ok());
        let writer = writer.unwrap();
        assert_eq!(writer.current_sheet_name, "CustomSheet");
        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_add_sheet() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        writer.write_row(["Sheet1 Data"]).unwrap();
        assert_eq!(writer.current_row(), 1);

        writer.add_sheet("Sheet2").unwrap();
        assert_eq!(writer.current_row(), 0);
        assert_eq!(writer.current_sheet_name, "Sheet2");

        writer.write_row(["Sheet2 Data"]).unwrap();
        assert_eq!(writer.current_row(), 1);

        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_write_header() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        writer.write_header(["ID", "Name", "Email"]).unwrap();
        writer
            .write_row(["1", "Alice", "alice@example.com"])
            .unwrap();

        assert_eq!(writer.current_row(), 2);
        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_batch_write() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        let data = vec![
            vec!["A1", "B1", "C1"],
            vec!["A2", "B2", "C2"],
            vec!["A3", "B3", "C3"],
        ];

        writer.write_rows_batch(&data).unwrap();
        assert_eq!(writer.current_row(), 3);

        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_formula_support() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        use crate::types::CellValue;

        // Write header
        writer.write_header(["Value 1", "Value 2", "Sum"]).unwrap();

        // Write data with formula
        writer
            .write_row_typed(&[
                CellValue::Int(10),
                CellValue::Int(20),
                CellValue::Formula("=A2+B2".to_string()),
            ])
            .unwrap();

        writer
            .write_row_typed(&[
                CellValue::Int(15),
                CellValue::Int(25),
                CellValue::Formula("=A3+B3".to_string()),
            ])
            .unwrap();

        // Add a summary row with SUM formula
        writer
            .write_row_typed(&[
                CellValue::String("Total".to_string()),
                CellValue::Empty,
                CellValue::Formula("=SUM(C2:C3)".to_string()),
            ])
            .unwrap();

        assert_eq!(writer.current_row(), 4);
        assert!(writer.save().is_ok());
    }
}
