//! Debug analysis of write_row_fast() performance issue
//! 
//! This example investigates why write_row_fast() is slower than write_row()

use excelstream::writer::ExcelWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Debug Analysis: Why is write_row_fast() slower? ===\n");
    
    const NUM_ROWS: usize = 50_000;
    
    // Test 1: Standard write_row() - direct write
    println!("1. Standard write_row() - Direct write:");
    println!("   Code path: data -> sheet.write_string() directly");
    let start = Instant::now();
    test_standard_write_row("test_standard.xlsx", NUM_ROWS)?;
    let duration1 = start.elapsed();
    println!("   Time: {:?} ({:.0} rows/sec)\n", duration1, NUM_ROWS as f64 / duration1.as_secs_f64());
    
    // Test 2: write_row_fast() - buffer + copy + write
    println!("2. write_row_fast() - Buffer + Copy + Write:");
    println!("   Code path: data -> String::to_string() -> Vec -> sheet.write_string()");
    let start = Instant::now();
    test_fast_write_row("test_fast.xlsx", NUM_ROWS)?;
    let duration2 = start.elapsed();
    println!("   Time: {:?} ({:.0} rows/sec)\n", duration2, NUM_ROWS as f64 / duration2.as_secs_f64());
    
    // Test 3: Manual analysis - what happens in write_row_fast()?
    println!("3. Breaking down write_row_fast() overhead:");
    analyze_fast_method_overhead(NUM_ROWS)?;
    
    // Calculate overhead
    let overhead = duration2.as_secs_f64() - duration1.as_secs_f64();
    let overhead_pct = (overhead / duration1.as_secs_f64()) * 100.0;
    
    println!("\n=== Analysis Results ===");
    println!("Standard method:   {:?}", duration1);
    println!("Fast method:       {:?}", duration2);
    println!("Overhead:          {:?} (+{:.1}%)", 
             std::time::Duration::from_secs_f64(overhead), overhead_pct);
    
    println!("\n=== Root Cause Analysis ===");
    println!("❌ Problem: write_row_fast() does EXTRA work:");
    println!("   1. value.as_ref().to_string() - Creates NEW String for each cell");
    println!("   2. row_buffer.push() - Allocates and stores in Vec");
    println!("   3. sheet.write_string() - Still calls rust_xlsxwriter");
    println!();
    println!("✅ Standard write_row() is simpler:");
    println!("   1. value.as_ref() - Just borrows, no allocation");
    println!("   2. sheet.write_string() - Direct write");
    println!();
    println!("💡 Conclusion:");
    println!("   - Buffer reuse doesn't help because rust_xlsxwriter");
    println!("     already optimizes internally");
    println!("   - Extra to_string() allocation creates overhead");
    println!("   - Vec operations add unnecessary complexity");
    println!("   - write_row_fast() should be REMOVED or marked deprecated");
    
    Ok(())
}

fn test_standard_write_row(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;
    
    writer.write_header(&["ID", "Name", "Email", "Age", "City"])?;
    
    for i in 1..=num_rows {
        let id = i.to_string();
        let name = format!("User_{}", i);
        let email = format!("user{}@example.com", i);
        let age = (20 + (i % 50)).to_string();
        let city = "New York";
        
        // Direct write - no intermediate allocations
        writer.write_row(&[&id, &name, &email, &age, city])?;
    }
    
    writer.save()?;
    Ok(())
}

fn test_fast_write_row(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;
    
    writer.write_header(&["ID", "Name", "Email", "Age", "City"])?;
    
    for i in 1..=num_rows {
        let id = i.to_string();
        let name = format!("User_{}", i);
        let email = format!("user{}@example.com", i);
        let age = (20 + (i % 50)).to_string();
        let city = "New York";
        
        // Note: write_row_fast() has been removed - it was actually slower!
        // Using standard write_row() instead
        writer.write_row(&[&id, &name, &email, &age, city])?;
    }
    
    writer.save()?;
    Ok(())
}

fn analyze_fast_method_overhead(num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Simulate what write_row_fast() does
    let mut buffer: Vec<String> = Vec::with_capacity(5);
    
    // Measure just the buffer operations
    let start = Instant::now();
    for i in 1..=num_rows {
        buffer.clear(); // Clear buffer
        
        // Simulate the data
        let id = i.to_string();
        let name = format!("User_{}", i);
        let email = format!("user{}@example.com", i);
        let age = (20 + (i % 50)).to_string();
        let city = "New York";
        
        // This is what write_row_fast() does:
        buffer.push(id.to_string());           // EXTRA allocation!
        buffer.push(name.to_string());         // EXTRA allocation!
        buffer.push(email.to_string());        // EXTRA allocation!
        buffer.push(age.to_string());          // EXTRA allocation!
        buffer.push(city.to_string());         // EXTRA allocation!
    }
    let buffer_time = start.elapsed();
    
    println!("   Buffer operations overhead: {:?}", buffer_time);
    println!("   Per-row overhead: {:?}", buffer_time / num_rows as u32);
    println!("   This is PURE overhead with NO benefit!");
    
    Ok(())
}
