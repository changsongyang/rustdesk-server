use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub device_id: String,
    pub name: String,
    pub hostname: Option<String>,
    pub os_type: Option<String>,
    pub os_version: Option<String>,
    pub ip_address: Option<String>,
    pub status: DeviceStatus,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
    pub approved: bool,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub last_online: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Away,
    Unknown,
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Away => write!(f, "away"),
            DeviceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCreateRequest {
    pub device_id: String,
    pub name: String,
    pub hostname: Option<String>,
    pub os_type: Option<String>,
    pub os_version: Option<String>,
    pub ip_address: Option<String>,
    pub organization_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUpdateRequest {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub os_type: Option<String>,
    pub os_version: Option<String>,
    pub ip_address: Option<String>,
    pub status: Option<DeviceStatus>,
    pub approved: Option<bool>,
}
