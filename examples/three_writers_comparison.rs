//! Comprehensive comparison of all 3 writer types
//! 
//! This example compares:
//! 1. ExcelWriter with write_row() - String-based (standard)
//! 2. ExcelWriter with write_row_typed() - Typed values
//! 3. FastWorkbook - Custom implementation (fastest)

use excelstream::writer::ExcelWriter;
use excelstream::fast_writer::FastWorkbook;
use excelstream::types::CellValue;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Three Writers Performance Comparison ===\n");
    println!("Test configuration:");
    println!("- Rows: 1,000,000");
    println!("- Columns: 30 (mixed data types)");
    println!("- Data types: String, Int, Float, Date, Email, URL, etc.\n");
    
    const NUM_ROWS: usize = 1_000_000;
    const NUM_COLS: usize = 30;
    
    // Test 1: ExcelWriter with write_row() - All strings
    println!("1. ExcelWriter.write_row() - String-based:");
    let start = Instant::now();
    test_write_row_strings("test_strings.xlsx", NUM_ROWS, NUM_COLS)?;
    let duration1 = start.elapsed();
    let speed1 = NUM_ROWS as f64 / duration1.as_secs_f64();
    println!("   Time: {:?}", duration1);
    println!("   Speed: {:.0} rows/sec\n", speed1);
    
    // Test 2: ExcelWriter with write_row_typed() - Typed values
    println!("2. ExcelWriter.write_row_typed() - Typed values:");
    let start = Instant::now();
    test_write_row_typed("test_typed.xlsx", NUM_ROWS, NUM_COLS)?;
    let duration2 = start.elapsed();
    let speed2 = NUM_ROWS as f64 / duration2.as_secs_f64();
    println!("   Time: {:?}", duration2);
    println!("   Speed: {:.0} rows/sec\n", speed2);
    
    // Test 3: FastWorkbook - Custom implementation
    println!("3. FastWorkbook - Custom XML + ZIP:");
    let start = Instant::now();
    test_fast_workbook("test_fast.xlsx", NUM_ROWS, NUM_COLS)?;
    let duration3 = start.elapsed();
    let speed3 = NUM_ROWS as f64 / duration3.as_secs_f64();
    println!("   Time: {:?}", duration3);
    println!("   Speed: {:.0} rows/sec\n", speed3);
    
    // Analysis
    println!("=== Performance Analysis ===");
    println!("write_row() strings:       {:.0} rows/sec (1.00x)", speed1);
    println!("write_row_typed():         {:.0} rows/sec ({:.2}x)", speed2, speed2 / speed1);
    println!("FastWorkbook:              {:.0} rows/sec ({:.2}x)", speed3, speed3 / speed1);
    println!();
    
    let diff2 = ((speed2 - speed1) / speed1 * 100.0).round();
    let diff3 = ((speed3 - speed1) / speed1 * 100.0).round();
    
    println!("=== Speed Comparison ===");
    println!("write_row_typed() vs write_row():");
    if diff2 > 0.0 {
        println!("  +{:.0}% faster", diff2);
    } else if diff2 < 0.0 {
        println!("  {:.0}% slower (type conversion overhead)", diff2.abs());
    } else {
        println!("  Same speed");
    }
    
    println!("\nFastWorkbook vs write_row():");
    if diff3 > 0.0 {
        println!("  +{:.0}% faster ⚡", diff3);
    } else {
        println!("  {:.0}% slower", diff3.abs());
    }
    println!();
    
    println!("=== Feature Comparison ===");
    println!("┌─────────────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ Feature             │ write_row()  │ typed()      │ FastWorkbook │");
    println!("├─────────────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ Speed               │ Base         │ {:<12} │ {:<12} │", 
             format!("{:+.0}%", diff2), format!("{:+.0}%", diff3));
    println!("│ Excel Formulas      │ ❌ No        │ ✅ Yes       │ ❌ No        │");
    println!("│ Number Types        │ ❌ Text      │ ✅ Number    │ ⚠️  Text     │");
    println!("│ Boolean Display     │ ❌ text      │ ✅ TRUE/FALSE│ ❌ text      │");
    println!("│ Multi-sheet         │ ✅ Yes       │ ✅ Yes       │ ✅ Yes       │");
    println!("│ Memory Efficient    │ ⚠️  Medium   │ ⚠️  Medium   │ ✅ High      │");
    println!("│ Large Datasets      │ ⚠️  OK       │ ⚠️  OK       │ ✅ Best      │");
    println!("└─────────────────────┴──────────────┴──────────────┴──────────────┘");
    println!();
    
    println!("=== Recommendations ===");
    println!("✅ Use write_row() when:");
    println!("   - Simple text export");
    println!("   - All data already strings");
    println!("   - Don't need Excel formulas");
    println!();
    println!("✅ Use write_row_typed() when:");
    println!("   - Need Excel formulas (SUM, AVERAGE, etc.)");
    println!("   - Want proper number/boolean types");
    println!("   - Users will do calculations in Excel");
    println!("   - Acceptable: ~{:.0}% performance trade-off", diff2.abs());
    println!();
    println!("✅ Use FastWorkbook when:");
    println!("   - Large datasets (100K+ rows)");
    println!("   - Performance is critical");
    println!("   - Memory-constrained environments");
    println!("   - Don't need Excel formulas");
    println!("   - Gain: +{:.0}% performance improvement ⚡", diff3);
    
    Ok(())
}

