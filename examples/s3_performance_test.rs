//! S3 Performance Test: 30 columns x 100K rows with mixed data types
//!
//! This example tests:
//! - Peak memory usage
//! - Write throughput (rows/sec)
//! - Upload speed to S3
//! - Data types: String, Int, Float, Bool, Date, Formula, Empty
//!
//! Run with:
//! ```bash
//! /usr/bin/time -v cargo run --example s3_performance_test --features cloud-s3 --release
//! ```

#[cfg(feature = "cloud-s3")]
use excelstream::cloud::S3ExcelWriter;
#[cfg(feature = "cloud-s3")]
use std::time::Instant;

#[cfg(feature = "cloud-s3")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 S3 Excel Performance Test - 30 Columns x 100K Rows\n");
    //println!("=" .repeat(70));

    // Configuration
    let bucket = std::env::var("AWS_S3_BUCKET").unwrap_or_else(|_| "lune-nonprod".to_string());
    let key = "performance/test_30col_100k_rows.xlsx";
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "ap-southeast-1".to_string());
    let num_rows = 100_000;

    println!("📋 Test Configuration:");
    println!("   Bucket: {}", bucket);
    println!("   Key: {}", key);
    println!("   Region: {}", region);
    println!("   Rows: {}", num_rows);
    println!("   Columns: 30 (mixed data types)");
    //println!("=" .repeat(70));
    println!();

    // Memory before start
    let memory_before = get_current_memory_kb();
    println!(
        "💾 Memory before start: {:.2} MB",
        memory_before as f64 / 1024.0
    );

    let total_start = Instant::now();

    // Create S3 Excel writer
    println!("\n⏳ Creating S3 Excel writer...");
    let writer_start = Instant::now();
    let mut writer = S3ExcelWriter::builder()
        .bucket(&bucket)
        .key(key)
        .region(&region)
        .compression_level(6)
        .build()
        .await?;

    let writer_time = writer_start.elapsed();
    println!("✅ Writer initialized in {:.2}s", writer_time.as_secs_f64());

    // Write header (30 columns with different data types)
    println!("\n📝 Writing header row (30 columns)...");
    writer.write_header_bold([
        "ID",         // Int
        "Name",       // String
        "Email",      // String
        "Age",        // Int
        "Salary",     // Float
        "IsActive",   // Bool
        "Department", // String
        "JoinDate",   // Date string
        "Score1",     // Float
        "Score2",     // Float
        "Score3",     // Float
        "Average",    // Formula
        "Total",      // Formula
        "City",       // String
        "Country",    // String
        "ZipCode",    // String
        "Phone",      // String
        "Website",    // String
        "Revenue",    // Float
        "Profit",     // Float
        "Margin",     // Float (calculated)
        "Quantity",   // Int
        "UnitPrice",  // Float
        "Discount",   // Float
        "Tax",        // Float
        "NetAmount",  // Float
        "Currency",   // String
        "Status",     // String
        "Priority",   // Int
        "Notes",      // String (long text)
    ])?;

    // Generate and write 100K rows
    println!("\n📊 Writing {} rows with 30 columns each...", num_rows);
    let write_start = Instant::now();

    let mut peak_memory = memory_before;
    let check_interval = 5000;

    for i in 1..=num_rows {
        // Generate row data with mixed types
        let id = i;
        let name = format!("User_{}", i);
        let email = format!("user{}@example.com", i);
        let age = 25 + (i % 40);
        let salary = 50000.0 + (i as f64 * 100.0);
        let is_active = i % 2 == 0;
        let dept = match i % 5 {
            0 => "Engineering",
            1 => "Sales",
            2 => "Marketing",
            3 => "HR",
            _ => "Operations",
        };
        let join_date = format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1);
        let score1 = 70.0 + (i % 30) as f64;
        let score2 = 75.0 + (i % 25) as f64;
        let score3 = 80.0 + (i % 20) as f64;

        let city = match i % 4 {
            0 => "San Francisco",
            1 => "New York",
            2 => "London",
            _ => "Tokyo",
        };
        let country = match i % 4 {
            0 => "USA",
            1 => "USA",
            2 => "UK",
            _ => "Japan",
        };
        let zipcode = format!("{:05}", (i % 99999) + 10000);
        let phone = format!("+1-555-{:04}", i % 10000);
        let website = format!("https://company{}.com", i);

        let revenue = 10000.0 + (i as f64 * 50.0);
        let profit = revenue * 0.25;
        let margin = (profit / revenue) * 100.0;

        let quantity = (i % 100) + 1;
        let unit_price = 10.0 + (i % 1000) as f64;
        let discount = (i % 20) as f64 / 100.0;
        let tax = 0.08;
        let net_amount = (quantity as f64 * unit_price) * (1.0 - discount) * (1.0 + tax);

        let currency = if i % 3 == 0 {
            "USD"
        } else if i % 3 == 1 {
            "EUR"
        } else {
            "JPY"
        };
        let status = match i % 3 {
            0 => "Active",
            1 => "Pending",
            _ => "Completed",
        };
        let priority = (i % 5) + 1;
        let notes = format!(
            "This is a test note for row {} with some additional text to simulate real data",
            i
        );

        // Convert to strings for write_row
        let id_str = id.to_string();
        let age_str = age.to_string();
        let salary_str = format!("{:.2}", salary);
        let is_active_str = is_active.to_string();
        let score1_str = format!("{:.1}", score1);
        let score2_str = format!("{:.1}", score2);
        let score3_str = format!("{:.1}", score3);
        let average_str = format!("{:.1}", (score1 + score2 + score3) / 3.0);
        let total_str = format!("{:.1}", score1 + score2 + score3);
        let revenue_str = format!("{:.2}", revenue);
        let profit_str = format!("{:.2}", profit);
        let margin_str = format!("{:.2}", margin);
        let quantity_str = quantity.to_string();
        let unit_price_str = format!("{:.2}", unit_price);
        let discount_str = format!("{:.2}", discount);
        let tax_str = format!("{:.2}", tax);
        let net_amount_str = format!("{:.2}", net_amount);
        let priority_str = priority.to_string();

        writer.write_row([
            &id_str,
            &name,
            &email,
            &age_str,
            &salary_str,
            &is_active_str,
            dept,
            &join_date,
            &score1_str,
            &score2_str,
            &score3_str,
            &average_str,
            &total_str,
            city,
            country,
            &zipcode,
            &phone,
            &website,
            &revenue_str,
            &profit_str,
            &margin_str,
            &quantity_str,
            &unit_price_str,
            &discount_str,
            &tax_str,
            &net_amount_str,
            currency,
            status,
            &priority_str,
            &notes,
        ])?;

        // Check memory every N rows
        if i % check_interval == 0 {
            let current_mem = get_current_memory_kb();
            if current_mem > peak_memory {
                peak_memory = current_mem;
            }

            let elapsed = write_start.elapsed().as_secs_f64();
            let rows_per_sec = i as f64 / elapsed;
            let progress = (i as f64 / num_rows as f64) * 100.0;

            println!(
                "   Progress: {:>6.2}% | Rows: {:>7} | Speed: {:>8.0} rows/sec | Memory: {:>6.2} MB",
                progress,
                i,
                rows_per_sec,
                current_mem as f64 / 1024.0
            );
        }
    }

    let write_time = write_start.elapsed();
    let rows_per_sec = num_rows as f64 / write_time.as_secs_f64();

    println!("\n✅ Write complete!");
    println!("   Time: {:.2}s", write_time.as_secs_f64());
    println!("   Throughput: {:.0} rows/sec", rows_per_sec);

    // Upload to S3
    println!("\n☁️  Uploading to S3...");
    let upload_start = Instant::now();
    writer.save().await?;
    let upload_time = upload_start.elapsed();

    let total_time = total_start.elapsed();

    // Final memory check
    let memory_after = get_current_memory_kb();

    //println!("\n" + &"=".repeat(70));
    println!("📊 Performance Results:");
    //println!("=" .repeat(70));
    println!("✅ Success! File uploaded to: s3://{}/{}", bucket, key);
    println!();
    println!("⏱️  Timing:");
    println!(
        "   Writer initialization: {:.2}s",
        writer_time.as_secs_f64()
    );
    println!(
        "   Writing {} rows:     {:.2}s",
        num_rows,
        write_time.as_secs_f64()
    );
    println!(
        "   S3 upload:             {:.2}s",
        upload_time.as_secs_f64()
    );
    println!("   Total time:            {:.2}s", total_time.as_secs_f64());
    println!();
    println!("🚀 Throughput:");
    println!("   Rows per second:       {:.0}", rows_per_sec);
    println!("   Cells per second:      {:.0}", rows_per_sec * 30.0);
    println!();
    println!("💾 Memory Usage:");
    println!(
        "   Memory before:         {:.2} MB",
        memory_before as f64 / 1024.0
    );
    println!(
        "   Peak memory:           {:.2} MB",
        peak_memory as f64 / 1024.0
    );
    println!(
        "   Memory after:          {:.2} MB",
        memory_after as f64 / 1024.0
    );
    println!(
        "   Peak delta:            {:.2} MB",
        (peak_memory - memory_before) as f64 / 1024.0
    );
    //println!("=" .repeat(70));

    Ok(())
}

#[cfg(feature = "cloud-s3")]
fn get_current_memory_kb() -> usize {
    // Read from /proc/self/status on Linux
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(mem_str) = line.split_whitespace().nth(1) {
                    if let Ok(mem) = mem_str.parse::<usize>() {
                        return mem;
                    }
                }
            }
        }
    }
    0
}

#[cfg(not(feature = "cloud-s3"))]
fn main() {
    eprintln!("❌ This example requires the 'cloud-s3' feature.");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example s3_performance_test --features cloud-s3 --release");
    eprintln!("\nFor detailed memory stats:");
    eprintln!(
        "  /usr/bin/time -v cargo run --example s3_performance_test --features cloud-s3 --release"
    );
    std::process::exit(1);
}
