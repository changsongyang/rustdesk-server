use base64::Engine;
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{errors::LicenseError, models::*};

type HmacSha256 = Hmac<Sha256>;

const LICENSE_KEY_VERSION: u8 = 1;

// 许可证签名密钥（生产环境应从安全存储加载）
const LICENSE_SIGNING_KEY: &[u8] = b"rustdesk-pro-license-v1-signing-key";

pub struct LicenseManager {
    active_license: Arc<RwLock<Option<ActiveLicense>>>,
}

impl LicenseManager {
    pub async fn new() -> Self {
        Self {
            active_license: Arc::new(RwLock::new(None)),
        }
    }

    /// 生成许可证签名
    fn sign_data(data: &[u8]) -> Vec<u8> {
        let mut mac =
            HmacSha256::new_from_slice(LICENSE_SIGNING_KEY).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// 验证许可证签名
    fn verify_signature(data: &[u8], signature: &[u8]) -> bool {
        let mut mac = match HmacSha256::new_from_slice(LICENSE_SIGNING_KEY) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(data);
        mac.verify_slice(signature).is_ok()
    }

    pub async fn generate_license(
        &self,
        license_type: &str,
        duration_days: i64,
        max_devices: Option<i32>,
    ) -> Result<String, LicenseError> {
        let license_type =
            LicenseType::from_str(license_type).map_err(|_| LicenseError::InvalidType)?;

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
            signature: Vec::new(), // 签名将在下面计算
        };

        let data_bytes = serde_json::to_vec(&license_data)
            .map_err(|e| LicenseError::CryptoError(e.to_string()))?;

        // 生成签名
        let signature = Self::sign_data(&data_bytes);

        // 构建许可证密钥：版本(1字节) + 签名(32字节) + 数据(JSON)
        let mut license_key = Vec::new();
        license_key.push(LICENSE_KEY_VERSION);
        license_key.extend_from_slice(&signature);
        license_key.extend_from_slice(&data_bytes);

        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&license_key))
    }

    pub async fn validate_license(&self, key: &str) -> Result<LicenseInfo, LicenseError> {
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(key)
            .map_err(|_| LicenseError::InvalidFormat)?;

        if decoded.is_empty() || decoded[0] != LICENSE_KEY_VERSION {
            return Err(LicenseError::InvalidFormat);
        }

        let signature_len = 32; // HMAC-SHA256 输出长度
        if decoded.len() <= signature_len + 1 {
            return Err(LicenseError::InvalidFormat);
        }

        let signature = &decoded[1..1 + signature_len];
        let data_bytes = &decoded[1 + signature_len..];

        // 验证签名
        if !Self::verify_signature(data_bytes, signature) {
            log::warn!("License signature verification failed");
            return Err(LicenseError::InvalidFormat);
        }

        let license_data: LicenseData =
            serde_json::from_slice(data_bytes).map_err(|_| LicenseError::InvalidFormat)?;

        // 检查签名字段是否为空（兼容旧格式）
        if !license_data.signature.is_empty() {
            return Err(LicenseError::InvalidFormat);
        }

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
                    let current = active
                        .device_count
                        .load(std::sync::atomic::Ordering::Relaxed);
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
                    let current = active
                        .device_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if current >= max_devices {
                        active
                            .device_count
                            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
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
