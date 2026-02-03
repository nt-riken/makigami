# GCS Refactor: Use `google-cloud-storage` Crate (Modification-Needed Parts)

Refactor the GCS backend to use the well-built **google-cloud-storage** crate and **confine async** to that layer. The rest of the app stays sync; the `LogStorage` trait remains sync.

---

## 1. Cargo.toml

**Current (hand-written OAuth + reqwest):**
- `gcs` feature: `reqwest`, `serde_json`, `dirs`
- `reqwest` with `blocking`, `json`, `rustls-tls`

**Change to:**
- Add `google-cloud-storage` (optional, behind `gcs`). Check crates.io for latest version (e.g. `1.7`).
- Add `tokio` with `rt` and `macros` (or minimal set) so we can run async GCS calls via `block_on`. Keep it optional under `gcs`.
- Remove or no longer use for GCS: `reqwest`, `serde_json` for OAuth (the crate handles auth). Keep `dirs` if still used for cache path.
- Feature: `gcs = ["google-cloud-storage", "tokio", "dirs"]` (adjust to actual crate names/features).

**Note:** `google-cloud-storage` uses async (`.await`). We will **not** make the rest of the app async; we only need a small tokio runtime to `block_on` the GCS calls inside the GCS storage implementation.

---

## 2. src/storage.rs — GCS module only

**Current:** `#[cfg(feature = "gcs")] mod gcs_storage { ... }` contains:
- `GcsStorage` with `reqwest::blocking::Client`
- Hand-written `get_access_token()` (metadata server + ADC file + refresh_token)
- Hand-written `refresh_token()`
- Hand-written `download_range()` (GET with Range header)

**Replace with:**
- **Remove:** All hand-written OAuth and reqwest-based download code.
- **Add:** Dependency on `google_cloud_storage::client::Storage` (or the type from the crate).
- **Bucket name format:** The crate uses `projects/_/buckets/{bucket_name}`. Convert `gs://bucket/path` → bucket `"projects/_/buckets/bucket"`, object `"path"`.
- **Create client:** `Storage::builder().build().await?` — this must run inside an async context. So we need either:
  - An async helper that creates the client and does the operations, then we call it via `tokio::runtime::Runtime::new()?.block_on(async { ... })`, or
  - Store a pre-built client (built once in `GcsStorage::new` using `block_on`).
- **fetch_index:** Use crate’s read_object (full object or with range). Check cache first (unchanged). On cache miss: call `client.read_object(bucket, idx_object).send().await?` (or with `.set_read_range(ReadRange::all())` if needed), collect bytes, then cache and return. Map crate errors to `StorageError::Gcs`.
- **read_block:** Use `client.read_object(bucket, zst_object).set_read_range(ReadRange::segment(offset, size)).send().await?`, collect bytes, return. Map crate errors to `StorageError::Gcs`.
- **Confine async:** All `.await` and async code stays inside the `gcs_storage` module. The public `impl LogStorage for GcsStorage` keeps `fn fetch_index(&self)` and `fn read_block(...)` **synchronous**; inside those methods, call something like `tokio::runtime::Handle::current().block_on(async { ... })` or create a runtime and `block_on` the async GCS calls. So the trait still returns `Result<Vec<u8>, StorageError>` and is sync.
- **Error handling:** Map `google_cloud_storage::Error` (or whatever the crate uses) to `StorageError::Gcs(String)` or a variant. No heavy abstraction; keep it minimal.

**Keep unchanged:**
- Cache path logic (`cache_path()`), cache-first for index, writing index to cache.
- `GcsStorage::new` signature and derivation of `idx_object` from `zst_object` (e.g. replace `.zst` with `.mg`).
- `create_storage()` in the same file: still parses `gs://` and constructs `GcsStorage`; no change to callers.

---

## 3. src/storage.rs — Trait and other code

- **LogStorage trait:** No change. Still sync: `fn fetch_index(&self) -> Result<Vec<u8>, StorageError>` and `fn read_block(...)`.
- **LocalFileStorage:** No change.
- **create_storage:** No change (still returns `Box<dyn LogStorage>`; only the GCS implementation is swapped).

---

## 4. src/search.rs

- No change. It already uses `create_storage()` and then `storage.fetch_index()` and `storage.read_block()` — both sync.

---

## 5. src/main.rs

- No change. Still calls `search::run_search(...)` synchronously.

---

## 6. Runtime usage (where to create tokio runtime)

**Option A — Inside each GCS call:**  
In `fetch_index` and `read_block`, create a runtime (or use a stored one) and `block_on(async { ... })`. Easiest but may create multiple runtimes if not careful.

**Option B — One runtime per GCS storage instance:**  
In `GcsStorage::new`, create `let rt = tokio::runtime::Runtime::new()?` and store it (e.g. `runtime: tokio::runtime::Runtime`). In `fetch_index` and `read_block`, call `self.runtime.block_on(async { ... })`. Single runtime, clear boundary.

Recommendation: **Option B** — store one `Runtime` in `GcsStorage` and use it for all GCS operations so async is confined and the rest of the app stays sync.

---

## Summary table

| Part | Action |
|------|--------|
| **Cargo.toml** | Add `google-cloud-storage`, `tokio` (optional, under `gcs`). Remove or stop using reqwest/serde_json for GCS. |
| **storage.rs (GCS module)** | Replace hand-written OAuth + reqwest with crate client; use `ReadRange::segment` for blocks; use one tokio runtime and `block_on` so trait impl stays sync. |
| **storage.rs (trait, local, create_storage)** | No change. |
| **search.rs** | No change. |
| **main.rs** | No change. |

After refactor, the only async code and the only use of the GCS crate and tokio should be inside the `gcs_storage` module, with a sync boundary at `LogStorage` and no async in `main` or `search`.
