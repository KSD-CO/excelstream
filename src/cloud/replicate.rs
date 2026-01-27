//! Cloud-to-Cloud Replicate & Transfer
//!
//! Transfer files between different cloud storage services (S3, GCS, etc.)
//! without downloading to local disk. Perfect for replicate and disaster recovery.

use crate::error::{ExcelError, Result};
use std::sync::Arc;

#[cfg(feature = "cloud-s3")]
use aws_sdk_s3::Client as S3Client;

/// Cloud storage backend type
#[derive(Debug, Clone)]
pub enum CloudProvider {
    /// Amazon S3
    S3,
    /// Google Cloud Storage
    GCS,
}

/// Source cloud configuration
#[derive(Clone, Debug)]
pub struct CloudSource {
    pub provider: CloudProvider,
    pub bucket: String,
    pub key: String,
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
}

/// Destination cloud configuration
#[derive(Clone, Debug)]
pub struct CloudDestination {
    pub provider: CloudProvider,
    pub bucket: String,
    pub key: String,
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
}

/// Cloud-to-Cloud replicate configuration
#[derive(Debug)]
pub struct ReplicateConfig {
    pub source: CloudSource,
    pub destination: CloudDestination,
    pub chunk_size: usize, // Default: 5MB
    pub max_retries: u32,
}

impl ReplicateConfig {
    /// Create new replicate configuration
    pub fn new(source: CloudSource, destination: CloudDestination) -> Self {
        Self {
            source,
            destination,
            chunk_size: 5 * 1024 * 1024, // 5MB
            max_retries: 3,
        }
    }

