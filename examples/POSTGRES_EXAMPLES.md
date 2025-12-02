# PostgreSQL to Excel Examples

This folder contains examples demonstrating how to export data from PostgreSQL to Excel files using ExcelStream.

## Examples

### 1. `postgres_to_excel.rs` - Basic Export
Simple synchronous example that queries PostgreSQL and writes to Excel.

**Features:**
- Direct connection to PostgreSQL
- Streaming data export
- Progress reporting
- Minimal dependencies

**Run:**
```bash
# Set database connection
export DATABASE_URL="postgresql://username:password@localhost/dbname"

# Run the example
cargo run --example postgres_to_excel --features postgres
```

### 2. `postgres_to_excel_advanced.rs` - Advanced Export
Advanced async example with connection pooling and multiple sheets.

**Features:**
- Connection pooling with deadpool-postgres
- Async/await for better performance
- Multiple table export to different sheets
- Custom queries and filtering
- Advanced type handling

**Run:**
```bash
# Set database configuration
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=postgres
export DB_PASSWORD=password
export DB_NAME=testdb

# Run the example
cargo run --example postgres_to_excel_advanced --features postgres-async
```

## Database Setup

### Create Test Database

```sql
-- Create database
CREATE DATABASE testdb;

-- Connect to the database
\c testdb

-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100),
    age INTEGER,
    city VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index for better query performance
CREATE INDEX idx_users_age ON users(age);
CREATE INDEX idx_users_city ON users(city);
CREATE INDEX idx_users_created_at ON users(created_at);
```

### Insert Sample Data

#### Option 1: Small dataset (1,000 rows)
```sql
INSERT INTO users (name, email, age, city)
SELECT 
    'User' || i,
    'user' || i || '@example.com',
    20 + (i % 50),
    'City' || (i % 100)
FROM generate_series(1, 1000) AS i;
```

#### Option 2: Medium dataset (100,000 rows)
```sql
INSERT INTO users (name, email, age, city)
SELECT 
    'User' || i,
    'user' || i || '@example.com',
    20 + (i % 50),
    'City' || (i % 100)
FROM generate_series(1, 100000) AS i;
```

#### Option 3: Large dataset (1,000,000 rows)
```sql
INSERT INTO users (name, email, age, city)
SELECT 
    'User' || i,
    'user' || i || '@example.com',
    20 + (i % 50),
    'City' || (i % 100)
FROM generate_series(1, 1000000) AS i;
```

### Verify Data

```sql
-- Check row count
SELECT COUNT(*) FROM users;

-- Sample data
SELECT * FROM users LIMIT 10;

-- Statistics
SELECT 
    COUNT(*) as total_users,
    MIN(age) as min_age,
    MAX(age) as max_age,
    COUNT(DISTINCT city) as unique_cities
FROM users;
```

## Environment Variables

### Basic Example
- `DATABASE_URL`: Full PostgreSQL connection string
  - Default: `postgresql://postgres:password@localhost/testdb`

### Advanced Example
- `DB_HOST`: Database host (default: `localhost`)
- `DB_PORT`: Database port (default: `5432`)
- `DB_USER`: Database user (default: `postgres`)
- `DB_PASSWORD`: Database password (default: `password`)
- `DB_NAME`: Database name (default: `testdb`)

## Performance Tips

### 1. Use Proper Indexing
```sql
-- Index frequently queried columns
CREATE INDEX idx_users_id ON users(id);
CREATE INDEX idx_users_created_at ON users(created_at DESC);
```

### 2. Optimize Query
```sql
-- Use ORDER BY with indexed columns
SELECT * FROM users ORDER BY id;  -- Fast with index

-- Limit columns to only what you need
SELECT id, name, email FROM users;  -- Faster than SELECT *
```

### 3. Connection Pooling
- Use the advanced example with connection pooling for better performance
- Adjust pool size based on your workload

### 4. Batch Processing
- The basic example uses portal/cursor for streaming
- The advanced example fetches all data but supports very large result sets

## Expected Performance

Based on typical hardware (SSD, 16GB RAM, modern CPU):

| Rows      | Export Time | Speed        | File Size |
|-----------|-------------|--------------|-----------|
| 1,000     | < 1 second  | 1,000+ r/s   | ~100 KB   |
| 10,000    | 1-2 seconds | 5,000+ r/s   | ~1 MB     |
| 100,000   | 10-20 sec   | 5,000+ r/s   | ~10 MB    |
| 1,000,000 | 2-3 minutes | 5,000-8,000 r/s | ~100 MB |

## Troubleshooting

### Connection Refused
```
Error: Connection refused (os error 111)
```
**Solution:** Make sure PostgreSQL is running and accessible.

### Authentication Failed
```
Error: password authentication failed
```
**Solution:** Check your username and password in the connection string.

### Table Not Found
```
Error: relation "users" does not exist
```
**Solution:** Run the database setup SQL commands first.

### Out of Memory
For very large datasets (millions of rows), the advanced example may use significant memory. Consider:
1. Exporting in smaller batches
2. Using the basic example with streaming
3. Increasing system memory or swap space

## Additional Resources

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Rust PostgreSQL Client](https://docs.rs/postgres/)
- [Tokio PostgreSQL](https://docs.rs/tokio-postgres/)
- [Deadpool Connection Pool](https://docs.rs/deadpool-postgres/)

## License

MIT
