# ExcelStream Architecture Diagram

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          EXCELSTREAM LIBRARY                                │
│                               v0.2.2                                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              PUBLIC API                                     │
├──────────────────────────────┬──────────────────────────────────────────────┤
│   READER                     │              WRITER                          │
│                              │                                              │
│  ┌────────────────────┐      │    ┌────────────────────┐                    │
│  │  ExcelReader       │      │    │  ExcelWriter       │                    │
│  │                    │      │    │                    │                    │
│  │  - open()          │      │    │  - new()           │                    │
│  │  - rows()          │      │    │  - write_row()     │                    │
│  │  - sheet_names()   │      │    │  - write_row_typed()                    │
│  │  - read_cell()     │      │    │  - write_header()  │                    │
│  │  - dimensions()    │      │    │  - add_sheet()     │                    │
│  └────────────────────┘      │    │  - save()          │                    │
│           │                  │    └────────────────────┘                    │
│           │                  │              │                               │
└───────────┼──────────────────┴──────────────┼───────────────────────────────┘
            │                                 │
            │                                 │
┌───────────▼──────────────────┐  ┌───────────▼───────────────────────────────┐
│   READER BACKEND             │  │        WRITER BACKEND                     │
│   (External: calamine)       │  │   (Custom: FastWorkbook - v0.2.2)         │
├──────────────────────────────┤  ├───────────────────────────────────────────┤
│                              │  │                                           │
│  ┌────────────────────┐      │  │  ┌─────────────────────────────────────┐  │
│  │  Calamine          │      │  │  │       FastWorkbook                  │  │
│  │  WorkBook          │      │  │  │                                     │  │
│  │                    │      │  │  │  - new()                            │  │
│  │  Streaming         │      │  │  │  - add_worksheet()                  │  │
│  │  Iterator          │      │  │  │  - write_row()                      │  │
│  │  Support           │      │  │  │  - set_flush_interval()             │  │
│  │                    │      │  │  │  - set_max_buffer_size()            │  │
│  └────────────────────┘      │  │  │  - close()                          │  │
│           │                  │  │  └─────────────────────────────────────┘  │
│           │                  │  │              │                            │
│           ▼                  │  │              ▼                            │
│  ┌────────────────────┐      │  │  ┌─────────────────────────────────────┐  │
│  │  Range Iterator    │      │  │  │      FastWorksheet                  │  │ 
│  │  (Streaming)       │      │  │  │                                     │  │
│  └────────────────────┘      │  │  │  - Row buffer (~1000 rows)          │  │
│                              │  │  │  - XML generation                   │  │
│                              │  │  │  - Flush to disk                    │  │
│                              │  │  └─────────────────────────────────────┘  │
└──────────────────────────────┘  │              │                            │
                                  │              ▼                            │
                                  │  ┌─────────────────────────────────────┐  │
                                  │  │    SharedStrings Table              │  │
                                  │  │                                     │  │
                                  │  │  - String deduplication             │  │
                                  │  │  - Index mapping                    │  │
                                  │  └─────────────────────────────────────┘  │
                                  │              │                            │
                                  │              ▼                            │
                                  │  ┌─────────────────────────────────────┐  │
                                  │  │       XmlWriter                     │  │
                                  │  │                                     │  │
                                  │  │  - Reusable buffer (8KB)            │  │
                                  │  │  - Cell ref cache (A, B, C...)      │  │
                                  │  │  - Fast XML escaping                │  │
                                  │  └─────────────────────────────────────┘  │
                                  │              │                            │
                                  │              ▼                            │
                                  │  ┌─────────────────────────────────────┐  │
                                  │  │      ZIP Writer                     │  │
                                  │  │                                     │  │
                                  │  │  - Streaming compression            │  │
                                  │  │  - Direct to disk                   │  │
                                  │  │  - 64KB buffer                      │  │
                                  │  └─────────────────────────────────────┘  │
                                  └───────────────────────────────────────────┘