    /// Set custom chunk size for transfer
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set maximum retries for failed chunks
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

/// Replicate status and statistics
#[derive(Debug, Clone)]
pub struct ReplicateStats {
    pub bytes_transferred: u64,
    pub chunks_transferred: u32,
    pub start_time: std::time::Instant,
    pub errors: Vec<String>,
}

impl ReplicateStats {
    /// Get transfer speed in MB/s
    pub fn speed_mbps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            0.0
        } else {
            (self.bytes_transferred as f64 / (1024.0 * 1024.0)) / elapsed
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Cloud-to-Cloud replicate handler
#[derive(Debug)]
pub struct CloudReplicate {
    config: ReplicateConfig,
    stats: Arc<tokio::sync::Mutex<ReplicateStats>>,
    #[cfg(feature = "cloud-s3")]
    source_client: Option<Arc<S3Client>>,
    #[cfg(feature = "cloud-s3")]
    dest_client: Option<Arc<S3Client>>,
}

impl CloudReplicate {
    /// Create new replicate handler with auto-generated clients
    pub fn new(config: ReplicateConfig) -> Self {
        Self {
            config,
            stats: Arc::new(tokio::sync::Mutex::new(ReplicateStats {
                bytes_transferred: 0,
                chunks_transferred: 0,
                start_time: std::time::Instant::now(),
                errors: Vec::new(),
            })),
            #[cfg(feature = "cloud-s3")]
            source_client: None,
            #[cfg(feature = "cloud-s3")]
            dest_client: None,
        }
    }

    /// Create a new builder for replicate with custom clients
    pub fn builder() -> CloudReplicateBuilder {
        CloudReplicateBuilder::default()
    }

    /// Create replicate with pre-configured source and destination clients
    #[cfg(feature = "cloud-s3")]
    pub fn with_clients(
        config: ReplicateConfig,
        source_client: S3Client,
        dest_client: S3Client,
    ) -> Self {
        Self {
            config,
            stats: Arc::new(tokio::sync::Mutex::new(ReplicateStats {
                bytes_transferred: 0,
                chunks_transferred: 0,
                start_time: std::time::Instant::now(),
                errors: Vec::new(),
            })),
            source_client: Some(Arc::new(source_client)),
            dest_client: Some(Arc::new(dest_client)),
        }
    }

    /// Create replicate with only source client (destination uses auto-generated)
    #[cfg(feature = "cloud-s3")]
    pub fn with_source_client(config: ReplicateConfig, source_client: S3Client) -> Self {
        Self {
            config,
            stats: Arc::new(tokio::sync::Mutex::new(ReplicateStats {
                bytes_transferred: 0,
                chunks_transferred: 0,
                start_time: std::time::Instant::now(),
                errors: Vec::new(),
            })),
            source_client: Some(Arc::new(source_client)),
            dest_client: None,
        }
    }

    /// Create replicate with only destination client (source uses auto-generated)
    #[cfg(feature = "cloud-s3")]
    pub fn with_dest_client(config: ReplicateConfig, dest_client: S3Client) -> Self {
        Self {
            config,
            stats: Arc::new(tokio::sync::Mutex::new(ReplicateStats {
                bytes_transferred: 0,
                chunks_transferred: 0,
                start_time: std::time::Instant::now(),
                errors: Vec::new(),
            })),
            source_client: None,
            dest_client: Some(Arc::new(dest_client)),
        }
    }

    /// Execute backup/transfer between clouds
    ///
    /// # Example
    ///
    /// ```no_run
    /// use excelstream::cloud::replicate::{CloudReplicate, ReplicateConfig, CloudSource, CloudDestination, CloudProvider};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let source = CloudSource {
    ///         provider: CloudProvider::S3,
    ///         bucket: "source-bucket".to_string(),
    ///         key: "path/to/file.xlsx".to_string(),
    ///         region: Some("us-east-1".to_string()),
    ///         endpoint_url: None,
    ///     };
    ///
    ///     let destination = CloudDestination {
    ///         provider: CloudProvider::S3,
    ///         bucket: "backup-bucket".to_string(),
    ///         key: "backups/file-2024.xlsx".to_string(),
    ///         region: Some("us-west-2".to_string()),
    ///         endpoint_url: None,
    ///     };
    ///
    ///     let config = ReplicateConfig::new(source, destination)
    ///         .with_chunk_size(10 * 1024 * 1024); // 10MB chunks
    ///
    ///     let replicate = CloudReplicate::new(config);
    ///     let stats = replicate.execute().await?;
    ///
    ///     println!("Transferred: {} bytes", stats.bytes_transferred);
    ///     println!("Speed: {:.2} MB/s", stats.speed_mbps());
    ///     Ok(())
    /// }
    /// ```
    /// ```
    #[cfg(feature = "cloud-s3")]
    pub async fn execute(&self) -> Result<ReplicateStats> {
        #[allow(unreachable_patterns)]
        match (
            &self.config.source.provider,
            &self.config.destination.provider,
        ) {
            (CloudProvider::S3, CloudProvider::S3) => self.s3_to_s3().await,
            #[cfg(feature = "cloud-gcs")]
            (CloudProvider::S3, CloudProvider::GCS) => self.s3_to_gcs().await,
            #[cfg(feature = "cloud-gcs")]
            (CloudProvider::GCS, CloudProvider::S3) => self.gcs_to_s3().await,
            #[cfg(feature = "cloud-gcs")]
            (CloudProvider::GCS, CloudProvider::GCS) => self.gcs_to_gcs().await,
            _ => Err(ExcelError::InvalidState(
                "Unsupported cloud provider combination".to_string(),
            )),
        }
    }

    #[cfg(feature = "cloud-s3")]
    async fn s3_to_s3(&self) -> Result<ReplicateStats> {
        let source_region = self
            .config
            .source
            .region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());
        let dest_region = self
            .config
            .destination
            .region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());

        // Get or create source client
        let source_client = if let Some(client) = &self.source_client {
            client.as_ref().clone()
        } else {
            let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_sdk_s3::config::Region::new(source_region.clone()))
                .load()
                .await;
            let mut builder = aws_sdk_s3::config::Builder::from(&config);

            if let Some(endpoint) = &self.config.source.endpoint_url {
                builder = builder.endpoint_url(endpoint);
            }

            S3Client::from_conf(builder.build())
        };

        // Get or create destination client
        let dest_client = if let Some(client) = &self.dest_client {
            client.as_ref().clone()
        } else {
            let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_sdk_s3::config::Region::new(dest_region.clone()))
                .load()
                .await;
            let mut builder = aws_sdk_s3::config::Builder::from(&config);

            if let Some(endpoint) = &self.config.destination.endpoint_url {
                builder = builder.endpoint_url(endpoint);
            }

