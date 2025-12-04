//! Shared strings table for string deduplication

use super::xml_writer::XmlWriter;
use crate::error::Result;
use indexmap::IndexMap;
use std::io::Write;

/// Shared strings table that deduplicates strings across the workbook
pub struct SharedStrings {
    strings: Vec<String>,
    string_map: IndexMap<String, u32>,
    max_unique_strings: usize, // Giới hạn số string unique để tiết kiệm memory
}

impl SharedStrings {
    pub fn new() -> Self {
        SharedStrings {
            strings: Vec::with_capacity(1000),
            string_map: IndexMap::with_capacity(1000),
            max_unique_strings: 100_000, // Giới hạn 100K unique strings
        }
    }

    /// Tạo với giới hạn số unique strings tùy chỉnh
    pub fn with_capacity(capacity: usize, max_unique: usize) -> Self {
        SharedStrings {
            strings: Vec::with_capacity(capacity),
            string_map: IndexMap::with_capacity(capacity),
            max_unique_strings: max_unique,
        }
    }

    /// Add a string and get its index
    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(&index) = self.string_map.get(s) {
            return index;
        }

        // Nếu đã đạt giới hạn, không lưu vào map nữa (tránh memory leak)
        // Nhưng vẫn lưu string để đảm bảo tính đúng
        if self.strings.len() >= self.max_unique_strings {
            let index = self.strings.len() as u32;
            self.strings.push(s.to_string());
            return index;
        }

        let index = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.string_map.insert(s.to_string(), index);
        index
    }

    /// Get number of unique strings
    pub fn count(&self) -> usize {
        self.strings.len()
    }

    /// Write shared strings XML
    pub fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<()> {
        // XML declaration
        writer.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n")?;

        // Start sst element
        writer.start_element("sst")?;
        writer.attribute(
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        )?;
        writer.attribute_int("count", self.strings.len() as i64)?;
        writer.attribute_int("uniqueCount", self.strings.len() as i64)?;
        writer.close_start_tag()?;

        // Write each string
        for s in &self.strings {
            writer.start_element("si")?;
            writer.close_start_tag()?;

            writer.start_element("t")?;
            writer.close_start_tag()?;
            writer.write_escaped(s)?;
            writer.end_element("t")?;

            writer.end_element("si")?;
        }

        writer.end_element("sst")?;
        Ok(())
    }
}

impl Default for SharedStrings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_strings() {
        let mut ss = SharedStrings::new();

        let idx1 = ss.add_string("Hello");
        let idx2 = ss.add_string("World");
        let idx3 = ss.add_string("Hello"); // Duplicate

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 0); // Should return same index
        assert_eq!(ss.count(), 2);
    }
}
