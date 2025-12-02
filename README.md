# excelstream

🦀 **High-performance Rust library for Excel import/export with streaming support**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KSD-CO/excelstream/workflows/Rust/badge.svg)](https://github.com/KSD-CO/excelstream/actions)

> **✨ What's New in v0.2.2:**
> - ✅ **Formula Support** - Write Excel formulas like `=SUM(A1:A10)`
> - ✅ **Better Error Messages** - Context-rich errors with sheet information
> - ✅ **21-47% Faster** - Optimized performance vs rust_xlsxwriter
> - ✅ **Comprehensive Tests** - 47 tests covering edge cases, unicode, special characters
> - ✅ **CI/CD Pipeline** - Automated testing and publishing

## ✨ Features

- 🚀 **Streaming Read** - Process large Excel files without loading entire file into memory
- 💾 **Streaming Write** - Write millions of rows with constant ~80MB memory usage
- ⚡ **High Performance** - 21-47% faster than rust_xlsxwriter baseline
- 📐 **Formula Support** - Write Excel formulas (=SUM, =AVERAGE, =IF, etc.)
- 🎯 **Typed Values** - Strong typing with Int, Float, Bool, DateTime, Formula
- 🔧 **Memory Efficient** - Configurable flush intervals for memory-limited environments
- ❌ **Better Errors** - Context-rich error messages with available sheets list
- 📊 **Multi-format Support** - Read XLSX, XLS, ODS formats
- 🔒 **Type-safe** - Leverage Rust's type system for safety
- 📝 **Multi-sheet** - Support multiple sheets in one workbook
- 🗄️ **Database Export** - PostgreSQL integration examples
- ✅ **Production Ready** - 47 tests, CI/CD, zero unsafe code

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
excelstream = "0.2"
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
- ✅ 21-47% faster than rust_xlsxwriter baseline
- ✅ Direct ZIP streaming - data written to disk immediately
- ⚠️ Note: Bold formatting and column width not yet implemented in streaming mode

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
- ✅ 40% faster than rust_xlsxwriter

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

**Performance (v0.2.0)**: 
- ExcelWriter.write_row(): **36,870 rows/sec** (+21% vs rust_xlsxwriter)
- ExcelWriter.write_row_typed(): **42,877 rows/sec** (+40% vs rust_xlsxwriter)
- FastWorkbook direct: **44,753 rows/sec** (+47% vs rust_xlsxwriter)

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

Tested with **1 million rows × 30 columns** (mixed data types):

| Writer Type | Speed (rows/s) | vs Baseline | Memory Usage |
|-------------|----------------|-------------|--------------|
| rust_xlsxwriter | 30,525 | baseline | ~300MB (grows) |
| **ExcelWriter.write_row()** | **36,870** | **+21%** ⚡ | **~80MB constant** ✅ |
| **ExcelWriter.write_row_typed()** | **42,877** | **+40%** ⚡ | **~80MB constant** ✅ |
| **FastWorkbook** | **44,753** | **+47%** ⚡ | **~80MB constant** ✅ |

**Key Advantages:**
- ✅ **21-47% faster** than rust_xlsxwriter
- ✅ **Constant memory** - doesn't grow with dataset size
- ✅ **True streaming** - data written directly to disk
- ✅ **No memory spikes** - predictable resource usage

### Recommendations

| Use Case | Recommended Writer | Why |
|----------|-------------------|-----|
| General use | `ExcelWriter.write_row_typed()` | Best balance of speed + features |
| Simple exports | `ExcelWriter.write_row()` | Easy API, good performance |
| Maximum speed | `FastWorkbook` | Fastest, lowest-level API |
| Advanced formatting | rust_xlsxwriter | Full Excel features |

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

- ✅ **Battle-tested** - Handles 1M+ row datasets
- ✅ **High performance** - 21-47% faster than alternatives
- ✅ **Memory efficient** - Constant ~80MB usage, works in K8s pods
- ✅ **Reliable** - 47 comprehensive tests covering edge cases
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
