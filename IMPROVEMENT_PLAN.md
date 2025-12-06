# ExcelStream Improvement Plan

This document outlines the planned improvements for the excelstream library based on comprehensive code review.

## Current Status (v0.3.0)

**Strengths:**
- ✅ Excellent performance (30K-45K rows/sec throughput)
- ✅ Streaming with constant ~80MB memory usage
- ✅ Good test coverage (50+ tests: 16 integration + 21 doc + 19 unit)
- ✅ Comprehensive documentation
- ✅ Rich examples (23 examples)
- ✅ Formula support
- ✅ Cell formatting & styling (14 predefined styles)
- ✅ Context-rich error messages

**Completed Phases:**
- ✅ Phase 1 (v0.2.1): Code quality fixes, basic formatting
- ✅ Phase 2 (v0.2.2): Formula support, improved error messages
- ✅ Phase 3 (v0.3.0): Cell formatting & styling API

---

## PHASE 1 - Immediate Fixes (v0.2.1) ✅ COMPLETED

**Target: Fix critical code quality and add basic missing features**

### 1.1 Code Quality Fixes ✓

- [x] Fix unused `mut` in [worksheet.rs:227](src/fast_writer/worksheet.rs#L227)
- [x] Fix needless borrow in [reader.rs:71,104,121](src/reader.rs)
- [x] Fix unnecessary cast in [reader.rs:141](src/reader.rs#L141)
- [x] Fix needless borrows in writer.rs tests
- [x] Fix PI constant usage in [writer.rs:367](src/writer.rs#L367)

### 1.2 Documentation Fixes ✓

- [x] Fix package name in [lib.rs:1](src/lib.rs#L1) (rust-excelize → excelstream)

### 1.3 Error Handling Cleanup ✓

- [x] Remove unused `XlsxWriterError` variant from [error.rs](src/error.rs)
- [x] Clean up outdated error documentation

### 1.4 Basic Formatting Support

**Priority: HIGH**

- [x] Implement bold header formatting ✅ (Completed in Phase 3)
  - [x] Add `CellStyle` enum with style properties (bold, italic, colors, borders)
  - [x] Modify FastWorkbook to support styles.xml generation
  - [x] Add `write_header_bold()` to apply bold formatting

- [ ] Implement column width support ⏸️ (Deferred to Phase 4)
  - Add column width tracking to FastWorksheet
  - Generate proper `<col>` elements in worksheet XML
  - Make `set_column_width()` functional (currently no-op)

### 1.5 Testing

- [x] Verify all clippy warnings are resolved
- [x] Run full test suite
- [ ] Add tests for new formatting features

**Estimated Time:** 2-4 hours
**Complexity:** Low-Medium

---

## PHASE 2 - Short Term (v0.2.2) ✅ COMPLETED

**Target: Essential Excel features**

### 2.1 Formula Support ✅

- [x] Added `Formula(String)` variant to CellValue enum
- [x] Support for writing Excel formulas in cells
- [x] Formulas calculate correctly when opened in Excel
- [x] Example: `write_row_typed(&[CellValue::Formula("=SUM(A1:A10)".into())])`

### 2.2 Cell Merging

_Deferred to Phase 4 - Not essential for v0.2.2_

### 2.3 Improved Error Messages ✅

- [x] Added context-rich error messages with debugging info
- [x] Better error descriptions for common failures
- [x] Sheet validation errors with available sheets listed

### 2.4 Additional Tests ✅

- [x] Edge case tests (empty strings, special characters)
- [x] XML escaping tests
- [x] Formula tests
- [x] Integration tests for full workflows

### 2.5 Dependency Updates

- [x] Dependencies reviewed and up to date

---

## PHASE 3 - Medium Term (v0.3.0) ✅ COMPLETED

**Target: Cell Formatting & Styling**

### 3.1 Cell Formatting & Styling API ✅

Implemented **14 predefined cell styles** for common use cases:

```rust
pub enum CellStyle {
    Default,           // No formatting
    HeaderBold,        // Bold text for headers
    NumberInteger,     // #,##0
    NumberDecimal,     // #,##0.00
    NumberCurrency,    // $#,##0.00
    NumberPercentage,  // 0.00%
    DateDefault,       // MM/DD/YYYY
    DateTimestamp,     // MM/DD/YYYY HH:MM:SS
    TextBold,          // Bold emphasis
    TextItalic,        // Italic notes
    HighlightYellow,   // Yellow background
    HighlightGreen,    // Green background
    HighlightRed,      // Red background
    BorderThin,        // Thin borders
}

pub struct StyledCell {
    pub value: CellValue,
    pub style: CellStyle,
}

impl ExcelWriter {
    // Write row with styled cells
    pub fn write_row_styled(&mut self, cells: &[(CellValue, CellStyle)]) -> Result<()>;

    // Write header with bold formatting
    pub fn write_header_bold<I, S>(&mut self, headers: I) -> Result<()>;

    // Write row with uniform style
    pub fn write_row_with_style(&mut self, values: &[CellValue], style: CellStyle) -> Result<()>;
}
```

**Features:**
- [x] 14 predefined styles covering common use cases
- [x] Constant memory usage (~80MB) maintained
- [x] Complete styles.xml generation with fonts, fills, borders, number formats
- [x] Easy-to-use API with convenience methods
- [x] Full example in `examples/cell_formatting.rs`
- [x] Comprehensive documentation

**Design Decision:** Pre-defined styles approach chosen over dynamic style builder for v0.3.0:
- ✅ Simpler implementation
- ✅ Predictable memory usage
- ✅ Covers 80% of use cases
- ✅ Fast - no dynamic style tracking
- ✅ Easy to extend later with dynamic styles in v0.4.0+

### 3.2 Parallel Reading Support

_Deferred to Phase 4 - Focus on styling for v0.3.0_

### 3.3 Data Validation

_Deferred to Phase 4_

### 3.4 Ergonomic API Improvements

_Deferred to Phase 4_

### 3.5 Performance Optimizations

- [x] Maintained streaming performance with styling
- [x] No regression in write speeds
- [x] Memory stays constant ~80MB

---

## PHASE 4 - Long Term (v0.4.0+) 🔜 NEXT

**Target: Advanced Excel features**

**Priority Items for v0.4.0:**
1. Dynamic Style Builder (custom colors, fonts, combinations)
2. Cell Merging
3. Column Width & Row Height
4. Data Validation

### 4.1 Conditional Formatting

```rust
pub enum ConditionalFormat {
    ColorScale {
        min_color: Color,
        mid_color: Option<Color>,
        max_color: Color,
    },
    DataBar {
        color: Color,
        show_value: bool,
    },
    IconSet {
        icons: IconSetType,
        reverse: bool,
    },
    CellValue {
        operator: ComparisonOperator,
        value: CellValue,
        format: CellStyle,
    },
}

impl ExcelWriter {
    pub fn add_conditional_format(&mut self, range: &str,
                                   format: ConditionalFormat) -> Result<()>;
}
```

### 4.2 Charts

```rust
pub enum ChartType {
    Line,
    Column,
    Bar,
    Pie,
    Scatter,
    Area,
}

pub struct Chart {
    chart_type: ChartType,
    series: Vec<ChartSeries>,
    title: Option<String>,
    x_axis: AxisOptions,
    y_axis: AxisOptions,
}

impl ExcelWriter {
    pub fn insert_chart(&mut self, sheet: &str, row: u32, col: u32,
                        chart: &Chart) -> Result<()>;
}
```

### 4.3 Images

```rust
impl ExcelWriter {
    pub fn insert_image(&mut self, sheet: &str, row: u32, col: u32,
                        path: &str) -> Result<()>;
    pub fn insert_image_with_options(&mut self, sheet: &str, row: u32, col: u32,
                                      path: &str, options: ImageOptions) -> Result<()>;
}
```

### 4.4 Rich Text

```rust
pub struct RichText {
    runs: Vec<TextRun>,
}

pub struct TextRun {
    text: String,
    font: FontStyle,
}

impl ExcelWriter {
    pub fn write_rich_text(&mut self, row: u32, col: u32,
                           rich_text: &RichText) -> Result<()>;
}
```

### 4.5 Worksheet Protection

```rust
pub struct ProtectionOptions {
    pub password: Option<String>,
    pub select_locked_cells: bool,
    pub select_unlocked_cells: bool,
    pub format_cells: bool,
    pub format_columns: bool,
    pub format_rows: bool,
}

impl ExcelWriter {
    pub fn protect_sheet(&mut self, options: ProtectionOptions) -> Result<()>;
}
```

**Estimated Time:** 8-12 weeks
**Complexity:** Very High

---

## PHASE 5 - Repository & Publishing

### 5.1 CI/CD Setup

```yaml
# .github/workflows/ci.yml
- Automated testing on push/PR
- Clippy checks
- Format checks
- Benchmark tracking
- Documentation deployment
```

### 5.2 Additional Badges

```markdown
[![Crates.io](https://img.shields.io/crates/v/excelstream.svg)]
[![Documentation](https://docs.rs/excelstream/badge.svg)]
[![Downloads](https://img.shields.io/crates/d/excelstream.svg)]
[![CI](https://github.com/KSD-CO/excelstream/workflows/CI/badge.svg)]
```

### 5.3 Documentation Improvements

- [ ] Create CHANGELOG.md
- [ ] Add CONTRIBUTING.md guidelines
- [ ] API documentation examples
- [ ] Migration guides for major versions
- [ ] Performance tuning guide

### 5.4 Community

- [ ] Set up issue templates
- [ ] PR templates
- [ ] Code of conduct
- [ ] Security policy

**Estimated Time:** 1-2 weeks
**Complexity:** Low

---

## Testing Strategy

### Unit Tests
- Test each module independently
- Cover edge cases and error conditions
- Test public APIs

### Integration Tests
- Test full read/write workflows
- Test multi-sheet operations
- Test large dataset handling

### Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_arbitrary_data(rows: Vec<Vec<String>>) {
        // Write and read back, should match
    }
}
```

### Performance Tests
- Benchmark critical operations
- Memory usage tests
- Streaming validation tests

### Compatibility Tests
- Test Excel compatibility
- Test LibreOffice compatibility
- Test different Excel versions

---

## Performance Goals

### Current Performance (v0.3.0)
- ExcelWriter.write_row(): 36,870 rows/s
- ExcelWriter.write_row_typed(): 42,877 rows/s
- ExcelWriter.write_row_styled(): ~42,000 rows/s (< 5% overhead)
- FastWorkbook direct: 44,753 rows/s
- Memory: ~80MB constant (with styling)

### Target Performance (v0.4.0+)
- Maintain or improve write speeds
- Keep memory usage under 100MB for streaming
- Parallel reading: 2-4x speedup on multi-core systems
- Zero-copy optimizations where possible
- Dynamic style builder with < 10% overhead

---

## Breaking Changes Policy

### Semantic Versioning
- Patch (0.2.x): Bug fixes, no API changes
- Minor (0.x.0): New features, backward compatible
- Major (x.0.0): Breaking API changes

### Deprecation Strategy
- Deprecate old APIs in minor version
- Keep deprecated APIs for at least one minor version
- Document migration path clearly
- Remove in next major version

---

## Success Metrics

### Code Quality
- Zero clippy warnings with `-D warnings`
- Test coverage > 80%
- All examples working
- Documentation for all public APIs

### Performance
- High throughput: 30K-45K rows/sec for all operations
- Memory usage stays constant (~80MB) for streaming
- No performance regressions

### Community
- GitHub stars growth
- crates.io downloads
- Issue response time < 48 hours
- Regular releases (monthly for active development)

---

## Dependencies Philosophy

### Core Dependencies (minimal)
- calamine: Excel reading
- zip: ZIP compression
- thiserror: Error handling

### Optional Dependencies
- serde: Serialization support
- rayon: Parallel processing
- chrono: Date/time handling (for examples)

### Dev Dependencies
- tempfile: Testing
- criterion: Benchmarking
- proptest: Property-based testing

---

## Notes

- Maintain backward compatibility within minor versions
- Keep streaming as the core feature
- Performance is a key differentiator
- Memory efficiency is non-negotiable
- Excel compatibility must be validated
- Documentation is as important as code

---

**Last Updated:** 2024-12-03
**Version:** v0.3.0
**Next Milestone:** v0.4.0 (Phase 4 - Advanced features: dynamic styles, cell merging, charts)
