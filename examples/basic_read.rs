//! Basic example of reading an Excel file

use excelstream::ExcelReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open Excel file (created by basic_write example)
    let mut reader = ExcelReader::open("examples/output.xlsx")?;

    // List all sheets
    println!("Available sheets:");
    for (i, name) in reader.sheet_names().iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    // Read first sheet
    println!("\nReading first sheet:");
    for row_result in reader.rows_by_index(0)? {
        let row = row_result?;
        println!("Row {}: {:?}", row.index + 1, row.to_strings());
    }

    Ok(())
}
