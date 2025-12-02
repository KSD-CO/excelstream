//! Advanced example: CSV to Excel converter with streaming

use excelstream::ExcelWriter;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Converting CSV to Excel with streaming...\n");

    // Open CSV file
    let csv_file = File::open("examples/data.csv")?;
    let reader = BufReader::new(csv_file);

    // Create Excel writer
    let mut writer = ExcelWriter::new("examples/converted.xlsx")?;

    let mut line_count = 0;
    for line_result in reader.lines() {
        let line = line_result?;
        let fields: Vec<&str> = line.split(',').collect();

        // Write header with formatting
        if line_count == 0 {
            writer.write_header(fields)?;
        } else {
            writer.write_row(fields)?;
        }

        line_count += 1;

        if line_count % 100 == 0 {
            println!("Processed {} lines...", line_count);
        }
    }

    writer.save()?;

    println!("\nConversion complete!");
    println!("Total lines: {}", line_count);
    println!("Output: examples/converted.xlsx");

    Ok(())
}
