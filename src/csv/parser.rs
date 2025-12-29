//! CSV parsing with RFC 4180-like behavior

/// CSV parser for reading CSV data
pub struct CsvParser {
    delimiter: u8,
    quote_char: u8,
}

impl CsvParser {
    /// Create a new CSV parser with custom delimiter and quote character
    pub fn new(delimiter: u8, quote_char: u8) -> Self {
        Self {
            delimiter,
            quote_char,
        }
    }

    /// Parse CSV line into fields
    pub fn parse_line(&self, line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == self.quote_char as char {
                if in_quotes {
                    // Check for escaped quote ("")
                    if chars.peek() == Some(&(self.quote_char as char)) {
                        current_field.push(self.quote_char as char);
                        chars.next(); // Skip second quote
                    } else {
                        // End of quoted field
                        in_quotes = false;
                    }
                } else {
                    // Start of quoted field
                    in_quotes = true;
                }
            } else if ch == self.delimiter as char && !in_quotes {
                // Field separator
                fields.push(current_field.clone());
                current_field.clear();
            } else {
                // Regular character
                current_field.push(ch);
            }
        }

        // Add last field
        fields.push(current_field);
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line("a,b,c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_quoted() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line(r#""a,b",c"#), vec!["a,b", "c"]);
    }

    #[test]
    fn test_escaped_quotes() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(
            parser.parse_line(r#""Say ""Hello""",world"#),
            vec![r#"Say "Hello""#, "world"]
        );
    }

    #[test]
    fn test_empty_fields() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line("a,,c"), vec!["a", "", "c"]);
    }

    #[test]
    fn test_all_empty() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line(",,"), vec!["", "", ""]);
    }

    #[test]
    fn test_quoted_with_newline() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(
            parser.parse_line("\"Line 1\nLine 2\",normal"),
            vec!["Line 1\nLine 2", "normal"]
        );
    }

    #[test]
    fn test_mixed_quoted_unquoted() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line(r#"a,"b,c",d"#), vec!["a", "b,c", "d"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new(b';', b'"');
        assert_eq!(parser.parse_line(r#"a;"b;c";d"#), vec!["a", "b;c", "d"]);
    }

    #[test]
    fn test_empty_line() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line(""), vec![""]);
    }

    #[test]
    fn test_single_field() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line("hello"), vec!["hello"]);
    }

    #[test]
    fn test_quoted_empty() {
        let parser = CsvParser::new(b',', b'"');
        assert_eq!(parser.parse_line(r#""","""#), vec!["", ""]);
    }
}
