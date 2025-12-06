//! Benchmark write performance exactly like README claims

use excelstream::types::CellValue;
use excelstream::writer::ExcelWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Write Performance Benchmark ===\n");

    const ROWS: usize = 100_000;
    let filename = "/tmp/write_benchmark.xlsx";

    // Test 1: write_row() with strings
    println!("Test 1: write_row() with strings");
    let start = Instant::now();
    {
        let mut writer = ExcelWriter::new(filename)?;
        writer.write_header(["Col1", "Col2", "Col3", "Col4", "Col5"])?;

        for i in 0..ROWS {
            writer.write_row([
                &format!("String{}", i),
                &format!("Value{}", i),
                &format!("{}", i * 100),
                &format!("{:.2}", i as f64 * 1.5),
                &format!("Status{}", i % 10),
            ])?;
        }
        writer.save()?;
    }
    let duration = start.elapsed();
    println!(
        "✅ write_row(): {:.2}s ({:.0} rows/sec)\n",
        duration.as_secs_f64(),
        ROWS as f64 / duration.as_secs_f64()
    );

    // Test 2: write_row_typed()
    println!("Test 2: write_row_typed() with typed values");
    let start = Instant::now();
    {
        let mut writer = ExcelWriter::new(filename)?;
        writer.write_header(["Col1", "Col2", "Col3", "Col4", "Col5"])?;

        for i in 0..ROWS {
            writer.write_row_typed(&[
                CellValue::String(format!("String{}", i)),
                CellValue::String(format!("Value{}", i)),
                CellValue::Int(i as i64 * 100),
                CellValue::Float(i as f64 * 1.5),
                CellValue::String(format!("Status{}", i % 10)),
            ])?;
        }
        writer.save()?;
    }
    let duration = start.elapsed();
    println!(
        "✅ write_row_typed(): {:.2}s ({:.0} rows/sec)\n",
        duration.as_secs_f64(),
        ROWS as f64 / duration.as_secs_f64()
    );

    // Test 3: write_rows_batch()
    println!("Test 3: write_rows_batch() for comparison");
    let start = Instant::now();
    {
        let mut writer = ExcelWriter::new(filename)?;
        writer.write_header(["Col1", "Col2", "Col3", "Col4", "Col5"])?;

        let batch_size = 1000;
        for chunk_start in (0..ROWS).step_by(batch_size) {
            let chunk_end = (chunk_start + batch_size).min(ROWS);
            let batch: Vec<Vec<String>> = (chunk_start..chunk_end)
                .map(|i| {
                    vec![
                        format!("String{}", i),
                        format!("Value{}", i),
                        format!("{}", i * 100),
                        format!("{:.2}", i as f64 * 1.5),
                        format!("Status{}", i % 10),
                    ]
                })
                .collect();

            writer.write_rows_batch(&batch)?;
        }
        writer.save()?;
    }
    let duration = start.elapsed();
    println!(
        "✅ write_rows_batch(): {:.2}s ({:.0} rows/sec)\n",
        duration.as_secs_f64(),
        ROWS as f64 / duration.as_secs_f64()
    );

    println!("=== Benchmark Complete ===");

    // Cleanup
    std::fs::remove_file(filename)?;

    Ok(())
}
