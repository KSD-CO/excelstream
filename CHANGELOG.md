# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.1] - 2024-12-16

### Changed
- **s-zip v0.2.0**: Updated to s-zip 0.2.0 with Write trait support
  - Enables `from_writer<W: Write + Seek>()` for in-memory Excel generation
  - Perfect for web servers (no temp files needed)
  - All backward compatible, zero breaking changes

### Internal
- Clarified comments for optional dependencies (PostgreSQL, AWS)
- All dependencies are optional features, only loaded when needed

### Performance
- Verified: Same performance as v0.11.0 (42K rows/sec, 2.5 MB memory)
- All 40 tests passing
- All 7 examples working


## [0.11.0] - 2024-12-15

### Changed
- **s-zip Library Integration**: Extracted ZIP operations into standalone [s-zip](https://crates.io/crates/s-zip) crate
  - **Code Reusability**: ~544 lines of ZIP code now reusable across projects
  - **Zero Performance Impact**: Identical speed (42K rows/sec) and memory (2-3 MB)
  - **Better Maintainability**: Single source of truth for ZIP operations
  - **Community Value**: s-zip now available for other Rust projects
  - **Backward Compatible**: All existing APIs work without changes

### Internal
- Replaced internal `streaming_zip_reader.rs` and `streaming_zip_writer.rs` with s-zip dependency
- Updated `fast_writer` modules to re-export s-zip types for backward compatibility
- Added `From<s_zip::SZipError>` conversion for seamless error handling

### Performance (unchanged)
- Write: 42,557 rows/sec (strings), 43,839 rows/sec (direct)
- Read: 36,396 rows/sec with streaming
- Memory: 2-3 MB constant for any file size
- File size: 180-193 MB (1M rows, 30 columns)


## [0.9.0] - 2024-12-08

### 🎉 Major Feature: Zero-Temp Streaming Architecture

**84% Memory Reduction** - Revolutionary streaming write implementation that eliminates temporary files entirely!

### Added
- **ZeroTempWorkbook**: Stream XML directly into ZIP compressor
  - **2.7 MB RAM** for ANY SIZE (vs 17 MB in v0.8.0) = **84% reduction**
  - **Zero temp files**: Direct streaming to final .xlsx file
  - **StreamingZipWriter**: Custom ZIP writer with on-the-fly compression
  - **Data descriptors**: Write CRC/sizes after compressed data (no seeking)
  - **4KB XML buffer**: Reused per row for minimal allocations
  - **Same speed**: 50K-60K rows/sec (comparable to zip crate)
  - **File size**: ~7% larger than zip crate (acceptable trade-off)

### Performance
- **Write Performance (1M rows)**:
  - `ZeroTempWorkbook`: **2.7 MB RAM**, ~1400ms, 16 MB file ✅
  - `UltraLowMemoryWorkbook` (v0.8.0): 17 MB RAM, ~1400ms, 15 MB file
  - Traditional libraries: 100+ MB RAM or crash
- **Architecture**: On-the-fly DEFLATE compression with streaming XML generation
- **Validation**: Tested with 1M-10M rows, verified with Excel and `unzip -t`

### Changed
- Bumped version to 0.9.0
- Updated Cargo.toml description to highlight 2.7 MB memory footprint
- Updated README with v0.9.0 performance numbers and new architecture
- Exposed `ZeroTempWorkbook` in public API alongside `UltraLowMemoryWorkbook`

### Technical Details
- **StreamingZipWriter**: 
  - Writes local file headers with bit 3 set (data descriptor flag)
  - Writes data descriptors after compressed data (CRC32, sizes)
  - Assembles central directory at end
  - No seeking, no temp files, pure streaming
- **ZeroTempWorkbook**:
  - `new(path, compression_level)`: Create workbook with configurable compression
  - `add_worksheet(name)`: Start new sheet with immediate XML header write
  - `write_row(values)`: Build row XML in 4KB buffer, stream to compressor
  - `close()`: Finish sheet, write metadata files, close ZIP
- **CrcCountingWriter**: Helper that computes CRC32 while writing compressed data

### Notes
- **Backward compatibility**: `UltraLowMemoryWorkbook` still available (uses temp files)
- **Migration**: Consider switching to `ZeroTempWorkbook` for 84% memory reduction
- **Trade-offs**: File size +7% (16 MB vs 15 MB) due to streaming compression

## [0.8.0] - 2024-12-06

### Changed
- **🎯 Constant Memory Streaming**: StreamingReader now achieves true constant memory
  - **104x Memory Reduction**: 1.2 GB XML → 11.6 MB RAM (was 1204 MB in v0.7.1)
  - **100% Accurate**: Fixed chunked reading algorithm to capture all rows without data loss
  - **Any File Size**: Process multi-GB files with only 10-12 MB RAM
  - **Tested**: 86 MB file (1.2 GB uncompressed) = 11.6 MB peak memory, 1M rows read correctly

### Performance
- **⚡ Read Performance**: 
  - `StreamingReader`: 50K-60K rows/sec with constant 10-12 MB memory
  - `ExcelReader`: 30K rows/sec (memory = file size)
  - **Production Ready**: Validated with 1 million row files in real-world scenarios

### Documentation
- **📖 Updated README**: Accurate v0.8.0 performance numbers and use cases
- **🎯 New Use Case**: "Processing Large Excel Imports" with StreamingReader
- **📊 Updated Comparison Table**: 10-12 MB constant memory for all file sizes

## [0.7.1] - 2024-12-06

### Added
- **🚀 StreamingReader**: Initial streaming reader implementation
  - Constant 20-30 MB memory for ANY file size
  - 2x faster than ExcelReader: 60K rows/sec vs 30K rows/sec
  - Trade-offs: No formula evaluation, no formatting, sequential only

### Known Issues (Fixed in v0.8.0)
- ⚠️ Memory usage was actually 1.2 GB for large files (loaded full XML)
- ⚠️ Missing ~8K rows per 1M rows due to chunking bug

## [0.7.0] - 2024-12-05

### Added
- **🔒 Worksheet Protection**: Password-based sheet protection
  - 12 granular permissions (select locked/unlocked, format, sort, etc.)
  - ECMA-376 compliant password hashing
  - Prevents accidental edits in production reports
  
- **📐 Cell Merging**: Horizontal and vertical cell merging
  - Support for ranges: A1:C1, A1:A3, B2:D5
  - Perfect for report headers and grouped data
  - Proper Excel format `<mergeCells>` implementation

- **📏 Column Width**: Set custom column widths
  - Previously no-op, now fully functional
  - Support for all 16,384 columns (A-XFD)
  - Width in Excel character units (default 8.43)

### Documentation
- **📖 README Overhaul**: Streaming use cases and comparisons
- **🎯 Real-World Examples**: 5 production scenarios
- **📊 Comparison Table**: ExcelStream vs traditional libraries

## [0.6.2] - 2024-12-05

### Changed
- **⬆️ Upgraded Dependencies**: Updated `zip` crate from 0.6 to 6.0
  - Fixed `flate2` conflict issues on GitHub Actions CI
  - Updated API: `FileOptions` → `SimpleFileOptions`
  - Updated compression_level type: `i32` → `i64`
  - Better compression algorithms and bug fixes

### Removed
- **🗑️ Removed Deprecated Function**: Deleted `fix_xlsx_zip_order()`
  - No longer needed as files are written in correct order by default since v0.6.0
  - Improves performance by eliminating unnecessary reordering step
  - Reduces code complexity

### Performance
- **💾 Memory Optimization**: Memory usage improved by ~2%
  - `write_row()`: 56 MB → 55 MB
  - `write_row_typed()`: 56 MB → 55 MB
  - `write_row_styled()`: 56 MB → 55 MB
  - All methods stay well under 80 MB target ✅

### Fixed
- **🔧 CI/CD**: Fixed GitHub Actions clippy failures
  - Resolved `flate2` version conflicts with `zip` crate
  - Updated workflow to use `--features serde,parallel` instead of `--all-features`

## [0.6.1] - 2024-12-05

### Fixed
- **🎨 Cell Formatting Now Working**: Complete styles.xml implementation
  - Fixed empty styles.xml (was 100 bytes stub, now complete 2651 bytes)
  - Implemented full `write_row_styled()` with proper style attributes (`s="X"`)
  - All 14 CellStyle variants working: HeaderBold, NumberInteger, NumberDecimal, NumberCurrency, NumberPercentage, DateDefault, DateTimestamp, TextBold, TextItalic, HighlightYellow, HighlightGreen, HighlightRed, BorderThin
  - Standard ECMA-376 format IDs used (3, 4, 5, 9, 14, 22)
  - Complete fonts, fills, borders, and cellXfs definitions

### Changed
- **🔧 Unified Architecture**: Removed legacy FastWorkbook
  - Deleted `src/fast_writer/workbook.rs` (~974 lines)
  - Unified on single `UltraLowMemoryWorkbook` implementation
  - Updated all internal references and examples
  - Simplified API surface - less confusion
  - All 18 library tests passing after cleanup

### Security
- **🔒 Security Improvements**: Removed hardcoded credentials
  - Removed database connection strings from `postgres_streaming.rs`
  - Added `.env.example` with configuration templates
  - Updated `.gitignore` to prevent committing `.env` files
  - Updated documentation with security warnings
  - All PostgreSQL examples now require `DATABASE_URL` environment variable

### Improved
- **📚 Better Examples**: Consistent API usage
  - Updated `large_dataset_multi_sheet.rs` to use `ExcelWriter` instead of low-level API
  - Fixed `memory_constrained_write.rs` for new architecture
  - Updated `writers_comparison.rs` to reflect UltraLowMemoryWorkbook
  - All examples use consistent `ExcelWriter` API with `add_sheet()` method
  - Added security warnings to example documentation

### Technical Details
- Complete styles.xml with 3 fonts, 5 fills, 2 borders, 14 cellXfs
- Style indices (0-13) properly mapped to cell attributes
- Memory functions updated for UltraLowMemoryWorkbook compatibility
- No performance regression - all optimizations preserved

## [0.5.0] - 2024-12-04

### Added
- **🧠 Hybrid SST Optimization**: Intelligent selective deduplication for optimal memory usage
  - Numbers automatically written inline as `<c t="n">` (no SST overhead)
  - Long strings (>50 chars) written inline (usually unique anyway)
  - Short repeated strings use SST for efficient deduplication
  - SST capped at 100k unique strings for graceful degradation
  - Automatic optimization - no API changes required
- **📊 Dramatic Memory Reduction**: 89% less memory usage
  - Simple workbooks (5 cols, 1M rows): 49 MB → **18.8 MB** (62% reduction)
  - Medium workbooks (19 cols, 1M rows): 125 MB → **15.4 MB** (88% reduction)
  - Complex workbooks (50 cols, 100K rows): ~200 MB → **22.7 MB** (89% reduction)
  - Multi-workbook scenarios: 251 MB → **25.3 MB** (90% reduction)
- **⚡ Performance Boost**: 58% faster with hybrid SST
  - FastWorkbook: 25,682 rows/sec (was ~16,000 rows/sec)
  - Fewer SST lookups for numbers and long strings
  - Better cache efficiency for repeated short strings
- **📝 Technical Documentation**: 
  - Added `HYBRID_SST_OPTIMIZATION.md` with full strategy details
  - Performance comparison tables and benchmarks
  - Recommendations for different data types

### Changed
- `FastWorkbook` now uses hybrid SST by default (automatic)
- Replaced `SharedStringsV2` (disk-based) with optimized in-memory hybrid approach
- Updated all examples to demonstrate memory efficiency

### Removed
- Removed unused `shared_strings_v2.rs` (disk-based deduplication approach)

### Performance
- Memory: 15-25 MB for most workbooks (was 80-250 MB)
- Speed: 25K+ rows/sec (was 16K-19K rows/sec)
- File size: +10-15% vs full SST (acceptable tradeoff for 89% memory reduction)

## [0.2.2] - 2024-12-02

### Added
- **Formula Support**: Added `CellValue::Formula` variant for Excel formulas
  - Write formulas like `=SUM(A1:A10)`, `=A2+B2`, etc.
  - Formulas are preserved in Excel and calculated correctly
  - Example: `writer.write_row_typed(&[CellValue::Formula("=SUM(A1:A10)".to_string())])?;`
- **Improved Error Messages**: Better context for errors
  - `SheetNotFound` now includes list of available sheets
  - `WriteRowError` includes row number and sheet name
  - Easier debugging with detailed error information
- **Comprehensive Tests**: Added 6 new integration tests
  - Special characters handling (XML, emojis, unicode)
  - Empty string and cell handling
  - Very long strings (10KB+)
  - Unicode sheet names (Russian, Chinese, French)
  - Error message validation
  - Formula support validation
- **CI/CD**: GitHub Actions workflow for automated testing
  - Automated format checking with `cargo fmt`
  - Clippy linting with strict warnings
  - Full test suite on every push/PR
  - Automated publishing to crates.io on version tags
- **Contributing Guide**: Comprehensive CONTRIBUTING.md with guidelines

### Changed
- Updated all error handling to provide better context
- Improved documentation with more examples
- Better type annotations in public APIs

### Fixed
- Fixed all clippy warnings (11 total)
- Fixed error pattern matching for sheet not found errors
- Corrected package name in documentation (was "rust-excelize")

### Changed
- Updated documentation to reflect current implementation
- Removed outdated migration warnings
- Clarified v0.2.x features and capabilities

## [0.2.0] - 2024-11-25

### Added
- **Streaming with constant memory**: ~80MB regardless of dataset size
- **21-47% faster** than rust_xlsxwriter baseline
- Custom `FastWorkbook` implementation for high-performance writing
- `set_flush_interval()` - Configure flush frequency
- `set_max_buffer_size()` - Configure buffer size
- Memory-constrained writing for Kubernetes pods

### Changed
- **BREAKING**: Removed `rust_xlsxwriter` dependency
- **BREAKING**: `ExcelWriter` now uses `FastWorkbook` internally
- All writes now stream directly to disk
- Memory usage reduced from ~300MB to ~80MB for large datasets

### Removed
- **BREAKING**: Bold header formatting (will be restored in future version)
- **BREAKING**: `set_column_width()` now a no-op (will be restored in future version)

## [0.1.0] - 2024-11-01

### Added
- Initial release
- Excel reading with streaming support (XLSX, XLS, ODS)
- Excel writing with `rust_xlsxwriter`
- Multi-sheet support
- Typed cell values
- PostgreSQL integration examples
- Basic examples and documentation

[0.2.2]: https://github.com/KSD-CO/excelstream/compare/v0.2.0...v0.2.2
[0.2.0]: https://github.com/KSD-CO/excelstream/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/KSD-CO/excelstream/releases/tag/v0.1.0
