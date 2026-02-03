use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Error type for storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    
    #[error("GCS error: {0}")]
    Gcs(String),
    
    #[error("Invalid range: offset={offset}, size={size}")]
    InvalidRange { offset: u64, size: u64 },
}

/// Storage abstraction trait for reading index and data blocks
pub trait LogStorage {
    /// Fetch the entire index file (.mg)
    fn fetch_index(&self) -> Result<Vec<u8>, StorageError>;
    
    /// Read a specific block from the compressed data file
    /// 
    /// # Arguments
    /// * `offset` - Byte offset in the compressed file
    /// * `size` - Number of bytes to read
    fn read_block(&self, offset: u64, size: u64) -> Result<Vec<u8>, StorageError>;
}

/// Local file system storage implementation
pub struct LocalFileStorage {
    zst_path: PathBuf,
    idx_path: PathBuf,
}

impl LocalFileStorage {
    pub fn new(zst_path: &str, idx_path: Option<&str>) -> Result<Self, StorageError> {
        let zst_path = PathBuf::from(zst_path);
        let idx_path = if let Some(idx) = idx_path {
            PathBuf::from(idx)
        } else {
            // Derive index path from zst path
            zst_path.with_extension("mg")
        };
        
        Ok(Self { zst_path, idx_path })
    }
}

impl LogStorage for LocalFileStorage {
    fn fetch_index(&self) -> Result<Vec<u8>, StorageError> {
        std::fs::read(&self.idx_path).map_err(StorageError::Io)
    }
    
    fn read_block(&self, offset: u64, size: u64) -> Result<Vec<u8>, StorageError> {
        use std::io::{Read, Seek, SeekFrom};
        
        let mut file = std::fs::File::open(&self.zst_path)?;
        file.seek(SeekFrom::Start(offset))?;
        
        let mut buffer = vec![0u8; size as usize];
        file.read_exact(&mut buffer)?;
        
        Ok(buffer)
    }
}

#[cfg(feature = "gcs")]
mod gcs_storage {
    use super::*;
    use google_cloud_storage::client::Storage;
    use google_cloud_storage::model_ext::ReadRange;
    use tokio::runtime::Runtime;

    /// Bucket name in crate format: projects/_/buckets/{name}
    fn bucket_resource(name: &str) -> String {
        format!("projects/_/buckets/{}", name)
    }

    /// GCS storage implementation using google-cloud-storage crate. Async is confined here; trait remains sync via block_on.
    pub struct GcsStorage {
        bucket: String,
        zst_object: String,
        idx_object: String,
        cache_dir: Option<PathBuf>,
        client: Storage,
        runtime: Runtime,
    }

    impl GcsStorage {
        pub fn new(
            bucket: &str,
            zst_object: &str,
            idx_object: Option<&str>,
            cache_dir: Option<PathBuf>,
        ) -> Result<Self, StorageError> {
            let idx_object = if let Some(idx) = idx_object {
                idx.to_string()
            } else if zst_object.ends_with(".zst") {
                zst_object.replace(".zst", ".mg")
            } else {
                format!("{}.mg", zst_object)
            };

            let runtime = Runtime::new().map_err(|e| StorageError::Gcs(format!("Failed to create runtime: {}", e)))?;
            let client = runtime
                .block_on(async {
                    Storage::builder().build().await.map_err(|e| StorageError::Gcs(format!("Failed to build GCS client: {}", e)))
                })?;

            Ok(Self {
                bucket: bucket.to_string(),
                zst_object: zst_object.to_string(),
                idx_object,
                cache_dir,
                client,
                runtime,
            })
        }

        fn cache_path(&self) -> PathBuf {
            let cache_base = self.cache_dir.as_ref().cloned().unwrap_or_else(|| {
                let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.push(".cache");
                home.push("makigami");
                home
            });
            let sanitized = self.idx_object.replace('/', "_");
            cache_base.join(&self.bucket).join(sanitized)
        }

        /// Download full object or a byte range. Runs async GCS calls via block_on.
        fn download(&self, object: &str, range: Option<(u64, u64)>) -> Result<Vec<u8>, StorageError> {
            let bucket = bucket_resource(&self.bucket);
            let object = object.to_string();
            let client = self.client.clone();

            let fut = async move {
                let mut request = client.read_object(&bucket, &object);
                if let Some((offset, count)) = range {
                    request = request.set_read_range(ReadRange::segment(offset, count));
                }
                let mut resp = request.send().await.map_err(|e| StorageError::Gcs(format!("GCS read failed: {}", e)))?;
                let mut contents = Vec::new();
                while let Some(chunk) = resp.next().await.transpose().map_err(|e| StorageError::Gcs(format!("GCS stream error: {}", e)))? {
                    contents.extend_from_slice(&chunk);
                }
                Ok(contents)
            };

            self.runtime.block_on(fut)
        }
    }

    impl LogStorage for GcsStorage {
        fn fetch_index(&self) -> Result<Vec<u8>, StorageError> {
            let cache_path = self.cache_path();
            if cache_path.exists() {
                if let Ok(data) = std::fs::read(&cache_path) {
                    return Ok(data);
                }
            }
            let data = self.download(&self.idx_object, None)?;
            if let Some(parent) = cache_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&cache_path, &data)?;
            Ok(data)
        }

        fn read_block(&self, offset: u64, size: u64) -> Result<Vec<u8>, StorageError> {
            if size == 0 {
                return Err(StorageError::InvalidRange { offset, size });
            }
            self.download(&self.zst_object, Some((offset, size)))
        }
    }
}

#[cfg(feature = "gcs")]
use gcs_storage::GcsStorage;

/// Create appropriate storage backend based on URL/path
pub fn create_storage(zst_path: &str, idx_path: Option<&str>) -> Result<Box<dyn LogStorage>, StorageError> {
    // Check if it's a GCS URL
    if zst_path.starts_with("gs://") {
        #[cfg(not(feature = "gcs"))]
        {
            return Err(StorageError::Gcs(
                "GCS support not enabled. Build with --features gcs".to_string()
            ));
        }
        
        #[cfg(feature = "gcs")]
        {
            // Parse gs://bucket/path/to/file.zst
            let path = &zst_path[5..]; // Remove "gs://"
            let parts: Vec<&str> = path.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(StorageError::InvalidUrl(format!(
                    "Invalid GCS URL format: {}", zst_path
                )));
            }
            
            let bucket = parts[0];
            let zst_object = parts[1];
            
            // Parse index path if provided
            let idx_object = idx_path.map(|p| {
                if p.starts_with("gs://") {
                    let idx_path = &p[5..];
                    idx_path.splitn(2, '/').nth(1).unwrap_or(idx_path).to_string()
                } else {
                    p.to_string()
                }
            });
            
            let storage = GcsStorage::new(bucket, zst_object, idx_object.as_deref(), None)?;
            Ok(Box::new(storage))
        }
    } else {
        // Local file
        let storage = LocalFileStorage::new(zst_path, idx_path)?;
        Ok(Box::new(storage))
    }
}

