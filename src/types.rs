//! Type definitions for Excel data

use std::fmt;

/// Cell style presets for formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellStyle {
    /// Default style - no formatting
    Default = 0,
    /// Bold text for headers
    HeaderBold = 1,
    /// Integer format with thousand separator (#,##0)
    NumberInteger = 2,
    /// Decimal format with 2 places (#,##0.00)
    NumberDecimal = 3,
    /// Currency format ($#,##0.00)
    NumberCurrency = 4,
    /// Percentage format (0.00%)
    NumberPercentage = 5,
    /// Date format (MM/DD/YYYY)
    DateDefault = 6,
    /// DateTime format (MM/DD/YYYY HH:MM:SS)
    DateTimestamp = 7,
    /// Bold text for emphasis
    TextBold = 8,
    /// Italic text for notes
    TextItalic = 9,
    /// Yellow background highlight
    HighlightYellow = 10,
    /// Green background highlight
    HighlightGreen = 11,
    /// Red background highlight
    HighlightRed = 12,
    /// Thin borders on all sides
    BorderThin = 13,
}

impl CellStyle {
    /// Get the style index for XML
    pub fn index(&self) -> u32 {
        *self as u32
    }
}

/// Styled cell value (combines value with formatting)
#[derive(Debug, Clone)]
pub struct StyledCell {
    /// The cell value
    pub value: CellValue,
    /// The cell style
    pub style: CellStyle,
}

impl StyledCell {
    /// Create a new styled cell
    pub fn new(value: CellValue, style: CellStyle) -> Self {
        StyledCell { value, style }
    }

    /// Create a cell with default style
    pub fn default_style(value: CellValue) -> Self {
        StyledCell {
            value,
            style: CellStyle::Default,
        }
    }
}

impl From<CellValue> for StyledCell {
    fn from(value: CellValue) -> Self {
        StyledCell::default_style(value)
    }
}

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
    /// Formula value (e.g., "=SUM(A1:A10)")
    /// The formula should start with '=' and use Excel formula syntax
    Formula(String),
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
            CellValue::Formula(f) => f.clone(),
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

/// Worksheet protection options
#[derive(Debug, Clone)]
pub struct ProtectionOptions {
    /// Password hash (optional) - use set_password() to hash
    pub password_hash: Option<String>,
    /// Allow selecting locked cells (default: true)
    pub select_locked_cells: bool,
    /// Allow selecting unlocked cells (default: true)
    pub select_unlocked_cells: bool,
    /// Allow formatting cells (default: false)
    pub format_cells: bool,
    /// Allow formatting columns (default: false)
    pub format_columns: bool,
    /// Allow formatting rows (default: false)
    pub format_rows: bool,
    /// Allow inserting columns (default: false)
    pub insert_columns: bool,
    /// Allow inserting rows (default: false)
    pub insert_rows: bool,
    /// Allow deleting columns (default: false)
    pub delete_columns: bool,
    /// Allow deleting rows (default: false)
    pub delete_rows: bool,
    /// Allow sorting (default: false)
    pub sort: bool,
    /// Allow auto filter (default: false)
    pub auto_filter: bool,
}

impl Default for ProtectionOptions {
    fn default() -> Self {
        ProtectionOptions {
            password_hash: None,
            select_locked_cells: true,
            select_unlocked_cells: true,
            format_cells: false,
            format_columns: false,
            format_rows: false,
            insert_columns: false,
            insert_rows: false,
            delete_columns: false,
            delete_rows: false,
            sort: false,
            auto_filter: false,
        }
    }
}

impl ProtectionOptions {
    /// Create new protection with default settings (most restrictive)
    pub fn new() -> Self {
        Self::default()
    }

    /// Set password for protection (hashed with Excel's algorithm)
    pub fn with_password(mut self, password: &str) -> Self {
        self.password_hash = Some(Self::hash_password(password));
        self
    }

    /// Allow users to select locked cells
    pub fn allow_select_locked_cells(mut self, allow: bool) -> Self {
        self.select_locked_cells = allow;
        self
    }

    /// Allow users to select unlocked cells
    pub fn allow_select_unlocked_cells(mut self, allow: bool) -> Self {
        self.select_unlocked_cells = allow;
        self
    }

    /// Allow users to format cells
    pub fn allow_format_cells(mut self, allow: bool) -> Self {
        self.format_cells = allow;
        self
    }

    /// Allow users to format columns
    pub fn allow_format_columns(mut self, allow: bool) -> Self {
        self.format_columns = allow;
        self
    }

    /// Allow users to format rows
    pub fn allow_format_rows(mut self, allow: bool) -> Self {
        self.format_rows = allow;
        self
    }

    /// Allow users to insert columns
    pub fn allow_insert_columns(mut self, allow: bool) -> Self {
        self.insert_columns = allow;
        self
    }

    /// Allow users to insert rows
    pub fn allow_insert_rows(mut self, allow: bool) -> Self {
        self.insert_rows = allow;
        self
    }

    /// Allow users to delete columns
    pub fn allow_delete_columns(mut self, allow: bool) -> Self {
        self.delete_columns = allow;
        self
    }

    /// Allow users to delete rows
    pub fn allow_delete_rows(mut self, allow: bool) -> Self {
        self.delete_rows = allow;
        self
    }

    /// Allow users to sort
    pub fn allow_sort(mut self, allow: bool) -> Self {
        self.sort = allow;
        self
    }

    /// Allow users to use auto filter
    pub fn allow_auto_filter(mut self, allow: bool) -> Self {
        self.auto_filter = allow;
        self
    }

    /// Hash password using Excel's algorithm (simple XOR-based)
    fn hash_password(password: &str) -> String {
        let mut hash: u16 = 0;
        for ch in password.chars().rev() {
            let val = (ch as u16).rotate_left(1);
            hash ^= val;
        }
        hash ^= password.len() as u16;
        hash ^= 0xCE4B;
        format!("{:04X}", hash)
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
