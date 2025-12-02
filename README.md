# excelstream

🦀 **High-performance Rust library for Excel import/export with streaming support**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ BREAKING CHANGE in v0.2.0:**  
> Removed `rust_xlsxwriter` dependency! ExcelWriter now uses custom FastWorkbook implementation for **streaming** with constant memory usage and **21-40% faster performance**.  
> See [MIGRATION_v0.2.0.md](MIGRATION_v0.2.0.md) for migration guide.

## ✨ Features

- 🚀 **Streaming Read** - Read large Excel files without loading entire content into memory
- 💾 **Streaming Write** - Write millions of rows with constant ~80MB memory usage (NEW in v0.2.0!)
- ⚡ **21-40% Faster** - ExcelWriter is now faster than rust_xlsxwriter (v0.2.0)
- 🎯 **Typed Values** - Write with proper data types for better Excel compatibility
- 🎯 **Memory Constrained** - Configurable flush intervals for memory-limited environments
- 📊 **Multi-format Support** - XLSX, XLS, ODS for reading
- 🔒 **Type-safe** - Leverage Rust's type system
- ⚡ **Zero-copy** - Minimize memory allocations
- 📝 **Multi-sheet** - Support multiple sheets in one workbook
- 🗄️ **PostgreSQL** - Database export examples included

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

**v0.2.0 Changes:**
- ✅ Streaming with constant ~80MB memory (was ~300MB in v0.1.x)
- ✅ 21-40% faster than rust_xlsxwriter
- ❌ Bold header formatting removed (will be added back in v0.2.1)
- ❌ `set_column_width()` is now a no-op (will be added back in v0.2.1)

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
- ✅ Better type safety
- ✅ 40% faster than rust_xlsxwriter in v0.2.0

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

### ExcelWriter (v0.2.0 - Streaming)

- `new(path)` - Create new writer
- `write_row(data)` - Write row with strings
- `write_row_typed(cells)` - Write row with typed values (recommended)
- `write_header(headers)` - Write header row (no formatting in v0.2.0)
- `add_sheet(name)` - Add new sheet
- `set_flush_interval(rows)` - Configure flush frequency (NEW in v0.2.0)
- `set_max_buffer_size(bytes)` - Configure buffer size (NEW in v0.2.0)
- `set_column_width(col, width)` - No-op in v0.2.0 (will be restored in v0.2.1)
- `save()` - Save and finalize workbook

### FastWorkbook (Direct Access)

- `new(path)` - Create fast writer
- `add_worksheet(name)` - Add worksheet
- `write_row(data)` - Write row (optimized)
- `set_flush_interval(rows)` - Set flush frequency
- `set_max_buffer_size(bytes)` - Set buffer limit
- `close()` - Finish and save file

### Types

- `CellValue` - Enum for cell values: Empty, String, Int, Float, Bool, DateTime, Error
- `Row` - Struct representing a row with index and cells
- `Cell` - Struct for a cell with position (row, col) and value

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

### v0.2.0 Performance (1M rows × 30 columns)

Tested with **1 million rows × 30 columns** (mixed data types):

| Writer Type | Speed (rows/s) | vs rust_xlsxwriter | Memory Usage |
|-------------|----------------|-------------------|--------------|
| rust_xlsxwriter direct | 30,525 | baseline (1.00x) | ~300MB (grows) |
| **ExcelWriter.write_row()** | **36,870** | **+21% faster** | **~80MB constant** ✅ |
| **ExcelWriter.write_row_typed()** | **42,877** | **+40% faster** | **~80MB constant** ✅ |
| **FastWorkbook direct** | **44,753** | **+47% faster** | **~80MB constant** ✅ |

**Key Improvements in v0.2.0:**
- ✅ **21-47% faster** than rust_xlsxwriter
- ✅ **Constant ~80MB memory** (was ~300MB with rust_xlsxwriter)
- ✅ **Streaming** - memory doesn't grow with dataset size
- ✅ **No external dependencies** for writing

### Memory Usage Comparison

**v0.1.x (using rust_xlsxwriter):**
- Memory grows with data: ~300MB for 1M rows
- All data kept in memory until save()

**v0.2.0 (using FastWorkbook):**
- Constant memory: ~80MB regardless of rows
- Data written directly to disk
- Configurable flush intervals

### Recommendations

| Use Case | Recommended Writer | Performance |
|----------|-------------------|-------------|
| General use | `ExcelWriter.write_row_typed()` | 42,877 rows/s, typed values |
| Simple text | `ExcelWriter.write_row()` | 36,870 rows/s, easy to use |
| Maximum speed | `FastWorkbook` direct | 44,753 rows/s, low-level API |
| Need formatting | Use rust_xlsxwriter directly | 30,525 rows/s, full features |

## 📖 Documentation

- [Quick Start Guide](docs/QUICKSTART.md) - Get started in 5 minutes
- [Fast Writer Guide](docs/FAST_WRITER.md) - High-performance writing
- [Memory-Constrained Guide](docs/MEMORY_CONSTRAINED.md) - For Kubernetes pods
- [Optimization Summary](docs/OPTIMIZATION_SUMMARY.md) - Performance details
- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute

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

- ✅ Tested with 1M+ row datasets
- ✅ Streaming with constant memory usage
- ✅ 21-47% faster than rust_xlsxwriter
- ✅ Memory-safe with Rust's ownership
- ✅ Works in Kubernetes pods with limited resources
- ✅ Comprehensive error handling
- ✅ Zero unsafe code
- ✅ Validated Excel output (readable by Excel/LibreOffice)

## 🤝 Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Credits

This library uses:
- [calamine](https://github.com/tafia/calamine) - Excel reader
- Custom FastWorkbook implementation - High-performance writer (v0.2.0+)

**Previous versions (v0.1.x)** used [rust_xlsxwriter](https://github.com/jmcnamara/rust_xlsxwriter) - removed in v0.2.0 for better performance and streaming.

## 📧 Contact

For questions or suggestions, please create an issue on GitHub.

---

Made with ❤️ and 🦀 by the Rust community
