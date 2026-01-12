//! Example: Convert Parquet file to Excel with streaming
//!
//! This example demonstrates how to convert a Parquet file to Excel format
//! using streaming to handle large files efficiently.
//!
//! Features:
//! - Constant memory usage (~10-20 MB)
//! - Streams data row-by-row
//! - Progress reporting
//! - Type-safe conversion
//!
//! Run with:
//! ```bash
//! cargo run --example parquet_to_excel --features parquet-support
//! ```

#[cfg(feature = "parquet-support")]
use excelstream::parquet::ParquetToExcelConverter;

#[cfg(feature = "parquet-support")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Parquet → Excel Conversion Example\n");

    // Input and output paths
    let parquet_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data.parquet".to_string());
    let excel_file = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "output.xlsx".to_string());

    println!("📂 Input:  {}", parquet_file);
    println!("📂 Output: {}\n", excel_file);

    // Create converter
    let converter = ParquetToExcelConverter::new(&parquet_file)?;

    // Convert with progress reporting
    println!("🔄 Converting...");
    let row_count = converter.convert_with_progress(&excel_file, |current, total| {
        let percent = (current as f64 / total as f64 * 100.0) as u32;
        println!("   Progress: {}% ({}/{} rows)", percent, current, total);
    })?;

    println!("\n✅ Conversion complete!");
    println!("   Rows converted: {}", row_count);
    println!("   Output file: {}", excel_file);
    println!("\n💡 Features:");
    println!("   ✅ Constant memory usage (~10-20 MB)");
    println!("   ✅ Streaming row-by-row processing");
    println!("   ✅ Automatic type conversion");

    Ok(())
}

#[cfg(not(feature = "parquet-support"))]
fn main() {
    eprintln!("❌ This example requires the 'parquet-support' feature.");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example parquet_to_excel --features parquet-support [input.parquet] [output.xlsx]");
    std::process::exit(1);
}
