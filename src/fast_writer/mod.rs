//! Fast Excel writer optimized for streaming
//! 
//! This module provides a high-performance Excel writer that focuses on:
//! - Minimal memory allocations
//! - Direct XML generation
//! - Optimized ZIP compression
//! - Streaming-first design

pub mod workbook;
pub mod worksheet;
pub mod shared_strings;
pub mod xml_writer;
pub mod memory;

use crate::error::Result;
use std::path::Path;

pub use workbook::FastWorkbook;
pub use worksheet::FastWorksheet;
pub use memory::{MemoryProfile, create_workbook_auto, create_workbook_with_profile};

/// Create a fast Excel writer optimized for large datasets
///
/// # Examples
///
/// ```no_run
/// use excelstream::fast_writer::FastWorkbook;
///
/// let mut workbook = FastWorkbook::new("output.xlsx")?;
/// workbook.add_worksheet("Sheet1")?;
/// 
/// workbook.write_row(&["Name", "Age", "Email"])?;
/// workbook.write_row(&["Alice", "30", "alice@example.com"])?;
/// 
/// workbook.close()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn create_fast_writer<P: AsRef<Path>>(path: P) -> Result<FastWorkbook> {
    FastWorkbook::new(path)
}
