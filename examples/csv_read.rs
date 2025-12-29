//! CSV Reader Examples
//!
//! Demonstrates various CSV reading capabilities:
//! - Reading plain CSV files
//! - Reading compressed CSV (Zstd, Gzip)
//! - Reading with headers
//! - Custom delimiters
//! - Handling large files with streaming

use excelstream::CsvReader;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== CSV Reader Examples ===\n");

    // Example 1: Read plain CSV
    println!("1. Reading plain CSV...");
    {
        let mut reader = CsvReader::open("examples/output.csv")?;

        println!("   First 5 rows:");
        for (i, row_result) in reader.rows().enumerate() {
            let row = row_result?;
            if i < 5 {
                println!("   Row {}: {:?}", i + 1, row);
            }
        }
        println!("   Total rows read: {}", reader.row_count());
    }

    // Example 2: Read with headers
    println!("\n2. Reading with headers...");
    {
        let mut reader = CsvReader::open("examples/typed.csv")?.has_header(true);

        // Read first row to get headers
        let first_row_result = reader.rows().next();
        if first_row_result.is_some() {
            // Headers are now available
            if let Some(headers) = reader.headers() {
                println!("   Headers: {:?}", headers);
            }
        }

        // Continue reading
        println!("   Data rows:");
        for row_result in reader.rows() {
            let row = row_result?;
            println!("   {:?}", row);
        }
    }

    // Example 3: Read Zstd compressed CSV
    println!("\n3. Reading Zstd compressed CSV (100K rows)...");
    {
        let mut reader = CsvReader::open("examples/large.csv.zst")?;

        let mut count = 0;
        for row_result in reader.rows() {
            let _row = row_result?;
            count += 1;

            // Show progress every 20K rows
            if count % 20_000 == 0 {
                println!("   Read {} rows...", count);
            }
        }
        println!("   Total rows: {}", count);
    }

    // Example 4: Read Gzip compressed CSV
    println!("\n4. Reading Deflate/Gzip compressed CSV...");
    {
        let mut reader = CsvReader::open("examples/data.csv.gz")?;

        println!("   Contents:");
        for row_result in reader.rows() {
            let row = row_result?;
            println!("   {:?}", row);
        }
        println!("   Total rows: {}", reader.row_count());
    }

    // Example 5: Read edge cases
    println!("\n5. Reading edge cases (quotes, commas, newlines)...");
    {
        let mut reader = CsvReader::open("examples/edge_cases.csv")?;

        println!("   Special characters handled:");
        for (i, row_result) in reader.rows().enumerate() {
            let row = row_result?;
            if i == 0 {
                println!("   Header: {:?}", row);
            } else if row.len() >= 2 {
                println!("   Row {}: Field={}, Value={}", i, row[0], row[1]);
            } else {
                println!("   Row {}: {:?}", i, row);
            }
        }
    }

    // Example 6: Read custom delimiter (semicolon)
    println!("\n6. Reading with custom delimiter (semicolon)...");
    {
        let mut reader = CsvReader::open("examples/semicolon.csv")?.delimiter(b';');

        println!("   Countries:");
        for row_result in reader.rows() {
            let row = row_result?;
            println!("   {:?}", row);
        }
    }

    // Example 7: Read specific columns
    println!("\n7. Reading specific columns...");
    {
        let mut reader = CsvReader::open("examples/output.csv")?.has_header(true);

        // Skip header in iteration
        if let Some(headers) = reader.headers() {
            println!("   Available columns: {:?}", headers);
        }

        println!("   Name and City only:");
        for row_result in reader.rows() {
            let row = row_result?;
            if row.len() >= 3 {
                println!("   {} lives in {}", row[0], row[2]);
            }
        }
    }

    // Example 8: Manual row reading
    println!("\n8. Manual row reading (read_row)...");
    {
        let mut reader = CsvReader::open("examples/batch.csv")?;

        println!("   Reading row by row:");
        while let Some(row) = reader.read_row()? {
            println!("   {:?}", row);
        }
        println!("   Total rows read: {}", reader.row_count());
    }

    // Example 9: Error handling
    println!("\n9. Error handling example...");
    {
        match CsvReader::open("examples/nonexistent.csv") {
            Ok(_) => println!("   File opened"),
            Err(e) => println!("   Expected error: {}", e),
        }
    }

    // Example 10: Count rows efficiently
    println!("\n10. Counting rows in large file...");
    {
        let mut reader = CsvReader::open("examples/large.csv.zst")?;

        let count = reader.rows().filter_map(|r| r.ok()).count();
        println!("   Total rows in large.csv.zst: {}", count);
    }

    println!("\n=== All examples completed successfully! ===");

    Ok(())
}
