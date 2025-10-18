use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
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
