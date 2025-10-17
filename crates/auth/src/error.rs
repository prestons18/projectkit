use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Password hashing failed: {0}")]
    HashingError(String),
    
    #[error("Password verification failed")]
    VerificationError,
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),
    
    #[error("Token validation failed: {0}")]
    TokenValidationError(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid token")]
    InvalidToken,
}

pub type Result<T> = std::result::Result<T, AuthError>;
