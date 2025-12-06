use excelstream::streaming_reader::StreamingReader;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📖 Testing Streaming Reader");
    println!("================================\n");

    // Use the large test file created earlier
    let path = "memory_test_aggressive.xlsx";

    if !std::path::Path::new(path).exists() {
        eprintln!("❌ File not found: {}", path);
        eprintln!("   Run: cargo run --release --example memory_test_read_large");
        return Ok(());
    }

    let file_size = std::fs::metadata(path)?.len() as f64 / (1024.0 * 1024.0);
    println!("📁 File: {} ({:.2} MB)", path, file_size);

    // Open file (loads SST only)
    println!("\n⏱️  Opening file...");
    let start = Instant::now();
    let mut reader = StreamingReader::open(path)?;
    let open_time = start.elapsed();
    println!("   ✅ Opened in {:.2}s", open_time.as_secs_f64());

    // Stream rows
    println!("\n⏱️  Streaming rows...");
    let start = Instant::now();

    let mut row_count = 0;
    let mut cell_count = 0;

    for row_result in reader.stream_rows("Sheet1")? {
        let row = row_result?;
        cell_count += row.len();
        row_count += 1;

        // Print first 3 rows
        if row_count <= 3 {
            println!("   Row {}: {:?}", row_count, row);
        }

        // Progress every 100K rows
        if row_count % 100_000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = row_count as f64 / elapsed;
            println!(
                "   📊 {:.0}K rows | {:.0} rows/sec",
                row_count as f64 / 1000.0,
                rate
            );
        }
    }

    let elapsed = start.elapsed();
    let rate = row_count as f64 / elapsed.as_secs_f64();

    println!("\n✅ Streaming Complete!");
    println!("   Rows: {}", row_count);
    println!("   Cells: {}", cell_count);
    println!("   Time: {:.2}s", elapsed.as_secs_f64());
    println!("   Rate: {:.0} rows/sec", rate);

    println!("\n📊 Memory Usage:");
    println!("   - SST loaded at open()");
    println!("   - Worksheet streamed (not loaded fully)");
    println!("   - Expected: constant ~20-30 MB");

    Ok(())
}
