// Core modules
mod error;
mod password;
mod jwt;

// ORM-integrated modules
pub mod model;
pub mod service;

// Re-export error types
pub use error::{AuthError, Result};

// Re-export crypto primitives (for standalone use without ORM)
pub use password::{hash_password, verify_password};
pub use jwt::{generate_token, validate_token, Claims};

// Re-export ORM-integrated types
pub use model::{User, Session, Role};
pub use service::AuthService;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        AuthError, Result,
        AuthService,
        User, Session, Role,
        Claims,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_token() {
        let secret = "test_secret_key_for_jwt";
        let user_id = "user_123";
        
        let token = generate_token(user_id, Role::User, secret, 3600).unwrap();
        let claims = validate_token(&token, secret).unwrap();
        
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, Role::User);
    }
}