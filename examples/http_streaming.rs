//! HTTP Streaming Example with Axum
//!
//! This example demonstrates how to stream Excel files directly via HTTP
//! using the HttpExcelWriter with Axum web framework.
//!
//! Run with:
//! ```bash
//! cargo run --example http_streaming --features cloud-http
//! ```
//!
//! Then test with:
//! ```bash
//! curl -o report.xlsx http://localhost:3000/download/sales-report
//! curl -o large.xlsx http://localhost:3000/download/large-dataset
//! ```

use axum::{
    extract::Path,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use excelstream::cloud::HttpExcelWriter;
use excelstream::types::CellValue;

#[tokio::main]
async fn main() {
    println!("🚀 Starting HTTP Excel Streaming Server...");
    println!("📊 Server listening on http://localhost:3000");
    println!("\nAvailable endpoints:");
    println!("  GET /download/sales-report    - Download sales report");
    println!("  GET /download/large-dataset   - Download large dataset (10K rows)");
    println!("  GET /download/multi-sheet     - Download multi-sheet workbook");
    println!("\nTest with:");
    println!("  curl -o report.xlsx http://localhost:3000/download/sales-report");

    let app = Router::new()
        .route("/", get(home))
        .route("/download/:report_name", get(download_report));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("\n✅ Server ready!\n");

    axum::serve(listener, app).await.unwrap();
}

async fn home() -> &'static str {
    "Excel Streaming Server\n\nEndpoints:\n\
    - GET /download/sales-report\n\
    - GET /download/large-dataset\n\
    - GET /download/multi-sheet\n"
}

async fn download_report(Path(report_name): Path<String>) -> Response {
    match report_name.as_str() {
        "sales-report" => generate_sales_report().await,
        "large-dataset" => generate_large_dataset().await,
        "multi-sheet" => generate_multi_sheet().await,
        _ => (
            axum::http::StatusCode::NOT_FOUND,
            "Report not found. Available: sales-report, large-dataset, multi-sheet",
        )
            .into_response(),
    }
}

async fn generate_sales_report() -> Response {
    println!("📊 Generating sales report...");

    let mut writer = HttpExcelWriter::new();

    // Write header
    writer
        .write_header_bold(["Month", "Revenue", "Profit", "Customers"])
        .unwrap();

    // Write data
    let data = [
        ("January", 125000.50, 45000.25, 1250),
        ("February", 135000.75, 48000.50, 1320),
        ("March", 142000.25, 51000.75, 1400),
        ("April", 138000.00, 49500.00, 1380),
        ("May", 155000.50, 55000.25, 1550),
        ("June", 168000.75, 60000.50, 1680),
    ];

    for (month, revenue, profit, customers) in data {
        writer
            .write_row_typed(&[
                CellValue::String(month.to_string()),
                CellValue::Float(revenue),
                CellValue::Float(profit),
                CellValue::Int(customers),
            ])
            .unwrap();
    }

    let bytes = writer.finish().unwrap();

    println!("✅ Generated {} bytes", bytes.len());

    (
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"sales-report.xlsx\"",
            ),
        ],
        bytes,
    )
        .into_response()
}

async fn generate_large_dataset() -> Response {
    println!("📊 Generating large dataset (10K rows)...");

    let mut writer = HttpExcelWriter::with_compression(6);

    // Write header
    writer
        .write_header_bold(["ID", "Name", "Email", "Score", "Status"])
        .unwrap();

    // Generate 10,000 rows
    for i in 1..=10_000 {
        writer
            .write_row_typed(&[
                CellValue::Int(i),
                CellValue::String(format!("User_{}", i)),
                CellValue::String(format!("user{}@example.com", i)),
                CellValue::Float(50.0 + (i % 50) as f64),
                CellValue::String(if i % 3 == 0 { "Active" } else { "Inactive" }.to_string()),
            ])
            .unwrap();

        if i % 1000 == 0 {
            println!("  Progress: {} rows written", i);
        }
    }

    let bytes = writer.finish().unwrap();

    println!(
        "✅ Generated {} bytes ({:.2} MB)",
        bytes.len(),
        bytes.len() as f64 / 1_048_576.0
    );

    (
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"large-dataset.xlsx\"",
            ),
        ],
        bytes,
    )
        .into_response()
}

async fn generate_multi_sheet() -> Response {
    println!("📊 Generating multi-sheet workbook...");

    let mut writer = HttpExcelWriter::new();

    // Sheet 1: Sales
    writer.add_worksheet("Sales").unwrap();
    writer
        .write_header_bold(["Product", "Quantity", "Price"])
        .unwrap();
    writer
        .write_row_typed(&[
            CellValue::String("Laptop".to_string()),
            CellValue::Int(50),
            CellValue::Float(1299.99),
        ])
        .unwrap();
    writer
        .write_row_typed(&[
            CellValue::String("Mouse".to_string()),
            CellValue::Int(200),
            CellValue::Float(29.99),
        ])
        .unwrap();

    // Sheet 2: Inventory
    writer.add_worksheet("Inventory").unwrap();
    writer
        .write_header_bold(["Item", "Stock", "Warehouse"])
        .unwrap();
    writer.write_row(["Laptop", "150", "Main"]).unwrap();
    writer.write_row(["Mouse", "500", "Secondary"]).unwrap();

    // Sheet 3: Summary
    writer.add_worksheet("Summary").unwrap();
    writer.write_header_bold(["Metric", "Value"]).unwrap();
    writer.write_row(["Total Products", "2"]).unwrap();
    writer.write_row(["Total Revenue", "69,997.50"]).unwrap();

    let bytes = writer.finish().unwrap();

    println!("✅ Generated {} bytes with 3 sheets", bytes.len());

    (
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"multi-sheet.xlsx\"",
            ),
        ],
        bytes,
    )
        .into_response()
}
