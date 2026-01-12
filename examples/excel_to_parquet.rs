//! Example: Convert Excel file to Parquet format
//!
//! This example demonstrates how to convert an Excel file to Parquet format
//! for efficient storage and analytics.
//!
//! Features:
//! - Columnar storage (smaller file size)
//! - Fast analytics queries
//! - Schema inference from Excel data
//! - Compression support
//!
//! Run with:
//! ```bash
//! cargo run --example excel_to_parquet --features parquet-support
//! ```

#[cfg(feature = "parquet-support")]
use excelstream::parquet::ExcelToParquetConverter;

#[cfg(feature = "parquet-support")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Excel → Parquet Conversion Example\n");

    // Input and output paths
    let excel_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data.xlsx".to_string());
    let parquet_file = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "output.parquet".to_string());

    println!("📂 Input:  {}", excel_file);
    println!("📂 Output: {}\n", parquet_file);

    // Create converter
    let converter = ExcelToParquetConverter::new(&excel_file)?;

    // Convert with progress reporting
    let row_count = converter.convert_with_progress(&parquet_file, |msg| {
        println!("   {}", msg);
    })?;

    println!("\n✅ Conversion complete!");
    println!("   Rows converted: {}", row_count);
    println!("   Output file: {}", parquet_file);
    println!("\n💡 Benefits:");
    println!("   ✅ Columnar storage (efficient analytics)");
    println!("   ✅ Automatic compression");
    println!("   ✅ Schema preservation");
    println!("   ✅ Fast query performance");
    println!("\n🔍 Verify with:");
    println!("   parquet-tools meta {}", parquet_file);
    println!("   parquet-tools cat {}", parquet_file);

    Ok(())
}

#[cfg(not(feature = "parquet-support"))]
fn main() {
    eprintln!("❌ This example requires the 'parquet-support' feature.");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example excel_to_parquet --features parquet-support [input.xlsx] [output.parquet]");
    std::process::exit(1);
}
