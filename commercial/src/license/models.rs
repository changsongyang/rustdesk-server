use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub id: String,
    pub license_type: LicenseType,
    pub valid_until: DateTime<Utc>,
    pub max_devices: Option<i32>,
    pub issued_at: DateTime<Utc>,
    pub is_trial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LicenseType {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "pro")]
    Pro,
    #[serde(rename = "enterprise")]
    Enterprise,
}

impl LicenseType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "basic" => Some(Self::Basic),
            "pro" => Some(Self::Pro),
            "enterprise" => Some(Self::Enterprise),
            _ => None,
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Pro => "pro",
            Self::Enterprise => "enterprise",
        }
    }
    
    pub fn max_devices_default(&self) -> i32 {
        match self {
            Self::Basic => 10,
            Self::Pro => 100,
            Self::Enterprise => i32::MAX,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub id: String,
    pub license_type: LicenseType,
    pub valid_until: DateTime<Utc>,
    pub max_devices: i32,
    pub issued_at: DateTime<Utc>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ActiveLicense {
    pub info: LicenseInfo,
    pub device_count: Arc<std::sync::atomic::AtomicI32>,
}

impl ActiveLicense {
    pub fn new(info: LicenseInfo) -> Self {
        Self {
            info,
            device_count: Arc::new(std::sync::atomic::AtomicI32::new(0)),
        }
    }
}

impl From<LicenseData> for LicenseInfo {
    fn from(data: LicenseData) -> Self {
        Self {
            id: data.id,
            license_type: data.license_type,
            valid_until: data.valid_until,
            max_devices: Some(data.max_devices),
            issued_at: data.issued_at,
            is_trial: false,
        }
    }
}
