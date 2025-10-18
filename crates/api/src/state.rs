use auth::AuthService;
use core::Database;
use storage::TransactionalStorageService;

/// Application state shared across all handlers
pub struct AppState {
    pub db: Database,
    pub auth_service: AuthService,
    pub storage_service: TransactionalStorageService,
}

impl AppState {
    pub fn new(db: Database, auth_service: AuthService, storage_service: TransactionalStorageService) -> Self {
        Self { 
            db, 
            auth_service,
            storage_service,
        }
    }
}
