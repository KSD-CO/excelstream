//! S3 Excel reader with direct streaming support
//!
//! This module provides reading Excel files directly from Amazon S3
//! by downloading to a temporary file and using StreamingReader for parsing.

use crate::error::{ExcelError, Result};
use crate::streaming_reader::{RowIterator, RowStructIterator, StreamingReader};
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::Client;
use std::io::Write;
use tokio::io::AsyncReadExt;

/// S3 Excel reader that downloads from Amazon S3 and streams rows
///
/// # Architecture
///
/// Downloads file from S3 to a temporary file, then uses StreamingReader
/// for efficient row-by-row processing. Temp file is automatically cleaned
/// up when S3ExcelReader is dropped.
///
/// # Memory Usage
///
/// - Temp file: Full file size (local disk)
/// - SST: 3-5 MB (in memory)
/// - Per-row processing: ~100 KB
///
/// # Example
///
/// ```no_run
/// use excelstream::cloud::S3ExcelReader;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut reader = S3ExcelReader::builder()
///         .bucket("my-data-bucket")
///         .key("monthly-report.xlsx")
///         .region("us-east-1")
///         .build()
///         .await?;
///
///     println!("Sheets: {:?}", reader.sheet_names());
///
///     for row in reader.rows("Sheet1")? {
///         let row = row?;
///         println!("Row {}: {:?}", row.index, row.to_strings());
///     }
///
///     Ok(())
/// }
/// ```
pub struct S3ExcelReader {
    bucket: String,
    key: String,
    _region: String,
    _s3_client: Option<Client>,
    _temp_file: Option<tempfile::NamedTempFile>,
    streaming_reader: Option<StreamingReader>,
}

impl std::fmt::Debug for S3ExcelReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3ExcelReader")
            .field("bucket", &self.bucket)
            .field("key", &self.key)
            .field("region", &self._region)
            .field("has_s3_client", &self._s3_client.is_some())
            .field("has_temp_file", &self._temp_file.is_some())
            .field("has_streaming_reader", &self.streaming_reader.is_some())
            .finish()
    }
}

impl S3ExcelReader {
    /// Create a new S3 Excel reader builder
    ///
    /// # Example
    /// ```no_run
    /// use excelstream::cloud::S3ExcelReader;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let reader = S3ExcelReader::builder()
    ///     .bucket("my-bucket")
    ///     .key("data.xlsx")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> S3ExcelReaderBuilder {
        S3ExcelReaderBuilder::default()
    }

    /// Get list of sheet names
    ///
    /// Returns the names of all worksheets in the workbook.
    ///
    /// # Example
    /// ```no_run
    /// # use excelstream::cloud::S3ExcelReader;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let reader = S3ExcelReader::builder()
    ///     .bucket("my-bucket")
    ///     .key("data.xlsx")
    ///     .build()
    ///     .await?;
    ///
    /// for sheet_name in reader.sheet_names() {
    ///     println!("Sheet: {}", sheet_name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn sheet_names(&self) -> Vec<String> {
        self.streaming_reader
            .as_ref()
            .map(|r| r.sheet_names())
            .unwrap_or_default()
    }

    /// Stream rows from a worksheet (returns Row structs)
    ///
    /// This is the primary method for reading data. Returns an iterator
    /// of Row structs that match the ExcelReader API for compatibility.
    ///
    /// # Arguments
    /// * `sheet_name` - Name of the worksheet to read
    ///
    /// # Returns
    /// Iterator of Row structs (RowStructIterator)
    ///
    /// # Example
    /// ```no_run
    /// # use excelstream::cloud::S3ExcelReader;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut reader = S3ExcelReader::builder()
    ///     .bucket("my-bucket")
    ///     .key("sales.xlsx")
    ///     .build()
    ///     .await?;
    ///
    /// for row in reader.rows("Sheet1")? {
    ///     let row = row?;
    ///     println!("Row {}: {:?}", row.index, row.to_strings());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn rows(&mut self, sheet_name: &str) -> Result<RowStructIterator<'_>> {
        self.streaming_reader
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Reader not initialized".to_string()))?
            .rows(sheet_name)
    }

    /// Stream rows by sheet index (for backward compatibility)
    ///
    /// # Arguments
    /// * `sheet_index` - Zero-based sheet index (0 = first sheet)
    pub fn rows_by_index(&mut self, sheet_index: usize) -> Result<RowStructIterator<'_>> {
        self.streaming_reader
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Reader not initialized".to_string()))?
            .rows_by_index(sheet_index)
    }

    /// Stream rows from a worksheet (returns Vec<String>)
    ///
    /// Alternative method that returns raw Vec<String> per row.
    /// Useful when you don't need the Row wrapper.
    ///
    /// # Example
    /// ```no_run
    /// # use excelstream::cloud::S3ExcelReader;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut reader = S3ExcelReader::builder()
    ///     .bucket("my-bucket")
    ///     .key("data.xlsx")
    ///     .build()
    ///     .await?;
    ///
    /// for row_vec in reader.stream_rows("Sheet1")? {
    ///     let row_vec = row_vec?;
    ///     println!("{:?}", row_vec);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_rows(&mut self, sheet_name: &str) -> Result<RowIterator<'_>> {
        self.streaming_reader
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Reader not initialized".to_string()))?
            .stream_rows(sheet_name)
    }

    /// Get worksheet dimensions (rows, columns)
    ///
    /// # Note
    /// This reads all rows to count them, which can be slow for large files.
    /// Consider using the row iterator instead if you just need to process data.
    pub fn dimensions(&mut self, sheet_name: &str) -> Result<(usize, usize)> {
        self.streaming_reader
            .as_mut()
            .ok_or_else(|| ExcelError::InvalidState("Reader not initialized".to_string()))?
            .dimensions(sheet_name)
    }

    /// Get S3 bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Get S3 object key
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl Default for S3ExcelReader {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            key: String::new(),
            _region: "us-east-1".to_string(),
            _s3_client: None,
            _temp_file: None,
            streaming_reader: None,
        }
    }
}

