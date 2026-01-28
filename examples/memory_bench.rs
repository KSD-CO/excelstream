use excelstream::ExcelWriter;
use std::fs;
use std::hint::black_box;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _profiler = dhat::Profiler::new_heap();

    let path = "memory_bench_test.xlsx";
    // Ensure cleanup from previous runs
    if std::path::Path::new(path).exists() {
        fs::remove_file(path)?;
    }

    let mut writer = ExcelWriter::new(path)?;

    // Simulate writing 100,000 rows
    for _ in 0..100_000 {
        writer.write_row(black_box([
            "Column 1 Data",
            "Column 2 Data",
            "12345",
            "2023-01-01",
            "Some longer text content to simulate real world data usage",
        ]))?;
    }

    writer.save()?;

    // Cleanup
    if std::path::Path::new(path).exists() {
        fs::remove_file(path)?;
    }

    println!("Benchmark complete");
    Ok(())
}
