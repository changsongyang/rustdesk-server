use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    
    #[error("Username already exists")]
    UsernameExists,
    
    #[error("Email already exists")]
    EmailExists,
    
    #[error("Invalid username or password")]
    InvalidCredentials,
    
    #[error("Invalid role")]
    InvalidRole,
    
    #[error("Password must be at least 8 characters")]
    PasswordTooShort,
    
    #[error("User is not active")]
    UserNotActive,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("JWT error: {0}")]
    JwtError(String),
    
    #[error("BCrypt error: {0}")]
    BcryptError(String),
}

impl From<sqlx::Error> for UserError {
    fn from(e: sqlx::Error) -> Self {
        UserError::DatabaseError(e.to_string())
    }
}

impl From<bcrypt::BcryptError> for UserError {
    fn from(e: bcrypt::BcryptError) -> Self {
        UserError::BcryptError(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for UserError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        UserError::JwtError(e.to_string())
    }
}
