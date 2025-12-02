//! Performance-optimized writing example

use excelstream::ExcelWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Performance Comparison: Standard vs Optimized Writing\n");

    // Test 1: Standard write_row
    println!("Test 1: Standard write_row (100,000 rows)");
    let start = Instant::now();
    {
        let mut writer = ExcelWriter::new("examples/perf_standard.xlsx")?;
        writer.write_header(&["ID", "Name", "Email", "Score"])?;
        
        for i in 0..100_000 {
            writer.write_row(&[
                &i.to_string(),
                &format!("User_{}", i),
                &format!("user_{}@example.com", i),
                &(i as f64 * 0.75).to_string(),
            ])?;
        }
        
        writer.save()?;
    }
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Throughput: {:.0} rows/sec\n", 100_000.0 / duration.as_secs_f64());

    // Test 2: Optimized write_row_fast
    println!("Test 2: Optimized write_row_fast (100,000 rows)");
    let start = Instant::now();
    {
        let mut writer = ExcelWriter::new("examples/perf_fast.xlsx")?;
        writer.write_header(&["ID", "Name", "Email", "Score"])?;
        
        for i in 0..100_000 {
            writer.write_row_fast(&[
                &i.to_string(),
                &format!("User_{}", i),
                &format!("user_{}@example.com", i),
                &(i as f64 * 0.75).to_string(),
            ])?;
        }
        
        writer.save()?;
    }
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Throughput: {:.0} rows/sec\n", 100_000.0 / duration.as_secs_f64());

    // Test 3: Batch writing
    println!("Test 3: Batch write_rows_batch (100,000 rows)");
    let start = Instant::now();
    {
        let mut writer = ExcelWriter::new("examples/perf_batch.xlsx")?;
        writer.write_header(&["ID", "Name", "Email", "Score"])?;
        
        // Write in batches of 1000
        for batch_start in (0..100_000).step_by(1000) {
            let batch: Vec<Vec<String>> = (batch_start..batch_start + 1000)
                .map(|i| vec![
                    i.to_string(),
                    format!("User_{}", i),
                    format!("user_{}@example.com", i),
                    (i as f64 * 0.75).to_string(),
                ])
                .collect();
            
            writer.write_rows_batch(&batch)?;
        }
        
        writer.save()?;
    }
    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Throughput: {:.0} rows/sec\n", 100_000.0 / duration.as_secs_f64());

    println!("All test files created in examples/ directory");
    println!("\nConclusion:");
    println!("- The main bottleneck is in the underlying xlsx library");
    println!("- For best performance, minimize string allocations in your code");
    println!("- Consider batch processing for very large datasets");

    Ok(())
}
