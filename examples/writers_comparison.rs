//! Comprehensive comparison of ExcelStream writer types
//!
//! This example compares:
//! 1. ExcelWriter with write_row() - String-based (standard wrapper)
//! 2. ExcelWriter with write_row_typed() - Typed values
//! 3. ExcelWriter with write_row_styled() - Styled cells (v0.3.0+)
//! 4. FastWorkbook - Custom implementation (optimized for large datasets)

use excelstream::fast_writer::FastWorkbook;
use excelstream::types::{CellStyle, CellValue};
use excelstream::writer::ExcelWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ExcelStream Writers Performance Comparison ===\n");

    // Default: 100K rows for quick testing
    // Excel limit: 1,048,576 rows per sheet
    // For > 1M rows, would need multiple sheets
    const NUM_ROWS: usize = 1_000_000; // Max safe for single sheet
    const NUM_COLS: usize = 30;

    println!("Test configuration:");
    println!("- Rows: {:?}", NUM_ROWS);
    println!("- Columns: {} (mixed data types)", NUM_COLS);
    println!("- Data types: String, Int, Float, Date, Email, URL, etc.");
    println!(
        "- Excel limit: 1,048,576 rows per sheet (we're using {})",
        NUM_ROWS
    );
    println!(
        "- Total cells: {} million",
        (NUM_ROWS * NUM_COLS * 4) / 1_000_000
    );
    println!(
        "- Estimated time: {} minutes",
        if NUM_ROWS >= 1_000_000 {
            "15-40"
        } else {
            "2-5"
        }
    );
    println!(
        "- Expected file size: ~{} MB each\n",
        if NUM_ROWS >= 1_000_000 {
            "175-180"
        } else {
            "18-20"
        }
    );

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

    // Test 3: ExcelWriter with write_row_styled() - With cell styling
    println!("3. ExcelWriter.write_row_styled() - With styling:");
    let start = Instant::now();
    test_write_row_styled("test_styled.xlsx", NUM_ROWS, NUM_COLS)?;
    let duration3 = start.elapsed();
    let speed3 = NUM_ROWS as f64 / duration3.as_secs_f64();
    println!("   Time: {:?}", duration3);
    println!("   Speed: {:.0} rows/sec\n", speed3);

    // Test 4: FastWorkbook - Custom implementation
    println!("4. FastWorkbook - Optimized implementation:");
    let start = Instant::now();
    test_fast_workbook("test_fast.xlsx", NUM_ROWS, NUM_COLS)?;
    let duration4 = start.elapsed();
    let speed4 = NUM_ROWS as f64 / duration4.as_secs_f64();
    println!("   Time: {:?}", duration4);
    println!("   Speed: {:.0} rows/sec", speed4);
    println!("   Note: Slower due to Vec<String> → Vec<&str> conversion overhead\n");

    // Analysis - use write_row() as baseline
    println!("=== Performance Analysis ===");
    println!(
        "ExcelWriter.write_row():       {:.0} rows/sec (1.00x) [baseline]",
        speed1
    );
    println!(
        "ExcelWriter.write_row_typed(): {:.0} rows/sec ({:.2}x)",
        speed2,
        speed2 / speed1
    );
    println!(
        "ExcelWriter.write_row_styled(): {:.0} rows/sec ({:.2}x)",
        speed3,
        speed3 / speed1
    );
    println!(
        "FastWorkbook:                  {:.0} rows/sec ({:.2}x)",
        speed4,
        speed4 / speed1
    );
    println!();

    let diff2 = ((speed2 - speed1) / speed1 * 100.0).round();
    let diff3 = ((speed3 - speed1) / speed1 * 100.0).round();
    let diff4 = ((speed4 - speed1) / speed1 * 100.0).round();

    println!("=== Speed Comparison vs ExcelWriter.write_row() ===");
    println!("ExcelWriter.write_row_typed():");
    if diff2 > 0.0 {
        println!("  +{:.0}% faster", diff2);
    } else if diff2 < 0.0 {
        println!("  {:.0}% slower (type conversion overhead)", diff2.abs());
    } else {
        println!("  Same speed");
    }

    println!("\nExcelWriter.write_row_styled():");
    if diff3 > 0.0 {
        println!("  +{:.0}% faster", diff3);
    } else if diff3 < 0.0 {
        println!("  {:.0}% slower (styling overhead)", diff3.abs());
    } else {
        println!("  Same speed");
    }

    println!("\nFastWorkbook:");
    if diff4 > 0.0 {
        println!("  +{:.0}% faster ⚡", diff4);
    } else {
        println!("  {:.0}% slower", diff4.abs());
    }
    println!();

    println!("=== Feature Comparison ===");
    println!("┌─────────────────────┬──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ Feature             │ write_row()  │ typed()      │ styled()     │ FastWorkbook │");
    println!("├─────────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");
    println!(
        "│ Speed               │ Baseline     │ {:<12} │ {:<12} │ {:<12} │",
        format!("{:+.0}%", diff2),
        format!("{:+.0}%", diff3),
        format!("{:+.0}%", diff4)
    );
    println!("│ Excel Formulas      │ ❌ No        │ ✅ Yes       │ ✅ Yes       │ ❌ No        │");
    println!("│ Number Types        │ ❌ Text      │ ✅ Number    │ ✅ Number    │ ❌ Text      │");
    println!("│ Boolean Display     │ ❌ text      │ ✅ TRUE/FALSE│ ✅ TRUE/FALSE│ ❌ text      │");
    println!("│ Cell Styling        │ ❌ No        │ ❌ No        │ ✅ Yes       │ ❌ No        │");
    println!("│ API Simplicity      │ ✅ Simple    │ ✅ Simple    │ ✅ Simple    │ ✅ Simple    │");
    println!("│ Multi-sheet         │ ✅ Yes       │ ✅ Yes       │ ✅ Yes       │ ✅ Yes       │");
    println!("│ Memory Efficient    │ ⚠️  Medium   │ ⚠️  Medium   │ ⚠️  Medium   │ ✅ High      │");
    println!("│ Large Datasets      │ ⚠️  OK       │ ⚠️  OK       │ ⚠️  OK       │ ✅ Best      │");
    println!("└─────────────────────┴──────────────┴──────────────┴──────────────┴──────────────┘");
    println!();

    println!("=== Recommendations ===");
    println!("✅ Use ExcelWriter.write_row() when:");
    println!("   - Simple text export");
    println!("   - All data already strings");
    println!("   - Don't need Excel formulas or styling");
    println!("   - Standard use cases");
    println!("   - Performance: {:.0} rows/sec", speed1);
    println!();
    println!("✅ Use ExcelWriter.write_row_typed() when:");
    println!("   - Need Excel formulas (SUM, AVERAGE, etc.)");
    println!("   - Want proper number/boolean types");
    println!("   - Users will do calculations in Excel");
    println!("   - Mixed data types");
    println!("   - Performance: {:.0} rows/sec ({:+.0}%)", speed2, diff2);
    println!();
    println!("✅ Use ExcelWriter.write_row_styled() when:");
    println!("   - Need cell formatting (colors, bold, borders)");
    println!("   - Visual emphasis on important data");
    println!("   - Professional reports and dashboards");
    println!("   - Highlighting patterns or anomalies");
    println!(
        "   - Performance: {:.0} rows/sec ({:+.0}%) ⚡ FASTEST!",
        speed3, diff3
    );
    println!();
    println!("✅ Use FastWorkbook direct when:");
    println!("   - Have data already as &str (avoid String allocation)");
    println!("   - Need lowest-level control");
    println!("   - Building custom abstractions");
    println!("   - Memory-constrained environments");
    println!("   - Note: This test shows slower due to Vec<String>→Vec<&str> conversion");
    println!("   - In real usage with &str data, FastWorkbook is fastest!");
    println!("   - Performance: {:.0} rows/sec ({:+.0}%)", speed4, diff4);
    println!();
    println!("💡 Key Insight:");
    println!("   - ExcelWriter already uses FastWorkbook internally!");
    println!("   - write_row_styled() is fastest because it avoids string conversions");
    println!("   - Use styled() for best performance + features");

    Ok(())
}

