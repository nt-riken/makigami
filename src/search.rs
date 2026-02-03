use std::io::{self, Write};
use std::io::Cursor;

use bincode::{config::standard, decode_from_std_read, Decode};
use xorf::{BinaryFuse8, Filter};

use crate::storage::{create_storage, StorageError};

/// Mirror the FrameInfo from build.rs for decoding
#[derive(Debug, Decode)]
struct FrameInfo {
    frame_offset: u64,
    frame_size: u64,
}

/// Main entry point for the "search" subcommand
/// - `zst_path`: .zst file (local path or gs://bucket/path)
/// - `maybe_idx_path`: optional index path (local path or gs://bucket/path)
/// - `pattern_str`: string pattern to search
pub fn run_search(
    zst_path: &str,
    maybe_idx_path: Option<&str>,
    pattern_str: &str,
) -> Result<(), StorageError> {
    // Create storage backend (local or GCS)
    let storage = create_storage(zst_path, maybe_idx_path)?;

    // Convert pattern to 8-byte windows (u64 keys)
    let pattern_bytes = pattern_str.as_bytes();
    let mut keys = Vec::new();
    for window in pattern_bytes.windows(8) {
        let key = u64::from_le_bytes(window.try_into().unwrap());
        keys.push(key);
    }

    // Fetch index file
    let index_data = storage.fetch_index()?;
    let mut cursor = Cursor::new(&index_data);
    let bin_cfg = standard();

    // We'll read FrameInfo + BinaryFuse8 pairs until EOF
    loop {
        // 1) Read FrameInfo
        let frame_info: FrameInfo = match decode_from_std_read(&mut cursor, bin_cfg) {
            Ok(fi) => fi,
            Err(_err) => {
                // Likely EOF
                break;
            }
        };
        
        // 2) Read BinaryFuse filter
        let filter: BinaryFuse8 = match decode_from_std_read(&mut cursor, bin_cfg) {
            Ok(flt) => flt,
            Err(_) => {
                // Likely EOF
                break;
            }
        };

        // 3) Check if chunk might contain all 8-byte windows
        let might_contain = keys.iter().all(|key| filter.contains(key));
        if might_contain {
            // Read the block from storage
            let compressed_chunk = storage.read_block(frame_info.frame_offset, frame_info.frame_size)?;

            // Decompress the chunk
            let decompressed = zstd::decode_all(&compressed_chunk[..])
                .map_err(|e| StorageError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Decompression failed: {}", e)
                )))?;

            // Output the decompressed data
            io::stdout().write_all(&decompressed)
                .map_err(|e| StorageError::Io(e))?;
        }
    }

    Ok(())
}
