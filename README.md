# excelstream

🦀 **High-performance Rust library for Excel import/export with streaming support**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KSD-CO/excelstream/workflows/Rust/badge.svg)](https://github.com/KSD-CO/excelstream/actions)

> **✨ What's New in v0.7.0:**
> - 🔒 **Worksheet Protection** - Protect sheets with password and granular permissions
> - 📐 **Cell Merging** - Merge cells horizontally and vertically (A1:C1, A1:A3)
> - 📏 **Column Width** - Set custom column widths (previously no-op, now functional)
> - 🎯 **Zero Overhead** - All new features maintain 15-25 MB memory and streaming performance


> **v0.6.1 Features:**
> - 🐛 **Leading Zero Bug Fixed** - String numbers like "090899" now preserve leading zeros
> - 🔧 **Improved Type Handling** - `write_row()` treats all values as strings (no auto number detection)
> - ✅ **Better Type Control** - Use `write_row_typed()` with `CellValue::Int/Float` for numbers
> - 🧠 **Hybrid SST in All Methods** - Memory optimization applied to all write functions
> - 💾 **Memory Verified** - All methods stay under 60 MB with realistic data (<80 MB target)

> **v0.5.1 Features:**
> - 🗜️ **Compression Level Configuration** - Control ZIP compression levels (0-9) for speed vs size trade-offs
> - ⚙️ **Flexible API** - Set compression at workbook creation or anytime during writing
> - ⚡ **Fast Mode** - Level 1 compression: 2x faster, suitable for development and testing
> - 📦 **Balanced Mode** - Level 3 compression: Good balance between speed and file size
> - 💾 **Production Mode** - Level 6 compression (default): Best file size for production exports
> - 🔧 **Memory Optimization** - PostgreSQL streaming with optimized batch sizes (500 rows)

> **v0.5.0 Features:**
> - 🚀 **Hybrid SST Optimization** - Intelligent selective deduplication for optimal memory usage
> - 💾 **Ultra-Low Memory** - 15-25 MB for 1M rows (was 125 MB), 89% reduction!
> - ⚡ **58% Faster** - 25K+ rows/sec with hybrid SST strategy
> - 🎯 **Smart Detection** - Numbers inline, long strings inline, only short repeated strings deduplicated

## ✨ Features

- 🚀 **Constant Memory Streaming** - Process millions of rows with only 15-25 MB RAM (no buffering!)
- 💾 **Ultra-Low Memory** - 89% less memory than alternatives (15 MB vs 125 MB for 1M rows)
- ⚡ **Blazing Fast** - 25K-70K rows/sec, 58% faster with hybrid SST optimization
- 🧠 **Smart Memory** - Intelligent deduplication: numbers inline, long strings inline, only repeated short strings deduplicated
- 🗜️ **Compression Control** - ZIP levels 0-9 for speed/size trade-offs (2x faster in dev mode)
- 🎨 **Cell Formatting** - 14 predefined styles (bold, currency, %, dates, highlights, borders)
- 📐 **Cell Merging** - Merge cells horizontally/vertically for headers and grouped data
- 📏 **Column Width & Row Height** - Full layout control for professional reports
- 🔒 **Worksheet Protection** - Password protection with 12 granular permissions
- 📐 **Formula Support** - Excel formulas (=SUM, =AVERAGE, =IF, etc.) calculate correctly
- 🎯 **Type Safety** - Strong typing: Int, Float, Bool, DateTime, Formula, String
- 🔧 **50+ Columns** - Handles complex schemas with mixed data types
- ❌ **Better Errors** - Context-rich error messages with debugging info
- 📊 **Multi-format** - Read XLSX, XLS, ODS; Write XLSX with full compatibility
- 🪟 **Cross-Platform** - Windows, Linux, macOS (tested on all three)
- 🐳 **K8s Ready** - Perfect for memory-limited containers (<512 MB RAM)
- ✅ **Production Proven** - 430K+ rows exported in production, 50+ tests, zero unsafe code

## 🎯 Why ExcelStream?

### The Problem with Traditional Excel Libraries

Most Excel libraries in Rust (and other languages) load entire files into memory:

```rust
// ❌ Traditional approach - Loads ENTIRE file into RAM
let workbook = Workbook::new("huge.xlsx")?;
for row in workbook.worksheet("Sheet1")?.rows() {
    // 1GB file = 1GB+ RAM usage!
}
```

**Problems:**
- 📈 Memory grows with file size (10MB file = 100MB+ RAM)
- 💥 OOM crashes on large files (>100MB)
- 🐌 Slow startup (must load everything first)
- 🔴 Impossible in containers (<512MB RAM)

### The ExcelStream Difference: Real Streaming

```rust
// ✅ ExcelStream - Constant 15-25 MB regardless of file size
let mut reader = ExcelReader::open("huge.xlsx")?;
for row in reader.rows("Sheet1")? {
    // 1GB file = still 15-25 MB RAM! 🚀
}
```

**Why This Matters:**

| Scenario | Traditional Library | ExcelStream | Benefit |
|----------|-------------------|-------------|---------|
| 10 MB file | 100 MB RAM | 15 MB RAM | **85% less memory** |
| 100 MB file | 1+ GB RAM | 15 MB RAM | **98% less memory** |
| 1 GB file | ❌ Crash | 25 MB RAM | ✅ **Works!** |
| K8s pod (<512MB) | ❌ OOMKilled | ✅ Works | **Container-ready** |

## 🚀 Real-World Use Cases

### 1. Processing Large Enterprise Files (>100 MB)

**Problem:** Sales team sends 500 MB Excel with 2M+ customer records. Traditional libraries crash.

```rust
use excelstream::reader::ExcelReader;

// ✅ Processes 2M rows with only 25 MB RAM
let mut reader = ExcelReader::open("customers_2M_rows.xlsx")?;
let mut total_revenue = 0.0;

for row in reader.rows("Sales")? {
    let row = row?;
    if let Some(amount) = row.get(5).and_then(|c| c.as_f64()) {
        total_revenue += amount;
    }
    // Memory stays constant! No accumulation!
}

println!("Total: ${:.2}", total_revenue);
```

**Why ExcelStream wins:**
- ✅ Constant 25 MB memory (traditional = 5+ GB)
- ✅ Processes row-by-row (no buffering)
- ✅ Works in K8s pods with 512 MB limit
- ⚡ Starts processing immediately (no load delay)

### 2. Daily Database Exports (Production ETL)

**Problem:** Export 430K+ invoice records to Excel every night. Must fit in 512 MB pod.

```rust
use excelstream::ExcelWriter;
use postgres::{Client, NoTls};

// ✅ Real production code - 430,099 rows in 94 seconds
let mut writer = ExcelWriter::with_compression("invoices.xlsx", 3)?;
writer.set_flush_interval(500);  // Flush every 500 rows

let mut client = Client::connect("postgresql://...", NoTls)?;
let mut tx = client.transaction()?;
tx.execute("DECLARE cursor CURSOR FOR SELECT * FROM invoices", &[])?;

loop {
    let rows = tx.query("FETCH 500 FROM cursor", &[])?;
    if rows.is_empty() { break; }
    
    for row in rows {
        writer.write_row_typed(&[
            CellValue::Int(row.get(0)),
            CellValue::String(row.get(1)),
            CellValue::Float(row.get(2)),
        ])?;
    }
}

writer.save()?; // 62 MB file, used only 25 MB RAM
```

**Production Results:**
- ✅ 430K rows exported successfully
- ✅ Peak memory: 25 MB (traditional = 500+ MB)
- ✅ Duration: 94 seconds (4,567 rows/sec)
- ✅ Runs nightly in K8s pod (512 MB limit)
- 🐳 Zero OOMKilled errors

### 3. Real-Time Streaming Exports (No Wait Time)

**Problem:** User clicks "Export" button. Traditional libraries must load ALL data first = 30+ second wait.

```rust
use excelstream::ExcelWriter;
use tokio_stream::StreamExt;

// ✅ Stream directly from async query - starts writing immediately!
let mut writer = ExcelWriter::new("report.xlsx")?;
writer.write_header_bold(&["Date", "User", "Action"])?;

let mut stream = db.query_stream("SELECT * FROM audit_log").await?;

// User sees progress immediately! No 30-second wait!
while let Some(row) = stream.next().await {
    let row = row?;
    writer.write_row(&[
        row.get("date"),
        row.get("user"),
        row.get("action"),
    ])?;
    // Every 100 rows = visible progress!
}

writer.save()?;
```

**User Experience:**
- ✅ Instant feedback (not 30-second blank screen)
- ✅ Progress bar possible (count rows written)
- ✅ Cancellable (user can abort early)
- 🚀 Feels 10x faster (starts immediately)

### 4. Kubernetes CronJobs (Memory-Limited)

**Problem:** K8s pods have 256-512 MB limits. Traditional libraries need 2+ GB for large exports.

```rust
use excelstream::ExcelWriter;

// ✅ Optimized for K8s - uses only 15 MB!
let mut writer = ExcelWriter::with_compression("export.xlsx", 1)?;
writer.set_flush_interval(100);      // Aggressive flushing
writer.set_max_buffer_size(256_000); // 256 KB buffer

// Export 1M rows in 256 MB pod - impossible with traditional libraries!
for i in 0..1_000_000 {
    writer.write_row(&[
        &i.to_string(),
        &format!("data_{}", i),
    ])?;
}

writer.save()?;
```

**K8s Benefits:**
- ✅ Works in 256 MB pods (traditional needs 2+ GB)
- ✅ Predictable memory (no spikes or OOM)
- ✅ Fast compression (level 1 = 2x faster)
- 🐳 Perfect for cost-optimized clusters

### 5. Multi-Tenant SaaS Exports

**Problem:** 100 concurrent users export reports. Traditional = 100 × 500 MB = 50 GB RAM!

```rust
use excelstream::ExcelWriter;

// ✅ Each export uses only 20 MB
async fn export_for_user(user_id: i64) -> Result<()> {
    let mut writer = ExcelWriter::new(&format!("user_{}.xlsx", user_id))?;
    
    let records = db.query("SELECT * FROM data WHERE user_id = ?", user_id).await?;
    for rec in records {
        writer.write_row_typed(&[
            CellValue::Int(rec.id),
            CellValue::String(rec.name),
        ])?;
    }
    
    writer.save()?;
    Ok(())
}

// 100 concurrent exports = 100 × 20 MB = 2 GB (not 50 GB!)
```

**SaaS Benefits:**
- ✅ 100 concurrent users = 2 GB (traditional = 50+ GB)
- ✅ Scales horizontally (predictable memory)
- ✅ No "export queue" needed
- 💰 Lower infrastructure costs

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
excelstream = "0.7"
```

**Latest version:** `0.7.0` - Worksheet protection, cell merging, functional column widths

## 🚀 Quick Start

### Reading Excel Files (Streaming)

```rust
use excelstream::reader::ExcelReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    // List all sheets
    for sheet_name in reader.sheet_names() {
        println!("Sheet: {}", sheet_name);
    }
    
    // Read rows one by one (streaming)
    for row_result in reader.rows("Sheet1")? {
        let row = row_result?;
        println!("Row {}: {:?}", row.index, row.to_strings());
    }
    
    Ok(())
}
```

### Writing Excel Files (Streaming - v0.2.0)

```rust
use excelstream::writer::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("output.xlsx")?;
    
    // Configure streaming behavior (optional)
    writer.set_flush_interval(500);  // Flush every 500 rows
    writer.set_max_buffer_size(512 * 1024);  // 512KB buffer
    
    // Write header (note: no bold formatting in v0.2.0)
    writer.write_header(&["ID", "Name", "Email"])?;
    
    // Write millions of rows with constant memory usage!
    for i in 1..=1_000_000 {
        writer.write_row(&[
            &i.to_string(),
            &format!("User{}", i),
            &format!("user{}@example.com", i)
        ])?;
    }
    
    // Save file (closes ZIP and finalizes)
    writer.save()?;
    
    Ok(())
}
```

**Key Benefits:**
- ✅ Constant ~80MB memory usage regardless of dataset size
- ✅ High throughput: 60K-70K rows/sec (UltraLowMemoryWorkbook fastest at 69.5K)
- ✅ Direct ZIP streaming - data written to disk immediately
- ✅ Full formatting support: bold, styles, column widths, row heights

### Writing with Typed Values (Recommended)

For better Excel compatibility and performance, use typed values:

```rust
use excelstream::writer::ExcelWriter;
use excelstream::types::CellValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("typed_output.xlsx")?;

    writer.write_header(&["Name", "Age", "Salary", "Active"])?;

    // Typed values: numbers are numbers, formulas work in Excel
    writer.write_row_typed(&[
        CellValue::String("Alice".to_string()),
        CellValue::Int(30),
        CellValue::Float(75000.50),
        CellValue::Bool(true),
    ])?;

    writer.save()?;
    Ok(())
}
```

**Benefits of `write_row_typed()`:**
- ✅ Numbers are stored as numbers (not text)
- ✅ Booleans display as TRUE/FALSE
- ✅ Excel formulas work correctly
- ✅ Better type safety
- ✅ Excellent performance: 62.7K rows/sec (+2% faster than string-based)

### Preserving Leading Zeros (Phone Numbers, IDs)

**New in v0.6.1:** Proper handling of string numbers with leading zeros!

#### Problem: Leading Zeros Lost

```rust
// ❌ WRONG: Auto number detection loses leading zeros
writer.write_row(&["090899"]);  // Displays as 90899 in Excel
```

#### Solution 1: Use `write_row()` (All Strings)

```rust
// ✅ CORRECT: write_row() treats ALL values as strings
writer.write_row(&["090899", "00123", "ID-00456"]);  
// Displays: "090899", "00123", "ID-00456" ✅ Leading zeros preserved!
```

**As of v0.6.1**, `write_row()` no longer auto-detects numbers. All values are treated as strings.

#### Solution 2: Use `write_row_typed()` with Explicit Types

```rust
use excelstream::types::CellValue;

