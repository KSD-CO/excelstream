//! Demo of automatic memory configuration

use excelstream::fast_writer::{create_workbook_auto, create_workbook_with_profile, MemoryProfile};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Auto Memory Configuration Demo ===\n");
    
    // Demo 1: Auto detect from environment
    println!("1. Auto-detect from MEMORY_LIMIT_MB:");
    env::set_var("MEMORY_LIMIT_MB", "512");
    demo_auto_detect()?;
    
    // Demo 2: Manual profile
    println!("\n2. Manual profile - Low memory (256MB):");
    demo_manual_profile(MemoryProfile::Low)?;
    
    println!("\n3. Manual profile - Custom:");
    demo_manual_profile(MemoryProfile::Custom {
        flush_interval: 250,
        max_buffer_size: 384 * 1024,
    })?;
    
    println!("\n=== COMPLETED ===");
    
    Ok(())
}

fn demo_auto_detect() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically detect memory limit from env and config
    let mut workbook = create_workbook_auto("auto_detect.xlsx")?;
    
    workbook.add_worksheet("Data")?;
    
    // Write 10K rows
    workbook.write_row(&["ID", "Name", "Email"])?;
    for i in 1..=10_000 {
        let id = i.to_string();
        let name = format!("User{}", i);
        let email = format!("user{}@example.com", i);
        workbook.write_row(&[&id, &name, &email])?;
    }
    
    workbook.close()?;
    println!("   ✓ Written 10K rows with auto-config");
    
    Ok(())
}

fn demo_manual_profile(profile: MemoryProfile) -> Result<(), Box<dyn std::error::Error>> {
    // Specify a specific profile
    let mut workbook = create_workbook_with_profile(
        format!("profile_{:?}.xlsx", profile),
        profile,
    )?;
    
    workbook.add_worksheet("Data")?;
    
    // Write 10K rows
    workbook.write_row(&["ID", "Name", "Email"])?;
    for i in 1..=10_000 {
        let id = i.to_string();
        let name = format!("User{}", i);
        let email = format!("user{}@example.com", i);
        workbook.write_row(&[&id, &name, &email])?;
    }
    
    workbook.close()?;
    println!("   ✓ Written 10K rows with profile {:?}", profile);
    
    Ok(())
}
