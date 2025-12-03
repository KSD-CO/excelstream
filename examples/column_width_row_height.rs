//! Example demonstrating column width and row height customization
//!
//! Run with: cargo run --example column_width_row_height
//!
//! This example shows how to:
//! - Set column widths for better readability
//! - Set row heights for headers or special rows
//! - Combine with cell styling for professional reports

use excelstream::types::{CellStyle, CellValue};
use excelstream::writer::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Excel file with custom column widths and row heights...");

    let mut writer = ExcelWriter::new("output_sizes.xlsx")?;

    // Set column widths BEFORE writing data
    writer.set_column_width(0, 25.0)?; // Product name - wider
    writer.set_column_width(1, 12.0)?; // Quantity
    writer.set_column_width(2, 15.0)?; // Price
    writer.set_column_width(3, 15.0)?; // Total

    // Write header with tall row
    writer.set_next_row_height(25.0)?;
    writer.write_header_bold(["Product", "Quantity", "Price", "Total"])?;

    // Regular data rows
    writer.write_row_styled(&[
        (CellValue::String("Laptop".to_string()), CellStyle::Default),
        (CellValue::Int(5), CellStyle::NumberInteger),
        (CellValue::Float(1200.00), CellStyle::NumberCurrency),
        (
            CellValue::Formula("=B2*C2".to_string()),
            CellStyle::NumberCurrency,
        ),
    ])?;

    writer.write_row_styled(&[
        (CellValue::String("Mouse".to_string()), CellStyle::Default),
        (CellValue::Int(15), CellStyle::NumberInteger),
        (CellValue::Float(25.00), CellStyle::NumberCurrency),
        (
            CellValue::Formula("=B3*C3".to_string()),
            CellStyle::NumberCurrency,
        ),
    ])?;

    writer.write_row_styled(&[
        (
            CellValue::String("Keyboard".to_string()),
            CellStyle::Default,
        ),
        (CellValue::Int(8), CellStyle::NumberInteger),
        (CellValue::Float(75.00), CellStyle::NumberCurrency),
        (
            CellValue::Formula("=B4*C4".to_string()),
            CellStyle::NumberCurrency,
        ),
    ])?;

    // Total row with custom height and bold style
    writer.set_next_row_height(22.0)?;
    writer.write_row_styled(&[
        (CellValue::String("TOTAL".to_string()), CellStyle::TextBold),
        (
            CellValue::Formula("=SUM(B2:B4)".to_string()),
            CellStyle::NumberInteger,
        ),
        (CellValue::String("".to_string()), CellStyle::Default),
        (
            CellValue::Formula("=SUM(D2:D4)".to_string()),
            CellStyle::NumberCurrency,
        ),
    ])?;

    writer.save()?;

    println!("✅ Successfully created output_sizes.xlsx");
    println!("   - Column A (Product): 25 units wide");
    println!("   - Column B (Quantity): 12 units wide");
    println!("   - Columns C-D: 15 units wide");
    println!("   - Header row: 25 points tall");
    println!("   - Total row: 22 points tall");
    println!();
    println!("Open the file in Excel to see the custom column widths and row heights!");

    Ok(())
}