fn test_write_row_strings(filename: &str, num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;
    
    // Write header
    let headers = vec![
        "ID", "Name", "Email", "Age", "Salary", "Active", "Score", "Department",
        "Join_Date", "Phone", "Address", "City", "Country", "Postal_Code", "Website",
        "Tax_ID", "Credit_Limit", "Balance", "Last_Login", "Status", "Notes",
        "Created_At", "Updated_At", "Version", "Priority", "Category", "Tags",
        "Description", "Metadata", "Reference"
    ];
    writer.write_header(&headers[..num_cols])?;
    
    // Write data rows - all as strings
    for i in 1..=num_rows {
        let row = generate_mixed_row_strings(i, num_cols);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        writer.write_row(&row_refs)?;
    }
    
    writer.save()?;
    Ok(())
}

fn test_write_row_typed(filename: &str, num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;
    
    // Write header
    let headers = vec![
        "ID", "Name", "Email", "Age", "Salary", "Active", "Score", "Department",
        "Join_Date", "Phone", "Address", "City", "Country", "Postal_Code", "Website",
        "Tax_ID", "Credit_Limit", "Balance", "Last_Login", "Status", "Notes",
        "Created_At", "Updated_At", "Version", "Priority", "Category", "Tags",
        "Description", "Metadata", "Reference"
    ];
    writer.write_header(&headers[..num_cols])?;
    
    // Write data rows - with proper types
    for i in 1..=num_rows {
        let row = generate_mixed_row_typed(i, num_cols);
        writer.write_row_typed(&row[..num_cols])?;
    }
    
    writer.save()?;
    Ok(())
}

fn test_fast_workbook(filename: &str, num_rows: usize, num_cols: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;
    workbook.add_worksheet("Sheet1")?;
    
    // Write header
    let headers = vec![
        "ID", "Name", "Email", "Age", "Salary", "Active", "Score", "Department",
        "Join_Date", "Phone", "Address", "City", "Country", "Postal_Code", "Website",
        "Tax_ID", "Credit_Limit", "Balance", "Last_Login", "Status", "Notes",
        "Created_At", "Updated_At", "Version", "Priority", "Category", "Tags",
        "Description", "Metadata", "Reference"
    ];
    workbook.write_row(&headers[..num_cols])?;
    
    // Write data rows
    for i in 1..=num_rows {
        let row = generate_mixed_row_strings(i, num_cols);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        workbook.write_row(&row_refs)?;
    }
    
    workbook.close()?;
    Ok(())
}

/// Generate a row with all values as strings
fn generate_mixed_row_strings(row_num: usize, num_cols: usize) -> Vec<String> {
    let mut row = Vec::with_capacity(num_cols);
    
    for col in 0..num_cols {
        let value = match col {
            0 => format!("{}", row_num), // ID
            1 => format!("User_{}", row_num), // Name
            2 => format!("user{}@example.com", row_num), // Email
            3 => format!("{}", 20 + (row_num % 50)), // Age (as string)
            4 => format!("{:.2}", 30000.0 + (row_num as f64 * 123.45)), // Salary (as string)
            5 => if row_num % 2 == 0 { "true" } else { "false" }.to_string(), // Active (as string)
            6 => format!("{:.1}", 50.0 + (row_num % 50) as f64), // Score (as string)
            7 => match row_num % 5 {
                0 => "Engineering",
                1 => "Sales",
                2 => "Marketing",
                3 => "HR",
                _ => "Operations",
            }.to_string(),
            8 => format!("2024-{:02}-{:02}", 1 + (row_num % 12), 1 + (row_num % 28)),
            9 => format!("+1-555-{:04}-{:04}", row_num % 1000, row_num % 10000),
            10 => format!("{} Main Street", row_num),
            11 => match row_num % 10 {
                0 => "New York", 1 => "Los Angeles", 2 => "Chicago", 3 => "Houston",
                4 => "Phoenix", 5 => "Philadelphia", 6 => "San Antonio", 7 => "San Diego",
                8 => "Dallas", _ => "San Jose",
            }.to_string(),
            12 => "USA".to_string(),
            13 => format!("{:05}", 10000 + (row_num % 90000)),
            14 => format!("https://example{}.com", row_num),
            15 => format!("TAX-{:08}", row_num),
            16 => format!("{:.2}", 5000.0 + (row_num as f64 * 50.0)),
            17 => format!("{:.2}", (row_num as f64 * 12.34) % 10000.0),
            18 => format!("2024-12-{:02} {:02}:{:02}:{:02}", 
                         1 + (row_num % 28), row_num % 24, row_num % 60, row_num % 60),
            19 => match row_num % 4 {
                0 => "Active", 1 => "Pending", 2 => "Suspended", _ => "Inactive",
            }.to_string(),
            20 => format!("Note for record #{}", row_num),
            21 => format!("2024-01-01T{:02}:{:02}:{:02}Z", row_num % 24, row_num % 60, row_num % 60),
            22 => format!("2024-12-01T{:02}:{:02}:{:02}Z", row_num % 24, row_num % 60, row_num % 60),
            23 => format!("v{}.{}.{}", row_num % 10, row_num % 100, row_num % 1000),
            24 => match row_num % 3 {
                0 => "High", 1 => "Medium", _ => "Low",
            }.to_string(),
            25 => match row_num % 6 {
                0 => "Category A", 1 => "Category B", 2 => "Category C",
                3 => "Category D", 4 => "Category E", _ => "Category F",
            }.to_string(),
            26 => format!("tag1,tag{},tag{}", row_num % 10, row_num % 20),
            27 => format!("Description for record {} with some longer text to test performance", row_num),
            28 => format!("{{\"key\":\"{}\",\"value\":{}}}", row_num, row_num * 100),
            _ => format!("REF-{:08}", row_num),
        };
        row.push(value);
    }
    
    row
}

