# excelstream

🦀 **High-performance Rust library for Excel import/export with streaming support**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- 🚀 **Streaming Read** - Read large Excel files without loading entire content into memory
- 💾 **Streaming Write** - Write Excel files row by row with optimized memory usage
- ⚡ **Fast Writer** - Custom optimized writer **25-44% faster** than standard writer for large datasets
- 🎯 **Typed Values** - Write with proper data types, Excel formulas work correctly (1-5% faster)
- 🎯 **Memory Constrained** - Configurable memory limits for Kubernetes pods with limited resources
- 📊 **Multi-format Support** - XLSX, XLS, ODS
- 🔒 **Type-safe** - Leverage Rust's type system
- ⚡ **Zero-copy** - Minimize memory allocations
- 📝 **Multi-sheet** - Support multiple sheets in one workbook
- 🎨 **Formatting** - Basic cell formatting support
- 🗄️ **PostgreSQL** - Database export examples included

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
excelstream = "0.1.0"
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

### Writing Excel Files (Streaming)

```rust
use excelstream::writer::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("output.xlsx")?;
    
    // Write header with formatting
    writer.write_header(&["ID", "Name", "Email"])?;
    
    // Write data rows (string-based, simple)
    writer.write_row(&["1", "Alice", "alice@example.com"])?;
    writer.write_row(&["2", "Bob", "bob@example.com"])?;
    
    // Set column widths
    writer.set_column_width(0, 5.0)?;
    writer.set_column_width(1, 20.0)?;
    writer.set_column_width(2, 25.0)?;
    
    // Save file
    writer.save()?;
    
    Ok(())
}
```

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
- ✅ Excel formulas work correctly (SUM, AVERAGE, etc.)
- ✅ Better type safety
- ✅ 1-5% faster than string-based writing

### High-Performance Writing (Fast Writer)

For maximum performance with large datasets (100K+ rows), use `FastWorkbook`:

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

**Performance**: Fast Writer achieves **40K rows/sec** (1M rows in 24.8 seconds), **25% faster** than standard writer for large datasets.

See [Fast Writer Documentation](docs/FAST_WRITER.md) for details.

### Memory-Constrained Writing (For Kubernetes Pods)

For pods with limited memory (< 512MB), use auto memory configuration:

```rust
use excelstream::fast_writer::create_workbook_auto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-detect memory limit from env MEMORY_LIMIT_MB
    let mut workbook = create_workbook_auto("output.xlsx")?;
    workbook.add_worksheet("Sheet1")?;
    
    // Write large dataset without OOMKilled
    workbook.write_row(&["ID", "Name", "Email"])?;
    for i in 1..=1_000_000 {
        workbook.write_row(&[
            &i.to_string(),
            &format!("User{}", i),
            &format!("user{}@example.com", i),
        ])?;
    }
    
    workbook.close()?;
    Ok(())
}
```

**Manual configuration for specific memory limits:**

```rust
use excelstream::fast_writer::FastWorkbook;

let mut workbook = FastWorkbook::new("output.xlsx")?;

// For pods < 512MB RAM - optimal configuration
workbook.set_flush_interval(1000);       // Flush every 1000 rows (best balance)
workbook.set_max_buffer_size(256 * 1024); // 256KB buffer

workbook.add_worksheet("Sheet1")?;
// ... write data ...
workbook.close()?;
```

**Kubernetes deployment:**

```yaml
env:
- name: MEMORY_LIMIT_MB
  value: "512"
```

See [Memory-Constrained Guide](docs/MEMORY_CONSTRAINED.md) for details.

### Writing with Typed Values

