# excelstream Examples

This directory contains examples demonstrating various features of the excelstream library.

## Quick Start Examples

### 1. basic_read.rs
Read a basic Excel file and display its contents.

```bash
cargo run --example basic_read
```

### 2. basic_write.rs
Write a basic Excel file with header and data rows.

```bash
cargo run --example basic_write
```

### 3. streaming_read.rs
Read large Excel files with streaming for memory optimization.

```bash
# Create large file first
cargo run --example streaming_write
# Then read it
cargo run --example streaming_read
```

### 4. streaming_write.rs
Write large Excel files (10,000 rows) with streaming.

```bash
cargo run --example streaming_write
```

## Performance Comparison Examples

### 5. three_writers_comparison.rs ⭐ **RECOMMENDED**
Comprehensive comparison of all 3 writer types with 1 million rows × 30 columns:
- `ExcelWriter.write_row()` - String-based writing (baseline)
- `ExcelWriter.write_row_typed()` - Typed value writing (1-5% faster, Excel formulas work)
- `FastWorkbook` - Custom fast writer (25-44% faster for large datasets)

```bash
# Run full comparison (1M rows, takes ~90 seconds)
cargo run --release --example three_writers_comparison

# Results show:
# - write_row(): 31.08s (32,177 rows/s)
# - write_row_typed(): 30.63s (32,649 rows/s) +1% faster
# - FastWorkbook: 24.80s (40,329 rows/s) +25% faster
```

**This example demonstrates:**
- Real-world mixed data types (strings, integers, floats, booleans)
- Performance at scale (1M rows)
- Memory efficiency
- Feature comparison matrix

### 6. write_row_comparison.rs
Demonstrates the difference between string-based and typed value writing.
Creates 3 Excel files to show:
- String-based: All values stored as text (formulas don't work)
- Typed: Numbers stored as numbers (formulas work correctly)
- Financial report example with proper types

```bash
cargo run --example write_row_comparison

# Creates 3 files:
# - examples/string_output.xlsx
# - examples/typed_output.xlsx
# - examples/financial_report.xlsx
```

### 7. writer_comparison.rs
Compare standard ExcelWriter vs FastWorkbook performance with different dataset sizes.

```bash
cargo run --release --example writer_comparison
```

### 8. fast_writer_test.rs
Fast writer performance benchmarks with 1 million rows.

```bash
cargo run --release --example fast_writer_test
```

## Advanced Features

### 9. csv_to_excel.rs
Convert CSV files to Excel format.

```bash
cargo run --example csv_to_excel
```

### 10. multi_sheet.rs
Create Excel workbooks with multiple sheets.

```bash
cargo run --example multi_sheet
```

### 11. memory_constrained_write.rs
Test memory-constrained writing with different flush intervals.
Ideal for Kubernetes pods with limited memory.

```bash
cargo run --release --example memory_constrained_write
```

### 12. auto_memory_config.rs
Demonstrates automatic memory configuration based on environment variables.

```bash
# Auto-detect memory limit
MEMORY_LIMIT_MB=512 cargo run --release --example auto_memory_config

# Default behavior (no limit)
cargo run --release --example auto_memory_config
```

## PostgreSQL Integration Examples

### 13. postgres_to_excel.rs
Export data from PostgreSQL to Excel (basic synchronous version).

```bash
# Setup database first
./setup_postgres_test.sh
# or
psql -U postgres -f setup_test_db.sql

# Run example
export DATABASE_URL="postgresql://rustfire:password@localhost/rustfire"
cargo run --example postgres_to_excel --features postgres
```

### 14. postgres_streaming.rs
Memory-efficient streaming export from PostgreSQL for very large datasets.

```bash
export DATABASE_URL="postgresql://rustfire:password@localhost/rustfire"
cargo run --example postgres_streaming --features postgres
```

### 15. postgres_to_excel_advanced.rs
Advanced async PostgreSQL export with connection pooling and multiple sheets.

```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=rustfire
export DB_PASSWORD=password
export DB_NAME=rustfire

cargo run --example postgres_to_excel_advanced --features postgres-async
```

See [POSTGRES_EXAMPLES.md](POSTGRES_EXAMPLES.md) for detailed PostgreSQL examples documentation.

## Performance Testing Examples

All performance examples should be run in release mode for accurate results:

```bash
# Full writer comparison (recommended)
cargo run --release --example three_writers_comparison

# Specific comparisons
cargo run --release --example write_row_comparison
cargo run --release --example writer_comparison
cargo run --release --example fast_writer_test

# Memory testing
cargo run --release --example memory_constrained_write
```

## Output Files

Examples will create output files in this directory:
- `output.xlsx` - From basic_write
- `large_output.xlsx` - Large file from streaming_write
- `converted.xlsx` - Converted file from CSV
- `multi_sheet.xlsx` - Multi-sheet file
- `string_output.xlsx` - String-based writing example
- `typed_output.xlsx` - Typed value writing example
- `financial_report.xlsx` - Financial report with proper types
- `comparison_*.xlsx` - Files from performance comparisons
- `memory_test_*.xlsx` - Files from memory testing
- `postgres_export.xlsx` - PostgreSQL export results

## Sample Data

- `data.csv` - Sample CSV file for conversion testing
- `sample.xlsx` - Sample Excel file (created by basic_write)

## Recommended Learning Path

1. Start with **basic_write.rs** and **basic_read.rs** to understand the basics
2. Try **streaming_write.rs** and **streaming_read.rs** for larger datasets
3. Run **three_writers_comparison.rs** to see performance differences
4. Explore **write_row_comparison.rs** to understand typed vs string writing
5. Test **memory_constrained_write.rs** if deploying to Kubernetes
6. Check PostgreSQL examples if integrating with databases