            S3Client::from_conf(builder.build())
        };

        // Check if same region - can use native copy_object (zero memory!)
        if source_region == dest_region
            && self.config.source.endpoint_url == self.config.destination.endpoint_url
        {
            return self.s3_copy_object_native(&source_client).await;
        }

        // Different region/endpoint - use streaming copy (constant memory!)
        self.s3_copy_streaming(&source_client, &dest_client).await
    }

    #[cfg(feature = "cloud-s3")]
    async fn s3_copy_object_native(&self, client: &S3Client) -> Result<ReplicateStats> {
        // Use native copy_object API - ZERO memory, server-side copy!
        let copy_source = format!("{}/{}", &self.config.source.bucket, &self.config.source.key);

        let start = std::time::Instant::now();

        client
            .copy_object()
            .copy_source(copy_source)
            .bucket(&self.config.destination.bucket)
            .key(&self.config.destination.key)
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        // Get actual size from destination
        let head = client
            .head_object()
            .bucket(&self.config.destination.bucket)
            .key(&self.config.destination.key)
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let actual_size = head.content_length().unwrap_or(0) as u64;

        Ok(ReplicateStats {
            bytes_transferred: actual_size,
            chunks_transferred: 1,
            start_time: start,
            errors: vec![],
        })
    }

    #[cfg(feature = "cloud-s3")]
    async fn s3_copy_streaming(
        &self,
        source_client: &S3Client,
        dest_client: &S3Client,
    ) -> Result<ReplicateStats> {
        // Get source object metadata
        let head_response = source_client
            .head_object()
            .bucket(&self.config.source.bucket)
            .key(&self.config.source.key)
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let file_size = head_response.content_length().unwrap_or(0) as u64;

        // Initiate multipart upload
        let multipart = dest_client
            .create_multipart_upload()
            .bucket(&self.config.destination.bucket)
            .key(&self.config.destination.key)
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let upload_id = multipart
            .upload_id()
            .ok_or_else(|| ExcelError::InvalidState("No upload ID".to_string()))?
            .to_string();

        let mut parts = Vec::new();
        let mut offset = 0;
        let mut part_number = 1u32;

        // Stream chunks without collecting entire buffer into memory
        while offset < file_size {
            let chunk_size = (self.config.chunk_size as u64).min(file_size - offset);
            let range = format!("bytes={}-{}", offset, offset + chunk_size - 1);

            // Get chunk stream from source
            let response = source_client
                .get_object()
                .bucket(&self.config.source.bucket)
                .key(&self.config.source.key)
                .range(&range)
                .send()
                .await
                .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

            // Collect ByteStream into Vec<u8> for proper checksum calculation
            // This is necessary for S3-compatible services (MinIO, FPT Cloud, etc.)
            // that strictly validate x-amz-content-sha256 header
            let byte_stream = response.body;
            let chunk_bytes = byte_stream
                .collect()
                .await
                .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?
                .into_bytes();

            // Upload chunk - SDK can now calculate checksum properly
            let part_response = dest_client
                .upload_part()
                .bucket(&self.config.destination.bucket)
                .key(&self.config.destination.key)
                .upload_id(&upload_id)
                .part_number(part_number as i32)
                .body(chunk_bytes.to_vec().into())
                .send()
                .await
                .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

            if let Some(etag) = part_response.e_tag() {
                parts.push(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .e_tag(etag)
                        .part_number(part_number as i32)
                        .build(),
                );
            }

            let mut stats = self.stats.lock().await;
            stats.bytes_transferred += chunk_size;
            stats.chunks_transferred += 1;

            offset += chunk_size;
            part_number += 1;
        }

        // Complete multipart upload
        dest_client
            .complete_multipart_upload()
            .bucket(&self.config.destination.bucket)
            .key(&self.config.destination.key)
            .upload_id(&upload_id)
            .multipart_upload(
                aws_sdk_s3::types::CompletedMultipartUpload::builder()
                    .set_parts(Some(parts))
                    .build(),
            )
            .send()
            .await
            .map_err(|e| ExcelError::IoError(std::io::Error::other(e.to_string())))?;

        let stats = self.stats.lock().await;
        Ok(ReplicateStats {
            bytes_transferred: stats.bytes_transferred,
            chunks_transferred: stats.chunks_transferred,
            start_time: stats.start_time,
            errors: stats.errors.clone(),
        })
    }

    #[cfg(not(feature = "cloud-s3"))]
    pub async fn execute(&self) -> Result<ReplicateStats> {
        Err(ExcelError::InvalidState(
            "cloud-s3 feature not enabled".to_string(),
        ))
    }

    #[cfg(feature = "cloud-gcs")]
    async fn s3_to_gcs(&self) -> Result<ReplicateStats> {
        Err(ExcelError::InvalidState(
            "S3 to GCS transfer not yet implemented".to_string(),
        ))
    }

    #[cfg(feature = "cloud-gcs")]
    async fn gcs_to_s3(&self) -> Result<ReplicateStats> {
        Err(ExcelError::InvalidState(
            "GCS to S3 transfer not yet implemented".to_string(),
        ))
    }

    #[cfg(feature = "cloud-gcs")]
    async fn gcs_to_gcs(&self) -> Result<ReplicateStats> {
        Err(ExcelError::InvalidState(
            "GCS to GCS transfer not yet implemented".to_string(),
        ))
    }
}

