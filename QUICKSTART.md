# Quick Start Guide

## Quick Installation

```bash
# Clone or create new project
cargo new my_excel_project
cd my_excel_project

# Add to Cargo.toml
[dependencies]
excelstream = "0.1"
```

## Test the Library

```bash
cargo test
```

Output:
```
running 35 tests
test result: ok. 35 passed
```

## Run Examples

### 1. Write Basic Excel File

```bash
cargo run --example basic_write
```

Creates `examples/output.xlsx` with sample data.

### 2. Read Excel File

```bash
cargo run --example basic_read
```

Output:
```
Available sheets:
  1. Sheet1

Reading first sheet:
Row 1: ["ID", "Name", "Email", "Age", "Salary"]
Row 2: ["1", "Alice Johnson", "alice@example.com", "30", "75000"]
...
```

### 3. Streaming with Large Files

```bash
# Create file with 10,000 rows
cargo run --example streaming_write

# Read and process with streaming
cargo run --example streaming_read
```

### 4. Performance Comparison ⭐ **RECOMMENDED**

```bash
# Compare all 3 writer types (1M rows)
cargo run --release --example three_writers_comparison

# Results:
# - write_row(): 32,177 rows/s (baseline)
# - write_row_typed(): 32,649 rows/s (+1% faster, Excel formulas work)
# - FastWorkbook: 40,329 rows/s (+25% faster)
```

### 5. Multiple Sheets

```bash
cargo run --example multi_sheet
```

Creates file with 3 sheets: Sales, Employees, Products.

### 6. Convert CSV to Excel

```bash
cargo run --example csv_to_excel
```

Converts `examples/data.csv` to Excel.

## API Usage

### Reading Excel

```rust
use excelstream::reader::ExcelReader;

let mut reader = ExcelReader::open("data.xlsx")?;

// List sheets
for name in reader.sheet_names() {
    println!("Sheet: {}", name);
}

// Read with streaming
for row_result in reader.rows("Sheet1")? {
    let row = row_result?;
    println!("{:?}", row.to_strings());
}
```

### Writing Excel (String-based)

```rust
use excelstream::writer::ExcelWriter;

let mut writer = ExcelWriter::new("output.xlsx")?;

// Write header
writer.write_header(&["ID", "Name", "Email"])?;

// Write rows (all values as strings)
writer.write_row(&["1", "Alice", "alice@example.com"])?;
writer.write_row(&["2", "Bob", "bob@example.com"])?;

writer.save()?;
```

### Writing Excel (Typed Values) ⭐ **RECOMMENDED**

```rust
use excelstream::writer::ExcelWriter;
use excelstream::types::CellValue;

let mut writer = ExcelWriter::new("output.xlsx")?;

writer.write_header(&["Name", "Age", "Salary", "Active"])?;

// Write rows with proper types (Excel formulas work!)
writer.write_row_typed(&[
    CellValue::String("Alice".to_string()),
    CellValue::Int(30),
    CellValue::Float(75000.50),
    CellValue::Bool(true),
])?;

writer.save()?;
```

**Benefits of typed values:**
- ✅ Numbers are numbers (not text)
- ✅ Excel formulas work (SUM, AVERAGE, etc.)
- ✅ 1-5% faster than string conversion
- ✅ Better type safety

### High-Performance Writing (Large Datasets)

For 100K+ rows, use `FastWorkbook` (25-44% faster):

```rust
use excelstream::fast_writer::FastWorkbook;

let mut workbook = FastWorkbook::new("large.xlsx")?;
workbook.add_worksheet("Data")?;

workbook.write_row(&["ID", "Name", "Value"])?;

for i in 1..=1_000_000 {
    workbook.write_row(&[
        &i.to_string(),
        &format!("User{}", i),
        &(i * 100).to_string(),
    ])?;
}

workbook.close()?;
```

**Performance:**
- 40,329 rows/sec (1M rows in 24.8 seconds)
- 25% faster than standard writer
- Lower memory usage

## Performance Comparison

### Which Writer Should You Use?

Tested with **1 million rows × 30 columns**:

| Writer Type | Throughput | Use Case |
|------------|------------|----------|
| `write_row()` | 32,177 rows/s | Simple string data |
| `write_row_typed()` | 32,649 rows/s | **Most use cases (recommended)** |
| `FastWorkbook` | 40,329 rows/s | Large datasets (100K+ rows) |

**Recommendations:**
1. **For most applications**: Use `write_row_typed()` 
   - Excel formulas work correctly
   - Better type safety
   - 1-5% faster than string-based
   
