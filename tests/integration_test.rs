//! Integration tests for rust-excelize

use excelstream::types::CellValue;
use excelstream::{ExcelReader, ExcelWriter};
use tempfile::NamedTempFile;

#[test]
fn test_write_and_read_roundtrip() {
    // Create temporary file
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    // Write data
    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer.write_header(["Name", "Age", "City"]).unwrap();
        writer.write_row(["Alice", "30", "NYC"]).unwrap();
        writer.write_row(["Bob", "25", "SF"]).unwrap();
        writer.save().unwrap();
    }

    // Read data back
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader
            .rows_by_index(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 3); // Header + 2 data rows

        // Check header
        let header = &rows[0];
        assert_eq!(header.to_strings(), vec!["Name", "Age", "City"]);

        // Check first data row
        let row1 = &rows[1];
        assert_eq!(row1.to_strings(), vec!["Alice", "30", "NYC"]);
    }
}

#[test]
fn test_typed_cells() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    // Write typed data
    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer
            .write_row_typed(&[
                CellValue::String("Alice".to_string()),
                CellValue::Int(30),
                CellValue::Float(1234.56),
                CellValue::Bool(true),
            ])
            .unwrap();
        writer.save().unwrap();
    }

    // Read and verify types
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader
            .rows_by_index(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 1);
        let row = &rows[0];

        assert_eq!(row.get(0).unwrap().as_string(), "Alice");
        assert_eq!(row.get(1).unwrap().as_i64(), Some(30));
        assert_eq!(row.get(2).unwrap().as_f64(), Some(1234.56));
        assert_eq!(row.get(3).unwrap().as_bool(), Some(true));
    }
}

#[test]
fn test_multi_sheet() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    // Write multiple sheets
    {
        let mut writer = ExcelWriter::new(&path).unwrap();

        writer.write_row(["Sheet1 Data"]).unwrap();

        writer.add_sheet("Sheet2").unwrap();
        writer.write_row(["Sheet2 Data"]).unwrap();

        writer.save().unwrap();
    }

    // Read and verify
    {
        let reader = ExcelReader::open(&path).unwrap();
        let sheets = reader.sheet_names();

        assert!(sheets.len() >= 2);
        // Note: Sheet names might have default names, check if our sheets exist
        assert!(sheets.iter().any(|s| s.contains("Sheet")));
    }
}

#[test]
fn test_large_dataset_streaming() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    let num_rows = 1000;

    // Write large dataset
    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer.write_header(["ID", "Value"]).unwrap();

        for i in 0..num_rows {
            writer
                .write_row([&i.to_string(), &(i * 2).to_string()])
                .unwrap();
        }

        writer.save().unwrap();
    }

    // Read and verify with streaming
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let row_count = reader.rows_by_index(0).unwrap().count();

        assert_eq!(row_count, num_rows + 1); // +1 for header
    }
}

#[test]
fn test_empty_cells() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer.write_row(["A", "", "C"]).unwrap();
        writer
            .write_row_typed(&[
                CellValue::String("X".to_string()),
                CellValue::Empty,
                CellValue::String("Z".to_string()),
            ])
            .unwrap();
        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader
            .rows_by_index(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 2);

        let row1 = &rows[0];
        assert_eq!(row1.get(0).unwrap().as_string(), "A");
        assert_eq!(row1.get(1).unwrap().as_string(), "");
        assert_eq!(row1.get(2).unwrap().as_string(), "C");
    }
}

#[test]
fn test_column_width() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    let mut writer = ExcelWriter::new(&path).unwrap();
    writer.set_column_width(0, 20.0).unwrap();
    writer.set_column_width(1, 30.0).unwrap();
    writer.write_row(["Col1", "Col2"]).unwrap();
    writer.save().unwrap();

    // Just verify no error occurred
    assert!(std::path::Path::new(&path).exists());
}

#[test]
fn test_sheet_dimensions() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer.write_row(["A", "B", "C"]).unwrap();
        writer.write_row(["1", "2", "3"]).unwrap();
        writer.write_row(["X", "Y", "Z"]).unwrap();
        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let (rows, cols) = reader.dimensions(&reader.sheet_names()[0]).unwrap();

        assert_eq!(rows, 3);
        assert_eq!(cols, 3);
    }
}

#[test]
fn test_special_characters() {
    let temp = NamedTempFile::new().unwrap();
    {
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        // Test various special characters
        writer
            .write_row([
                "Text with <xml> tags",
                "Quote: \"Hello\"",
                "Ampersand: &",
                "Apostrophe: '",
            ])
            .unwrap();

        writer
            .write_row(["Emoji: 😀🎉", "Unicode: Ñoño", "Math: ∑∏∫", "Currency: €£¥"])
            .unwrap();

        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(temp.path()).unwrap();
        let mut rows = reader.rows("Sheet1").unwrap();

        let row1 = rows.next().unwrap().unwrap();
        assert_eq!(row1.get(0).unwrap().as_string(), "Text with <xml> tags");
        assert_eq!(row1.get(1).unwrap().as_string(), "Quote: \"Hello\"");
        assert_eq!(row1.get(2).unwrap().as_string(), "Ampersand: &");

        let row2 = rows.next().unwrap().unwrap();
        assert!(row2.get(0).unwrap().as_string().contains("Emoji"));
        assert!(row2.get(1).unwrap().as_string().contains("Unicode"));
    }
}

