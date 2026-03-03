//! Example demonstrating cell formatting with all available styles
//!
//! Run with: cargo run --example cell_formatting
//!
//! This example creates an Excel file showcasing all 14 predefined cell styles including:
//! - Header bold formatting
//! - Number formats (integer, decimal, currency, percentage)
//! - Date/timestamp formats
//! - Text styles (bold, italic)
//! - Background highlights (yellow, green, red)
//! - Border styles

use excelstream::types::{CellStyle, CellValue};
use excelstream::writer::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating styled Excel file...");

    let mut writer = ExcelWriter::new("output_formatted.xlsx")?;

    // 1. Header with bold formatting
    println!("Writing bold header...");
    writer.write_header_bold(["Style", "Example Value", "Description"])?;

    // 2. Default style (no formatting)
    writer.write_row_styled(&[
        (CellValue::String("Default".to_string()), CellStyle::Default),
        (
            CellValue::String("Plain text".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("No special formatting".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 3. Number Integer (#,##0)
    writer.write_row_styled(&[
        (
            CellValue::String("NumberInteger".to_string()),
            CellStyle::Default,
        ),
        (CellValue::Int(1234567), CellStyle::NumberInteger),
        (
            CellValue::String("Integer with thousand separator".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 4. Number Decimal (#,##0.00)
    writer.write_row_styled(&[
        (
            CellValue::String("NumberDecimal".to_string()),
            CellStyle::Default,
        ),
        (CellValue::Float(1234567.89), CellStyle::NumberDecimal),
        (
            CellValue::String("Decimal with 2 places".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 5. Number Currency ($#,##0.00)
    writer.write_row_styled(&[
        (
            CellValue::String("NumberCurrency".to_string()),
            CellStyle::Default,
        ),
        (CellValue::Float(1234.56), CellStyle::NumberCurrency),
        (
            CellValue::String("Currency format with $ symbol".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 6. Number Percentage (0.00%)
    writer.write_row_styled(&[
        (
            CellValue::String("NumberPercentage".to_string()),
            CellStyle::Default,
        ),
        (CellValue::Float(0.95), CellStyle::NumberPercentage),
        (
            CellValue::String("Percentage format (0.95 = 95%)".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 7. Date Default (MM/DD/YYYY)
    writer.write_row_styled(&[
        (
            CellValue::String("DateDefault".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::Float(44927.0), // Excel date serial for 2023-01-01
            CellStyle::DateDefault,
        ),
        (
            CellValue::String("Date format MM/DD/YYYY".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 8. Date Timestamp (MM/DD/YYYY HH:MM:SS)
    writer.write_row_styled(&[
        (
            CellValue::String("DateTimestamp".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::Float(44927.5), // Excel datetime serial
            CellStyle::DateTimestamp,
        ),
        (
            CellValue::String("DateTime with time component".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 9. Text Bold
    writer.write_row_styled(&[
        (
            CellValue::String("TextBold".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("Bold Text".to_string()),
            CellStyle::TextBold,
        ),
        (
            CellValue::String("Bold formatting for emphasis".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 10. Text Italic
    writer.write_row_styled(&[
        (
            CellValue::String("TextItalic".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("Italic Text".to_string()),
            CellStyle::TextItalic,
        ),
        (
            CellValue::String("Italic formatting for notes".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 11. Highlight Yellow
    writer.write_row_styled(&[
        (
            CellValue::String("HighlightYellow".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("Yellow Background".to_string()),
            CellStyle::HighlightYellow,
        ),
        (
            CellValue::String("Yellow background highlight".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 12. Highlight Green
    writer.write_row_styled(&[
        (
            CellValue::String("HighlightGreen".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("Green Background".to_string()),
            CellStyle::HighlightGreen,
        ),
        (
            CellValue::String("Green background highlight".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 13. Highlight Red
    writer.write_row_styled(&[
        (
            CellValue::String("HighlightRed".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("Red Background".to_string()),
            CellStyle::HighlightRed,
        ),
        (
            CellValue::String("Red background highlight".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 14. Border Thin
    writer.write_row_styled(&[
        (
            CellValue::String("BorderThin".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::String("With Borders".to_string()),
            CellStyle::BorderThin,
        ),
        (
            CellValue::String("Thin borders on all sides".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // 15. Date Time Short (MM/DD/YYYY HH:MM)
    writer.write_row_styled(&[
        (
            CellValue::String("DateTimeShort".to_string()),
            CellStyle::Default,
        ),
        (
            CellValue::Float(44927.5), // Excel datetime serial
            CellStyle::DateTimeShort,
        ),
        (
            CellValue::String("DateTime without seconds".to_string()),
            CellStyle::Default,
        ),
    ])?;

    // Demonstrate write_row_with_style() - all cells with same style
    writer.write_row(["", "", ""])?; // Empty row
    writer.write_header_bold(["Convenience Method Demo"])?;

    writer.write_row_with_style(
        &[
            CellValue::String("All".to_string()),
            CellValue::String("cells".to_string()),
            CellValue::String("are".to_string()),
            CellValue::String("bold".to_string()),
        ],
        CellStyle::TextBold,
    )?;

    // Practical example: Financial report
    writer.write_row(["", "", ""])?; // Empty row
    writer.write_header_bold(["Item", "Amount", "Change %"])?;

    writer.write_row_styled(&[
        (CellValue::String("Revenue".to_string()), CellStyle::Default),
        (CellValue::Float(150000.00), CellStyle::NumberCurrency),
        (CellValue::Float(0.15), CellStyle::NumberPercentage),
    ])?;

    writer.write_row_styled(&[
        (
            CellValue::String("Expenses".to_string()),
            CellStyle::Default,
        ),
        (CellValue::Float(95000.00), CellStyle::NumberCurrency),
        (CellValue::Float(0.08), CellStyle::NumberPercentage),
    ])?;

    writer.write_row_styled(&[
        (CellValue::String("Profit".to_string()), CellStyle::TextBold),
        (CellValue::Float(55000.00), CellStyle::NumberCurrency),
        (CellValue::Float(0.22), CellStyle::NumberPercentage),
    ])?;

    writer.save()?;

    println!("✅ Successfully created output_formatted.xlsx");
    println!("   Open the file in Excel to see all 15 cell styles!");
    println!();
    println!("Available styles:");
    println!("  - Default: No formatting");
    println!("  - HeaderBold: Bold header text");
    println!("  - NumberInteger: #,##0");
    println!("  - NumberDecimal: #,##0.00");
    println!("  - NumberCurrency: $#,##0.00");
    println!("  - NumberPercentage: 0.00%");
    println!("  - DateDefault: MM/DD/YYYY");
    println!("  - DateTimestamp: MM/DD/YYYY HH:MM:SS");
    println!("  - TextBold: Bold text");
    println!("  - TextItalic: Italic text");
    println!("  - HighlightYellow: Yellow background");
    println!("  - HighlightGreen: Green background");
    println!("  - HighlightRed: Red background");
    println!("  - BorderThin: Thin borders");
    println!("  - DateTimeShort: MM/DD/YYYY HH:MM");

    Ok(())
}