/// Builder for S3ExcelReader
///
/// Supports AWS S3 and S3-compatible services (MinIO, Cloudflare R2, DigitalOcean Spaces, etc.)
pub struct S3ExcelReaderBuilder {
    bucket: Option<String>,
    key: Option<String>,
    region: Option<String>,
    endpoint_url: Option<String>,
    force_path_style: bool,
}

impl Default for S3ExcelReaderBuilder {
    fn default() -> Self {
        Self {
            bucket: None,
            key: None,
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
            force_path_style: false,
        }
    }
}

impl S3ExcelReaderBuilder {
    /// Set the S3 bucket name
    ///
    /// # Example
    /// ```no_run
    /// use excelstream::cloud::S3ExcelReader;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let reader = S3ExcelReader::builder()
    ///     .bucket("my-data-bucket")
    ///     .key("reports/data.xlsx")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Set the S3 object key (file path)
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the AWS region (defaults to us-east-1)
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set custom endpoint URL for S3-compatible services
    ///
    /// # Supported Services
    ///
    /// - MinIO: `http://localhost:9000`
    /// - Cloudflare R2: `https://<account_id>.r2.cloudflarestorage.com`
    /// - DigitalOcean Spaces: `https://nyc3.digitaloceanspaces.com`
    pub fn endpoint_url(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint_url = Some(endpoint.into());
        self
    }

    /// Force path-style addressing (required for MinIO and some S3-compatible services)
    ///
    /// When enabled, uses `http://endpoint/bucket/key` instead of `http://bucket.endpoint/key`
    pub fn force_path_style(mut self, force: bool) -> Self {
        self.force_path_style = force;
        self
    }

    /// Build the S3ExcelReader
    ///
    /// # Process
    /// 1. Validate bucket + key
    /// 2. Initialize AWS SDK client
    /// 3. Download file from S3 to temp file
    /// 4. Open StreamingReader from temp file
    /// 5. Return S3ExcelReader wrapper
    ///
    /// # Errors
    /// - Missing bucket or key
    /// - S3 access errors (NoSuchKey, AccessDenied, etc.)
    /// - Network errors
    /// - Invalid Excel file format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use excelstream::cloud::S3ExcelReader;
    ///
    /// // AWS S3
    /// let reader = S3ExcelReader::builder()
    ///     .bucket("my-bucket")
    ///     .key("data.xlsx")
    ///     .region("us-east-1")
    ///     .build()
    ///     .await?;
    ///
    /// // MinIO
    /// let reader = S3ExcelReader::builder()
    ///     .endpoint_url("http://localhost:9000")
    ///     .bucket("my-bucket")
    ///     .key("data.xlsx")
    ///     .region("us-east-1")
    ///     .force_path_style(true)
    ///     .build()
    ///     .await?;
    /// ```
    pub async fn build(self) -> Result<S3ExcelReader> {
        let bucket = self
            .bucket
            .ok_or_else(|| ExcelError::InvalidState("Bucket name required".to_string()))?;

        let key = self
            .key
            .ok_or_else(|| ExcelError::InvalidState("Object key required".to_string()))?;

        let region_str = self.region.unwrap_or_else(|| "us-east-1".to_string());

        let region_provider = aws_sdk_s3::config::Region::new(region_str.clone());
        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);

