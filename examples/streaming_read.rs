//! Streaming read example - Process large Excel files with minimal memory usage

use excelstream::ExcelReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Streaming read example - Processing large Excel file");
    println!("This example demonstrates memory-efficient reading\n");

    // Open Excel file
    let mut reader = ExcelReader::open("examples/large_output.xlsx")?;

    // Get dimensions
    let (rows, cols) = reader.dimensions("Sheet1")?;
    println!("File dimensions: {} rows x {} columns\n", rows, cols);

    // Stream through rows one at a time
    let mut row_count = 0;
    let mut total_sum = 0.0;

    println!("Processing rows...");
    for row_result in reader.rows("Sheet1")? {
        let row = row_result?;
        row_count += 1;

        // Process each row (e.g., sum numeric values)
        for cell in &row.cells {
            if let Some(value) = cell.as_f64() {
                total_sum += value;
            }
        }

        // Print progress every 1000 rows
        if row_count % 1000 == 0 {
            println!("  Processed {} rows...", row_count);
        }
    }

    println!("\nProcessing complete:");
    println!("  Total rows processed: {}", row_count);
    println!("  Sum of all numeric values: {}", total_sum);

    Ok(())
}