2. **For large datasets (100K+ rows)**: Use `FastWorkbook`
   - 25-44% faster
   - Lower memory usage
   - Best for batch processing

3. **For simple cases**: Use `write_row()`
   - Simplest API
   - Good enough for small datasets

### Test Performance Yourself

```bash
# Run comprehensive comparison
cargo run --release --example three_writers_comparison

# Compare string vs typed writing
cargo run --release --example write_row_comparison
```

## Memory-Constrained Environments

For Kubernetes pods with limited memory (< 512MB):

```rust
use excelstream::fast_writer::create_workbook_auto;

// Auto-detect from MEMORY_LIMIT_MB env variable
let mut workbook = create_workbook_auto("output.xlsx")?;
workbook.add_worksheet("Sheet1")?;

// Write large dataset without OOMKilled
for i in 1..=1_000_000 {
    workbook.write_row(&[
        &i.to_string(),
        &format!("User{}", i),
    ])?;
}

workbook.close()?;
```

**Kubernetes deployment:**
```yaml
env:
- name: MEMORY_LIMIT_MB
  value: "512"
```

## PostgreSQL Integration

Export database to Excel with connection pooling:

```bash
# Setup test database
./examples/setup_postgres_test.sh

# Run example
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=rustfire
export DB_PASSWORD=password
export DB_NAME=rustfire

cargo run --example postgres_to_excel_advanced --features postgres-async
```

See `examples/POSTGRES_EXAMPLES.md` for detailed guide.

## Benchmark

```bash
cargo bench
```

Performance characteristics:
- **Write**: 32K-40K rows/s depending on writer type
- **Read**: Streaming with minimal memory usage
- **Memory**: <250 MB for 1M rows

## Documentation

- **README.md** - Overview and features
- **QUICKSTART.md** - This guide
- **examples/README.md** - All examples explained
- **docs/FAST_WRITER.md** - High-performance writing guide
- **docs/MEMORY_CONSTRAINED.md** - Kubernetes deployment guide
- **docs/OPTIMIZATION_SUMMARY.md** - Performance analysis
- **CONTRIBUTING.md** - Contribution guidelines

## Project Structure

```
excelstream/
├── src/
│   ├── lib.rs              # Entry point
│   ├── error.rs            # Error handling
│   ├── types.rs            # Data types (CellValue, Row, Cell)
│   ├── reader.rs           # Excel reader (streaming)
│   ├── writer.rs           # Standard Excel writer
│   └── fast_writer/        # Fast writer module
│       ├── mod.rs          # Public API
│       ├── workbook.rs     # Workbook implementation
│       ├── worksheet.rs    # Worksheet implementation
│       ├── shared_strings.rs # String optimization
│       ├── xml_writer.rs   # XML generation
│       └── memory.rs       # Memory profiles
├── examples/               # 15+ usage examples
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── docs/                   # Detailed documentation
```

## Features

✅ **Streaming I/O** - Efficient processing of large files  
✅ **Multi-format** - XLSX, XLS, ODS  
✅ **Type-safe** - Strong typing with Rust  
✅ **Multi-sheet** - Multiple sheets in workbook  
✅ **Formatting** - Bold headers, column width  
✅ **Error handling** - Comprehensive error types  
✅ **Zero-copy** - Optimized memory allocation  
✅ **High performance** - 40K rows/sec with FastWorkbook
✅ **Memory efficient** - Configurable for limited resources
✅ **PostgreSQL** - Database export support
✅ **Production ready** - Tested with 1M+ rows
✅ **Documentation** - Complete docs and examples  

## Main Dependencies

- **calamine** 0.32 - Excel reader (multi-format support)
- **rust_xlsxwriter** 0.92 - Excel writer (standard)
- **zip** 2.2 - Fast writer ZIP handling
- **thiserror** 2.0 - Error handling
- **postgres** 0.19 - PostgreSQL sync client (optional)
- **tokio-postgres** 0.7 - PostgreSQL async client (optional)

## Next Steps

1. Review examples in `examples/` directory
2. Run `cargo run --release --example three_writers_comparison` to see performance
3. Read API docs: `cargo doc --open`
4. Check advanced guides in `docs/` directory
4. Tham khảo architecture: `docs/ARCHITECTURE.md`

## Support

- 📧 Email: your.email@example.com
- 🐛 Issues: GitHub Issues
- 📖 Docs: [Documentation](https://docs.rs/rust-excelize)

## License

MIT License - Free to use and modify
