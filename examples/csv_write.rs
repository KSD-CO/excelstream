//! CSV Writer Examples
//!
//! Demonstrates various CSV writing capabilities:
//! - Plain CSV writing
//! - Zstd compressed CSV
//! - Deflate/Gzip compressed CSV
//! - Writing with typed values
//! - Handling edge cases (quotes, commas, newlines)

use excelstream::types::CellValue;
use excelstream::{CompressionMethod, CsvWriter};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== CSV Writer Examples ===\n");

    // Example 1: Plain CSV
    println!("1. Writing plain CSV...");
    {
        let mut writer = CsvWriter::new("examples/output.csv")?;
        writer.write_row(["Name", "Age", "City"])?;
        writer.write_row(["Alice", "30", "New York"])?;
        writer.write_row(["Bob", "25", "San Francisco"])?;
        writer.write_row(["Charlie", "35", "Los Angeles"])?;
        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/output.csv");
    }

    // Example 2: Zstd compressed CSV (large file)
    println!("\n2. Writing Zstd compressed CSV (100K rows)...");
    {
        let mut writer = CsvWriter::with_compression(
            "examples/large.csv.zst",
            CompressionMethod::Zstd,
            3, // Level 3 for balanced compression
        )?;

        writer.write_row(["ID", "Product", "Price", "Quantity"])?;

        for i in 0..100_000 {
            writer.write_row([
                &i.to_string(),
                &format!("Product_{}", i % 1000),
                &format!("{:.2}", (i as f64 * 0.99) % 100.0),
                &((i % 50) + 1).to_string(),
            ])?;
        }

        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/large.csv.zst");

        // Show file size
        let metadata = std::fs::metadata("examples/large.csv.zst")?;
        println!(
            "   File size: {:.2} MB",
            metadata.len() as f64 / 1024.0 / 1024.0
        );
    }

    // Example 3: Deflate/Gzip compressed CSV
    println!("\n3. Writing Deflate/Gzip compressed CSV...");
    {
        let mut writer = CsvWriter::new("examples/data.csv.gz")?; // Auto-detects Deflate
        writer.write_row(["Product", "Category", "Stock"])?;
        writer.write_row(["Laptop", "Electronics", "150"])?;
        writer.write_row(["Chair", "Furniture", "75"])?;
        writer.write_row(["Desk", "Furniture", "50"])?;
        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/data.csv.gz");
    }

    // Example 4: Writing with typed values
    println!("\n4. Writing with typed values...");
    {
        let mut writer = CsvWriter::new("examples/typed.csv")?;
        writer.write_row(["Name", "Score", "Pass", "Grade"])?;

        writer.write_row_typed(&[
            CellValue::String("Alice".to_string()),
            CellValue::Float(95.5),
            CellValue::Bool(true),
            CellValue::String("A".to_string()),
        ])?;

        writer.write_row_typed(&[
            CellValue::String("Bob".to_string()),
            CellValue::Float(78.3),
            CellValue::Bool(true),
            CellValue::String("B".to_string()),
        ])?;

        writer.write_row_typed(&[
            CellValue::String("Charlie".to_string()),
            CellValue::Int(65),
            CellValue::Bool(true),
            CellValue::String("C".to_string()),
        ])?;

        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/typed.csv");
    }

    // Example 5: Edge cases (quotes, commas, newlines)
    println!("\n5. Writing edge cases...");
    {
        let mut writer = CsvWriter::new("examples/edge_cases.csv")?;
        writer.write_row(["Field Type", "Value", "Description"])?;

        // Comma in field
        writer.write_row(["Comma", "a,b,c", "Contains commas"])?;

        // Quotes in field
        writer.write_row(["Quotes", r#"Say "Hello""#, "Contains quotes"])?;

        // Newline in field
        writer.write_row(["Multiline", "Line 1\nLine 2\nLine 3", "Contains newlines"])?;

        // Mixed special characters
        writer.write_row([
            "Complex",
            r#"Name: "John, Jr.", Age: 30"#,
            "Quotes and commas",
        ])?;

        // Empty fields
        writer.write_row(["Empty", "", "Empty middle field"])?;

        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/edge_cases.csv");
    }

    // Example 6: Custom delimiter (semicolon)
    println!("\n6. Writing with custom delimiter (semicolon)...");
    {
        let mut writer = CsvWriter::new("examples/semicolon.csv")?.delimiter(b';');
        writer.write_row(["Country", "Capital", "Population"])?;
        writer.write_row(["France", "Paris", "67M"])?;
        writer.write_row(["Germany", "Berlin", "83M"])?;
        writer.write_row(["Italy", "Rome", "60M"])?;
        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/semicolon.csv");
    }

    // Example 7: Batch writing
    println!("\n7. Writing multiple rows at once...");
    {
        let mut writer = CsvWriter::new("examples/batch.csv")?;
        writer.write_row(["Month", "Revenue"])?;

        let rows = vec![
            vec!["January", "125000"],
            vec!["February", "130000"],
            vec!["March", "145000"],
            vec!["April", "138000"],
            vec!["May", "152000"],
        ];

        writer.write_rows_batch(rows)?;
        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
        println!("   ✓ Created examples/batch.csv");
    }

    println!("\n=== All examples completed successfully! ===");
    println!("\nGenerated files:");
    println!("  - examples/output.csv (plain CSV)");
    println!("  - examples/large.csv.zst (Zstd compressed, 100K rows)");
    println!("  - examples/data.csv.gz (Gzip compressed)");
    println!("  - examples/typed.csv (typed values)");
    println!("  - examples/edge_cases.csv (special characters)");
    println!("  - examples/semicolon.csv (custom delimiter)");
    println!("  - examples/batch.csv (batch writing)");

    Ok(())
}
