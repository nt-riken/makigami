use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

use std::sync::mpsc;

use memmap2::Mmap;
use zstd::Encoder;
use bincode::{
    config::{standard, Configuration},
    encode_to_vec,
    Encode,
};
use xorf::{BinaryFuse8, Filter};
use rand::Rng;

use num_cpus;

use crate::fastu64set::FastSet;

use crate::utils::{default_output_names_if_omitted, CHUNK_SIZE, HASH_CAPACITY};

#[derive(Debug, Encode)]
struct FrameInfo {
    frame_offset: u64,
    frame_size: u64,
}

/// A small struct carrying all data needed by the writer to finalize output.
struct ChunkResult {
    chunk_index: usize,
    /// The uncompressed chunk is optional if you want to do compression in the Writer thread
    /// or do it in the Worker thread. Shown here for illustration; you can store
    /// compressed bytes if you prefer compressing inside the worker.
    chunk_data: Vec<u8>,  
    filter: BinaryFuse8,
}

/// The main entry point for the "build" subcommand
pub fn run_build(
    input_path: &str,
    maybe_zst_path: Option<&str>,
    maybe_idx_path: Option<&str>,
) -> io::Result<()> {
    // 1) Figure out output paths
    let (zst_path, idx_path) = default_output_names_if_omitted(input_path, maybe_zst_path, maybe_idx_path);

    // 2) Open the input and memory-map
    let input_file = File::open(input_path)?;
    let metadata = input_file.metadata()?;
    let file_len = metadata.len() as usize;
    let mmap = unsafe { Mmap::map(&input_file)? };

    // 3) Set up output files (opened here but only written in the writer thread)
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&zst_path)?;

    let index_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&idx_path)?;

    // We'll share these file handles with the writer thread via Arc<Mutex<...>>
    let output_file = Arc::new(Mutex::new(output_file));
    let index_file = Arc::new(Mutex::new(index_file));

    // 4) Channels for pipeline
    //    - `chunk_sender`: main thread -> worker threads
    //    - `chunk_receiver`: worker threads -> writer thread
    let (chunk_sender, worker_rx) = mpsc::channel::<(usize, Vec<u8>)>();
    let (result_sender, chunk_receiver) = mpsc::channel::<ChunkResult>();

    // 5) Spawn some worker threads that build filters (the expensive part).
    //    You can tweak the number of threads as needed.
    let num_workers = num_cpus::get(); // or set to some fixed number

    let arc_rx = Arc::new(Mutex::new(worker_rx));
    println!("num_workers: {}", num_workers);
    for _ in 0..num_workers {
        let result_sender = result_sender.clone();
        let arc_rx = Arc::clone(&arc_rx);

        thread::spawn(move || {
            let rx = arc_rx.lock().unwrap();
            let mut set = FastSet::new(HASH_CAPACITY);

            while let Ok((chunk_index, chunk_data)) = rx.recv() {
                // Build the filter (expensive)
                let filter = build_binaryfuse_filter(&chunk_data, &mut set);
                println!("collision_count: {}", set.collision_count());

                // Possibly compress here, or return the raw chunk_data and let
                // the writer do the compression. In this example, we do NOT compress
                // in the worker. We just build the filter.

                // Send result to writer
                let res = ChunkResult {
                    chunk_index,
                    chunk_data,
                    filter,
                };
                if let Err(e) = result_sender.send(res) {
                    eprintln!("Worker->Writer channel send error: {:?}", e);
                    break;
                }
            }
        });
    };
    drop(result_sender);   // We’ll keep the one in the worker threads alive. This copy is not needed in main.

    // 6) Spawn the single writer thread
    let writer_handle = {
        let output_file = Arc::clone(&output_file);
        let index_file = Arc::clone(&index_file);
        thread::spawn(move || {
            // We must preserve chunk order. We'll store results by chunk_index
            // and write them in ascending order.
            let mut next_to_write = 0usize;
            let mut pending: HashMap<usize, ChunkResult> = HashMap::new();

            let bin_cfg: Configuration = standard();

            while let Ok(chunk_res) = chunk_receiver.recv() {
                // Insert into a pending map
                pending.insert(chunk_res.chunk_index, chunk_res);

                // Now, try to write out any chunk results in ascending order
                // that are ready (i.e., next_to_write, next_to_write+1, etc.)
                while let Some(res) = pending.remove(&next_to_write) {
                    // Actually write to .zst
                    let mut of = output_file.lock().unwrap();
                    let frame_offset = of.seek(SeekFrom::Current(0))
                        .expect("Failed to get zst offset");

                    // If you do the compression here:
                    let frame_size = match compress_and_write_chunk(&res.chunk_data, &mut *of) {
                        Ok(sz) => sz,
                        Err(e) => {
                            eprintln!("Writer error while compressing chunk: {:?}", e);
                            return;
                        }
                    };

                    drop(of); // release the lock

                    // Next, write FrameInfo + filter into .idx
                    let frame_info = FrameInfo {
                        frame_offset,
                        frame_size,
                    };

                    let frame_info_bytes =
                        encode_to_vec(&frame_info, bin_cfg)
                            .expect("Failed to encode FrameInfo");
                    let filter_bytes =
                        encode_to_vec(&res.filter, bin_cfg)
                            .expect("Failed to encode BinaryFuse8");

                    let mut idxf = index_file.lock().unwrap();
                    if let Err(e) = idxf.write_all(&frame_info_bytes) {
                        eprintln!("Writer error writing FrameInfo: {:?}", e);
                        return;
                    }
                    if let Err(e) = idxf.write_all(&filter_bytes) {
                        eprintln!("Writer error writing filter: {:?}", e);
                        return;
                    }
                    drop(idxf);

                    // Move on to the next chunk
                    next_to_write += 1;
                }
            }
        })
    };

    // 7) Chunk the file in the main thread and send to worker threads.
    let mut start = 0usize;
    let mut chunk_index = 0usize;
    while start < file_len {
        // Same logic as before to find chunk boundary
        let end_candidate = (start + CHUNK_SIZE).min(file_len);
        let actual_end = if end_candidate < file_len {
            match find_line_boundary(&mmap[end_candidate..]) {
                Some(rel_pos) => end_candidate + rel_pos + 1,
                None => file_len,
            }
        } else {
            file_len
        };

        // Copy the chunk into a Vec<u8> so we can send it to a worker thread
        let chunk = mmap[start..actual_end].to_vec();

        // Send it off
        if let Err(e) = chunk_sender.send((chunk_index, chunk)) {
            eprintln!("Error sending chunk to worker: {:?}", e);
            break;
        }

        chunk_index += 1;
        start = actual_end;
    }
    drop(chunk_sender); // no more chunks will be produced

    // 8) Wait for writer thread to finish
    writer_handle.join().expect("Writer thread panicked");

    println!("Build complete. ZST: {:?} | IDX: {:?}", zst_path, idx_path);
    Ok(())
}

