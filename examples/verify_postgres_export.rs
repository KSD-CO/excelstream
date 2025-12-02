//! Quick verification of PostgreSQL export files

use excelstream::reader::ExcelReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Verifying PostgreSQL Export Files ===\n");
    
    // Verify postgres_export.xlsx
    println!("1. Checking postgres_export.xlsx...");
    let mut reader = ExcelReader::open("postgres_export.xlsx")?;
    let (rows, cols) = reader.dimensions("PostgreSQL Data")?;
    println!("   Dimensions: {} rows x {} cols", rows, cols);
    
    let mut row_iter = reader.rows("PostgreSQL Data")?;
    if let Some(header) = row_iter.next() {
        println!("   Header: {:?}", header);
    }
    if let Some(first_row) = row_iter.next() {
        println!("   First data row: {:?}", first_row);
    }
    println!("   ✓ Valid!\n");
    
    // Verify postgres_large_export.xlsx
    println!("2. Checking postgres_large_export.xlsx...");
    let mut reader = ExcelReader::open("postgres_large_export.xlsx")?;
    let (rows, cols) = reader.dimensions("Data")?;
    println!("   Dimensions: {} rows x {} cols", rows, cols);
    println!("   ✓ Valid!\n");
    
    // Verify users_export.xlsx
    println!("3. Checking users_export.xlsx...");
    let mut reader = ExcelReader::open("users_export.xlsx")?;
    let (rows, cols) = reader.dimensions("Users")?;
    println!("   Dimensions: {} rows x {} cols", rows, cols);
    println!("   ✓ Valid!\n");
    
    // Verify multi_table_export.xlsx
    println!("4. Checking multi_table_export.xlsx...");
    let mut reader = ExcelReader::open("multi_table_export.xlsx")?;
    let sheets = reader.sheet_names();
    println!("   Sheets: {:?}", sheets);
    for sheet in &sheets {
        let (rows, cols) = reader.dimensions(sheet)?;
        println!("   - {}: {} rows x {} cols", sheet, rows, cols);
    }
    println!("   ✓ Valid!\n");
    
    println!("=== All PostgreSQL exports verified successfully! ===");
    
    Ok(())
}
