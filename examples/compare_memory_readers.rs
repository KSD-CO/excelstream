use excelstream::streaming_reader::StreamingReader;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔬 Memory Profiling: Streaming Reader vs Calamine");
    println!("================================================\n");

    let path = "large_test_1M.xlsx";

    if !std::path::Path::new(path).exists() {
        eprintln!("❌ File not found: {}", path);
        return Ok(());
    }

    let file_size = std::fs::metadata(path)?.len() as f64 / (1024.0 * 1024.0);
    println!("📁 File: {} ({:.2} MB)\n", path, file_size);

    // Test 1: Our Streaming Reader
    println!("🧪 Test 1: StreamingReader (constant memory)");
    println!("--------------------------------------------");

    let start = Instant::now();
    let mut reader = StreamingReader::open(path)?;
    println!("   ✅ File opened (SST loaded)");
    println!("   💡 Check memory now - should be ~10-20 MB\n");

    println!("   Press Enter to start streaming rows...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let mut count = 0;
    for row_result in reader.stream_rows("Sheet1")? {
        let _ = row_result?;
        count += 1;

        if count % 250_000 == 0 {
            println!(
                "   📊 {} rows | {:.0} rows/sec",
                count,
                count as f64 / start.elapsed().as_secs_f64()
            );
            println!("   💡 Check memory - should still be ~20-30 MB (constant)");
        }
    }

    println!(
        "\n   ✅ Streaming complete: {} rows in {:.2}s",
        count,
        start.elapsed().as_secs_f64()
    );
    println!("   💡 Final memory check - should be same as start\n");

    // Test 2: Calamine (loads entire file)
    println!("\n🧪 Test 2: Calamine (loads entire file)");
    println!("--------------------------------------------");

    println!("   Press Enter to test calamine...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    use excelstream::ExcelReader;

    let start = Instant::now();
    let mut reader = ExcelReader::open(path)?;
    println!("   ✅ File opened");
    println!(
        "   💡 Check memory now - should be ~{:.0} MB (file size loaded)",
        file_size
    );

    println!("\n   Press Enter to iterate rows...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let mut count = 0;
    for row_result in reader.rows("Sheet1")? {
        let _ = row_result?;
        count += 1;

        if count % 250_000 == 0 {
            println!(
                "   📊 {} rows | {:.0} rows/sec",
                count,
                count as f64 / start.elapsed().as_secs_f64()
            );
        }
    }

    println!(
        "\n   ✅ Reading complete: {} rows in {:.2}s",
        count,
        start.elapsed().as_secs_f64()
    );
    println!(
        "   💡 Memory should still be ~{:.0} MB (held until drop)",
        file_size
    );

    println!("\n📊 Summary");
    println!("==========");
    println!("StreamingReader: Constant ~20-30 MB memory");
    println!("Calamine:        ~{:.0} MB memory (entire file)", file_size);
    println!("\nFor very large files (>1 GB), StreamingReader is MUCH better! 🚀");

    Ok(())
}
