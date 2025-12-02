# Memory-Constrained Writing Guide

## Problem

When writing large Excel files (1M+ rows) in Kubernetes pods with limited resources, you may encounter:
- ❌ OOM (Out of Memory) kills
- ❌ Pod eviction due to memory pressure
- ❌ Slow performance due to memory swapping

## Solution

Fast Writer provides mechanisms to limit memory usage:

### 1. Flush Interval (Most Important)

Flush buffer to disk periodically instead of keeping everything in memory:

```rust
use excelstream::fast_writer::FastWorkbook;

let mut workbook = FastWorkbook::new("output.xlsx")?;

// Flush every 100 rows (most memory-efficient)
workbook.set_flush_interval(100);

workbook.add_worksheet("Sheet1")?;
// ... write data ...
workbook.close()?;
```

### 2. Max Buffer Size

Limit maximum buffer size:

```rust
let mut workbook = FastWorkbook::new("output.xlsx")?;

// Limit buffer to 256KB
workbook.set_max_buffer_size(256 * 1024);

workbook.add_worksheet("Sheet1")?;
```

### 3. SharedStrings Limit

Automatically limit number of unique strings to avoid large HashMap:

```rust
// Default: limit 100K unique strings
// Automatically applied, no configuration needed
```

## Configuration by Pod Resources

### Small Pod (< 512MB RAM)

**Configuration:**
```rust
let mut workbook = FastWorkbook::new("output.xlsx")?;
workbook.set_flush_interval(100);      // Flush mỗi 100 rows
workbook.set_max_buffer_size(256 * 1024); // 256KB buffer
```

**Memory usage:** ~50-100MB  
**Performance:** ~95K rows/sec  
**Trade-off:** Chậm hơn 6% nhưng tiết kiệm memory

### Medium Pod (512MB - 1GB RAM)

**Configuration:**
```rust
let mut workbook = FastWorkbook::new("output.xlsx")?;
workbook.set_flush_interval(500);      // Flush mỗi 500 rows
workbook.set_max_buffer_size(512 * 1024); // 512KB buffer
```

**Memory usage:** ~100-200MB  
**Performance:** ~70K rows/sec  
**Trade-off:** Cân bằng tốt giữa speed và memory

### Large Pod (> 1GB RAM)

**Configuration:**
```rust
let mut workbook = FastWorkbook::new("output.xlsx")?;
// Dùng default settings
// workbook.set_flush_interval(1000);
// workbook.set_max_buffer_size(1024 * 1024);
```

**Memory usage:** ~200-400MB  
**Performance:** ~75K rows/sec (fastest)  
**Trade-off:** Performance tối ưu

## Benchmark Results

Test with 1M rows, 19 columns:

| Flush Interval | Thời gian | Tốc độ | Memory Peak |
|----------------|-----------|--------|-------------|
| **100 rows** (aggressive) | 10.5s | ~95K rows/s | ~80MB |
| **500 rows** (balanced) | 10.9s | ~91K rows/s | ~150MB |
| **1000 rows** (default) | 9.9s | ~101K rows/s | ~250MB |

## Ví dụ Production Code

```rust
use excelstream::fast_writer::FastWorkbook;
use std::env;

fn export_to_excel(data: Vec<Invoice>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = FastWorkbook::new(filename)?;
    
    // Detect resource constraints từ environment
    let memory_limit = env::var("MEMORY_LIMIT_MB")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1024); // Default 1GB
    
    // Auto-tune based on available memory
    if memory_limit < 512 {
        // Low memory pod
        workbook.set_flush_interval(100);
        workbook.set_max_buffer_size(256 * 1024);
    } else if memory_limit < 1024 {
        // Medium memory pod
        workbook.set_flush_interval(500);
        workbook.set_max_buffer_size(512 * 1024);
    }
    // else: use defaults for high memory
    
    workbook.add_worksheet("Invoices")?;
    
    // Write header
    workbook.write_row(&["ID", "Date", "Amount", "Status"])?;
    
    // Write data
    for (i, invoice) in data.iter().enumerate() {
        workbook.write_row(&[
            &invoice.id,
            &invoice.date,
            &invoice.amount,
            &invoice.status,
        ])?;
        
        // Progress logging
        if i % 10_000 == 0 {
            println!("Processed {} records", i);
        }
    }
    
    workbook.close()?;
    Ok(())
}
```

## Kubernetes Deployment

### Pod Resource Limits

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: excel-exporter
spec:
  containers:
  - name: app
    image: your-app:latest
    resources:
      requests:
        memory: "512Mi"
        cpu: "500m"
      limits:
        memory: "1Gi"
        cpu: "1000m"
    env:
    - name: MEMORY_LIMIT_MB
      value: "1024"
```

### Health Checks

```rust
// Add health check to monitor memory
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Track memory usage
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

// Trong code export:
let allocated_mb = ALLOCATED.load(Ordering::Relaxed) / 1024 / 1024;
if allocated_mb > 800 {  // 80% of 1GB limit
    // Force flush hoặc pause
    workbook.zip.flush()?;
    println!("WARNING: High memory usage: {}MB", allocated_mb);
}
```

## Memory Profiling

Để check memory usage trong development:

```bash
# Install heaptrack
sudo apt install heaptrack

# Run with profiling
heaptrack cargo run --release --example memory_constrained_write

# View results
heaptrack_gui heaptrack.*.gz
```

## Best Practices

### ✅ DO:
1. Set `flush_interval` dựa trên pod resources
2. Monitor memory usage trong production
3. Log progress mỗi N rows
4. Test with production-sized datasets
5. Set resource limits rõ ràng trong K8s

### ❌ DON'T:
1. Giữ toàn bộ data trong memory trước khi write
2. Set `flush_interval` too large (> 5000)
3. Bỏ qua memory limits trong K8s
4. Write quá nhiều unique strings (> 100K)
5. Using default settings for all environments

## Troubleshooting

### OOMKilled trong K8s

**Cause:** flush_interval too large  
**Giải pháp:**
```rust
workbook.set_flush_interval(100);  // Giảm xuống 100
```

### Performance quá chậm

**Cause:** flush_interval quá nhỏ  
**Giải pháp:**
```rust
workbook.set_flush_interval(500);  // Tăng lên 500-1000
```

### File bị corrupt

**Cause:** Không gọi `close()` hoặc crash giữa chừng  
**Giải pháp:**
```rust
// Always close properly
workbook.close()?;

// Or use RAII pattern with Drop trait
```

## Kết luận

- ✅ **Flush interval = 100**: Pods < 512MB RAM
- ✅ **Flush interval = 500**: Pods 512MB-1GB RAM  
- ✅ **Flush interval = 1000**: Pods > 1GB RAM
- ✅ Luôn set resource limits trong K8s
- ✅ Monitor memory usage trong production
