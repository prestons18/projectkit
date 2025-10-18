use axum::{Router, routing::{get, post}};
use std::sync::Arc;

use crate::{auth_handlers, db_handlers, AppState};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { "Project Kit API running" }))
        // Auth routes
        .route("/auth/signup", post(auth_handlers::signup))
        .route("/auth/login", post(auth_handlers::login))
        // Database routes
        .route("/db/{table}", get(db_handlers::get_table))
        .route("/db/{table}", post(db_handlers::post_table))
        .with_state(state)
}