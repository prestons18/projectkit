use crate::{
    error::{AuthError, Result},
    jwt::{generate_token, validate_token},
    model::{Session, User, Role},
    password::{hash_password, verify_password},
};
use chrono::{Duration, Utc};
use orm::prelude::*;

/// Authentication service that integrates ORM with auth logic
pub struct AuthService {
    db: Database,
    jwt_secret: String,
    token_expiry_seconds: i64,
}

impl AuthService {
    /// Create a new AuthService
    /// 
    /// # Arguments
    /// * `db` - Database connection from ORM
    /// * `jwt_secret` - Secret key for JWT signing
    /// * `token_expiry_seconds` - Token expiration time in seconds (default: 3600 for 1 hour)
    pub fn new(db: Database, jwt_secret: String, token_expiry_seconds: i64) -> Self {
        Self {
            db,
            jwt_secret,
            token_expiry_seconds,
        }
    }

    /// Register a new user with default role (User)
    /// 
    /// # Arguments
    /// * `email` - User's email address
    /// * `password` - User's plain text password (will be hashed)
    pub async fn signup(&self, email: &str, password: &str) -> Result<User> {
        self.signup_with_role(email, password, Role::User).await
    }

    /// Register a new user with specified role
    /// 
    /// # Arguments
    /// * `email` - User's email address
    /// * `password` - User's plain text password (will be hashed)
    /// * `role` - User's role (User or Service)
    pub async fn signup_with_role(&self, email: &str, password: &str, role: Role) -> Result<User> {
        // Check if user already exists
        let existing = self.find_user_by_email(email).await?;
        if existing.is_some() {
            return Err(AuthError::TokenValidationError("User already exists".to_string()));
        }

        // Hash password
        let password_hash = hash_password(password)?;

        // Create user with specified role
        let mut user = User::new_with_role(email.to_string(), password_hash, role);

        // Insert into database
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();
        
        let values = user.to_values();
        let columns: Vec<&str> = values.keys().map(|s| s.as_str()).collect();
        let query_values: Vec<_> = values.values().map(|v| v.to_query_value()).collect();
        
        query_builder.insert_into(User::table_name(), &columns);
        query_builder.values_params(&query_values);
        
        // Use RETURNING clause for SQLite or LAST_INSERT_ID() for MySQL
        if backend.supports_feature(orm::backend::BackendFeature::Returning) {
            // SQLite: Use RETURNING clause
            query_builder.returning(&["id", "email", "password_hash", "role", "created_at", "updated_at"]);
            let sql = query_builder.build()
                .map_err(|e| AuthError::TokenGenerationError(format!("Query build error: {}", e)))?;
            
            let result = backend.fetch_one_params(&sql, query_builder.params()).await
                .map_err(|e| AuthError::TokenGenerationError(format!("Database error: {}", e)))?;
            
            match result {
                Some(json) => {
                    user = User::from_json(&json)
                        .map_err(|e| AuthError::TokenGenerationError(format!("Deserialization error: {}", e)))?;
                }
                None => return Err(AuthError::TokenGenerationError("Failed to create user".to_string())),
            }
        } else {
            // MySQL: Execute insert, then fetch LAST_INSERT_ID()
            let sql = query_builder.build()
                .map_err(|e| AuthError::TokenGenerationError(format!("Query build error: {}", e)))?;
            
            backend.execute(&sql, query_builder.params()).await
                .map_err(|e| AuthError::TokenGenerationError(format!("Database error: {}", e)))?;
            
            // Get the last inserted ID
            let last_id_sql = "SELECT LAST_INSERT_ID() as id";
            #[allow(deprecated)]
            let result = backend.fetch_one(last_id_sql).await
                .map_err(|e| AuthError::TokenGenerationError(format!("Failed to get last insert ID: {}", e)))?;
            
            match result {
                Some(json) => {
                    let id = json.get("id")
                        .and_then(|v| v.as_i64())
                        .ok_or_else(|| AuthError::TokenGenerationError("Invalid ID returned".to_string()))?;
                    user.id = Some(id);
                    
                    // Fetch the complete user record
                    user = self.find_user_by_id(id).await?
                        .ok_or_else(|| AuthError::TokenGenerationError("Failed to fetch created user".to_string()))?;
                }
                None => return Err(AuthError::TokenGenerationError("Failed to get last insert ID".to_string())),
            }
        }

        Ok(user)
    }