```rust
use excelstream::writer::ExcelWriter;
use excelstream::types::CellValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("typed_output.xlsx")?;
    
    writer.write_header(&["Name", "Age", "Salary", "Active"])?;
    
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

**Why use typed values?**
- Numbers stored as numbers (not text)
- Excel formulas work correctly
- Better Excel compatibility
- 1-5% faster than string conversion

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

### ExcelWriter

- `new(path)` - Create new writer
- `write_row(data)` - Write row with strings
- `write_row_typed(cells)` - Write row with typed values
- `write_header(headers)` - Write header with formatting
- `add_sheet(name)` - Add new sheet
- `set_column_width(col, width)` - Set column width
- `save()` - Save workbook to file

### FastWorkbook (High Performance)

- `new(path)` - Create fast writer
- `add_worksheet(name)` - Add worksheet
- `write_row(data)` - Write row (optimized)
- `set_flush_interval(rows)` - Set flush frequency
- `set_max_buffer_size(bytes)` - Set buffer limit
- `close()` - Finish and save file

### Memory Helpers

- `create_workbook_auto(path)` - Auto-detect memory config from env
- `create_workbook_with_profile(path, profile)` - Use specific memory profile
- `MemoryProfile::Low` - For pods < 512MB
- `MemoryProfile::Medium` - For pods 512MB-1GB
- `MemoryProfile::High` - For pods > 1GB

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

The library is designed for high performance with three writer options:

### Writer Performance Comparison

Tested with **1 million rows × 30 columns** (mixed data types):

| Writer Type | Time | Throughput | Use Case |
|------------|------|------------|----------|
| **ExcelWriter.write_row()** | 31.08s | 32,177 rows/s | Simple string data |
| **ExcelWriter.write_row_typed()** | 30.63s | 32,649 rows/s | **Recommended for most cases** |
| **FastWorkbook** | 24.80s | 40,329 rows/s | Large datasets (100K+ rows) |

**Performance at 100K rows:**
- `write_row_typed()`: +5% faster than `write_row()`
- `FastWorkbook`: +44% faster than ExcelWriter methods

**Key Insights:**
- 🏆 **FastWorkbook** is 25-44% faster for large datasets
- ✅ **write_row_typed()** is recommended for most use cases (better Excel compatibility + 1-5% faster)
- 📊 **write_row()** is simplest for basic string data

### Memory Usage

**Standard vs Fast Writer (100K rows, 5 columns):**
| Writer | Time | Speed | Memory |
|--------|------|-------|--------|
| Standard | 491ms | 203K rows/s | ~300MB |
| Fast | 434ms | 230K rows/s | ~250MB |
| **Improvement** | **-11.6%** | **+13.1%** | **-16.7%** |

**Fast Writer with different flush intervals (1M rows):**
| Configuration | Time | Speed | Memory Peak |
|--------------|------|-------|-------------|
| Default (1000 flush) | 9.9s | 101K rows/s | ~250MB |
| Balanced (500 flush) | 10.9s | 91K rows/s | ~150MB |
| Low memory (100 flush) | 10.5s | 95K rows/s | ~80MB |

**Recommendation:** Use 1000-row flush interval for best balance of speed and memory.

### Features by Writer Type

| Feature | write_row() | write_row_typed() | FastWorkbook |
|---------|-------------|-------------------|--------------|
| Simple API | ✅ | ✅ | ✅ |
| Excel formulas work | ❌ | ✅ | ⚠️ Limited |
| Type safety | ❌ | ✅ | ❌ |
| Speed | Baseline | +1-5% | +25-44% |
| Memory efficient | ✅ | ✅ | ✅✅ |
| Good for large datasets | ✅ | ✅ | ✅✅✅ |

See [Performance Documentation](docs/OPTIMIZATION_SUMMARY.md) for more details.

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
  - `rust_xlsxwriter` - Standard Excel writer
  - `zip` - Custom fast writer ZIP handling
  - `thiserror` - Error handling

## 🚀 Production Ready

- ✅ Tested with 1M+ row datasets
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
- [rust_xlsxwriter](https://github.com/jmcnamara/rust_xlsxwriter) - Excel writer

## 📧 Contact

For questions or suggestions, please create an issue on GitHub.

---

Made with ❤️ and 🦀 by the Rust community