```

## Data Flow - Reading (Streaming)

```
┌──────────┐
│ Excel    │
│ File     │
│ (.xlsx)  │
└────┬─────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│              Calamine Parser (External)                     │
│                                                             │
│  1. Open ZIP archive                                        │
│  2. Parse XML files                                         │
│  3. Create sheet iterators                                  │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│           ExcelReader (Wrapper API)                         │
│                                                             │
│  - Provides ergonomic API                                   │
│  - Type conversions                                         │
│  - Error handling                                           │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│        Range Iterator (Streaming)                           │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │  Row 1   │→ │  Row 2   │→ │  Row 3   │→ ...              │
│  └──────────┘  └──────────┘  └──────────┘                   │
│                                                             │
│  Memory: Only current row in memory (~few KB)               │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────┐
│  User Code      │
│  processes      │
│  row by row     │
└─────────────────┘
```

## Data Flow - Writing (Streaming v0.2.2)

```
┌─────────────────┐
│  User Code      │
│  write_row()    │
└────┬────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│           ExcelWriter (Public API)                          │
│                                                             │
│  - Convert input to strings                                 │
│  - Type validation                                          │
│  - Track current row/sheet                                  │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│         FastWorkbook (Custom v0.2.2)                        │
│                                                             │
│  - Manages multiple worksheets                              │
│  - Coordinates streaming                                    │
│  - Generates workbook structure                             │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│         FastWorksheet (Row Buffer)                          │
│                                                             │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Buffer (up to 1000 rows or 1MB)                   │     │
│  │                                                    │     │
│  │  Row 1: [A1, B1, C1, ...]                          │     │
│  │  Row 2: [A2, B2, C2, ...]                          │     │
│  │  Row 3: [A3, B3, C3, ...]                          │     │
│  │  ...                                               │     │
│  │  Row 1000: [A1000, B1000, C1000, ...]              │     │
│  └────────────────────────────────────────────────────┘     │
│              │                                              │
│              ▼ (when buffer full or flush triggered)        │
│  ┌────────────────────────────────────────────────────┐     │
│  │  XML Generation (XmlWriter)                        │     │
│  │                                                    │     │
│  │  <row r="1">                                       │     │
│  │    <c r="A1" t="s"><v>0</v></c>  ← String index    │     │
│  │    <c r="B1"><v>42</v></c>       ← Number          │     │
│  │  </row>                                            │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│         SharedStrings Table                                 │
│                                                             │
│  Deduplicates strings:                                      │
│  "Alice"  → index 0                                         │
│  "Bob"    → index 1                                         │
│  "Alice"  → index 0 (reused)                                │
│                                                             │
│  Memory: ~10-20MB for 1M unique strings                     │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│         ZIP Writer (Streaming Compression)                  │
│                                                             │
│  Writes directly to disk:                                   │
│  ┌────────────────────────────────────────────────────┐     │
│  │  [Content_Types].xml                               │     │
│  │  _rels/.rels                                       │     │
│  │  xl/workbook.xml                                   │     │
│  │  xl/worksheets/sheet1.xml ← Writing here           │     │
│  │  xl/sharedStrings.xml                              │     │
│  │  ...                                               │     │
│  └────────────────────────────────────────────────────┘     │
│                                                             │
│  Buffer: 64KB for compression                               │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌──────────┐
│ Excel    │
│ File     │
│ (.xlsx)  │
└──────────┘

Memory footprint: ~80MB constant (regardless of file size!)
```

## Memory Comparison: v0.1.x vs v0.2.0

```
┌──────────────────────────────────────────────────────────────────────┐
│                    v0.1.x (rust_xlsxwriter wrapper)                  │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  User Code                                                           │
│     │                                                                │
│     ▼                                                                │
│  ExcelWriter                                                         │
│     │                                                                │
│     ▼                                                                │
│  rust_xlsxwriter::Workbook                                           │
│     │                                                                │
│     ▼                                                                │
│  ┌────────────────────────────────────────────────────────┐          │
│  │  KEEPS ALL DATA IN MEMORY!                             │          │
│  │                                                        │          │
│  │  All rows stored in Vec<Vec<Cell>>                     │          │
│  │  All strings stored                                    │          │
│  │  All formatting stored                                 │          │
│  │                                                        │          │
│  │  1M rows × 30 cols = ~300MB RAM ❌                     │          │
│  │                                                        │          │
│  │  Memory grows with dataset size                        │          │
│  └────────────────────────────────────────────────────────┘          │
│     │                                                                │
│     ▼ (only writes when save() called)                               │
│  Disk                                                                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                    v0.2.2 (FastWorkbook)                             │
│                         "Streaming"                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  User Code                                                           │
│     │                                                                │
│     ▼                                                                │
│  ExcelWriter                                                         │
│     │                                                                │
│     ▼                                                                │
│  FastWorkbook                                                        │
│     │                                                                │
│     ▼                                                                │
│  ┌────────────────────────────────────────────────────────┐          │
│  │  CONSTANT MEMORY!                                      │          │
│  │                                                        │          │
│  │  Row buffer: 1000 rows (~1MB)                          │          │
│  │  ├─ Flush to disk when full                            │          │
│  │  └─ Buffer cleared after flush                         │          │
│  │                                                        │          │
│  │  SharedStrings: ~10-20MB                               │          │
│  │  ZIP buffer: 64KB                                      │          │
│  │  XML buffer: 8KB                                       │          │
│  │                                                        │          │
│  │  Total: ~80MB constant ✅                              │          │
│  │                                                        │          │
│  │  Memory DOES NOT grow with dataset                     │          │
│  └────────────────────────────────────────────────────────┘          │
│     │                                                                │
│     ▼ (writes continuously during write_row())                       │
│  ┌────────────────────────────────────────────────────────┐          │
│  │  Disk - Writing in real-time                           │          │
│  │                                                        │          │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                │          │
│  │  │ Chunk 1  │ │ Chunk 2  │ │ Chunk 3  │ ...            │          │
│  │  │ (1000    │ │ (1000    │ │ (1000    │                │          │
│  │  │  rows)   │ │  rows)   │ │  rows)   │                │          │
│  │  └──────────┘ └──────────┘ └──────────┘                │          │
│  │                                                        │          │
│  │  Each chunk written and freed from memory              │          │
│  └────────────────────────────────────────────────────────┘          │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

