//! High-level converters for Parquet ↔ Excel

use crate::error::Result;
use crate::parquet::reader::ParquetReader;
use crate::{ExcelReader, ExcelWriter};
use std::path::Path;

/// High-level converter for Parquet → Excel
///
/// This converter provides a simple one-step conversion from Parquet to Excel format.
///
/// # Example
///
/// ```no_run
/// use excelstream::parquet::ParquetToExcelConverter;
///
/// let converter = ParquetToExcelConverter::new("data.parquet")?;
/// converter.convert_to_excel("output.xlsx")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ParquetToExcelConverter {
    parquet_path: String,
}

impl ParquetToExcelConverter {
    /// Create a new converter for the given Parquet file
    ///
    /// # Arguments
    ///
    /// * `parquet_path` - Path to the input Parquet file
    pub fn new<P: AsRef<Path>>(parquet_path: P) -> Result<Self> {
        let path_str = parquet_path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                crate::error::ExcelError::InvalidState("Invalid parquet path".to_string())
            })?
            .to_string();

        Ok(Self {
            parquet_path: path_str,
        })
    }

    /// Convert the Parquet file to Excel
    ///
    /// # Arguments
    ///
    /// * `excel_path` - Path for the output Excel file
    ///
    /// # Returns
    ///
    /// Number of rows converted (excluding header)
    pub fn convert_to_excel<P: AsRef<Path>>(&self, excel_path: P) -> Result<usize> {
        let reader = ParquetReader::open(&self.parquet_path)?;
        let mut writer = ExcelWriter::new(excel_path)?;

        // Write headers
        let headers = reader.column_names();
        writer.write_header_bold(&headers)?;

        // Stream rows
        let mut row_count = 0;
        for row in reader.rows()? {
            let row_data = row?;
            writer.write_row(&row_data)?;
            row_count += 1;
        }

        writer.save()?;
        Ok(row_count)
    }

    /// Convert with progress callback
    ///
    /// # Arguments
    ///
    /// * `excel_path` - Path for the output Excel file
    /// * `callback` - Function called with (current_row, total_rows) after each batch
    ///
    /// # Returns
    ///
    /// Number of rows converted
    pub fn convert_with_progress<P, F>(&self, excel_path: P, mut callback: F) -> Result<usize>
    where
        P: AsRef<Path>,
        F: FnMut(usize, usize),
    {
        let reader = ParquetReader::open(&self.parquet_path)?;
        let total_rows = reader.row_count();
        let mut writer = ExcelWriter::new(excel_path)?;

        // Write headers
        let headers = reader.column_names();
        writer.write_header_bold(&headers)?;

        // Stream rows with progress
        let mut row_count = 0;
        for (idx, row) in reader.rows()?.enumerate() {
            let row_data = row?;
            writer.write_row(&row_data)?;
            row_count += 1;

            // Report progress every 1000 rows
            if (idx + 1) % 1000 == 0 || idx + 1 == total_rows {
                callback(idx + 1, total_rows);
            }
        }

        writer.save()?;
        Ok(row_count)
    }
}

/// High-level converter for Excel → Parquet
///
/// Note: This converter reads the entire Excel file into memory first,
/// then writes to Parquet. For very large files, consider using
/// streaming approaches.
///
/// # Example
///
/// ```no_run
/// use excelstream::parquet::ExcelToParquetConverter;
///
/// let converter = ExcelToParquetConverter::new("data.xlsx")?;
/// converter.convert_to_parquet("output.parquet")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ExcelToParquetConverter {
    excel_path: String,
}

