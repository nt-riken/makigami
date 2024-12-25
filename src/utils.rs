use std::path::{Path, PathBuf};

/// Our default chunk size for splitting
pub const CHUNK_SIZE: usize = 64 * 1024 * 1024;

/// Derive default `.zst` and `.idx` output filenames from the input.
/// If the user provided `--zst` or `--idx`, we respect that. Otherwise, 
/// we generate something like `input.log` -> `input.zst`, `input.idx`.
pub fn default_output_names_if_omitted(
    input_path: &str,
    maybe_zst: Option<&str>,
    maybe_idx: Option<&str>,
) -> (PathBuf, PathBuf) {
    let input = Path::new(input_path);
    let stem = input.file_stem().unwrap_or_else(|| std::ffi::OsStr::new("output"));
    
    // Build the .zst path
    let zst_path = if let Some(given) = maybe_zst {
        PathBuf::from(given)
    } else {
        // If extension is .log, we produce .zst
        input.with_file_name(format!("{}.zst", stem.to_string_lossy()))
    };

    // Build the .idx path
    let idx_path = if let Some(given) = maybe_idx {
        PathBuf::from(given)
    } else {
        input.with_file_name(format!("{}.idx", stem.to_string_lossy()))
    };

    (zst_path, idx_path)
}

/// If the user omits `--idx` in the search subcommand,
/// we'll guess `XXX.idx` from `XXX.zst`.
pub fn default_index_if_omitted(
    maybe_idx: Option<&str>,
    zst_path: &str,
) -> PathBuf {
    if let Some(idx) = maybe_idx {
        PathBuf::from(idx)
    } else {
        // Replace .zst with .idx if extension is .zst
        // or just append .idx if there's no extension
        let path = Path::new(zst_path);
        path.with_extension("idx")
    }
}
