//! Parquet ↔ Excel streaming conversion
//!
//! This module provides bidirectional conversion between Parquet and Excel formats
//! with streaming support for handling large files efficiently.
//!
//! # Features
//!
//! - Convert Parquet → Excel with constant memory usage
//! - Convert Excel → Parquet with columnar optimization
//! - Support for common data types (string, int, float, datetime, bool)
//! - Streaming row-by-row processing
//!
//! # Parquet → Excel Example
//!
//! ```no_run
//! use excelstream::parquet::ParquetToExcelConverter;
//!
//! // Convert entire file
//! let converter = ParquetToExcelConverter::new("data.parquet")?;
//! converter.convert_to_excel("output.xlsx")?;
//!
//! // Streaming conversion
//! use excelstream::{ExcelWriter, parquet::ParquetReader};
//!
//! let mut reader = ParquetReader::open("large.parquet")?;
//! let mut writer = ExcelWriter::new("output.xlsx")?;
//!
//! // Write headers from schema
//! writer.write_header_bold(&reader.column_names())?;
//!
//! // Stream rows
//! for row in reader.rows()? {
//!     writer.write_row(&row?)?;
//! }
//! writer.save()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Excel → Parquet Example
//!
//! ```no_run
//! use excelstream::parquet::ExcelToParquetConverter;
//!
//! // Convert with schema inference
//! let converter = ExcelToParquetConverter::new("data.xlsx")?;
//! converter.convert_to_parquet("output.parquet")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(feature = "parquet-support")]
pub mod reader;

#[cfg(feature = "parquet-support")]
pub mod converter;

#[cfg(feature = "parquet-support")]
pub use reader::ParquetReader;

#[cfg(feature = "parquet-support")]
pub use converter::{ExcelToParquetConverter, ParquetToExcelConverter};
