use crate::{File, StorageService, StorageError, Result};
use orm::prelude::*;
use orm::query::QueryValue;

/// Transactional storage service that integrates filesystem storage with database persistence
pub struct TransactionalStorageService {
    storage: StorageService,
    db: Database,
}

impl TransactionalStorageService {
    /// Create a new transactional storage service
    pub fn new(storage: StorageService, db: Database) -> Self {
        Self { storage, db }
    }

    /// Store a file with database metadata tracking
    /// Uses compensating transaction pattern: write file first, then DB, cleanup on failure
    pub async fn store_with_metadata(
        &self,
        data: &[u8],
        original_name: &str,
        user_id: i64,
        mime_type: Option<String>,
    ) -> Result<File> {
        // Step 1: Write file to disk
        let file_metadata = self.storage.store(data, original_name, mime_type.clone()).await?;

        // Step 2: Insert metadata into database
        let file = File::new(
            file_metadata.id.clone(),
            user_id,
            file_metadata.original_name,
            file_metadata.stored_name.clone(),
            file_metadata.size as i64,
            mime_type,
            self.storage.base_path().to_string_lossy().to_string(),
        );

        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();

        let values = file.to_values();
        let columns: Vec<&str> = values.keys().map(|s| s.as_str()).collect();
        let query_values: Vec<_> = values.values().map(|v| v.to_query_value()).collect();

        query_builder.insert_into(File::table_name(), &columns);
        query_builder.values_params(&query_values);

        let sql = query_builder.build()
            .map_err(|e| StorageError::StorageError(format!("Query build error: {}", e)))?;

        // Execute insert with compensating action on failure
        match backend.execute(&sql, query_builder.params()).await {
            Ok(_) => Ok(file),
            Err(e) => {
                // Compensating action: delete the file we just wrote
                let _ = self.storage.delete(&file_metadata.stored_name).await;
                Err(StorageError::StorageError(format!("Database insert failed: {}", e)))
            }
        }
    }

    /// Delete a file and its metadata (transactional)
    pub async fn delete_with_metadata(&self, file_id: &str, user_id: i64) -> Result<()> {
        // Step 1: Fetch file metadata to verify ownership and get stored_name
        let file = self.get_file_by_id(file_id).await?
            .ok_or_else(|| StorageError::FileNotFound(file_id.to_string()))?;

        // Step 2: Verify ownership
        if file.user_id != user_id {
            return Err(StorageError::StorageError("Access denied: file belongs to another user".to_string()));
        }

        // Step 3: Delete from database first (safer - if disk delete fails, we can retry)
        let backend = self.db.backend();
        let sql = format!("DELETE FROM {} WHERE id = ?1", File::table_name());
        backend.execute(&sql, &[QueryValue::String(file_id.to_string())]).await
            .map_err(|e| StorageError::StorageError(format!("Database delete failed: {}", e)))?;

        // Step 4: Delete file from disk
        self.storage.delete(&file.stored_name).await?;

        Ok(())
    }

    /// Retrieve a file's data (with permission check)
    pub async fn retrieve_with_permission(&self, file_id: &str, user_id: i64) -> Result<Vec<u8>> {
        // Verify ownership
        let file = self.get_file_by_id(file_id).await?
            .ok_or_else(|| StorageError::FileNotFound(file_id.to_string()))?;

        if file.user_id != user_id {
            return Err(StorageError::StorageError("Access denied: file belongs to another user".to_string()));
        }

        // Retrieve file data
        self.storage.retrieve(&file.stored_name).await
    }

    /// Get file metadata by ID
    pub async fn get_file_by_id(&self, file_id: &str) -> Result<Option<File>> {
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();

        query_builder.from(File::table_name());
        query_builder.select(&[]);
        query_builder.where_eq("id", QueryValue::String(file_id.to_string()));
        query_builder.limit(1);

        let sql = query_builder.build()
            .map_err(|e| StorageError::StorageError(format!("Query build error: {}", e)))?;

        let json_rows = backend.fetch_all_params(&sql, query_builder.params()).await
            .map_err(|e| StorageError::StorageError(format!("Database error: {}", e)))?;

        if json_rows.is_empty() {
            return Ok(None);
        }

        let file = File::from_json(&json_rows[0])
            .map_err(|e| StorageError::StorageError(format!("Deserialization error: {}", e)))?;

        Ok(Some(file))
    }

    /// List all files for a user
    pub async fn list_user_files(&self, user_id: i64) -> Result<Vec<File>> {
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();

        query_builder.from(File::table_name());
        query_builder.select(&[]);
        query_builder.where_eq("user_id", QueryValue::I64(user_id));
        query_builder.order_by("created_at", orm::query::OrderDirection::Desc);

        let sql = query_builder.build()
            .map_err(|e| StorageError::StorageError(format!("Query build error: {}", e)))?;

        let json_rows = backend.fetch_all_params(&sql, query_builder.params()).await
            .map_err(|e| StorageError::StorageError(format!("Database error: {}", e)))?;

        let files: Result<Vec<File>> = json_rows.iter()
            .map(|json| File::from_json(json)
                .map_err(|e| StorageError::StorageError(format!("Deserialization error: {}", e))))
            .collect();

        files
    }

    /// Check if a file exists and belongs to the user
    pub async fn file_exists_for_user(&self, file_id: &str, user_id: i64) -> Result<bool> {
        match self.get_file_by_id(file_id).await? {
            Some(file) => Ok(file.user_id == user_id),
            None => Ok(false),
        }
    }

    /// Get storage statistics for a user
    pub async fn get_user_storage_stats(&self, user_id: i64) -> Result<UserStorageStats> {
        let backend = self.db.backend();
        let sql = format!(
            "SELECT COUNT(*) as file_count, COALESCE(SUM(size), 0) as total_size FROM {} WHERE user_id = ?1",
            File::table_name()
        );

        #[allow(deprecated)]
        let result = backend.fetch_one(&sql.replace("?1", &format!("'{}'", user_id))).await
            .map_err(|e| StorageError::StorageError(format!("Database error: {}", e)))?;

        let stats = result.map(|json| {
            let file_count = json.get("file_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let total_size = json.get("total_size")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            UserStorageStats {
                file_count,
                total_size,
            }
        }).unwrap_or_default();

        Ok(stats)
    }
}

/// Storage statistics for a user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserStorageStats {
    pub file_count: i64,
    pub total_size: i64,
}

impl Default for UserStorageStats {
    fn default() -> Self {
        Self {
            file_count: 0,
            total_size: 0,
        }
    }
}
