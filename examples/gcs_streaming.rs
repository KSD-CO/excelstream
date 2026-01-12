//! Example: Stream Excel file directly to Google Cloud Storage (TRUE STREAMING - NO TEMP FILES!)
//!
//! This example demonstrates how to generate Excel files and upload them
//! directly to GCS using s-zip's cloud support - NO temporary files needed!
//!
//! Benefits:
//! - ✅ ZERO disk usage (perfect for Cloud Run/Cloud Functions)
//! - ✅ Works in read-only filesystems
//! - ✅ Constant ~4 KB memory usage for buffering
//! - ✅ TRUE streaming using s-zip's GCSZipWriter
//! - ✅ Automatic multipart upload
//!
//! Prerequisites:
//! 1. GCP credentials configured (via gcloud or environment variables)
//! 2. GCS bucket exists with proper permissions
//!
//! Run with:
//! ```bash
//! # Authenticate with gcloud
//! gcloud auth application-default login
//!
//! # Or set service account key
//! export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account-key.json"
//!
//! # Set bucket name
//! export GCS_BUCKET="your-bucket-name"
//!
//! cargo run --example gcs_streaming --features cloud-gcs
//! ```

#[cfg(feature = "cloud-gcs")]
use excelstream::cloud::GCSExcelWriter;

#[cfg(feature = "cloud-gcs")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ExcelStream GCS Direct Streaming Example (v0.14.0)");
    println!("   Using s-zip's cloud support - NO TEMP FILES!\n");

    // Configuration
    let bucket = std::env::var("GCS_BUCKET").unwrap_or_else(|_| "my-excel-reports".to_string());
    let object = "reports/monthly_sales_2024.xlsx";

    println!("📍 Target: gs://{}/{}", bucket, object);
    println!();

    // Create GCS Excel writer - streams directly to GCS!
    println!("⏳ Creating GCS Excel writer...");
    let mut writer = GCSExcelWriter::builder()
        .bucket(&bucket)
        .object(object)
        .build()
        .await?;

    println!("✅ GCS writer initialized (using s-zip's GCSZipWriter)\n");

    // Write header
    println!("📝 Writing header row...");
    writer
        .write_header_bold(["Month", "Product", "Sales", "Profit"])
        .await?;

    // Generate sample data
    println!("📊 Writing sales data...");
    let months = ["January", "February", "March", "April", "May", "June"];
    let products = ["Laptop", "Phone", "Tablet", "Monitor", "Keyboard"];

    let mut row_count = 0;
    for month in &months {
        for product in &products {
            let sales = (row_count * 1000 + 5000) as f64;
            let profit = sales * 0.25;

            let sales_str = format!("{:.2}", sales);
            let profit_str = format!("{:.2}", profit);

            writer
                .write_row([*month, *product, &sales_str, &profit_str])
                .await?;

            row_count += 1;
        }
    }

    println!("✅ Wrote {} rows\n", row_count);

    // Upload to GCS
    println!("☁️  Streaming to GCS (completing multipart upload)...");
    writer.save().await?;

    println!("✅ Upload complete!\n");
    println!("🎉 File available at: gs://{}/{}", bucket, object);
    println!("\n💡 Features:");
    println!("   ✅ ZERO disk usage (no temp files!)");
    println!("   ✅ Constant ~4 KB memory for buffering");
    println!("   ✅ Uses s-zip 0.6.0 cloud support");
    println!("   ✅ Async streaming with tokio");
    println!("\n🔍 Verify with:");
    println!("   gsutil ls -l gs://{}/{}", bucket, object);
    println!(
        "   gsutil cp gs://{}/{} . && unzip -l {}",
        bucket,
        object,
        object.split('/').next_back().unwrap()
    );

    Ok(())
}

#[cfg(not(feature = "cloud-gcs"))]
fn main() {
    eprintln!("❌ This example requires the 'cloud-gcs' feature.");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example gcs_streaming --features cloud-gcs");
    eprintln!("\nMake sure you have GCP credentials configured:");
    eprintln!("  # Option 1: gcloud CLI");
    eprintln!("  gcloud auth application-default login");
    eprintln!("\n  # Option 2: Service account");
    eprintln!("  export GOOGLE_APPLICATION_CREDENTIALS=/path/to/key.json");
    eprintln!("\n  # Set bucket");
    eprintln!("  export GCS_BUCKET=your-bucket-name");
    std::process::exit(1);
}
