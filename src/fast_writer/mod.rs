//! Fast Excel writer optimized for streaming
//!
//! This module provides a high-performance Excel writer that focuses on:
//! - Minimal memory allocations (< 80 MB for any file size)
//! - Direct XML generation
//! - Optimized ZIP compression
//! - Streaming-first design

pub mod memory;
pub mod shared_strings;
pub mod ultra_low_memory;
pub mod workbook;
pub mod worksheet;
pub mod xml_writer;

use crate::error::Result;
use std::path::Path;

pub use memory::{create_workbook_auto, create_workbook_with_profile, MemoryProfile};
pub use ultra_low_memory::UltraLowMemoryWorkbook;

// Re-export UltraLowMemoryWorkbook as FastWorkbook (new default)
pub use ultra_low_memory::UltraLowMemoryWorkbook as FastWorkbook;

// Keep old FastWorkbook available under different name for backwards compatibility
pub use workbook::FastWorkbook as LegacyFastWorkbook;
pub use worksheet::FastWorksheet;

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
