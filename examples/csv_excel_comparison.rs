//! CSV vs Excel Writers Performance Comparison
//!
//! This example compares performance between:
//! 1. CsvWriter (plain) - Uncompressed CSV
//! 2. CsvWriter (Zstd) - Zstd compressed CSV
//! 3. CsvWriter (Deflate) - Deflate/Gzip compressed CSV
//! 4. ExcelWriter - Standard Excel XLSX format

use excelstream::types::CellValue;
use excelstream::{CompressionMethod, CsvWriter, ExcelWriter};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CSV vs Excel Writers Performance Comparison ===\n");

    const NUM_ROWS: usize = 100_000;
    const NUM_COLS: usize = 10;

    println!("Test configuration:");
    println!("- Rows: {}", NUM_ROWS);
    println!("- Columns: {}", NUM_COLS);
    println!(
        "- Total cells: {} million\n",
        (NUM_ROWS * NUM_COLS) / 1_000_000
    );

    // Test 1: CSV Plain
    println!("1. CSV Plain (uncompressed):");
    let start = Instant::now();
    test_csv_plain("test_csv_plain.csv", NUM_ROWS, NUM_COLS)?;
    let duration1 = start.elapsed();
    let speed1 = NUM_ROWS as f64 / duration1.as_secs_f64();
    let size1 = std::fs::metadata("test_csv_plain.csv")?.len();
    println!("   Time: {:?}", duration1);
    println!("   Speed: {:.0} rows/sec", speed1);
    println!("   File size: {:.2} MB\n", size1 as f64 / 1024.0 / 1024.0);

    // Test 2: CSV Zstd
    println!("2. CSV Zstd (level 3):");
    let start = Instant::now();
    test_csv_zstd("test_csv_zstd.csv.zst", NUM_ROWS, NUM_COLS)?;
    let duration2 = start.elapsed();
    let speed2 = NUM_ROWS as f64 / duration2.as_secs_f64();
    let size2 = std::fs::metadata("test_csv_zstd.csv.zst")?.len();
    println!("   Time: {:?}", duration2);
    println!("   Speed: {:.0} rows/sec", speed2);
    println!("   File size: {:.2} MB\n", size2 as f64 / 1024.0 / 1024.0);

    // Test 3: CSV Deflate
    println!("3. CSV Deflate/Gzip (level 6):");
    let start = Instant::now();
    test_csv_deflate("test_csv_deflate.csv.gz", NUM_ROWS, NUM_COLS)?;
    let duration3 = start.elapsed();
    let speed3 = NUM_ROWS as f64 / duration3.as_secs_f64();
    let size3 = std::fs::metadata("test_csv_deflate.csv.gz")?.len();
    println!("   Time: {:?}", duration3);
    println!("   Speed: {:.0} rows/sec", speed3);
    println!("   File size: {:.2} MB\n", size3 as f64 / 1024.0 / 1024.0);

    // Test 4: Excel XLSX
    println!("4. Excel XLSX (Deflate level 6):");
    let start = Instant::now();
    test_excel("test_excel.xlsx", NUM_ROWS, NUM_COLS)?;
    let duration4 = start.elapsed();
    let speed4 = NUM_ROWS as f64 / duration4.as_secs_f64();
    let size4 = std::fs::metadata("test_excel.xlsx")?.len();
    println!("   Time: {:?}", duration4);
    println!("   Speed: {:.0} rows/sec", speed4);
    println!("   File size: {:.2} MB\n", size4 as f64 / 1024.0 / 1024.0);

    // Performance Analysis
    println!("=== Performance Summary ===");
    println!("┌──────────────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ Format               │ Speed        │ File Size    │ Compression  │");
    println!("├──────────────────────┼──────────────┼──────────────┼──────────────┤");
    println!(
        "│ CSV Plain            │ {:>6.0} r/s   │ {:>8.2} MB  │ None         │",
        speed1,
        size1 as f64 / 1024.0 / 1024.0
    );
    println!(
        "│ CSV Zstd (level 3)   │ {:>6.0} r/s   │ {:>8.2} MB  │ {:.1}%        │",
        speed2,
        size2 as f64 / 1024.0 / 1024.0,
        (size2 as f64 / size1 as f64) * 100.0
    );
    println!(
        "│ CSV Gzip (level 6)   │ {:>6.0} r/s   │ {:>8.2} MB  │ {:.1}%        │",
        speed3,
        size3 as f64 / 1024.0 / 1024.0,
        (size3 as f64 / size1 as f64) * 100.0
    );
    println!(
        "│ Excel XLSX           │ {:>6.0} r/s   │ {:>8.2} MB  │ {:.1}%        │",
        speed4,
        size4 as f64 / 1024.0 / 1024.0,
        (size4 as f64 / size1 as f64) * 100.0
    );
    println!("└──────────────────────┴──────────────┴──────────────┴──────────────┘");
    println!();

    // Speed comparison
    println!("=== Speed Comparison (vs CSV Plain) ===");
    println!(
        "CSV Zstd:     {:.2}x ({:+.0}%)",
        speed2 / speed1,
        ((speed2 - speed1) / speed1) * 100.0
    );
    println!(
        "CSV Gzip:     {:.2}x ({:+.0}%)",
        speed3 / speed1,
        ((speed3 - speed1) / speed1) * 100.0
    );
    println!(
        "Excel XLSX:   {:.2}x ({:+.0}%)",
        speed4 / speed1,
        ((speed4 - speed1) / speed1) * 100.0
    );
    println!();

    // Compression ratio
    println!("=== Compression Ratio (vs CSV Plain) ===");
    println!(
        "CSV Zstd:     {:.1}x smaller ({:.1}% of original)",
        size1 as f64 / size2 as f64,
        (size2 as f64 / size1 as f64) * 100.0
    );
    println!(
        "CSV Gzip:     {:.1}x smaller ({:.1}% of original)",
        size1 as f64 / size3 as f64,
        (size3 as f64 / size1 as f64) * 100.0
    );
    println!(
        "Excel XLSX:   {:.1}x smaller ({:.1}% of original)",
        size1 as f64 / size4 as f64,
        (size4 as f64 / size1 as f64) * 100.0
    );
    println!();

    println!("=== Recommendations ===");
    println!();
    println!("✅ Use CSV Plain when:");
    println!("   - Maximum write speed is critical");
    println!("   - Disk space is not a concern");
    println!("   - Processing with other tools (awk, grep, etc.)");
    println!("   - Speed: {:.0} rows/sec (fastest)", speed1);
    println!();
    println!("✅ Use CSV Zstd when:");
    println!("   - Need best compression ratio");
    println!("   - Modern tools that support Zstd");
    println!("   - Bandwidth/storage cost optimization");
    println!(
        "   - Speed: {:.0} rows/sec, Size: {:.1}% of plain CSV",
        speed2,
        (size2 as f64 / size1 as f64) * 100.0
    );
    println!();
    println!("✅ Use CSV Gzip when:");
    println!("   - Wide compatibility needed");
    println!("   - Good balance of speed and compression");
    println!("   - Standard Unix tools support");
    println!(
        "   - Speed: {:.0} rows/sec, Size: {:.1}% of plain CSV",
        speed3,
        (size3 as f64 / size1 as f64) * 100.0
    );
    println!();
    println!("✅ Use Excel XLSX when:");
    println!("   - Need formulas, formatting, multiple sheets");
    println!("   - End users work with Excel/LibreOffice");
    println!("   - Rich data types and styling required");
    println!(
        "   - Speed: {:.0} rows/sec, Size: {:.1}% of plain CSV",
        speed4,
        (size4 as f64 / size1 as f64) * 100.0
    );
    println!();

    // Cleanup
    std::fs::remove_file("test_csv_plain.csv").ok();
    std::fs::remove_file("test_csv_zstd.csv.zst").ok();
    std::fs::remove_file("test_csv_deflate.csv.gz").ok();
    std::fs::remove_file("test_excel.xlsx").ok();

    Ok(())
}

