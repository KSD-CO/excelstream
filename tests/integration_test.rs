//! Integration tests for rust-excelize

use excelstream::{ExcelReader, ExcelWriter};
use excelstream::types::CellValue;
use tempfile::NamedTempFile;

#[test]
fn test_write_and_read_roundtrip() {
    // Create temporary file
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    // Write data
    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer.write_header(&["Name", "Age", "City"]).unwrap();
        writer.write_row(&["Alice", "30", "NYC"]).unwrap();
        writer.write_row(&["Bob", "25", "SF"]).unwrap();
        writer.save().unwrap();
    }

    // Read data back
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader.rows_by_index(0)
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
        writer.write_row_typed(&[
            CellValue::String("Alice".to_string()),
            CellValue::Int(30),
            CellValue::Float(1234.56),
            CellValue::Bool(true),
        ]).unwrap();
        writer.save().unwrap();
    }

    // Read and verify types
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader.rows_by_index(0)
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
        
        writer.write_row(&["Sheet1 Data"]).unwrap();
        
        writer.add_sheet("Sheet2").unwrap();
        writer.write_row(&["Sheet2 Data"]).unwrap();
        
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
        writer.write_header(&["ID", "Value"]).unwrap();
        
        for i in 0..num_rows {
            writer.write_row(&[&i.to_string(), &(i * 2).to_string()]).unwrap();
        }
        
        writer.save().unwrap();
    }

    // Read and verify with streaming
    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let row_count = reader.rows_by_index(0)
            .unwrap()
            .count();
        
        assert_eq!(row_count, num_rows + 1); // +1 for header
    }
}

#[test]
fn test_empty_cells() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().to_string();

    {
        let mut writer = ExcelWriter::new(&path).unwrap();
        writer.write_row(&["A", "", "C"]).unwrap();
        writer.write_row_typed(&[
            CellValue::String("X".to_string()),
            CellValue::Empty,
            CellValue::String("Z".to_string()),
        ]).unwrap();
        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let rows: Vec<_> = reader.rows_by_index(0)
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
    writer.write_row(&["Col1", "Col2"]).unwrap();
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
        writer.write_row(&["A", "B", "C"]).unwrap();
        writer.write_row(&["1", "2", "3"]).unwrap();
        writer.write_row(&["X", "Y", "Z"]).unwrap();
        writer.save().unwrap();
    }

    {
        let mut reader = ExcelReader::open(&path).unwrap();
        let (rows, cols) = reader.dimensions(&reader.sheet_names()[0]).unwrap();
        
        assert_eq!(rows, 3);
        assert_eq!(cols, 3);
    }
}
