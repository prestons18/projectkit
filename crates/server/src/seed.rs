use auth::{AuthService, Role};
use orm::error::Result;

/// Seed the database with initial data
pub async fn seed_database(auth_service: &AuthService) -> Result<()> {
    println!("🌱 Checking for initial service account...");
    
    // Check if any service accounts exist
    let backend = auth_service.db_backend();
    let check_sql = "SELECT COUNT(*) as count FROM users WHERE role = 'service'";
    
    #[allow(deprecated)]
    let result = backend.fetch_one(check_sql).await?;
    
    let count = result
        .and_then(|json| json.get("count").and_then(|v| v.as_i64()))
        .unwrap_or(0);
    
    if count == 0 {
        println!("   No service accounts found. Creating default service account...");
        
        // Create default service account
        let email = "admin@projectkit.local";
        let password = "admin123"; // Change this in production!
        
        match auth_service.signup_with_role(email, password, Role::Service).await {
            Ok(user) => {
                println!("   ✓ Created service account: {}", email);
                println!("   ⚠️  Default password: {}", password);
                println!("   ⚠️  IMPORTANT: Change this password in production!");
                println!("   User ID: {:?}", user.id);
            }
            Err(e) => {
                println!("   ✗ Failed to create service account: {}", e);
                return Err(orm::error::Error::QueryError(format!("Seed failed: {}", e)));
            }
        }
    } else {
        println!("   ✓ Service account(s) already exist (count: {})", count);
    }
    
    Ok(())
}
