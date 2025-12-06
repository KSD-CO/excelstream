//! Memory profiling for reading large Excel files
//!
//! This example shows ACTUAL memory usage with delays for monitoring

use excelstream::{CellValue, ExcelReader};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "memory_test_default.xlsx";

    println!("=== Memory Profiling: Reading Large Excel File ===\n");

    let file_size = std::fs::metadata(test_file)?.len();
    let file_size_mb = file_size as f64 / (1024.0 * 1024.0);

    println!("📁 File: {}", test_file);
    println!("📦 Size: {:.2} MB\n", file_size_mb);

    println!("⏳ Starting in 5 seconds...");
    println!("💡 Open Activity Monitor NOW to see baseline memory\n");
    std::thread::sleep(Duration::from_secs(5));

    println!("📖 Opening file...");
    let start = Instant::now();
    let mut reader = ExcelReader::open(test_file)?;
    println!("✅ File opened in {:.2}s", start.elapsed().as_secs_f64());

    println!("\n⏸️  Pausing 5 seconds - CHECK MEMORY USAGE NOW!");
    println!("   (This shows memory AFTER opening file)");
    std::thread::sleep(Duration::from_secs(5));

    println!("\n📖 Starting to read rows...");
    let start = Instant::now();
    let mut row_count = 0u64;
    let mut sum_values = 0u64;

    for row_result in reader.rows("Sheet1")? {
        let row = row_result?;

        // Process data
        if row.cells.len() > 15 {
            if let CellValue::String(val_str) = &row.cells[15] {
                if let Some(dot_pos) = val_str.find('.') {
                    if let Ok(num) = val_str[..dot_pos].parse::<u64>() {
                        sum_values += num;
                    }
                }
            }
        }

        row_count += 1;

        // Pause at checkpoints to observe memory
        if row_count == 100_000 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = row_count as f64 / elapsed;
            println!("\n  📊 Read {} rows ({:.0} rows/sec)", row_count, rate);
            println!("  ⏸️  Pausing 3 seconds - CHECK MEMORY NOW!");
            std::thread::sleep(Duration::from_secs(3));
        } else if row_count.is_multiple_of(200_000) {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = row_count as f64 / elapsed;
            println!("  📊 Read {} rows ({:.0} rows/sec)", row_count, rate);
        }
    }

    let read_duration = start.elapsed();
    let throughput = row_count as f64 / read_duration.as_secs_f64();

    println!("\n✅ Reading complete!");
    println!("   Total rows: {}", row_count);
    println!("   Duration: {:.2}s", read_duration.as_secs_f64());
    println!("   Throughput: {:.0} rows/sec", throughput);
    println!("   Avg value: {:.2}", sum_values as f64 / row_count as f64);

    println!("\n⏸️  Pausing 5 seconds - CHECK FINAL MEMORY!");
    std::thread::sleep(Duration::from_secs(5));

    println!("\n📊 Expected Memory Usage:");
    println!("   - Baseline: ~10-20 MB (program overhead)");
    println!(
        "   - After open: ~{:.0}-{:.0} MB (calamine loads entire file)",
        file_size_mb * 0.8,
        file_size_mb * 1.2
    );
    println!("   - During read: Same as after open (no additional allocation)");
    println!("   - After read: Same (data not released until drop)");

    println!("\n⚠️  IMPORTANT FINDINGS:");
    println!("   - calamine library loads ENTIRE file into memory");
    println!(
        "   - File size: {:.2} MB = Memory usage: ~{:.0} MB",
        file_size_mb, file_size_mb
    );
    println!("   - Iterator is over pre-loaded data (not streaming from disk)");
    println!("   - Memory is constant during iteration (good)");
    println!("   - But memory equals file size (bad for very large files)");

    println!("\n💡 Conclusion:");
    println!("   ExcelStream uses calamine which:");
    println!("   ✅ Provides iterator interface (memory-efficient iteration)");
    println!("   ❌ But loads entire file first (not disk streaming)");
    println!("   ✅ Suitable for files up to available RAM");
    println!("   ❌ Will OOM if file > RAM");

    Ok(())
}