fn test_csv_plain(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = CsvWriter::new(filename)?;

    // Header
    let headers: Vec<String> = (0..num_cols).map(|i| format!("Column_{}", i)).collect();
    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    writer.write_row(header_refs)?;

    // Data
    for i in 0..num_rows {
        let row = generate_row(i, num_cols);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        writer.write_row(row_refs)?;
    }

    writer.save()?;
    Ok(())
}

fn test_csv_zstd(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = CsvWriter::with_compression(filename, CompressionMethod::Zstd, 3)?;

    // Header
    let headers: Vec<String> = (0..num_cols).map(|i| format!("Column_{}", i)).collect();
    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    writer.write_row(header_refs)?;

    // Data
    for i in 0..num_rows {
        let row = generate_row(i, num_cols);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        writer.write_row(row_refs)?;
    }

    writer.save()?;
    Ok(())
}

fn test_csv_deflate(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = CsvWriter::with_compression(filename, CompressionMethod::Deflate, 6)?;

    // Header
    let headers: Vec<String> = (0..num_cols).map(|i| format!("Column_{}", i)).collect();
    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    writer.write_row(header_refs)?;

    // Data
    for i in 0..num_rows {
        let row = generate_row(i, num_cols);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        writer.write_row(row_refs)?;
    }

    writer.save()?;
    Ok(())
}

