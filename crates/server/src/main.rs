use api::{router, AppState};
use auth::AuthService;
use projectkit_core::{AppConfig, Database};
use storage::{StorageService, TransactionalStorageService};
use std::sync::Arc;

mod migrations;
mod seed;

#[tokio::main]
async fn main() {
    // Load configuration from projectkit.toml with environment variable overrides
    let (config, overrides) = AppConfig::load_with_env()
        .expect("Failed to load configuration. Make sure projectkit.toml exists or set PROJECTKIT_* environment variables.");
    
    println!("📦 Loaded config from projectkit.toml");
    if !overrides.is_empty() {
        println!("🌍 Overridden from environment:");
        for override_key in &overrides {
            println!("  - {}", override_key);
        }
    }
    
    // Connect to database
    let db = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations (only prints if migrations are executed)
    let dialect = if config.database.url.starts_with("sqlite") {
        orm::query::builder::Dialect::SQLite
    } else {
        orm::query::builder::Dialect::MySQL
    };
    
    let _ = migrations::run_migrations(db.backend(), dialect)
        .await
        .expect("Failed to run migrations");
    
    // Connect second database instance for auth service
    let db_for_auth = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database for auth");
    
    // Initialize auth service
    let auth_service = AuthService::new(
        db_for_auth,
        config.auth.jwt_secret.clone(),
        config.auth.token_expiry_seconds
    );
    
    // Seed database with initial data (creates default service account if needed)
    let _ = seed::seed_database(&auth_service)
        .await
        .expect("Failed to seed database");
    
    // Initialize storage service
    let storage_base_path = std::env::var("PROJECTKIT_STORAGE_PATH")
        .unwrap_or_else(|_| "./storage".to_string());
    
    let storage = StorageService::new(&storage_base_path)
        .await
        .expect("Failed to initialize storage service");
    
    println!("💾 Storage initialized at: {}", storage_base_path);
    
    // Connect third database instance for storage service
    let db_for_storage = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database for storage");
    
    let storage_service = TransactionalStorageService::new(storage, db_for_storage);
    
    // Create app state
    let state = Arc::new(AppState::new(db, auth_service, storage_service));
    
    // Create router with state
    let app = router::router(state);
    
    // Start server
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect(&format!("Failed to bind to {}", bind_addr));
    let addr = listener.local_addr().unwrap();
    
    println!("🚀 Running on http://{}", addr);
    println!();
    
    axum::serve(listener, app).await.unwrap();
}