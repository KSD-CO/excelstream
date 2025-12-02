//! Example: Query data from PostgreSQL and write to Excel file
//! 
//! This example demonstrates how to:
//! - Connect to PostgreSQL database
//! - Execute a query to fetch large datasets
//! - Stream data directly to Excel file using FastWorkbook
//! - Handle large datasets efficiently with minimal memory usage
//!
//! Prerequisites:
//! - PostgreSQL server running
//! - Database and table created (see setup instructions below)
//!
//! Database setup:
//! ```sql
//! CREATE DATABASE testdb;
//! CREATE TABLE users (
//!     id SERIAL PRIMARY KEY,
//!     name VARCHAR(100),
//!     email VARCHAR(100),
//!     age INTEGER,
//!     city VARCHAR(100),
//!     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
//! );
//! 
//! -- Insert sample data
//! INSERT INTO users (name, email, age, city)
//! SELECT 
//!     'User' || i,
//!     'user' || i || '@example.com',
//!     20 + (i % 50),
//!     'City' || (i % 100)
//! FROM generate_series(1, 100000) AS i;
//! ```

use postgres::{Client, NoTls};
use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PostgreSQL to Excel Export ===\n");

    // Configuration
    let connection_string = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/testdb".to_string());
    
    let output_file = "postgres_export.xlsx";
    let query = "SELECT id, name, email, age, city, created_at FROM users ORDER BY id";

    println!("Connecting to PostgreSQL...");
    let start = Instant::now();
    
    // Connect to PostgreSQL
    let mut client = Client::connect(&connection_string, NoTls)?;
    
    println!("Connected in {:?}\n", start.elapsed());
    println!("Executing query: {}\n", query);
    
    let query_start = Instant::now();
    
    // Execute query and get cursor for streaming
    let mut transaction = client.transaction()?;
    let portal = transaction.bind(query, &[])?;
    
    // Create Excel workbook
    println!("Creating Excel workbook...");
    let mut workbook = FastWorkbook::new(output_file)?;
    workbook.add_worksheet("PostgreSQL Data")?;
    
    // Write header
    workbook.write_row(&["ID", "Name", "Email", "Age", "City", "Created At"])?;
    
    println!("Streaming data to Excel file...\n");
    
    let mut row_count = 0;
    let batch_size = 1000;
    
    // Fetch and write data in batches
    loop {
        let rows = transaction.query_portal(&portal, batch_size)?;
        
        if rows.is_empty() {
            break;
        }
        
        for row in rows {
            // Extract data from PostgreSQL row
            let id: i32 = row.get(0);
            let name: String = row.get(1);
            let email: String = row.get(2);
            let age: i32 = row.get(3);
            let city: String = row.get(4);
            let created_at: chrono::NaiveDateTime = row.get(5);
            
            // Write to Excel
            workbook.write_row(&[
                &id.to_string(),
                &name,
                &email,
                &age.to_string(),
                &city,
                &created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ])?;
            
            row_count += 1;
        }
        
        // Show progress
        if row_count % 10000 == 0 {
            println!("  Processed {} rows...", row_count);
        }
    }
    
    // Commit transaction and close workbook
    transaction.commit()?;
    workbook.close()?;
    
    let total_duration = query_start.elapsed();
    
    println!("\n=== Export Complete ===");
    println!("Total rows exported: {}", row_count);
    println!("Total time: {:?}", total_duration);
    println!("Speed: {:.0} rows/sec", row_count as f64 / total_duration.as_secs_f64());
    println!("Output file: {}", output_file);
    
    Ok(())
}
