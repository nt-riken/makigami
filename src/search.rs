use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write,self};
use std::path::PathBuf;

use bincode::{config::standard, decode_from_std_read, Decode};
use xorf::{BinaryFuse8,Filter};

use crate::utils::default_index_if_omitted;

/// Mirror the FrameInfo from build.rs for decoding
#[derive(Debug, Decode)]
struct FrameInfo {
    frame_offset: u64,
    frame_size: u64,
}

/// Main entry point for the "search" subcommand
/// - `zst_path`: .zst file
/// - `maybe_idx_path`: optional index path
/// - `pattern_str`: string pattern to search
pub fn run_search(
    zst_path: &str,
    maybe_idx_path: Option<&str>,
    pattern_str: &str,
) -> io::Result<()> {
    // Derive .idx if user didn't specify
    let idx_path: PathBuf = default_index_if_omitted(maybe_idx_path, zst_path);

    // Convert pattern to 3-byte windows (u64 keys)
    let pattern_bytes = pattern_str.as_bytes();
    let mut keys = Vec::new();
    for window in pattern_bytes.windows(3) {
        let key = ((window[0] as u64) << 16)
            | ((window[1] as u64) << 8)
            | (window[2] as u64);
        keys.push(key);
    }

    // Open .zst and .idx
    let mut idx_file = File::open(&idx_path)?;
    let mut zst_file = File::open(zst_path)?;

    let bin_cfg = standard();

    // We'll read FrameInfo + BinaryFuse8 pairs until EOF
    loop {
        // 1) Read FrameInfo
        let frame_info: FrameInfo = match decode_from_std_read(&mut idx_file, bin_cfg) {
            Ok(fi) => fi,
            Err(_err) => {
                // Likely EOF
                break;
            }
        };
        // 2) Read BinaryFuse filter
        let filter: BinaryFuse8 = match decode_from_std_read(&mut idx_file, bin_cfg) {
            Ok(flt) => flt,
            Err(_) => {
                // Likely EOF
                break;
            }
        };

        // 3) Check if chunk might contain all 3-byte windows
        let might_contain = keys.iter().all(|key| filter.contains(key));
        if might_contain {
            // Seek to the chunk in the .zst file
            zst_file.seek(SeekFrom::Start(frame_info.frame_offset))?;
            let mut compressed_chunk = vec![0u8; frame_info.frame_size as usize];
            zst_file.read_exact(&mut compressed_chunk)?;

            // Decompress the chunk
            let decompressed = zstd::decode_all(&compressed_chunk[..])?;

            // Here, you can do whatever you want with the decompressed text.
            // For demonstration, we just print it to STDOUT if it's a "candidate".
            // (In real usage, you might want line-based matching, etc.)
            io::stdout().write_all(&decompressed)?;
        }
    }

    Ok(())
}
