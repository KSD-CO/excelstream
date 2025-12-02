# PostgreSQL to Excel - Quick Reference

## Quick Start (5 minutes)

```bash
# 1. Setup database with 100K sample rows
cd examples
./setup_postgres_test.sh

# 2. Set connection
export DATABASE_URL="postgresql://postgres:password@localhost/testdb"

# 3. Run export
cargo run --example postgres_streaming --features postgres
```

## All Commands

### Setup Database
```bash
# Interactive setup (recommended)
./setup_postgres_test.sh

# Manual SQL setup
psql -U postgres -f setup_test_db.sql

# Custom row count
psql -U postgres -c "
  INSERT INTO users (name, email, age, city)
  SELECT 'User' || i, 'user' || i || '@example.com', 20 + (i % 50), 'City' || (i % 100)
  FROM generate_series(1, 1000000) AS i;
" testdb
```

### Run Examples

#### Basic Export
```bash
export DATABASE_URL="postgresql://postgres:password@localhost/testdb"
cargo run --example postgres_to_excel --features postgres
```

#### Streaming Export (Best for large data)
```bash
export DATABASE_URL="postgresql://postgres:password@localhost/testdb"
cargo run --example postgres_streaming --features postgres
```

#### Advanced Export (Multiple sheets)
```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=postgres
export DB_PASSWORD=password
export DB_NAME=testdb
cargo run --example postgres_to_excel_advanced --features postgres-async
```

## Environment Variables

### Basic/Streaming Examples
- `DATABASE_URL` - Full connection string
  - Format: `postgresql://[user[:password]@][host][:port][/dbname]`
  - Example: `postgresql://postgres:pass123@localhost:5432/testdb`

### Advanced Example
- `DB_HOST` - Database host (default: localhost)
- `DB_PORT` - Database port (default: 5432)
- `DB_USER` - Database user (default: postgres)
- `DB_PASSWORD` - Database password
- `DB_NAME` - Database name (default: testdb)

## Useful SQL Commands

```bash
# Check row count
psql -U postgres -d testdb -c "SELECT COUNT(*) FROM users;"

# View sample data
psql -U postgres -d testdb -c "SELECT * FROM users LIMIT 10;"

# Database statistics
psql -U postgres -d testdb -c "
  SELECT 
    COUNT(*) as total_users,
    MIN(age) as min_age,
    MAX(age) as max_age,
    COUNT(DISTINCT city) as unique_cities
  FROM users;
"

# Drop and recreate database
psql -U postgres -c "DROP DATABASE testdb;"
psql -U postgres -c "CREATE DATABASE testdb;"
```

## Performance Guide

| Rows      | Recommended Example    | Expected Time | Memory  |
|-----------|------------------------|---------------|---------|
| < 10K     | postgres_to_excel      | < 5s          | Low     |
| 10K-100K  | postgres_to_excel      | 10-30s        | Medium  |
| 100K-1M   | postgres_streaming     | 1-5 min       | Low     |
| 1M-10M    | postgres_streaming     | 5-30 min      | Low     |
| 10M+      | postgres_streaming     | 30+ min       | Low     |

## Troubleshooting

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Start PostgreSQL
sudo systemctl start postgresql

# Check connection
psql -U postgres -c "SELECT version();"

# Test connection string
psql "postgresql://postgres:password@localhost/testdb" -c "SELECT 1;"

# View PostgreSQL logs
sudo tail -f /var/log/postgresql/postgresql-*.log
```

## Common Errors

```bash
# Connection refused
sudo systemctl start postgresql

# Authentication failed
# Edit: /etc/postgresql/*/main/pg_hba.conf
# Change: peer -> trust or md5

# Table not found
./setup_postgres_test.sh

# Permission denied
sudo -u postgres psql
```

## Output Files

- `postgres_export.xlsx` - Basic export
- `postgres_large_export.xlsx` - Streaming export
- `users_export.xlsx` - Advanced: all users
- `users_filtered_export.xlsx` - Advanced: filtered users
- `multi_table_export.xlsx` - Advanced: multiple sheets

## Documentation

- [POSTGRES_EXAMPLES.md](POSTGRES_EXAMPLES.md)
- [README.md](README.md) - All examples overview
