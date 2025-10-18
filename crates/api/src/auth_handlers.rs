use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::middleware::AuthUser;
use auth::Role;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceAccountRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Option<i64>,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupRequest>,
) -> impl IntoResponse {
    match state.auth_service.signup(&payload.email, &payload.password).await {
        Ok(_user) => {
            // After signup, automatically log them in
            match state.auth_service.login(&payload.email, &payload.password).await {
                Ok((token, user)) => {
                    let response = AuthResponse {
                        token,
                        user: UserResponse {
                            id: user.id,
                            email: user.email,
                        },
                    };
                    (StatusCode::CREATED, Json(response)).into_response()
                }
                Err(e) => {
                    let error = ErrorResponse {
                        error: format!("Signup succeeded but login failed: {}", e),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Signup failed: {}", e),
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.auth_service.login(&payload.email, &payload.password).await {
        Ok((token, user)) => {
            let response = AuthResponse {
                token,
                user: UserResponse {
                    id: user.id,
                    email: user.email,
                },
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Login failed: {}", e),
            };
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
    }
}

/// Create a service account - requires existing service account authentication
pub async fn create_service_account(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateServiceAccountRequest>,
) -> impl IntoResponse {
    // Only service accounts can create other service accounts
    if !user.is_service() {
        let error = ErrorResponse {
            error: "Access denied. Service role required to create service accounts.".to_string(),
        };
        return (StatusCode::FORBIDDEN, Json(error)).into_response();
    }
    
    // Create the service account
    match state.auth_service.signup_with_role(&payload.email, &payload.password, Role::Service).await {
        Ok(_user) => {
            // After creation, automatically log them in
            match state.auth_service.login(&payload.email, &payload.password).await {
                Ok((token, user)) => {
                    let response = AuthResponse {
                        token,
                        user: UserResponse {
                            id: user.id,
                            email: user.email,
                        },
                    };
                    (StatusCode::CREATED, Json(response)).into_response()
                }
                Err(e) => {
                    let error = ErrorResponse {
                        error: format!("Service account created but login failed: {}", e),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to create service account: {}", e),
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}
