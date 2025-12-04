//! Bounded-memory compression helper for large files

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use crate::error::Result;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Compress a file into ZIP with bounded memory (chunked reading)
pub fn compress_file_bounded<W: Write>(
    zip: &mut ZipWriter<W>,
    source_path: &str,
    zip_path: &str,
    options: FileOptions,
    chunk_size: usize,
) -> Result<()> {
    zip.start_file(zip_path, options)?;
    
    let file = File::open(source_path)?;
    let mut reader = BufReader::with_capacity(chunk_size, file);
    let mut buffer = vec![0u8; chunk_size];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        zip.write_all(&buffer[..bytes_read])?;
    }
    
    Ok(())
}

/// Options for bounded compression
pub fn bounded_compress_options() -> FileOptions {
    FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(1))  // Minimal compression
        .large_file(true)
}