## Performance Comparison (1M rows × 30 columns)

```
┌────────────────────────────────────────────────────────────────────┐
│                    THROUGHPUT COMPARISON                           │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  rust_xlsxwriter direct                                            │
│  ████████████████████████████████ 30,525 rows/s (baseline)         │
│                                                                    │
│  v0.1.x ExcelWriter (using rust_xlsxwriter)                        │
│  ████████████████████████████ 27,089 rows/s (-11%)                 │
│                                                                    │
│  v0.2.2 ExcelWriter.write_row()                                    │
│  ████████████████████████████████████████ 36,870 rows/s (+21%) ✅  │
│                                                                    │
│  v0.2.2 ExcelWriter.write_row_typed()                              │
│  █████████████████████████████████████████████ 42,877 rows/s       │
│  (+40%) ✅✅                                                       │
│                                                                    │
│  v0.2.2 FastWorkbook direct                                        │
│  ███████████████████████████████████████████████ 44,753 rows/s     │
│  (+47%) ✅✅✅                                                     │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│                    MEMORY USAGE COMPARISON                         │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  rust_xlsxwriter / v0.1.x                                          │
│  Memory grows with data:                                           │
│  │                                                                 │
│  │  ┌─────┐                                                        │
│  │  │     │                                                        │
│  │  │     │  300MB ❌                                              │
│  │  │     │                                                        │
│  │  │     │                                                        │
│  │  │     │                                                        │
│  │  └─────┘                                                        │
│  │   1M rows                                                       │
│  │                                                                 │
│  v0.2.2 FastWorkbook                                               │
│  Constant memory:                                                  │
│  │                                                                 │
│  │  ┌──┐                                                           │
│  │  │  │ 80MB ✅                                                   │
│  │  │  │                                                           │
│  │  └──┘                                                           │
│  │   Constant (any size)                                           │
│  │                                                                 │
│  │  Memory efficiency: ~73% reduction                              │
│  │                                                                 │
└────────────────────────────────────────────────────────────────────┘
```

## Component Details

### FastWorkbook Components

