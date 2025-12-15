//! Error types for the rust-excelize library

use thiserror::Error;

/// Result type alias for rust-excelize operations
pub type Result<T> = std::result::Result<T, ExcelError>;

/// Main error type for all Excel operations
#[derive(Error, Debug)]
pub enum ExcelError {
    /// Error occurred while reading Excel file
    #[error("Failed to read Excel file: {0}")]
    ReadError(String),

    /// Error occurred while writing Excel file
    #[error("Failed to write Excel file: {0}")]
    WriteError(String),

    /// Invalid sheet name or sheet not found
    #[error("Sheet '{sheet}' not found. Available sheets: {available}")]
    SheetNotFound { sheet: String, available: String },

    /// Error occurred while writing a row
    #[error("Failed to write row {row} to sheet '{sheet}': {source}")]
    WriteRowError {
        row: u32,
        sheet: String,
        #[source]
        source: Box<ExcelError>,
    },

    /// Invalid cell reference
    #[error("Invalid cell reference: {0}")]
    InvalidCell(String),

    /// IO error wrapper
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Calamine error wrapper
    #[error("Calamine error: {0}")]
    CaliamineError(String),

    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Feature not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),

    /// Invalid state or operation
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// ZIP error
    #[error("ZIP error: {0}")]
    ZipError(String),
}

// Convert s-zip errors to ExcelError for backward compatibility
impl From<s_zip::SZipError> for ExcelError {
    fn from(err: s_zip::SZipError) -> Self {
        match err {
            s_zip::SZipError::Io(e) => ExcelError::IoError(e),
            s_zip::SZipError::InvalidFormat(msg) => ExcelError::InvalidFormat(msg),
            s_zip::SZipError::EntryNotFound(name) => {
                ExcelError::ReadError(format!("ZIP entry not found: {}", name))
            }
            s_zip::SZipError::UnsupportedCompression(method) => {
                ExcelError::NotSupported(format!("Unsupported compression method: {}", method))
            }
        }
    }
}

// Note: std::io::Error is already mapped via the `IoError(#[from] std::io::Error)` variant above.
