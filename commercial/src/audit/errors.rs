use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Log not found")]
    NotFound,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<sqlx::Error> for AuditError {
    fn from(e: sqlx::Error) -> Self {
        AuditError::DatabaseError(e.to_string())
    }
}

impl From<serde_json::Error> for AuditError {
    fn from(e: serde_json::Error) -> Self {
        AuditError::SerializationError(e.to_string())
    }
}
