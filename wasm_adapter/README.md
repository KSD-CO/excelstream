# excelstream_wasm

WebAssembly adapter for [excelstream](https://github.com/KSD-CO/excelstream) — parse CSV and XLSX files directly in the browser with near-native performance.

[![npm](https://img.shields.io/npm/v/excelstream_wasm)](https://www.npmjs.com/package/excelstream_wasm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

```bash
npm install excelstream_wasm
```

---

## Quick Start

```js
import init, { parse_csv_full } from 'excelstream_wasm';

// Initialize the WASM module first (required once)
await init();

const rows = parse_csv_full("name,age\nAlice,30\nBob,25");
console.log(rows);
// [["name", "age"], ["Alice", "30"], ["Bob", "25"]]
```

---

## CSV Parsing

### Parse a full CSV string

```js
import init, { parse_csv_full } from 'excelstream_wasm';

await init();

const csv = `id,name,score
1,Alice,95
2,Bob,87
3,Charlie,92`;

const rows = parse_csv_full(csv);
// rows[0] → ["id", "name", "score"]  (header)
// rows[1] → ["1", "Alice", "95"]
```

### Parse CSV from a file input

```js
import init, { parse_csv_full } from 'excelstream_wasm';

await init();

document.querySelector('#file-input').addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const text = await file.text();
  const rows = parse_csv_full(text);
  console.log(`Parsed ${rows.length} rows`);
});
```

### Parse CSV from URL (fetch)

```js
import init, { parse_csv_full } from 'excelstream_wasm';

await init();

const res = await fetch('/data/report.csv');
const text = await res.text();
const rows = parse_csv_full(text);
```

### Streaming CSV (line by line with callback)

Use this for large files or when you want to process rows as they arrive:

```js
import init, { init_parser, register_callback, feed_line } from 'excelstream_wasm';

await init();

// delimiter = 44 (','), quote = 34 ('"')
init_parser(44, 34);

const results = [];
register_callback((fields) => {
  results.push(fields);
});

// Feed lines one by one
feed_line("name,age,city");
feed_line("Alice,30,Hanoi");
feed_line("Bob,25,HCMC");

console.log(results);
// [["name","age","city"], ["Alice","30","Hanoi"], ["Bob","25","HCMC"]]
```

**Custom delimiters:**

```js
// Tab-separated values (TSV)
init_parser(9, 34);   // delimiter='\t', quote='"'

// Semicolon-separated (common in European locales)
init_parser(59, 34);  // delimiter=';', quote='"'
```

---

## XLSX Parsing

XLSX files are ZIP archives containing XML files. You need to unzip them first (e.g., with [fflate](https://github.com/101arrowz/fflate) or [JSZip](https://stuk.github.io/jszip/)), then pass the XML content to this library.

### Install a ZIP library

```bash
npm install fflate
```

### Parse XLSX from a file input

```js
import init, { load_shared_strings, parse_sheet_xml } from 'excelstream_wasm';
import { unzipSync, strFromU8 } from 'fflate';

await init();

document.querySelector('#file-input').addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const buffer = await file.arrayBuffer();
  const unzipped = unzipSync(new Uint8Array(buffer));

  // Load shared strings (string table used by XLSX)
  const sharedStringsXml = strFromU8(unzipped['xl/sharedStrings.xml'] ?? new Uint8Array());
  load_shared_strings(sharedStringsXml);

  // Parse the first sheet
  const sheetXml = strFromU8(unzipped['xl/worksheets/sheet1.xml']);
  const rows = parse_sheet_xml(sheetXml);

  console.log(`Parsed ${rows.length} rows`);
  console.log('Header:', rows[0]);
  console.log('First data row:', rows[1]);
});
```

### Parse XLSX from URL (fetch)

```js
import init, { load_shared_strings, parse_sheet_xml } from 'excelstream_wasm';
import { unzipSync, strFromU8 } from 'fflate';

await init();

const res = await fetch('/data/report.xlsx');
const buffer = await res.arrayBuffer();
const unzipped = unzipSync(new Uint8Array(buffer));

load_shared_strings(strFromU8(unzipped['xl/sharedStrings.xml'] ?? new Uint8Array()));
const rows = parse_sheet_xml(strFromU8(unzipped['xl/worksheets/sheet1.xml']));
```

### Parse multiple sheets

```js
// List available sheets from workbook.xml
const workbookXml = strFromU8(unzipped['xl/workbook.xml']);
const sheetNames = [...workbookXml.matchAll(/name="([^"]+)"/g)].map(m => m[1]);

// Parse sheet2
load_shared_strings(strFromU8(unzipped['xl/sharedStrings.xml'] ?? new Uint8Array()));
const sheet2Rows = parse_sheet_xml(strFromU8(unzipped['xl/worksheets/sheet2.xml']));
```

---

## Framework Examples

### React

```jsx
import { useState, useEffect } from 'react';
import init, { parse_csv_full } from 'excelstream_wasm';

export function CsvParser() {
  const [ready, setReady] = useState(false);
  const [rows, setRows] = useState([]);

  useEffect(() => {
    init().then(() => setReady(true));
  }, []);

  const handleFile = async (e) => {
    const text = await e.target.files[0].text();
    setRows(parse_csv_full(text));
  };

  return (
    <div>
      <input type="file" accept=".csv" onChange={handleFile} disabled={!ready} />
      <p>{rows.length} rows parsed</p>
    </div>
  );
}
```

### Vue 3

```vue
<script setup>
import { ref, onMounted } from 'vue';
import init, { parse_csv_full } from 'excelstream_wasm';

const rows = ref([]);
onMounted(() => init());

async function handleFile(e) {
  const text = await e.target.files[0].text();
  rows.value = parse_csv_full(text);
}
</script>

<template>
  <input type="file" accept=".csv" @change="handleFile" />
  <p>{{ rows.length }} rows parsed</p>
</template>
```

---

## API Reference

### `init()`

Initialize the WASM module. Must be called once before using any other function.

```js
await init();
```

---

### `parse_csv_full(contents: string): string[][]`

Parse a complete CSV string. Returns an array of rows, each row is an array of field strings.

| Parameter | Type | Description |
|-----------|------|-------------|
| `contents` | `string` | Full CSV content |

**Returns:** `string[][]` — array of rows

```js
const rows = parse_csv_full("a,b\n1,2");
// [["a","b"], ["1","2"]]
```

> Uses comma (`,`) as delimiter and double-quote (`"`) as quote character. For custom delimiters, use the streaming API.

---

### `init_parser(delimiter: number, quote: number)`

Initialize the streaming CSV parser with custom delimiter and quote characters.

| Parameter | Type | Description |
|-----------|------|-------------|
| `delimiter` | `number` | ASCII code of delimiter (e.g. `44` for `,`, `9` for tab, `59` for `;`) |
| `quote` | `number` | ASCII code of quote character (e.g. `34` for `"`) |

```js
init_parser(44, 34); // comma + double-quote
```

---

### `register_callback(fn: (fields: string[]) => void)`

Register a callback invoked for each line fed to the streaming parser.

```js
register_callback((fields) => {
  console.log(fields); // string[] for each parsed line
});
```

---

### `feed_line(line: string)`

Feed a single line to the streaming parser. The registered callback is called immediately.

```js
feed_line("Alice,30,Hanoi");
```

---

### `load_shared_strings(xml: string)`

Load the XLSX shared strings table (`xl/sharedStrings.xml`). Must be called before `parse_sheet_xml` if the sheet references shared strings.

| Parameter | Type | Description |
|-----------|------|-------------|
| `xml` | `string` | Content of `xl/sharedStrings.xml` |

```js
load_shared_strings(sharedStringsXmlContent);
```

---

### `parse_sheet_xml(xml: string): string[][]`

Parse an XLSX worksheet XML (`xl/worksheets/sheetN.xml`). Returns all rows as string arrays.

| Parameter | Type | Description |
|-----------|------|-------------|
| `xml` | `string` | Content of `xl/worksheets/sheet1.xml` |

**Returns:** `string[][]` — array of rows

```js
const rows = parse_sheet_xml(sheetXmlContent);
```

---

## Notes

- All cell values are returned as **strings**. Type conversion (number, date, boolean) is left to the caller.
- XLSX dates are stored as numeric serial values (e.g. `44927` = Jan 1 2023). Convert with: `new Date(Date.UTC(1899, 11, 30) + value * 86400000)`.
- The WASM binary is ~36 KB — suitable for browser use without bundling concerns.
- Supports all modern browsers and Node.js 18+.

---

## Related

- [excelstream (Rust)](https://crates.io/crates/excelstream) — Full-featured Rust library with S3, GCS, Parquet support
- [Source code](https://github.com/KSD-CO/excelstream)

## License

MIT
