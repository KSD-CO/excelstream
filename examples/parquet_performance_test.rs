//! Parquet Performance Test - Measure memory usage and throughput
//!
//! This example creates a test Excel file, converts to Parquet and back,
//! measuring memory usage and performance.
//!
//! Run with:
//! ```bash
//! # Small test (1K rows)
//! TEST_ROWS=1000 /usr/bin/time -v cargo run --release --example parquet_performance_test --features parquet-support
//!
//! # Medium test (10K rows)
//! TEST_ROWS=10000 /usr/bin/time -v cargo run --release --example parquet_performance_test --features parquet-support
//!
//! # Large test (100K rows)
//! TEST_ROWS=100000 /usr/bin/time -v cargo run --release --example parquet_performance_test --features parquet-support
//! ```

#[cfg(feature = "parquet-support")]
use excelstream::parquet::{ExcelToParquetConverter, ParquetToExcelConverter};
#[cfg(feature = "parquet-support")]
use excelstream::ExcelWriter;
#[cfg(feature = "parquet-support")]
use std::time::Instant;

#[cfg(feature = "parquet-support")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_rows: usize = std::env::var("TEST_ROWS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .unwrap_or(10000);

    println!("🔬 Parquet Performance Test");
    println!("============================");
    println!("Test rows: {}\n", test_rows);

    // Step 1: Create test Excel file
    println!("📝 Step 1: Creating test Excel file...");
    let excel_file = "test_data.xlsx";
    let start = Instant::now();

    let mut writer = ExcelWriter::new(excel_file)?;
    writer.write_header_bold([
        "ID",
        "Name",
        "Email",
        "Department",
        "Salary",
        "Hire Date",
        "Active",
    ])?;

    for i in 1..=test_rows {
        let id = i.to_string();
        let name = format!("Employee {}", i);
        let email = format!("emp{}@company.com", i);
        let dept = ["Engineering", "Sales", "Marketing", "HR"][i % 4];
        let salary = format!("{:.2}", 50000.0 + (i as f64 * 100.0));
        let hire_date = format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1);
        let active = if i % 10 == 0 { "false" } else { "true" };

        writer.write_row([&id, &name, &email, dept, &salary, &hire_date, active])?;
    }

    writer.save()?;
    let excel_time = start.elapsed();

    println!(
        "   ✅ Created Excel: {} rows in {:.2}s",
        test_rows,
        excel_time.as_secs_f64()
    );
    println!(
        "   📊 Throughput: {:.0} rows/sec\n",
        test_rows as f64 / excel_time.as_secs_f64()
    );

    // Step 2: Convert Excel → Parquet
    println!("🔄 Step 2: Converting Excel → Parquet...");
    let parquet_file = "test_data.parquet";
    let start = Instant::now();

    let converter = ExcelToParquetConverter::new(excel_file)?;
    let row_count = converter.convert_to_parquet(parquet_file)?;

    let convert_time = start.elapsed();

    println!(
        "   ✅ Converted {} rows in {:.2}s",
        row_count,
        convert_time.as_secs_f64()
    );
    println!(
        "   📊 Throughput: {:.0} rows/sec\n",
        row_count as f64 / convert_time.as_secs_f64()
    );

    // Check file sizes
    let excel_size = std::fs::metadata(excel_file)?.len();
    let parquet_size = std::fs::metadata(parquet_file)?.len();
    let compression_ratio = excel_size as f64 / parquet_size as f64;

    println!(
        "   📦 Excel size:   {:.2} MB",
        excel_size as f64 / 1_048_576.0
    );
    println!(
        "   📦 Parquet size: {:.2} MB",
        parquet_size as f64 / 1_048_576.0
    );
    println!("   📉 Compression:  {:.2}x smaller\n", compression_ratio);

    // Step 3: Convert Parquet → Excel
    println!("🔄 Step 3: Converting Parquet → Excel...");
    let output_file = "test_output.xlsx";
    let start = Instant::now();

    let converter = ParquetToExcelConverter::new(parquet_file)?;
    let row_count = converter.convert_with_progress(output_file, |current, total| {
        if current % 5000 == 0 || current == total {
            let percent = (current as f64 / total as f64 * 100.0) as u32;
            println!("   Progress: {}% ({}/{} rows)", percent, current, total);
        }
    })?;

    let convert_back_time = start.elapsed();

    println!(
        "   ✅ Converted {} rows in {:.2}s",
        row_count,
        convert_back_time.as_secs_f64()
    );
    println!(
        "   📊 Throughput: {:.0} rows/sec\n",
        row_count as f64 / convert_back_time.as_secs_f64()
    );

    // Summary
    println!("📈 PERFORMANCE SUMMARY");
    println!("========================");
    println!("Total rows:        {}", test_rows);
    println!();
    println!(
        "Excel write:       {:.2}s ({:.0} rows/s)",
        excel_time.as_secs_f64(),
        test_rows as f64 / excel_time.as_secs_f64()
    );
    println!(
        "Excel → Parquet:   {:.2}s ({:.0} rows/s)",
        convert_time.as_secs_f64(),
        test_rows as f64 / convert_time.as_secs_f64()
    );
    println!(
        "Parquet → Excel:   {:.2}s ({:.0} rows/s)",
        convert_back_time.as_secs_f64(),
        test_rows as f64 / convert_back_time.as_secs_f64()
    );
    println!();
    println!("File sizes:");
    println!("  Excel:    {:.2} MB", excel_size as f64 / 1_048_576.0);
    println!(
        "  Parquet:  {:.2} MB ({:.2}x compression)",
        parquet_size as f64 / 1_048_576.0,
        compression_ratio
    );
    println!();
    println!("💡 Memory usage:");
    println!("   Check 'Maximum resident set size' from /usr/bin/time output");
    println!();
    println!("🧹 Cleanup:");
    println!(
        "   Files created: {}, {}, {}",
        excel_file, parquet_file, output_file
    );

    Ok(())
}

#[cfg(not(feature = "parquet-support"))]
fn main() {
    eprintln!("❌ This example requires the 'parquet-support' feature.");
    eprintln!("\nRun with:");
    eprintln!("  TEST_ROWS=10000 /usr/bin/time -v cargo run --release --example parquet_performance_test --features parquet-support");
    std::process::exit(1);
}
