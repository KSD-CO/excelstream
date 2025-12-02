//! Helper functions để tự động cấu hình memory constraints

use crate::fast_writer::FastWorkbook;
use crate::error::Result;
use std::path::Path;

/// Memory profile cho các loại pods khác nhau
#[derive(Debug, Clone, Copy)]
pub enum MemoryProfile {
    /// Pods nhỏ (< 512MB): flush mỗi 100 rows, buffer 256KB
    Low,
    /// Pods trung bình (512MB-1GB): flush mỗi 500 rows, buffer 512KB
    Medium,
    /// Pods lớn (> 1GB): flush mỗi 1000 rows, buffer 1MB (default)
    High,
    /// Custom profile
    Custom { flush_interval: u32, max_buffer_size: usize },
}

impl MemoryProfile {
    /// Tạo profile từ memory limit (MB)
    pub fn from_memory_mb(memory_mb: usize) -> Self {
        if memory_mb < 512 {
            MemoryProfile::Low
        } else if memory_mb < 1024 {
            MemoryProfile::Medium
        } else {
            MemoryProfile::High
        }
    }

    /// Detect từ environment variable MEMORY_LIMIT_MB
    pub fn from_env() -> Self {
        std::env::var("MEMORY_LIMIT_MB")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .map(Self::from_memory_mb)
            .unwrap_or(MemoryProfile::High)
    }

    fn apply(&self, workbook: &mut FastWorkbook) {
        match self {
            MemoryProfile::Low => {
                workbook.set_flush_interval(100);
                workbook.set_max_buffer_size(256 * 1024);
            }
            MemoryProfile::Medium => {
                workbook.set_flush_interval(500);
                workbook.set_max_buffer_size(512 * 1024);
            }
            MemoryProfile::High => {
                workbook.set_flush_interval(1000);
                workbook.set_max_buffer_size(1024 * 1024);
            }
            MemoryProfile::Custom { flush_interval, max_buffer_size } => {
                workbook.set_flush_interval(*flush_interval);
                workbook.set_max_buffer_size(*max_buffer_size);
            }
        }
    }
}

/// Tạo FastWorkbook với memory profile tự động
pub fn create_workbook_auto<P: AsRef<Path>>(path: P) -> Result<FastWorkbook> {
    let profile = MemoryProfile::from_env();
    create_workbook_with_profile(path, profile)
}

/// Tạo FastWorkbook với memory profile chỉ định
pub fn create_workbook_with_profile<P: AsRef<Path>>(
    path: P,
    profile: MemoryProfile,
) -> Result<FastWorkbook> {
    let mut workbook = FastWorkbook::new(path)?;
    profile.apply(&mut workbook);
    Ok(workbook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profile_from_mb() {
        assert!(matches!(MemoryProfile::from_memory_mb(256), MemoryProfile::Low));
        assert!(matches!(MemoryProfile::from_memory_mb(768), MemoryProfile::Medium));
        assert!(matches!(MemoryProfile::from_memory_mb(2048), MemoryProfile::High));
    }
}