fn test_excel(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;

    // Header
    let headers: Vec<String> = (0..num_cols).map(|i| format!("Column_{}", i)).collect();
    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    writer.write_row(&header_refs)?;

    // Data
    for i in 0..num_rows {
        let row = generate_row_typed(i, num_cols);
        writer.write_row_typed(&row)?;
    }

    writer.save()?;
    Ok(())
}

fn generate_row(row_num: usize, num_cols: usize) -> Vec<String> {
    (0..num_cols)
        .map(|col| match col {
            0 => format!("{}", row_num),
            1 => format!("User_{}", row_num),
            2 => format!("user{}@example.com", row_num),
            3 => format!("{}", 20 + (row_num % 50)),
            4 => format!("{:.2}", 50000.0 + (row_num as f64 * 123.45)),
            5 => if row_num.is_multiple_of(2) {
                "true"
            } else {
                "false"
            }
            .to_string(),
            6 => format!("{:.1}", 50.0 + (row_num % 100) as f64),
            7 => format!("Department_{}", row_num % 5),
            8 => format!("2024-{:02}-{:02}", 1 + (row_num % 12), 1 + (row_num % 28)),
            _ => format!("Data_{}_{}", row_num, col),
        })
        .collect()
}

fn generate_row_typed(row_num: usize, num_cols: usize) -> Vec<CellValue> {
    (0..num_cols)
        .map(|col| match col {
            0 => CellValue::Int(row_num as i64),
            1 => CellValue::String(format!("User_{}", row_num)),
            2 => CellValue::String(format!("user{}@example.com", row_num)),
            3 => CellValue::Int((20 + (row_num % 50)) as i64),
            4 => CellValue::Float(50000.0 + (row_num as f64 * 123.45)),
            5 => CellValue::Bool(row_num.is_multiple_of(2)),
            6 => CellValue::Float(50.0 + (row_num % 100) as f64),
            7 => CellValue::String(format!("Department_{}", row_num % 5)),
            8 => CellValue::String(format!(
                "2024-{:02}-{:02}",
                1 + (row_num % 12),
                1 + (row_num % 28)
            )),
            _ => CellValue::String(format!("Data_{}_{}", row_num, col)),
        })
        .collect()
}
