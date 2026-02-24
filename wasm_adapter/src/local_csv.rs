//! Minimal CSV parser copied for WASM adapter to avoid heavy native deps.
/// CSV parser for reading CSV data (simplified, line-based)
pub struct CsvParser {
    delimiter: u8,
    quote_char: u8,
}

impl CsvParser {
    pub fn new(delimiter: u8, quote_char: u8) -> Self {
        Self {
            delimiter,
            quote_char,
        }
    }

    pub fn parse_line(&self, line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == self.quote_char as char {
                if in_quotes {
                    if chars.peek() == Some(&(self.quote_char as char)) {
                        current.push(self.quote_char as char);
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            } else if ch == self.delimiter as char && !in_quotes {
                fields.push(current.clone());
                current.clear();
            } else {
                current.push(ch);
            }
        }
        fields.push(current);
        fields
    }
}
