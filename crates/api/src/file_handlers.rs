use axum::{
    extract::{Path, State, Multipart},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::middleware::AuthUser;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: String,
    pub original_name: String,
    pub stored_name: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub file: FileResponse,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

/// POST /files/upload - Upload a file
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Extract file from multipart form data
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut mime_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            mime_type = field.content_type().map(|s| s.to_string());

            match field.bytes().await {
                Ok(bytes) => {
                    file_data = Some(bytes.to_vec());
                }
                Err(e) => {
                    let error = ErrorResponse {
                        error: format!("Failed to read file data: {}", e),
                    };
                    return (StatusCode::BAD_REQUEST, Json(error)).into_response();
                }
            }
        }
    }

    // Validate we got file data
    let data = match file_data {
        Some(d) => d,
        None => {
            let error = ErrorResponse {
                error: "No file provided in request".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    let original_name = file_name.unwrap_or_else(|| "unnamed".to_string());
    let user_id = user.id.unwrap();

    // Store file with metadata
    match state
        .storage_service
        .store_with_metadata(&data, &original_name, user_id, mime_type)
        .await
    {
        Ok(file) => {
            let response = UploadResponse {
                success: true,
                file: FileResponse {
                    id: file.id.unwrap_or_default(),
                    original_name: file.original_name,
                    stored_name: file.stored_name,
                    size: file.size,
                    mime_type: file.mime_type,
                    created_at: file.created_at.to_rfc3339(),
                },
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to upload file: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// GET /files/:id - Download a file
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let user_id = user.id.unwrap();

    // Get file metadata first to check permissions and get original name
    let file = match state.storage_service.get_file_by_id(&file_id).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            let error = ErrorResponse {
                error: "File not found".to_string(),
            };
            return (StatusCode::NOT_FOUND, Json(error)).into_response();
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to fetch file metadata: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
        }
    };

    // Check permission
    if file.user_id != user_id {
        let error = ErrorResponse {
            error: "Access denied: file belongs to another user".to_string(),
        };
        return (StatusCode::FORBIDDEN, Json(error)).into_response();
    }

    // Retrieve file data
    match state
        .storage_service
        .retrieve_with_permission(&file_id, user_id)
        .await
    {
        Ok(data) => {
            let mut headers = axum::http::HeaderMap::new();
            
            // Set content type
            if let Some(mime) = &file.mime_type {
                if let Ok(header_value) = mime.parse() {
                    headers.insert(header::CONTENT_TYPE, header_value);
                }
            }
            
            // Set content disposition with original filename
            let disposition = format!("attachment; filename=\"{}\"", file.original_name);
            if let Ok(header_value) = disposition.parse() {
                headers.insert(header::CONTENT_DISPOSITION, header_value);
            }

            (StatusCode::OK, headers, data).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to download file: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// DELETE /files/:id - Delete a file
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let user_id = user.id.unwrap();

    match state
        .storage_service
        .delete_with_metadata(&file_id, user_id)
        .await
    {
        Ok(_) => {
            let response = DeleteResponse {
                success: true,
                message: format!("File {} deleted successfully", file_id),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to delete file: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// GET /files - List all files for the authenticated user
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let user_id = user.id.unwrap();

    match state.storage_service.list_user_files(user_id).await {
        Ok(files) => {
            let file_responses: Vec<FileResponse> = files
                .into_iter()
                .map(|f| FileResponse {
                    id: f.id.unwrap_or_default(),
                    original_name: f.original_name,
                    stored_name: f.stored_name,
                    size: f.size,
                    mime_type: f.mime_type,
                    created_at: f.created_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(file_responses)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to list files: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// GET /files/stats - Get storage statistics for the authenticated user
pub async fn get_storage_stats(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let user_id = user.id.unwrap();

    match state
        .storage_service
        .get_user_storage_stats(user_id)
        .await
    {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to get storage stats: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}