// ✅ String type: preserves leading zeros
writer.write_row_typed(&[
    CellValue::String("090899".to_string()),  // Phone: "090899" ✅
    CellValue::String("00123".to_string()),   // ID: "00123" ✅
])?;

// ✅ Int type: actual number (no leading zero)
writer.write_row_typed(&[
    CellValue::Int(90899),  // Number: 90899 (no leading zero)
    CellValue::Int(123),    // Number: 123
])?;
```

#### Solution 3: Use `write_row_styled()` for Full Control

```rust
use excelstream::types::{CellValue, CellStyle};

writer.write_row_styled(&[
    (CellValue::String("090899".to_string()), CellStyle::Default),  // Preserves "090899"
    (CellValue::Int(1234), CellStyle::NumberInteger),               // Formats as "1,234"
])?;
```

**Type Handling Summary:**

| Method | String "090899" | Int 90899 | Use When |
|--------|----------------|-----------|----------|
| `write_row(&[&str])` | "090899" ✅ | N/A | All data is text (IDs, codes) |
| `write_row_typed(CellValue)` | "090899" ✅ | 90899 (number) | Mixed types, explicit control |
| `write_row_styled()` | "090899" ✅ | 90899 (number) | Need formatting + type control |

**Best Practice:** 
- Phone numbers, IDs, ZIP codes → Use `CellValue::String` or `write_row()`
- Amounts, quantities, calculations → Use `CellValue::Int` or `CellValue::Float`

### Writing Excel Formulas

```rust
use excelstream::writer::ExcelWriter;
use excelstream::types::CellValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("formulas.xlsx")?;

    // Header row
    writer.write_header(&["Value 1", "Value 2", "Sum", "Average"])?;

    // Data with formulas
    writer.write_row_typed(&[
        CellValue::Int(10),
        CellValue::Int(20),
        CellValue::Formula("=A2+B2".to_string()),      // Sum
        CellValue::Formula("=AVERAGE(A2:B2)".to_string()), // Average
    ])?;

    writer.write_row_typed(&[
        CellValue::Int(15),
        CellValue::Int(25),
        CellValue::Formula("=A3+B3".to_string()),
        CellValue::Formula("=AVERAGE(A3:B3)".to_string()),
    ])?;

    // Total row with SUM formula
    writer.write_row_typed(&[
        CellValue::String("Total".to_string()),
        CellValue::Empty,
        CellValue::Formula("=SUM(C2:C3)".to_string()),
        CellValue::Formula("=AVERAGE(D2:D3)".to_string()),
    ])?;

    writer.save()?;
    Ok(())
}
```

**Supported Formulas:**
- ✅ Basic arithmetic: `=A1+B1`, `=A1*B1`, `=A1/B1`
- ✅ SUM, AVERAGE, COUNT, MIN, MAX
- ✅ Cell ranges: `=SUM(A1:A10)`
- ✅ Complex formulas: `=IF(A1>100, "High", "Low")`
- ✅ All standard Excel functions

### Cell Formatting and Styling

**New in v0.3.0:** Apply 14 predefined cell styles including bold headers, number formats, highlights, and borders!

#### Bold Headers

```rust
use excelstream::writer::ExcelWriter;

