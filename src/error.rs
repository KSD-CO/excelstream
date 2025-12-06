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
}

impl From<zip::result::ZipError> for ExcelError {
    fn from(err: zip::result::ZipError) -> Self {
        ExcelError::WriteError(err.to_string())
    }
}
