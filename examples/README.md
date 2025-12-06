# ExcelStream Examples

Clean, organized examples demonstrating the excelstream library's capabilities.

## 📚 Quick Start (Beginners)

### 1. Basic Operations

**basic_write.rs** - Write a basic Excel file with headers and data
```bash
cargo run --example basic_write
```

**basic_read.rs** - Read an Excel file and display contents
```bash
cargo run --example basic_read
```

### 2. Streaming for Large Files

**streaming_write.rs** - Write 10,000+ rows efficiently
```bash
cargo run --example streaming_write
```

**streaming_read.rs** - Read large files with constant memory
```bash
# Create large file first
cargo run --example streaming_write
# Then read it
cargo run --example streaming_read
```

### 3. Features

**cell_formatting.rs** - Apply cell styles (bold, colors, borders, number formats)
```bash
cargo run --example cell_formatting
```

**column_width_row_height.rs** - Customize column widths and row heights
```bash
cargo run --example column_width_row_height
```

**worksheet_protection.rs** - Protect sheets with passwords and permissions
```bash
cargo run --example worksheet_protection
```

**multi_sheet.rs** - Create workbooks with multiple sheets
```bash
cargo run --example multi_sheet
```

**csv_to_excel.rs** - Convert CSV files to Excel
```bash
cargo run --example csv_to_excel
```

## ⚡ Performance Testing

**writers_comparison.rs** ⭐ **RECOMMENDED** - Compare all writer methods
```bash
# 1M rows × 30 columns (takes ~90 seconds)
cargo run --release --example writers_comparison

# Results (v0.7.0):
# write_row():        18,307 rows/s (baseline)
# write_row_typed():  19,722 rows/s (+8%)
# write_row_styled(): 19,474 rows/s (+6%)
# UltraLowMemory:     24,451 rows/s (+34%) ⚡
```

**compression_level_config.rs** - Test different compression levels (0-9)
```bash
cargo run --release --example compression_level_config
```

## 💾 Memory Management

**memory_constrained_write.rs** - Optimize for K8s/containers (<512 MB RAM)
```bash
# Simulate 512 MB memory limit
MEMORY_LIMIT_MB=512 cargo run --release --example memory_constrained_write
```

**auto_memory_config.rs** - Automatic memory configuration
```bash
# With memory limit
MEMORY_LIMIT_MB=256 cargo run --release --example auto_memory_config

# Auto-detect
cargo run --release --example auto_memory_config
```

**large_dataset_multi_sheet.rs** - Handle 10M+ rows (auto-split at 1M rows/sheet)
```bash
# 10M rows (takes 3-5 minutes)
cargo run --release --example large_dataset_multi_sheet

# Results:
# - 10 sheets created (1M rows each)
# - ~62,000 rows/s sustained
# - ~1.7 GB output file
# - Constant memory usage
```

## 🐘 PostgreSQL Integration

**Setup database first:**
```bash
./setup_postgres_test.sh
# or
psql -U postgres -f setup_test_db.sql
```

**postgres_streaming.rs** - Stream from PostgreSQL (production-tested 430K+ rows)
```bash
export DATABASE_URL="postgresql://rustfire:password@localhost/rustfire"
cargo run --example postgres_streaming --features postgres
```

**postgres_to_excel_advanced.rs** - Async with connection pooling
```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=rustfire
export DB_PASSWORD=password
export DB_NAME=rustfire

cargo run --example postgres_to_excel_advanced --features postgres-async
```

**verify_postgres_export.rs** - Quick verification tool
```bash
cargo run --example verify_postgres_export
```

See [POSTGRES_EXAMPLES.md](POSTGRES_EXAMPLES.md) for detailed PostgreSQL documentation.

## 📖 Recommended Learning Path

### For Beginners:
1. `basic_write.rs` + `basic_read.rs` - Understand basics
2. `streaming_write.rs` + `streaming_read.rs` - Large files
3. `cell_formatting.rs` - Styling
4. `multi_sheet.rs` - Multiple sheets

### For Performance:
1. `writers_comparison.rs` - See all methods side-by-side ⭐
2. `large_dataset_multi_sheet.rs` - Push the limits (10M rows)
3. `memory_constrained_write.rs` - Production deployments

### For Production:
1. `postgres_streaming.rs` - Database integration
2. `memory_constrained_write.rs` - Container environments (K8s)
3. `compression_level_config.rs` - Optimize speed vs file size

## 📝 Output Files

Examples create output files in this directory:
- `output.xlsx` - From basic_write
- `large_output.xlsx` - From streaming_write
- `test_*.xlsx` - From performance tests
- `postgres_export.xlsx` - From PostgreSQL exports

## 🔧 Performance Tips

**Always use `--release` mode for performance testing:**
```bash
cargo run --release --example writers_comparison
```

**Key insights (v0.7.0):**
- All methods use constant ~80 MB memory (streaming architecture)
- `UltraLowMemoryWorkbook` is fastest: 24K rows/s (+34%)
- `write_row_typed()` is 8% faster than string-based
- Typical throughput: 18K-25K rows/s with 30 columns
- Compression level 1 = 2x faster (dev), level 6 = smallest (prod)