let mut writer = ExcelWriter::new("report.xlsx")?;

// Write bold header
writer.write_header_bold(&["Name", "Amount", "Status"])?;

// Regular data rows
writer.write_row(&["Alice", "1,234.56", "Active"])?;
writer.write_row(&["Bob", "2,345.67", "Pending"])?;

writer.save()?;
```

#### Styled Cells

Apply different styles to individual cells:

```rust
use excelstream::types::{CellValue, CellStyle};
use excelstream::writer::ExcelWriter;

let mut writer = ExcelWriter::new("report.xlsx")?;

writer.write_header_bold(&["Item", "Amount", "Change %"])?;

// Mix different styles in one row
writer.write_row_styled(&[
    (CellValue::String("Revenue".to_string()), CellStyle::TextBold),
    (CellValue::Float(150000.00), CellStyle::NumberCurrency),
    (CellValue::Float(0.15), CellStyle::NumberPercentage),
])?;

writer.write_row_styled(&[
    (CellValue::String("Profit".to_string()), CellStyle::HighlightGreen),
    (CellValue::Float(55000.00), CellStyle::NumberCurrency),
    (CellValue::Float(0.22), CellStyle::NumberPercentage),
])?;

writer.save()?;
```

#### Uniform Row Styling

Apply the same style to all cells in a row:

```rust
// All cells bold
writer.write_row_with_style(&[
    CellValue::String("IMPORTANT".to_string()),
    CellValue::String("READ THIS".to_string()),
    CellValue::String("URGENT".to_string()),
], CellStyle::TextBold)?;

