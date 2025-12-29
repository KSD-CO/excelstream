//! CSV utilities for encoding and parsing

mod encoder;
mod parser;

pub use encoder::CsvEncoder;
pub use parser::CsvParser;

// Re-export CompressionMethod from s-zip for convenience
pub use s_zip::CompressionMethod;
