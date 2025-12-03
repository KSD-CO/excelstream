//! Large dataset example with automatic multi-sheet splitting
//!
//! Excel has a limit of 1,048,576 rows per sheet.
//! This example automatically splits large datasets across multiple sheets.
//!
//! For 10M rows:
//! - Sheet1: rows 1-1,000,000
//! - Sheet2: rows 1,000,001-2,000,000
//! - Sheet3: rows 2,000,001-3,000,000
//! - ... and so on

use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

const EXCEL_MAX_ROWS: usize = 1_048_576; // Excel's hard limit
const ROWS_PER_SHEET: usize = 1_000_000; // Safe limit with header

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Large Dataset Multi-Sheet Test ===\n");

    const TOTAL_ROWS: usize = 10_000_000;
    const NUM_COLS: usize = 30;

    let num_sheets = TOTAL_ROWS.div_ceil(ROWS_PER_SHEET);

    println!("Test configuration:");
    println!("- Total rows: {:?}", TOTAL_ROWS);
    println!("- Columns: {}", NUM_COLS);
    println!("- Excel limit: {} rows/sheet", EXCEL_MAX_ROWS);
    println!("- Rows per sheet: {}", ROWS_PER_SHEET);
    println!("- Number of sheets: {}", num_sheets);
    println!(
        "- Total cells: {} million",
        (TOTAL_ROWS * NUM_COLS) / 1_000_000
    );
    println!("- Estimated time: 3-5 minutes");
    println!("- Expected file size: ~1.7 GB\n");

    println!("Starting multi-sheet write...");
    let start = Instant::now();

    let mut workbook = FastWorkbook::new("test_10m_multi_sheet.xlsx")?;

    // Headers for all sheets
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

    let mut current_sheet = 0;

    // Create first sheet
    workbook.add_worksheet(&format!("Data_Part_{}", current_sheet + 1))?;
    workbook.write_row(&headers[..NUM_COLS])?;
    let mut rows_in_current_sheet = 1; // Header counts as 1 row

    println!("Sheet 1: Starting...");

    for row_num in 1..=TOTAL_ROWS {
        // Check if we need a new sheet
        if rows_in_current_sheet >= ROWS_PER_SHEET {
            current_sheet += 1;
            println!(
                "Sheet {}: Completed {} rows",
                current_sheet, rows_in_current_sheet
            );
            println!("Sheet {}: Starting...", current_sheet + 1);

            workbook.add_worksheet(&format!("Data_Part_{}", current_sheet + 1))?;
            workbook.write_row(&headers[..NUM_COLS])?;
            rows_in_current_sheet = 1; // Reset counter (header)
        }

        // Generate and write row
        let row = generate_row_data(row_num, NUM_COLS);
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        workbook.write_row(&row_refs)?;
        rows_in_current_sheet += 1;

        // Progress indicator every 100K rows
        if row_num % 100_000 == 0 {
            let elapsed = start.elapsed();
            let speed = row_num as f64 / elapsed.as_secs_f64();
            println!(
                "  Progress: {}/{} rows ({:.1}%) - {:.0} rows/sec - {:?} elapsed",
                row_num,
                TOTAL_ROWS,
                (row_num as f64 / TOTAL_ROWS as f64) * 100.0,
                speed,
                elapsed
            );
        }
    }

    println!(
        "Sheet {}: Completed {} rows",
        current_sheet + 1,
        rows_in_current_sheet
    );

    println!("\nClosing workbook...");
    workbook.close()?;

    let duration = start.elapsed();
    let speed = TOTAL_ROWS as f64 / duration.as_secs_f64();

    println!("\n=== Results ===");
    println!("Total time: {:?}", duration);
    println!("Average speed: {:.0} rows/sec", speed);
    println!("Total sheets created: {}", current_sheet + 1);
    println!("File: test_10m_multi_sheet.xlsx");

    // Check file size
    let metadata = std::fs::metadata("test_10m_multi_sheet.xlsx")?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
    println!("File size: {:.2} MB ({:.2} GB)", size_mb, size_mb / 1024.0);

    println!(
        "\n✅ Success! Created Excel file with {} million rows across {} sheets",
        TOTAL_ROWS / 1_000_000,
        current_sheet + 1
    );

    Ok(())
}

/// Generate a row with mixed data types (all as strings for simplicity)
fn generate_row_data(row_num: usize, num_cols: usize) -> Vec<String> {
    let mut row = Vec::with_capacity(num_cols);

    for col in 0..num_cols {
        let value = match col {
            0 => format!("{}", row_num),
            1 => format!("User_{}", row_num),
            2 => format!("user{}@example.com", row_num),
            3 => format!("{}", 20 + (row_num % 50)),
            4 => format!("{:.2}", 30000.0 + (row_num as f64 * 123.45)),
            5 => if row_num.is_multiple_of(2) {
                "true"
            } else {
                "false"
            }
            .to_string(),
            6 => format!("{:.1}", 50.0 + (row_num % 50) as f64),
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
            27 => format!("Description for record {} with some longer text", row_num),
            28 => format!("{{\"key\":\"{}\",\"value\":{}}}", row_num, row_num * 100),
            _ => format!("REF-{:08}", row_num),
        };
        row.push(value);
    }

    row
}