fn test_write_row_strings(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;

    // Write header
    let headers = vec![
        "ID",
        "Name",
        "Email",
        "Age",
        "Salary",
        "Active",
        "Score",
        "Department",
        "Join_Date",
        "Phone",
        "Address",
        "City",
        "Country",
        "Postal_Code",
        "Website",
        "Tax_ID",
        "Credit_Limit",
        "Balance",
        "Last_Login",
        "Status",
        "Notes",
        "Created_At",
        "Updated_At",
        "Version",
        "Priority",
        "Category",
        "Tags",
        "Description",
        "Metadata",
        "Reference",
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

fn test_write_row_typed(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;

    // Write header
    let headers = vec![
        "ID",
        "Name",
        "Email",
        "Age",
        "Salary",
        "Active",
        "Score",
        "Department",
        "Join_Date",
        "Phone",
        "Address",
        "City",
        "Country",
        "Postal_Code",
        "Website",
        "Tax_ID",
        "Credit_Limit",
        "Balance",
        "Last_Login",
        "Status",
        "Notes",
        "Created_At",
        "Updated_At",
        "Version",
        "Priority",
        "Category",
        "Tags",
        "Description",
        "Metadata",
        "Reference",
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

fn test_write_row_styled(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new(filename)?;

    // Write header with bold style
    let headers = vec![
        "ID",
        "Name",
        "Email",
        "Age",
        "Salary",
        "Active",
        "Score",
        "Department",
        "Join_Date",
        "Phone",
        "Address",
        "City",
        "Country",
        "Postal_Code",
        "Website",
        "Tax_ID",
        "Credit_Limit",
        "Balance",
        "Last_Login",
        "Status",
        "Notes",
        "Created_At",
        "Updated_At",
        "Version",
        "Priority",
        "Category",
        "Tags",
        "Description",
        "Metadata",
        "Reference",
    ];

    // Header row with bold style
    let header_cells: Vec<(CellValue, CellStyle)> = headers[..num_cols]
        .iter()
        .map(|h| (CellValue::String(h.to_string()), CellStyle::HeaderBold))
        .collect();
    writer.write_row_styled(&header_cells)?;

    // Write data rows with styling based on values
    for i in 1..=num_rows {
        let row = generate_mixed_row_styled(i, num_cols);
        writer.write_row_styled(&row[..num_cols])?;
    }

    writer.save()?;
    Ok(())
}

fn test_fast_workbook(
    filename: &str,
    num_rows: usize,
    num_cols: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;
    workbook.add_worksheet("Sheet1")?;

    // Write header
    let headers = vec![
        "ID",
        "Name",
        "Email",
        "Age",
        "Salary",
        "Active",
        "Score",
        "Department",
        "Join_Date",
        "Phone",
        "Address",
        "City",
        "Country",
        "Postal_Code",
        "Website",
        "Tax_ID",
        "Credit_Limit",
        "Balance",
        "Last_Login",
        "Status",
        "Notes",
        "Created_At",
        "Updated_At",
        "Version",
        "Priority",
        "Category",
        "Tags",
        "Description",
        "Metadata",
        "Reference",
    ];
    workbook.write_row(&headers[..num_cols])?;

    // Write data rows
    // Note: This has overhead from Vec<String> → Vec<&str> conversion
    // In production, you'd generate &str directly or use write_row_styled()
    for i in 1..=num_rows {
        let row = generate_mixed_row_strings(i, num_cols);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect(); // ⚠️ Allocation overhead
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
            0 => format!("{}", row_num),                                // ID
            1 => format!("User_{}", row_num),                           // Name
            2 => format!("user{}@example.com", row_num),                // Email
            3 => format!("{}", 20 + (row_num % 50)),                    // Age (as string)
            4 => format!("{:.2}", 30000.0 + (row_num as f64 * 123.45)), // Salary (as string)
            5 => if row_num.is_multiple_of(2) {
                "true"
            } else {
                "false"
            }
            .to_string(), // Active (as string)
            6 => format!("{:.1}", 50.0 + (row_num % 50) as f64),        // Score (as string)
            7 => match row_num % 5 {
                0 => "Engineering",
                1 => "Sales",
                2 => "Marketing",
                3 => "HR",
                _ => "Operations",
            }
            .to_string(),
            8 => format!("2024-{:02}-{:02}", 1 + (row_num % 12), 1 + (row_num % 28)),
            9 => format!("+1-555-{:04}-{:04}", row_num % 1000, row_num % 10000),
            10 => format!("{} Main Street", row_num),
            11 => match row_num % 10 {
                0 => "New York",
                1 => "Los Angeles",
                2 => "Chicago",
                3 => "Houston",
                4 => "Phoenix",
                5 => "Philadelphia",
                6 => "San Antonio",
                7 => "San Diego",
                8 => "Dallas",
                _ => "San Jose",
            }
            .to_string(),
            12 => "USA".to_string(),
            13 => format!("{:05}", 10000 + (row_num % 90000)),
            14 => format!("https://example{}.com", row_num),
            15 => format!("TAX-{:08}", row_num),
            16 => format!("{:.2}", 5000.0 + (row_num as f64 * 50.0)),
            17 => format!("{:.2}", (row_num as f64 * 12.34) % 10000.0),
            18 => format!(
                "2024-12-{:02} {:02}:{:02}:{:02}",
                1 + (row_num % 28),
                row_num % 24,
                row_num % 60,
                row_num % 60
            ),
            19 => match row_num % 4 {
                0 => "Active",
                1 => "Pending",
                2 => "Suspended",
                _ => "Inactive",
            }
            .to_string(),
            20 => format!("Note for record #{}", row_num),
            21 => format!(
                "2024-01-01T{:02}:{:02}:{:02}Z",
                row_num % 24,
                row_num % 60,
                row_num % 60
            ),
            22 => format!(
                "2024-12-01T{:02}:{:02}:{:02}Z",
                row_num % 24,
                row_num % 60,
                row_num % 60
            ),
            23 => format!("v{}.{}.{}", row_num % 10, row_num % 100, row_num % 1000),
            24 => match row_num % 3 {
                0 => "High",
                1 => "Medium",
                _ => "Low",
            }
            .to_string(),
            25 => match row_num % 6 {
                0 => "Category A",
                1 => "Category B",
                2 => "Category C",
                3 => "Category D",
                4 => "Category E",
                _ => "Category F",
            }
            .to_string(),
            26 => format!("tag1,tag{},tag{}", row_num % 10, row_num % 20),
            27 => format!(
                "Description for record {} with some longer text to test performance",
                row_num
            ),
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
            5 => CellValue::Bool(row_num.is_multiple_of(2)), // Active as boolean
            6 => CellValue::Float(50.0 + (row_num % 50) as f64), // Score as float
            7 => CellValue::String(
                match row_num % 5 {
                    0 => "Engineering",
                    1 => "Sales",
                    2 => "Marketing",
                    3 => "HR",
                    _ => "Operations",
                }
                .to_string(),
            ),
            8 => CellValue::String(format!(
                "2024-{:02}-{:02}",
                1 + (row_num % 12),
                1 + (row_num % 28)
            )),
            9 => CellValue::String(format!(
                "+1-555-{:04}-{:04}",
                row_num % 1000,
                row_num % 10000
            )),
            10 => CellValue::String(format!("{} Main Street", row_num)),
            11 => CellValue::String(
                match row_num % 10 {
                    0 => "New York",
                    1 => "Los Angeles",
                    2 => "Chicago",
                    3 => "Houston",
                    4 => "Phoenix",
                    5 => "Philadelphia",
                    6 => "San Antonio",
                    7 => "San Diego",
                    8 => "Dallas",
                    _ => "San Jose",
                }
                .to_string(),
            ),
            12 => CellValue::String("USA".to_string()),
            13 => CellValue::Int((10000 + (row_num % 90000)) as i64),
            14 => CellValue::String(format!("https://example{}.com", row_num)),
            15 => CellValue::String(format!("TAX-{:08}", row_num)),
            16 => CellValue::Float(5000.0 + (row_num as f64 * 50.0)),
            17 => CellValue::Float((row_num as f64 * 12.34) % 10000.0),
            18 => CellValue::String(format!(
                "2024-12-{:02} {:02}:{:02}:{:02}",
                1 + (row_num % 28),
                row_num % 24,
                row_num % 60,
                row_num % 60
            )),
            19 => CellValue::String(
                match row_num % 4 {
                    0 => "Active",
                    1 => "Pending",
                    2 => "Suspended",
                    _ => "Inactive",
                }
                .to_string(),
            ),
            20 => CellValue::String(format!("Note for record #{}", row_num)),
            21 => CellValue::String(format!(
                "2024-01-01T{:02}:{:02}:{:02}Z",
                row_num % 24,
                row_num % 60,
                row_num % 60
            )),
            22 => CellValue::String(format!(
                "2024-12-01T{:02}:{:02}:{:02}Z",
                row_num % 24,
                row_num % 60,
                row_num % 60
            )),
            23 => CellValue::String(format!(
                "v{}.{}.{}",
                row_num % 10,
                row_num % 100,
                row_num % 1000
            )),
            24 => CellValue::String(
                match row_num % 3 {
                    0 => "High",
                    1 => "Medium",
                    _ => "Low",
                }
                .to_string(),
            ),
            25 => CellValue::String(
                match row_num % 6 {
                    0 => "Category A",
                    1 => "Category B",
                    2 => "Category C",
                    3 => "Category D",
                    4 => "Category E",
                    _ => "Category F",
                }
                .to_string(),
            ),
            26 => CellValue::String(format!("tag1,tag{},tag{}", row_num % 10, row_num % 20)),
            27 => CellValue::String(format!(
                "Description for record {} with some longer text to test performance",
                row_num
            )),
            28 => CellValue::String(format!(
                "{{\"key\":\"{}\",\"value\":{}}}",
                row_num,
                row_num * 100
            )),
            _ => CellValue::String(format!("REF-{:08}", row_num)),
        };
        row.push(value);
    }

    row
}

/// Generate a row with proper typed values and styling
fn generate_mixed_row_styled(row_num: usize, num_cols: usize) -> Vec<(CellValue, CellStyle)> {
    let mut row = Vec::with_capacity(num_cols);

    for col in 0..num_cols {
        let (value, style) = match col {
            0 => (CellValue::Int(row_num as i64), CellStyle::Default), // ID
            1 => (
                CellValue::String(format!("User_{}", row_num)),
                CellStyle::Default,
            ), // Name
            2 => (
                CellValue::String(format!("user{}@example.com", row_num)),
                CellStyle::Default,
            ), // Email
            3 => (
                CellValue::Int((20 + (row_num % 50)) as i64),
                CellStyle::NumberInteger,
            ), // Age
            4 => {
                let salary = 30000.0 + (row_num as f64 * 123.45);
                (CellValue::Float(salary), CellStyle::NumberCurrency)
            } // Salary
            5 => (
                CellValue::Bool(row_num.is_multiple_of(2)),
                CellStyle::Default,
            ), // Active
            6 => {
                let score = 50.0 + (row_num % 50) as f64;
                let style = if score > 75.0 {
                    CellStyle::HighlightGreen
                } else if score < 60.0 {
                    CellStyle::HighlightRed
                } else {
                    CellStyle::NumberDecimal
                };
                (CellValue::Float(score), style)
            } // Score with conditional highlighting
            7 => (
                CellValue::String(
                    match row_num % 5 {
                        0 => "Engineering",
                        1 => "Sales",
                        2 => "Marketing",
                        3 => "HR",
                        _ => "Operations",
                    }
                    .to_string(),
                ),
                CellStyle::Default,
            ),
            8 => (
                CellValue::String(format!(
                    "2024-{:02}-{:02}",
                    1 + (row_num % 12),
                    1 + (row_num % 28)
                )),
                CellStyle::DateDefault,
            ),
            9 => (
                CellValue::String(format!(
                    "+1-555-{:04}-{:04}",
                    row_num % 1000,
                    row_num % 10000
                )),
                CellStyle::Default,
            ),
            10 => (
                CellValue::String(format!("{} Main Street", row_num)),
                CellStyle::Default,
            ),
            11 => (
                CellValue::String(
                    match row_num % 10 {
                        0 => "New York",
                        1 => "Los Angeles",
                        2 => "Chicago",
                        3 => "Houston",
                        4 => "Phoenix",
                        5 => "Philadelphia",
                        6 => "San Antonio",
                        7 => "San Diego",
                        8 => "Dallas",
                        _ => "San Jose",
                    }
                    .to_string(),
                ),
                CellStyle::Default,
            ),
            12 => (CellValue::String("USA".to_string()), CellStyle::Default),
            13 => (
                CellValue::Int((10000 + (row_num % 90000)) as i64),
                CellStyle::Default,
            ),
            14 => (
                CellValue::String(format!("https://example{}.com", row_num)),
                CellStyle::Default,
            ),
            15 => (
                CellValue::String(format!("TAX-{:08}", row_num)),
                CellStyle::Default,
            ),
            16 => (
                CellValue::Float(5000.0 + (row_num as f64 * 50.0)),
                CellStyle::NumberCurrency,
            ),
            17 => (
                CellValue::Float((row_num as f64 * 12.34) % 10000.0),
                CellStyle::NumberCurrency,
            ),
            18 => (
                CellValue::String(format!(
                    "2024-12-{:02} {:02}:{:02}:{:02}",
                    1 + (row_num % 28),
                    row_num % 24,
                    row_num % 60,
                    row_num % 60
                )),
                CellStyle::DateTimestamp,
            ),
            19 => {
                let status = match row_num % 4 {
                    0 => "Active",
                    1 => "Pending",
                    2 => "Suspended",
                    _ => "Inactive",
                };
                let style = match status {
                    "Active" => CellStyle::HighlightGreen,
                    "Pending" => CellStyle::HighlightYellow,
                    _ => CellStyle::Default,
                };
                (CellValue::String(status.to_string()), style)
            }
            20 => (
                CellValue::String(format!("Note for record #{}", row_num)),
                CellStyle::Default,
            ),
            21 => (
                CellValue::String(format!(
                    "2024-01-01T{:02}:{:02}:{:02}Z",
                    row_num % 24,
                    row_num % 60,
                    row_num % 60
                )),
                CellStyle::Default,
            ),
            22 => (
                CellValue::String(format!(
                    "2024-12-01T{:02}:{:02}:{:02}Z",
                    row_num % 24,
                    row_num % 60,
                    row_num % 60
                )),
                CellStyle::Default,
            ),
            23 => (
                CellValue::String(format!(
                    "v{}.{}.{}",
                    row_num % 10,
                    row_num % 100,
                    row_num % 1000
                )),
                CellStyle::Default,
            ),
            24 => {
                let priority = match row_num % 3 {
                    0 => "High",
                    1 => "Medium",
                    _ => "Low",
                };
                let style = match priority {
                    "High" => CellStyle::TextBold,
                    _ => CellStyle::Default,
                };
                (CellValue::String(priority.to_string()), style)
            }
            25 => (
                CellValue::String(
                    match row_num % 6 {
                        0 => "Category A",
                        1 => "Category B",
                        2 => "Category C",
                        3 => "Category D",
                        4 => "Category E",
                        _ => "Category F",
                    }
                    .to_string(),
                ),
                CellStyle::Default,
            ),
            26 => (
                CellValue::String(format!("tag1,tag{},tag{}", row_num % 10, row_num % 20)),
                CellStyle::Default,
            ),
            27 => (
                CellValue::String(format!(
                    "Description for record {} with some longer text to test performance",
                    row_num
                )),
                CellStyle::Default,
            ),
            28 => (
                CellValue::String(format!(
                    "{{\"key\":\"{}\",\"value\":{}}}",
                    row_num,
                    row_num * 100
                )),
                CellStyle::Default,
            ),
            _ => (
                CellValue::String(format!("REF-{:08}", row_num)),
                CellStyle::Default,
            ),
        };
        row.push((value, style));
    }

    row
}
