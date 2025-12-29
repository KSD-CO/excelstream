//! Test Zstd compression specifically

use excelstream::{CompressionMethod, CsvReader, CsvWriter};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Testing Zstd Compression ===\n");

    // Write with Zstd compression
    let zstd_path = "examples/test_zstd.csv.zst";
    println!("1. Writing 10K rows with Zstd level 5...");
    {
        let mut writer = CsvWriter::with_compression(zstd_path, CompressionMethod::Zstd, 5)?;

        writer.write_row(["ID", "Name", "Value", "Description"])?;
        for i in 0..10_000 {
            writer.write_row([
                &i.to_string(),
                &format!("Item_{}", i),
                &format!("{:.2}", i as f64 * 1.5),
                &format!("Description for item {}", i),
            ])?;
        }

        println!("   Rows written: {}", writer.row_count());
        writer.save()?;
    }

    // Check file size
    let metadata = std::fs::metadata(zstd_path)?;
    println!(
        "   Compressed file size: {:.2} KB",
        metadata.len() as f64 / 1024.0
    );

    // Read it back
    println!("\n2. Reading Zstd compressed file...");
    {
        let mut reader = CsvReader::open(zstd_path)?;
        let mut count = 0;

        for row_result in reader.rows() {
            let _row = row_result?;
            count += 1;

            if count % 2000 == 0 {
                println!("   Read {} rows...", count);
            }
        }

        println!("   Total rows read: {}", count);
    }

    // Compare different compression levels
    println!("\n3. Comparing Zstd compression levels...");
    for level in [1, 3, 5, 9] {
        let path = format!("examples/test_zstd_level_{}.csv.zst", level);

        let mut writer = CsvWriter::with_compression(&path, CompressionMethod::Zstd, level)?;

        writer.write_row(["ID", "Data"])?;
        for i in 0..5000 {
            writer.write_row([&i.to_string(), &format!("Data_{}", i)])?;
        }
        writer.save()?;

        let size = std::fs::metadata(&path)?.len();
        println!("   Level {}: {:.2} KB", level, size as f64 / 1024.0);

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    // Cleanup
    std::fs::remove_file(zstd_path).ok();

    println!("\n✓ Zstd compression verified successfully!");

    Ok(())
}
