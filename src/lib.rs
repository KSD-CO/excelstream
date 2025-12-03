//! # excelstream
//!
//! A high-performance Rust library for streaming Excel import/export operations.
//!
//! ## Features
//!
//! - **Streaming Read**: Read large Excel files without loading entire file into memory
//! - **Streaming Write**: Write millions of rows with constant ~80MB memory usage
//! - **Formula Support**: Write Excel formulas that calculate correctly
//! - **High Performance**: 30K-45K rows/sec throughput with true streaming
//! - **Better Errors**: Context-rich error messages with debugging info
//! - **Multiple Formats**: Support for XLSX, XLS, ODS formats
//! - **Type Safety**: Strong typing with Rust's type system
//! - **Zero-copy**: Minimize memory allocations where possible
//!
//! ## Quick Start
//!
//! ### Reading Excel Files (Streaming)
//!
//! ```rust,no_run
//! use excelstream::reader::ExcelReader;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut reader = ExcelReader::open("data.xlsx")?;
//!
//! for row_result in reader.rows("Sheet1")? {
//!     let row = row_result?;
//!     println!("Row: {:?}", row);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Writing Excel Files (Streaming)
//!
//! ```rust,no_run
//! use excelstream::writer::ExcelWriter;
//! use excelstream::types::CellValue;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut writer = ExcelWriter::new("output.xlsx")?;
//!
//! // Write header
//! writer.write_header(&["Name", "Age", "City"])?;
//!
//! // Write data rows with typed values
//! writer.write_row_typed(&[
//!     CellValue::String("Alice".to_string()),
//!     CellValue::Int(30),
//!     CellValue::String("New York".to_string()),
//! ])?;
//!
//! // Write with formulas
//! writer.write_row_typed(&[
//!     CellValue::String("Total".to_string()),
//!     CellValue::Formula("=COUNT(B2:B10)".to_string()),
//!     CellValue::Empty,
//! ])?;
//!
//! writer.save()?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod fast_writer;
pub mod reader;
pub mod types;
pub mod writer;

pub use error::{ExcelError, Result};
pub use reader::ExcelReader;
pub use types::{Cell, CellStyle, CellValue, Row, StyledCell};
pub use writer::ExcelWriter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        // Test that all public types are accessible
        let _ = std::marker::PhantomData::<ExcelError>;
        let _ = std::marker::PhantomData::<ExcelReader>;
        let _ = std::marker::PhantomData::<ExcelWriter>;
    }
}
