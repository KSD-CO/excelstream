//! Demonstrates worksheet protection features
//!
//! This example shows:
//! - Protecting worksheet with password
//! - Controlling what users can/cannot do
//! - Different protection levels

use excelstream::{CellStyle, CellValue, ExcelWriter, ProtectionOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Excel file with worksheet protection...\n");

    let mut writer = ExcelWriter::new("worksheet_protection.xlsx")?;

    // ===== Sheet 1: Basic Protection with Password =====
    println!("Sheet 1: Basic protection with password");
    let protection = ProtectionOptions::new()
        .with_password("secret123")
        .allow_select_locked_cells(true)
        .allow_select_unlocked_cells(true);

    writer.protect_sheet(protection)?;

    writer.write_row_styled(&[(
        CellValue::String("Protected Sheet".to_string()),
        CellStyle::HeaderBold,
    )])?;
    writer.write_row([""])?;
    writer.write_row(["This sheet is protected with password: secret123"])?;
    writer.write_row(["Users can view but cannot edit without password"])?;

    // ===== Sheet 2: Read-Only Protection (No Password) =====
    writer.add_sheet("Read Only")?;
    println!("Sheet 2: Read-only protection (no password)");

    let protection = ProtectionOptions::new(); // No password
    writer.protect_sheet(protection)?;

    writer.write_row_styled(&[(
        CellValue::String("Read-Only Sheet".to_string()),
        CellStyle::HeaderBold,
    )])?;
    writer.write_row([""])?;
    writer.write_row(["This sheet is protected without password"])?;
    writer.write_row(["Users cannot edit, but can unprotect from Excel menu"])?;

    // ===== Sheet 3: Selective Permissions =====
    writer.add_sheet("Selective Permissions")?;
    println!("Sheet 3: Allow formatting but not inserting/deleting");

    let protection = ProtectionOptions::new()
        .with_password("format123")
        .allow_select_locked_cells(true)
        .allow_select_unlocked_cells(true)
        .allow_format_cells(true) // Allow formatting
        .allow_format_columns(true) // Allow column formatting
        .allow_format_rows(true); // Allow row formatting

    writer.protect_sheet(protection)?;

    writer.write_row_styled(&[(
        CellValue::String("Selective Permissions".to_string()),
        CellStyle::HeaderBold,
    )])?;
    writer.write_row([""])?;
    writer.write_row(["Password: format123"])?;
    writer.write_row(["Users CAN: Format cells, columns, rows"])?;
    writer.write_row(["Users CANNOT: Insert/delete rows/columns, edit cell values"])?;

    // ===== Sheet 4: Data Entry Sheet =====
    writer.add_sheet("Data Entry")?;
    println!("Sheet 4: Allow inserting rows for data entry");

    let protection = ProtectionOptions::new()
        .with_password("data456")
        .allow_select_locked_cells(true)
        .allow_select_unlocked_cells(true)
        .allow_insert_rows(true) // Allow inserting rows
        .allow_delete_rows(true) // Allow deleting rows
        .allow_sort(true); // Allow sorting

    writer.protect_sheet(protection)?;

    writer.write_row_styled(&[
        (CellValue::String("Name".to_string()), CellStyle::HeaderBold),
        (
            CellValue::String("Email".to_string()),
            CellStyle::HeaderBold,
        ),
        (
            CellValue::String("Phone".to_string()),
            CellStyle::HeaderBold,
        ),
    ])?;
    writer.write_row(["Alice", "alice@example.com", "555-0001"])?;
    writer.write_row(["Bob", "bob@example.com", "555-0002"])?;
    writer.write_row([""])?;
    writer.write_row(["Password: data456"])?;
    writer.write_row(["Users CAN: Insert/delete rows, sort data"])?;
    writer.write_row(["Users CANNOT: Edit header, insert/delete columns"])?;

    // ===== Sheet 5: No Protection (For Comparison) =====
    writer.add_sheet("No Protection")?;
    println!("Sheet 5: No protection (fully editable)");

    writer.write_row_styled(&[(
        CellValue::String("Unprotected Sheet".to_string()),
        CellStyle::HeaderBold,
    )])?;
    writer.write_row([""])?;
    writer.write_row(["This sheet has NO protection"])?;
    writer.write_row(["Users can edit anything"])?;

    writer.save()?;

    println!("\n✅ Created worksheet_protection.xlsx");
    println!("\nFeatures demonstrated:");
    println!("  1. Basic protection with password");
    println!("  2. Read-only protection without password");
    println!("  3. Selective permissions (formatting allowed)");
    println!("  4. Data entry protection (insert/delete rows allowed)");
    println!("  5. Unprotected sheet for comparison");
    println!("\nTry opening in Excel and testing different protection levels!");

    Ok(())
}
