use thiserror::Error;

#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("Invalid license key format")]
    InvalidFormat,
    
    #[error("License signature verification failed")]
    InvalidSignature,
    
    #[error("License has expired")]
    Expired,
    
    #[error("License type not recognized")]
    InvalidType,
    
    #[error("License key not found")]
    NotFound,
    
    #[error("Maximum device limit reached")]
    DeviceLimitExceeded,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl From<sqlx::Error> for LicenseError {
    fn from(e: sqlx::Error) -> Self {
        LicenseError::DatabaseError(e.to_string())
    }
}

impl From<std::io::Error> for LicenseError {
    fn from(e: std::io::Error) -> Self {
        LicenseError::IoError(e.to_string())
    }
}

impl From<sodiumoxide::crypto::sign::Error> for LicenseError {
    fn from(_e: sodiumoxide::crypto::sign::Error) -> Self {
        LicenseError::CryptoError("Signature verification failed".to_string())
    }
}
