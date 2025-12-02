# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