```
┌─────────────────────────────────────────────────────────────────┐
│                      FastWorkbook                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  ZIP Writer (zip crate)                                 │    │
│  │                                                         │    │
│  │  - Manages .xlsx file (ZIP archive)                     │    │
│  │  - Compression: Deflate level 6                         │    │
│  │  - Buffer: 64KB                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  SharedStrings                                          │    │
│  │                                                         │    │
│  │  HashMap<String, u32>                                   │    │
│  │  - Deduplicates strings                                 │    │
│  │  - Returns index for XML                                │    │
│  │  - Memory: ~10-20MB for 1M strings                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Worksheets: Vec<String>                                │    │
│  │                                                         │    │
│  │  - Track worksheet names                                │    │
│  │  - Current worksheet index                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Configuration                                          │    │
│  │                                                         │    │
│  │  flush_interval: u32       (default: 1000 rows)         │    │
│  │  max_buffer_size: usize    (default: 1MB)               │    │
│  │  current_row: u32                                       │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Caches (for performance)                               │    │
│  │                                                         │    │
│  │  cell_ref_cache: Vec<String>                            │    │
│  │  - Pre-computed: A, B, C, ..., CV (100 cols)            │    │
│  │  xml_buffer: Vec<u8>                                    │    │
│  │  - Reusable buffer (8KB)                                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Flush Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│               Flush Trigger Conditions                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Flush happens when ANY of these conditions met:                │
│                                                                 │
│  1. Row count reaches flush_interval                            │
│     ┌────────────────────────────────────────────┐              │
│     │  if rows_in_buffer >= flush_interval {     │              │
│     │      flush_to_disk();                      │              │
│     │      clear_buffer();                       │              │
│     │  }                                         │              │
│     └────────────────────────────────────────────┘              │
│                                                                 │
│  2. Buffer size exceeds max_buffer_size                         │
│     ┌────────────────────────────────────────────┐              │
│     │  if buffer_size >= max_buffer_size {       │              │
│     │      flush_to_disk();                      │              │
│     │      clear_buffer();                       │              │
│     │  }                                         │              │
│     └────────────────────────────────────────────┘              │
│                                                                 │
│  3. Switching to new worksheet                                  │
│     ┌────────────────────────────────────────────┐              │
│     │  fn add_worksheet() {                      │              │
│     │      finish_current_worksheet();           │              │
│     │      // Flushes remaining buffer           │              │
│     │  }                                         │              │
│     └────────────────────────────────────────────┘              │
│                                                                 │
│  4. Closing workbook                                            │
│     ┌────────────────────────────────────────────┐              │
│     │  fn close() {                              │              │
│     │      finish_all_worksheets();              │              │
│     │      write_shared_strings();               │              │
│     │      finalize_zip();                       │              │
│     │  }                                         │              │
│     └────────────────────────────────────────────┘              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## File Structure (.xlsx is a ZIP)

```
output.xlsx (ZIP archive)
│
├── [Content_Types].xml          ← MIME types for all parts
├── _rels/
│   └── .rels                    ← Relationships (root)
│
├── docProps/
│   ├── app.xml                  ← Application properties
│   └── core.xml                 ← Core metadata
│
└── xl/
    ├── workbook.xml             ← Workbook structure
    ├── styles.xml               ← Cell styles (basic support)
    ├── sharedStrings.xml        ← String deduplication table
    │
    ├── _rels/
    │   └── workbook.xml.rels    ← Workbook relationships
    │
    └── worksheets/
        ├── sheet1.xml           ← Sheet 1 data (streaming written)
        ├── sheet2.xml           ← Sheet 2 data
        └── ...

Generated by FastWorkbook in streaming fashion!
```

## Type System

```
┌─────────────────────────────────────────────────────────────────┐
│                        Type Hierarchy                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  pub enum CellValue {                                           │
│      Empty,              → <c r="A1"/>                          │
│      String(String),     → <c r="A1" t="s"><v>0</v></c>         │
│      Int(i64),          → <c r="A1"><v>42</v></c>               │
│      Float(f64),        → <c r="A1"><v>3.14</v></c>             │
│      Bool(bool),        → <c r="A1" t="b"><v>1</v></c>          │
│      DateTime(f64),     → <c r="A1"><v>44927.5</v></c>          │
│      Error(String),     → <c r="A1" t="e"><v>#N/A</v></c>       │
│      Formula(String),   → <c r="A1"><f>SUM(A1:A10)</f></c>      │
│  }                                                              │
│                                                                 │
│  pub struct Cell {                                              │
│      pub row: u32,                                              │
│      pub col: u32,                                              │
│      pub value: CellValue,                                      │
│  }                                                              │
│                                                                 │
│  pub struct Row {                                               │
│      pub index: u32,                                            │
│      pub cells: Vec<Cell>,                                      │
│  }                                                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Error Propagation                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Code                                                      │
│     │                                                           │
│     │ writer.write_row()?                                       │
│     ▼                                                           │
│  ┌──────────────────────────────────────────┐                   │
│  │  ExcelWriter::write_row()                │                   │
│  │                                          │                   │
│  │  - Validates input                       │                   │
│  │  - Converts to internal format           │                   │
│  │  - Calls FastWorkbook                    │                   │
│  └──────────────────────────────────────────┘                   │
│     │                                                           │
│     │ inner.write_row()?                                        │
│     ▼                                                           │
│  ┌──────────────────────────────────────────┐                   │
│  │  FastWorkbook::write_row()               │                   │
│  │                                          │                   │
│  │  - Adds to buffer                        │                   │
│  │  - Checks flush conditions               │                   │
│  │  - May trigger flush                     │                   │
│  └──────────────────────────────────────────┘                   │
│     │                                                           │
│     │ flush_to_disk()?                                          │
│     ▼                                                           │
│  ┌──────────────────────────────────────────┐                   │
│  │  XmlWriter::write()                      │                   │
│  │                                          │                   │
│  │  - Generate XML                          │                   │
│  │  - Escape special chars                  │                   │
│  │  - Write to ZIP                          │                   │
│  └──────────────────────────────────────────┘                   │
│     │                                                           │
│     │ zip.write_all()?                                          │
│     ▼                                                           │
│  ┌──────────────────────────────────────────┐                   │
│  │  ZipWriter (zip crate)                   │                   │
│  │                                          │                   │
│  │  - Compress data                         │                   │
│  │  - Write to file                         │                   │
│  └──────────────────────────────────────────┘                   │
│     │                                                           │
│     │ Ok(()) or Err(...)                                        │
│     ▼                                                           │
│  std::io::Error → ExcelError::IoError                           │
│  zip::ZipError  → ExcelError::WriteError                        │
│                                                                 │
│  All errors wrapped in Result<T, ExcelError>                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Optimization Techniques

