//! Test that fast writer output can be read back correctly

use excelstream::fast_writer::FastWorkbook;
use excelstream::reader::ExcelReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fast Writer Output Validation ===\n");

    // Write test file
    println!("1. Writing test file with Fast Writer...");
    let filename = "fast_writer_validation.xlsx";
    let mut workbook = FastWorkbook::new(filename)?;
    workbook.add_worksheet("TestSheet")?;
    
    workbook.write_row(&["ID", "Name", "Email"])?;
    workbook.write_row(&["1", "Alice", "alice@example.com"])?;
    workbook.write_row(&["2", "Bob", "bob@example.com"])?;
    workbook.write_row(&["3", "Charlie", "charlie@example.com"])?;
    
    workbook.close()?;
    println!("   ✓ File written successfully\n");

    // Read back and validate
    println!("2. Reading back with ExcelReader...");
    let mut reader = ExcelReader::open(filename)?;
    
    let sheets = reader.sheet_names();
    println!("   Sheets: {:?}", sheets);
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0], "TestSheet");
    
    let mut rows_data = Vec::new();
    for row_result in reader.rows("TestSheet")? {
        let row = row_result?;
        let strings = row.to_strings();
        rows_data.push(strings);
        println!("   Row {}: {:?}", row.index, row.to_strings());
    }
    
    println!("\n3. Validating data integrity...");
    assert_eq!(rows_data.len(), 4, "Should have 4 rows");
    assert_eq!(rows_data[0], vec!["ID", "Name", "Email"], "Header mismatch");
    assert_eq!(rows_data[1], vec!["1", "Alice", "alice@example.com"], "Row 1 mismatch");
    assert_eq!(rows_data[2], vec!["2", "Bob", "bob@example.com"], "Row 2 mismatch");
    assert_eq!(rows_data[3], vec!["3", "Charlie", "charlie@example.com"], "Row 3 mismatch");
    
    println!("   ✓ All data validated correctly\n");
    println!("=== SUCCESS: Fast Writer output is valid and readable! ===");

    Ok(())
}
