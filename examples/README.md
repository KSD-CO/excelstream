# excelstream Examples# excelstream Examples



Clean, organized examples demonstrating the excelstream library's capabilities.This directory contains examples demonstrating various features of the excelstream library.



## 📚 Quick Start (Beginner)## Quick Start Examples



### 1. basic_write.rs### 1. basic_read.rs

Create a simple Excel file with headers and data rows.Read a basic Excel file and display its contents.

```bash

cargo run --example basic_write```bash

```cargo run --example basic_read

```

### 2. basic_read.rs

Read an Excel file and display its contents.### 2. basic_write.rs

```bashWrite a basic Excel file with header and data rows.

cargo run --example basic_read

``````bash

cargo run --example basic_write

### 3. streaming_write.rs```

Write large Excel files (10,000 rows) efficiently.

```bash### 3. streaming_read.rs

cargo run --example streaming_writeRead large Excel files with streaming for memory optimization.

```

```bash

### 4. streaming_read.rs# Create large file first

Read large Excel files with minimal memory usage.cargo run --example streaming_write

```bash# Then read it

cargo run --example streaming_readcargo run --example streaming_read

``````



## 🚀 Advanced Features### 4. streaming_write.rs

Write large Excel files (10,000 rows) with streaming.

### 5. multi_sheet.rs

Create workbooks with multiple sheets.```bash

```bashcargo run --example streaming_write

cargo run --example multi_sheet```

```

## Performance Comparison Examples

### 6. cell_formatting.rs

Apply various cell styles and formatting options.### 5. three_writers_comparison.rs ⭐ **RECOMMENDED**

```bashComprehensive comparison of all 3 writer types with 1 million rows × 30 columns:

cargo run --example cell_formatting- `ExcelWriter.write_row()` - String-based writing (baseline)

```- `ExcelWriter.write_row_typed()` - Typed value writing (1-5% faster, Excel formulas work)

- `FastWorkbook` - Custom fast writer (25-44% faster for large datasets)

### 7. column_width_row_height.rs

Customize column widths and row heights.```bash

```bash# Run full comparison (1M rows, takes ~90 seconds)

cargo run --example column_width_row_heightcargo run --release --example three_writers_comparison

```

# Results show:

### 8. csv_to_excel.rs# - write_row(): 31.08s (32,177 rows/s)

Convert CSV files to Excel format with streaming.# - write_row_typed(): 30.63s (32,649 rows/s) +1% faster

```bash# - FastWorkbook: 24.80s (40,329 rows/s) +25% faster

cargo run --example csv_to_excel```

```

**This example demonstrates:**

## ⚡ Performance & Comparison- Real-world mixed data types (strings, integers, floats, booleans)

- Performance at scale (1M rows)

### 9. writers_comparison.rs ⭐ **RECOMMENDED**- Memory efficiency

Comprehensive comparison of 4 writer methods with 100K-1M rows:- Feature comparison matrix



**Writer Methods Compared:**### 6. write_row_comparison.rs

1. `write_row()` - String-based (baseline)Demonstrates the difference between string-based and typed value writing.

2. `write_row_typed()` - Typed values (Excel formulas work)Creates 3 Excel files to show:

3. `write_row_styled()` - With conditional formatting (FASTEST! +15%)- String-based: All values stored as text (formulas don't work)

4. `FastWorkbook` - Direct low-level API- Typed: Numbers stored as numbers (formulas work correctly)

- Financial report example with proper types

```bash

# Quick test (100K rows, ~7 seconds)```bash

cargo run --release --example writers_comparisoncargo run --example write_row_comparison



# Full test (1M rows, ~90 seconds) - Uncomment in code# Creates 3 files:

# Results with 1M rows × 30 columns:# - examples/string_output.xlsx

# - write_row():        22.58s (44,293 rows/s)# - examples/typed_output.xlsx

# - write_row_typed():  21.77s (45,931 rows/s) +4% faster# - examples/financial_report.xlsx

# - write_row_styled(): 19.51s (51,266 rows/s) +15% faster ⚡```

# - FastWorkbook:       20.62s (48,497 rows/s) +9% faster

```### 7. writer_comparison.rs

Compare standard ExcelWriter vs FastWorkbook performance with different dataset sizes.

**Key Insights:**

- `write_row_styled()` is fastest because it avoids intermediate string conversions```bash

- All methods use FastWorkbook internallycargo run --release --example writer_comparison

- Memory usage is constant (streaming architecture)```



## 💾 Memory Management### 8. fast_writer_test.rs

Fast writer performance benchmarks with 1 million rows.

### 10. memory_constrained_write.rs

Optimize for memory-limited environments (Kubernetes, containers).```bash

```bashcargo run --release --example fast_writer_test

# Simulate 512MB memory limit```

MEMORY_LIMIT_MB=512 cargo run --release --example memory_constrained_write

```## Advanced Features



### 11. auto_memory_config.rs### 9. csv_to_excel.rs

Automatic memory configuration based on environment.Convert CSV files to Excel format.

```bash

# With memory limit```bash

MEMORY_LIMIT_MB=256 cargo run --release --example auto_memory_configcargo run --example csv_to_excel

```

# Auto-detect

cargo run --release --example auto_memory_config### 10. multi_sheet.rs

```Create Excel workbooks with multiple sheets.



## 📊 Large Datasets```bash

cargo run --example multi_sheet

### 12. large_dataset_multi_sheet.rs```

Handle datasets exceeding Excel's 1,048,576 row/sheet limit.

Automatically splits data across multiple sheets.### 11. memory_constrained_write.rs

Test memory-constrained writing with different flush intervals.

```bashIdeal for Kubernetes pods with limited memory.

# Test with 10 million rows (takes ~3-5 minutes)

cargo run --release --example large_dataset_multi_sheet```bash

cargo run --release --example memory_constrained_write

# Results:```

# - 10 sheets created (1M rows each)

# - ~62,000 rows/sec sustained throughput### 12. auto_memory_config.rs

# - ~1.7 GB output fileDemonstrates automatic memory configuration based on environment variables.

# - Constant memory usage

``````bash

# Auto-detect memory limit

**Features:**MEMORY_LIMIT_MB=512 cargo run --release --example auto_memory_config

- Automatic sheet splitting at 1M rows

- Progress reporting every 100K rows# Default behavior (no limit)

- ZIP64 support for large files (>4GB)cargo run --release --example auto_memory_config

- Excel hard limit safe (1,048,576 rows/sheet)```



## 🐘 PostgreSQL Integration## PostgreSQL Integration Examples



See [POSTGRES_EXAMPLES.md](POSTGRES_EXAMPLES.md) and [POSTGRES_QUICKREF.md](POSTGRES_QUICKREF.md) for detailed PostgreSQL documentation.### 13. postgres_to_excel.rs

Export data from PostgreSQL to Excel (basic synchronous version).

### 13. postgres_streaming.rs

Memory-efficient streaming export from PostgreSQL.```bash

```bash# Setup database first

# Setup database./setup_postgres_test.sh

./setup_postgres_test.sh# or

psql -U postgres -f setup_test_db.sql

# Run export

export DATABASE_URL="postgresql://rustfire:password@localhost/rustfire"# Run example

cargo run --example postgres_streaming --features postgresexport DATABASE_URL="postgresql://rustfire:password@localhost/rustfire"

```cargo run --example postgres_to_excel --features postgres

```

### 14. postgres_to_excel_advanced.rs

Async export with connection pooling and optimizations.### 14. postgres_streaming.rs

```bashMemory-efficient streaming export from PostgreSQL for very large datasets.

export DB_HOST=localhost

export DB_USER=rustfire```bash

export DB_PASSWORD=passwordexport DATABASE_URL="postgresql://rustfire:password@localhost/rustfire"

export DB_NAME=rustfirecargo run --example postgres_streaming --features postgres

```

cargo run --example postgres_to_excel_advanced --features postgres-async

```### 15. postgres_to_excel_advanced.rs

Advanced async PostgreSQL export with connection pooling and multiple sheets.

### 15. verify_postgres_export.rs

Quick verification tool for PostgreSQL exports.```bash

```bashexport DB_HOST=localhost

cargo run --example verify_postgres_exportexport DB_PORT=5432

```export DB_USER=rustfire

export DB_PASSWORD=password

## 📖 Recommended Learning Pathexport DB_NAME=rustfire



**For Beginners:**cargo run --example postgres_to_excel_advanced --features postgres-async

1. `basic_write.rs` → `basic_read.rs` - Understand the basics```

2. `streaming_write.rs` → `streaming_read.rs` - Larger datasets

3. `cell_formatting.rs` - Add stylingSee [POSTGRES_EXAMPLES.md](POSTGRES_EXAMPLES.md) for detailed PostgreSQL examples documentation.

4. `multi_sheet.rs` - Multiple sheets

## Performance Testing Examples

**For Performance:**

1. `writers_comparison.rs` - See all methods side-by-sideAll performance examples should be run in release mode for accurate results:

2. `large_dataset_multi_sheet.rs` - Push the limits

3. `memory_constrained_write.rs` - Production deployments```bash

# Full writer comparison (recommended)

**For Production:**cargo run --release --example three_writers_comparison

1. `postgres_streaming.rs` - Database integration

2. `memory_constrained_write.rs` - Container environments# Specific comparisons

3. `large_dataset_multi_sheet.rs` - Large exportscargo run --release --example write_row_comparison

cargo run --release --example writer_comparison

## 📝 Output Filescargo run --release --example fast_writer_test



Examples create output files in the `examples/` directory:# Memory testing

- `output.xlsx` - Basic write outputcargo run --release --example memory_constrained_write

- `large_output.xlsx` - Streaming write output```

- `comparison_*.xlsx` - Writer comparison outputs

- `test_10m_multi_sheet.xlsx` - Large dataset multi-sheet output## Output Files

- `postgres_export*.xlsx` - PostgreSQL export outputs

Examples will create output files in this directory:

## 🔧 Performance Tips- `output.xlsx` - From basic_write

- `large_output.xlsx` - Large file from streaming_write

**Always use `--release` mode for performance testing:**- `converted.xlsx` - Converted file from CSV

```bash- `multi_sheet.xlsx` - Multi-sheet file

cargo run --release --example writers_comparison- `string_output.xlsx` - String-based writing example

```- `typed_output.xlsx` - Typed value writing example

- `financial_report.xlsx` - Financial report with proper types

**Key Performance Factors:**- `comparison_*.xlsx` - Files from performance comparisons

- `write_row_styled()` is fastest (+15% over baseline)- `memory_test_*.xlsx` - Files from memory testing

- FastWorkbook provides true streaming (constant memory)- `postgres_export.xlsx` - PostgreSQL export results

- ZIP64 automatically enabled for large files

- Typical throughput: 40K-60K rows/sec with 30 columns## Sample Data



## 🗑️ Removed Examples- `data.csv` - Sample CSV file for conversion testing

- `sample.xlsx` - Sample Excel file (created by basic_write)

The following redundant examples were removed in cleanup:

- `write_row_comparison.rs` - Merged into `writers_comparison.rs`## Recommended Learning Path

- `debug_write_row_fast.rs` - Debug file, not needed

- `fast_writer_test.rs` - Covered by `writers_comparison.rs`1. Start with **basic_write.rs** and **basic_read.rs** to understand the basics

- `fast_writer_validation.rs` - Not necessary2. Try **streaming_write.rs** and **streaming_read.rs** for larger datasets

- `memory_usage_test.rs` - Covered by `memory_constrained_write.rs`3. Run **three_writers_comparison.rs** to see performance differences

- `performance_optimized.rs` - Covered by `writers_comparison.rs`4. Explore **write_row_comparison.rs** to understand typed vs string writing

- `performance_test_with_sizing.rs` - Covered by `writers_comparison.rs`5. Test **memory_constrained_write.rs** if deploying to Kubernetes

6. Check PostgreSQL examples if integrating with databases

All functionality from removed examples is preserved in the remaining, better-organized examples.
