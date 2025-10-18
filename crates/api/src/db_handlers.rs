use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// GET /db/:table - Fetch all records from a table
pub async fn get_table(
    State(state): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    let backend = state.db.backend();
    
    // Build a simple SELECT * query
    let sql = format!("SELECT * FROM {}", table);
    
    match backend.fetch_all_params(&sql, &[]).await {
        Ok(rows) => {
            (StatusCode::OK, Json(rows)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to fetch from table '{}': {}", table, e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// POST /db/:table - Insert a new record into a table
pub async fn post_table(
    State(state): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(payload): Json<JsonValue>,
) -> impl IntoResponse {
    let backend = state.db.backend();
    
    // Extract columns and values from the JSON payload
    let obj = match payload.as_object() {
        Some(obj) => obj,
        None => {
            let error = ErrorResponse {
                error: "Payload must be a JSON object".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };
    
    if obj.is_empty() {
        let error = ErrorResponse {
            error: "Payload cannot be empty".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }
    
    // Build column names and parameter placeholders
    let columns: Vec<String> = obj.keys().cloned().collect();
    let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("?{}", i)).collect();
    
    // Convert JSON values to ORM QueryValue
    let mut params = Vec::new();
    for col in &columns {
        let json_val = &obj[col];
        let query_val = json_to_query_value(json_val);
        params.push(query_val);
    }
    
    // Build INSERT query
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        placeholders.join(", ")
    );
    
    match backend.execute(&sql, &params).await {
        Ok(rows_affected) => {
            let response = serde_json::json!({
                "success": true,
                "rows_affected": rows_affected,
            });
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to insert into table '{}': {}", table, e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Helper function to convert serde_json::Value to orm::query::QueryValue
fn json_to_query_value(val: &JsonValue) -> orm::query::QueryValue {
    match val {
        JsonValue::Null => orm::query::QueryValue::Null,
        JsonValue::Bool(b) => orm::query::QueryValue::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                orm::query::QueryValue::I64(i)
            } else if let Some(f) = n.as_f64() {
                orm::query::QueryValue::F64(f)
            } else {
                orm::query::QueryValue::Null
            }
        }
        JsonValue::String(s) => orm::query::QueryValue::String(s.clone()),
        _ => orm::query::QueryValue::Null,
    }
}
