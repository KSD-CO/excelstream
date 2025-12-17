//! HTTP Memory Test - Check peak memory usage
//!
//! Run with:
//! ```bash
//! cargo run --example http_memory_test --features cloud-http --release
//! ```

use excelstream::cloud::HttpExcelWriter;
use excelstream::types::CellValue;

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

fn test_memory_usage(rows: usize, desc: &str) {
    println!("\n📊 Testing: {} ({} rows)", desc, rows);

    let mut writer = HttpExcelWriter::with_compression(6);

    // Write header
    writer
        .write_header_bold(["ID", "Name", "Email", "Score", "Status", "Date", "Amount"])
        .unwrap();

    // Write data
    for i in 1..=rows {
        writer
            .write_row_typed(&[
                CellValue::Int(i as i64),
                CellValue::String(format!("User_{}", i)),
                CellValue::String(format!("user{}@example.com", i)),
                CellValue::Float(50.0 + (i % 50) as f64),
                CellValue::String(if i % 3 == 0 { "Active" } else { "Inactive" }.to_string()),
                CellValue::String(format!("2024-01-{:02}", (i % 28) + 1)),
                CellValue::Float(100.0 * (i as f64)),
            ])
            .unwrap();

        if i % 10000 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }

    println!("\n  Finishing...");
    let bytes = writer.finish().unwrap();

    println!("  ✅ Generated: {}", format_bytes(bytes.len()));
    println!("  📦 Final size: {}", format_bytes(bytes.len()));
}

fn main() {
    println!("🧪 HTTP Excel Writer Memory Test\n");
    println!("This test generates Excel files of various sizes to check memory usage.");
    println!("Monitor with: ps aux | grep http_memory_test\n");

    // Small test
    test_memory_usage(1_000, "Small dataset");

    // Medium test
    test_memory_usage(10_000, "Medium dataset");

    // Large test
    test_memory_usage(50_000, "Large dataset");

    // Very large test
    test_memory_usage(100_000, "Very large dataset");

    println!("\n✅ All tests completed!");
    println!("\nMemory characteristics:");
    println!("- Pure in-memory implementation (no temp files)");
    println!("- Peak memory includes: compressed data + XML buffers + ZIP overhead");
    println!("- Memory efficient due to streaming XML generation");
    println!("\nFor >100MB files, consider using file-based streaming instead.");
}
