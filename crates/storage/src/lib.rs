//! Storage module for file management
//! 
//! Provides functionality for:
//! - Uploading files to local filesystem
//! - Downloading files
//! - Deleting files
//! - Listing files
//! - Metadata tracking with database persistence

pub mod model;
pub mod service;

pub use model::File;
pub use service::{TransactionalStorageService, UserStorageStats};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Metadata for a stored file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub original_name: String,
    pub stored_name: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Storage service for managing files
pub struct StorageService {
    base_path: PathBuf,
}

impl StorageService {
    /// Create a new storage service
    /// 
    /// # Arguments
    /// * `base_path` - Base directory for storing files
    pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        // Create base directory if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(&base_path).await?;
        }
        
        Ok(Self { base_path })
    }
    
    /// Store a file with optional metadata
    /// 
    /// # Arguments
    /// * `data` - File data as bytes
    /// * `original_name` - Original filename
    /// * `mime_type` - Optional MIME type
    /// 
    /// # Returns
    /// FileMetadata with generated ID and storage information
    pub async fn store(&self, data: &[u8], original_name: &str, mime_type: Option<String>) -> Result<FileMetadata> {
        let id = Uuid::new_v4().to_string();
        let extension = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let stored_name = if extension.is_empty() {
            id.clone()
        } else {
            format!("{}.{}", id, extension)
        };
        
        let file_path = self.base_path.join(&stored_name);
        
        // Write file
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(data).await?;
        file.flush().await?;
        
        let metadata = FileMetadata {
            id,
            original_name: original_name.to_string(),
            stored_name,
            size: data.len() as u64,
            mime_type,
            created_at: Utc::now(),
        };
        
        Ok(metadata)
    }
    
    /// Retrieve a file by its ID
    /// 
    /// # Arguments
    /// * `file_id` - The file ID or stored name
    /// 
    /// # Returns
    /// File data as bytes
    pub async fn retrieve(&self, file_id: &str) -> Result<Vec<u8>> {
        // Try direct lookup first
        let mut file_path = self.base_path.join(file_id);
        
        // If not found, try with common extensions
        if !file_path.exists() {
            let extensions = ["", ".jpg", ".png", ".pdf", ".txt", ".json"];
            let mut found = false;
            
            for ext in extensions {
                let test_path = self.base_path.join(format!("{}{}", file_id, ext));
                if test_path.exists() {
                    file_path = test_path;
                    found = true;
                    break;
                }
            }
            
            if !found {
                return Err(StorageError::FileNotFound(file_id.to_string()));
            }
        }
        
        let mut file = fs::File::open(&file_path).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;
        
        Ok(data)
    }
    
    /// Delete a file by its ID
    /// 
    /// # Arguments
    /// * `file_id` - The file ID or stored name
    pub async fn delete(&self, file_id: &str) -> Result<()> {
        let mut file_path = self.base_path.join(file_id);
        
        // If not found, try with common extensions
        if !file_path.exists() {
            let extensions = ["", ".jpg", ".png", ".pdf", ".txt", ".json"];
            let mut found = false;
            
            for ext in extensions {
                let test_path = self.base_path.join(format!("{}{}", file_id, ext));
                if test_path.exists() {
                    file_path = test_path;
                    found = true;
                    break;
                }
            }
            
            if !found {
                return Err(StorageError::FileNotFound(file_id.to_string()));
            }
        }
        
        fs::remove_file(&file_path).await?;
        Ok(())
    }
    
    /// Check if a file exists
    /// 
    /// # Arguments
    /// * `file_id` - The file ID or stored name
    pub async fn exists(&self, file_id: &str) -> bool {
        let file_path = self.base_path.join(file_id);
        
        if file_path.exists() {
            return true;
        }
        
        // Try with common extensions
        let extensions = ["", ".jpg", ".png", ".pdf", ".txt", ".json"];
        for ext in extensions {
            let test_path = self.base_path.join(format!("{}{}", file_id, ext));
            if test_path.exists() {
                return true;
            }
        }
        
        false
    }
    
    /// List all files in storage
    /// 
    /// # Returns
    /// Vector of file paths relative to base_path
    pub async fn list_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }
        
        Ok(files)
    }
    
    /// Get file metadata
    /// 
    /// # Arguments
    /// * `file_id` - The file ID or stored name
    pub async fn get_metadata(&self, file_id: &str) -> Result<std::fs::Metadata> {
        let file_path = self.base_path.join(file_id);
        
        if !file_path.exists() {
            return Err(StorageError::FileNotFound(file_id.to_string()));
        }
        
        let metadata = fs::metadata(&file_path).await?;
        Ok(metadata)
    }
    
    /// Get the base storage path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageService::new(temp_dir.path()).await.unwrap();
        
        let data = b"Hello, World!";
        let metadata = storage.store(data, "test.txt", Some("text/plain".to_string())).await.unwrap();
        
        assert_eq!(metadata.original_name, "test.txt");
        assert_eq!(metadata.size, data.len() as u64);
        
        let retrieved = storage.retrieve(&metadata.id).await.unwrap();
        assert_eq!(retrieved, data);
    }
    
    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageService::new(temp_dir.path()).await.unwrap();
        
        let data = b"Test data";
        let metadata = storage.store(data, "test.txt", None).await.unwrap();
        
        assert!(storage.exists(&metadata.id).await);
        
        storage.delete(&metadata.id).await.unwrap();
        
        assert!(!storage.exists(&metadata.id).await);
    }
    
    #[tokio::test]
    async fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageService::new(temp_dir.path()).await.unwrap();
        
        storage.store(b"file1", "file1.txt", None).await.unwrap();
        storage.store(b"file2", "file2.txt", None).await.unwrap();
        
        let files = storage.list_files().await.unwrap();
        assert_eq!(files.len(), 2);
    }
}
