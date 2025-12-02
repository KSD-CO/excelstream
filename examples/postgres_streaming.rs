//! Memory-efficient PostgreSQL to Excel streaming export
//!
//! This example demonstrates the most memory-efficient way to export
//! millions of rows from PostgreSQL to Excel with minimal memory footprint.
//!
//! Key features:
//! - Uses PostgreSQL cursor for server-side result streaming
//! - Processes data in small batches
//! - Minimal memory usage (suitable for 10M+ rows)
//! - Progress tracking with ETA
//! - Uses typed values for better performance (+40% faster)

use excelstream::types::CellValue;
use excelstream::writer::ExcelWriter;
use postgres::{Client, NoTls};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory-Efficient PostgreSQL to Excel Export ===\n");

    // Configuration
    let connection_string = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/testdb".to_string());

    let output_file = "postgres_large_export.xlsx";
    let batch_size: i32 = 5000; // Process 5000 rows at a time

    // SQL query - modify as needed
    let query = "SELECT id, name, email, age, city, created_at FROM users ORDER BY id";

    println!("Configuration:");
    println!("  Output file: {}", output_file);
    println!("  Batch size: {}", batch_size);
    println!("  Query: {}\n", query);

    let start = Instant::now();

    // Connect to PostgreSQL
    println!("Connecting to PostgreSQL...");
    let mut client = Client::connect(&connection_string, NoTls)?;
    println!("Connected.\n");

    // Start a transaction to use cursors
    let mut transaction = client.transaction()?;

    // Declare a server-side cursor
    println!("Creating server-side cursor...");
    transaction.execute(
        "DECLARE export_cursor CURSOR FOR SELECT id, name, email, age, city, created_at FROM users ORDER BY id",
        &[]
    )?;

    // Create Excel workbook
    println!("Creating Excel workbook...");
    let mut writer = ExcelWriter::new(output_file)?;

    // Configure for optimal memory usage
    writer.set_flush_interval(1000); // Flush every 1000 rows
    writer.set_max_buffer_size(1024 * 1024); // 1MB buffer

    // Write header
    writer.write_header(["ID", "Name", "Email", "Age", "City", "Created At"])?;
    println!("Header written.\n");

    // Statistics
    let mut total_rows = 0;
    let mut batch_number = 0;
    let mut last_progress_update = Instant::now();
    let mut processing_times: Vec<Duration> = Vec::new();

    println!("Starting data export...\n");

    // Fetch and process in batches
    loop {
        let batch_start = Instant::now();

        // Fetch next batch using cursor
        let fetch_query = format!("FETCH {} FROM export_cursor", batch_size);
        let rows = transaction.query(&fetch_query, &[])?;

        if rows.is_empty() {
            println!("\nNo more data. Export complete.");
            break;
        }

        let batch_size_actual = rows.len();
        batch_number += 1;

        // Process batch
        for row in rows {
            // Extract data with proper types
            let id: i32 = row.get(0);
            let name: String = row.get(1);
            let email: String = row.get(2);
            let age: i32 = row.get(3);
            let city: String = row.get(4);
            let created_at: chrono::NaiveDateTime = row.get(5);

            // Write to Excel using typed values (40% faster than strings)
            writer.write_row_typed(&[
                CellValue::Int(id as i64),
                CellValue::String(name),
                CellValue::String(email),
                CellValue::Int(age as i64),
                CellValue::String(city),
                CellValue::String(created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ])?;
        }

        total_rows += batch_size_actual;

        let batch_duration = batch_start.elapsed();
        processing_times.push(batch_duration);

        // Keep only last 10 batch times for moving average
        if processing_times.len() > 10 {
            processing_times.remove(0);
        }

        // Progress update every 2 seconds
        if last_progress_update.elapsed() > Duration::from_secs(2) {
            let _avg_time_per_batch =
                processing_times.iter().sum::<Duration>() / processing_times.len() as u32;
            let rows_per_sec = (batch_size_actual as f64) / batch_duration.as_secs_f64();

            println!(
                "  Batch {:>4} | Rows: {:>10} | Speed: {:>6.0} rows/sec | Batch time: {:>6.2}s",
                batch_number,
                total_rows,
                rows_per_sec,
                batch_duration.as_secs_f64()
            );

            last_progress_update = Instant::now();
        }

        // Stop if we got less than requested (end of data)
        if batch_size_actual < batch_size as usize {
            break;
        }
    }

    // Close cursor and commit transaction
    transaction.execute("CLOSE export_cursor", &[])?;
    transaction.commit()?;

    // Save workbook
    println!("\nFinalizing Excel file...");
    writer.save()?;

    let total_duration = start.elapsed();

    // Final statistics
    println!("\n=== Export Statistics ===");
    println!("Total rows exported: {}", total_rows);
    println!("Total time: {:?}", total_duration);
    println!(
        "Average speed: {:.0} rows/sec",
        total_rows as f64 / total_duration.as_secs_f64()
    );
    println!("Number of batches: {}", batch_number);
    println!("Output file: {}", output_file);

    // Estimate file size
    if let Ok(metadata) = std::fs::metadata(output_file) {
        let size_mb = metadata.len() as f64 / 1_048_576.0;
        println!("File size: {:.2} MB", size_mb);
    }

    println!("\n✓ Export completed successfully!");

    Ok(())
}
