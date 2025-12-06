//! Performance test: Read and Write with memory tracking
//! Tests streaming read/write with 100K rows to measure throughput and memory

use excelstream::streaming_reader::StreamingReader;
use excelstream::writer::ExcelWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Performance Test: Read & Write ===\n");

    const ROWS: usize = 100_000;
    const COLS: usize = 10;
    let filename = "/tmp/perf_test.xlsx";

    // === WRITE TEST ===
    println!("📝 Write Test: {} rows × {} columns", ROWS, COLS);
    let write_start = Instant::now();

    {
        let mut writer = ExcelWriter::new(filename)?;
        writer.write_header((0..COLS).map(|i| format!("Col{}", i)).collect::<Vec<_>>())?;

        for i in 0..ROWS {
            let row: Vec<String> = (0..COLS).map(|j| format!("R{}C{}", i, j)).collect();
            writer.write_row(&row)?;

            if (i + 1) % 10000 == 0 {
                print!("\r  Written {} rows...", i + 1);
            }
        }

        writer.save()?;
    }

    let write_duration = write_start.elapsed();
    let write_throughput = ROWS as f64 / write_duration.as_secs_f64();

    println!("\r✅ Write completed:");
    println!("   Time: {:.2}s", write_duration.as_secs_f64());
    println!("   Throughput: {:.0} rows/sec", write_throughput);

    // File size
    let file_size = std::fs::metadata(filename)?.len();
    println!("   File size: {:.2} MB", file_size as f64 / 1_048_576.0);

    println!();

    // === READ TEST ===
    println!("📖 Read Test: {} rows", ROWS);
    let read_start = Instant::now();

    let mut reader = StreamingReader::open(filename)?;
    let mut count = 0;

    for row_result in reader.rows("Sheet1")? {
        let _row = row_result?;
        count += 1;

        if count % 10000 == 0 {
            print!("\r  Read {} rows...", count);
        }
    }

    let read_duration = read_start.elapsed();
    let read_throughput = count as f64 / read_duration.as_secs_f64();

    println!("\r✅ Read completed:");
    println!("   Rows: {}", count);
    println!("   Time: {:.2}s", read_duration.as_secs_f64());
    println!("   Throughput: {:.0} rows/sec", read_throughput);

    println!("\n=== Summary ===");
    println!(
        "Write: {:.0} rows/sec ({:.2}s)",
        write_throughput,
        write_duration.as_secs_f64()
    );
    println!(
        "Read:  {:.0} rows/sec ({:.2}s)",
        read_throughput,
        read_duration.as_secs_f64()
    );
    println!("File:  {:.2} MB", file_size as f64 / 1_048_576.0);

    // Cleanup
    std::fs::remove_file(filename)?;

    Ok(())
}
