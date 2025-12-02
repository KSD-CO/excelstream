//! Memory usage comparison between ExcelWriter and FastWorkbook
//! 
//! This example demonstrates the difference in memory usage between:
//! 1. ExcelWriter (fake streaming - keeps everything in memory)
//! 2. FastWorkbook (true streaming - writes to disk immediately)

use excelstream::writer::ExcelWriter;
use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Usage Comparison ===\n");
    println!("This test writes 100K rows × 10 columns");
    println!("Monitor memory usage with: ps aux | grep memory_usage_test\n");
    
    const NUM_ROWS: usize = 100_000;
    const NUM_COLS: usize = 10;
    
    println!("Press Enter to start Test 1: ExcelWriter (fake streaming)...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    println!("\n=== Test 1: ExcelWriter ===");
    println!("Expected: Memory usage INCREASES as we write more rows");
    println!("Because: All data kept in rust_xlsxwriter Worksheet in memory\n");
    
    let start = Instant::now();
    test_excel_writer(NUM_ROWS, NUM_COLS)?;
    let duration = start.elapsed();
    
    println!("✅ Completed in {:?}", duration);
    println!("💡 Check memory usage during execution - it should be HIGH\n");
    
    println!("Press Enter to start Test 2: FastWorkbook (true streaming)...");
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    
    println!("\n=== Test 2: FastWorkbook ===");
    println!("Expected: Memory usage CONSTANT (low) throughout");
    println!("Because: Data written directly to disk, flushed every 1000 rows\n");
    
    let start = Instant::now();
    test_fast_workbook(NUM_ROWS, NUM_COLS)?;
    let duration = start.elapsed();
    
    println!("✅ Completed in {:?}", duration);
    println!("💡 Check memory usage during execution - it should be LOW and CONSTANT\n");
    
    println!("=== Analysis ===");
    println!("\nExcelWriter behavior:");
    println!("  1. write_row() → stores in rust_xlsxwriter Worksheet");
    println!("  2. Memory grows with each row");
    println!("  3. save() → rust_xlsxwriter builds XML and writes file");
    println!("  4. ❌ NOT true streaming!");
    println!("\nFastWorkbook behavior:");
    println!("  1. write_row() → builds XML in small buffer");
    println!("  2. Immediately writes to ZIP file");
    println!("  3. Flushes every 1000 rows to disk");
    println!("  4. Memory stays constant");
    println!("  5. ✅ TRUE streaming!");
    
    Ok(())
}

fn test_excel_writer(num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("memory_test_excel_writer.xlsx")?;
    
    // Write header
    let header: Vec<String> = (0..num_cols).map(|i| format!("Column{}", i)).collect();
    let header_refs: Vec<&str> = header.iter().map(|s| s.as_str()).collect();
    writer.write_header(&header_refs)?;
    
    println!("Writing rows...");
    let checkpoint_interval = num_rows / 10;
    
    for i in 1..=num_rows {
        // Generate row data
        let row: Vec<String> = (0..num_cols)
            .map(|col| format!("Row{}_Col{}_Data", i, col))
            .collect();
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        
        writer.write_row(&row_refs)?;
        
        // Progress checkpoints
        if i % checkpoint_interval == 0 {
            println!("  {} rows written - Check memory now!", i);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    
    println!("All rows written. Now saving file...");
    println!("  (rust_xlsxwriter will now build entire workbook and write to disk)");
    writer.save()?;
    println!("  File saved!");
    
    Ok(())
}

fn test_fast_workbook(num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new("memory_test_fast_workbook.xlsx")?;
    workbook.add_worksheet("Sheet1")?;
    
    // Write header
    let header: Vec<String> = (0..num_cols).map(|i| format!("Column{}", i)).collect();
    let header_refs: Vec<&str> = header.iter().map(|s| s.as_str()).collect();
    workbook.write_row(&header_refs)?;
    
    println!("Writing rows (streaming to disk)...");
    let checkpoint_interval = num_rows / 10;
    
    for i in 1..=num_rows {
        // Generate row data
        let row: Vec<String> = (0..num_cols)
            .map(|col| format!("Row{}_Col{}_Data", i, col))
            .collect();
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        
        workbook.write_row(&row_refs)?;  // ← Data written to ZIP immediately!
        
        // Progress checkpoints
        if i % checkpoint_interval == 0 {
            println!("  {} rows written - Check memory now! (should be constant)", i);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    
    println!("All rows written. Now closing workbook...");
    println!("  (Just finalizing ZIP file, data already on disk)");
    workbook.close()?;
    println!("  File closed!");
    
    Ok(())
}