#[test]
fn test_empty_strings() {
    let temp = NamedTempFile::new().unwrap();
    {
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        writer.write_row(["", "Empty", "", "Values"]).unwrap();
        writer.write_row(["A", "", "C", ""]).unwrap();

        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(temp.path()).unwrap();
        let mut rows = reader.rows("Sheet1").unwrap();

        let row1 = rows.next().unwrap().unwrap();
        assert_eq!(row1.len(), 4);
        assert_eq!(row1.get(0).unwrap().as_string(), "");
        assert_eq!(row1.get(1).unwrap().as_string(), "Empty");
    }
}

#[test]
fn test_very_long_strings() {
    let temp = NamedTempFile::new().unwrap();
    {
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        // Test with a very long string (10KB)
        let long_string = "A".repeat(10_000);
        writer.write_row([&long_string, "Normal", "Text"]).unwrap();

        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(temp.path()).unwrap();
        let mut rows = reader.rows("Sheet1").unwrap();

        let row = rows.next().unwrap().unwrap();
        assert_eq!(row.get(0).unwrap().as_string().len(), 10_000);
        assert_eq!(row.get(1).unwrap().as_string(), "Normal");
    }
}

#[test]
fn test_unicode_sheet_names() {
    let temp = NamedTempFile::new().unwrap();
    {
        let mut writer = ExcelWriter::new(temp.path()).unwrap();

        // Add sheet with Unicode name
        writer.add_sheet("Данные").unwrap(); // Russian
        writer.write_row(["Russian", "Data"]).unwrap();

        writer.add_sheet("数据").unwrap(); // Chinese
        writer.write_row(["Chinese", "Data"]).unwrap();

        writer.add_sheet("Données").unwrap(); // French
        writer.write_row(["French", "Data"]).unwrap();

        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(temp.path()).unwrap();
        let sheets = reader.sheet_names();

        assert!(sheets.contains(&"Данные".to_string()));
        assert!(sheets.contains(&"数据".to_string()));
        assert!(sheets.contains(&"Données".to_string()));

        // Read from Unicode sheet
        let mut rows = reader.rows("Данные").unwrap();
        let row = rows.next().unwrap().unwrap();
        assert_eq!(row.get(0).unwrap().as_string(), "Russian");
    }
}

#[test]
fn test_error_messages() {
    let temp = NamedTempFile::new().unwrap();
    {
        let mut writer = ExcelWriter::new(temp.path()).unwrap();
        writer.write_row(["Data"]).unwrap();
        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(temp.path()).unwrap();

        // Test improved error message for missing sheet
        let result = reader.rows("NonExistent");
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = e.to_string();
            println!("Error message: {}", error_msg);
            // Error should mention both the missing sheet and available sheets
            assert!(error_msg.contains("NonExistent"));
            assert!(error_msg.contains("Available"));
            assert!(error_msg.contains("Sheet1"));
        }
    }
}

#[test]
fn test_row_height() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();

        // Set height for first row
        writer.set_next_row_height(30.0).unwrap();
        writer.write_row(["Tall row"]).unwrap();

        // Normal row (default height)
        writer.write_row(["Normal row"]).unwrap();

        // Another tall row
        writer.set_next_row_height(25.0).unwrap();
        writer.write_row(["Another tall row"]).unwrap();

        writer.save().unwrap();
    }

    // Verify file was created successfully
    assert!(std::path::Path::new(&path).exists());
}

#[test]
fn test_column_width_and_row_height_together() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();

        // Set column widths
        writer.set_column_width(0, 25.0).unwrap();
        writer.set_column_width(1, 15.0).unwrap();
        writer.set_column_width(2, 20.0).unwrap();

        // Write header with custom height
        writer.set_next_row_height(25.0).unwrap();
        writer.write_header_bold(["Name", "Age", "Email"]).unwrap();

        // Regular data rows
        writer
            .write_row(["Alice", "30", "alice@example.com"])
            .unwrap();
        writer.write_row(["Bob", "25", "bob@example.com"]).unwrap();

        writer.save().unwrap();
    }

    // Verify file was created successfully
    assert!(std::path::Path::new(&path).exists());

    // Read back and verify data
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader
            .rows_by_index(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].get(0).unwrap().as_string(), "Name");
        assert_eq!(rows[1].get(0).unwrap().as_string(), "Alice");
        assert_eq!(rows[2].get(0).unwrap().as_string(), "Bob");
    }
}

#[test]
fn test_column_width_must_be_set_before_rows() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();

        // Write a row first
        writer.write_row(["Data"]).unwrap();

        // Try to set column width after writing rows - should fail
        let result = writer.set_column_width(0, 20.0);
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Cannot set column width after writing rows"));
        }

        writer.save().unwrap();
    }
}

#[test]
fn test_multiple_sheets_with_column_widths() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();

        // Sheet 1 with column widths
        writer.set_column_width(0, 20.0).unwrap();
        writer.set_column_width(1, 15.0).unwrap();
        writer.write_row(["Sheet1", "Data"]).unwrap();

        // Sheet 2 with different column widths
        writer.add_sheet("Sheet2").unwrap();
        writer.set_column_width(0, 30.0).unwrap();
        writer.set_column_width(1, 25.0).unwrap();
        writer.write_row(["Sheet2", "Data"]).unwrap();

        writer.save().unwrap();
    }

    // Verify both sheets exist
    {
        let reader = ExcelReader::open(&path).unwrap();
        let sheets = reader.sheet_names();
        assert!(sheets.len() >= 2);
    }
}
