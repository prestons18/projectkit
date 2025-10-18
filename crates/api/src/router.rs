use axum::{Router, routing::{get, post, delete}, middleware};
use std::sync::Arc;

use crate::{auth_handlers, db_handlers, file_handlers, middleware as auth_middleware, AppState};

pub fn router(state: Arc<AppState>) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/", get(|| async { "Project Kit API running" }))
        .route("/auth/signup", post(auth_handlers::signup))
        .route("/auth/login", post(auth_handlers::login));

    // Protected auth routes (require service role)
    let service_routes = Router::new()
        .route("/auth/service", post(auth_handlers::create_service_account))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::require_service_role,
        ));

    // Protected database routes (require authentication)
    let db_routes = Router::new()
        .route("/db/{table}", get(db_handlers::get_table))
        .route("/db/{table}", post(db_handlers::post_table))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::require_auth,
        ));

    // Protected file routes (require authentication)
    let file_routes = Router::new()
        .route("/files", get(file_handlers::list_files))
        .route("/files/upload", post(file_handlers::upload_file))
        .route("/files/stats", get(file_handlers::get_storage_stats))
        .route("/files/{id}", get(file_handlers::download_file))
        .route("/files/{id}", delete(file_handlers::delete_file))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::require_auth,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(service_routes)
        .merge(db_routes)
        .merge(file_routes)
        .with_state(state)
}