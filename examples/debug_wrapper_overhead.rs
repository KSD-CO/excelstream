//! Debug example to understand why write_row() is slower than direct rust_xlsxwriter
//! 
//! This example measures the overhead at different levels:
//! 1. Direct rust_xlsxwriter with write_string()
//! 2. ExcelWriter.write_row() wrapper
//! 3. Break down the overhead sources

use excelstream::writer::ExcelWriter;
use rust_xlsxwriter::Workbook;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Analyzing write_row() Overhead ===\n");
    
    const NUM_ROWS: usize = 100_000;
    const NUM_COLS: usize = 10;
    
    // Test 1: Direct rust_xlsxwriter with write_string
    println!("1. Direct rust_xlsxwriter.write_string():");
    let start = Instant::now();
    test_direct_write_string(NUM_ROWS, NUM_COLS)?;
    let duration1 = start.elapsed();
    println!("   Time: {:?}", duration1);
    println!("   Per-row: {:.2}µs\n", duration1.as_micros() as f64 / NUM_ROWS as f64);
    
    // Test 2: ExcelWriter.write_row()
    println!("2. ExcelWriter.write_row():");
    let start = Instant::now();
    test_write_row(NUM_ROWS, NUM_COLS)?;
    let duration2 = start.elapsed();
    println!("   Time: {:?}", duration2);
    println!("   Per-row: {:.2}µs\n", duration2.as_micros() as f64 / NUM_ROWS as f64);
    
    // Test 3: Measure overhead sources
    println!("3. Measuring overhead sources:\n");
    
    // 3a. AsRef<str> conversion overhead
    let test_data: Vec<String> = (0..NUM_COLS).map(|i| format!("Value{}", i)).collect();
    let test_refs: Vec<&str> = test_data.iter().map(|s| s.as_str()).collect();
    
    let start = Instant::now();
    for _ in 0..NUM_ROWS {
        for value in &test_refs {
            let _: &str = value.as_ref(); // This is what write_row does
        }
    }
    let as_ref_overhead = start.elapsed();
    println!("   a) AsRef<str> conversion: {:?}", as_ref_overhead);
    println!("      Per-row: {:.2}µs", as_ref_overhead.as_micros() as f64 / NUM_ROWS as f64);
    
    // 3b. Iterator overhead
    let start = Instant::now();
    for _ in 0..NUM_ROWS {
        for (col, value) in test_refs.iter().enumerate() {
            let _col_idx = col as u16;
            let _val: &str = *value;
        }
    }
    let iterator_overhead = start.elapsed();
    println!("\n   b) Iterator + enumerate: {:?}", iterator_overhead);
    println!("      Per-row: {:.2}µs", iterator_overhead.as_micros() as f64 / NUM_ROWS as f64);
    
    // 3c. Method call overhead
    struct TestWrapper {
        counter: u32,
    }
    impl TestWrapper {
        fn get_mut(&mut self) -> &mut u32 {
            &mut self.counter
        }
    }
    
    let mut wrapper = TestWrapper { counter: 0 };
    let start = Instant::now();
    for _ in 0..NUM_ROWS {
        for _ in 0..NUM_COLS {
            let _sheet = wrapper.get_mut(); // Similar to current_sheet.as_mut().unwrap()
        }
    }
    let method_call_overhead = start.elapsed();
    println!("\n   c) Method calls (as_mut/unwrap): {:?}", method_call_overhead);
    println!("      Per-row: {:.2}µs", method_call_overhead.as_micros() as f64 / NUM_ROWS as f64);
    
    // Analysis
    println!("\n=== Analysis ===");
    let total_overhead = duration2.as_micros() as i64 - duration1.as_micros() as i64;
    let overhead_pct = (total_overhead as f64 / duration1.as_micros() as f64) * 100.0;
    
    println!("Direct rust_xlsxwriter:    {:?} (baseline)", duration1);
    println!("ExcelWriter.write_row():   {:?}", duration2);
    
    if total_overhead > 0 {
        println!("Total overhead:            {:.2}ms ({:.1}%)", total_overhead as f64 / 1000.0, overhead_pct);
    } else {
        println!("ExcelWriter is actually FASTER by: {:.2}ms ({:.1}%)", 
                 -total_overhead as f64 / 1000.0, -overhead_pct);
    }
    println!();
    
    if total_overhead > 0 {
        let estimated_overhead = as_ref_overhead.as_micros() + iterator_overhead.as_micros() + method_call_overhead.as_micros();
        println!("Estimated overhead breakdown:");
        println!("  AsRef conversion:  {:.2}ms ({:.1}%)", as_ref_overhead.as_micros() as f64 / 1000.0, 
                 (as_ref_overhead.as_micros() as f64 / total_overhead as f64) * 100.0);
        println!("  Iterator/enumerate: {:.2}ms ({:.1}%)", iterator_overhead.as_micros() as f64 / 1000.0,
                 (iterator_overhead.as_micros() as f64 / total_overhead as f64) * 100.0);
        println!("  Method calls:      {:.2}ms ({:.1}%)", method_call_overhead.as_micros() as f64 / 1000.0,
                 (method_call_overhead.as_micros() as f64 / total_overhead as f64) * 100.0);
        println!("  Unaccounted:       {:.2}ms ({:.1}%)", 
                 (total_overhead - estimated_overhead as i64) as f64 / 1000.0,
                 ((total_overhead - estimated_overhead as i64) as f64 / total_overhead as f64) * 100.0);
    }
    println!();
    
    println!("=== Explanation ===");
    println!("The write_row() wrapper is slower because:");
    println!("1. Generic iterator conversion (IntoIterator + AsRef<str>)");
    println!("2. enumerate() adds overhead vs direct indexing");
    println!("3. as_mut().unwrap() called on every cell write");
    println!("4. Extra function call indirection");
    println!();
    println!("💡 Why write_row_typed() is FASTER:");
    println!("   - write_number() is more efficient than write_string()");
    println!("   - rust_xlsxwriter has to parse strings to detect numbers");
    println!("   - Typed values skip the string parsing overhead");
    println!("   - The wrapper overhead is offset by parsing savings");
    
    Ok(())
}

fn test_direct_write_string(num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    for row in 0..num_rows {
        for col in 0..num_cols {
            worksheet.write_string(row as u32, col as u16, &format!("Value{}", col))?;
        }
    }
    
    workbook.save("debug_direct.xlsx")?;
    Ok(())
}

fn test_write_row(num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("debug_wrapper.xlsx")?;
    
    let row_data: Vec<String> = (0..num_cols).map(|i| format!("Value{}", i)).collect();
    
    for _ in 0..num_rows {
        let row_refs: Vec<&str> = row_data.iter().map(|s| s.as_str()).collect();
        writer.write_row(&row_refs)?;
    }
    
    writer.save()?;
    Ok(())
}