        if let Some(endpoint) = &self.endpoint_url {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
        }

        if self.force_path_style {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let s3_client = Client::from_conf(s3_config_builder.build());

        Self::build_reader_from_client(s3_client, bucket, key, region_str).await
    }

    /// Build S3ExcelReader with a custom pre-configured AWS S3 client
    pub async fn build_with_client(self, s3_client: Client) -> Result<S3ExcelReader> {
        let bucket = self
            .bucket
            .ok_or_else(|| ExcelError::InvalidState("Bucket name required".to_string()))?;

        let key = self
            .key
            .ok_or_else(|| ExcelError::InvalidState("Object key required".to_string()))?;

        let region_str = self.region.unwrap_or_else(|| "us-east-1".to_string());

        Self::build_reader_from_client(s3_client, bucket, key, region_str).await
    }

    async fn download_from_s3(
        client: &Client,
        bucket: &str,
        key: &str,
    ) -> Result<aws_sdk_s3::operation::get_object::GetObjectOutput> {
        client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let error_code = e.code().unwrap_or("");
                let error_message = e.message().unwrap_or("Unknown error");

                match error_code {
                    "NoSuchKey" => ExcelError::FileNotFound(format!("s3://{}/{}", bucket, key)),
                    "NoSuchBucket" => {
                        ExcelError::ReadError(format!("Bucket '{}' does not exist", bucket))
                    }
                    "AccessDenied" => ExcelError::ReadError(format!(
                        "Access denied to s3://{}/{}. Error: {}",
                        bucket, key, error_message
                    )),
                    _ => ExcelError::ReadError(format!(
                        "S3 GetObject failed ({}): {}",
                        error_code, error_message
                    )),
                }
            })
    }

    async fn create_reader_from_s3_response(
        get_object_output: aws_sdk_s3::operation::get_object::GetObjectOutput,
    ) -> Result<(tempfile::NamedTempFile, StreamingReader)> {
        let mut body = get_object_output.body.into_async_read();
        let mut buffer = Vec::new();

        body.read_to_end(&mut buffer)
            .await
            .map_err(ExcelError::IoError)?;

        let mut temp_file = tempfile::NamedTempFile::new().map_err(ExcelError::IoError)?;

        temp_file.write_all(&buffer).map_err(ExcelError::IoError)?;
        temp_file.flush().map_err(ExcelError::IoError)?;

        let temp_path = temp_file.path().to_path_buf();
        let streaming_reader = StreamingReader::open(&temp_path)?;

        Ok((temp_file, streaming_reader))
    }

    async fn build_reader_from_client(
        s3_client: Client,
        bucket: String,
        key: String,
        region_str: String,
    ) -> Result<S3ExcelReader> {
        let get_object_output = Self::download_from_s3(&s3_client, &bucket, &key).await?;
        let (temp_file, streaming_reader) =
            Self::create_reader_from_s3_response(get_object_output).await?;

        Ok(S3ExcelReader {
            bucket,
            key,
            _region: region_str,
            _s3_client: Some(s3_client),
            _temp_file: Some(temp_file),
            streaming_reader: Some(streaming_reader),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_config::BehaviorVersion;
    use aws_sdk_s3::config::Region;

    #[test]
    fn test_builder_validation_missing_bucket() {
        let builder = S3ExcelReaderBuilder::default().key("test.xlsx");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(builder.build());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Bucket name required"));
    }

    #[test]
    fn test_builder_validation_missing_key() {
        let builder = S3ExcelReaderBuilder::default().bucket("test-bucket");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(builder.build());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Object key required"));
    }

    #[test]
    fn test_default_region() {
        let builder = S3ExcelReaderBuilder::default();
        assert_eq!(builder.region, Some("us-east-1".to_string()));
    }

    #[test]
    fn test_builder_methods() {
        let builder = S3ExcelReaderBuilder::default()
            .bucket("my-bucket")
            .key("path/to/file.xlsx")
            .region("ap-southeast-1");

        assert_eq!(builder.bucket, Some("my-bucket".to_string()));
        assert_eq!(builder.key, Some("path/to/file.xlsx".to_string()));
        assert_eq!(builder.region, Some("ap-southeast-1".to_string()));
    }

    #[tokio::test]
    async fn test_build_with_client() {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .load()
            .await;
        let client = Client::new(&config);

        let result = S3ExcelReaderBuilder::default()
            .bucket("test-bucket")
            .key("test.xlsx")
            .build_with_client(client)
            .await;

        assert!(result.is_err());
    }
}
