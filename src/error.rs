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
    #[error("Sheet not found: {0}")]
    SheetNotFound(String),

    /// Invalid cell reference
    #[error("Invalid cell reference: {0}")]
    InvalidCell(String),

    /// IO error wrapper
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Calamine error wrapper
    #[error("Calamine error: {0}")]
    CaliamineError(String),

    /// XlsxWriter error wrapper
    #[error("XlsxWriter error: {0}")]
    XlsxWriterError(String),

    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Feature not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),
}

impl From<calamine::Error> for ExcelError {
    fn from(err: calamine::Error) -> Self {
        ExcelError::CaliamineError(err.to_string())
    }
}

impl From<calamine::XlsxError> for ExcelError {
    fn from(err: calamine::XlsxError) -> Self {
        ExcelError::CaliamineError(err.to_string())
    }
}

impl From<calamine::XlsError> for ExcelError {
    fn from(err: calamine::XlsError) -> Self {
        ExcelError::CaliamineError(err.to_string())
    }
}

impl From<rust_xlsxwriter::XlsxError> for ExcelError {
    fn from(err: rust_xlsxwriter::XlsxError) -> Self {
        ExcelError::XlsxWriterError(err.to_string())
    }
}

impl From<zip::result::ZipError> for ExcelError {
    fn from(err: zip::result::ZipError) -> Self {
        ExcelError::WriteError(err.to_string())
    }
}
