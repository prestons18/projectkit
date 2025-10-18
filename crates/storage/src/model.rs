use chrono::{DateTime, Utc};
use orm::prelude::*;
use orm::model::Row;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// File model for database persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: Option<String>,
    pub user_id: i64,
    pub original_name: String,
    pub stored_name: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

impl File {
    /// Create a new file record
    pub fn new(
        id: String,
        user_id: i64,
        original_name: String,
        stored_name: String,
        size: i64,
        mime_type: Option<String>,
        storage_path: String,
    ) -> Self {
        Self {
            id: Some(id),
            user_id,
            original_name,
            stored_name,
            size,
            mime_type,
            storage_path,
            created_at: Utc::now(),
        }
    }
}

impl Model for File {
    fn table_name() -> &'static str {
        "files"
    }

    fn primary_key() -> &'static str {
        "id"
    }

    fn primary_key_value(&self) -> Option<Value> {
        self.id.as_ref().map(|id| Value::String(id.clone()))
    }

    fn to_values(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        if let Some(id) = &self.id {
            map.insert("id".to_string(), Value::String(id.clone()));
        }
        map.insert("user_id".to_string(), Value::I64(self.user_id));
        map.insert("original_name".to_string(), Value::String(self.original_name.clone()));
        map.insert("stored_name".to_string(), Value::String(self.stored_name.clone()));
        map.insert("size".to_string(), Value::I64(self.size));
        if let Some(mime_type) = &self.mime_type {
            map.insert("mime_type".to_string(), Value::String(mime_type.clone()));
        }
        map.insert("storage_path".to_string(), Value::String(self.storage_path.clone()));
        map.insert("created_at".to_string(), Value::String(self.created_at.to_rfc3339()));
        map
    }

    fn columns() -> Vec<&'static str> {
        vec!["user_id", "original_name", "stored_name", "size", "mime_type", "storage_path", "created_at"]
    }
}

impl FromRow for File {
    fn from_row(row: &Row) -> Result<Self> {
        let id = row.get("id")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            });

        let user_id = row.get("user_id")
            .and_then(|v| match v {
                Value::I64(i) => Some(*i),
                Value::I32(i) => Some(*i as i64),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing user_id".to_string()))?;

        let original_name = row.get("original_name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing original_name".to_string()))?;

        let stored_name = row.get("stored_name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing stored_name".to_string()))?;

        let size = row.get("size")
            .and_then(|v| match v {
                Value::I64(i) => Some(*i),
                Value::I32(i) => Some(*i as i64),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing size".to_string()))?;

        let mime_type = row.get("mime_type")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            });

        let storage_path = row.get("storage_path")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing storage_path".to_string()))?;

        let created_at = row.get("created_at")
            .and_then(|v| match v {
                Value::String(s) => DateTime::parse_from_rfc3339(s.as_str()).ok().map(|dt| dt.with_timezone(&Utc)),
                _ => None,
            })
            .unwrap_or_else(Utc::now);

        Ok(File {
            id,
            user_id,
            original_name,
            stored_name,
            size,
            mime_type,
            storage_path,
            created_at,
        })
    }
}