// All cells highlighted yellow
writer.write_row_with_style(&[
    CellValue::String("Warning".to_string()),
    CellValue::String("Check values".to_string()),
    CellValue::String("Need review".to_string()),
], CellStyle::HighlightYellow)?;
```

#### Available Cell Styles

| Style | Format Code | Example | Use Case |
|-------|------------|---------|----------|
| `CellStyle::Default` | None | Plain text | Regular data |
| `CellStyle::HeaderBold` | Bold | **Header** | Column headers |
| `CellStyle::NumberInteger` | #,##0 | 1,234 | Whole numbers |
| `CellStyle::NumberDecimal` | #,##0.00 | 1,234.56 | Decimals |
| `CellStyle::NumberCurrency` | $#,##0.00 | $1,234.56 | Money amounts |
| `CellStyle::NumberPercentage` | 0.00% | 95.00% | Percentages |
| `CellStyle::DateDefault` | MM/DD/YYYY | 01/15/2024 | Dates |
| `CellStyle::DateTimestamp` | MM/DD/YYYY HH:MM:SS | 01/15/2024 14:30:00 | Timestamps |
| `CellStyle::TextBold` | Bold | **Bold text** | Emphasis |
| `CellStyle::TextItalic` | Italic | *Italic text* | Notes |
| `CellStyle::HighlightYellow` | Yellow bg | 🟨 Text | Warnings |
| `CellStyle::HighlightGreen` | Green bg | 🟩 Text | Success |
| `CellStyle::HighlightRed` | Red bg | 🟥 Text | Errors |
| `CellStyle::BorderThin` | Thin borders | ▭ Text | Boundaries |

#### Complete Example

```rust
use excelstream::writer::ExcelWriter;
use excelstream::types::{CellValue, CellStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("quarterly_report.xlsx")?;

    // Bold header
    writer.write_header_bold(&["Quarter", "Revenue", "Expenses", "Profit", "Margin %"])?;

    // Q1 - Green highlight for good performance
    writer.write_row_styled(&[
        (CellValue::String("Q1 2024".to_string()), CellStyle::Default),
        (CellValue::Float(500000.00), CellStyle::NumberCurrency),
        (CellValue::Float(320000.00), CellStyle::NumberCurrency),
        (CellValue::Float(180000.00), CellStyle::NumberCurrency),
        (CellValue::Float(0.36), CellStyle::NumberPercentage),
    ])?;

    // Q2 - Yellow highlight for warning
    writer.write_row_styled(&[
        (CellValue::String("Q2 2024".to_string()), CellStyle::Default),
        (CellValue::Float(450000.00), CellStyle::NumberCurrency),
        (CellValue::Float(380000.00), CellStyle::NumberCurrency),
        (CellValue::Float(70000.00), CellStyle::HighlightYellow),
        (CellValue::Float(0.16), CellStyle::NumberPercentage),
    ])?;

    // Total row with formulas and bold
    writer.write_row_styled(&[
        (CellValue::String("Total".to_string()), CellStyle::TextBold),
        (CellValue::Formula("=SUM(B2:B3)".to_string()), CellStyle::NumberCurrency),
        (CellValue::Formula("=SUM(C2:C3)".to_string()), CellStyle::NumberCurrency),
        (CellValue::Formula("=SUM(D2:D3)".to_string()), CellStyle::NumberCurrency),
        (CellValue::Formula("=AVERAGE(E2:E3)".to_string()), CellStyle::NumberPercentage),
    ])?;

    writer.save()?;
    Ok(())
}
```

**See also:** Run `cargo run --example cell_formatting` to see all 14 styles in action!

### Column Width, Row Height, and Cell Merging

**New in v0.7.0:** Full layout control with column widths, row heights, and cell merging!

#### Column Width

Set column widths **before** writing any rows:

```rust
use excelstream::writer::ExcelWriter;

let mut writer = ExcelWriter::new("report.xlsx")?;

// Set column widths BEFORE writing rows
writer.set_column_width(0, 25.0)?;  // Column A = 25 units wide
writer.set_column_width(1, 12.0)?;  // Column B = 12 units wide
writer.set_column_width(2, 15.0)?;  // Column C = 15 units wide

// Now write rows
writer.write_header_bold(&["Product Name", "Quantity", "Price"])?;
writer.write_row(&["Laptop", "5", "$1,200.00"])?;

writer.save()?;
```

**Important:**
- ⚠️ Column widths must be set **before** writing any rows
- Default column width is 8.43 units
- One unit ≈ width of one character in default font
- Typical range: 8-50 units

#### Row Height

Set row height for the next row to be written:

```rust
use excelstream::writer::ExcelWriter;

let mut writer = ExcelWriter::new("report.xlsx")?;

// Set height for header row (taller)
writer.set_next_row_height(25.0)?;
writer.write_header_bold(&["Name", "Age", "Email"])?;

// Regular row (default height)
writer.write_row(&["Alice", "30", "alice@example.com"])?;

// Set height for next row
writer.set_next_row_height(40.0)?;
writer.write_row(&["Bob", "25", "bob@example.com"])?;

writer.save()?;
```

**Important:**
- Height is in points (1 point = 1/72 inch)
- Default row height is 15 points
- Typical range: 10-50 points
- Setting is consumed by next `write_row()` call

#### Complete Example with Sizing

```rust
use excelstream::writer::ExcelWriter;
use excelstream::types::{CellValue, CellStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("sales_report.xlsx")?;

    // Set column widths
    writer.set_column_width(0, 25.0)?; // Product name - wider
    writer.set_column_width(1, 12.0)?; // Quantity
    writer.set_column_width(2, 15.0)?; // Price
    writer.set_column_width(3, 15.0)?; // Total

    // Tall header row
    writer.set_next_row_height(25.0)?;
    writer.write_header_bold(&["Product", "Quantity", "Price", "Total"])?;

    // Data rows
    writer.write_row_styled(&[
        (CellValue::String("Laptop".to_string()), CellStyle::Default),
        (CellValue::Int(5), CellStyle::NumberInteger),
        (CellValue::Float(1200.00), CellStyle::NumberCurrency),
        (CellValue::Formula("=B2*C2".to_string()), CellStyle::NumberCurrency),
    ])?;

    // Total row with custom height
    writer.set_next_row_height(22.0)?;
    writer.write_row_styled(&[
        (CellValue::String("TOTAL".to_string()), CellStyle::TextBold),
        (CellValue::Formula("=SUM(B2:B4)".to_string()), CellStyle::NumberInteger),
        (CellValue::String("".to_string()), CellStyle::Default),
        (CellValue::Formula("=SUM(D2:D4)".to_string()), CellStyle::NumberCurrency),
    ])?;

    writer.save()?;
    Ok(())
}
```

#### Cell Merging

Merge cells horizontally (for titles) or vertically (for grouped data):

```rust
use excelstream::ExcelWriter;
use excelstream::types::{CellValue, CellStyle};

let mut writer = ExcelWriter::new("report.xlsx")?;

// Set column widths
writer.set_column_width(1, 30.0)?;
writer.set_column_width(2, 15.0)?;

// Title row spanning 3 columns
writer.write_row_styled(&[
    (CellValue::String("Q4 Sales Report".to_string()), CellStyle::HeaderBold),
])?;
writer.merge_cells("A1:C1")?; // Horizontal merge

writer.write_row(&[""])?; // Empty row

// Headers
writer.write_header_bold(&["Region", "City", "Sales"])?;

// Data with vertical merge for region
writer.write_row(&["North", "Boston", "125,000"])?;
writer.write_row(&["", "New York", "245,000"])?;
writer.write_row(&["", "Chicago", "198,000"])?;
writer.merge_cells("A4:A6")?; // Vertical merge - "North" spans 3 rows

