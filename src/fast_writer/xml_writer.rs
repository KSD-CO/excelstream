//! Optimized XML writer with minimal allocations

use std::io::Write;
use crate::error::Result;

/// Fast XML writer that writes directly to output without intermediate buffers
pub struct XmlWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
}

impl<W: Write> XmlWriter<W> {
    pub fn new(writer: W) -> Self {
        XmlWriter {
            writer,
            buffer: Vec::with_capacity(8192), // 8KB buffer
        }
    }

    /// Write raw bytes directly
    #[inline]
    pub fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(data);
        if self.buffer.len() > 4096 {
            self.flush()?;
        }
        Ok(())
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
        for byte in text.bytes() {
            match byte {
                b'&' => self.write_raw(b"&amp;")?,
                b'<' => self.write_raw(b"&lt;")?,
                b'>' => self.write_raw(b"&gt;")?,
                b'"' => self.write_raw(b"&quot;")?,
                b'\'' => self.write_raw(b"&apos;")?,
                _ => self.buffer.push(byte),
            }
        }
        if self.buffer.len() > 4096 {
            self.flush()?;
        }
        Ok(())
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
