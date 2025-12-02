//! Advanced example: PostgreSQL to Excel with connection pooling and optimizations
//!
//! This example demonstrates:
//! - Connection pooling with deadpool-postgres
//! - Async operations for better performance
//! - Progress reporting
//! - Error handling and recovery
//! - Custom query support
//! - Multiple sheet export
//! - Typed values for +40% better performance

use deadpool_postgres::{Config, Pool, Runtime};
use excelstream::types::CellValue;
use excelstream::writer::ExcelWriter;
use std::time::Instant;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced PostgreSQL to Excel Export ===\n");

    // Configuration from environment or defaults
    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "testdb".to_string());

    // Create connection pool
    println!("Setting up connection pool...");
    let mut cfg = Config::new();
    cfg.host = Some(db_host);
    cfg.port = Some(db_port.parse()?);
    cfg.user = Some(db_user);
    cfg.password = Some(db_password);
    cfg.dbname = Some(db_name);

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    println!("Connection pool created\n");

    // Example 1: Export single table
    println!("Example 1: Exporting users table...");
    export_table(
        &pool,
        "SELECT id, name, email, age, city, created_at FROM users ORDER BY id",
        "users_export.xlsx",
        "Users",
    )
    .await?;

    // Example 2: Export with custom query
    println!("\nExample 2: Exporting filtered data...");
    export_table(
        &pool,
        "SELECT id, name, email, age, city FROM users WHERE age >= 30 AND age <= 40 ORDER BY age",
        "users_filtered_export.xlsx",
        "Users 30-40",
    )
    .await?;

    // Example 3: Export multiple related tables to different sheets
    println!("\nExample 3: Exporting multiple tables to one workbook...");
    export_multiple_tables(&pool).await?;

    println!("\n=== All exports completed successfully ===");

    Ok(())
}

/// Export a single query result to Excel file
async fn export_table(
    pool: &Pool,
    query: &str,
    output_file: &str,
    _sheet_name: &str, // Not used in v0.2.0 (ExcelWriter creates default "Sheet1")
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // Get connection from pool
    let client = pool.get().await?;

    // Execute query
    let rows = client.query(query, &[]).await?;
    let row_count = rows.len();

    println!("  Found {} rows", row_count);

    // Create writer
    let mut writer = ExcelWriter::new(output_file)?;

    // Configure for optimal performance
    writer.set_flush_interval(1000);
    writer.set_max_buffer_size(1024 * 1024);

    if rows.is_empty() {
        writer.save()?;
        println!("  No data to export");
        return Ok(());
    }

    // Write header from first row columns
    let first_row = &rows[0];
    let columns: Vec<&str> = first_row.columns().iter().map(|col| col.name()).collect();

    writer.write_header(&columns)?;

    // Write data rows with typed values
    println!("  Writing data...");
    for (idx, row) in rows.iter().enumerate() {
        let mut row_data: Vec<CellValue> = Vec::new();

        for col_idx in 0..row.len() {
            let value = format_cell_value_typed(row, col_idx);
            row_data.push(value);
        }

        writer.write_row_typed(&row_data)?;

        // Progress indicator
        if (idx + 1) % 10000 == 0 {
            println!("    Processed {}/{} rows...", idx + 1, row_count);
        }
    }

    writer.save()?;

    let duration = start.elapsed();
    println!("  ✓ Exported {} rows in {:?}", row_count, duration);
    println!(
        "  ✓ Speed: {:.0} rows/sec",
        row_count as f64 / duration.as_secs_f64()
    );
    println!("  ✓ File: {}", output_file);

    Ok(())
}