    /// Login a user and return a JWT token
    /// 
    /// # Arguments
    /// * `email` - User's email address
    /// * `password` - User's plain text password
    pub async fn login(&self, email: &str, password: &str) -> Result<(String, User)> {
        // Find user by email
        let user = self.find_user_by_email(email).await?
            .ok_or(AuthError::InvalidPassword)?;

        // Verify password
        if !verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidPassword);
        }

        // Generate JWT token with user's role
        let user_id_str = user.id
            .ok_or(AuthError::TokenGenerationError("User has no ID".to_string()))?
            .to_string();
        
        let token = generate_token(&user_id_str, user.role, &self.jwt_secret, self.token_expiry_seconds)?;

        // Optionally store session in database
        let expires_at = Utc::now() + Duration::seconds(self.token_expiry_seconds);
        let session = Session::new(user.id.unwrap(), token.clone(), expires_at);
        
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();
        
        let values = session.to_values();
        let columns: Vec<&str> = values.keys().map(|s| s.as_str()).collect();
        let query_values: Vec<_> = values.values().map(|v| v.to_query_value()).collect();
        
        query_builder.insert_into(Session::table_name(), &columns);
        query_builder.values_params(&query_values);
        
        if let Ok(sql) = query_builder.build() {
            let _ = backend.execute(&sql, query_builder.params()).await;
        }

        Ok((token, user))
    }

    /// Validate a JWT token and return the user
    /// Also verifies that the role in the token matches the user's current role
    pub async fn validate(&self, token: &str) -> Result<User> {
        // Validate JWT
        let claims = validate_token(token, &self.jwt_secret)?;

        // Parse user ID from claims
        let user_id: i64 = claims.sub.parse()
            .map_err(|_| AuthError::InvalidToken)?;

        // Find user by ID
        let user = self.find_user_by_id(user_id).await?
            .ok_or(AuthError::InvalidToken)?;

        // Verify role hasn't changed
        if user.role != claims.role {
            return Err(AuthError::TokenValidationError(
                "User role has changed, please login again".to_string()
            ));
        }

        Ok(user)
    }

    /// Validate a JWT token and return both the user and claims
    pub async fn validate_with_claims(&self, token: &str) -> Result<(User, crate::jwt::Claims)> {
        let claims = validate_token(token, &self.jwt_secret)?;
        let user = self.validate(token).await?;
        Ok((user, claims))
    }

    /// Logout a user by invalidating their session
    pub async fn logout(&self, token: &str) -> Result<()> {
        // Delete session from database
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();
        
        query_builder.delete_from(Session::table_name());
        query_builder.where_eq("token", orm::query::QueryValue::String(token.to_string()));
        
        let sql = query_builder.build()
            .map_err(|e| AuthError::TokenValidationError(format!("Query build error: {}", e)))?;
        
        backend.execute(&sql, query_builder.params()).await
            .map_err(|e| AuthError::TokenValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    /// Find user by email
    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();
        
        query_builder.from(User::table_name());
        query_builder.select(&[]);
        query_builder.where_eq("email", orm::query::QueryValue::String(email.to_string()));
        query_builder.limit(1);
        
        let sql = query_builder.build()
            .map_err(|e| AuthError::TokenValidationError(format!("Query build error: {}", e)))?;
        
        let json_rows = backend.fetch_all_params(&sql, query_builder.params()).await
            .map_err(|e| AuthError::TokenValidationError(format!("Database error: {}", e)))?;

        if json_rows.is_empty() {
            return Ok(None);
        }

        let user = User::from_json(&json_rows[0])
            .map_err(|e| AuthError::TokenValidationError(format!("Deserialization error: {}", e)))?;

        Ok(Some(user))
    }

    /// Find user by ID
    async fn find_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let backend = self.db.backend();
        let mut query_builder = backend.query_builder();
        
        query_builder.from(User::table_name());
        query_builder.select(&[]);
        query_builder.where_eq("id", orm::query::QueryValue::I64(id));
        query_builder.limit(1);
        
        let sql = query_builder.build()
            .map_err(|e| AuthError::TokenValidationError(format!("Query build error: {}", e)))?;
        
        let json_rows = backend.fetch_all_params(&sql, query_builder.params()).await
            .map_err(|e| AuthError::TokenValidationError(format!("Database error: {}", e)))?;

        if json_rows.is_empty() {
            return Ok(None);
        }

        let user = User::from_json(&json_rows[0])
            .map_err(|e| AuthError::TokenValidationError(format!("Deserialization error: {}", e)))?;

        Ok(Some(user))
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let now = Utc::now().to_rfc3339();
        
        let backend = self.db.backend();
        let sql = format!("DELETE FROM {} WHERE expires_at < '{}'", Session::table_name(), now);
        
        let rows_affected = backend.execute(&sql, &[]).await
            .map_err(|e| AuthError::TokenValidationError(format!("Database error: {}", e)))?;

        Ok(rows_affected)
    }
    
    /// Get the database backend (for seeding and admin operations)
    pub fn db_backend(&self) -> &dyn orm::backend::Backend {
        self.db.backend()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a real database connection
    // In a real application, you'd use a test database or mocking

    #[tokio::test]
    #[ignore] // Ignore by default since it needs a database
    async fn test_signup_and_login() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        
        // Create users table
        let create_table = r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#;
        db.execute(create_table).await.unwrap();

        let service = AuthService::new(db, "test_secret".to_string(), 3600);

        // Signup
        let user = service.signup("test@example.com", "password123").await.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert!(user.id.is_some());

        // Login
        let (token, logged_in_user) = service.login("test@example.com", "password123").await.unwrap();
        assert!(!token.is_empty());
        assert_eq!(logged_in_user.email, user.email);

        // Validate token
        let validated_user = service.validate(&token).await.unwrap();
        assert_eq!(validated_user.email, user.email);
    }
}
