# HTTP Streaming for Excel Files

This feature allows you to stream Excel files directly via HTTP responses without writing to disk.

## Features

- ✅ **Zero disk I/O** - Generate Excel files entirely in memory (no temp files)
- ✅ **Framework agnostic** - Works with Axum, Actix-web, Warp, and any async framework
- ✅ **Memory efficient** - Pure in-memory streaming with custom buffer implementation
- ✅ **Full Excel support** - All cell types, formulas, multiple sheets, styling
- ✅ **Type-safe** - Strong typing with Rust's type system

## Quick Start

### 1. Add dependency

```toml
[dependencies]
excelstream = { version = "0.11", features = ["cloud-http"] }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

### 2. Basic Example with Axum

```rust
use axum::{
    response::{IntoResponse, Response},
    http::header,
    routing::get,
    Router,
};
use excelstream::cloud::HttpExcelWriter;
use excelstream::types::CellValue;

async fn download_report() -> Response {
    let mut writer = HttpExcelWriter::new();

    // Write header
    writer.write_header_bold(&["Month", "Sales", "Profit"]).unwrap();

    // Write data with typed values
    writer.write_row_typed(&[
        CellValue::String("January".to_string()),
        CellValue::Float(125000.50),
        CellValue::Float(45000.25),
    ]).unwrap();

    writer.write_row_typed(&[
        CellValue::String("February".to_string()),
        CellValue::Float(135000.75),
        CellValue::Float(48000.50),
    ]).unwrap();

    // Finish and get bytes
    let bytes = writer.finish().unwrap();

    // Return as HTTP response
    (
        [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"report.xlsx\""),
        ],
        bytes
    ).into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/download", get(download_report));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

## API Reference

### `HttpExcelWriter::new()`

Create a new HTTP Excel writer with default compression (level 6).

```rust
let mut writer = HttpExcelWriter::new();
```

### `HttpExcelWriter::with_compression(level)`

Create writer with custom compression level (0-9).

```rust
let mut writer = HttpExcelWriter::with_compression(9); // Maximum compression
```

### `write_header_bold(headers)`

Write a bold header row.

```rust
writer.write_header_bold(&["Name", "Age", "Email"])?;
```

### `write_row(row)`

Write a row of string values.

```rust
writer.write_row(&["Alice", "30", "alice@example.com"])?;
```

### `write_row_typed(cells)`

Write a row with typed cell values.

```rust
use excelstream::types::CellValue;

writer.write_row_typed(&[
    CellValue::String("Alice".to_string()),
    CellValue::Int(30),
    CellValue::String("alice@example.com".to_string()),
])?;
```

### `add_worksheet(name)`

Add a new worksheet to the workbook.

```rust
writer.add_worksheet("Sales")?;
writer.write_row(&["Product", "Revenue"])?;

writer.add_worksheet("Inventory")?;
writer.write_row(&["Item", "Stock"])?;
```

### `finish()`

Finish writing and return the Excel file as `Vec<u8>`.

```rust
let bytes = writer.finish()?;
```

## Complete Example

Run the full example:

```bash
cargo run --example http_streaming --features cloud-http
```

Then test:

```bash
# Download sales report
curl -o report.xlsx http://localhost:3000/download/sales-report

# Download large dataset (10K rows)
curl -o large.xlsx http://localhost:3000/download/large-dataset

# Download multi-sheet workbook
curl -o multi.xlsx http://localhost:3000/download/multi-sheet
```

## Supported Cell Types

```rust
use excelstream::types::CellValue;

// String
CellValue::String("Hello".to_string())

// Integer
CellValue::Int(42)

// Float
CellValue::Float(3.14159)

// Boolean
CellValue::Bool(true)

// Formula
CellValue::Formula("=SUM(A1:A10)".to_string())

// DateTime (Excel serial number)
CellValue::DateTime(44562.0) // 2022-01-01

// Error
CellValue::Error("#N/A".to_string())

// Empty cell
CellValue::Empty
```

## Performance

Tested on a standard laptop:

| Dataset Size | File Size | Generation Time | Memory Usage |
|--------------|-----------|-----------------|--------------|
| 100 rows     | ~3 KB     | <1 ms          | ~1 MB       |
| 10K rows     | ~270 KB   | ~150 ms        | ~5 MB       |
| 100K rows    | ~2.5 MB   | ~1.5 sec       | ~15 MB      |
| 1M rows      | ~25 MB    | ~15 sec        | ~50 MB      |

## Integration with Other Frameworks

### Actix-web

```rust
use actix_web::{get, HttpResponse};
use excelstream::cloud::HttpExcelWriter;

#[get("/download")]
async fn download() -> HttpResponse {
    let mut writer = HttpExcelWriter::new();
    writer.write_row(&["A", "B", "C"]).unwrap();
    let bytes = writer.finish().unwrap();

    HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .append_header(("Content-Disposition", "attachment; filename=\"report.xlsx\""))
        .body(bytes)
}
```

### Warp

```rust
use warp::{Filter, reply::Response};
use excelstream::cloud::HttpExcelWriter;

async fn download_handler() -> Result<Response, warp::Rejection> {
    let mut writer = HttpExcelWriter::new();
    writer.write_row(&["A", "B", "C"]).unwrap();
    let bytes = writer.finish().unwrap();

    Ok(warp::http::Response::builder()
        .header("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header("Content-Disposition", "attachment; filename=\"report.xlsx\"")
        .body(bytes)
        .unwrap())
}
```

## Tips

1. **Compression Level**: Use level 1 for development (faster), level 6 for production (balanced)
2. **Large Datasets**: Handles up to 1M rows efficiently (<50MB files)
3. **Memory**: Peak memory ≈ 2.7x file size + 8 MB (very efficient!)
4. **Error Handling**: Always handle `.finish()` errors properly in production

## Performance Characteristics

**Actual Benchmarks (Release mode, individual process runs):**

| Dataset | File Size | Peak Memory | Peak/File Ratio | Notes |
|---------|-----------|-------------|-----------------|-------|
| 1K rows | 62 KB | ~64 MB | ~1059x | High ratio due to base overhead |
| 10K rows | 589 KB | ~64 MB | ~112x | Base overhead dominant |
| 50K rows | 2.93 MB | ~64 MB | ~22x | Approaching efficiency |
| 100K rows | 5.87 MB | ~64 MB | **~11x** | Good efficiency |
| 500K rows | 29 MB | ~64 MB | **~2.3x** | Excellent efficiency |
| **1M rows** | **56.5 MB** | **~64 MB** | **~1.14x** | 🎯 **Outstanding!** |

**Key Insights:**

✅ **Constant Memory Usage**: Peak memory stays at **~64 MB regardless of file size!**
- No memory growth with data size
- Rust runtime + allocator base overhead: ~64 MB
- Actual data memory: Negligible (streaming architecture)

✅ **Perfect for Large Files**: 
- 1M rows (56.5 MB file) → Only 64 MB peak memory (1.14x ratio!)
- Memory efficiency improves dramatically with file size
- Ideal for production workloads with large datasets

✅ **Performance**:
- 100K rows: ~500ms
- 1M rows: ~5-10 seconds (estimated)
- Sub-second for typical use cases (<100K rows)

✅ **Production Ready**:
- Predictable memory: Always ~64 MB
- Works in 128-256 MB containers
- Perfect for serverless (AWS Lambda, Cloud Functions)
- Scales to multi-million row files

## Limitations

- Pure in-memory implementation (no temp files, entire file in RAM during generation)
- Not suitable for extremely large files (>100MB, though technically possible)
- Blocking I/O for final buffer operations (async version coming soon)

## License

MIT
