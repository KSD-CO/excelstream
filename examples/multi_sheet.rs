//! Advanced example: Multi-sheet workbook creation

use excelstream::writer::ExcelWriterBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating multi-sheet workbook...\n");

    // Create writer with custom sheet name
    let mut writer = ExcelWriterBuilder::new("examples/multi_sheet.xlsx")
        .with_sheet_name("Sales")
        .build()?;

    // Write Sales sheet
    println!("Writing Sales sheet...");
    writer.write_header(&["Month", "Revenue", "Costs", "Profit"])?;
    writer.write_row(&["January", "50000", "30000", "20000"])?;
    writer.write_row(&["February", "55000", "32000", "23000"])?;
    writer.write_row(&["March", "60000", "35000", "25000"])?;

    // Add Employees sheet
    println!("Writing Employees sheet...");
    writer.add_sheet("Employees")?;
    writer.write_header(&["ID", "Name", "Department", "Salary"])?;
    writer.write_row(&["1", "Alice", "Engineering", "75000"])?;
    writer.write_row(&["2", "Bob", "Sales", "65000"])?;
    writer.write_row(&["3", "Carol", "Marketing", "70000"])?;

    // Add Products sheet
    println!("Writing Products sheet...");
    writer.add_sheet("Products")?;
    writer.write_header(&["SKU", "Name", "Price", "Stock"])?;
    writer.write_row(&["P001", "Widget A", "19.99", "100"])?;
    writer.write_row(&["P002", "Widget B", "29.99", "50"])?;
    writer.write_row(&["P003", "Widget C", "39.99", "75"])?;

    // Save workbook
    writer.save()?;

    println!("\nMulti-sheet workbook created successfully!");
    println!("File: examples/multi_sheet.xlsx");

    Ok(())
}
