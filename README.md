# excelstream

🦀 **High-performance Rust library for Excel import/export with streaming support**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KSD-CO/excelstream/workflows/Rust/badge.svg)](https://github.com/KSD-CO/excelstream/actions)

> **✨ What's New in v0.4.1:**
> - 🪟 **Windows Compatibility Fix** - Fixed critical OOXML structure bug for Excel on Windows
> - ✅ **100% OOXML Compliance** - All files now match reference implementation exactly
> - 🔄 **Deterministic Output** - Using IndexMap for consistent shared strings ordering
> - 🚀 **No Performance Impact** - Fixes maintain full performance (~42K rows/sec)

> **v0.4.0 Features:**
> - 📏 **Column Width & Row Height** - Customize column widths and row heights for perfect formatting!
> - 🎨 **Cell Formatting** - 14 predefined styles: bold, italic, highlights, borders, number formats!
> - 🎨 **Easy Styling API** - `write_header_bold()`, `write_row_styled()`, `write_row_with_style()`
> - 💰 **Number Formats** - Currency, percentage, decimal, integer formats
> - 📅 **Date Formats** - MM/DD/YYYY and timestamp formats

## ✨ Features

- 🚀 **Streaming Read** - Process large Excel files without loading entire file into memory
- 💾 **Streaming Write** - Write millions of rows with constant ~80MB memory usage
- ⚡ **High Performance** - 30K-45K rows/sec throughput with true streaming
- 🎨 **Cell Formatting** - 14 predefined styles (bold, currency, %, highlights, borders)
- 📏 **Column Width & Row Height** - Customize column widths and row heights
- 📐 **Formula Support** - Write Excel formulas (=SUM, =AVERAGE, =IF, etc.)
- 🎯 **Typed Values** - Strong typing with Int, Float, Bool, DateTime, Formula
- 🔧 **Memory Efficient** - Configurable flush intervals for memory-limited environments
- ❌ **Better Errors** - Context-rich error messages with available sheets list
- 📊 **Multi-format Support** - Read XLSX, XLS, ODS formats
- 🔒 **Type-safe** - Leverage Rust's type system for safety
- 📝 **Multi-sheet** - Support multiple sheets in one workbook
- 🗄️ **Database Export** - PostgreSQL integration examples
- 🪟 **Cross-Platform** - Works perfectly on Windows, Linux, and macOS
- ✅ **Production Ready** - 50+ tests, CI/CD, zero unsafe code

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
excelstream = "0.4"
```

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
- ✅ High throughput: 30K-45K rows/sec with true streaming
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
- ✅ Excellent performance: 42K+ rows/sec

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

### Column Width and Row Height

**New in v0.4.0:** Customize column widths and row heights for better formatting!

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

**See also:** Run `cargo run --example column_width_row_height` for a complete demonstration!

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

**Performance Metrics**:
- ExcelWriter.write_row(): **36,870 rows/sec**
- ExcelWriter.write_row_typed(): **42,877 rows/sec**
- FastWorkbook direct: **44,753 rows/sec**
- Memory usage: **Constant ~80MB** for any dataset size

### Memory-Constrained Writing (For Kubernetes Pods)

In v0.2.0, all writers use streaming with constant memory:

```rust
use excelstream::writer::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("output.xlsx")?;
    
    // For pods with < 512MB RAM
    writer.set_flush_interval(500);       // Flush more frequently
    writer.set_max_buffer_size(256 * 1024); // 256KB buffer
    
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

**Memory usage in v0.2.0:**
- Constant ~80MB regardless of dataset size
- Configurable flush interval and buffer size
- Suitable for Kubernetes pods with limited resources

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
- `memory_constrained_write.rs` - Memory-limited writing for pods
- `auto_memory_config.rs` - Auto memory configuration demo
- `csv_to_excel.rs` - CSV to Excel conversion
- `multi_sheet.rs` - Creating multi-sheet workbooks

**PostgreSQL Integration:**
- `postgres_to_excel.rs` - Basic PostgreSQL export
- `postgres_streaming.rs` - Streaming PostgreSQL export
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

# Memory-constrained writing
cargo run --release --example memory_constrained_write
MEMORY_LIMIT_MB=512 cargo run --release --example auto_memory_config

# Multi-sheet workbooks
cargo run --example multi_sheet

# PostgreSQL examples (requires database setup)
cargo run --example postgres_to_excel --features postgres
cargo run --example postgres_streaming --features postgres
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

- `new(path)` - Create new writer
- `write_row(data)` - Write row with strings
- `write_row_typed(cells)` - Write row with typed values (recommended)
- `write_header(headers)` - Write header row
- `add_sheet(name)` - Add new sheet
- `set_flush_interval(rows)` - Configure flush frequency (default: 1000)
- `set_max_buffer_size(bytes)` - Configure buffer size (default: 1MB)
- `set_column_width(col, width)` - Not yet implemented in streaming mode
- `save()` - Save and finalize workbook

### FastWorkbook (Direct Access)

- `new(path)` - Create fast writer
- `add_worksheet(name)` - Add worksheet
- `write_row(data)` - Write row (optimized)
- `set_flush_interval(rows)` - Set flush frequency
- `set_max_buffer_size(bytes)` - Set buffer limit
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
| **ExcelWriter.write_row()** | **36,870 rows/sec** | **~80MB constant** ✅ | Simple API, string-based |
| **ExcelWriter.write_row_typed()** | **42,877 rows/sec** | **~80MB constant** ✅ | Type-safe, best balance |
| **ExcelWriter.write_row_styled()** | **~42,000 rows/sec** | **~80MB constant** ✅ | Cell formatting + styles |
| **FastWorkbook** (direct) | **44,753 rows/sec** | **~80MB constant** ✅ | Maximum speed, low-level |

**Key Characteristics:**
- ✅ **High throughput** - 30K-45K rows/sec depending on method
- ✅ **Constant memory** - stays at ~80MB regardless of dataset size
- ✅ **True streaming** - data written directly to disk via ZIP
- ✅ **Predictable performance** - no memory spikes or slowdowns

### Recommendations

| Use Case | Recommended Method | Why |
|----------|-------------------|-----|
| **General use** | `write_row_typed()` | Best balance of speed, type safety, and features |
| **Simple exports** | `write_row()` | Easy API, good performance |
| **Formatted reports** | `write_row_styled()` | Cell formatting with minimal overhead |
| **Maximum speed** | `FastWorkbook` | Direct access, highest throughput |

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
- ✅ **High performance** - 30K-45K rows/sec with true streaming
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
