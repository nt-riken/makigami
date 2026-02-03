#!/usr/bin/env bash
# Rebuild test_log.zst and test_log.mg from test_log.txt, then upload to GCS.
# Usage: ./upload-to-gcs.sh [bucket]
# Default bucket: makigami-bucket-1

set -e
BUCKET="${1:-makigami-bucket-1}"

echo "Building from test_log.txt..."
./target/debug/mg build test_log.txt

echo "Uploading to gs://$BUCKET/..."
gsutil cp test_log.zst "gs://$BUCKET/test.zst"
gsutil cp test_log.mg  "gs://$BUCKET/test.idx"

echo "Done. Objects: gs://$BUCKET/test.zst, gs://$BUCKET/test.idx"
