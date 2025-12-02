//! Type definitions for Excel data

use std::fmt;

/// Represents a single cell value in an Excel worksheet
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// Empty cell
    Empty,
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// DateTime value (Excel serial date number)
    DateTime(f64),
    /// Error value
    Error(String),
}

impl CellValue {
    /// Convert cell value to string
    pub fn as_string(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::String(s) => s.clone(),
            CellValue::Int(i) => i.to_string(),
            CellValue::Float(f) => f.to_string(),
            CellValue::Bool(b) => b.to_string(),
            CellValue::DateTime(d) => d.to_string(),
            CellValue::Error(e) => format!("ERROR: {}", e),
        }
    }

    /// Check if cell is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    /// Try to convert to integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            CellValue::Int(i) => Some(*i),
            CellValue::Float(f) => Some(*f as i64),
            CellValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Try to convert to float
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CellValue::Float(f) => Some(*f),
            CellValue::Int(i) => Some(*i as f64),
            CellValue::DateTime(d) => Some(*d),
            CellValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Try to convert to boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CellValue::Bool(b) => Some(*b),
            CellValue::Int(i) => Some(*i != 0),
            CellValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" => Some(true),
                "false" | "no" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        CellValue::String(s.to_string())
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        CellValue::String(s)
    }
}

impl From<i64> for CellValue {
    fn from(i: i64) -> Self {
        CellValue::Int(i)
    }
}

impl From<f64> for CellValue {
    fn from(f: f64) -> Self {
        CellValue::Float(f)
    }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self {
        CellValue::Bool(b)
    }
}

/// Represents a cell with its position
#[derive(Debug, Clone)]
pub struct Cell {
    /// Row index (0-based)
    pub row: u32,
    /// Column index (0-based)
    pub col: u32,
    /// Cell value
    pub value: CellValue,
}

impl Cell {
    /// Create a new cell
    pub fn new(row: u32, col: u32, value: CellValue) -> Self {
        Cell { row, col, value }
    }

    /// Get Excel-style cell reference (e.g., "A1", "B2")
    pub fn reference(&self) -> String {
        format!("{}{}", Self::col_to_letter(self.col), self.row + 1)
    }

    /// Convert column index to Excel letter (0 -> A, 25 -> Z, 26 -> AA)
    fn col_to_letter(col: u32) -> String {
        let mut result = String::new();
        let mut col = col + 1;
        
        while col > 0 {
            col -= 1;
            result.insert(0, (b'A' + (col % 26) as u8) as char);
            col /= 26;
        }
        
        result
    }
}

/// Represents a row of cells
#[derive(Debug, Clone)]
pub struct Row {
    /// Row index (0-based)
    pub index: u32,
    /// Cells in this row
    pub cells: Vec<CellValue>,
}

impl Row {
    /// Create a new row
    pub fn new(index: u32, cells: Vec<CellValue>) -> Self {
        Row { index, cells }
    }

    /// Get cell at column index
    pub fn get(&self, col: usize) -> Option<&CellValue> {
        self.cells.get(col)
    }

    /// Get number of cells
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if row is empty
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty() || self.cells.iter().all(|c| c.is_empty())
    }

    /// Convert row to vector of strings
    pub fn to_strings(&self) -> Vec<String> {
        self.cells.iter().map(|c| c.as_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_reference() {
        let cell = Cell::new(0, 0, CellValue::Empty);
        assert_eq!(cell.reference(), "A1");

        let cell = Cell::new(0, 25, CellValue::Empty);
        assert_eq!(cell.reference(), "Z1");

        let cell = Cell::new(0, 26, CellValue::Empty);
        assert_eq!(cell.reference(), "AA1");
    }

    #[test]
    fn test_cell_value_conversions() {
        let val = CellValue::Int(42);
        assert_eq!(val.as_i64(), Some(42));
        assert_eq!(val.as_f64(), Some(42.0));

        let val = CellValue::String("true".to_string());
        assert_eq!(val.as_bool(), Some(true));
    }
}
