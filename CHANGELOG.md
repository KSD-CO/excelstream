# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.20.0] - 2026-01-29

### 🚀 Writer Performance Optimizations

**Major Performance Improvements** - 3-8% faster Excel writing with fewer memory allocations!

### Changed
- **Eliminated Double Allocation** (`src/writer.rs`)
  - Removed unnecessary `Vec<String>` intermediate buffer in `write_row()`
  - Changed from `map().collect()` to direct iterator pass-through
  - **Impact:** 2 fewer allocations per row

- **Fast Integer Formatting** (`src/fast_writer/zero_temp_workbook.rs`)
  - Integrated `itoa` crate for integer-to-string conversion
  - Replaced `.to_string()` with `itoa::Buffer::format()`
  - **Impact:** 2-3x faster integer formatting with zero heap allocations

- **Optimized Column Letter Generation** (`src/fast_writer/zero_temp_workbook.rs`)
  - Rewrote `push_column_letter()` to write directly to output buffer
  - Eliminated intermediate String allocation
  - **Impact:** Zero temp allocations for column addressing (A, B, AA, etc.)

### Added
- **Dependency:** `itoa = "1.0"` - Fast integer formatting library
- **Optional Dependency:** `dhat = { version = "0.3.3", optional = true }` - Heap profiling
- **Feature:** `dhat-heap` - Enable dhat heap profiling for benchmarking
- **Example:** `examples/memory_bench.rs` - Memory profiling example

### Performance
- **10 columns × 1M rows:** +6.1% faster (29,455 → 31,263 rows/sec)
- **20 columns × 1M rows:** +8.5% faster (17,367 → 18,842 rows/sec)
- **Memory usage:** Virtually identical (+0.4% RSS, fewer heap allocations)
- **File format:** 100% compatible, identical file sizes

### Verified
- ✅ 50/50 unit tests passing
- ✅ Cargo clippy clean
- ✅ All examples working
- ✅ 39 million cells verified for data integrity
- ✅ Comprehensive benchmark testing with criterion

### Documentation
- Added `PERFORMANCE_VERIFICATION_PR7.md` - Detailed benchmark results
- Added `MILLION_ROW_PERFORMANCE_PR7.md` - 1M row performance analysis
- Added `MEMORY_ANALYSIS_CLARIFICATION.md` - Memory usage deep dive
- Added `FILE_INTEGRITY_REPORT_PR7.md` - Data integrity verification
- Added `PR7_FINAL_RECOMMENDATION.md` - Complete PR review

