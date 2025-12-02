//! Advanced example: PostgreSQL to Excel with connection pooling and optimizations
//! 
//! This example demonstrates:
//! - Connection pooling with deadpool-postgres
//! - Async operations for better performance
//! - Progress reporting
//! - Error handling and recovery
//! - Custom query support
//! - Multiple sheet export

use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

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
        "Users"
    ).await?;

    // Example 2: Export with custom query
    println!("\nExample 2: Exporting filtered data...");
    export_table(
        &pool,
        "SELECT id, name, email, age, city FROM users WHERE age >= 30 AND age <= 40 ORDER BY age",
        "users_filtered_export.xlsx",
        "Users 30-40"
    ).await?;

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
    sheet_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    // Get connection from pool
    let client = pool.get().await?;
    
    // Execute query
    let rows = client.query(query, &[]).await?;
    let row_count = rows.len();
    
    println!("  Found {} rows", row_count);
    
    // Create workbook
    let mut workbook = FastWorkbook::new(output_file)?;
    workbook.add_worksheet(sheet_name)?;
    
    if rows.is_empty() {
        workbook.close()?;
        println!("  No data to export");
        return Ok(());
    }
    
    // Write header from first row columns
    let first_row = &rows[0];
    let columns: Vec<&str> = first_row
        .columns()
        .iter()
        .map(|col| col.name())
        .collect();
    
    workbook.write_row(&columns)?;
    
    // Write data rows
    println!("  Writing data...");
    for (idx, row) in rows.iter().enumerate() {
        let mut row_data: Vec<String> = Vec::new();
        
        for col_idx in 0..row.len() {
            let value = format_cell_value(row, col_idx);
            row_data.push(value);
        }
        
        let row_refs: Vec<&str> = row_data.iter().map(|s| s.as_str()).collect();
        workbook.write_row(&row_refs)?;
        
        // Progress indicator
        if (idx + 1) % 10000 == 0 {
            println!("    Processed {}/{} rows...", idx + 1, row_count);
        }
    }
    
    workbook.close()?;
    
    let duration = start.elapsed();
    println!("  ✓ Exported {} rows in {:?}", row_count, duration);
    println!("  ✓ Speed: {:.0} rows/sec", row_count as f64 / duration.as_secs_f64());
    println!("  ✓ File: {}", output_file);
    
    Ok(())
}

/// Export multiple tables to different sheets in one workbook
async fn export_multiple_tables(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let output_file = "multi_table_export.xlsx";
    
    let mut workbook = FastWorkbook::new(output_file)?;
    
    // Define queries for different sheets
    let queries = vec![
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
    
    for (sheet_name, query) in queries {
        println!("  Processing sheet: {}", sheet_name);
        
        workbook.add_worksheet(sheet_name)?;
        
        let rows = client.query(query, &[]).await?;
        
        if rows.is_empty() {
            continue;
        }
        
        // Write header
        let columns: Vec<&str> = rows[0]
            .columns()
            .iter()
            .map(|col| col.name())
            .collect();
        workbook.write_row(&columns)?;
        
        // Write data
        for row in &rows {
            let mut row_data: Vec<String> = Vec::new();
            for col_idx in 0..row.len() {
                row_data.push(format_cell_value(row, col_idx));
            }
            let row_refs: Vec<&str> = row_data.iter().map(|s| s.as_str()).collect();
            workbook.write_row(&row_refs)?;
        }
        
        println!("    ✓ {} rows written", rows.len());
    }
    
    workbook.close()?;
    
    println!("  ✓ Multi-table export completed in {:?}", start.elapsed());
    println!("  ✓ File: {}", output_file);
    
    Ok(())
}

/// Format a PostgreSQL cell value to string
fn format_cell_value(row: &tokio_postgres::Row, col_idx: usize) -> String {
    use tokio_postgres::types::Type;
    
    let column = &row.columns()[col_idx];
    
    match *column.type_() {
        Type::INT2 => {
            row.try_get::<_, i16>(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_default()
        }
        Type::INT4 => {
            row.try_get::<_, i32>(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_default()
        }
        Type::INT8 => {
            row.try_get::<_, i64>(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_default()
        }
        Type::FLOAT4 => {
            row.try_get::<_, f32>(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_default()
        }
        Type::FLOAT8 => {
            row.try_get::<_, f64>(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_default()
        }
        Type::VARCHAR | Type::TEXT | Type::BPCHAR => {
            row.try_get::<_, String>(col_idx)
                .unwrap_or_default()
        }
        Type::TIMESTAMP => {
            row.try_get::<_, chrono::NaiveDateTime>(col_idx)
                .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        }
        Type::TIMESTAMPTZ => {
            row.try_get::<_, chrono::DateTime<chrono::Utc>>(col_idx)
                .map(|v| v.format("%Y-%m-%d %H:%M:%S %Z").to_string())
                .unwrap_or_default()
        }
        Type::DATE => {
            row.try_get::<_, chrono::NaiveDate>(col_idx)
                .map(|v| v.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        }
        Type::BOOL => {
            row.try_get::<_, bool>(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_default()
        }
        _ => {
            // For other types, try to get as string
            row.try_get::<_, String>(col_idx)
                .unwrap_or_else(|_| "NULL".to_string())
        }
    }
}
