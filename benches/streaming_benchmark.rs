use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use excelstream::{ExcelReader, ExcelWriter};
use excelstream::types::CellValue;
use tempfile::NamedTempFile;

fn benchmark_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write");
    
    for size in [100, 1000, 10000, 100000, 1000000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let temp = NamedTempFile::new().unwrap();
                let mut writer = ExcelWriter::new(temp.path()).unwrap();
                
                writer.write_header(&["ID", "Name", "Value"]).unwrap();
                
                for i in 0..size {
                    writer.write_row(&[
                        &i.to_string(),
                        &format!("Name_{}", i),
                        &(i * 100).to_string(),
                    ]).unwrap();
                }
                
                writer.save().unwrap();
            });
        });
    }
    
    group.finish();
}

fn benchmark_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    
    for size in [1000, 10000, 100000].iter() {
        // Prepare test file
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_string_lossy().to_string();
        
        {
            let mut writer = ExcelWriter::new(&path).unwrap();
            writer.write_header(&["ID", "Name", "Value"]).unwrap();
            for i in 0..*size {
                writer.write_row(&[
                    &i.to_string(),
                    &format!("Name_{}", i),
                    &(i * 100).to_string(),
                ]).unwrap();
            }
            writer.save().unwrap();
        }
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut reader = ExcelReader::open(&path).unwrap();
                for row_result in reader.rows_by_index(0).unwrap() {
                    let row = row_result.unwrap();
                    black_box(row);
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_typed_write(c: &mut Criterion) {
    c.bench_function("typed_write_1000_rows", |b| {
        b.iter(|| {
            let temp = NamedTempFile::new().unwrap();
            let mut writer = ExcelWriter::new(temp.path()).unwrap();
            
            for i in 0..1000 {
                writer.write_row_typed(&[
                    CellValue::Int(i),
                    CellValue::String(format!("Name_{}", i)),
                    CellValue::Float(i as f64 * 1.5),
                    CellValue::Bool(i % 2 == 0),
                ]).unwrap();
            }
            
            writer.save().unwrap();
        });
    });
}

fn benchmark_fast_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_write");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let temp = NamedTempFile::new().unwrap();
                let mut writer = ExcelWriter::new(temp.path()).unwrap();
                
                writer.write_header(&["ID", "Name", "Value"]).unwrap();
                
                for i in 0..size {
                    // Use optimized write_row_fast
                    writer.write_row_fast(&[
                        &i.to_string(),
                        &format!("Name_{}", i),
                        &(i * 100).to_string(),
                    ]).unwrap();
                }
                
                writer.save().unwrap();
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_write, benchmark_read, benchmark_typed_write, benchmark_fast_write);
criterion_main!(benches);
