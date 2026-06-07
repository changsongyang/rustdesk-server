use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device not found")]
    NotFound,
    
    #[error("Device already exists")]
    AlreadyExists,
    
    #[error("Invalid device status")]
    InvalidStatus,
    
    #[error("Device not approved")]
    NotApproved,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for DeviceError {
    fn from(e: sqlx::Error) -> Self {
        DeviceError::DatabaseError(e.to_string())
    }
}
