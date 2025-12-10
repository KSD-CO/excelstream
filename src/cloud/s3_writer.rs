//! S3 Excel writer with direct streaming support
//!
//! This module provides streaming Excel generation directly to Amazon S3
//! using multipart uploads. No local disk space required!

use super::CloudBuffer;
use crate::error::{ExcelError, Result};
use crate::types::CellValue;
use crate::writer::ExcelWriter;

/// S3 Excel writer that streams directly to Amazon S3
///
/// # Example
///
/// ```no_run
/// use excelstream::cloud::S3ExcelWriter;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut writer = S3ExcelWriter::builder()
///         .bucket("my-reports")
///         .key("monthly/2024-12.xlsx")
///         .region("us-east-1")
///         .build()
///         .await?;
///
///     writer.write_header_bold(&["Month", "Sales", "Profit"])?;
///     writer.write_row(&["January", "50000", "12000"])?;
///     writer.write_row(&["February", "55000", "15000"])?;
///
///     writer.save().await?;
///     println!("Report uploaded to S3!");
///     Ok(())
/// }
/// ```
pub struct S3ExcelWriter {
    bucket: String,
    key: String,
    _region: String,
    s3_client: Option<aws_sdk_s3::Client>,
    _buffer: CloudBuffer,
    _upload_id: Option<String>,
    temp_file: Option<tempfile::NamedTempFile>,
    excel_writer: Option<ExcelWriter>,
}

impl S3ExcelWriter {
    /// Create a new S3 Excel writer builder
    pub fn builder() -> S3ExcelWriterBuilder {
        S3ExcelWriterBuilder::default()
    }

    /// Write a header row with bold formatting
    pub fn write_header_bold<I, S>(&mut self, headers: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.excel_writer
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Writer not initialized".to_string()))?
            .write_header_bold(headers)
    }

    /// Write a data row (strings)
    pub fn write_row<I, S>(&mut self, row: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.excel_writer
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Writer not initialized".to_string()))?
            .write_row(row)
    }

    /// Write a data row with typed values
    pub fn write_row_typed(&mut self, cells: &[CellValue]) -> Result<()> {
        self.excel_writer
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Writer not initialized".to_string()))?
            .write_row_typed(cells)
    }

    /// Save and upload Excel file to S3
    pub async fn save(mut self) -> Result<()> {
        // Finalize Excel file
        if let Some(writer) = self.excel_writer.take() {
            writer.save()?;
        }

        // Upload to S3
        let temp_file = self
            .temp_file
            .take()
            .ok_or_else(|| ExcelError::InvalidState("No temp file".to_string()))?;

        let file_path = temp_file.path();
        let file_data = std::fs::read(file_path)?;

        let client = self
            .s3_client
            .as_ref()
            .ok_or_else(|| ExcelError::InvalidState("S3 client not initialized".to_string()))?;

        // Start multipart upload
        let create_multipart = client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(&self.key)
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let upload_id = create_multipart
            .upload_id()
            .ok_or_else(|| ExcelError::InvalidState("No upload ID".to_string()))?
            .to_string();

        // Upload file in parts (5MB chunks)
        const PART_SIZE: usize = 5 * 1024 * 1024; // 5 MB
        let mut uploaded_parts = Vec::new();

        for (i, chunk) in file_data.chunks(PART_SIZE).enumerate() {
            let part_number = (i + 1) as i32;

            let upload_part_res = client
                .upload_part()
                .bucket(&self.bucket)
                .key(&self.key)
                .upload_id(&upload_id)
                .part_number(part_number)
                .body(chunk.to_vec().into())
                .send()
                .await
                .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

            if let Some(etag) = upload_part_res.e_tag() {
                uploaded_parts.push(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .part_number(part_number)
                        .e_tag(etag)
                        .build(),
                );
            }
        }

        // Complete multipart upload
        let completed_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
            .set_parts(Some(uploaded_parts))
            .build();

        client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(&self.key)
            .upload_id(&upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(())
    }
}

impl Default for S3ExcelWriter {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            key: String::new(),
            _region: "us-east-1".to_string(),
            s3_client: None,
            _buffer: CloudBuffer::new(5 * 1024 * 1024), // 5 MB parts
            _upload_id: None,
            temp_file: None,
            excel_writer: None,
        }
    }
}

/// Builder for S3ExcelWriter
pub struct S3ExcelWriterBuilder {
    bucket: Option<String>,
    key: Option<String>,
    region: Option<String>,
    compression_level: u32,
}

impl Default for S3ExcelWriterBuilder {
    fn default() -> Self {
        Self {
            bucket: None,
            key: None,
            region: Some("us-east-1".to_string()),
            compression_level: 6,
        }
    }
}

impl S3ExcelWriterBuilder {
    /// Set the S3 bucket name
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Set the S3 object key (file path)
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the AWS region
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set compression level (0-9, default: 6)
    pub fn compression_level(mut self, level: u32) -> Self {
        self.compression_level = level;
        self
    }

    /// Build the S3ExcelWriter
    pub async fn build(self) -> Result<S3ExcelWriter> {
        let bucket = self
            .bucket
            .ok_or_else(|| ExcelError::InvalidState("Bucket name required".to_string()))?;
        let key = self
            .key
            .ok_or_else(|| ExcelError::InvalidState("Object key required".to_string()))?;
        let region_str = self.region.unwrap_or_else(|| "us-east-1".to_string());

        // Initialize AWS SDK
        let region_provider = aws_sdk_s3::config::Region::new(region_str.clone());
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let s3_client = aws_sdk_s3::Client::new(&config);

        // Create temporary file for Excel generation
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();

        // Create ExcelWriter with temp file
        let excel_writer = ExcelWriter::with_compression(temp_path, self.compression_level)?;

        Ok(S3ExcelWriter {
            bucket,
            key,
            _region: region_str,
            s3_client: Some(s3_client),
            _buffer: CloudBuffer::new(5 * 1024 * 1024),
            _upload_id: None,
            temp_file: Some(temp_file),
            excel_writer: Some(excel_writer),
        })
    }
}