writer.save()?;
```

**Common Patterns:**
- **Title rows**: `merge_cells("A1:F1")` - Header spanning all columns
- **Grouped data**: `merge_cells("A2:A5")` - Category name for multiple items
- **Subtotals**: `merge_cells("A10:C10")` - "Total" label spanning columns

**See also:** 
- `cargo run --example column_width_row_height` - Layout control demo
- `cargo run --example column_merge_demo` - Complete merging examples

### Direct FastWorkbook Usage (Maximum Performance)

For maximum performance, use `FastWorkbook` directly:

```rust
use excelstream::fast_writer::FastWorkbook;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new("large_output.xlsx")?;
    workbook.add_worksheet("Sheet1")?;
    
    // Write header
    workbook.write_row(&["ID", "Name", "Email", "Age"])?;
    
    // Write 1 million rows efficiently (40K rows/sec)
    for i in 1..=1_000_000 {
        workbook.write_row(&[
            &i.to_string(),
            &format!("User{}", i),
            &format!("user{}@example.com", i),
            &(20 + (i % 50)).to_string(),
        ])?;
    }
    
    workbook.close()?;
    Ok(())
}
```

### 🧠 Hybrid SST Optimization (v0.5.0)

**New in v0.5.0:** Intelligent selective deduplication for optimal memory usage!

#### How It Works

The Hybrid Shared String Table (SST) strategy intelligently decides which strings to deduplicate:

```rust
// Automatic optimization - no code changes needed!
let mut workbook = FastWorkbook::new("output.xlsx")?;
workbook.add_worksheet("Data")?;

// Numbers → Inline as number type (no SST)
workbook.write_row(&["123", "456.78", "999"])?;

// Long strings (>50 chars) → Inline (usually unique)
workbook.write_row(&["This is a very long description that exceeds 50 characters..."])?;

// Short repeated strings → SST (efficient deduplication)
workbook.write_row(&["Active", "Pending", "Active", "Completed"])?;
```

#### Memory Improvements

| Workbook Type | Before v0.5.0 | After v0.5.0 | Reduction |
|---------------|---------------|--------------|-----------|
| Simple (5 cols, 1M rows) | 49 MB | **18.8 MB** | **62%** |
| Medium (19 cols, 1M rows) | 125 MB | **15.4 MB** | **88%** |
| Complex (50 cols, 100K rows) | ~200 MB | **22.7 MB** | **89%** |
| Multi-workbook (4 × 100K rows) | 251 MB | **25.3 MB** | **90%** |

#### Strategy Details

The hybrid approach uses these rules:

1. **Numbers** (`123`, `456.78`) → Inline as `<c t="n">` (no SST)
2. **Long strings** (>50 chars) → Inline as `<c t="inlineStr">` (usually unique)
3. **SST Full** (>100k unique) → New strings inline (graceful degradation)
4. **Short strings** (≤50 chars) → SST for deduplication (efficient)

#### Performance Impact

```
ExcelWriter.write_row():       16,250 rows/sec (baseline)
ExcelWriter.write_row_typed(): 19,642 rows/sec (+21%)
ExcelWriter.write_row_styled(): 18,581 rows/sec (+14%)
FastWorkbook (hybrid SST):     25,682 rows/sec (+58%) ⚡
```

**Key Benefits:**
- ✅ **89% less memory** for complex workbooks
- ✅ **58% faster** due to fewer SST lookups
- ✅ **Handles 50+ columns** with mixed data types
- ✅ **Automatic** - no API changes required
- ✅ **Graceful degradation** - caps at 100k unique strings

**See also:** `HYBRID_SST_OPTIMIZATION.md` for technical details

### 🔒 Worksheet Protection (v0.7.0)

**New in v0.7.0:** Protect worksheets with passwords and granular permissions!

#### Basic Protection

```rust
use excelstream::{ExcelWriter, ProtectionOptions};

let mut writer = ExcelWriter::new("protected.xlsx")?;

// Protect with password - users can view but not edit
let protection = ProtectionOptions::new()
    .with_password("secret123");

writer.protect_sheet(protection)?;
writer.write_header_bold(&["Protected", "Data"])?;
writer.write_row(&["Cannot", "Edit"])?;

writer.save()?;
```

#### Granular Permissions

Control exactly what users can do:

```rust
use excelstream::{ExcelWriter, ProtectionOptions};

let mut writer = ExcelWriter::new("template.xlsx")?;

// Allow formatting but prevent data changes
let protection = ProtectionOptions::new()
    .with_password("format123")
    .allow_select_locked_cells(true)
    .allow_select_unlocked_cells(true)
    .allow_format_cells(true)      // ✅ Can format
    .allow_format_columns(true)     // ✅ Can resize columns
    .allow_format_rows(true);       // ✅ Can resize rows
    // Everything else is protected (insert, delete, edit)

writer.protect_sheet(protection)?;
writer.save()?;
```

#### Data Entry Forms

Allow users to insert/delete rows but protect headers:

```rust
let protection = ProtectionOptions::new()
    .with_password("data456")
    .allow_insert_rows(true)        // ✅ Can add rows
    .allow_delete_rows(true)        // ✅ Can delete rows
    .allow_sort(true);              // ✅ Can sort data

writer.protect_sheet(protection)?;

// Headers are protected, but users can add data rows
writer.write_header_bold(&["Name", "Email", "Phone"])?;
writer.write_row(&["Alice", "alice@example.com", "555-0001"])?;

writer.save()?;
```

#### Available Permissions

| Permission | Description | Use Case |
|-----------|-------------|----------|
| `allow_select_locked_cells` | Can select protected cells | View-only (default: true) |
| `allow_select_unlocked_cells` | Can select editable cells | Data entry (default: true) |
| `allow_format_cells` | Can change cell formats | Template customization |
| `allow_format_columns` | Can resize columns | Layout adjustments |
| `allow_format_rows` | Can resize rows | Layout adjustments |
| `allow_insert_rows` | Can insert new rows | Data entry forms |
| `allow_delete_rows` | Can delete rows | Data cleanup |
| `allow_insert_columns` | Can insert new columns | Schema changes |
| `allow_delete_columns` | Can delete columns | Schema changes |
| `allow_sort` | Can sort data | Data analysis |
| `allow_auto_filter` | Can use filters | Data analysis |

**Common Use Cases:**
- **Templates**: Protect formulas, allow data entry
- **Reports**: Lock everything (read-only)
- **Data Collection**: Allow insert/delete rows, protect headers
- **Shared Sheets**: Allow formatting, prevent structure changes

**See also:** `cargo run --example worksheet_protection` - Complete protection demo

## 🗜️ Compression Level Configuration (v0.5.1)

**New in v0.5.1:** Control ZIP compression levels to balance speed vs file size!

### Understanding Compression Levels

Excel files (.xlsx) are ZIP archives. ExcelStream lets you control the compression level:

| Level | Speed | File Size | Use Case | Recommended For |
|-------|-------|-----------|----------|-----------------|
| **0** | Fastest | Largest (no compression) | Debugging only | Testing |
| **1** | Very Fast ⚡ | ~2x larger | Fast exports | Development, testing |
| **3** | Fast | Balanced | Good compromise | CI/CD pipelines |
| **6** | Moderate | Smallest 📦 | Best compression | Production exports |
| **9** | Slowest | Smallest | Maximum compression | Archives, long-term storage |

**Default:** Level 6 (balanced performance and file size)

### Setting Compression Level

#### Method 1: At Workbook Creation

```rust
use excelstream::writer::ExcelWriter;

