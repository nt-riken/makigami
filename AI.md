# makigami - Technical Specification: GCS Integration Phase

## 1. Project Overview

`makigami` is a high-performance log compression and search tool in Rust. It uses a Bloom Filter-based hash index to identify and extract relevant data blocks without full decompression.

## 2. Strategic Goal: IaaS Adaptation (GCS)

The current priority is to transition from a local-only file system to a cloud-native architecture using **Google Cloud Storage (GCS)**, while maintaining local file support through abstraction.

## 3. Storage Architecture & Rules

### A. Storage Abstraction (The `LogStorage` Trait)

* Define a trait `LogStorage` to unify access to both `LocalFile` and `GCS`.
* **Requirements**:
  * `fetch_index() -> Result<Vec<u8>>`: Retrieve the `.mg` index metadata file.
  * `read_block(offset: u64, size: u64) -> Result<Vec<u8>>`: Fetch a specific compressed block at the given offset and size.
* **Public API**: The `LogStorage` trait and all callers (e.g. search) remain **synchronous**. Local storage is fully sync.

### B. GCS Implementation Details

* **Prefer well-built packages**: Use an established GCS Rust crate (e.g. `google-cloud-storage`) rather than hand-written OAuth/HTTP. Such crates are typically async; that is acceptable.
* **Confine async**: Keep async only inside the GCS storage implementation. At the boundary (e.g. where search calls `fetch_index` / `read_block`), use a small runtime and `block_on` so the rest of the app stays sync. Do not spread async across the whole codebase.
* **Byte Range Requests**: Must fetch only specific blocks identified by the index (range reads).
* **Index Management**:
  * Store `.mg` index file alongside the `.zst` data file in the GCS bucket.
  * Implement a local cache (e.g., in `~/.cache/makigami/`) to store downloaded indexes.
  * Cache key format: `~/.cache/makigami/{bucket}/{sanitized_object_path}.mg`
* **Authentication**: Use Application Default Credentials (ADC); support browser auth via `gcloud auth application-default login`.
* **URL Format**: Auto-detect `gs://bucket/path/to/file.zst` format. No separate flags needed.

### C. Write Path (Compression)

* Implement a "Local-then-Upload" strategy.
* Compress and create the `.zst` and `.mg` files locally (build command remains unchanged).
* External tools (e.g., `gsutil`, `gcloud`) handle uploading to GCS bucket. No upload functionality needed in makigami.

## 4. Technical Constraints

* **Zero-Copy**: Minimize memory allocations during block transfer (use `Vec<u8>` for now, can optimize with `Bytes` later if needed).
* **Feature Flags**: GCS dependencies must be optional via Cargo features (e.g., `cargo build --features gcs`).
* **No Line Matching**: `makigami` remains a block-level extractor. Final line-level filtering is handled by external tools (e.g., `grep`).
* **Sync-by-default**: Keep the application surface synchronous. Use async only where a well-built package requires it (e.g. GCS), and confine that to a single layer with `block_on` at the boundary.

## 5. Coding Standards for Cursor

* Use `thiserror` for granular error handling of network/file I/O.
* Follow Idiomatic Rust: prefer `Result` handling over `unwrap()`.
* Use `std::fs::File` for local file operations.
* For GCS: prefer a well-built crate (e.g. `google-cloud-storage`). If that crate is async, implement the GCS backend with async internally and call it via `tokio::runtime::Runtime::block_on` (or equivalent) so the `LogStorage` trait remains sync.
* Do not implement heavy abstraction. Least abstraction is prefer.
* Do not remain non running code part.
* Do not implement heavy error handling. Least error handling first, and add when it is required.
* Do not invent wheel again. Using well developed library is prefer.

## 6. Principles: Packages and Async

* **Prefer well-built packages**: Do not reimplement OAuth or GCS APIs. Use a maintained crate (e.g. `google-cloud-storage`) so auth, retries, and API quirks are handled correctly.
* **Confine async**: Many such crates are async. Keep async only in the GCS storage module. The rest of the app (main, build, search entrypoint, local storage) stays synchronous. Use a small runtime and `block_on` at the boundary so the public `LogStorage` API remains sync.
* **Build command**: Stays fully sync (local disk only).
* **Search command**: Stays sync from the caller’s view; only the GCS backend may use async internally.
