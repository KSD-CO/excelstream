# Fast Writer Module

## Overview

The `fast_writer` module is a high-performance Excel writer implementation optimized for streaming large datasets. It achieves **13-24% better performance** compared to the standard `rust_xlsxwriter`-based writer.

## Performance Comparison

| Writer Type | 100K Rows | 1M Rows | Speedup |
|-------------|-----------|---------|---------|
| Standard Writer (rust_xlsxwriter) | 203K rows/sec | 205K rows/sec | 1.0x |
| Fast Writer (custom) | 230K rows/sec | 255K rows/sec | **1.13-1.24x** |

### Benchmark Results

```
=== Standard Writer ===
100,000 rows: 491ms (203,540 rows/sec)

=== Fast Writer ===
100,000 rows: 434ms (230,132 rows/sec)
1,000,000 rows: 3.9s (255,418 rows/sec)

Result: Fast Writer is 13.1% faster
```

## Key Optimizations

1. **Direct XML Generation**: Writes XML directly to byte buffers without intermediate XmlWriter overhead
2. **Reusable Buffers**: Pre-allocated 8KB buffer reused across all row writes
3. **Minimal Allocations**: Cell references and XML tags generated with minimal string allocations
4. **Shared String Deduplication**: HashMap-based string deduplication for efficient string storage
5. **Streaming ZIP Compression**: Direct streaming to ZIP with optimal compression settings

## Implementation Details

### Architecture

```
FastWorkbook
├── ZipWriter<BufWriter<File>> - Buffered ZIP writing (64KB buffer)
├── SharedStrings - String deduplication with HashMap
├── xml_buffer (8KB) - Reusable buffer for XML row generation
└── Direct XML generation - No intermediate XmlWriter overhead
```

### XML Generation Strategy

Instead of using an XmlWriter that adds method call overhead:

**Before (Standard):**
```rust
xml_writer.start_element("row")?;
xml_writer.attribute_int("r", row_num)?;
xml_writer.close_start_tag()?;
// ... more method calls
xml_writer.flush()?;
```

**After (Fast):**
```rust
buffer.clear();
buffer.extend_from_slice(b"<row r=\"");
buffer.extend_from_slice(row_num.to_string().as_bytes());
buffer.extend_from_slice(b"\">");
// ... direct buffer writes
zip.write_all(&buffer)?;
```

## Usage

```rust
use excelstream::fast_writer::FastWorkbook;

let mut workbook = FastWorkbook::new("output.xlsx")?;
workbook.add_worksheet("Sheet1")?;

// Write header
workbook.write_row(&["ID", "Name", "Email", "Age"])?;

// Write data rows
for i in 1..=1_000_000 {
    workbook.write_row(&[
        &i.to_string(),
        &format!("User{}", i),
        &format!("user{}@example.com", i),
        &(20 + (i % 50)).to_string(),
    ])?;
}

workbook.close()?;
```

## When to Use

### Use Fast Writer When:
- ✅ Writing **large datasets** (100K+ rows)
- ✅ Performance is **critical**
- ✅ Simple string data (no complex formatting needed)
- ✅ Streaming/one-pass writing pattern

### Use Standard Writer When:
- ⚠️ Need **rich formatting** (colors, borders, fonts)
- ⚠️ Need **formulas** and cell references
- ⚠️ Need **conditional formatting**
- ⚠️ Need **charts** and images
- ⚠️ Small datasets where performance doesn't matter

## Technical Specifications

### Memory Usage
- **ZipWriter Buffer**: 64KB
- **XML Buffer**: 8KB (reused)
- **SharedStrings**: Grows with unique strings (HashMap overhead)
- **Total Base**: ~72KB + string data

### File Format
- XLSX (Office Open XML)
- ZIP compression level 6 (balanced)
- Minimal XLSX structure (no styling)

### Limitations
- ⚠️ **String data only** (numbers converted to strings)
- ⚠️ **No styling** (fonts, colors, borders)
- ⚠️ **No formulas**
- ⚠️ **Single worksheet** per write session
- ⚠️ **Write-only** (no read-back)

## Future Optimizations

Potential improvements for even better performance:

1. **Type-aware writing**: Support native number types instead of string conversion
2. **Batch row writing**: Write multiple rows in single ZIP operation
3. **Parallel compression**: Use rayon for parallel ZIP compression
4. **Memory-mapped files**: Use mmap for very large files
5. **SIMD XML escaping**: Vectorized XML character escaping

## Examples

See these examples:
- `examples/fast_writer_test.rs` - Performance benchmarks
- `examples/writer_comparison.rs` - Side-by-side comparison

Run comparisons:
```bash
cargo run --release --example writer_comparison
```

## License

MIT License - Same as parent crate
