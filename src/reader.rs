//! Excel file reading with streaming support

use crate::error::{ExcelError, Result};
use crate::types::{CellValue, Row};
use calamine::{open_workbook_auto, Data, Range, Reader, Sheets};
use std::path::Path;

/// Excel file reader with streaming capabilities
pub struct ExcelReader {
    workbook: Sheets<std::io::BufReader<std::fs::File>>,
}

impl ExcelReader {
    /// Open an Excel file for reading
    ///
    /// Supports XLSX, XLS, and ODS formats. Format is auto-detected from file extension.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::reader::ExcelReader;
    ///
    /// let reader = ExcelReader::open("data.xlsx").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let workbook =
            open_workbook_auto(path).map_err(|e| ExcelError::ReadError(e.to_string()))?;

        Ok(ExcelReader { workbook })
    }

    /// Get list of sheet names in the workbook
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::reader::ExcelReader;
    ///
    /// let reader = ExcelReader::open("data.xlsx").unwrap();
    /// let sheets = reader.sheet_names();
    /// println!("Available sheets: {:?}", sheets);
    /// ```
    pub fn sheet_names(&self) -> Vec<String> {
        self.workbook.sheet_names().to_vec()
    }

    /// Get the number of sheets in the workbook
    pub fn sheet_count(&self) -> usize {
        self.workbook.sheet_names().len()
    }

    /// Read all rows from a specific sheet (streaming iterator)
    ///
    /// Returns an iterator that yields rows one at a time, minimizing memory usage.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::reader::ExcelReader;
    ///
    /// let mut reader = ExcelReader::open("data.xlsx").unwrap();
    /// for row_result in reader.rows("Sheet1").unwrap() {
    ///     let row = row_result.unwrap();
    ///     println!("Row {}: {:?}", row.index, row.cells);
    /// }
    /// ```
    pub fn rows(&mut self, sheet_name: &str) -> Result<RowIterator> {
        let range = self.workbook.worksheet_range(sheet_name).map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let available = self.sheet_names().join(", ");
                ExcelError::SheetNotFound {
                    sheet: sheet_name.to_string(),
                    available,
                }
            } else {
                ExcelError::from(e)
            }
        })?;

        Ok(RowIterator::new(range))
    }

    /// Read all rows from a sheet by index (0-based)
    pub fn rows_by_index(&mut self, index: usize) -> Result<RowIterator> {
        let sheet_names = self.sheet_names();
        let sheet_name = sheet_names.get(index).ok_or_else(|| {
            let available = sheet_names.join(", ");
            ExcelError::SheetNotFound {
                sheet: format!("index {}", index),
                available,
            }
        })?;

        self.rows(sheet_name)
    }

    /// Read a specific cell value
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use excelstream::reader::ExcelReader;
    ///
    /// let mut reader = ExcelReader::open("data.xlsx").unwrap();
    /// let value = reader.read_cell("Sheet1", 0, 0).unwrap();
    /// println!("Cell A1: {}", value);
    /// ```
    pub fn read_cell(&mut self, sheet_name: &str, row: u32, col: u32) -> Result<CellValue> {
        let range = self.workbook.worksheet_range(sheet_name).map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let available = self.sheet_names().join(", ");
                ExcelError::SheetNotFound {
                    sheet: sheet_name.to_string(),
                    available,
                }
            } else {
                ExcelError::from(e)
            }
        })?;

        let cell = range
            .get_value((row, col))
            .map(datatype_to_cellvalue)
            .unwrap_or(CellValue::Empty);

        Ok(cell)
    }

    /// Get the dimensions of a sheet (rows, cols)
    pub fn dimensions(&mut self, sheet_name: &str) -> Result<(u32, u32)> {
        let range = self.workbook.worksheet_range(sheet_name).map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let available = self.sheet_names().join(", ");
                ExcelError::SheetNotFound {
                    sheet: sheet_name.to_string(),
                    available,
                }
            } else {
                ExcelError::from(e)
            }
        })?;

        let (rows, cols) = range.get_size();
        Ok((rows as u32, cols as u32))
    }
}

/// Iterator over rows in an Excel sheet
pub struct RowIterator {
    range: Range<Data>,
    current_row: u32,
    max_row: u32,
}

impl RowIterator {
    fn new(range: Range<Data>) -> Self {
        let (rows, _) = range.get_size();
        let start = range.start().map(|(r, _)| r).unwrap_or(0);

        RowIterator {
            range,
            current_row: start,
            max_row: start + rows as u32,
        }
    }
}

impl Iterator for RowIterator {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.max_row {
            return None;
        }

        let row_idx = self.current_row;
        self.current_row += 1;

        let (_, cols) = self.range.get_size();
        let mut cells = Vec::with_capacity(cols);

        // Optimize: iterate directly without intermediate conversions
        let start_col = self.range.start().map(|(_, c)| c).unwrap_or(0);
        for col in start_col..start_col + cols as u32 {
            let cell_value = self
                .range
                .get_value((row_idx, col))
                .map(datatype_to_cellvalue)
                .unwrap_or(CellValue::Empty);

            cells.push(cell_value);
        }

        Some(Ok(Row::new(row_idx, cells)))
    }
}

/// Convert calamine Data to our CellValue
#[inline]
fn datatype_to_cellvalue(dt: &Data) -> CellValue {
    match dt {
        Data::Empty => CellValue::Empty,
        Data::String(s) => CellValue::String(s.clone()),
        Data::Float(f) => CellValue::Float(*f),
        Data::Int(i) => CellValue::Int(*i),
        Data::Bool(b) => CellValue::Bool(*b),
        Data::DateTime(d) => CellValue::DateTime(d.as_f64()),
        Data::Error(e) => CellValue::Error(format!("{:?}", e)),
        Data::DateTimeIso(s) => CellValue::String(s.clone()),
        Data::DurationIso(s) => CellValue::String(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn test_datatype_conversion() {
        let dt = Data::String("test".to_string());
        let cv = datatype_to_cellvalue(&dt);
        assert_eq!(cv, CellValue::String("test".to_string()));

        let dt = Data::Int(42);
        let cv = datatype_to_cellvalue(&dt);
        assert_eq!(cv, CellValue::Int(42));
    }
}