// Create writer with fast compression (level 1)
let mut writer = ExcelWriter::with_compression("output.xlsx", 1)?;

writer.write_header(&["ID", "Name", "Amount"])?;
writer.write_row(&["1", "Alice", "1000"])?;
writer.save()?;
```

#### Method 2: After Creation

```rust
use excelstream::writer::ExcelWriter;

let mut writer = ExcelWriter::new("output.xlsx")?;

// Change compression level
writer.set_compression_level(3); // Fast balanced compression

writer.write_header(&["ID", "Name"])?;
writer.write_row(&["1", "Alice"])?;
writer.save()?;
```

#### Method 3: With UltraLowMemoryWorkbook

```rust
use excelstream::fast_writer::UltraLowMemoryWorkbook;

let mut workbook = UltraLowMemoryWorkbook::with_compression("output.xlsx", 1)?;
workbook.add_worksheet("Data")?;

workbook.write_row(&["Header1", "Header2"])?;
workbook.write_row(&["Value1", "Value2"])?;

workbook.close()?;
```

#### Method 4: Environment-Based Configuration

```rust
use excelstream::writer::ExcelWriter;

// Use fast compression for debug builds, production compression for release
let compression_level = if cfg!(debug_assertions) { 1 } else { 6 };
let mut writer = ExcelWriter::with_compression("output.xlsx", compression_level)?;

writer.write_header(&["Data"])?;
writer.write_row(&["Value"])?;
writer.save()?;
```

### Real-World Performance (1M rows)

Tested with 1 million rows × 4 columns on production hardware:

| Configuration | Flush Interval | Buffer Size | Compression | Time | File Size | Memory |
|--------------|----------------|-------------|-------------|------|-----------|--------|
| **Aggressive** | 100 | 256 KB | Level 1 | **3.93s** ⚡ | 172 MB | <30 MB |
| **Balanced** | 500 | 512 KB | Level 3 | **5.03s** | 110 MB | <30 MB |
| **Default** | 1000 | 1 MB | Level 6 | **7.37s** | **88 MB** 📦 | <30 MB |
| **Conservative** | 5000 | 2 MB | Level 6 | 8.00s | 88 MB | <30 MB |

**Key Findings:**
- Level 1 is **~2x faster** but files are **~2x larger** than level 6
- Level 3 provides a **good balance** between speed and size
- Memory usage is **constant <30 MB** regardless of compression level
- Production exports typically use level 6 for optimal file size

### Complete Example: Configurable Compression

```rust
use excelstream::writer::ExcelWriter;

fn export_data(compression: u32) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("export_level_{}.xlsx", compression);
    let mut writer = ExcelWriter::with_compression(&filename, compression)?;
    
    // Optional: Combine with memory optimization
    if compression <= 1 {
        // Fast compression - flush more aggressively
        writer.set_flush_interval(100);
        writer.set_max_buffer_size(256 * 1024);
    } else if compression <= 3 {
        // Balanced compression
        writer.set_flush_interval(500);
        writer.set_max_buffer_size(512 * 1024);
    } else {
        // Production compression - use larger buffers
        writer.set_flush_interval(1000);
        writer.set_max_buffer_size(1024 * 1024);
    }
    
    writer.write_header(&["ID", "Name", "Email", "Status"])?;
    
    for i in 1..=1_000_000 {
        writer.write_row(&[
            &i.to_string(),
            &format!("User{}", i),
            &format!("user{}@example.com", i),
            if i % 3 == 0 { "Active" } else { "Pending" },
        ])?;
    }
    
    writer.save()?;
    println!("Exported with compression level {}: {}", compression, filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test different compression levels
    export_data(1)?; // Fast: ~4s, 172 MB
    export_data(3)?; // Balanced: ~5s, 110 MB
    export_data(6)?; // Production: ~7s, 88 MB
    
    Ok(())
}
```

### Recommendations

**For Development & Testing:**
```rust
let mut writer = ExcelWriter::with_compression("test.xlsx", 1)?;
writer.set_flush_interval(100);
```
- ✅ Fast exports (2x speed improvement)
- ✅ Quick iteration cycles
- ⚠️ Larger files (not for production)

**For CI/CD Pipelines:**
```rust
let mut writer = ExcelWriter::with_compression("report.xlsx", 3)?;
writer.set_flush_interval(500);
```
- ✅ Good balance of speed and size
- ✅ Reasonable export times
- ✅ Acceptable file sizes

**For Production Exports:**
```rust
let mut writer = ExcelWriter::with_compression("export.xlsx", 6)?; // Default
writer.set_flush_interval(1000);
```
- ✅ Smallest file size
- ✅ Best for network transfers
- ✅ Optimal for storage

**For Archives:**
```rust
let mut writer = ExcelWriter::with_compression("archive.xlsx", 9)?;
```
- ✅ Maximum compression
- ⚠️ Slower export times
- 📦 Best for long-term storage

**See also:** Run `cargo run --example compression_level_config` for a complete demonstration!

### Memory-Constrained Writing (For Kubernetes Pods)

With v0.5.0+ and compression configuration (v0.5.1), memory usage is ultra-low:

```rust
use excelstream::writer::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::with_compression("output.xlsx", 1)?; // Fast compression
    
    // For pods with < 512MB RAM (optimized configuration)
    writer.set_flush_interval(500);       // Flush every 500 rows
    writer.set_max_buffer_size(512 * 1024); // 512KB buffer
    
    writer.write_header(&["ID", "Name", "Email"])?;
    
    // Write large dataset without OOMKilled
    for i in 1..=1_000_000 {
        writer.write_row(&[
            &i.to_string(),
            &format!("User{}", i),
            &format!("user{}@example.com", i),
        ])?;
    }
    
    writer.save()?;
    Ok(())
}
```

**Memory usage in v0.5.1:**
- **10-30 MB peak** with optimized settings (was 80-100 MB)
- **80-90% memory reduction** vs v0.4.x
- Handles 50+ columns with mixed data types
- Suitable for Kubernetes pods with limited resources
- Automatic hybrid SST optimization

**Configuration Presets:**

| Preset | Flush | Buffer | Compression | Use Case | Memory Peak |
|--------|-------|--------|-------------|----------|-------------|
| **Aggressive** | 100 | 256 KB | Level 1 | <256 MB RAM pods | 10-15 MB |
| **Balanced** | 500 | 512 KB | Level 3 | <512 MB RAM pods | 15-20 MB |
| **Default** | 1000 | 1 MB | Level 6 | Standard pods | 20-30 MB |
| **Conservative** | 5000 | 2 MB | Level 6 | High-memory pods | 25-35 MB |

### PostgreSQL Streaming Export (Production-Tested)

ExcelStream has been tested with real production databases. Example: 430,099 e-invoice records exported successfully.

```rust
use excelstream::writer::ExcelWriter;
use postgres::{Client, NoTls};

