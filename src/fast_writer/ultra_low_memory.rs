//! Ultra-low memory workbook - wrapper around ZeroTempWorkbook

use super::zero_temp_workbook::ZeroTempWorkbook;
use crate::error::Result;
use crate::types::{CellValue, ProtectionOptions};
use std::path::Path;

pub struct UltraLowMemoryWorkbook {
    inner: ZeroTempWorkbook,
    compression_level: u32,
}

impl UltraLowMemoryWorkbook {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::with_compression(path, 6)
    }

    pub fn with_compression<P: AsRef<Path>>(path: P, compression_level: u32) -> Result<Self> {
        let inner = ZeroTempWorkbook::new(
            path.as_ref().to_str().unwrap_or("output.xlsx"),
            compression_level.min(9),
        )?;

        Ok(UltraLowMemoryWorkbook {
            inner,
            compression_level: compression_level.min(9),
        })
    }

    pub fn protect_sheet(&mut self, options: ProtectionOptions) -> Result<()> {
        self.inner.protect_sheet(options)
    }

    pub fn add_worksheet(&mut self, name: &str) -> Result<()> {
        self.inner.add_worksheet(name)
    }

    pub fn write_row<I, S>(&mut self, values: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.inner.write_row(values)
    }

    pub fn write_row_typed(&mut self, values: &[CellValue]) -> Result<()> {
        // Convert to StyledCell with default style to preserve types
        let styled_cells: Vec<crate::types::StyledCell> = values
            .iter()
            .map(|v| crate::types::StyledCell::new(v.clone(), crate::types::CellStyle::Default))
            .collect();

        self.inner.write_row_styled(&styled_cells)
    }

    pub fn write_row_styled(&mut self, values: &[crate::types::StyledCell]) -> Result<()> {
        // Delegate to ZeroTempWorkbook which now supports styling
        self.inner.write_row_styled(values)
    }

    pub fn set_compression_level(&mut self, level: u32) {
        self.compression_level = level.min(9);
    }

    pub fn compression_level(&self) -> u32 {
        self.compression_level
    }

    pub fn close(self) -> Result<()> {
        self.inner.close()
    }

    // Stub methods for API compatibility
    pub fn set_column_width(&mut self, _col: u32, _width: f64) -> Result<()> {
        // TODO: Implement in ZeroTempWorkbook
        Ok(())
    }

    pub fn set_next_row_height(&mut self, _height: f64) -> Result<()> {
        // TODO: Implement in ZeroTempWorkbook
        Ok(())
    }

    pub fn set_flush_interval(&mut self, _interval: u32) {
        // Not applicable for ZeroTempWorkbook (always streaming)
    }

    pub fn set_max_buffer_size(&mut self, _size: usize) {
        // Not applicable for ZeroTempWorkbook (uses fixed 4KB buffer)
    }
}
