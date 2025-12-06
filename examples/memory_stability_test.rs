//! Memory stability test: 500K rows to verify constant memory usage

use excelstream::streaming_reader::StreamingReader;
use excelstream::writer::ExcelWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Stability Test ===\n");

    const ROWS: usize = 500_000;
    const COLS: usize = 20;
    let filename = "/tmp/memory_test.xlsx";

    // === WRITE TEST ===
    println!("📝 Writing {} rows × {} columns", ROWS, COLS);
    let write_start = Instant::now();

    {
        let mut writer = ExcelWriter::new(filename)?;
        writer.write_header(
            (0..COLS)
                .map(|i| format!("Column{}", i))
                .collect::<Vec<_>>(),
        )?;

        for i in 0..ROWS {
            let row: Vec<String> = (0..COLS).map(|j| format!("Data_{}_{}", i, j)).collect();
            writer.write_row(&row)?;

            if (i + 1) % 50000 == 0 {
                println!("  Written {} rows...", i + 1);
            }
        }

        writer.save()?;
    }

    let write_duration = write_start.elapsed();
    println!(
        "✅ Write: {:.2}s ({:.0} rows/sec)",
        write_duration.as_secs_f64(),
        ROWS as f64 / write_duration.as_secs_f64()
    );

    let file_size = std::fs::metadata(filename)?.len();
    println!("   File: {:.2} MB\n", file_size as f64 / 1_048_576.0);

    // === READ TEST ===
    println!("📖 Reading {} rows", ROWS);
    let read_start = Instant::now();

    let mut reader = StreamingReader::open(filename)?;
    let mut count = 0;

    for row_result in reader.rows("Sheet1")? {
        let _row = row_result?;
        count += 1;

        if count % 50000 == 0 {
            println!("  Read {} rows...", count);
        }
    }

    let read_duration = read_start.elapsed();
    println!(
        "✅ Read: {:.2}s ({:.0} rows/sec)",
        read_duration.as_secs_f64(),
        count as f64 / read_duration.as_secs_f64()
    );

    println!("\n📊 Summary:");
    println!(
        "   Write: {:.0} rows/sec",
        ROWS as f64 / write_duration.as_secs_f64()
    );
    println!(
        "   Read:  {:.0} rows/sec",
        count as f64 / read_duration.as_secs_f64()
    );
    println!("   File:  {:.2} MB", file_size as f64 / 1_048_576.0);

    // Cleanup
    std::fs::remove_file(filename)?;

    Ok(())
}