/// Generate a row with proper typed values
fn generate_mixed_row_typed(row_num: usize, num_cols: usize) -> Vec<CellValue> {
    let mut row = Vec::with_capacity(num_cols);
    
    for col in 0..num_cols {
        let value = match col {
            0 => CellValue::Int(row_num as i64), // ID as number
            1 => CellValue::String(format!("User_{}", row_num)), // Name
            2 => CellValue::String(format!("user{}@example.com", row_num)), // Email
            3 => CellValue::Int((20 + (row_num % 50)) as i64), // Age as number
            4 => CellValue::Float(30000.0 + (row_num as f64 * 123.45)), // Salary as float
            5 => CellValue::Bool(row_num % 2 == 0), // Active as boolean
            6 => CellValue::Float(50.0 + (row_num % 50) as f64), // Score as float
            7 => CellValue::String(match row_num % 5 {
                0 => "Engineering", 1 => "Sales", 2 => "Marketing", 3 => "HR", _ => "Operations",
            }.to_string()),
            8 => CellValue::String(format!("2024-{:02}-{:02}", 1 + (row_num % 12), 1 + (row_num % 28))),
            9 => CellValue::String(format!("+1-555-{:04}-{:04}", row_num % 1000, row_num % 10000)),
            10 => CellValue::String(format!("{} Main Street", row_num)),
            11 => CellValue::String(match row_num % 10 {
                0 => "New York", 1 => "Los Angeles", 2 => "Chicago", 3 => "Houston",
                4 => "Phoenix", 5 => "Philadelphia", 6 => "San Antonio", 7 => "San Diego",
                8 => "Dallas", _ => "San Jose",
            }.to_string()),
            12 => CellValue::String("USA".to_string()),
            13 => CellValue::Int((10000 + (row_num % 90000)) as i64),
            14 => CellValue::String(format!("https://example{}.com", row_num)),
            15 => CellValue::String(format!("TAX-{:08}", row_num)),
            16 => CellValue::Float(5000.0 + (row_num as f64 * 50.0)),
            17 => CellValue::Float((row_num as f64 * 12.34) % 10000.0),
            18 => CellValue::String(format!("2024-12-{:02} {:02}:{:02}:{:02}", 
                                           1 + (row_num % 28), row_num % 24, row_num % 60, row_num % 60)),
            19 => CellValue::String(match row_num % 4 {
                0 => "Active", 1 => "Pending", 2 => "Suspended", _ => "Inactive",
            }.to_string()),
            20 => CellValue::String(format!("Note for record #{}", row_num)),
            21 => CellValue::String(format!("2024-01-01T{:02}:{:02}:{:02}Z", row_num % 24, row_num % 60, row_num % 60)),
            22 => CellValue::String(format!("2024-12-01T{:02}:{:02}:{:02}Z", row_num % 24, row_num % 60, row_num % 60)),
            23 => CellValue::String(format!("v{}.{}.{}", row_num % 10, row_num % 100, row_num % 1000)),
            24 => CellValue::String(match row_num % 3 {
                0 => "High", 1 => "Medium", _ => "Low",
            }.to_string()),
            25 => CellValue::String(match row_num % 6 {
                0 => "Category A", 1 => "Category B", 2 => "Category C",
                3 => "Category D", 4 => "Category E", _ => "Category F",
            }.to_string()),
            26 => CellValue::String(format!("tag1,tag{},tag{}", row_num % 10, row_num % 20)),
            27 => CellValue::String(format!("Description for record {} with some longer text to test performance", row_num)),
            28 => CellValue::String(format!("{{\"key\":\"{}\",\"value\":{}}}", row_num, row_num * 100)),
            _ => CellValue::String(format!("REF-{:08}", row_num)),
        };
        row.push(value);
    }
    
    row
}
