# GCS Testing Guide

## Prerequisites

1. **Rust 1.85 or newer** (required for GCS feature; project uses edition 2024 to support the latest `google-cloud-storage` crate). Check with `rustc --version`; update with `rustup update stable`.
2. **Google Cloud Project** with GCS enabled
3. **gcloud CLI** installed and configured
4. **GCS bucket** created (or use existing one)

## Setup Steps

### Option 1: Local Development (Recommended for Testing)

1. **Install gcloud CLI** (if not already installed):
   ```bash
   # macOS
   brew install google-cloud-sdk
   
   # Or download from: https://cloud.google.com/sdk/docs/install
   ```

2. **Authenticate with Application Default Credentials**:
   ```bash
   gcloud auth application-default login
   ```
   This will open a browser for authentication and save credentials to:
   `~/.config/gcloud/application_default_credentials.json`

3. **Set your GCP project** (if needed):
   ```bash
   gcloud config set project YOUR_PROJECT_ID
   ```

4. **Create a test bucket** (if you don't have one):
   ```bash
   gsutil mb gs://your-test-bucket-name
   ```

5. **Upload test files to GCS**:
   ```bash
   # Build local test files first
   ./target/debug/mg build test_log.txt
   
   # Upload to GCS
   gsutil cp test_log.zst gs://your-test-bucket-name/
   gsutil cp test_log.mg gs://your-test-bucket-name/
   ```

6. **Build makigami with GCS feature**:
   ```bash
   cargo build --features gcs
   ```

7. **Test GCS search**:
   ```bash
   ./target/debug/mg search gs://your-test-bucket-name/test_log.zst "ERROR"
   ```

### Option 2: GCE/GKE Instance (Production-like)

1. **Create a GCE VM** or use existing GKE cluster

2. **Ensure the VM has GCS access**:
   - Attach service account with Storage Object Viewer/Reader role
   - Or use default compute service account

3. **Build and test on the instance**:
   ```bash
   cargo build --features gcs
   ./target/debug/mg search gs://bucket/path/file.zst "pattern"
   ```

## Testing Checklist

- [ ] Build with `--features gcs`
- [ ] Authenticate with `gcloud auth application-default login`
- [ ] Upload test files to GCS bucket
- [ ] Test search with `gs://` URL
- [ ] Verify cache works (`~/.cache/makigami/`)
- [ ] Test with missing file (error handling)
- [ ] Test with non-existent pattern (no false positives)

## Troubleshooting

### Error: "No valid credentials found"
- Run `gcloud auth application-default login`
- Check `~/.config/gcloud/application_default_credentials.json` exists

### Error: "Metadata server request failed"
- You're not on GCE/GKE, but ADC should work as fallback
- Check ADC file exists and is valid

### Error: "HTTP error: 403"
- Check bucket permissions
- Verify service account has Storage Object Viewer role
- Check bucket name is correct

### Error: "GCS support not enabled"
- Build with `--features gcs` flag