/// Just like original
fn find_line_boundary(data: &[u8]) -> Option<usize> {
    data.iter().position(|&c| c == b'\n')
}

/// Just like original
fn compress_and_write_chunk(chunk: &[u8], output: &mut File) -> io::Result<u64> {
    let start_pos = output.seek(SeekFrom::Current(0))?;
    let mut encoder = Encoder::new(output, 0)?; // Level=0 for speed
    encoder.write_all(chunk)?;
    let output_file = encoder.finish()?;
    let end_pos = output_file.seek(SeekFrom::Current(0))?;
    Ok(end_pos - start_pos)
}

/// Build a BinaryFuse8 filter for 8-byte windows in the chunk
fn build_binaryfuse_filter(chunk: &[u8],set: &mut FastSet) -> BinaryFuse8 {
    set.clear();

    for window in chunk.windows(8) {
        let key = u64::from_le_bytes(window.try_into().unwrap());
        set.insert(key);
    }

    let numkeys = set.len();
    let keys = set.extract();

    let filter = BinaryFuse8::try_from(&keys[..]).expect("Failed to build BinaryFuse8 filter");

    // Some debug stats
    let bpe = (filter.len() as f64) * 16.0 / (numkeys as f64);
    let mut rng = rand::thread_rng();
    let false_positives: usize = (0..numkeys)
        .map(|_| rng.gen::<u64>())
        .filter(|n| filter.contains(n))
        .count();
    let fp_rate: f64 = (false_positives * 100) as f64 / numkeys as f64;
    println!("set size: {}, bpe: {}, fp_rate: {}%", numkeys, bpe, fp_rate);

    filter
}
