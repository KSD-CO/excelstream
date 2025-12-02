//! Demonstration of write_row() vs write_row_typed()
//! 
//! This example shows the difference between:
//! - write_row(): Takes strings only
//! - write_row_typed(): Takes typed values (Int, Float, Bool, String, etc.)

use excelstream::writer::ExcelWriter;
use excelstream::types::CellValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== write_row() vs write_row_typed() ===\n");
    
    // Example 1: write_row() - Everything is a string
    println!("1. write_row() - All values as strings:");
    {
        let mut writer = ExcelWriter::new("test_write_row.xlsx")?;
        writer.write_header(&["Name", "Age", "Salary", "Active"])?;
        
        // Must convert everything to string
        writer.write_row(&[
            "Alice",              // String
            "30",                 // Number as string
            "75000.50",           // Float as string
            "true",               // Boolean as string
        ])?;
        
        writer.write_row(&[
            "Bob",
            "25",
            "60000.00",
            "false",
        ])?;
        
        writer.save()?;
    }
    println!("   ✓ File created: test_write_row.xlsx");
    println!("   - All cells stored as TEXT in Excel");
    println!("   - Cannot use Excel formulas on numbers (e.g., SUM, AVERAGE)");
    println!("   - Boolean shows as text 'true'/'false'\n");
    
    // Example 2: write_row_typed() - Preserve data types
    println!("2. write_row_typed() - Typed values:");
    {
        let mut writer = ExcelWriter::new("test_write_row_typed.xlsx")?;
        writer.write_header(&["Name", "Age", "Salary", "Active"])?;
        
        // Use proper types
        writer.write_row_typed(&[
            CellValue::String("Alice".to_string()),  // String type
            CellValue::Int(30),                       // Integer type
            CellValue::Float(75000.50),               // Float type
            CellValue::Bool(true),                    // Boolean type
        ])?;
        
        writer.write_row_typed(&[
            CellValue::String("Bob".to_string()),
            CellValue::Int(25),
            CellValue::Float(60000.00),
            CellValue::Bool(false),
        ])?;
        
        writer.save()?;
    }
    println!("   ✓ File created: test_write_row_typed.xlsx");
    println!("   - Numbers stored as NUMBERS in Excel");
    println!("   - Can use Excel formulas (SUM, AVERAGE, etc.)");
    println!("   - Boolean shows as TRUE/FALSE checkbox\n");
    
    // Example 3: Real-world use case - Financial data
    println!("3. Real-world example - Financial report:");
    {
        let mut writer = ExcelWriter::new("financial_report.xlsx")?;
        writer.write_header(&["Date", "Product", "Quantity", "Unit Price", "Total", "Paid"])?;
        
        writer.write_row_typed(&[
            CellValue::String("2024-12-01".to_string()),
            CellValue::String("Laptop".to_string()),
            CellValue::Int(5),
            CellValue::Float(1299.99),
            CellValue::Float(6499.95),
            CellValue::Bool(true),
        ])?;
        
        writer.write_row_typed(&[
            CellValue::String("2024-12-02".to_string()),
            CellValue::String("Mouse".to_string()),
            CellValue::Int(20),
            CellValue::Float(29.99),
            CellValue::Float(599.80),
            CellValue::Bool(false),
        ])?;
        
        writer.write_row_typed(&[
            CellValue::String("2024-12-03".to_string()),
            CellValue::String("Keyboard".to_string()),
            CellValue::Int(10),
            CellValue::Float(89.99),
            CellValue::Float(899.90),
            CellValue::Bool(true),
        ])?;
        
        writer.save()?;
    }
    println!("   ✓ File created: financial_report.xlsx");
    println!("   - Numbers can be formatted as currency in Excel");
    println!("   - SUM/AVERAGE formulas work correctly");
    println!("   - Boolean shows paid/unpaid status clearly\n");
    
    println!("=== Summary ===");
    println!("✅ Use write_row() when:");
    println!("   - All data is already strings");
    println!("   - Simple text-based exports");
    println!("   - Performance is critical (slightly faster)");
    println!();
    println!("✅ Use write_row_typed() when:");
    println!("   - Need proper data types in Excel");
    println!("   - Users will use Excel formulas");
    println!("   - Want numbers formatted as numbers, not text");
    println!("   - Working with mixed data types (int, float, bool, string)");
    
    Ok(())
}
