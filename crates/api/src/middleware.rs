use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;
use auth::{Role, User};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Extract and validate JWT token from Authorization header
pub async fn extract_user_from_token(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<User, Response> {
    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            let error = ErrorResponse {
                error: "Missing or invalid Authorization header".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        })?;

    // Validate token and get user
    state
        .auth_service
        .validate(token)
        .await
        .map_err(|e| {
            let error = ErrorResponse {
                error: format!("Invalid token: {}", e),
            };
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        })
}

/// Middleware to require authentication
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let user = extract_user_from_token(&state, request.headers()).await?;
    
    // Store user in request extensions for handlers to access
    request.extensions_mut().insert(user);
    
    Ok(next.run(request).await)
}

/// Middleware to require a specific role
pub fn require_role(required_role: Role) -> impl Fn(State<Arc<AppState>>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone {
    move |State(state): State<Arc<AppState>>, mut request: Request, next: Next| {
        let required_role = required_role;
        Box::pin(async move {
            let user = extract_user_from_token(&state, request.headers()).await?;
            
            // Check if user has required role
            if !user.has_role(required_role) {
                let error = ErrorResponse {
                    error: format!("Access denied. Required role: {:?}", required_role),
                };
                return Err((StatusCode::FORBIDDEN, Json(error)).into_response());
            }
            
            // Store user in request extensions for handlers to access
            request.extensions_mut().insert(user);
            
            Ok(next.run(request).await)
        })
    }
}

/// Middleware to require user role (regular users only)
pub async fn require_user_role(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let user = extract_user_from_token(&state, request.headers()).await?;
    
    if !user.is_user() {
        let error = ErrorResponse {
            error: "Access denied. User role required".to_string(),
        };
        return Err((StatusCode::FORBIDDEN, Json(error)).into_response());
    }
    
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

/// Middleware to require service role (service accounts only)
pub async fn require_service_role(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let user = extract_user_from_token(&state, request.headers()).await?;
    
    if !user.is_service() {
        let error = ErrorResponse {
            error: "Access denied. Service role required".to_string(),
        };
        return Err((StatusCode::FORBIDDEN, Json(error)).into_response());
    }
    
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

/// Extension trait to extract authenticated user from request
pub trait AuthenticatedUser {
    fn user(&self) -> Option<&User>;
}

impl AuthenticatedUser for Request {
    fn user(&self) -> Option<&User> {
        self.extensions().get::<User>()
    }
}

/// Extractor for authenticated user
/// Use this in handlers that are protected by auth middleware
#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .map(AuthUser)
            .ok_or_else(|| {
                let error = ErrorResponse {
                    error: "User not authenticated".to_string(),
                };
                (StatusCode::UNAUTHORIZED, Json(error))
            })
    }
}