/// Builder for CloudReplicate with custom client support
#[derive(Default)]
pub struct CloudReplicateBuilder {
    config: Option<ReplicateConfig>,
    #[cfg(feature = "cloud-s3")]
    source_client: Option<S3Client>,
    #[cfg(feature = "cloud-s3")]
    dest_client: Option<S3Client>,
}

impl CloudReplicateBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the replicate configuration
    pub fn config(mut self, config: ReplicateConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set custom source S3 client
    #[cfg(feature = "cloud-s3")]
    pub fn source_client(mut self, client: S3Client) -> Self {
        self.source_client = Some(client);
        self
    }

    /// Set custom destination S3 client
    #[cfg(feature = "cloud-s3")]
    pub fn dest_client(mut self, client: S3Client) -> Self {
        self.dest_client = Some(client);
        self
    }

    /// Build CloudReplicate instance
    pub fn build(self) -> Result<CloudReplicate> {
        let config = self
            .config
            .ok_or_else(|| ExcelError::InvalidState("ReplicateConfig required".to_string()))?;

        #[cfg(feature = "cloud-s3")]
        {
            match (self.source_client, self.dest_client) {
                (Some(src), Some(dst)) => Ok(CloudReplicate::with_clients(config, src, dst)),
                (Some(src), None) => Ok(CloudReplicate::with_source_client(config, src)),
                (None, Some(dst)) => Ok(CloudReplicate::with_dest_client(config, dst)),
                (None, None) => Ok(CloudReplicate::new(config)),
            }
        }

        #[cfg(not(feature = "cloud-s3"))]
        Ok(CloudReplicate::new(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_config_creation() {
        let source = CloudSource {
            provider: CloudProvider::S3,
            bucket: "source".to_string(),
            key: "file.xlsx".to_string(),
            region: None,
            endpoint_url: None,
        };

        let dest = CloudDestination {
            provider: CloudProvider::S3,
            bucket: "dest".to_string(),
            key: "backup/file.xlsx".to_string(),
            region: None,
            endpoint_url: None,
        };

        let config = ReplicateConfig::new(source, dest);
        assert_eq!(config.chunk_size, 5 * 1024 * 1024);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_backup_stats() {
        let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(100));

        let stats = ReplicateStats {
            bytes_transferred: 1024 * 1024, // 1 MB
            chunks_transferred: 1,
            start_time: start,
            errors: vec![],
        };

        let speed = stats.speed_mbps();
        assert!(speed > 0.0, "Speed should be positive");
        assert!(speed < 100.0, "Speed should be reasonable");
    }

    #[test]
    fn test_backup_config_with_options() {
        let source = CloudSource {
            provider: CloudProvider::S3,
            bucket: "source".to_string(),
            key: "file.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
        };

        let dest = CloudDestination {
            provider: CloudProvider::S3,
            bucket: "dest".to_string(),
            key: "backup/file.xlsx".to_string(),
            region: Some("us-west-2".to_string()),
            endpoint_url: None,
        };

        let config = ReplicateConfig::new(source, dest)
            .with_chunk_size(10 * 1024 * 1024)
            .with_max_retries(5);

        assert_eq!(config.chunk_size, 10 * 1024 * 1024);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_builder_without_clients() {
        let source = CloudSource {
            provider: CloudProvider::S3,
            bucket: "source".to_string(),
            key: "file.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
        };

        let dest = CloudDestination {
            provider: CloudProvider::S3,
            bucket: "dest".to_string(),
            key: "backup/file.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
        };

        let config = ReplicateConfig::new(source, dest);
        let result = CloudReplicateBuilder::new().config(config).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_without_config() {
        let result = CloudReplicateBuilder::new().build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ReplicateConfig required"));
    }
}
