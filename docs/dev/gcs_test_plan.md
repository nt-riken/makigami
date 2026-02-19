# GCS Test Plan: test.zst + test.idx on Google Cloud Storage

## Bucket

- **Bucket:** `makigami-bucket-1`
- **Objects:** `test.zst`, `test.idx` (index; built locally as `test_log.mg`)

---

## Update GCS with latest built files

Rebuild from the source log, then upload so the bucket has the latest data.

**1. Rebuild locally (from project root):**
```bash
./target/debug/mg build test_log.txt
# or: cargo run -- build test_log.txt
```
This creates/updates `test_log.zst` and `test_log.mg`.

**2. Upload to GCS (overwrites existing objects):**
```bash
export BUCKET=makigami-bucket-1

gsutil cp test_log.zst gs://$BUCKET/test.zst
gsutil cp test_log.mg  gs://$BUCKET/test.idx
```
Use `test.idx` on GCS so search can use `--idx gs://$BUCKET/test.idx`.

**One-liner (rebuild + upload):**
```bash
./target/debug/mg build test_log.txt && \
gsutil cp test_log.zst gs://makigami-bucket-1/test.zst && \
gsutil cp test_log.mg  gs://makigami-bucket-1/test.idx
```

---

## Prerequisites

- [ ] GCS bucket with objects: `test.zst`, `test.idx` (or `test.mg`)
- [ ] Authentication: `gcloud auth application-default login` (done)
- [ ] Build: `cargo build --features gcs` (develop/debug build)

## Information

| Item | Value |
|------|--------|
| **Bucket name** | `makigami-bucket-1` |
| **Index object name** | `test.idx` (upload `test_log.mg` as this) |

## Test Cases

### 0. Pre-check: List objects in bucket

```bash
export BUCKET=makigami-bucket-1
gsutil ls gs://$BUCKET/
```

Expected: You see `test.zst` and `test.idx`.

---

### 1. Success: Search with explicit index (use this if your index is `test.idx`)

```bash
./target/debug/mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/test.idx "PATTERN"
```

- **Pattern that should match**: Pick a string you know exists in the original log (e.g. a word from the log).
- **Expected**: Exit 0, candidate block(s) printed (pipe to `grep PATTERN` to see matching lines only).

Example (if your log contained "ERROR"):

```bash
./target/debug/mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/test.idx "ERROR" | grep "ERROR"
```

---

### 2. Success: Search with auto-derived index (use only if your index object is `test.mg`)

If you uploaded the index as `test.mg` (not `test.idx`):

```bash
./target/debug/mg search gs://$BUCKET/test.zst "PATTERN"
```

- **Expected**: Same as test 1 (exit 0, output as above).

---

### 3. Success: Non-existent pattern (no false positives)

```bash
./target/debug/mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/test.idx "XYZZY_NONEXISTENT_123" | wc -l
```

- **Expected**: Exit 0, line count = 0 (no output).

---

### 4. Failure: Wrong bucket / missing object

```bash
./target/debug/mg search gs://$BUCKET/nonexistent.zst "PATTERN" 2>&1
```

- **Expected**: Exit non-zero, error message (e.g. HTTP 404 or "No such file").

---

### 5. Failure: Missing index (wrong --idx)

```bash
./target/debug/mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/missing.idx "PATTERN" 2>&1
```

- **Expected**: Exit non-zero, error (e.g. HTTP 404).

---

### 6. Cache behavior (optional)

- Run a search twice with the same `gs://` index URL.
- Second run should still succeed; index is cached under `~/.cache/makigami/`.
- Check cache: `ls -la ~/.cache/makigami/$BUCKET/` (path may vary by object name).

---

## Summary Checklist

| # | Test | Command / check | Pass? |
|---|------|------------------|-------|
| 0 | Objects exist | `gsutil ls gs://$BUCKET/` | |
| 1 | Search success (explicit index) | `mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/test.idx "PATTERN"` | |
| 2 | Search success (auto index) | Only if index is `test.mg`: `mg search gs://$BUCKET/test.zst "PATTERN"` | |
| 3 | Non-existent pattern | Output lines = 0 | |
| 4 | Missing .zst | Non-zero exit, error message | |
| 5 | Missing index | Non-zero exit, error message | |
| 6 | Cache | Second run works; cache dir exists | |

---

## Quick copy-paste (after setting BUCKET)

Replace `YOUR_BUCKET` and `YOUR_PATTERN` (e.g. a word from your log).

```bash
export BUCKET=YOUR_BUCKET
export PATTERN=YOUR_PATTERN

# Success
./target/debug/mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/test.idx "$PATTERN" | grep "$PATTERN"

# No false positives
./target/debug/mg search gs://$BUCKET/test.zst --idx gs://$BUCKET/test.idx "XYZZY_NONEXISTENT" | wc -l
```

---

## If You Need to Provide

1. **Bucket name** – so commands can be filled in exactly.
2. **Exact index object name** – `test.idx` or `test.mg` (or other). If it’s `test.idx`, always use `--idx gs://$BUCKET/test.idx` in the plan above.
