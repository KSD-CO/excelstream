//! Basic example of writing an Excel file

use excelstream::ExcelWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Excel writer
    let mut writer = ExcelWriter::new("examples/output.xlsx")?;

    // Write header row with formatting
    writer.write_header(&["ID", "Name", "Email", "Age", "Salary"])?;

    // Write data rows
    writer.write_row(&["1", "Alice Johnson", "alice@example.com", "30", "75000"])?;
    writer.write_row(&["2", "Bob Smith", "bob@example.com", "25", "65000"])?;
    writer.write_row(&["3", "Carol White", "carol@example.com", "35", "85000"])?;
    writer.write_row(&["4", "David Brown", "david@example.com", "28", "70000"])?;

    // Set column widths
    writer.set_column_width(0, 5.0)?;  // ID
    writer.set_column_width(1, 20.0)?; // Name
    writer.set_column_width(2, 25.0)?; // Email
    writer.set_column_width(3, 8.0)?;  // Age
    writer.set_column_width(4, 12.0)?; // Salary

    // Save the file
    writer.save()?;

    println!("Excel file created successfully: examples/output.xlsx");
    Ok(())
}
