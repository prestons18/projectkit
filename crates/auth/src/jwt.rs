use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::{AuthError, Result};
use crate::model::Role;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User role
    pub role: Role,
    /// Issued at (timestamp)
    pub iat: i64,
    /// Expiration time (timestamp)
    pub exp: i64,
}

impl Claims {
    /// Create new claims with the given subject, role, and expiration duration in seconds
    pub fn new(subject: String, role: Role, expires_in_seconds: i64) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::seconds(expires_in_seconds);
        
        Self {
            sub: subject,
            role,
            iat: now.timestamp(),
            exp: expiration.timestamp(),
        }
    }
    
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// Generate a JWT token for a user
/// 
/// # Arguments
/// * `user_id` - The user identifier
/// * `role` - The user's role
/// * `secret` - The secret key for signing the token
/// * `expires_in_seconds` - Token expiration time in seconds (e.g., 3600 for 1 hour)
pub fn generate_token(user_id: &str, role: Role, secret: &str, expires_in_seconds: i64) -> Result<String> {
    let claims = Claims::new(user_id.to_string(), role, expires_in_seconds);
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
}

/// Validate a JWT token and return the claims
/// 
/// # Arguments
/// * `token` - The JWT token to validate
/// * `secret` - The secret key used to sign the token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AuthError::TokenValidationError(e.to_string()))?;
    
    let claims = token_data.claims;
    
    if claims.is_expired() {
        return Err(AuthError::TokenExpired);
    }
    
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_validation() {
        let secret = "test_secret";
        let user_id = "user_123";
        
        let token = generate_token(user_id, Role::User, secret, 3600).unwrap();
        let claims = validate_token(&token, secret).unwrap();
        
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, Role::User);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_invalid_secret() {
        let secret = "correct_secret";
        let wrong_secret = "wrong_secret";
        let user_id = "user_123";
        
        let token = generate_token(user_id, Role::User, secret, 3600).unwrap();
        let result = validate_token(&token, wrong_secret);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let secret = "test_secret";
        let user_id = "user_123";
        
        // Create a token that expires in -1 seconds (already expired)
        let token = generate_token(user_id, Role::User, secret, -1).unwrap();
        
        // Wait a moment to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let result = validate_token(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new("user_456".to_string(), Role::Service, 3600);
        
        assert_eq!(claims.sub, "user_456");
        assert_eq!(claims.role, Role::Service);
        assert!(!claims.is_expired());
        assert!(claims.exp > claims.iat);
    }
}