use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, SeekFrom};
use memmap2::Mmap;
use zstd::Encoder;
use std::collections::HashSet;
use rand::Rng;

use bincode::{
    config::{standard, Configuration},
    encode_to_vec,
    Encode,
};
use gxhash::GxBuildHasher;
use xorf::{Filter, BinaryFuse8};

use crate::utils::{default_output_names_if_omitted, CHUNK_SIZE};

#[derive(Debug, Encode)]
struct FrameInfo {
    frame_offset: u64,
    frame_size: u64,
}

/// The main entry point for the "build" subcommand
/// - `input_path`: The path to the input log file
/// - `maybe_zst_path`: Optional .zst path from user
/// - `maybe_idx_path`: Optional .idx path from user
pub fn run_build(
    input_path: &str,
    maybe_zst_path: Option<&str>,
    maybe_idx_path: Option<&str>,
) -> io::Result<()> {
    // Determine default .zst/.idx if omitted
    let (zst_path, idx_path) = default_output_names_if_omitted(input_path, maybe_zst_path, maybe_idx_path);

    // 1) Memory-map the input
    let input_file = File::open(input_path)?;
    let metadata = input_file.metadata()?;
    let file_len = metadata.len() as usize;
    let mmap = unsafe { Mmap::map(&input_file)? };

    // 2) Open output files
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&zst_path)?;

    let mut index_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&idx_path)?;

    let bin_cfg: Configuration = standard();

    let mut start = 0;
    while start < file_len {
        // 3) Determine chunk boundaries (64MB, align to next newline)
        let end_candidate = (start + CHUNK_SIZE).min(file_len);
        let actual_end = if end_candidate < file_len {
            match find_line_boundary(&mmap[end_candidate..]) {
                Some(rel_pos) => end_candidate + rel_pos + 1,
                None => file_len,
            }
        } else {
            file_len
        };

        let chunk = &mmap[start..actual_end];

        // 4) Compress the chunk
        let frame_offset = output_file.seek(SeekFrom::Current(0))?;
        let frame_size = compress_and_write_chunk(chunk, &mut output_file)?;

        // 5) Build BinaryFuse filter
        let filter = build_binaryfuse_filter(chunk);

        // 6) Write FrameInfo + Filter to index
        let frame_info = FrameInfo {
            frame_offset,
            frame_size,
        };
        let frame_info_bytes =
            encode_to_vec(&frame_info, bin_cfg).expect("Failed to encode FrameInfo");
        index_file.write_all(&frame_info_bytes)?;

        let filter_bytes =
            encode_to_vec(&filter, bin_cfg).expect("Failed to encode BinaryFuse16");
        index_file.write_all(&filter_bytes)?;

        start = actual_end;
    }

    println!("Build complete. ZST: {:?} | IDX: {:?}", zst_path, idx_path);
    Ok(())
}

/// Find the first newline in `data`, returns its index
fn find_line_boundary(data: &[u8]) -> Option<usize> {
    data.iter().position(|&c| c == b'\n')
}

/// Compress a chunk using ZSTD, returns the size of the compressed frame
fn compress_and_write_chunk(chunk: &[u8], output: &mut File) -> io::Result<u64> {
    let start_pos = output.seek(SeekFrom::Current(0))?;
    let mut encoder = Encoder::new(output, 0)?; // Level=0 for speed
    encoder.write_all(chunk)?;
    let mut output_file = encoder.finish()?;
    let end_pos = output_file.seek(SeekFrom::Current(0))?;
    Ok(end_pos - start_pos)
}

/// Build a BinaryFuse8 filter for unique 3-byte triplets
fn build_binaryfuse_filter(chunk: &[u8]) -> BinaryFuse8 {
    let mut set = HashSet::with_hasher(GxBuildHasher::default());
    
    for window in chunk.windows(8) {
        /*
        let key = ((window[0] as u64) << 16)
            | ((window[1] as u64) << 8)
            | (window[2] as u64)
            | (window[3] as u64) << 24;
            */
        let key = u64::from_le_bytes(window.try_into().unwrap());
        set.insert(key);
    }

    let numkeys = set.len();
    let keys: Vec<u64> = set.into_iter().collect();
    let filter = BinaryFuse8::try_from(&keys[..]).expect("Failed to build BinaryFuse filter");

    
    let bpe = (filter.len() as f64) * 16.0 / (numkeys as f64);
    let mut rng = rand::thread_rng();
    let false_positives: usize = (0..numkeys)
        .map(|_| rng.gen::<u64>())
        .filter(|n| filter.contains(n))
        .count();
    let fp_rate: f64 = (false_positives * 100) as f64 / numkeys as f64;

    println!("set size: {}", numkeys);
    println!("bpe: {}", bpe);
    println!("fp rate: {}", fp_rate);
    
    filter
}
