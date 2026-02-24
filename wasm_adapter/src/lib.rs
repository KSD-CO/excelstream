//! Minimal WASM adapter exposing CSV parsing to JS for demo
#![allow(non_snake_case)]

use js_sys::Function;
use serde_wasm_bindgen::to_value;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

// Use a minimal local CSV parser to avoid pulling workspace native deps into the wasm build.
mod local_csv;
use local_csv::CsvParser;

thread_local! {
    static PARSER: RefCell<Option<CsvParser>> = RefCell::new(None);
    static CALLBACK: RefCell<Option<Function>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn init_parser(delimiter: u8, quote: u8) {
    PARSER.with(|p| {
        *p.borrow_mut() = Some(CsvParser::new(delimiter, quote));
    });
}

#[wasm_bindgen]
pub fn register_callback(cb: &Function) {
    CALLBACK.with(|c| {
        *c.borrow_mut() = Some(cb.clone());
    });
}

#[wasm_bindgen]
pub fn feed_line(line: &str) {
    PARSER.with(|p| {
        if let Some(parser) = &*p.borrow() {
            let fields = parser.parse_line(line);
            CALLBACK.with(|c| {
                if let Some(cb) = &*c.borrow() {
                    let jsv = to_value(&fields).unwrap_or_else(|_| JsValue::NULL);
                    let _ = cb.call1(&JsValue::NULL, &jsv);
                }
            });
        }
    });
}

#[wasm_bindgen]
pub fn parse_csv_full(contents: &str) -> JsValue {
    // Simple convenience: parse full CSV string into array of arrays
    let mut rows: Vec<Vec<String>> = Vec::new();
    let parser = CsvParser::new(b',', b'"');
    for line in contents.split('\n') {
        rows.push(parser.parse_line(line));
    }
    to_value(&rows).unwrap_or_else(|_| JsValue::NULL)
}

// --- XLSX (sheet XML + sharedStrings) helpers (simple, naive parser for demo) ---
thread_local! {
    static SHARED_STRINGS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

#[wasm_bindgen]
pub fn load_shared_strings(xml: &str) {
    let mut list = Vec::new();
    let mut start = 0usize;
    while let Some(si_start) = xml[start..].find("<si") {
        let si_pos = start + si_start;
        if let Some(si_end_rel) = xml[si_pos..].find("</si>") {
            let si_end = si_pos + si_end_rel + 5; // include </si>
            let si_block = &xml[si_pos..si_end];
            // find first <t>...</t> inside si_block
            if let Some(t_start) = si_block.find("<t") {
                if let Some(t_open) = si_block[t_start..].find('>') {
                    let vstart = t_start + t_open + 1;
                    if let Some(t_close_rel) = si_block[vstart..].find("</t>") {
                        let t_close = vstart + t_close_rel;
                        let val = &si_block[vstart..t_close];
                        list.push(xml_unescape(val));
                    }
                }
            }
            start = si_end;
        } else {
            break;
        }
    }
    SHARED_STRINGS.with(|ss| *ss.borrow_mut() = list);
}

#[wasm_bindgen]
pub fn parse_sheet_xml(xml: &str) -> JsValue {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut pos = 0usize;
    while let Some(row_start_rel) = xml[pos..].find("<row") {
        let row_start = pos + row_start_rel;
        if let Some(row_end_rel) = xml[row_start..].find("</row>") {
            let row_end = row_start + row_end_rel + 6; // include </row>
            let row_block = &xml[row_start..row_end];
            // find <c ...>...</c> cells
            let mut cells: Vec<String> = Vec::new();
            let mut cpos = 0usize;
            while let Some(c_start_rel) = row_block[cpos..].find("<c ") {
                let c_start = cpos + c_start_rel;
                if let Some(c_end_rel) = row_block[c_start..].find("</c>") {
                    let c_end = c_start + c_end_rel + 4;
                    let c_block = &row_block[c_start..c_end];
                    // detect type t="s" for shared string
                    let is_shared = c_block.contains(" t=\"s\"") || c_block.contains(" t='s'");
                    // extract <v>...</v>
                    let mut cell_text = String::new();
                    if let Some(v_rel) = c_block.find("<v>") {
                        let v_start = v_rel + 3;
                        if let Some(v_close_rel) = c_block[v_start..].find("</v>") {
                            let v_close = v_start + v_close_rel;
                            let val = &c_block[v_start..v_close];
                            if is_shared {
                                if let Ok(idx) = val.trim().parse::<usize>() {
                                    SHARED_STRINGS.with(|ss| {
                                        let arr = ss.borrow();
                                        if idx < arr.len() {
                                            cell_text = arr[idx].clone();
                                        }
                                    });
                                }
                            } else {
                                cell_text = xml_unescape(val.trim());
                            }
                        }
                    } else {
                        // maybe inlineStr <is><t>...
                        if let Some(is_rel) = c_block.find("<is>") {
                            if let Some(t_rel) = c_block[is_rel..].find("<t>") {
                                let t_start = is_rel + t_rel + 3;
                                if let Some(t_close_rel) = c_block[t_start..].find("</t>") {
                                    let t_close = t_start + t_close_rel;
                                    let val = &c_block[t_start..t_close];
                                    cell_text = xml_unescape(val);
                                }
                            }
                        }
                    }
                    cells.push(cell_text);
                    cpos = c_end;
                } else {
                    break;
                }
            }
            rows.push(cells);
            pos = row_end;
        } else {
            break;
        }
    }
    to_value(&rows).unwrap_or_else(|_| JsValue::NULL)
}
