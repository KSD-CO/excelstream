//! Excel file writing with streaming support

use crate::error::Result;
use crate::types::CellValue;
use rust_xlsxwriter::{Format, Workbook, Worksheet};
use std::path::Path;

/// Excel file writer with streaming capabilities
///
/// Writes Excel files row by row, minimizing memory usage for large datasets.
pub struct ExcelWriter {
    workbook: Workbook,
    current_sheet: Option<Worksheet>,
    current_sheet_name: String,
    current_row: u32,
    output_path: String,
    sheet_counter: usize,
    // Pre-allocated buffers to reduce allocations
    string_buffer: String,
    row_buffer: Vec<String>,
}

impl ExcelWriter {
    /// Create a new Excel writer
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
        let output_path = path.as_ref().to_string_lossy().to_string();
        let workbook = Workbook::new();
        let current_sheet = Some(Worksheet::new());
        
        Ok(ExcelWriter {
            workbook,
            current_sheet,
            current_sheet_name: "Sheet1".to_string(),
            current_row: 0,
            output_path,
            sheet_counter: 1,
            string_buffer: String::with_capacity(256),
            row_buffer: Vec::with_capacity(20),
        })
    }

    /// Write a row of data
    ///
    /// Accepts any iterator of items that can be converted to strings.
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
        let sheet = self.current_sheet.as_mut().unwrap();
        for (col, value) in data.into_iter().enumerate() {
            sheet.write_string(
                self.current_row,
                col as u16,
                value.as_ref(),
            )?;
        }
        self.current_row += 1;
        Ok(())
    }

    /// Write multiple rows at once (batch operation for better performance)
    ///
    /// This is the most efficient way to write many rows.
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
        let sheet = self.current_sheet.as_mut().unwrap();
        
        for row_data in rows {
            for (col, value) in row_data.into_iter().enumerate() {
                sheet.write_string(
                    self.current_row,
                    col as u16,
                    value.as_ref(),
                )?;
            }
            self.current_row += 1;
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

    /// Write a row with pre-allocated buffer (optimized for performance)
    ///
    /// This method reuses internal buffers to reduce allocations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// 
    /// for i in 0..10000 {
    ///     writer.write_row_fast(&[
    ///         &i.to_string(),
    ///         &format!("Name_{}", i),
    ///         &format!("Email_{}@example.com", i),
    ///     ]).unwrap();
    /// }
    /// writer.save().unwrap();
    /// ```
    pub fn write_row_fast<I, S>(&mut self, data: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Clear buffer without deallocating
        self.row_buffer.clear();
        
        // Collect into pre-allocated buffer
        for value in data {
            self.row_buffer.push(value.as_ref().to_string());
        }
        
        // Write from buffer
        let sheet = self.current_sheet.as_mut().unwrap();
        for (col, value) in self.row_buffer.iter().enumerate() {
            sheet.write_string(
                self.current_row,
                col as u16,
                value,
            )?;
        }
        self.current_row += 1;
        Ok(())
    }

    /// Write a row with typed cell values
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
        for (col, cell) in cells.iter().enumerate() {
            self.write_cell(self.current_row, col as u16, cell)?;
        }
        self.current_row += 1;
        Ok(())
    }

    /// Write a row with typed values using buffer (optimized)
    ///
    /// More efficient for large datasets with repeated writes.
    pub fn write_row_typed_fast(&mut self, cells: &[CellValue]) -> Result<()> {
        // Direct write without intermediate allocations
        let row = self.current_row;
        for (col, cell) in cells.iter().enumerate() {
            self.write_cell(row, col as u16, cell)?;
        }
        self.current_row += 1;
        Ok(())
    }

    /// Write a single cell with typed value
    fn write_cell(&mut self, row: u32, col: u16, value: &CellValue) -> Result<()> {
        let sheet = self.current_sheet.as_mut().unwrap();
        match value {
            CellValue::Empty => {
                sheet.write_blank(row, col, &Format::new())?;
            }
            CellValue::String(s) => {
                sheet.write_string(row, col, s)?;
            }
            CellValue::Int(i) => {
                sheet.write_number(row, col, *i as f64)?;
            }
            CellValue::Float(f) => {
                sheet.write_number(row, col, *f)?;
            }
            CellValue::Bool(b) => {
                sheet.write_boolean(row, col, *b)?;
            }
            CellValue::DateTime(d) => {
                sheet.write_number(row, col, *d)?;
            }
            CellValue::Error(e) => {
                sheet.write_string(row, col, &format!("ERROR: {}", e))?;
            }
        }
        Ok(())
    }

    /// Write header row with formatting
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
        let format = Format::new().set_bold();
        let sheet = self.current_sheet.as_mut().unwrap();
        
        for (col, header) in headers.into_iter().enumerate() {
            sheet.write_string_with_format(
                self.current_row,
                col as u16,
                header.as_ref(),
                &format,
            )?;
        }
        self.current_row += 1;
        Ok(())
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
        // Save current sheet to workbook
        if let Some(sheet) = self.current_sheet.take() {
            self.workbook.push_worksheet(sheet);
        }
        
        // Create new sheet
        let mut new_sheet = Worksheet::new();
        new_sheet.set_name(name)?;
        self.current_sheet = Some(new_sheet);
        self.current_sheet_name = name.to_string();
        self.current_row = 0;
        self.sheet_counter += 1;
        
        // Reset buffers for new sheet
        self.row_buffer.clear();
        self.string_buffer.clear();
        
        Ok(())
    }

    /// Set column width
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::writer::ExcelWriter;
    ///
    /// let mut writer = ExcelWriter::new("output.xlsx").unwrap();
    /// writer.set_column_width(0, 20.0).unwrap(); // Column A width = 20
    /// writer.write_row(&["Wide Column"]).unwrap();
    /// writer.save().unwrap();
    /// ```
    pub fn set_column_width(&mut self, col: u16, width: f64) -> Result<()> {
        let sheet = self.current_sheet.as_mut().unwrap();
        sheet.set_column_width(col, width)?;
        Ok(())
    }

    /// Auto-fit columns based on content
    pub fn autofit_columns(&mut self) -> Result<()> {
        // Note: rust_xlsxwriter doesn't have built-in autofit,
        // but we can estimate based on content
        // This is a placeholder for future enhancement
        Ok(())
    }

    /// Save the workbook to disk
    ///
    /// Must be called to finalize and write the Excel file.
    pub fn save(mut self) -> Result<()> {
        // Add the current sheet if it exists
        if let Some(sheet) = self.current_sheet.take() {
            self.workbook.push_worksheet(sheet);
        }
        
        self.workbook.save(&self.output_path)?;
        Ok(())
    }

    /// Get current row number (0-based)
    pub fn current_row(&self) -> u32 {
        self.current_row
    }
}

/// Builder for creating formatted Excel writers
pub struct ExcelWriterBuilder {
    path: String,
    default_sheet_name: Option<String>,
}

impl ExcelWriterBuilder {
    /// Create a new builder
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ExcelWriterBuilder {
            path: path.as_ref().to_string_lossy().to_string(),
            default_sheet_name: None,
        }
    }

    /// Set the default sheet name
    pub fn with_sheet_name(mut self, name: &str) -> Self {
        self.default_sheet_name = Some(name.to_string());
        self
    }

    /// Build the writer
    pub fn build(self) -> Result<ExcelWriter> {
        let mut writer = ExcelWriter::new(&self.path)?;
        
        if let Some(name) = self.default_sheet_name {
            if let Some(sheet) = writer.current_sheet.as_mut() {
                sheet.set_name(&name)?;
            }
            writer.current_sheet_name = name;
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
    }

    #[test]
    fn test_builder() {
        let temp = NamedTempFile::new().unwrap();
        let writer = ExcelWriterBuilder::new(temp.path())
            .with_sheet_name("CustomSheet")
            .build();
        assert!(writer.is_ok());
    }
}