/// Export multiple tables to different sheets in one workbook
async fn export_multiple_tables(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let output_file = "multi_table_export.xlsx";

    let mut writer = ExcelWriter::new(output_file)?;

    // Configure for optimal performance
    writer.set_flush_interval(1000);
    writer.set_max_buffer_size(1024 * 1024);

    // Define queries for different sheets
    let queries = [
        (
            "Users Summary",
            "SELECT city, COUNT(*) as user_count, AVG(age) as avg_age FROM users GROUP BY city ORDER BY user_count DESC LIMIT 100"
        ),
        (
            "Age Distribution",
            "SELECT age, COUNT(*) as count FROM users GROUP BY age ORDER BY age"
        ),
        (
            "Recent Users",
            "SELECT id, name, email, created_at FROM users ORDER BY created_at DESC LIMIT 1000"
        ),
    ];

    let client = pool.get().await?;

    for (idx, (sheet_name, query)) in queries.iter().enumerate() {
        println!("  Processing sheet: {}", sheet_name);

        // Add new sheet (first sheet already exists)
        if idx > 0 {
            writer.add_sheet(sheet_name)?;
        }

        let rows = client.query(*query, &[]).await?;

        if rows.is_empty() {
            continue;
        }

        // Write header
        let columns: Vec<&str> = rows[0].columns().iter().map(|col| col.name()).collect();
        writer.write_header(&columns)?;

        // Write data with typed values
        for row in &rows {
            let mut row_data: Vec<CellValue> = Vec::new();
            for col_idx in 0..row.len() {
                row_data.push(format_cell_value_typed(row, col_idx));
            }
            writer.write_row_typed(&row_data)?;
        }

        println!("    ✓ {} rows written", rows.len());
    }

    writer.save()?;

    println!("  ✓ Multi-table export completed in {:?}", start.elapsed());
    println!("  ✓ File: {}", output_file);

    Ok(())
}

/// Format a PostgreSQL cell value to CellValue (typed)
fn format_cell_value_typed(row: &tokio_postgres::Row, col_idx: usize) -> CellValue {
    use tokio_postgres::types::Type;

    let column = &row.columns()[col_idx];

    match *column.type_() {
        Type::INT2 => row
            .try_get::<_, i16>(col_idx)
            .map(|v| CellValue::Int(v as i64))
            .unwrap_or(CellValue::Empty),
        Type::INT4 => row
            .try_get::<_, i32>(col_idx)
            .map(|v| CellValue::Int(v as i64))
            .unwrap_or(CellValue::Empty),
        Type::INT8 => row
            .try_get::<_, i64>(col_idx)
            .map(CellValue::Int)
            .unwrap_or(CellValue::Empty),
        Type::FLOAT4 => row
            .try_get::<_, f32>(col_idx)
            .map(|v| CellValue::Float(v as f64))
            .unwrap_or(CellValue::Empty),
        Type::FLOAT8 => row
            .try_get::<_, f64>(col_idx)
            .map(CellValue::Float)
            .unwrap_or(CellValue::Empty),
        Type::VARCHAR | Type::TEXT | Type::BPCHAR => row
            .try_get::<_, String>(col_idx)
            .map(CellValue::String)
            .unwrap_or(CellValue::Empty),
        Type::TIMESTAMP => row
            .try_get::<_, chrono::NaiveDateTime>(col_idx)
            .map(|v| CellValue::String(v.format("%Y-%m-%d %H:%M:%S").to_string()))
            .unwrap_or(CellValue::Empty),
        Type::TIMESTAMPTZ => row
            .try_get::<_, chrono::DateTime<chrono::Utc>>(col_idx)
            .map(|v| CellValue::String(v.format("%Y-%m-%d %H:%M:%S %Z").to_string()))
            .unwrap_or(CellValue::Empty),
        Type::DATE => row
            .try_get::<_, chrono::NaiveDate>(col_idx)
            .map(|v| CellValue::String(v.format("%Y-%m-%d").to_string()))
            .unwrap_or(CellValue::Empty),
        Type::BOOL => row
            .try_get::<_, bool>(col_idx)
            .map(CellValue::Bool)
            .unwrap_or(CellValue::Empty),
        _ => {
            // For other types, try to get as string
            row.try_get::<_, String>(col_idx)
                .map(CellValue::String)
                .unwrap_or(CellValue::Empty)
        }
    }
}
