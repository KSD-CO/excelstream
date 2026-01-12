//! GCS Performance Test - Test streaming performance to Google Cloud Storage
//!
//! This example measures:
//! - Memory usage during streaming
//! - Throughput (rows/second)
//! - Upload speed
//!
//! Usage:
//! ```bash
//! # Small test (10K rows)
//! export TEST_ROWS=10000
//! export GCS_BUCKET=your-bucket
//! cargo run --release --example gcs_performance_test --features cloud-gcs
//!
//! # Medium test (100K rows)
//! export TEST_ROWS=100000
//! cargo run --release --example gcs_performance_test --features cloud-gcs
//!
//! # Large test (500K rows)
//! export TEST_ROWS=500000
//! cargo run --release --example gcs_performance_test --features cloud-gcs
//! ```

#[cfg(feature = "cloud-gcs")]
use excelstream::cloud::GCSExcelWriter;
#[cfg(feature = "cloud-gcs")]
use std::time::Instant;

#[cfg(feature = "cloud-gcs")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_rows: usize = std::env::var("TEST_ROWS")
        .unwrap_or_else(|_| "100000".to_string())
        .parse()
        .unwrap_or(100000);

    let bucket = std::env::var("GCS_BUCKET").unwrap_or_else(|_| "excelstream-test".to_string());
    let object = format!("performance/test_{}_rows.xlsx", test_rows);

    println!("🔬 GCS Performance Test");
    println!("========================");
    println!("Target:  gs://{}/{}", bucket, object);
    println!("Rows:    {}", test_rows);
    println!();

    let start = Instant::now();

    println!("⏳ Creating GCS writer...");
    let mut writer = GCSExcelWriter::builder()
        .bucket(&bucket)
        .object(&object)
        .build()
        .await?;

    println!("✅ Writer created");
    println!();

    // Write header
    writer
        .write_header_bold([
            "ID",
            "Name",
            "Email",
            "Department",
            "Salary",
            "Hire Date",
            "Status",
        ])
        .await?;

    println!("📊 Writing {} rows...", test_rows);

    for i in 1..=test_rows {
        let id = i.to_string();
        let name = format!("Employee {}", i);
        let email = format!("emp{}@company.com", i);
        let dept = ["Engineering", "Sales", "Marketing", "HR"][i % 4];
        let salary = format!("{:.2}", 50000.0 + (i as f64 * 100.0));
        let hire_date = format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1);
        let status = if i % 10 == 0 { "Inactive" } else { "Active" };

        writer
            .write_row([&id, &name, &email, dept, &salary, &hire_date, status])
            .await?;

        // Progress indicator
        if i % 10000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = i as f64 / elapsed;
            println!(
                "  Progress: {}/{} rows ({:.1}%) - {:.0} rows/sec",
                i,
                test_rows,
                (i as f64 / test_rows as f64) * 100.0,
                rate
            );
        }
    }

    let write_duration = start.elapsed();
    println!();
    println!("✅ All rows written");
    println!();

    println!("☁️  Uploading to GCS...");
    writer.save().await?;

    let total_duration = start.elapsed();

    println!();
    println!("✅ PERFORMANCE RESULTS");
    println!("========================");
    println!("Total rows:       {}", test_rows);
    println!("Write time:       {:.2}s", write_duration.as_secs_f64());
    println!("Upload time:      {:.2}s", total_duration.as_secs_f64());
    println!(
        "Overall rate:     {:.0} rows/sec",
        test_rows as f64 / total_duration.as_secs_f64()
    );
    println!();
    println!("🎉 File available at: gs://{}/{}", bucket, object);
    println!();
    println!("💡 To verify:");
    println!("   gsutil ls -lh gs://{}/{}", bucket, object);

    Ok(())
}

#[cfg(not(feature = "cloud-gcs"))]
fn main() {
    eprintln!("❌ This example requires the 'cloud-gcs' feature.");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --release --example gcs_performance_test --features cloud-gcs");
    std::process::exit(1);
}
