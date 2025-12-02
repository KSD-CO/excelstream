//! Streaming write example - Write large Excel files efficiently

use excelstream::ExcelWriter;
use excelstream::types::CellValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Streaming write example - Creating large Excel file");
    println!("This example demonstrates memory-efficient writing\n");

    // Create writer
    let mut writer = ExcelWriter::new("examples/large_output.xlsx")?;

    // Write header
    writer.write_header(&[
        "ID",
        "Name",
        "Value",
        "Timestamp",
        "Status",
    ])?;

    println!("Generating and writing data...");

    // Generate and write rows one at a time
    let num_rows = 10_000;
    for i in 0..num_rows {
        let row = vec![
            CellValue::Int(i as i64),
            CellValue::String(format!("Item_{}", i)),
            CellValue::Float(i as f64 * 1.5),
            CellValue::String(format!("2024-12-{:02} 10:00:00", (i % 28) + 1)),
            CellValue::String(if i % 3 == 0 { "Active" } else { "Inactive" }.to_string()),
        ];

        writer.write_row_typed(&row)?;

        // Print progress
        if (i + 1) % 1000 == 0 {
            println!("  Written {} rows...", i + 1);
        }
    }

    // Save file
    println!("\nSaving file...");
    writer.save()?;

    println!("Successfully created file with {} rows!", num_rows);
    println!("File: examples/large_output.xlsx");

    Ok(())
}
