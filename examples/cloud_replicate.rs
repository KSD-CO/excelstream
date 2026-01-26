//! Cloud-to-Cloud Replicate Example
//!
//! This example demonstrates how to backup/transfer Excel files
//! between different cloud storage services with custom clients.

use excelstream::cloud::replicate::{
    CloudDestination, CloudProvider, CloudReplicate, CloudReplicateBuilder, CloudSource,
    ReplicateConfig,
};

#[cfg(feature = "cloud-s3")]
use aws_config::BehaviorVersion;
#[cfg(feature = "cloud-s3")]
use aws_sdk_s3::config::Region;
#[cfg(feature = "cloud-s3")]
use aws_sdk_s3::Client as S3Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: S3 to S3 replicate (same region) with auto-generated clients
    println!("=== Example 1: S3 to S3 Replicate (Auto-generated clients) ===");
    let source = CloudSource {
        provider: CloudProvider::S3,
        bucket: "production-bucket".to_string(),
        key: "reports/2024-01-report.xlsx".to_string(),
        region: Some("us-east-1".to_string()),
        endpoint_url: None,
    };

    let destination = CloudDestination {
        provider: CloudProvider::S3,
        bucket: "backup-bucket".to_string(),
        key: "backups/2024-01-report-backup.xlsx".to_string(),
        region: Some("us-east-1".to_string()),
        endpoint_url: None,
    };

    let config = ReplicateConfig::new(source, destination)
        .with_chunk_size(10 * 1024 * 1024) // 10MB chunks
        .with_max_retries(3);

    let replicate = CloudReplicate::new(config);
    match replicate.execute().await {
        Ok(stats) => {
            println!("✓ Replicate completed successfully!");
            println!("  - Transferred: {} bytes", stats.bytes_transferred);
            println!("  - Chunks: {}", stats.chunks_transferred);
            println!("  - Speed: {:.2} MB/s", stats.speed_mbps());
            println!("  - Duration: {:.2}s", stats.elapsed().as_secs_f64());
        }
        Err(e) => println!("✗ Replicate failed: {}", e),
    }

    // Example 2: Custom clients with explicit credentials
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 2: S3 to S3 with Custom Clients ===");
        let source_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .load()
            .await;
        let source_client = S3Client::new(&source_config);

        let dest_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .load()
            .await;
        let dest_client = S3Client::new(&dest_config);

        let source = CloudSource {
            provider: CloudProvider::S3,
            bucket: "us-east-data".to_string(),
            key: "critical/database-dump.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
        };

        let destination = CloudDestination {
            provider: CloudProvider::S3,
            bucket: "us-west-backup".to_string(),
            key: "dr/database-dump-backup.xlsx".to_string(),
            region: Some("us-west-2".to_string()),
            endpoint_url: None,
        };

        let config = ReplicateConfig::new(source, destination);
        let replicate = CloudReplicate::with_clients(config, source_client, dest_client);

        match replicate.execute().await {
            Ok(stats) => {
                println!("✓ Cross-region replicate with custom clients completed!");
                println!("  - Transferred: {} bytes", stats.bytes_transferred);
                println!("  - Speed: {:.2} MB/s", stats.speed_mbps());
            }
            Err(e) => println!("✗ Replicate failed: {}", e),
        }
    }

    // Example 3: Builder pattern with custom clients
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 3: Builder Pattern with Custom Clients ===");
        let config_val = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .load()
            .await;
        let source_client = S3Client::new(&config_val);

        let source = CloudSource {
            provider: CloudProvider::S3,
            bucket: "prod-bucket".to_string(),
            key: "export.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
        };

        let destination = CloudDestination {
            provider: CloudProvider::S3,
            bucket: "archive-bucket".to_string(),
            key: "exports/export-v1.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: None,
        };

        let config = ReplicateConfig::new(source, destination).with_chunk_size(5 * 1024 * 1024);

        let replicate = CloudReplicateBuilder::new()
            .config(config)
            .source_client(source_client)
            .build()?;

        match replicate.execute().await {
            Ok(stats) => {
                println!("✓ Builder replicate completed!");
                println!("  - Transferred: {} bytes", stats.bytes_transferred);
            }
            Err(e) => println!("✗ Replicate failed: {}", e),
        }
    }

    // Example 4: MinIO replicate (S3-compatible) with custom client & different secrets
    println!("\n=== Example 4: MinIO (S3-Compatible) with Different Secrets ===");

    #[cfg(feature = "cloud-s3")]
    {
        use aws_sdk_s3::config::Credentials;
        use aws_sdk_s3::config::Region;

        // Setup source MinIO client with credentials
        let source_creds = Credentials::new(
            "source-access-key", // Different from destination
            "source-secret-key",
            None,
            None,
            "minio-source",
        );

        let source_config = aws_sdk_s3::Config::builder()
            .region(Region::new("us-east-1"))
            .credentials_provider(source_creds)
            .endpoint_url("http://localhost:9000")
            .build();

        let source_client = S3Client::from_conf(source_config);

        // Setup destination MinIO client with different credentials
        let dest_creds = Credentials::new(
            "dest-access-key", // Different from source!
            "dest-secret-key",
            None,
            None,
            "minio-dest",
        );

        let dest_config = aws_sdk_s3::Config::builder()
            .region(Region::new("us-east-1"))
            .credentials_provider(dest_creds)
            .endpoint_url("http://backup-minio:9000")
            .build();

        let dest_client = S3Client::from_conf(dest_config);

        let source = CloudSource {
            provider: CloudProvider::S3,
            bucket: "reports".to_string(),
            key: "monthly/2024-01.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: Some("http://localhost:9000".to_string()),
        };

        let destination = CloudDestination {
            provider: CloudProvider::S3,
            bucket: "reports-backup".to_string(),
            key: "archive/2024-01-backup.xlsx".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint_url: Some("http://backup-minio:9000".to_string()),
        };

        let config = ReplicateConfig::new(source, destination).with_chunk_size(5 * 1024 * 1024);

        // Use builder with custom clients (different secrets)
        let replicate = CloudReplicateBuilder::new()
            .config(config)
            .source_client(source_client)
            .dest_client(dest_client)
            .build()?;

        match replicate.execute().await {
            Ok(stats) => {
                println!("✓ MinIO replicate with different secrets completed!");
                println!("  - Source: localhost:9000 (source-access-key)");
                println!("  - Dest: backup-minio:9000 (dest-access-key)");
                println!("  - Transferred: {} bytes", stats.bytes_transferred);
                println!("  - Speed: {:.2} MB/s", stats.speed_mbps());
            }
            Err(e) => println!("✗ MinIO replicate failed: {}", e),
        }
    }

    #[cfg(not(feature = "cloud-s3"))]
    {
        println!("  (Skipped - cloud-s3 feature not enabled)");
    }

    // Example 5: DigitalOcean Spaces backup
    println!("\n=== Example 5: DigitalOcean Spaces Replicate ===");
    let source = CloudSource {
        provider: CloudProvider::S3,
        bucket: "do-spaces-bucket".to_string(),
        key: "data/export.xlsx".to_string(),
        region: Some("nyc3".to_string()),
        endpoint_url: Some("https://nyc3.digitaloceanspaces.com".to_string()),
    };

    let destination = CloudDestination {
        provider: CloudProvider::S3,
        bucket: "do-backup-bucket".to_string(),
        key: "backups/export-backup.xlsx".to_string(),
        region: Some("sfo3".to_string()),
        endpoint_url: Some("https://sfo3.digitaloceanspaces.com".to_string()),
    };

    let config = ReplicateConfig::new(source, destination);

    let replicate = CloudReplicate::new(config);
    match replicate.execute().await {
        Ok(stats) => {
            println!("✓ DigitalOcean Spaces replicate completed!");
            println!("  - Transferred: {} bytes", stats.bytes_transferred);
        }
        Err(e) => println!("✗ DigitalOcean Spaces replicate failed: {}", e),
    }

    Ok(())
}
