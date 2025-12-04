//! Optimized XML writer with minimal allocations

use crate::error::Result;
use std::io::Write;

/// Fast XML writer that writes directly to output without intermediate buffers
pub struct XmlWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
    flush_threshold: usize,
}

impl<W: Write> XmlWriter<W> {
    pub fn new(writer: W) -> Self {
        Self::with_capacity(writer, 8192)
    }

    pub fn with_capacity(writer: W, capacity: usize) -> Self {
        XmlWriter {
            writer,
            buffer: Vec::with_capacity(capacity),
            flush_threshold: capacity / 2, // Flush at 50% capacity
        }
    }

    /// Auto-flush if buffer exceeds threshold
    #[inline]
    fn auto_flush(&mut self) -> Result<()> {
        if self.buffer.len() >= self.flush_threshold {
            self.flush()?;
        }
        Ok(())
    }

    /// Write raw bytes directly
    #[inline]
    pub fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(data);
        self.auto_flush()
    }

    /// Write string data
    #[inline]
    pub fn write_str(&mut self, s: &str) -> Result<()> {
        self.write_raw(s.as_bytes())
    }

    /// Write XML element start tag
    #[inline]
    pub fn start_element(&mut self, name: &str) -> Result<()> {
        self.write_raw(b"<")?;
        self.write_str(name)?;
        Ok(())
    }

    /// Write XML element end tag
    #[inline]
    pub fn end_element(&mut self, name: &str) -> Result<()> {
        self.write_raw(b"</")?;
        self.write_str(name)?;
        self.write_raw(b">")
    }

    /// Write self-closing element
    #[inline]
    pub fn empty_element(&mut self, name: &str) -> Result<()> {
        self.write_raw(b"<")?;
        self.write_str(name)?;
        self.write_raw(b"/>")
    }

    /// Write attribute
    #[inline]
    pub fn attribute(&mut self, name: &str, value: &str) -> Result<()> {
        self.write_raw(b" ")?;
        self.write_str(name)?;
        self.write_raw(b"=\"")?;
        self.write_escaped(value)?;
        self.write_raw(b"\"")
    }

    /// Write attribute with integer value
    #[inline]
    pub fn attribute_int(&mut self, name: &str, value: i64) -> Result<()> {
        self.write_raw(b" ")?;
        self.write_str(name)?;
        self.write_raw(b"=\"")?;
        self.write_str(&value.to_string())?;
        self.write_raw(b"\"")
    }

    /// Close start tag
    #[inline]
    pub fn close_start_tag(&mut self) -> Result<()> {
        self.write_raw(b">")
    }

    /// Write text content with XML escaping
    #[inline]
    pub fn write_escaped(&mut self, text: &str) -> Result<()> {
        // Iterate over Unicode scalar values to avoid splitting UTF-8 sequences.
        for ch in text.chars() {
            match ch {
                '&' => self.write_raw(b"&amp;")?,
                '<' => self.write_raw(b"&lt;")?,
                '>' => self.write_raw(b"&gt;")?,
                '"' => self.write_raw(b"&quot;")?,
                '\'' => self.write_raw(b"&apos;")?,
                // Allowed XML whitespace characters: tab (U+0009), LF (U+000A), CR (U+000D)
                c if (c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r' => {
                    // Drop illegal control characters (they would break XML parsers on Windows)
                    continue;
                }
                c => {
                    // Write UTF-8 bytes of the character
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    self.buffer.extend_from_slice(s.as_bytes());
                }
            }
        }

        self.auto_flush()
    }

    /// Flush buffer to underlying writer
    pub fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer)?;
            self.buffer.clear();
        }
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_writer() {
        let mut output = Vec::new();
        let mut writer = XmlWriter::new(&mut output);

        writer.start_element("root").unwrap();
        writer.attribute("attr", "value").unwrap();
        writer.close_start_tag().unwrap();
        writer.write_str("content").unwrap();
        writer.end_element("root").unwrap();
        writer.flush().unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "<root attr=\"value\">content</root>"
        );
    }

    #[test]
    fn test_xml_escaping() {
        let mut output = Vec::new();
        let mut writer = XmlWriter::new(&mut output);

        writer.write_escaped("<test>&value</test>").unwrap();
        writer.flush().unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "&lt;test&gt;&amp;value&lt;/test&gt;"
        );
    }
}
