//! Cloud storage integration for ExcelStream
//!
//! This module provides direct streaming to/from cloud storage (S3, GCS, Azure)
//! without requiring local disk space. Perfect for serverless and containerized environments.
//!
//! # Features
//!
//! - Stream Excel files directly to S3/GCS/Azure
//! - Multipart upload for large files
//! - Constant memory usage (~30-35 MB)
//! - No temporary files
//!
//! # S3 Example
//!
//! ```no_run
//! use excelstream::cloud::S3ExcelWriter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut writer = S3ExcelWriter::builder()
//!         .bucket("my-bucket")
//!         .key("reports/monthly.xlsx")
//!         .region("us-east-1")
//!         .build()
//!         .await?;
//!
//!     writer.write_row(&["ID", "Name", "Amount"]).await?;
//!     writer.write_row(&["1", "Alice", "1000"]).await?;
//!
//!     writer.save().await?;
//!     Ok(())
//! }
//! ```
//!
//! # GCS Example
//!
//! ```no_run
//! use excelstream::cloud::GCSExcelWriter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut writer = GCSExcelWriter::builder()
//!         .bucket("my-bucket")
//!         .object("reports/monthly.xlsx")
//!         .build()
//!         .await?;
//!
//!     writer.write_row(&["ID", "Name", "Amount"]).await?;
//!     writer.write_row(&["1", "Alice", "1000"]).await?;
//!
//!     writer.save().await?;
//!     Ok(())
//! }
//! ```

#[cfg(feature = "cloud-s3")]
pub mod s3_writer;

#[cfg(feature = "cloud-s3")]
pub mod s3_reader;

#[cfg(feature = "cloud-gcs")]
pub mod gcs_writer;

#[cfg(feature = "cloud-http")]
pub mod http_writer;

pub mod replicate;

#[cfg(feature = "cloud-s3")]
pub use s3_writer::S3ExcelWriter;

#[cfg(feature = "cloud-s3")]
pub use s3_reader::S3ExcelReader;

#[cfg(feature = "cloud-gcs")]
pub use gcs_writer::GCSExcelWriter;

#[cfg(feature = "cloud-http")]
pub use http_writer::HttpExcelWriter;

use crate::error::Result;
use std::io::Write;

/// Trait for cloud storage backends
///
/// This trait abstracts different cloud storage providers (S3, GCS, Azure)
/// to provide a unified interface for streaming Excel files.
pub trait CloudStorage: Write + Send {
    /// Start a multipart upload session
    fn start_upload(&mut self) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Upload a part of the file
    fn upload_part(
        &mut self,
        upload_id: &str,
        part_number: u32,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Complete the multipart upload
    fn complete_upload(
        &mut self,
        upload_id: &str,
        parts: Vec<(u32, String)>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Abort the upload if something goes wrong
    fn abort_upload(
        &mut self,
        upload_id: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// In-memory buffer for cloud uploads
///
/// Accumulates data until it reaches the minimum part size (5 MB for S3),
/// then triggers an upload.
pub struct CloudBuffer {
    buffer: Vec<u8>,
    part_size: usize,
    uploaded_parts: Vec<(u32, String)>,
    current_part_number: u32,
}

impl CloudBuffer {
    /// Create a new cloud buffer
    ///
    /// # Arguments
    ///
    /// * `part_size` - Minimum size for each part (e.g., 5MB for S3)
    pub fn new(part_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(part_size),
            part_size,
            uploaded_parts: Vec::new(),
            current_part_number: 1,
        }
    }

    /// Check if buffer is full and ready to upload
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.part_size
    }

    /// Get current buffer data
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear buffer after upload
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Record uploaded part
    pub fn add_uploaded_part(&mut self, etag: String) {
        self.uploaded_parts.push((self.current_part_number, etag));
        self.current_part_number += 1;
    }

    /// Get all uploaded parts (for completing multipart upload)
    pub fn uploaded_parts(&self) -> &[(u32, String)] {
        &self.uploaded_parts
    }

    /// Get current part number
    pub fn current_part_number(&self) -> u32 {
        self.current_part_number
    }
}

impl Write for CloudBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