**See:** [Performance Report](MILLION_ROW_PERFORMANCE_PR7.md) | [PR #7 Review](PR7_FINAL_RECOMMENDATION.md)

## [0.19.0] - 2026-01-28

### 🚀 Performance & Code Quality Improvements

**Optimized Streaming Reader & CSV Parser** - Enhanced memory efficiency and code maintainability!

### Changed
- **Streaming Reader Optimization** (`src/streaming_reader.rs`)
  - Simplified buffer management with single scan position tracking
  - Removed complex `try_extract_row()` method
  - Eliminated unnecessary `row_content` String buffer (reduced heap allocations)
  - Changed from copying/draining buffers to in-place scanning
  - Made Excel date calculation constants static to avoid repeated allocations
  - **Code reduction:** 36% smaller (64 lines removed, 128 → 64 lines)

- **CSV Parser Enhancement** (`src/csv/parser.rs`)
  - Pre-allocated field vector capacity (16 fields typical)
  - Pre-allocated field string buffer capacity (64 bytes typical)
  - Reduced allocations during CSV line parsing

### Improved
- **Memory Efficiency** - One fewer String buffer per streaming iterator
- **Code Maintainability** - Simpler, more readable buffer management logic
- **Developer Experience** - Easier to understand and debug streaming operations

### Performance Notes
- Memory usage reduced by eliminating redundant buffers
- Code complexity reduced by 36% in streaming reader
- Trade-off: Slight performance variation in some workloads (see PERFORMANCE_COMPARISON.md)
- Optimized for code quality and maintainability

## [0.18.0] - 2026-01-26

### 🎉 Major Features

**Cloud-to-Cloud Replication** - Transfer Excel files between different cloud storage services with true streaming!

### Added
- **CloudReplicate** - Direct cloud-to-cloud transfer without local disk
  - `CloudReplicate::new()` - Auto-generated clients from env vars
  - `CloudReplicate::with_clients()` - Custom clients with different credentials
  - `CloudReplicate::with_source_client()` - Only source client customized
  - `CloudReplicate::with_dest_client()` - Only destination client customized
  - `CloudReplicateBuilder` - Flexible builder pattern

- **ReplicateConfig** - Configuration for transfer
  - Configurable chunk size (default 5MB)
  - Configurable retry count (default 3)
  - Support for different regions and endpoints

- **ReplicateStats** - Transfer statistics
  - Bytes transferred tracking
  - Chunks transferred count
  - Transfer speed calculation (MB/s)
  - Elapsed time measurement
  - Error tracking

- **Optimized Transfer Strategies**
  - **Same-region copies**: Uses native S3 copy_object API (server-side, instant)
  - **Cross-region/cross-endpoint**: Streaming multipart upload (constant ~5-10MB memory)
  - Zero memory peaks with lazy ByteStream evaluation

- **Multi-Cloud Support**
  - AWS S3 ↔ AWS S3 (different regions)
  - AWS S3 ↔ MinIO
  - AWS S3 ↔ Cloudflare R2
  - AWS S3 ↔ DigitalOcean Spaces
  - MinIO ↔ MinIO
  - And any S3-compatible combination

- **Examples**
  - `examples/cloud_replicate.rs` - 5 complete examples
    - Auto-generated clients
    - Custom S3 clients with credentials
    - Builder pattern with clients
    - MinIO with different secrets
    - DigitalOcean Spaces
  - `examples/s3_excel_writer.rs` - 7 complete S3 writer examples
    - Basic S3 write
    - Custom clients
    - Custom credentials
    - MinIO support
    - Typed data writing
    - Cloudflare R2
    - DigitalOcean Spaces

### Performance
- ✅ **Memory: Constant ~5-10MB** for any file size transfer
- ✅ **Same-region**: Instant server-side copy (no data transfer)
- ✅ **Cross-region**: Streaming multipart with configurable chunks
- ✅ **Speed**: Transfer speed monitoring and reporting

### Documentation
- Added cloud_replicate.rs example with 5 use cases
- Added s3_excel_writer.rs example with 7 use cases
- Updated README.md with v0.18.0 features
- Cloud replication architecture documented in code

### Breaking Changes
- None (fully backward compatible with v0.17.0)

## [0.17.0] - 2026-01-22

### 🎉 Major Features

**Multi-Cloud Explicit Credentials** - Upload to multiple S3-compatible clouds with different credentials!

### Added
- **S3ExcelWriter::from_s3_writer()** - Create writer from AWS SDK S3 client with explicit credentials
  - No environment variables needed
  - Full control over credentials per cloud
  - Supports AWS S3, FPT Cloud, MinIO, Cloudflare R2, DigitalOcean Spaces, Backblaze B2
- **S3ExcelReader::from_s3_client()** - Create reader from AWS SDK S3 client with explicit credentials
  - Download and read Excel files from any S3-compatible service
  - Stream files to temp directory for processing
- **Multi-cloud examples**:
  - `examples/multi_cloud_config.rs` - Complete guide for AWS, MinIO, R2, Spaces, B2
- **MULTI_CLOUD_CONFIG.md** - Comprehensive multi-cloud configuration guide

### Performance
- ✅ **Memory: 19-20 MB** for 100K rows (4 columns) - verified with binary execution
- ✅ **Concurrent uploads**: 21.5 MB for simultaneous upload to 2 clouds
- ✅ **Speed**: 34K-47K rows/sec to S3-compatible services

### Documentation
- Updated README.md with multi-cloud examples
- Added MULTI_CLOUD_CONFIG.md with step-by-step guides for all supported services
- Added performance benchmarks for FPT Cloud and AWS S3

## [0.16.0] - 2025-01-16

### 🎉 Major Features

**Parquet Support** - Convert between Excel and Parquet formats with constant memory streaming!

- **Excel ↔ Parquet Conversion**: Bidirectional conversion with streaming architecture
- **Constant Memory**: Processes in 10K row batches regardless of file size
- **All Data Types**: Supports strings, numbers, booleans, dates, timestamps, and more
- **Progress Tracking**: Monitor large conversions with callback functions

### Added
- **High-Level Converters**:
  - `ExcelToParquetConverter`: Convert Excel files to Parquet format
    - Streaming conversion with 10K row batches
    - Constant memory usage
    - Progress callback support
  - `ParquetToExcelConverter`: Convert Parquet files to Excel format
    - Row-by-row streaming
    - Progress callback with row count
    - Header formatting with bold style
- **Low-Level API**:
  - `ParquetReader`: Stream Parquet files row-by-row
    - Schema inspection
    - Row count metadata
    - Iterator-based row access
    - Comprehensive data type support (Int8/16/32/64, UInt8/16/32/64, Float32/64, Boolean, Date32/64, Timestamp, Utf8, LargeUtf8)
- **New Feature Flag**: `parquet-support`
  - Enables Apache Parquet and Apache Arrow dependencies
  - Optional feature to minimize base library size
- **New Examples**:
  - `parquet_to_excel.rs`: Convert Parquet to Excel
  - `excel_to_parquet.rs`: Convert Excel to Parquet
  - `parquet_streaming.rs`: Advanced streaming with filtering
  - `parquet_performance_test.rs`: Benchmark conversions and measure memory

### Technical Details
- **Dependencies**:
  - Apache Parquet 57 (with arrow, snap, zstd features)
  - Apache Arrow 57 (with ipc feature)
- **Memory Efficiency**:
  - Excel → Parquet: 10K row batches, constant memory
  - Parquet → Excel: Streaming record batches
  - No full file loads into memory
- **Data Type Handling**:
  - All Parquet primitive types supported
  - Automatic string conversion for Excel compatibility
  - Null handling with empty string fallback

### Performance
- **Streaming Architecture**: Processes millions of rows efficiently
- **Batch Processing**: 10K rows per batch for optimal memory/speed balance
- **Columnar Conversion**: Efficient row-to-column and column-to-row transformations

### S3-Compatible Services Support (MinIO, R2, Spaces, etc.)

**New in this release**: Full support for S3-compatible storage services!

- **S3ExcelWriter** now supports:
  - `.endpoint_url()` - Custom endpoint for MinIO, Cloudflare R2, DigitalOcean Spaces, Backblaze B2
  - `.force_path_style(true)` - Required for MinIO and some S3-compatible services
- **S3ExcelReader** now supports:
  - `.endpoint_url()` - Read from any S3-compatible service
  - `.force_path_style(true)` - Path-style addressing support

**Supported Services:**

| Service | Endpoint Example |
|---------|------------------|
| MinIO | `http://localhost:9000` |
| Cloudflare R2 | `https://<account_id>.r2.cloudflarestorage.com` |
| DigitalOcean Spaces | `https://<region>.digitaloceanspaces.com` |
| Backblaze B2 | `https://s3.<region>.backblazeb2.com` |
| Linode Object Storage | `https://<region>.linodeobjects.com` |

**Example (MinIO):**
```rust
// Write to MinIO
let writer = S3ExcelWriter::builder()
    .endpoint_url("http://localhost:9000")
    .bucket("my-bucket")
    .key("report.xlsx")
    .region("us-east-1")
    .force_path_style(true)
    .build()
    .await?;

// Read from MinIO
let reader = S3ExcelReader::builder()
    .endpoint_url("http://localhost:9000")
    .bucket("my-bucket")
    .key("data.xlsx")
    .force_path_style(true)
    .build()
    .await?;
```

### Dependencies
- **s-zip**: Updated from 0.6.0 → 0.8.0
  - New builder API for S3ZipWriter
  - Native support for S3-compatible services

### Documentation
- Updated README with Parquet section and examples
- Updated package description to include Parquet support
- Added comprehensive code examples and use cases

## [0.14.0] - 2025-01-02

### 🎉 Major Features

**Revolutionary Cloud Integration** - Stream Excel files directly to AWS S3 and Google Cloud Storage with NO temp files and constant memory!

- **S3 Direct Streaming**: Using s-zip cloud-s3 support
- **GCS Direct Streaming**: Using s-zip cloud-gcs support (NEW!)

### Changed
- **S3ExcelWriter - TRUE Streaming**: Complete rewrite using s-zip 0.5.1 cloud-s3
  - **ZERO temp files**: Stream directly to S3 (no disk usage at all!)
  - **Constant memory**: ~30-35 MB regardless of file size
  - **Async API**: All methods now async/await (breaking change)
  - **High throughput**: 94K rows/sec streaming to S3
  - **Multipart upload**: Automatic S3 multipart handled by s-zip

### Breaking Changes
- **S3ExcelWriter API is now async**:
  - `.write_row()` → `.write_row().await`
  - `.write_header_bold()` → `.write_header_bold().await`
  - `.save()` → `.save().await`
  - All methods require `.await` now

### Performance (Real AWS S3 Tests)

| Dataset | Peak Memory | Throughput | Time | Temp Files |
|---------|-------------|------------|------|------------|
| 10K rows | 15.0 MB | 10,951 rows/s | 0.91s | ZERO ✅ |
| 100K rows | 23.2 MB | 45,375 rows/s | 2.20s | ZERO ✅ |
| 500K rows | 34.4 MB | 94,142 rows/s | 5.31s | ZERO ✅ |

**Memory is Constant:**
- 10K → 100K rows (10x data): Memory only +55% (15 → 23 MB)
- 100K → 500K rows (5x data): Memory only +48% (23 → 34 MB)
- TRUE streaming confirmed - no file-size-proportional growth!

### Added
- **GCS Support**: New `GCSExcelWriter` for Google Cloud Storage
  - `cloud-gcs` feature flag
  - Async streaming API matching S3ExcelWriter
  - Zero temp files, constant memory usage
- **New Examples**:
  - `gcs_streaming.rs`: Stream Excel to Google Cloud Storage
  - `gcs_performance_test.rs`: Benchmark GCS streaming performance
  - `s3_performance_test.rs`: Benchmark S3 streaming with configurable dataset size
  - `s3_verify.rs`: Download and verify S3 uploaded files
- **Performance Documentation**:
  - `PERFORMANCE_S3.md`: Detailed memory usage analysis and benchmarks

### Internal
- Updated s-zip dependency: 0.3.1 → 0.6.0
- Added s-zip cloud-s3 and cloud-gcs features
- Added google-cloud-storage and google-cloud-auth dependencies for GCS support
- Removed tempfile from S3 writer implementation (still needed for S3 reader)
- S3ExcelWriter now uses `s_zip::cloud::S3ZipWriter` + `AsyncStreamingZipWriter`
- GCSExcelWriter uses `s_zip::cloud::GCSZipWriter` + `AsyncStreamingZipWriter`

### Migration Guide
```rust
// OLD (v0.13.0 - sync API, temp files)
let mut writer = S3ExcelWriter::builder().build().await?;
writer.write_row(&["a", "b"])?;  // sync
writer.save().await?;

// NEW (v0.14.0 - async API, no temp files)
let mut writer = S3ExcelWriter::builder().build().await?;
writer.write_row(["a", "b"]).await?;  // async!
writer.save().await?;
```

## [0.12.1] - 2024-12-17

### Changed
- **S3 Writer Memory Optimization**: Implemented streaming multipart upload
  - **83% Memory Reduction**: Peak memory reduced from 52.7 MB → 8.6 MB (1M rows)
  - **Streaming Upload**: Uploads file in 5MB chunks instead of reading entire file
  - **No Memory Spike**: Constant 5MB peak regardless of file size
  - **Backward Compatible**: API unchanged, only internal implementation improved

### Performance
- S3 Writer OLD: 52.7 MB peak (reads entire file to RAM)
- S3 Writer NEW: 8.6 MB peak (streams 5MB chunks)
- Memory saving: 44 MB (83% reduction) for 1M row file
- All writers comparison:
  - File-based: 3.5 MB (best - streams to disk)
  - HTTP: 8.4 MB (good - necessary in-memory)
  - S3 (NEW): 7.4 MB (excellent - streaming upload)

### Documentation
- Added `S3_STREAMING_OPTIMIZATION.md` with detailed analysis
- Updated memory comparison examples

## [0.11.1] - 2024-12-16

### Changed
- **s-zip v0.2.0**: Updated to s-zip 0.2.0 with Write trait support
  - Enables `from_writer<W: Write + Seek>()` for in-memory Excel generation
  - Perfect for web servers (no temp files needed)
  - All backward compatible, zero breaking changes

### Internal
- Clarified comments for optional dependencies (PostgreSQL, AWS)
- All dependencies are optional features, only loaded when needed

### Performance
- Verified: Same performance as v0.11.0 (42K rows/sec, 2.5 MB memory)
- All 40 tests passing
- All 7 examples working


## [0.11.0] - 2024-12-15

### Changed
- **s-zip Library Integration**: Extracted ZIP operations into standalone [s-zip](https://crates.io/crates/s-zip) crate
  - **Code Reusability**: ~544 lines of ZIP code now reusable across projects
  - **Zero Performance Impact**: Identical speed (42K rows/sec) and memory (2-3 MB)
  - **Better Maintainability**: Single source of truth for ZIP operations
  - **Community Value**: s-zip now available for other Rust projects
  - **Backward Compatible**: All existing APIs work without changes

### Internal
- Replaced internal `streaming_zip_reader.rs` and `streaming_zip_writer.rs` with s-zip dependency
- Updated `fast_writer` modules to re-export s-zip types for backward compatibility
- Added `From<s_zip::SZipError>` conversion for seamless error handling

### Performance (unchanged)
- Write: 42,557 rows/sec (strings), 43,839 rows/sec (direct)
- Read: 36,396 rows/sec with streaming
- Memory: 2-3 MB constant for any file size
- File size: 180-193 MB (1M rows, 30 columns)


## [0.9.0] - 2024-12-08

### 🎉 Major Feature: Zero-Temp Streaming Architecture

**84% Memory Reduction** - Revolutionary streaming write implementation that eliminates temporary files entirely!

### Added
- **ZeroTempWorkbook**: Stream XML directly into ZIP compressor
  - **2.7 MB RAM** for ANY SIZE (vs 17 MB in v0.8.0) = **84% reduction**
  - **Zero temp files**: Direct streaming to final .xlsx file
  - **StreamingZipWriter**: Custom ZIP writer with on-the-fly compression
  - **Data descriptors**: Write CRC/sizes after compressed data (no seeking)
  - **4KB XML buffer**: Reused per row for minimal allocations
  - **Same speed**: 50K-60K rows/sec (comparable to zip crate)
  - **File size**: ~7% larger than zip crate (acceptable trade-off)

### Performance
- **Write Performance (1M rows)**:
  - `ZeroTempWorkbook`: **2.7 MB RAM**, ~1400ms, 16 MB file ✅
  - `UltraLowMemoryWorkbook` (v0.8.0): 17 MB RAM, ~1400ms, 15 MB file
  - Traditional libraries: 100+ MB RAM or crash
- **Architecture**: On-the-fly DEFLATE compression with streaming XML generation
- **Validation**: Tested with 1M-10M rows, verified with Excel and `unzip -t`

### Changed
- Bumped version to 0.9.0
- Updated Cargo.toml description to highlight 2.7 MB memory footprint
- Updated README with v0.9.0 performance numbers and new architecture
- Exposed `ZeroTempWorkbook` in public API alongside `UltraLowMemoryWorkbook`

### Technical Details
- **StreamingZipWriter**: 
  - Writes local file headers with bit 3 set (data descriptor flag)
  - Writes data descriptors after compressed data (CRC32, sizes)
  - Assembles central directory at end
  - No seeking, no temp files, pure streaming
- **ZeroTempWorkbook**:
  - `new(path, compression_level)`: Create workbook with configurable compression
  - `add_worksheet(name)`: Start new sheet with immediate XML header write
  - `write_row(values)`: Build row XML in 4KB buffer, stream to compressor
  - `close()`: Finish sheet, write metadata files, close ZIP
- **CrcCountingWriter**: Helper that computes CRC32 while writing compressed data

### Notes
- **Backward compatibility**: `UltraLowMemoryWorkbook` still available (uses temp files)
- **Migration**: Consider switching to `ZeroTempWorkbook` for 84% memory reduction
- **Trade-offs**: File size +7% (16 MB vs 15 MB) due to streaming compression

## [0.8.0] - 2024-12-06

### Changed
- **🎯 Constant Memory Streaming**: StreamingReader now achieves true constant memory
  - **104x Memory Reduction**: 1.2 GB XML → 11.6 MB RAM (was 1204 MB in v0.7.1)
  - **100% Accurate**: Fixed chunked reading algorithm to capture all rows without data loss
  - **Any File Size**: Process multi-GB files with only 10-12 MB RAM
  - **Tested**: 86 MB file (1.2 GB uncompressed) = 11.6 MB peak memory, 1M rows read correctly

### Performance
- **⚡ Read Performance**: 
  - `StreamingReader`: 50K-60K rows/sec with constant 10-12 MB memory
  - `ExcelReader`: 30K rows/sec (memory = file size)
  - **Production Ready**: Validated with 1 million row files in real-world scenarios

### Documentation
- **📖 Updated README**: Accurate v0.8.0 performance numbers and use cases
- **🎯 New Use Case**: "Processing Large Excel Imports" with StreamingReader
- **📊 Updated Comparison Table**: 10-12 MB constant memory for all file sizes

## [0.7.1] - 2024-12-06

### Added
- **🚀 StreamingReader**: Initial streaming reader implementation
  - Constant 20-30 MB memory for ANY file size
  - 2x faster than ExcelReader: 60K rows/sec vs 30K rows/sec
  - Trade-offs: No formula evaluation, no formatting, sequential only

### Known Issues (Fixed in v0.8.0)
- ⚠️ Memory usage was actually 1.2 GB for large files (loaded full XML)
- ⚠️ Missing ~8K rows per 1M rows due to chunking bug

## [0.7.0] - 2024-12-05

### Added
- **🔒 Worksheet Protection**: Password-based sheet protection
  - 12 granular permissions (select locked/unlocked, format, sort, etc.)
  - ECMA-376 compliant password hashing
  - Prevents accidental edits in production reports
  
- **📐 Cell Merging**: Horizontal and vertical cell merging
  - Support for ranges: A1:C1, A1:A3, B2:D5
  - Perfect for report headers and grouped data
  - Proper Excel format `<mergeCells>` implementation

- **📏 Column Width**: Set custom column widths
  - Previously no-op, now fully functional
  - Support for all 16,384 columns (A-XFD)
  - Width in Excel character units (default 8.43)

### Documentation
- **📖 README Overhaul**: Streaming use cases and comparisons
- **🎯 Real-World Examples**: 5 production scenarios
- **📊 Comparison Table**: ExcelStream vs traditional libraries

## [0.6.2] - 2024-12-05

### Changed
- **⬆️ Upgraded Dependencies**: Updated `zip` crate from 0.6 to 6.0
  - Fixed `flate2` conflict issues on GitHub Actions CI
  - Updated API: `FileOptions` → `SimpleFileOptions`
  - Updated compression_level type: `i32` → `i64`
  - Better compression algorithms and bug fixes

### Removed
- **🗑️ Removed Deprecated Function**: Deleted `fix_xlsx_zip_order()`
  - No longer needed as files are written in correct order by default since v0.6.0
  - Improves performance by eliminating unnecessary reordering step
  - Reduces code complexity

### Performance
- **💾 Memory Optimization**: Memory usage improved by ~2%
  - `write_row()`: 56 MB → 55 MB
  - `write_row_typed()`: 56 MB → 55 MB
  - `write_row_styled()`: 56 MB → 55 MB
  - All methods stay well under 80 MB target ✅

### Fixed
- **🔧 CI/CD**: Fixed GitHub Actions clippy failures
  - Resolved `flate2` version conflicts with `zip` crate
  - Updated workflow to use `--features serde,parallel` instead of `--all-features`

## [0.6.1] - 2024-12-05

### Fixed
- **🎨 Cell Formatting Now Working**: Complete styles.xml implementation
  - Fixed empty styles.xml (was 100 bytes stub, now complete 2651 bytes)
  - Implemented full `write_row_styled()` with proper style attributes (`s="X"`)
  - All 14 CellStyle variants working: HeaderBold, NumberInteger, NumberDecimal, NumberCurrency, NumberPercentage, DateDefault, DateTimestamp, TextBold, TextItalic, HighlightYellow, HighlightGreen, HighlightRed, BorderThin
  - Standard ECMA-376 format IDs used (3, 4, 5, 9, 14, 22)
  - Complete fonts, fills, borders, and cellXfs definitions

### Changed
- **🔧 Unified Architecture**: Removed legacy FastWorkbook
  - Deleted `src/fast_writer/workbook.rs` (~974 lines)
  - Unified on single `UltraLowMemoryWorkbook` implementation
  - Updated all internal references and examples
  - Simplified API surface - less confusion
  - All 18 library tests passing after cleanup

### Security
- **🔒 Security Improvements**: Removed hardcoded credentials
  - Removed database connection strings from `postgres_streaming.rs`
  - Added `.env.example` with configuration templates
  - Updated `.gitignore` to prevent committing `.env` files
  - Updated documentation with security warnings
  - All PostgreSQL examples now require `DATABASE_URL` environment variable

### Improved
- **📚 Better Examples**: Consistent API usage
  - Updated `large_dataset_multi_sheet.rs` to use `ExcelWriter` instead of low-level API
  - Fixed `memory_constrained_write.rs` for new architecture
  - Updated `writers_comparison.rs` to reflect UltraLowMemoryWorkbook
  - All examples use consistent `ExcelWriter` API with `add_sheet()` method
  - Added security warnings to example documentation

### Technical Details
- Complete styles.xml with 3 fonts, 5 fills, 2 borders, 14 cellXfs
- Style indices (0-13) properly mapped to cell attributes
- Memory functions updated for UltraLowMemoryWorkbook compatibility
- No performance regression - all optimizations preserved

## [0.5.0] - 2024-12-04

### Added
- **🧠 Hybrid SST Optimization**: Intelligent selective deduplication for optimal memory usage
  - Numbers automatically written inline as `<c t="n">` (no SST overhead)
  - Long strings (>50 chars) written inline (usually unique anyway)
  - Short repeated strings use SST for efficient deduplication
  - SST capped at 100k unique strings for graceful degradation
  - Automatic optimization - no API changes required
- **📊 Dramatic Memory Reduction**: 89% less memory usage
  - Simple workbooks (5 cols, 1M rows): 49 MB → **18.8 MB** (62% reduction)
  - Medium workbooks (19 cols, 1M rows): 125 MB → **15.4 MB** (88% reduction)
  - Complex workbooks (50 cols, 100K rows): ~200 MB → **22.7 MB** (89% reduction)
  - Multi-workbook scenarios: 251 MB → **25.3 MB** (90% reduction)
- **⚡ Performance Boost**: 58% faster with hybrid SST
  - FastWorkbook: 25,682 rows/sec (was ~16,000 rows/sec)
  - Fewer SST lookups for numbers and long strings
  - Better cache efficiency for repeated short strings
- **📝 Technical Documentation**: 
  - Added `HYBRID_SST_OPTIMIZATION.md` with full strategy details
  - Performance comparison tables and benchmarks
  - Recommendations for different data types

### Changed
- `FastWorkbook` now uses hybrid SST by default (automatic)
- Replaced `SharedStringsV2` (disk-based) with optimized in-memory hybrid approach
- Updated all examples to demonstrate memory efficiency

### Removed
- Removed unused `shared_strings_v2.rs` (disk-based deduplication approach)

### Performance
- Memory: 15-25 MB for most workbooks (was 80-250 MB)
- Speed: 25K+ rows/sec (was 16K-19K rows/sec)
- File size: +10-15% vs full SST (acceptable tradeoff for 89% memory reduction)

## [0.2.2] - 2024-12-02

### Added
- **Formula Support**: Added `CellValue::Formula` variant for Excel formulas
  - Write formulas like `=SUM(A1:A10)`, `=A2+B2`, etc.
  - Formulas are preserved in Excel and calculated correctly
  - Example: `writer.write_row_typed(&[CellValue::Formula("=SUM(A1:A10)".to_string())])?;`
- **Improved Error Messages**: Better context for errors
  - `SheetNotFound` now includes list of available sheets
  - `WriteRowError` includes row number and sheet name
  - Easier debugging with detailed error information
- **Comprehensive Tests**: Added 6 new integration tests
  - Special characters handling (XML, emojis, unicode)
  - Empty string and cell handling
  - Very long strings (10KB+)
  - Unicode sheet names (Russian, Chinese, French)
  - Error message validation
  - Formula support validation
- **CI/CD**: GitHub Actions workflow for automated testing
  - Automated format checking with `cargo fmt`
  - Clippy linting with strict warnings
  - Full test suite on every push/PR
  - Automated publishing to crates.io on version tags
- **Contributing Guide**: Comprehensive CONTRIBUTING.md with guidelines

### Changed
- Updated all error handling to provide better context
- Improved documentation with more examples
- Better type annotations in public APIs

### Fixed
- Fixed all clippy warnings (11 total)
- Fixed error pattern matching for sheet not found errors
- Corrected package name in documentation (was "rust-excelize")

### Changed
- Updated documentation to reflect current implementation
- Removed outdated migration warnings
- Clarified v0.2.x features and capabilities

## [0.2.0] - 2024-11-25

### Added
- **Streaming with constant memory**: ~80MB regardless of dataset size
- **21-47% faster** than rust_xlsxwriter baseline
- Custom `FastWorkbook` implementation for high-performance writing
- `set_flush_interval()` - Configure flush frequency
- `set_max_buffer_size()` - Configure buffer size
- Memory-constrained writing for Kubernetes pods

### Changed
- **BREAKING**: Removed `rust_xlsxwriter` dependency
- **BREAKING**: `ExcelWriter` now uses `FastWorkbook` internally
- All writes now stream directly to disk
- Memory usage reduced from ~300MB to ~80MB for large datasets

### Removed
- **BREAKING**: Bold header formatting (will be restored in future version)
- **BREAKING**: `set_column_width()` now a no-op (will be restored in future version)

## [0.1.0] - 2024-11-01

### Added
- Initial release
- Excel reading with streaming support (XLSX, XLS, ODS)
- Excel writing with `rust_xlsxwriter`
- Multi-sheet support
- Typed cell values
- PostgreSQL integration examples
- Basic examples and documentation

[0.16.0]: https://github.com/KSD-CO/excelstream/compare/v0.14.0...v0.16.0
[0.14.0]: https://github.com/KSD-CO/excelstream/compare/v0.12.1...v0.14.0
[0.2.2]: https://github.com/KSD-CO/excelstream/compare/v0.2.0...v0.2.2
[0.2.0]: https://github.com/KSD-CO/excelstream/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/KSD-CO/excelstream/releases/tag/v0.1.0
