//! # rust-excelize
//!
//! A high-performance Rust library for streaming Excel import/export operations.
//!
//! ## Features
//!
//! - **Streaming Read**: Read large Excel files without loading entire file into memory
//! - **Streaming Write**: Write Excel files row by row efficiently
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
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut writer = ExcelWriter::new("output.xlsx")?;
//! 
//! writer.write_row(&["Name", "Age", "City"])?;
//! writer.write_row(&["Alice", "30", "New York"])?;
//! writer.write_row(&["Bob", "25", "San Francisco"])?;
//! 
//! writer.save()?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod reader;
pub mod writer;
pub mod types;
pub mod fast_writer;

pub use error::{ExcelError, Result};
pub use reader::ExcelReader;
pub use writer::ExcelWriter;
pub use types::{Cell, CellValue, Row};

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
