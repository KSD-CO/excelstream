# Advanced Usage Guide

## Table of Contents
- [Performance Optimization](#performance-optimization)
- [Memory Management](#memory-management)
- [Error Handling](#error-handling)
- [Type Conversions](#type-conversions)
- [Advanced Writing](#advanced-writing)
- [Advanced Reading](#advanced-reading)
- [Best Practices](#best-practices)

## Performance Optimization

### Large File Processing

When processing large Excel files (>100MB), use streaming to optimize memory:

```rust
use excelstream::ExcelReader;

fn process_large_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("huge_file.xlsx")?;
    
    // Process one row at a time, don't load everything into memory
    let mut count = 0;
    let mut sum = 0.0;
    
    for row_result in reader.rows("Sheet1")? {
        let row = row_result?;
        
        // Extract and process data
        if let Some(value) = row.get(2).and_then(|c| c.as_f64()) {
            sum += value;
            count += 1;
        }
        
        // Progress report every 10000 rows
        if count % 10000 == 0 {
            println!("Processed {} rows, current sum: {}", count, sum);
        }
    }
    
    println!("Total: {} rows, sum: {}", count, sum);
    Ok(())
}
```

### Batch Writing

When writing many rows, avoid creating unnecessary new Strings:

```rust
use excelstream::{ExcelWriter, types::CellValue};

fn batch_write_optimized() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("output.xlsx")?;
    
    // Pre-allocate buffers
    let mut row_buffer = Vec::with_capacity(10);
    
    for i in 0..100_000 {
        row_buffer.clear();
        row_buffer.push(CellValue::Int(i));
        row_buffer.push(CellValue::String(format!("Item {}", i)));
        row_buffer.push(CellValue::Float(i as f64 * 1.5));
        
        writer.write_row_typed(&row_buffer)?;
    }
    
    writer.save()?;
    Ok(())
}
```

## Memory Management

### Understanding Memory Usage

```rust
use excelstream::{ExcelReader, ExcelWriter};

// ❌ BAD: Loads all rows into memory
fn bad_approach() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    let all_rows: Vec<_> = reader.rows("Sheet1")?.collect();
    // all_rows holds all data in memory!
    Ok(())
}

// ✅ GOOD: Process one row at a time
fn good_approach() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    for row_result in reader.rows("Sheet1")? {
        let row = row_result?;
        // Process row
        // Row automatically freed after each iteration
    }
    Ok(())
}
```

### Streaming Pipeline

Combine read and write in a pipeline:

```rust
use excelstream::{ExcelReader, ExcelWriter};

fn transform_excel() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("input.xlsx")?;
    let mut writer = ExcelWriter::new("output.xlsx")?;
    
    // Copy header
    let mut rows_iter = reader.rows("Sheet1")?;
    if let Some(header_result) = rows_iter.next() {
        let header = header_result?;
        writer.write_header(&header.to_strings())?;
    }
    
    // Transform and write each row
    for row_result in rows_iter {
        let row = row_result?;
        
        // Transform data (e.g., filter, modify)
        if let Some(value) = row.get(2).and_then(|c| c.as_i64()) {
            if value > 100 {
                writer.write_row(&row.to_strings())?;
            }
        }
    }
    
    writer.save()?;
    Ok(())
}
```

## Error Handling

### Granular Error Handling

```rust
use excelstream::{ExcelReader, ExcelError};

fn handle_errors() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = match ExcelReader::open("data.xlsx") {
        Ok(r) => r,
        Err(ExcelError::IoError(e)) => {
            eprintln!("Failed to open file: {}", e);
            return Err(e.into());
        }
        Err(e) => {
            eprintln!("Excel error: {}", e);
            return Err(e.into());
        }
    };
    
    // Handle missing sheet gracefully
    let rows = match reader.rows("NonExistentSheet") {
        Ok(r) => r,
        Err(ExcelError::SheetNotFound(name)) => {
            println!("Sheet '{}' not found, using first sheet", name);
            reader.rows_by_index(0)?
        }
        Err(e) => return Err(e.into()),
    };
    
    // Process with error recovery
    for (i, row_result) in rows.enumerate() {
        match row_result {
            Ok(row) => {
                // Process row
            }
            Err(e) => {
                eprintln!("Error at row {}: {}", i, e);
                continue; // Skip bad row
            }
        }
    }
    
    Ok(())
}
```

### Validation and Recovery

```rust
use excelstream::ExcelReader;

fn validate_and_process() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    let mut valid_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();
    
    for (i, row_result) in reader.rows("Sheet1")?.enumerate() {
        match row_result {
            Ok(row) => {
                // Validate row structure
                if row.len() < 3 {
                    error_count += 1;
                    errors.push(format!("Row {} has insufficient columns", i));
                    continue;
                }
                
                // Validate data types
                if row.get(1).and_then(|c| c.as_i64()).is_none() {
                    error_count += 1;
                    errors.push(format!("Row {} has invalid age", i));
                    continue;
                }
                
                valid_count += 1;
            }
            Err(e) => {
                error_count += 1;
                errors.push(format!("Row {}: {}", i, e));
            }
        }
    }
    
    println!("Validation complete:");
    println!("  Valid rows: {}", valid_count);
    println!("  Errors: {}", error_count);
    
    if !errors.is_empty() {
        println!("\nError details:");
        for err in errors.iter().take(10) {
            println!("  {}", err);
        }
    }
    
    Ok(())
}
```

## Type Conversions

### Safe Type Extraction

```rust
use excelstream::{ExcelReader, types::CellValue};

fn extract_typed_data() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    struct Record {
        id: i64,
        name: String,
        score: f64,
        active: bool,
    }
    
    let mut records = Vec::new();
    
    for row_result in reader.rows("Sheet1")?.skip(1) { // Skip header
        let row = row_result?;
        
        // Safe extraction with default values
        let id = row.get(0)
            .and_then(|c| c.as_i64())
            .unwrap_or(0);
        
        let name = row.get(1)
            .map(|c| c.as_string())
            .unwrap_or_default();
        
        let score = row.get(2)
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0);
        
        let active = row.get(3)
            .and_then(|c| c.as_bool())
            .unwrap_or(false);
        
        records.push(Record { id, name, score, active });
    }
    
    println!("Loaded {} records", records.len());
    Ok(())
}
```

### Custom Type Conversions

```rust
use excelstream::types::CellValue;
use chrono::{NaiveDate, NaiveDateTime};

trait CellValueExt {
    fn as_date(&self) -> Option<NaiveDate>;
    fn as_datetime(&self) -> Option<NaiveDateTime>;
}

impl CellValueExt for CellValue {
    fn as_date(&self) -> Option<NaiveDate> {
        match self {
            CellValue::DateTime(d) => {
                // Excel date: days since 1900-01-01
                let days = d.floor() as i64;
                NaiveDate::from_ymd_opt(1900, 1, 1)?
                    .checked_add_signed(chrono::Duration::days(days - 2))
            }
            CellValue::String(s) => {
                NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
            }
            _ => None,
        }
    }
    
    fn as_datetime(&self) -> Option<NaiveDateTime> {
        // Implementation
        None
    }
}
```

## Advanced Writing

### Complex Workbooks

```rust
use excelstream::{ExcelWriter, types::CellValue};

fn create_complex_workbook() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("complex.xlsx")?;
    
    // Summary sheet
    writer.write_header(&["Metric", "Value", "Change %"])?;
    writer.write_row_typed(&[
        CellValue::String("Total Sales".to_string()),
        CellValue::Float(1_234_567.89),
        CellValue::Float(12.5),
    ])?;
    
    // Detail sheet
    writer.add_sheet("Details")?;
    writer.write_header(&["Date", "Product", "Quantity", "Revenue"])?;
    
    for i in 1..=100 {
        writer.write_row_typed(&[
            CellValue::String(format!("2024-12-{:02}", i % 30 + 1)),
            CellValue::String(format!("Product {}", i % 10)),
            CellValue::Int(i * 10),
            CellValue::Float(i as f64 * 99.99),
        ])?;
    }
    
    // Set column widths
    writer.set_column_width(0, 12.0)?;
    writer.set_column_width(1, 15.0)?;
    writer.set_column_width(2, 10.0)?;
    writer.set_column_width(3, 12.0)?;
    
    writer.save()?;
    Ok(())
}
```

### Conditional Formatting (Future)

```rust
// Future API design
use excelstream::{ExcelWriter, Format, Condition};

fn conditional_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("formatted.xlsx")?;
    
    // Write data with conditions
    writer.write_header(&["Name", "Score", "Grade"])?;
    
    for score in [95, 85, 75, 65, 55] {
        let grade = match score {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        };
        
        writer.write_row(&[
            &format!("Student {}", score),
            &score.to_string(),
            grade,
        ])?;
    }
    
    writer.save()?;
    Ok(())
}
```

## Advanced Reading

### Selective Reading

```rust
use excelstream::ExcelReader;

fn read_specific_columns() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    // Only extract needed columns
    let columns_needed = [0, 2, 5]; // ID, Name, Score
    
    for row_result in reader.rows("Sheet1")? {
        let row = row_result?;
        
        let selected: Vec<String> = columns_needed
            .iter()
            .filter_map(|&col| row.get(col).map(|c| c.as_string()))
            .collect();
        
        println!("{:?}", selected);
    }
    
    Ok(())
}
```

### Multi-Sheet Processing

```rust
use excelstream::ExcelReader;

fn process_all_sheets() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("workbook.xlsx")?;
    
    let sheets = reader.sheet_names();
    println!("Found {} sheets", sheets.len());
    
    for (idx, sheet_name) in sheets.iter().enumerate() {
        println!("\nProcessing sheet {}: {}", idx + 1, sheet_name);
        
        let (rows, cols) = reader.dimensions(sheet_name)?;
        println!("  Dimensions: {} x {}", rows, cols);
        
        let row_count = reader.rows(sheet_name)?.count();
        println!("  Row count: {}", row_count);
    }
    
    Ok(())
}
```

## Best Practices

### 1. Resource Management

```rust
// ✅ GOOD: Use RAII
fn good_resource_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("output.xlsx")?;
    writer.write_row(&["data"])?;
    writer.save()?; // Explicitly save
    Ok(())
} // writer dropped, resources freed

// ❌ BAD: Forgetting to save
fn bad_resource_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ExcelWriter::new("output.xlsx")?;
    writer.write_row(&["data"])?;
    Ok(())
} // File not saved!
```

### 2. Error Propagation

```rust
use excelstream::{ExcelReader, ExcelWriter};

// ✅ GOOD: Use ? operator
fn good_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("input.xlsx")?;
    let mut writer = ExcelWriter::new("output.xlsx")?;
    
    for row in reader.rows("Sheet1")? {
        writer.write_row(&row?.to_strings())?;
    }
    
    writer.save()?;
    Ok(())
}

// ❌ BAD: Swallowing errors
fn bad_error_handling() {
    let reader = ExcelReader::open("input.xlsx");
    // Error ignored!
}
```

### 3. Performance Patterns

```rust
// ✅ GOOD: Efficient iteration
fn efficient_processing() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    let count = reader.rows("Sheet1")?
        .filter_map(|r| r.ok())
        .filter(|row| !row.is_empty())
        .count();
    
    println!("Non-empty rows: {}", count);
    Ok(())
}

// ❌ BAD: Unnecessary allocations
fn inefficient_processing() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ExcelReader::open("data.xlsx")?;
    
    let all_rows: Vec<_> = reader.rows("Sheet1")?.collect();
    let count = all_rows.iter()
        .filter(|r| r.is_ok())
        .count();
    
    Ok(())
}
```

### 4. Documentation

```rust
/// Process employee data from Excel file
/// 
/// # Arguments
/// * `path` - Path to Excel file
/// * `sheet` - Sheet name to process
/// 
/// # Returns
/// Vector of valid employee records
/// 
/// # Errors
/// Returns error if file doesn't exist or format is invalid
/// 
/// # Examples
/// ```no_run
/// let employees = process_employees("data.xlsx", "Employees")?;
/// ```
fn process_employees(path: &str, sheet: &str) 
    -> Result<Vec<Employee>, Box<dyn std::error::Error>> 
{
    // Implementation
    Ok(vec![])
}

struct Employee {
    id: i64,
    name: String,
}
```

## Integration Examples

### With Web Frameworks (Actix-web)

```rust
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use excelstream::ExcelWriter;

async fn export_to_excel() -> Result<HttpResponse> {
    let mut writer = ExcelWriter::new("temp_export.xlsx")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    writer.write_header(&["ID", "Name", "Email"])
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Add data...
    
    writer.save()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .body("Excel file generated"))
}
```

### With Databases (SQLx)

```rust
use sqlx::postgres::PgPool;
use excelstream::ExcelWriter;

async fn export_from_database(pool: &PgPool) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    let mut writer = ExcelWriter::new("export.xlsx")?;
    writer.write_header(&["ID", "Name", "Email"])?;
    
    let mut rows = sqlx::query!("SELECT id, name, email FROM users")
        .fetch(pool);
    
    while let Some(row) = rows.next().await {
        let row = row?;
        writer.write_row(&[
            &row.id.to_string(),
            &row.name,
            &row.email,
        ])?;
    }
    
    writer.save()?;
    Ok(())
}
```
