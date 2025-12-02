//! Fast worksheet implementation with optimized row writing

use std::io::Write;
use crate::error::Result;
use super::xml_writer::XmlWriter;
use super::shared_strings::SharedStrings;

/// Cell reference generator
struct CellRef {
    row: u32,
    col: u32,
}

impl CellRef {
    fn new() -> Self {
        CellRef { row: 1, col: 0 }
    }

    fn next_cell(&mut self) -> String {
        self.col += 1;
        self.to_cell_ref(self.row, self.col)
    }

    fn next_row(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    fn to_cell_ref(&self, row: u32, col: u32) -> String {
        let mut col_str = String::new();
        let mut n = col;
        while n > 0 {
            let rem = (n - 1) % 26;
            col_str.insert(0, (b'A' + rem as u8) as char);
            n = (n - 1) / 26;
        }
        format!("{}{}", col_str, row)
    }
}

/// Fast worksheet writer
pub struct FastWorksheet<W: Write> {
    xml_writer: XmlWriter<W>,
    shared_strings: SharedStrings,
    cell_ref: CellRef,
    row_count: u32,
}

impl<W: Write> FastWorksheet<W> {
    pub fn new(writer: W, shared_strings: SharedStrings) -> Result<Self> {
        let mut xml_writer = XmlWriter::new(writer);
        
        // Write XML header
        xml_writer.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n")?;
        
        // Start worksheet
        xml_writer.start_element("worksheet")?;
        xml_writer.attribute("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main")?;
        xml_writer.attribute("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships")?;
        xml_writer.close_start_tag()?;
        
        // Start sheetData
        xml_writer.start_element("sheetData")?;
        xml_writer.close_start_tag()?;

        Ok(FastWorksheet {
            xml_writer,
            shared_strings,
            cell_ref: CellRef::new(),
            row_count: 0,
        })
    }

    /// Write a row of string data
    pub fn write_row(&mut self, values: &[&str]) -> Result<()> {
        self.cell_ref.next_row();
        self.row_count += 1;

        // Start row element
        self.xml_writer.start_element("row")?;
        self.xml_writer.attribute_int("r", self.row_count as i64)?;
        self.xml_writer.close_start_tag()?;

        // Write cells
        for value in values {
            let cell_ref = self.cell_ref.next_cell();
            let string_index = self.shared_strings.add_string(value);

            self.xml_writer.start_element("c")?;
            self.xml_writer.attribute("r", &cell_ref)?;
            self.xml_writer.attribute("t", "s")?; // String type
            self.xml_writer.close_start_tag()?;

            self.xml_writer.start_element("v")?;
            self.xml_writer.close_start_tag()?;
            self.xml_writer.write_str(&string_index.to_string())?;
            self.xml_writer.end_element("v")?;

            self.xml_writer.end_element("c")?;
        }

        // End row
        self.xml_writer.end_element("row")?;
        Ok(())
    }

    /// Write a row of typed data
    pub fn write_row_typed(&mut self, values: &[crate::types::CellValue]) -> Result<()> {
        use crate::types::CellValue;
        
        self.cell_ref.next_row();
        self.row_count += 1;

        // Start row element
        self.xml_writer.start_element("row")?;
        self.xml_writer.attribute_int("r", self.row_count as i64)?;
        self.xml_writer.close_start_tag()?;

        // Write cells
        for value in values {
            let cell_ref = self.cell_ref.next_cell();
            
            match value {
                CellValue::Empty => {
                    // Skip empty cells
                }
                CellValue::String(s) => {
                    let string_index = self.shared_strings.add_string(s);
                    
                    self.xml_writer.start_element("c")?;
                    self.xml_writer.attribute("r", &cell_ref)?;
                    self.xml_writer.attribute("t", "s")?;
                    self.xml_writer.close_start_tag()?;
                    
                    self.xml_writer.start_element("v")?;
                    self.xml_writer.close_start_tag()?;
                    self.xml_writer.write_str(&string_index.to_string())?;
                    self.xml_writer.end_element("v")?;
                    
                    self.xml_writer.end_element("c")?;
                }
                CellValue::Int(n) => {
                    self.xml_writer.start_element("c")?;
                    self.xml_writer.attribute("r", &cell_ref)?;
                    self.xml_writer.close_start_tag()?;
                    
                    self.xml_writer.start_element("v")?;
                    self.xml_writer.close_start_tag()?;
                    self.xml_writer.write_str(&n.to_string())?;
                    self.xml_writer.end_element("v")?;
                    
                    self.xml_writer.end_element("c")?;
                }
                CellValue::Float(f) => {
                    self.xml_writer.start_element("c")?;
                    self.xml_writer.attribute("r", &cell_ref)?;
                    self.xml_writer.close_start_tag()?;
                    
                    self.xml_writer.start_element("v")?;
                    self.xml_writer.close_start_tag()?;
                    self.xml_writer.write_str(&f.to_string())?;
                    self.xml_writer.end_element("v")?;
                    
                    self.xml_writer.end_element("c")?;
                }
                CellValue::Bool(b) => {
                    self.xml_writer.start_element("c")?;
                    self.xml_writer.attribute("r", &cell_ref)?;
                    self.xml_writer.attribute("t", "b")?;
                    self.xml_writer.close_start_tag()?;
                    
                    self.xml_writer.start_element("v")?;
                    self.xml_writer.close_start_tag()?;
                    self.xml_writer.write_str(if *b { "1" } else { "0" })?;
                    self.xml_writer.end_element("v")?;
                    
                    self.xml_writer.end_element("c")?;
                }
                _ => {
                    // For DateTime and Error, convert to string for now
                    let s = format!("{:?}", value);
                    let string_index = self.shared_strings.add_string(&s);
                    
                    self.xml_writer.start_element("c")?;
                    self.xml_writer.attribute("r", &cell_ref)?;
                    self.xml_writer.attribute("t", "s")?;
                    self.xml_writer.close_start_tag()?;
                    
                    self.xml_writer.start_element("v")?;
                    self.xml_writer.close_start_tag()?;
                    self.xml_writer.write_str(&string_index.to_string())?;
                    self.xml_writer.end_element("v")?;
                    
                    self.xml_writer.end_element("c")?;
                }
            }
        }

        // End row
        self.xml_writer.end_element("row")?;
        Ok(())
    }

    /// Finish writing the worksheet
    pub fn finish(mut self) -> Result<SharedStrings> {
        // End sheetData
        self.xml_writer.end_element("sheetData")?;
        
        // End worksheet
        self.xml_writer.end_element("worksheet")?;
        
        self.xml_writer.flush()?;
        Ok(self.shared_strings)
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_ref() {
        let mut cell_ref = CellRef::new();
        assert_eq!(cell_ref.to_cell_ref(1, 1), "A1");
        assert_eq!(cell_ref.to_cell_ref(1, 26), "Z1");
        assert_eq!(cell_ref.to_cell_ref(1, 27), "AA1");
        assert_eq!(cell_ref.to_cell_ref(100, 1), "A100");
    }

    #[test]
    fn test_worksheet_write() {
        let mut output = Vec::new();
        let ss = SharedStrings::new();
        let mut ws = FastWorksheet::new(&mut output, ss).unwrap();
        
        ws.write_row(&["Name", "Age"]).unwrap();
        ws.write_row(&["Alice", "30"]).unwrap();
        
        let ss = ws.finish().unwrap();
        
        let xml = String::from_utf8(output).unwrap();
        assert!(xml.contains("<row r=\"1\">"));
        assert!(xml.contains("<row r=\"2\">"));
        assert_eq!(ss.count(), 4); // Name, Age, Alice, 30
    }
}
