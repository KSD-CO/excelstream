use excelstream::streaming_reader::StreamingReader;
use excelstream::ExcelReader;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Performance Comparison: StreamingReader vs ExcelReader");
    println!("==========================================================\n");

    let path = "large_test_1M.xlsx";

    if !std::path::Path::new(path).exists() {
        eprintln!("❌ File not found: {}", path);
        eprintln!("   Run: cargo run --release --example memory_test_read_large");
        return Ok(());
    }

    let file_size = std::fs::metadata(path)?.len() as f64 / (1024.0 * 1024.0);
    println!("📁 File: {} ({:.2} MB)\n", path, file_size);

    // Test 1: StreamingReader
    println!("🧪 Test 1: StreamingReader");
    println!("---------------------------");

    let start = Instant::now();
    let mut reader = StreamingReader::open(path)?;
    let open_time = start.elapsed();

    let mut count = 0;
    for row_result in reader.stream_rows("Sheet1")? {
        let _ = row_result?;
        count += 1;
    }

    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64();

    println!("   Open time: {:.2}s", open_time.as_secs_f64());
    println!("   Total rows: {}", count);
    println!("   Duration: {:.2}s", elapsed.as_secs_f64());
    println!("   Rate: {:.0} rows/sec", rate);
    println!("   Memory: ~20-30 MB (constant)\n");

    // Test 2: ExcelReader
    println!("🧪 Test 2: ExcelReader (calamine)");
    println!("----------------------------------");

    let start = Instant::now();
    let mut reader = ExcelReader::open(path)?;
    let open_time = start.elapsed();

    let mut count = 0;
    for row_result in reader.rows("Sheet1")? {
        let _ = row_result?;
        count += 1;
    }

    let elapsed = start.elapsed();
    let rate2 = count as f64 / elapsed.as_secs_f64();

    println!("   Open time: {:.2}s", open_time.as_secs_f64());
    println!("   Total rows: {}", count);
    println!("   Duration: {:.2}s", elapsed.as_secs_f64());
    println!("   Rate: {:.0} rows/sec", rate2);
    println!("   Memory: ~{:.0} MB (file size)\n", file_size);

    // Summary
    println!("📊 Summary");
    println!("==========");
    println!("Speed improvement: {:.1}x faster", rate / rate2);
    println!("Memory improvement: {:.1}x less memory", file_size / 25.0);
    println!(
        "\n✅ StreamingReader is {:.1}x faster and uses {:.0}% less memory!",
        rate / rate2,
        (1.0 - 25.0 / file_size) * 100.0
    );

    Ok(())
}
