use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub log_type: AuditLogType,
    pub action: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditLogType {
    Authentication,
    Authorization,
    Device,
    User,
    License,
    System,
    Session,
    Configuration,
}

impl fmt::Display for AuditLogType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuditLogType::Authentication => write!(f, "authentication"),
            AuditLogType::Authorization => write!(f, "authorization"),
            AuditLogType::Device => write!(f, "device"),
            AuditLogType::User => write!(f, "user"),
            AuditLogType::License => write!(f, "license"),
            AuditLogType::System => write!(f, "system"),
            AuditLogType::Session => write!(f, "session"),
            AuditLogType::Configuration => write!(f, "configuration"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRequest {
    pub log_type: AuditLogType,
    pub action: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
}