fn export_database_to_excel() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PostgreSQL
    let mut client = Client::connect(
        "postgresql://user:password@host:5432/database",
        NoTls,
    )?;
    
    // Create Excel writer with optimized settings
    let mut writer = ExcelWriter::with_compression("export.xlsx", 3)?;
    
    // Memory-optimized for large datasets
    writer.set_flush_interval(500);
    writer.set_max_buffer_size(512 * 1024);
    
    // Write header
    writer.write_header_bold(&[
        "ID", "Date", "Invoice Number", "Customer", "Amount", "Status"
    ])?;
    
    // Use cursor for streaming (handles millions of rows)
    let mut transaction = client.transaction()?;
    transaction.execute("DECLARE export_cursor CURSOR FOR SELECT * FROM invoices", &[])?;
    
    let mut total_rows = 0u64;
    let batch_size = 500; // Optimized batch size
    
    loop {
        let rows = transaction.query(
            &format!("FETCH {} FROM export_cursor", batch_size),
            &[],
        )?;
        
        if rows.is_empty() {
            break;
        }
        
        for row in rows {
            // Handle NULL values properly
            let id: i64 = row.get(0);
            let date: Option<String> = row.try_get(1).ok().flatten();
            let invoice_no: Option<String> = row.try_get(2).ok().flatten();
            let customer: Option<String> = row.try_get(3).ok().flatten();
            let amount: Option<f64> = row.try_get(4).ok().flatten();
            let status: Option<String> = row.try_get(5).ok().flatten();
            
            writer.write_row(&[
                &id.to_string(),
                &date.unwrap_or_default(),
                &invoice_no.unwrap_or_default(),
                &customer.unwrap_or_default(),
                &amount.map(|a| format!("{:.2}", a)).unwrap_or_default(),
                &status.unwrap_or_default(),
            ])?;
            
            total_rows += 1;
            
            if total_rows % 10_000 == 0 {
                println!("Exported {} rows...", total_rows);
            }
        }
    }
    
    transaction.execute("CLOSE export_cursor", &[])?;
    transaction.commit()?;
    
    writer.save()?;
    println!("✅ Exported {} rows successfully!", total_rows);
    Ok(())
}
```

**Production Results (430K rows):**
- **Duration:** 1m 34s (94.17 seconds)
- **Throughput:** 4,567 rows/sec
- **File Size:** 62.22 MB
- **Memory Peak:** <30 MB
- **Columns:** 25 mixed data types (int, float, text, dates)

**Key Optimizations:**
- ✅ Cursor-based streaming (no full table load)
- ✅ Small batch size (500 rows) for memory efficiency
- ✅ Proper NULL handling with `try_get().ok().flatten()`
- ✅ Fast compression (level 3) for balanced performance
- ✅ Frequent flushing (500 rows) to disk

**See also:** `examples/postgres_streaming.rs` for complete implementation

### Multi-sheet workbook

```rust
use excelstream::writer::ExcelWriterBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriterBuilder::new("multi.xlsx")
        .with_sheet_name("Sales")
        .build()?;
    
    // Sheet 1: Sales
    writer.write_header(&["Month", "Revenue"])?;
    writer.write_row(&["Jan", "50000"])?;
    
    // Sheet 2: Employees
    writer.add_sheet("Employees")?;
    writer.write_header(&["Name", "Department"])?;
    writer.write_row(&["Alice", "Engineering"])?;
    
    writer.save()?;
    Ok(())
}
```

## 📚 Examples

The `examples/` directory contains detailed examples:

**Basic Usage:**
- `basic_read.rs` - Basic Excel file reading
- `basic_write.rs` - Basic Excel file writing
- `streaming_read.rs` - Reading large files with streaming
- `streaming_write.rs` - Writing large files with streaming

**Performance Comparisons:**
- `three_writers_comparison.rs` - **Compare all 3 writer types** (recommended!)
- `write_row_comparison.rs` - String vs typed value writing
- `writer_comparison.rs` - Standard vs fast writer comparison
- `fast_writer_test.rs` - Fast writer performance benchmarks

**Advanced Features:**
- `memory_constrained_write.rs` - Memory-limited writing with compression config
- `auto_memory_config.rs` - Auto memory configuration demo
- `compression_level_config.rs` - Compression level configuration examples
- `csv_to_excel.rs` - CSV to Excel conversion
- `multi_sheet.rs` - Creating multi-sheet workbooks

**PostgreSQL Integration:**
- `postgres_to_excel.rs` - Basic PostgreSQL export
- `postgres_streaming.rs` - Production-tested streaming export (430K rows)
- `postgres_to_excel_advanced.rs` - Advanced async with connection pooling

Running examples:

```bash
# Create sample data first
cargo run --example basic_write

# Read Excel file
cargo run --example basic_read

# Streaming with large files
cargo run --example streaming_write
cargo run --example streaming_read

# Performance comparisons (RECOMMENDED)
cargo run --release --example three_writers_comparison  # Compare all writers
cargo run --release --example write_row_comparison      # String vs typed
cargo run --release --example writer_comparison         # Standard vs fast

# Memory-constrained writing with compression
cargo run --release --example memory_constrained_write  # Test 4 configurations
MEMORY_LIMIT_MB=512 cargo run --release --example auto_memory_config

# Compression level examples
cargo run --release --example compression_level_config  # Test levels 0-9

# Multi-sheet workbooks
cargo run --example multi_sheet

