//! S3 Excel Writer Example
//!
//! This example demonstrates how to stream Excel files directly to S3
//! with custom clients, different credentials, and S3-compatible services.

use excelstream::cloud::S3ExcelWriter;
use excelstream::types::CellValue;

#[cfg(feature = "cloud-s3")]
use aws_config::BehaviorVersion;
#[cfg(feature = "cloud-s3")]
use aws_sdk_s3::config::{Credentials, Region};
#[cfg(feature = "cloud-s3")]
use aws_sdk_s3::Client as S3Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic S3 write (auto-generated client from env vars)
    println!("=== Example 1: Basic S3 Write ===");
    let mut writer = S3ExcelWriter::builder()
        .bucket("my-reports")
        .key("monthly/sales-2024-01.xlsx")
        .region("us-east-1")
        .build()
        .await?;

    writer
        .write_header_bold(&["Month", "Sales", "Profit"])
        .await?;
    writer.write_row(&["January", "50000", "12000"]).await?;
    writer.write_row(&["February", "55000", "15000"]).await?;
    writer.write_row(&["March", "60000", "18000"]).await?;

    writer.save().await?;
    println!("✓ Basic S3 write completed!");

    // Example 2: Write with custom S3 client
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 2: S3 Write with Custom Client ===");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-west-2"))
            .load()
            .await;
        let client = S3Client::new(&config);

        let mut writer = S3ExcelWriter::builder()
            .bucket("data-warehouse")
            .key("exports/quarterly-report-q1-2024.xlsx")
            .region("us-west-2")
            .build_with_client(client)
            .await?;

        writer
            .write_header_bold(&["Quarter", "Revenue", "Expenses", "Net Income"])
            .await?;
        writer
            .write_row(&["Q1", "1000000", "750000", "250000"])
            .await?;
        writer
            .write_row(&["Q2", "1100000", "800000", "300000"])
            .await?;

        writer.save().await?;
        println!("✓ Custom client S3 write completed!");
    }

    // Example 3: Write with custom credentials
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 3: S3 Write with Custom Credentials ===");
        let creds = Credentials::new(
            "AKIAIOSFODNN7EXAMPLE",                     // Access Key
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY", // Secret Key
            None,
            None,
            "explicit",
        );

        let config = aws_sdk_s3::Config::builder()
            .region(Region::new("eu-west-1"))
            .credentials_provider(creds)
            .build();

        let client = S3Client::from_conf(config);

        let mut writer = S3ExcelWriter::builder()
            .bucket("eu-reports")
            .key("archive/2024-q1-sales.xlsx")
            .region("eu-west-1")
            .build_with_client(client)
            .await?;

        writer
            .write_header_bold(&["Country", "Region", "Sales"])
            .await?;
        writer.write_row(&["Germany", "Europe", "500000"]).await?;
        writer.write_row(&["France", "Europe", "450000"]).await?;
        writer.write_row(&["UK", "Europe", "400000"]).await?;

        writer.save().await?;
        println!("✓ Custom credentials S3 write completed!");
    }

    // Example 4: MinIO write (S3-compatible)
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 4: MinIO (S3-Compatible) Write ===");
        let creds = Credentials::new("minioadmin", "minioadmin", None, None, "minio");

        let config = aws_sdk_s3::Config::builder()
            .region(Region::new("us-east-1"))
            .credentials_provider(creds)
            .endpoint_url("http://localhost:9000")
            .build();

        let client = S3Client::from_conf(config);

        let mut writer = S3ExcelWriter::builder()
            .bucket("reports")
            .key("monthly/2024-01-report.xlsx")
            .region("us-east-1")
            .endpoint_url("http://localhost:9000")
            .force_path_style(true)
            .build_with_client(client)
            .await?;

        writer
            .write_header_bold(&["Date", "Event", "Count"])
            .await?;
        writer.write_row(&["2024-01-01", "Login", "1234"]).await?;
        writer.write_row(&["2024-01-02", "Purchase", "567"]).await?;
        writer.write_row(&["2024-01-03", "Error", "89"]).await?;

        writer.save().await?;
        println!("✓ MinIO write completed!");
    }

    // Example 5: Write typed data
    println!("\n=== Example 5: Write Typed Data ===");
    let mut writer = S3ExcelWriter::builder()
        .bucket("my-reports")
        .key("data/typed-data-2024.xlsx")
        .region("us-east-1")
        .build()
        .await?;

    writer
        .write_header_bold(&["ID", "Amount", "Active", "Date"])
        .await?;

    // Write rows with different data types
    let row1 = vec![
        CellValue::Int(1),
        CellValue::Float(1234.56),
        CellValue::Bool(true),
        CellValue::String("2024-01-01".to_string()),
    ];
    writer.write_row_typed(&row1).await?;

    let row2 = vec![
        CellValue::Int(2),
        CellValue::Float(5678.90),
        CellValue::Bool(false),
        CellValue::String("2024-01-02".to_string()),
    ];
    writer.write_row_typed(&row2).await?;

    writer.save().await?;
    println!("✓ Typed data write completed!");

    // Example 6: Cloudflare R2 write
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 6: Cloudflare R2 Write ===");
        let creds = Credentials::new("r2-access-key", "r2-secret-key", None, None, "r2");

        let config = aws_sdk_s3::Config::builder()
            .region(Region::new("auto")) // R2 uses "auto"
            .credentials_provider(creds)
            .endpoint_url("https://abc123.r2.cloudflarestorage.com")
            .build();

        let client = S3Client::from_conf(config);

        let mut writer = S3ExcelWriter::builder()
            .bucket("my-bucket")
            .key("reports/r2-test.xlsx")
            .region("auto")
            .endpoint_url("https://abc123.r2.cloudflarestorage.com")
            .build_with_client(client)
            .await?;

        writer
            .write_header_bold(&["Name", "Size", "Modified"])
            .await?;
        writer
            .write_row(&["file1.txt", "1024", "2024-01-01"])
            .await?;
        writer
            .write_row(&["file2.txt", "2048", "2024-01-02"])
            .await?;

        writer.save().await?;
        println!("✓ Cloudflare R2 write completed!");
    }

    // Example 7: DigitalOcean Spaces write
    #[cfg(feature = "cloud-s3")]
    {
        println!("\n=== Example 7: DigitalOcean Spaces Write ===");
        let creds = Credentials::new("do-access-key", "do-secret-key", None, None, "do-spaces");

        let config = aws_sdk_s3::Config::builder()
            .region(Region::new("nyc3"))
            .credentials_provider(creds)
            .endpoint_url("https://nyc3.digitaloceanspaces.com")
            .build();

        let client = S3Client::from_conf(config);

        let mut writer = S3ExcelWriter::builder()
            .bucket("my-space")
            .key("exports/do-test.xlsx")
            .region("nyc3")
            .endpoint_url("https://nyc3.digitaloceanspaces.com")
            .build_with_client(client)
            .await?;

        writer
            .write_header_bold(&["Timestamp", "Event", "Status"])
            .await?;
        writer
            .write_row(&["2024-01-01 10:00", "Started", "OK"])
            .await?;
        writer
            .write_row(&["2024-01-01 10:30", "Processing", "OK"])
            .await?;
        writer
            .write_row(&["2024-01-01 11:00", "Completed", "SUCCESS"])
            .await?;

        writer.save().await?;
        println!("✓ DigitalOcean Spaces write completed!");
    }

    Ok(())
}