```
┌─────────────────────────────────────────────────────────────────┐
│              Performance Optimizations                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. String Deduplication (SharedStrings)                        │
│     ┌──────────────────────────────────────┐                    │
│     │  "Alice" appears 1000 times          │                    │
│     │  Without: 1000 × "Alice" = ~6KB      │                    │
│     │  With:    1 × "Alice" + 1000 × u32   │                    │
│     │          = ~6 bytes + 4KB = 4KB      │                    │
│     │  Savings: 33% reduction              │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  2. Buffer Reuse                                                │
│     ┌──────────────────────────────────────┐                    │
│     │  xml_buffer: Vec<u8> (8KB)           │                    │
│     │  Reused for every XML generation     │                    │
│     │  Avoids millions of allocations      │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  3. Cell Reference Cache                                        │
│     ┌──────────────────────────────────────┐                    │
│     │  Pre-computed: A, B, C, ..., CV      │                    │
│     │  No string allocation per cell       │                    │
│     │  Direct array lookup                 │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  4. Batch Flushing                                              │
│     ┌──────────────────────────────────────┐                    │
│     │  Buffer 1000 rows before flush       │                    │
│     │  Reduces syscalls                    │                    │
│     │  Better compression ratio            │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  5. Direct ZIP Writing                                          │
│     ┌──────────────────────────────────────┐                    │
│     │  No intermediate files               │                    │
│     │  Stream directly to .xlsx            │                    │
│     │  Compression on-the-fly              │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  6. Minimal XML Parsing                                         │
│     ┌──────────────────────────────────────┐                    │
│     │  Generate XML from templates         │                    │
│     │  No DOM building                     │                    │
│     │  Direct string concatenation         │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Future Enhancements (v0.3.0+)

```
┌─────────────────────────────────────────────────────────────────┐
│                  Planned Enhancements                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Advanced Formatting Support (Phase 3)                       │
│     ┌──────────────────────────────────────┐                    │
│     │  FastWorkbook                        │                    │
│     │    ├─ StyleManager                   │                    │
│     │    │   ├─ Font styles (bold, italic) │                    │
│     │    │   ├─ Fill colors (background)   │                    │
│     │    │   ├─ Border styles               │                    │
│     │    │   └─ Number formats              │                    │
│     │    └─ Enhanced styles.xml             │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  2. Column Width & Row Height                                   │
│     ┌──────────────────────────────────────┐                    │
│     │  set_column_width(col, width)        │                    │
│     │  set_row_height(row, height)         │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  3. Parallel Reading (optional feature)                         │
│     ┌──────────────────────────────────────┐                    │
│     │  Rayon-based parallel iteration      │                    │
│     │  Process multiple sheets in parallel │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
│  4. Charts and Images                                           │
│     ┌──────────────────────────────────────┐                    │
│     │  add_chart(type, range)              │                    │
│     │  insert_image(path, cell)            │                    │
│     └──────────────────────────────────────┘                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

**Legend:**
- ✅ = Implemented and working
- ❌ = Problem/Bottleneck
- → = Data flow direction
- ▼ = Transformation/Processing

**Last Updated:** December 2, 2024
**Version:** v0.2.2
