//! Excel file writing with TRUE streaming support
//!
//! **Breaking Change in v0.2.0:** ExcelWriter now uses true streaming with constant memory usage.
//! Data is written directly to disk as you call write_row(), not kept in memory.

use crate::error::Result;
use crate::types::CellValue;
use crate::fast_writer::FastWorkbook;
use std::path::Path;

/// Excel file writer with TRUE streaming capabilities
///
/// **V0.2.0 Breaking Change:** Now uses true streaming underneath. 
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
    inner: FastWorkbook,
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
        let mut inner = FastWorkbook::new(path)?;
        inner.add_worksheet("Sheet1")?;
        
        Ok(ExcelWriter {
            inner,
            current_sheet_name: "Sheet1".to_string(),
            current_row: 0,
        })
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
        let values: Vec<String> = data.into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        let refs: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
        self.inner.write_row(&refs)?;
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
        let values: Vec<String> = cells.iter()
            .map(|cell| match cell {
                CellValue::Empty => String::new(),
                CellValue::String(s) => s.clone(),
                CellValue::Int(i) => i.to_string(),
                CellValue::Float(f) => f.to_string(),
                CellValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                CellValue::DateTime(d) => d.to_string(),
                CellValue::Error(e) => format!("ERROR: {}", e),
            })
            .collect();
        let refs: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
        self.inner.write_row(&refs)?;
        self.current_row += 1;
        Ok(())
    }

    /// Write header row
    ///
    /// Note: In v0.2.0, bold formatting is not yet supported in streaming mode.
    /// This writes a regular row. Formatting support will be added in future versions.
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
        // TODO: Add bold formatting support in FastWorkbook
        // For now, just write as regular row
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

    /// Set column width
    ///
    /// Note: Column width customization is not yet supported in streaming mode.
    /// This is a no-op for compatibility. Will be added in future versions.
    pub fn set_column_width(&mut self, _col: u16, _width: f64) -> Result<()> {
        // TODO: Add column width support in FastWorkbook
        Ok(())
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
        let mut inner = FastWorkbook::new(&self.path)?;
        
        let sheet_name = self.default_sheet_name.unwrap_or_else(|| "Sheet1".to_string());
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
        
        assert!(writer.write_row(&["A", "B", "C"]).is_ok());
        assert!(writer.write_row(&["1", "2", "3"]).is_ok());
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
            CellValue::Float(3.14),
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
        
        writer.write_row(&["Sheet1 Data"]).unwrap();
        assert_eq!(writer.current_row(), 1);
        
        writer.add_sheet("Sheet2").unwrap();
        assert_eq!(writer.current_row(), 0);
        assert_eq!(writer.current_sheet_name, "Sheet2");
        
        writer.write_row(&["Sheet2 Data"]).unwrap();
        assert_eq!(writer.current_row(), 1);
        
        assert!(writer.save().is_ok());
    }

    #[test]
    fn test_write_header() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = ExcelWriter::new(temp.path()).unwrap();
        
        writer.write_header(&["ID", "Name", "Email"]).unwrap();
        writer.write_row(&["1", "Alice", "alice@example.com"]).unwrap();
        
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
}
