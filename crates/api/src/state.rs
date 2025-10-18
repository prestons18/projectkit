use auth::AuthService;
use core::Database;

/// Application state shared across all handlers
pub struct AppState {
    pub db: Database,
    pub auth_service: AuthService,
}

impl AppState {
    pub fn new(db: Database, auth_service: AuthService) -> Self {
        Self { db, auth_service }
    }
}
