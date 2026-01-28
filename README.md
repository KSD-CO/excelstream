# excelstream

🦀 **High-performance streaming Excel, CSV & Parquet library for Rust with constant memory usage**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KSD-CO/excelstream/workflows/Rust/badge.svg)](https://github.com/KSD-CO/excelstream/actions)

## ✨ Highlights

- 📊 **XLSX, CSV & Parquet Support** - Read/write Excel, CSV, and Parquet files
- 📉 **Constant Memory** - ~3-35 MB regardless of file size
- ☁️ **Cloud Streaming** - Direct S3/GCS uploads with ZERO temp files
- ⚡ **High Performance** - 94K rows/sec (S3), 1.2M rows/sec (CSV)
- 🔄 **True Streaming** - Process files row-by-row, no buffering
- 🗜️ **Parquet Conversion** - Stream Excel ↔ Parquet with constant memory
- 🐳 **Production Ready** - Works in 256 MB containers

## 🔥 What's New in v0.19.0

**Performance & Memory Optimizations** - Enhanced streaming reader and CSV parser!

- 🚀 **Optimized Streaming Reader** - Simplified buffer management with single-scan approach
- 💾 **Reduced Memory Allocations** - One fewer String buffer per iterator (lower heap usage)
- 📝 **Smarter CSV Parsing** - Pre-allocated buffers for typical row sizes
- 🎯 **Cleaner Codebase** - 36% code reduction in streaming reader (64 lines removed)
- 🔧 **Better Maintainability** - Simpler logic for easier debugging and contributions

```rust
// Streaming reader now uses optimized single-pass buffer scanning
let mut reader = ExcelReader::open("large_file.xlsx")?;
for row in reader.rows_by_index(0)? {
    let row_data = row?;
    // Process row with improved memory efficiency
}
```

### Previous Release: v0.18.0

**Cloud Replication & Transfer** - Replicate Excel files between different cloud storage services!

```rust
use excelstream::cloud::replicate::{CloudReplicate, ReplicateConfig, CloudSource, CloudDestination, CloudProvider};

let source = CloudSource {
    provider: CloudProvider::S3,
    bucket: "production-bucket".to_string(),
    key: "reports/data.xlsx".to_string(),
    region: Some("us-east-1".to_string()),
    endpoint_url: None,
};

let destination = CloudDestination {
    provider: CloudProvider::S3,
    bucket: "backup-bucket".to_string(),
    key: "backups/data-backup.xlsx".to_string(),
    region: Some("us-west-2".to_string()),
    endpoint_url: None,
};

let config = ReplicateConfig::new(source, destination)
    .with_chunk_size(10 * 1024 * 1024); // 10MB chunks

let replicate = CloudReplicate::with_clients(config, source_client, dest_client);
let stats = replicate.execute().await?;

println!("Transferred: {} bytes at {:.2} MB/s", stats.bytes_transferred, stats.speed_mbps());
```

**Features:**
- 🔄 **Cloud-to-Cloud Transfer** - Replicate between S3, MinIO, R2, DO Spaces
- ⚡ **True Streaming** - Constant memory usage (~5-10MB), no memory peaks
- 🚀 **Server-side Copy** - Same-region transfers use native S3 copy API (instant)
- 🔑 **Different Credentials** - Each cloud can have different API keys
- 📊 **Transfer Stats** - Speed (MB/s), duration, bytes transferred
- 🏗️ **Builder Pattern** - Flexible configuration with custom clients

**Also includes:** v0.17.0 Multi-Cloud Explicit Credentials + v0.16.0 Parquet Support

[See full changelog](CHANGELOG.md) | [Multi-cloud guide →](MULTI_CLOUD_CONFIG.md) | [Cloud Replication →](examples/cloud_replicate.rs)

---

## 📦 Quick Start

### Installation

```toml
[dependencies]
excelstream = "0.18"

# Optional features
excelstream = { version = "0.18", features = ["cloud-s3"] }        # S3 support
excelstream = { version = "0.18", features = ["cloud-gcs"] }       # GCS support
excelstream = { version = "0.18", features = ["parquet-support"] } # Parquet conversion
```

### Write Excel (Local)

```rust
use excelstream::ExcelWriter;

let mut writer = ExcelWriter::new("output.xlsx")?;

// Write 1M rows with only 3 MB memory!
writer.write_header_bold(&["ID", "Name", "Amount"])?;
for i in 1..=1_000_000 {
    writer.write_row(&[&i.to_string(), "Item", "1000"])?;
}
writer.save()?;
```

### Read Excel (Streaming)

```rust
use excelstream::ExcelReader;

let mut reader = ExcelReader::open("large.xlsx")?;

// Process 1 GB file with only 12 MB memory!
for row in reader.rows("Sheet1")? {
    let row = row?;
    println!("{:?}", row.to_strings());
}
```

### S3 Streaming (v0.14+)

```rust
use excelstream::cloud::S3ExcelWriter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = S3ExcelWriter::builder()
        .bucket("reports")
        .key("sales.xlsx")
        .build()
        .await?;

    writer.write_header_bold(["Date", "Revenue"]).await?;
    writer.write_row(["2024-01-01", "125000"]).await?;
    writer.save().await?;  // Streams to S3, no disk!
    Ok(())
}
```

[More examples →](examples/)

---

## 🎯 Why ExcelStream?

**The Problem:** Traditional libraries load entire files into memory

```rust
// ❌ Traditional: 1 GB file = 1+ GB RAM (OOM in containers!)
let workbook = Workbook::new("huge.xlsx")?;
```

**The Solution:** True streaming with constant memory

```rust
// ✅ ExcelStream: 1 GB file = 12 MB RAM
let mut reader = ExcelReader::open("huge.xlsx")?;
for row in reader.rows("Sheet1")? { /* streaming! */ }
```

### Performance Comparison

| Operation | Traditional | ExcelStream | Improvement |
|-----------|-------------|-------------|-------------|
| Write 1M rows | 100+ MB | **2.7 MB** | **97% less memory** |
| Read 1GB file | ❌ Crash | **12 MB** | Works! |
| S3 upload 500K rows | Temp file | **34 MB** | **Zero disk** |
| K8s pod (256MB) | ❌ OOMKilled | ✅ Works | Production ready |

---

## ☁️ Cloud Features

### S3 Direct Streaming (v0.14)

Upload Excel files directly to S3 with **ZERO temp files**:

```bash
cargo add excelstream --features cloud-s3
```

**Performance (Real AWS S3):**

| Dataset | Memory | Throughput | Temp Files |
|---------|--------|------------|------------|
| 10K rows | 15 MB | 11K rows/s | **ZERO** ✅ |
| 100K rows | 23 MB | 45K rows/s | **ZERO** ✅ |
| 500K rows | 34 MB | 94K rows/s | **ZERO** ✅ |

Perfect for:
- ✅ AWS Lambda (read-only filesystem)
- ✅ Docker containers (no disk space)
- ✅ Kubernetes CronJobs (limited memory)

[See S3 performance details →](PERFORMANCE_S3.md)

### S3-Compatible Services (v0.17+)

Stream to **AWS S3, MinIO, Cloudflare R2, DigitalOcean Spaces**, and other S3-compatible services with **explicit credentials** - no environment variables needed!

```rust
use excelstream::cloud::{S3ExcelWriter, S3ExcelReader};
use s_zip::cloud::S3ZipWriter;
use aws_sdk_s3::{Client, config::Credentials};

// Example 1: AWS S3 with explicit credentials
let aws_creds = Credentials::new(
    "AKIA...",           // access_key_id
    "secret...",         // secret_access_key
    None, None, "aws"
);
let aws_config = aws_sdk_s3::Config::builder()
    .credentials_provider(aws_creds)
    .region(aws_sdk_s3::config::Region::new("ap-southeast-1"))
    .build();
let aws_client = Client::from_conf(aws_config);

// Example 2: MinIO with explicit credentials
let minio_creds = Credentials::new("minioadmin", "minioadmin", None, None, "minio");
let minio_config = aws_sdk_s3::Config::builder()
    .credentials_provider(minio_creds)
    .endpoint_url("http://localhost:9000")
    .region(aws_sdk_s3::config::Region::new("us-east-1"))
    .force_path_style(true)  // Required for MinIO
    .build();
let minio_client = Client::from_conf(minio_config);

// Example 3: Cloudflare R2 with explicit credentials
let r2_creds = Credentials::new("access_key", "secret_key", None, None, "r2");
let r2_config = aws_sdk_s3::Config::builder()
    .credentials_provider(r2_creds)
    .endpoint_url("https://<account-id>.r2.cloudflarestorage.com")
    .region(aws_sdk_s3::config::Region::new("auto"))
    .build();
let r2_client = Client::from_conf(r2_config);

// Write Excel file to ANY S3-compatible service
let s3_writer = S3ZipWriter::new(aws_client.clone(), "my-bucket", "report.xlsx").await?;
let mut writer = S3ExcelWriter::from_s3_writer(s3_writer);
writer.write_header_bold(["Name", "Value"]).await?;
writer.write_row(["Test", "123"]).await?;
writer.save().await?;

// Read Excel file from ANY S3-compatible service
let mut reader = S3ExcelReader::from_s3_client(aws_client, "my-bucket", "data.xlsx").await?;
for row in reader.rows("Sheet1")? {
    println!("{:?}", row?.to_strings());
}
```

**Supported Services:**

| Service | Endpoint Example | Region |
|---------|------------------|--------|
| AWS S3 | (default) | `us-east-1`, `ap-southeast-1`, etc. |
| MinIO | `http://localhost:9000` | `us-east-1` |
| Cloudflare R2 | `https://<account>.r2.cloudflarestorage.com` | `auto` |
| DigitalOcean Spaces | `https://nyc3.digitaloceanspaces.com` | `us-east-1` |
| Backblaze B2 | `https://s3.us-west-000.backblazeb2.com` | `us-west-000` |
| Linode | `https://us-east-1.linodeobjects.com` | `us-east-1` |

**✨ Key Features:**
- 🔑 **Explicit credentials** - no environment variables needed
- 🌍 **Multi-cloud support** - use different credentials for each cloud
- 🚀 **True streaming** - only **19-20 MB memory** for 100K rows
- ⚡ **Concurrent uploads** - upload to multiple clouds simultaneously
- 🔒 **Type-safe** - full compile-time checking

**🔑 Full Multi-Cloud Guide** → [MULTI_CLOUD_CONFIG.md](MULTI_CLOUD_CONFIG.md) - Complete examples for AWS, MinIO, R2, Spaces, and B2!

### GCS Direct Streaming (v0.14)

Upload Excel files directly to Google Cloud Storage with **ZERO temp files**:

```bash
cargo add excelstream --features cloud-gcs
```

```rust
use excelstream::cloud::GCSExcelWriter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = GCSExcelWriter::builder()
        .bucket("my-bucket")
        .object("report.xlsx")
        .build()
        .await?;

    writer.write_header_bold(["Month", "Sales"]).await?;
    writer.write_row(["January", "50000"]).await?;
    writer.save().await?; // ✅ Streams directly to GCS!
    Ok(())
}
```

Perfect for:
- ✅ Cloud Run (read-only filesystem)
- ✅ Cloud Functions (no disk space)
- ✅ GKE workloads (limited memory)

[See GCS example →](examples/gcs_streaming.rs)

### HTTP Streaming

Stream Excel files directly to web responses:

```rust
use excelstream::cloud::HttpExcelWriter;

async fn download() -> impl IntoResponse {
    let mut writer = HttpExcelWriter::new();
    writer.write_row(&["Data"])?;
    ([(header::CONTENT_TYPE, "application/vnd....")], writer.finish()?)
}
```

[HTTP streaming guide →](examples/http_streaming.rs)

---

## 📊 CSV Support

**13.5x faster** than Excel for CSV workloads:

```rust
use excelstream::csv::CsvWriter;

let mut writer = CsvWriter::new("data.csv")?;
writer.write_row(&["A", "B", "C"])?;  // 1.2M rows/sec!
writer.save()?;
```

**Features:**
- ✅ Zstd compression (`.csv.zst` - 2.9x smaller)
- ✅ Auto-detection (`.csv`, `.csv.gz`, `.csv.zst`)
- ✅ Streaming (< 5 MB memory)

[CSV examples →](examples/csv_write.rs)

---

## 🗜️ Parquet Support (v0.16+)

Convert between Excel and Parquet with **constant memory** streaming:

```bash
cargo add excelstream --features parquet-support
```

### Excel → Parquet

```rust
use excelstream::parquet::ExcelToParquetConverter;

let converter = ExcelToParquetConverter::new("data.xlsx")?;
let rows = converter.convert_to_parquet("output.parquet")?;
println!("Converted {} rows", rows);
```

### Parquet → Excel

```rust
use excelstream::parquet::ParquetToExcelConverter;

let converter = ParquetToExcelConverter::new("data.parquet")?;
let rows = converter.convert_to_excel("output.xlsx")?;
println!("Converted {} rows", rows);
```

### Streaming with Progress

```rust
let converter = ParquetToExcelConverter::new("large.parquet")?;
converter.convert_with_progress("output.xlsx", |current, total| {
    println!("Progress: {}/{} rows", current, total);
})?;
```

**Features:**
- ✅ **Constant memory** - Processes in 10K row batches
- ✅ **All data types** - Strings, numbers, booleans, dates, timestamps
- ✅ **Progress tracking** - Monitor large conversions
- ✅ **High performance** - Efficient columnar format handling

**Use Cases:**
- Convert Excel reports to Parquet for data lakes
- Export Parquet data to Excel for analysis
- Integrate with Apache Arrow/Spark workflows

[Parquet examples →](examples/parquet_to_excel.rs)

---

## 🚀 Use Cases

### 1. Large File Processing

```rust
// Process 500 MB Excel with only 25 MB RAM
let mut reader = ExcelReader::open("customers.xlsx")?;
for row in reader.rows("Sales")? {
    // Process row-by-row, constant memory!
}
```

### 2. Database Exports

```rust
// Export 1M database rows to Excel
let mut writer = ExcelWriter::new("export.xlsx")?;
let rows = db.query("SELECT * FROM large_table")?;
for row in rows {
    writer.write_row(&[row.get(0), row.get(1)])?;
}
writer.save()?;  // Only 3 MB memory used!
```

### 3. Cloud Pipelines

```rust
// Lambda function: DB → Excel → S3
let mut writer = S3ExcelWriter::builder()
    .bucket("data-lake").key("export.xlsx").build().await?;

let rows = db.query_stream("SELECT * FROM events").await?;
while let Some(row) = rows.next().await {
    writer.write_row(row).await?;
}
writer.save().await?;  // No temp files, no disk!
```

---

## 📚 Documentation

- [API Docs](https://docs.rs/excelstream) - Full API reference
- [Examples](examples/) - Code examples for all features
- [CHANGELOG](CHANGELOG.md) - Version history
- [Performance](PERFORMANCE_S3.md) - Detailed benchmarks
- [Multi-Cloud Config](MULTI_CLOUD_CONFIG.md) - AWS, MinIO, R2, Spaces, B2 setup

### Key Topics

- [Excel Writing](examples/basic_write.rs) - Basic & advanced writing
- [Excel Reading](examples/basic_read.rs) - Streaming read
- [S3 Streaming](examples/s3_streaming.rs) - AWS S3 uploads
- [Multi-Cloud Config](examples/multi_cloud_config.rs) - Multi-cloud credentials
- [GCS Streaming](examples/gcs_streaming.rs) - Google Cloud Storage uploads
- [CSV Support](examples/csv_write.rs) - CSV operations
- [Parquet Conversion](examples/parquet_to_excel.rs) - Excel ↔ Parquet
- [Styling](examples/cell_formatting.rs) - Cell formatting & colors

---

## 🔧 Features

| Feature | Description |
|---------|-------------|
| `default` | Core Excel/CSV with Zstd compression |
| `cloud-s3` | S3 direct streaming (async) |
| `cloud-gcs` | GCS direct streaming (async) |
| `cloud-http` | HTTP response streaming |
| `parquet-support` | Parquet ↔ Excel conversion |
| `serde` | Serde serialization support |
| `parallel` | Parallel processing with Rayon |

---

## ⚡ Performance

**Memory Usage (Constant):**
- Excel write: **2.7 MB** (any size)
- Excel read: **10-12 MB** (any size)
- S3 streaming: **30-35 MB** (any size)
- CSV write: **< 5 MB** (any size)

**Throughput:**
- Excel write: 42K rows/sec
- Excel read: 50K rows/sec
- S3 streaming: 94K rows/sec
- CSV write: 1.2M rows/sec

---

## 🛠️ Migration from v0.13

**S3ExcelWriter** is now async:

```rust
// OLD (v0.13 - sync)
writer.write_row(&["a", "b"])?;

// NEW (v0.14 - async)
writer.write_row(["a", "b"]).await?;
```

All other APIs unchanged!

---

## 📋 Requirements

- Rust 1.70+
- Optional: AWS credentials for S3 features

---

## 🤝 Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 📄 License

MIT License - See [LICENSE](LICENSE) for details

---

## 🙏 Credits

- Built with [s-zip](https://crates.io/crates/s-zip) for streaming ZIP
- AWS SDK for Rust
- All contributors and users!

---

**Need help?** [Open an issue](https://github.com/KSD-CO/excelstream/issues) | **Questions?** [Discussions](https://github.com/KSD-CO/excelstream/discussions)
