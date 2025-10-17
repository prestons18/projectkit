use chrono::{DateTime, Utc};
use orm::prelude::*;
use orm::model::Row;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User model for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with hashed password
    pub fn new(email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Model for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn primary_key() -> &'static str {
        "id"
    }

    fn primary_key_value(&self) -> Option<Value> {
        self.id.map(|id| Value::I64(id))
    }

    fn to_values(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        if let Some(id) = self.id {
            map.insert("id".to_string(), Value::I64(id));
        }
        map.insert("email".to_string(), Value::String(self.email.clone()));
        map.insert("password_hash".to_string(), Value::String(self.password_hash.clone()));
        map.insert("created_at".to_string(), Value::String(self.created_at.to_rfc3339()));
        map.insert("updated_at".to_string(), Value::String(self.updated_at.to_rfc3339()));
        map
    }

    fn columns() -> Vec<&'static str> {
        vec!["email", "password_hash", "created_at", "updated_at"]
    }
}

impl FromRow for User {
    fn from_row(row: &Row) -> Result<Self> {
        let id = row.get("id")
            .and_then(|v| match v {
                Value::I64(i) => Some(*i),
                Value::I32(i) => Some(*i as i64),
                _ => None,
            });

        let email = row.get("email")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing email".to_string()))?;

        let password_hash = row.get("password_hash")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing password_hash".to_string()))?;

        let created_at = row.get("created_at")
            .and_then(|v| match v {
                Value::String(s) => DateTime::parse_from_rfc3339(s.as_str()).ok().map(|dt| dt.with_timezone(&Utc)),
                _ => None,
            })
            .unwrap_or_else(Utc::now);

        let updated_at = row.get("updated_at")
            .and_then(|v| match v {
                Value::String(s) => DateTime::parse_from_rfc3339(s.as_str()).ok().map(|dt| dt.with_timezone(&Utc)),
                _ => None,
            })
            .unwrap_or_else(Utc::now);

        Ok(User {
            id,
            email,
            password_hash,
            created_at,
            updated_at,
        })
    }
}

/// Session model for managing user sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<i64>,
    pub user_id: i64,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: i64, token: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            user_id,
            token,
            expires_at,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

impl Model for Session {
    fn table_name() -> &'static str {
        "sessions"
    }

    fn primary_key() -> &'static str {
        "id"
    }

    fn primary_key_value(&self) -> Option<Value> {
        self.id.map(|id| Value::I64(id))
    }

    fn to_values(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        if let Some(id) = self.id {
            map.insert("id".to_string(), Value::I64(id));
        }
        map.insert("user_id".to_string(), Value::I64(self.user_id));
        map.insert("token".to_string(), Value::String(self.token.clone()));
        map.insert("expires_at".to_string(), Value::String(self.expires_at.to_rfc3339()));
        map.insert("created_at".to_string(), Value::String(self.created_at.to_rfc3339()));
        map
    }

    fn columns() -> Vec<&'static str> {
        vec!["user_id", "token", "expires_at", "created_at"]
    }
}

impl FromRow for Session {
    fn from_row(row: &Row) -> Result<Self> {
        let id = row.get("id")
            .and_then(|v| match v {
                Value::I64(i) => Some(*i),
                Value::I32(i) => Some(*i as i64),
                _ => None,
            });

        let user_id = row.get("user_id")
            .and_then(|v| match v {
                Value::I64(i) => Some(*i),
                Value::I32(i) => Some(*i as i64),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing user_id".to_string()))?;

        let token = row.get("token")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing token".to_string()))?;

        let expires_at = row.get("expires_at")
            .and_then(|v| match v {
                Value::String(s) => DateTime::parse_from_rfc3339(s.as_str()).ok().map(|dt| dt.with_timezone(&Utc)),
                _ => None,
            })
            .ok_or_else(|| Error::SerializationError("Missing expires_at".to_string()))?;

        let created_at = row.get("created_at")
            .and_then(|v| match v {
                Value::String(s) => DateTime::parse_from_rfc3339(s.as_str()).ok().map(|dt| dt.with_timezone(&Utc)),
                _ => None,
            })
            .unwrap_or_else(Utc::now);

        Ok(Session {
            id,
            user_id,
            token,
            expires_at,
            created_at,
        })
    }
}