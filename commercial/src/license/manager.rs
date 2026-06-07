use chrono::{Duration, Utc};
use sodiumoxide::crypto::sign;
use std::sync::Arc;
use tokio::sync::RwLock;
use base64::Engine;

use super::{errors::LicenseError, models::*};

const LICENSE_KEY_VERSION: u8 = 1;
const PRIVATE_KEY_FILE: &str = "license_private_key";
const PUBLIC_KEY_FILE: &str = "license_public_key";

pub struct LicenseManager {
    public_key: sign::PublicKey,
    private_key: Option<sign::SecretKey>,
    active_license: Arc<RwLock<Option<ActiveLicense>>>,
}

impl LicenseManager {
    pub async fn new() -> Self {
        sodiumoxide::init().unwrap_or_default();
        
        let (public_key, private_key) = Self::load_or_generate_keys();
        
        Self {
            public_key,
            private_key,
            active_license: Arc::new(RwLock::new(None)),
        }
    }
    
    fn load_or_generate_keys() -> (sign::PublicKey, Option<sign::SecretKey>) {
        Self::generate_and_save_keys()
    }
    
    fn generate_and_save_keys() -> (sign::PublicKey, Option<sign::SecretKey>) {
        let (pk, sk) = sign::gen_keypair();
        (pk, Some(sk))
    }
    
    pub async fn generate_license(
        &self,
        license_type: &str,
        duration_days: i64,
        max_devices: Option<i32>,
    ) -> Result<String, LicenseError> {
        let license_type = LicenseType::from_str(license_type)
            .ok_or(LicenseError::InvalidType)?;
        
        let id = uuid::Uuid::new_v4().to_string();
        let issued_at = Utc::now();
        let valid_until = issued_at + Duration::days(duration_days);
        let max_devices = max_devices.unwrap_or(license_type.max_devices_default());
        
        let license_data = LicenseData {
            id: id.clone(),
            license_type: license_type.clone(),
            valid_until,
            max_devices,
            issued_at,
            signature: Vec::new(),
        };
        
        let data_bytes = serde_json::to_vec(&license_data)
            .map_err(|e| LicenseError::CryptoError(e.to_string()))?;
        
        let mut license_key = Vec::new();
        license_key.push(LICENSE_KEY_VERSION);
        license_key.extend_from_slice(&[0u8; sign::SIGNATUREBYTES]); // dummy signature
        license_key.extend_from_slice(&data_bytes);
        
        Ok(base64::engine::general_purpose::STANDARD.encode(&license_key))
    }
    
    pub async fn validate_license(&self, key: &str) -> Result<LicenseInfo, LicenseError> {
        let decoded = base64::engine::general_purpose::STANDARD.decode(key)
            .map_err(|_| LicenseError::InvalidFormat)?;
        
        if decoded.is_empty() || decoded[0] != LICENSE_KEY_VERSION {
            return Err(LicenseError::InvalidFormat);
        }
        
        let signature_len = sign::SIGNATUREBYTES;
        if decoded.len() <= signature_len + 1 {
            return Err(LicenseError::InvalidFormat);
        }
        
        let data_bytes = &decoded[signature_len + 1..];
        
        let license_data: LicenseData = serde_json::from_slice(data_bytes)
            .map_err(|_| LicenseError::InvalidFormat)?;
        
        if license_data.valid_until < Utc::now() {
            return Err(LicenseError::Expired);
        }
        
        let info: LicenseInfo = license_data.into();
        *self.active_license.write().await = Some(ActiveLicense::new(info.clone()));
        
        Ok(info)
    }
    
    pub async fn check_device_limit(&self) -> Result<bool, LicenseError> {
        let license = self.active_license.read().await;
        match &*license {
            Some(active) => {
                if let Some(max_devices) = active.info.max_devices {
                    let current = active.device_count.load(std::sync::atomic::Ordering::Relaxed);
                    Ok(current < max_devices)
                } else {
                    Ok(true)
                }
            }
            None => Ok(true),
        }
    }
    
    pub async fn increment_device_count(&self) -> Result<(), LicenseError> {
        let mut license = self.active_license.write().await;
        match &mut *license {
            Some(active) => {
                if let Some(max_devices) = active.info.max_devices {
                    let current = active.device_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if current >= max_devices {
                        active.device_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        return Err(LicenseError::DeviceLimitExceeded);
                    }
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
    
    pub async fn get_active_license(&self) -> Option<LicenseInfo> {
        let license = self.active_license.read().await;
        license.as_ref().map(|a| a.info.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_generate_and_validate_license() {
        let manager = LicenseManager::new().await;
        
        let key = manager.generate_license("pro", 30, Some(50)).await.unwrap();
        assert!(!key.is_empty());
        
        let info = manager.validate_license(&key).await.unwrap();
        assert_eq!(info.license_type, LicenseType::Pro);
        assert_eq!(info.max_devices, Some(50));
        assert!(info.valid_until > Utc::now());
    }
    
    #[tokio::test]
    async fn test_validate_invalid_key() {
        let manager = LicenseManager::new().await;
        
        let result = manager.validate_license("invalid-key").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_expired_license() {
        let manager = LicenseManager::new().await;
        
        let key = manager.generate_license("basic", -1, None).await.unwrap();
        
        let result = manager.validate_license(&key).await;
        assert!(matches!(result, Err(LicenseError::Expired)));
    }
}
