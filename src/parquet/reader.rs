//! Parquet file reader with streaming support

use crate::error::{ExcelError, Result};
use arrow::array::*;
use arrow::datatypes::*;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Parquet file reader that provides row-by-row streaming access
///
/// This reader converts Parquet columnar data to row-oriented format
/// suitable for Excel conversion.
///
/// # Example
///
/// ```no_run
/// use excelstream::parquet::ParquetReader;
/// use excelstream::ExcelWriter;
///
/// let mut reader = ParquetReader::open("data.parquet")?;
/// let mut writer = ExcelWriter::new("output.xlsx")?;
///
/// // Write headers
/// writer.write_header_bold(&reader.column_names())?;
///
/// // Stream rows
/// for row in reader.rows()? {
///     writer.write_row(&row?)?;
/// }
/// writer.save()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ParquetReader {
    file_path: String,
    schema: SchemaRef,
    row_count: usize,
}

impl ParquetReader {
    /// Open a Parquet file for reading
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Parquet file
    ///
    /// # Returns
    ///
    /// A new ParquetReader instance
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| ExcelError::ReadError("Invalid file path".to_string()))?
            .to_string();

        let file = File::open(&path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| ExcelError::ReadError(format!("Failed to open Parquet file: {}", e)))?;

        let metadata = builder.metadata();
        let schema = builder.schema().clone();

        // Calculate total row count
        let row_count = metadata.file_metadata().num_rows().try_into().unwrap_or(0);

        Ok(Self {
            file_path: path_str,
            schema,
            row_count,
        })
    }

    /// Get column names from the Parquet schema
    pub fn column_names(&self) -> Vec<String> {
        self.schema
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect()
    }

    /// Get the schema of the Parquet file
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// Get total number of rows in the file
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Create an iterator over rows
    ///
    /// Returns an iterator that yields rows as Vec<String>
    pub fn rows(&self) -> Result<ParquetRowIterator> {
        let file = File::open(&self.file_path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| ExcelError::ReadError(format!("Failed to open Parquet file: {}", e)))?;

        let reader = builder
            .build()
            .map_err(|e| ExcelError::ReadError(format!("Failed to build reader: {}", e)))?;

        Ok(ParquetRowIterator {
            reader: Box::new(reader),
            current_batch: None,
            current_row: 0,
            schema: self.schema.clone(),
        })
    }
}

/// Iterator over Parquet rows converted to string vectors
pub struct ParquetRowIterator {
    reader: Box<dyn Iterator<Item = arrow::error::Result<RecordBatch>>>,
    current_batch: Option<RecordBatch>,
    current_row: usize,
    #[allow(dead_code)]
    schema: SchemaRef,
}

impl Iterator for ParquetRowIterator {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current batch, try to get the next row from it
            if let Some(ref batch) = self.current_batch {
                if self.current_row < batch.num_rows() {
                    let row = self.extract_row(batch, self.current_row);
                    self.current_row += 1;
                    return Some(row);
                }
            }

            // Need to load next batch
            match self.reader.next() {
                Some(Ok(batch)) => {
                    self.current_batch = Some(batch);
                    self.current_row = 0;
                }
                Some(Err(e)) => {
                    return Some(Err(ExcelError::ReadError(format!(
                        "Failed to read batch: {}",
                        e
                    ))))
                }
                None => return None, // End of data
            }
        }
    }
}

impl ParquetRowIterator {
    fn extract_row(&self, batch: &RecordBatch, row_idx: usize) -> Result<Vec<String>> {
        let mut row = Vec::with_capacity(batch.num_columns());

        for col_idx in 0..batch.num_columns() {
            let array = batch.column(col_idx);
            let value = self.array_value_to_string(array, row_idx)?;
            row.push(value);
        }

        Ok(row)
    }

    fn array_value_to_string(&self, array: &Arc<dyn Array>, row_idx: usize) -> Result<String> {
        if array.is_null(row_idx) {
            return Ok(String::new());
        }

        let value = match array.data_type() {
            DataType::Utf8 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to StringArray".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::LargeUtf8 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<LargeStringArray>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to LargeStringArray".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::Int8 => {
                let arr = array.as_any().downcast_ref::<Int8Array>().ok_or_else(|| {
                    ExcelError::ReadError("Failed to downcast to Int8Array".to_string())
                })?;
                arr.value(row_idx).to_string()
            }
            DataType::Int16 => {
                let arr = array.as_any().downcast_ref::<Int16Array>().ok_or_else(|| {
                    ExcelError::ReadError("Failed to downcast to Int16Array".to_string())
                })?;
                arr.value(row_idx).to_string()
            }
            DataType::Int32 => {
                let arr = array.as_any().downcast_ref::<Int32Array>().ok_or_else(|| {
                    ExcelError::ReadError("Failed to downcast to Int32Array".to_string())
                })?;
                arr.value(row_idx).to_string()
            }
            DataType::Int64 => {
                let arr = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                    ExcelError::ReadError("Failed to downcast to Int64Array".to_string())
                })?;
                arr.value(row_idx).to_string()
            }
            DataType::UInt8 => {
                let arr = array.as_any().downcast_ref::<UInt8Array>().ok_or_else(|| {
                    ExcelError::ReadError("Failed to downcast to UInt8Array".to_string())
                })?;
                arr.value(row_idx).to_string()
            }
            DataType::UInt16 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<UInt16Array>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to UInt16Array".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::UInt32 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to UInt32Array".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::UInt64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<UInt64Array>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to UInt64Array".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::Float32 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to Float32Array".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::Float64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to Float64Array".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::Boolean => {
                let arr = array
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .ok_or_else(|| {
                        ExcelError::ReadError("Failed to downcast to BooleanArray".to_string())
                    })?;
                arr.value(row_idx).to_string()
            }
            DataType::Date32 | DataType::Date64 => {
                // Convert date to string representation
                if let Some(arr) = array.as_any().downcast_ref::<Date32Array>() {
                    let days = arr.value(row_idx);
                    format!("DATE({})", days)
                } else if let Some(arr) = array.as_any().downcast_ref::<Date64Array>() {
                    let millis = arr.value(row_idx);
                    format!("DATE({})", millis)
                } else {
                    String::new()
                }
            }
            DataType::Timestamp(_, _) => {
                // Generic timestamp handling
                "TIMESTAMP".to_string()
            }
            _ => {
                // Fallback for unsupported types
                format!("<{:?}>", array.data_type())
            }
        };

        Ok(value)
    }
}