impl ExcelToParquetConverter {
    /// Create a new converter for the given Excel file
    ///
    /// # Arguments
    ///
    /// * `excel_path` - Path to the input Excel file
    pub fn new<P: AsRef<Path>>(excel_path: P) -> Result<Self> {
        let path_str = excel_path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                crate::error::ExcelError::InvalidState("Invalid excel path".to_string())
            })?
            .to_string();

        Ok(Self {
            excel_path: path_str,
        })
    }

    /// Convert the Excel file to Parquet with streaming (constant memory)
    ///
    /// This method:
    /// 1. Reads the Excel file row-by-row
    /// 2. Processes data in batches (10K rows per batch)
    /// 3. Writes to Parquet format with constant memory usage
    ///
    /// # Arguments
    ///
    /// * `parquet_path` - Path for the output Parquet file
    ///
    /// # Returns
    ///
    /// Number of rows converted
    pub fn convert_to_parquet<P: AsRef<Path>>(&self, parquet_path: P) -> Result<usize> {
        use arrow::datatypes::{DataType, Field, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;
        use parquet::file::properties::WriterProperties;
        use std::fs::File;
        use std::sync::Arc;

        const BATCH_SIZE: usize = 10_000; // Process 10K rows at a time

        // Read Excel file
        let mut reader = ExcelReader::open(&self.excel_path)?;
        let sheet_names = reader.sheet_names();

        if sheet_names.is_empty() {
            return Err(crate::error::ExcelError::ReadError(
                "No sheets found in Excel file".to_string(),
            ));
        }

        // Use first sheet
        let sheet_name = &sheet_names[0];
        let mut rows_iter = reader.rows(sheet_name)?;

        // Read first row (headers)
        let headers = match rows_iter.next() {
            Some(Ok(row)) => row.to_strings(),
            Some(Err(e)) => return Err(e),
            None => {
                return Err(crate::error::ExcelError::ReadError(
                    "No data found in Excel file".to_string(),
                ))
            }
        };

        // Create schema (all strings for simplicity)
        let fields: Vec<Field> = headers
            .iter()
            .map(|name| Field::new(name, DataType::Utf8, true))
            .collect();
        let schema = Arc::new(Schema::new(fields));
        let num_columns = headers.len();

        // Create Parquet writer
        let file = File::create(parquet_path)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
            .map_err(|e| crate::error::ExcelError::WriteError(e.to_string()))?;

        // Process rows in batches
        let mut total_rows = 0;
        let mut batch_buffer: Vec<Vec<String>> = Vec::with_capacity(BATCH_SIZE);

        for row_result in rows_iter {
            let row = row_result?;
            batch_buffer.push(row.to_strings());

            // When batch is full, write it and clear buffer
            if batch_buffer.len() >= BATCH_SIZE {
                Self::write_batch(&mut writer, &schema, &batch_buffer, num_columns)?;
                total_rows += batch_buffer.len();
                batch_buffer.clear(); // Free memory
            }
        }

        // Write remaining rows
        if !batch_buffer.is_empty() {
            Self::write_batch(&mut writer, &schema, &batch_buffer, num_columns)?;
            total_rows += batch_buffer.len();
        }

        // Close writer
        writer
            .close()
            .map_err(|e| crate::error::ExcelError::WriteError(e.to_string()))?;

        Ok(total_rows)
    }

    /// Helper method to write a batch of rows to Parquet
    fn write_batch(
        writer: &mut parquet::arrow::arrow_writer::ArrowWriter<std::fs::File>,
        schema: &std::sync::Arc<arrow::datatypes::Schema>,
        rows: &[Vec<String>],
        num_columns: usize,
    ) -> Result<()> {
        use arrow::array::{ArrayRef, StringArray};
        use arrow::record_batch::RecordBatch;
        use std::sync::Arc;

        if rows.is_empty() {
            return Ok(());
        }

        // Convert rows to columnar format
        let mut columns: Vec<ArrayRef> = Vec::with_capacity(num_columns);

        for col_idx in 0..num_columns {
            let col_data: Vec<Option<&str>> = rows
                .iter()
                .map(|row| {
                    if col_idx < row.len() && !row[col_idx].is_empty() {
                        Some(row[col_idx].as_str())
                    } else {
                        None
                    }
                })
                .collect();

            let array = StringArray::from(col_data);
            columns.push(Arc::new(array) as ArrayRef);
        }

        // Create and write record batch
        let batch = RecordBatch::try_new(schema.clone(), columns)
            .map_err(|e| crate::error::ExcelError::WriteError(e.to_string()))?;

        writer
            .write(&batch)
            .map_err(|e| crate::error::ExcelError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// Convert with progress callback
    ///
    /// # Arguments
    ///
    /// * `parquet_path` - Path for the output Parquet file
    /// * `callback` - Function called with progress updates
    pub fn convert_with_progress<P, F>(&self, parquet_path: P, mut callback: F) -> Result<usize>
    where
        P: AsRef<Path>,
        F: FnMut(&str),
    {
        callback("Reading Excel file...");
        let row_count = self.convert_to_parquet(parquet_path)?;
        callback(&format!("Converted {} rows to Parquet", row_count));
        Ok(row_count)
    }
}
