//! Example demonstrating the fast writer optimized for streaming large datasets

use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fast Writer Performance Test ===\n");

    // Test 1: Small dataset
    println!("Test 1: Writing 1,000 rows...");
    let start = Instant::now();
    test_fast_writer("fast_small.xlsx", 1_000)?;
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Speed: {:.0} rows/sec\n", 1_000.0 / duration.as_secs_f64());

    // Test 2: Medium dataset
    println!("Test 2: Writing 10,000 rows...");
    let start = Instant::now();
    test_fast_writer("fast_medium.xlsx", 10_000)?;
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Speed: {:.0} rows/sec\n", 10_000.0 / duration.as_secs_f64());

    // Test 3: Large dataset
    println!("Test 3: Writing 100,000 rows...");
    let start = Instant::now();
    test_fast_writer("fast_large.xlsx", 100_000)?;
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Speed: {:.0} rows/sec\n", 100_000.0 / duration.as_secs_f64());

    // Test 4: Very large dataset
    println!("Test 4: Writing 1,000,000 rows...");
    let start = Instant::now();
    test_fast_writer("fast_xlarge.xlsx", 1_000_000)?;
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Speed: {:.0} rows/sec\n", 1_000_000.0 / duration.as_secs_f64());

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
