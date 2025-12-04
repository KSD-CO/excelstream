# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- **TRUE Streaming with constant memory**: ~80MB regardless of dataset size
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
