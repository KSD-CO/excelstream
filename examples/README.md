# rust-excelize Examples

This directory contains examples of using the rust-excelize library.

## Example List

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

### 5. csv_to_excel.rs
Convert CSV files to Excel format.

```bash
cargo run --example csv_to_excel
```

### 6. multi_sheet.rs
Create Excel workbooks with multiple sheets.

```bash
cargo run --example multi_sheet
```

### 7. postgres_to_excel.rs
Export data from PostgreSQL to Excel (basic synchronous version).

```bash
# Setup database first
./setup_postgres_test.sh
# or
psql -U postgres -f setup_test_db.sql

# Run example
export DATABASE_URL="postgresql://postgres:password@localhost/testdb"
cargo run --example postgres_to_excel --features postgres
```

### 8. postgres_streaming.rs
Memory-efficient streaming export from PostgreSQL for very large datasets.

```bash
export DATABASE_URL="postgresql://postgres:password@localhost/testdb"
cargo run --example postgres_streaming --features postgres
```

### 9. postgres_to_excel_advanced.rs
Advanced async PostgreSQL export with connection pooling and multiple sheets.

```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=postgres
export DB_PASSWORD=password
export DB_NAME=testdb

cargo run --example postgres_to_excel_advanced --features postgres-async
```

See [POSTGRES_EXAMPLES.md](POSTGRES_EXAMPLES.md) for detailed PostgreSQL examples documentation.

## Output Files

Examples will create output files in this directory:
- `output.xlsx` - From basic_write
- `large_output.xlsx` - Large file from streaming_write
- `converted.xlsx` - Converted file from CSV
- `multi_sheet.xlsx` - Multi-sheet file

## Sample Data

- `data.csv` - Sample CSV file for conversion testing
- `sample.xlsx` - Sample Excel file (needs to be created with basic_write first)