# PostgreSQL examples (requires database setup)
cargo run --example postgres_to_excel --features postgres
cargo run --example postgres_streaming --features postgres  # Production-tested 430K rows
cargo run --example postgres_to_excel_advanced --features postgres-async
```

## 🔧 API Documentation

### ExcelReader

- `open(path)` - Open Excel file for reading
- `sheet_names()` - Get list of sheet names
- `rows(sheet_name)` - Iterator for streaming row reading
- `read_cell(sheet, row, col)` - Read specific cell
- `dimensions(sheet_name)` - Get sheet dimensions (rows, cols)

### ExcelWriter (Streaming)

- `new(path)` - Create new writer with default compression (level 6)
- `with_compression(path, level)` - Create with custom compression level (0-9)
- `write_row(data)` - Write row with strings
- `write_row_typed(cells)` - Write row with typed values (recommended)
- `write_header(headers)` - Write header row
- `write_header_bold(headers)` - Write bold header row
- `write_row_styled(cells)` - Write row with individual cell styles
- `write_row_with_style(cells, style)` - Write row with uniform style
- `add_sheet(name)` - Add new sheet
- `set_flush_interval(rows)` - Configure flush frequency (default: 1000)
- `set_max_buffer_size(bytes)` - Configure buffer size (default: 1MB)
- `set_compression_level(level)` - Set compression level (0-9, default: 6)
- `compression_level()` - Get current compression level
- `set_column_width(col, width)` - Set column width (before writing rows)
- `set_next_row_height(height)` - Set height for next row
- `save()` - Save and finalize workbook

### FastWorkbook (Direct Access)

- `new(path)` - Create fast writer with default compression (level 6)
- `with_compression(path, level)` - Create with custom compression level (0-9)
- `add_worksheet(name)` - Add worksheet
- `write_row(data)` - Write row (optimized)
- `set_flush_interval(rows)` - Set flush frequency
- `set_max_buffer_size(bytes)` - Set buffer limit
- `set_compression_level(level)` - Set compression level (0-9)
- `compression_level()` - Get current compression level
- `close()` - Finish and save file

### Types

- `CellValue` - Enum: Empty, String, Int, Float, Bool, DateTime, Error, Formula
- `Row` - Row with index and cells vector
- `Cell` - Cell with position (row, col) and value

## 🎯 Use Cases

### Processing Large Excel Files (100MB+)

```rust
// Streaming ensures only small portions are loaded into memory
let mut reader = ExcelReader::open("huge_file.xlsx")?;
let mut total = 0.0;

for row_result in reader.rows("Sheet1")? {
    let row = row_result?;
    if let Some(val) = row.get(2).and_then(|c| c.as_f64()) {
        total += val;
    }
}
```

### Exporting Database to Excel

```rust
let mut writer = ExcelWriter::new("export.xlsx")?;
writer.write_header(&["ID", "Name", "Created"])?;

// Fetch from database and write directly
for record in database.query("SELECT * FROM users")? {
    writer.write_row(&[
        &record.id.to_string(),
        &record.name,
        &record.created_at.to_string(),
    ])?;
}

writer.save()?;
```

### Converting CSV to Excel

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

let csv = BufReader::new(File::open("data.csv")?);
let mut writer = ExcelWriter::new("output.xlsx")?;

for (i, line) in csv.lines().enumerate() {
    let fields: Vec<&str> = line?.split(',').collect();
    if i == 0 {
        writer.write_header(fields)?;
    } else {
        writer.write_row(fields)?;
    }
}

writer.save()?;
```

## ⚡ Performance

Benchmarked with **1 million rows × 30 columns** (mixed data types):

| Writer Method | Throughput | Memory Usage | Features |
|--------------|------------|--------------|----------|
| **UltraLowMemoryWorkbook** (direct) | **69,500 rows/sec** ⚡ | **~80MB constant** ✅ | **FASTEST** - Low-level API, max control |
| **ExcelWriter.write_row_typed()** | **62,700 rows/sec** | **~80MB constant** ✅ | Type-safe, best balance (+2% vs baseline) |
| **ExcelWriter.write_row()** | **61,400 rows/sec** | **~80MB constant** ✅ | Simple API, string-based (baseline) |
| **ExcelWriter.write_row_styled()** | **43,500 rows/sec** | **~80MB constant** ✅ | Cell formatting - 29% slower due to styling overhead |

**Key Characteristics:**
- ✅ **High throughput** - 43K-70K rows/sec depending on method
- ✅ **Constant memory** - stays at ~80MB regardless of dataset size
- ✅ **True streaming** - data written directly to disk via ZIP
- ✅ **Predictable performance** - no memory spikes or slowdowns
- ⚡ **UltraLowMemoryWorkbook is FASTEST** - Direct low-level access (+13% vs baseline)
- ⚠️ **Styling has overhead** - write_row_styled() is 29% slower but adds formatting

### Recommendations

| Use Case | Recommended Method | Why |
|----------|-------------------|-----|
| **General use** | `write_row_typed()` | **Best balance** - Type-safe, fast (62.7K rows/sec, +2%) |
| **Simple exports** | `write_row()` | Easy API, good performance (61.4K rows/sec) |
| **Formatted reports** | `write_row_styled()` | Cell formatting - slower but worth it (43.5K rows/sec, -29%) |
| **Maximum speed** | `UltraLowMemoryWorkbook` | **FASTEST** - Direct low-level access (69.5K rows/sec, +13%) |

## 📖 Documentation

- **README.md** (this file) - Complete guide with examples
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [CHANGELOG.md](CHANGELOG.md) - Version history
- Examples in `/examples` directory
- Doc tests in source code

## 🛠️ Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
```

### Run examples

```bash
cargo run --example basic_write
cargo run --example streaming_read
```

### Benchmark

```bash
cargo bench
```

## 📋 Requirements

- Rust 1.70 or higher
- Dependencies:
  - `calamine` - Reading Excel files
  - `zip` - ZIP compression for writing
  - `thiserror` - Error handling

## 🚀 Production Ready

- ✅ **Battle-tested** - Handles 1M+ row datasets with ease
- ✅ **High performance** - 43K-70K rows/sec depending on method (UltraLowMemoryWorkbook fastest!)
- ✅ **Memory efficient** - Constant ~80MB usage, perfect for K8s pods
- ✅ **Reliable** - 50+ comprehensive tests covering edge cases
- ✅ **Safe** - Zero unsafe code, full Rust memory safety
- ✅ **Compatible** - Excel, LibreOffice, Google Sheets
- ✅ **Unicode support** - Special characters, emojis, CJK
- ✅ **CI/CD** - Automated testing on every commit

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Areas for Contribution:**
- Cell formatting and styling (Phase 3)
- Conditional formatting
- Charts and images support
- More examples and documentation
- Performance optimizations

All contributions must:
- Pass `cargo fmt --check`
- Pass `cargo clippy -- -D warnings`
- Pass all tests `cargo test --all-features`
- Include tests for new features

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Credits

This library uses:
- [calamine](https://github.com/tafia/calamine) - Excel reader
- Custom FastWorkbook - High-performance streaming writer

## 📧 Contact

For questions or suggestions, please create an issue on GitHub.

---

Made with ❤️ and 🦀 by the Rust community
