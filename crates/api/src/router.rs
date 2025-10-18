use axum::{Router, routing::{get, post}, middleware};
use std::sync::Arc;

use crate::{auth_handlers, db_handlers, middleware as auth_middleware, AppState};

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

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(service_routes)
        .merge(db_routes)
        .with_state(state)
}