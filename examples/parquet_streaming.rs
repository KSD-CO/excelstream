//! Example: Advanced Parquet streaming with custom processing
//!
//! This example shows how to use the low-level ParquetReader API
//! for custom streaming workflows.
//!
//! Run with:
//! ```bash
//! cargo run --example parquet_streaming --features parquet-support
//! ```

#[cfg(feature = "parquet-support")]
use excelstream::parquet::ParquetReader;
#[cfg(feature = "parquet-support")]
use excelstream::ExcelWriter;

#[cfg(feature = "parquet-support")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Advanced Parquet Streaming Example\n");

    let parquet_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data.parquet".to_string());

    println!("📂 Input: {}\n", parquet_file);

    // Open Parquet file
    let reader = ParquetReader::open(&parquet_file)?;

    // Print schema information
    println!("📊 Schema Information:");
    println!("   Columns: {}", reader.column_names().len());
    println!("   Total rows: {}", reader.row_count());
    println!("\n   Column names:");
    for (idx, name) in reader.column_names().iter().enumerate() {
        println!("     {}. {}", idx + 1, name);
    }
    println!();

    // Stream to Excel with filtering
    println!("🔄 Streaming with filter (rows with data in first column)...");
    let mut writer = ExcelWriter::new("filtered_output.xlsx")?;

    // Write headers
    writer.write_header_bold(reader.column_names())?;

    // Stream and filter rows
    let mut processed = 0;
    let mut filtered = 0;
    for row in reader.rows()? {
        let row_data = row?;
        processed += 1;

        // Filter: only keep rows where first column is not empty
        if !row_data.is_empty() && !row_data[0].is_empty() {
            writer.write_row(&row_data)?;
            filtered += 1;
        }

        // Progress every 10K rows
        if processed % 10000 == 0 {
            println!("   Processed: {} rows, filtered: {}", processed, filtered);
        }
    }

    writer.save()?;

    println!("\n✅ Complete!");
    println!("   Total processed: {}", processed);
    println!("   Rows filtered out: {}", processed - filtered);
    println!("   Rows written: {}", filtered);
    println!("   Output: filtered_output.xlsx");
    println!("\n💡 This example demonstrates:");
    println!("   ✅ Low-level ParquetReader API");
    println!("   ✅ Schema inspection");
    println!("   ✅ Row-by-row filtering");
    println!("   ✅ Memory-efficient streaming");

    Ok(())
}

#[cfg(not(feature = "parquet-support"))]
fn main() {
    eprintln!("❌ This example requires the 'parquet-support' feature.");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example parquet_streaming --features parquet-support [input.parquet]");
    std::process::exit(1);
}
