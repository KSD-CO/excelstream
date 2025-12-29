//! CSV encoding with RFC 4180-like behavior

/// CSV encoder for writing properly formatted CSV data
pub struct CsvEncoder {
    delimiter: u8,
    quote_char: u8,
}

impl CsvEncoder {
    /// Create a new CSV encoder with custom delimiter and quote character
    pub fn new(delimiter: u8, quote_char: u8) -> Self {
        Self {
            delimiter,
            quote_char,
        }
    }

    /// Encode entire row into buffer
    pub fn encode_row(&self, fields: &[&str], buffer: &mut Vec<u8>) {
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                buffer.push(self.delimiter);
            }
            self.encode_field(field, buffer);
        }
    }

    /// Encode single field with proper quoting/escaping
    fn encode_field(&self, field: &str, buffer: &mut Vec<u8>) {
        if self.needs_quoting(field) {
            // Quote the field
            buffer.push(self.quote_char);
            for byte in field.bytes() {
                if byte == self.quote_char {
                    // Escape quotes by doubling: " -> ""
                    buffer.push(self.quote_char);
                    buffer.push(self.quote_char);
                } else {
                    buffer.push(byte);
                }
            }
            buffer.push(self.quote_char);
        } else {
            // No quoting needed
            buffer.extend_from_slice(field.as_bytes());
        }
    }

    /// Check if field requires quoting
    fn needs_quoting(&self, field: &str) -> bool {
        field
            .bytes()
            .any(|b| b == self.delimiter || b == self.quote_char || b == b'\n' || b == b'\r')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fields() {
        let encoder = CsvEncoder::new(b',', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&["a", "b", "c"], &mut buffer);
        assert_eq!(String::from_utf8(buffer).unwrap(), "a,b,c");
    }

    #[test]
    fn test_quoted_fields() {
        let encoder = CsvEncoder::new(b',', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&["a,b", "c"], &mut buffer);
        assert_eq!(String::from_utf8(buffer).unwrap(), r#""a,b",c"#);
    }

    #[test]
    fn test_escaped_quotes() {
        let encoder = CsvEncoder::new(b',', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&[r#"Say "Hello""#, "world"], &mut buffer);
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            r#""Say ""Hello""",world"#
        );
    }

    #[test]
    fn test_newlines() {
        let encoder = CsvEncoder::new(b',', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&["Line 1\nLine 2", "normal"], &mut buffer);
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            "\"Line 1\nLine 2\",normal"
        );
    }

    #[test]
    fn test_empty_fields() {
        let encoder = CsvEncoder::new(b',', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&["a", "", "c"], &mut buffer);
        assert_eq!(String::from_utf8(buffer).unwrap(), "a,,c");
    }

    #[test]
    fn test_all_empty() {
        let encoder = CsvEncoder::new(b',', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&["", "", ""], &mut buffer);
        assert_eq!(String::from_utf8(buffer).unwrap(), ",,");
    }

    #[test]
    fn test_custom_delimiter() {
        let encoder = CsvEncoder::new(b';', b'"');
        let mut buffer = Vec::new();
        encoder.encode_row(&["a", "b;c", "d"], &mut buffer);
        assert_eq!(String::from_utf8(buffer).unwrap(), r#"a;"b;c";d"#);
    }
}
