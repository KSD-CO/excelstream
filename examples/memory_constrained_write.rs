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

fn test_aggressive_flush(filename: &str, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
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

fn write_data(workbook: &mut FastWorkbook, num_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Write header
    workbook.write_row(&[
        "KHO", "NGÀY HÓA ĐƠN", "MẪU SỐ", "KÝ HIỆU", "SỐ HÓA ĐƠN",
        "TRẠNG THÁI", "MÃ SỐ THUẾ", "TÊN KHÁCH HÀNG", "ĐỊA CHỈ", "EMAIL",
        "MÃ CQT CẤP", "MÃ SỐ BÍ MẬT", "NGÀY KÝ HĐ", "MÃ ĐƠN HÀNG",
        "NGÀY CHỨNG TỪ", "TỔNG TIỀN HÀNG", "VAT", "TỔNG THANH TOÁN", "THÔNG TIN LỖI"
    ])?;
    
    // Write data rows
    for i in 1..=num_rows {
        let kho = format!("Central Hub Dĩ An");
        let ngay_hd = format!("29-11-2025");
        let mau_so = format!("{}", i % 10);
        let ky_hieu = format!("C25TSF");
        let so_hd = format!("{:08}", 260000 + i);
        let trang_thai = format!("CQT đã duyệt");
        let ma_thue = format!("");
        let ten_kh = format!("Khách hàng không cung cấp thông tin");
        let dia_chi = format!("Khách hàng không cung cấp thông tin");
        let email = format!("");
        let ma_cqt = format!("{:032X}", i);
        let ma_bi_mat = format!("key{:08x}", i);
        let ngay_ky = format!("29-11-2025 18:35");
        let ma_dh = format!("841495{:04}", i % 10000);
        let ngay_ct = format!("29-11-2025 18:35:{:02}", i % 60);
        let tong_tien = format!("{}.00", 100000 + (i % 500000));
        let vat = format!("{}.00", 5000 + (i % 50000));
        let thanh_toan = format!("{}.00", 105000 + (i % 550000));
        let loi = format!("");
        
        workbook.write_row(&[
            &kho, &ngay_hd, &mau_so, &ky_hieu, &so_hd,
            &trang_thai, &ma_thue, &ten_kh, &dia_chi, &email,
            &ma_cqt, &ma_bi_mat, &ngay_ky, &ma_dh,
            &ngay_ct, &tong_tien, &vat, &thanh_toan, &loi
        ])?;
        
        // In progress every 100K rows
        if i % 100_000 == 0 {
            println!("   Written {} rows...", i);
        }
    }
    
    Ok(())
}
