//! Memory-constrained writing demo for resource-limited environments

use excelstream::fast_writer::FastWorkbook;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test Memory-Constrained Writing ===\n");

    const NUM_ROWS: usize = 1_000_000;

    // Test 1: Unlimited (default)
    println!("1. Default (flush every 1000 rows):");
    let start = Instant::now();
    test_default("memory_test_default.xlsx", NUM_ROWS)?;
    println!("   Time: {:?}\n", start.elapsed());

    // Test 2: Aggressive flush (every 100 rows) - more memory-efficient
    println!("2. Aggressive flush (every 100 rows) - Memory-efficient:");
    let start = Instant::now();
    test_aggressive_flush("memory_test_aggressive.xlsx", NUM_ROWS)?;
    println!("   Time: {:?}\n", start.elapsed());

    // Test 3: Balanced (every 500 rows)
    println!("3. Balanced (every 500 rows):");
    let start = Instant::now();
    test_balanced("memory_test_balanced.xlsx", NUM_ROWS)?;
    println!("   Time: {:?}\n", start.elapsed());

    // Test 4: Conservative flush (every 5000 rows) - High performance
    println!("4. Conservative flush (every 5000 rows) - High performance:");
    let start = Instant::now();
    test_conservative("memory_test_conservative.xlsx", NUM_ROWS)?;
    println!("   Time: {:?}\n", start.elapsed());

    println!("=== Recommendations ===");
    println!("- Small pods (< 512MB): Use flush_interval = 100");
    println!("- Medium pods (512MB - 1GB): Use flush_interval = 500");
    println!("- Large pods (1-2GB): Use flush_interval = 1000 (default)");
    println!("- Extra large pods (> 2GB): Use flush_interval = 5000 (maximum performance)");

    Ok(())
}

fn test_default(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;
    workbook.add_worksheet("Sheet1")?;

    // Default: flush every 1000 rows
    write_data(&mut workbook, num_rows)?;
    workbook.close()?;
    Ok(())
}

fn test_aggressive_flush(
    filename: &str,
    num_rows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;

    // Configuration for memory-constrained pods
    workbook.set_flush_interval(100); // Flush every 100 rows
    workbook.set_max_buffer_size(256 * 1024); // 256KB max buffer

    workbook.add_worksheet("Sheet1")?;
    write_data(&mut workbook, num_rows)?;
    workbook.close()?;
    Ok(())
}

fn test_balanced(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;

    // Cấu hình cân bằng
    workbook.set_flush_interval(500); // Flush every 500 rows
    workbook.set_max_buffer_size(512 * 1024); // 512KB max buffer

    workbook.add_worksheet("Sheet1")?;
    write_data(&mut workbook, num_rows)?;
    workbook.close()?;
    Ok(())
}

fn test_conservative(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;

    // Configuration for maximum performance
    workbook.set_flush_interval(5000); // Flush every 5000 rows
    workbook.set_max_buffer_size(2 * 1024 * 1024); // 2MB max buffer

    workbook.add_worksheet("Sheet1")?;
    write_data(&mut workbook, num_rows)?;
    workbook.close()?;
    Ok(())
}

fn write_data(
    workbook: &mut FastWorkbook,
    num_rows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Write header
    workbook.write_row(&[
        "KHO",
        "NGÀY HÓA ĐƠN",
        "MẪU SỐ",
        "KÝ HIỆU",
        "SỐ HÓA ĐƠN",
        "TRẠNG THÁI",
        "MÃ SỐ THUẾ",
        "TÊN KHÁCH HÀNG",
        "ĐỊA CHỈ",
        "EMAIL",
        "MÃ CQT CẤP",
        "MÃ SỐ BÍ MẬT",
        "NGÀY KÝ HĐ",
        "MÃ ĐƠN HÀNG",
        "NGÀY CHỨNG TỪ",
        "TỔNG TIỀN HÀNG",
        "VAT",
        "TỔNG THANH TOÁN",
        "THÔNG TIN LỖI",
    ])?;

    // Pre-allocate reusable buffers to avoid per-cell allocation
    let mut mau_so = String::with_capacity(16);
    let mut so_hd = String::with_capacity(16);
    let mut ma_cqt = String::with_capacity(40);
    let mut ma_bi_mat = String::with_capacity(20);
    let mut ma_dh = String::with_capacity(16);
    let mut ngay_ct = String::with_capacity(32);
    let mut tong_tien = String::with_capacity(16);
    let mut vat = String::with_capacity(16);
    let mut thanh_toan = String::with_capacity(16);

    // Static strings (no allocation per row)
    const KHO: &str = "Central Hub Dĩ An";
    const NGAY_HD: &str = "29-11-2025";
    const KY_HIEU: &str = "C25TSF";
    const TRANG_THAI: &str = "CQT đã duyệt";
    const MA_THUE: &str = "";
    const TEN_KH: &str = "Khách hàng không cung cấp thông tin";
    const DIA_CHI: &str = "Khách hàng không cung cấp thông tin";
    const EMAIL: &str = "";
    const NGAY_KY: &str = "29-11-2025 18:35";
    const LOI: &str = "";

    // Write data rows
    for i in 1..=num_rows {
        // Reuse buffers - clear and write
        mau_so.clear();
        use std::fmt::Write;
        write!(&mut mau_so, "{}", i % 10).unwrap();

        so_hd.clear();
        write!(&mut so_hd, "{:08}", 260000 + i).unwrap();

        ma_cqt.clear();
        write!(&mut ma_cqt, "{:032X}", i).unwrap();

        ma_bi_mat.clear();
        write!(&mut ma_bi_mat, "key{:08x}", i).unwrap();

        ma_dh.clear();
        write!(&mut ma_dh, "841495{:04}", i % 10000).unwrap();

        ngay_ct.clear();
        write!(&mut ngay_ct, "29-11-2025 18:35:{:02}", i % 60).unwrap();

        tong_tien.clear();
        write!(&mut tong_tien, "{}.00", 100000 + (i % 500000)).unwrap();

        vat.clear();
        write!(&mut vat, "{}.00", 5000 + (i % 50000)).unwrap();

        thanh_toan.clear();
        write!(&mut thanh_toan, "{}.00", 105000 + (i % 550000)).unwrap();

        workbook.write_row(&[
            KHO,
            NGAY_HD,
            &mau_so,
            KY_HIEU,
            &so_hd,
            TRANG_THAI,
            MA_THUE,
            TEN_KH,
            DIA_CHI,
            EMAIL,
            &ma_cqt,
            &ma_bi_mat,
            NGAY_KY,
            &ma_dh,
            &ngay_ct,
            &tong_tien,
            &vat,
            &thanh_toan,
            LOI,
        ])?;

        // In progress every 100K rows
        if i % 100_000 == 0 {
            println!("   Written {} rows...", i);
        }
    }

    Ok(())
}
