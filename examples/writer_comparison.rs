//! Comparison between standard writer and fast writer

use excelstream::writer::ExcelWriter;
use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Standard Writer vs Fast Writer Comparison ===\n");

    const NUM_ROWS: usize = 100_000;

    // Test standard writer
    println!("Standard Writer (rust_xlsxwriter): Writing {} rows...", NUM_ROWS);
    let start = Instant::now();
    test_standard_writer("standard_writer.xlsx", NUM_ROWS)?;
    let standard_duration = start.elapsed();
    let standard_speed = NUM_ROWS as f64 / standard_duration.as_secs_f64();
    println!("  Time: {:?}", standard_duration);
    println!("  Speed: {:.0} rows/sec\n", standard_speed);

    // Test fast writer
    println!("Fast Writer (custom implementation): Writing {} rows...", NUM_ROWS);
    let start = Instant::now();
    test_fast_writer("fast_writer.xlsx", NUM_ROWS)?;
    let fast_duration = start.elapsed();
    let fast_speed = NUM_ROWS as f64 / fast_duration.as_secs_f64();
    println!("  Time: {:?}", fast_duration);
    println!("  Speed: {:.0} rows/sec\n", fast_speed);

    // Calculate improvement
    let improvement = (fast_speed / standard_speed - 1.0) * 100.0;
    println!("=== Results ===");
    println!("Fast Writer is {:.1}% faster than Standard Writer", improvement);
    println!("Speedup: {:.2}x", fast_speed / standard_speed);

    Ok(())
}

fn test_standard_writer(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;

    // Write header
    writer.write_row(&["ID", "Name", "Email", "Age", "City"])?;

    // Write data rows
    for i in 1..=num_rows {
        let values = vec![
            format!("{}", i),
            format!("User{}", i),
            format!("user{}@example.com", i),
            format!("{}", 20 + (i % 50)),
            format!("City{}", i % 100),
        ];
        let values_str: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
        writer.write_row(&values_str)?;
    }

    writer.save()?;
    Ok(())
}

fn test_fast_writer(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;
    workbook.add_worksheet("Sheet1")?;

    // Write header
    workbook.write_row(&["ID", "Name", "Email", "Age", "City"])?;

    // Write data rows
    for i in 1..=num_rows {
        let id = i.to_string();
        let name = format!("User{}", i);
        let email = format!("user{}@example.com", i);
        let age = (20 + (i % 50)).to_string();
        let city = format!("City{}", i % 100);

        workbook.write_row(&[&id, &name, &email, &age, &city])?;
    }

    workbook.close()?;
    Ok(())
}
