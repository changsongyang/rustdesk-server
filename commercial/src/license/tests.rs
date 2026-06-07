#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::models::*;
    use crate::db::init::init_database;
    use std::env;

    #[tokio::test]
    async fn test_generate_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let result = manager.generate_license("pro", 30, Some(10)).await;
        
        assert!(result.is_ok());
        
        let key = result.unwrap();
        assert!(!key.is_empty());
        assert!(key.len() > 10);
    }

    #[tokio::test]
    async fn test_generate_basic_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let result = manager.generate_license("basic", 365, None).await;
        
        assert!(result.is_ok());
        
        let key = result.unwrap();
        assert!(!key.is_empty());
    }

    #[tokio::test]
    async fn test_generate_enterprise_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let result = manager.generate_license("enterprise", 365, Some(1000)).await;
        
        assert!(result.is_ok());
        
        let key = result.unwrap();
        assert!(!key.is_empty());
    }

    #[tokio::test]
    async fn test_validate_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let key = manager.generate_license("pro", 30, Some(50)).await.unwrap();
        
        let result = manager.validate_license(&key).await;
        
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert_eq!(info.license_type, "pro");
        assert_eq!(info.max_devices, Some(50));
        assert!(info.is_valid);
    }

    #[tokio::test]
    async fn test_validate_invalid_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let result = manager.validate_license("invalid_license_key_123").await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_expired_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        // Generate a license that expires immediately (duration 0 days)
        let key = manager.generate_license("basic", 0, None).await.unwrap();
        
        let result = manager.validate_license(&key).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_active_license() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        // Initially no active license
        let result = manager.get_active_license().await;
        assert!(result.is_none());
        
        // Generate and activate a license
        let key = manager.generate_license("pro", 90, Some(100)).await.unwrap();
        manager.validate_license(&key).await.unwrap();
        
        // Now there should be an active license
        let active = manager.get_active_license().await;
        assert!(active.is_some());
        assert_eq!(active.unwrap().license_type, "pro");
    }

    #[tokio::test]
    async fn test_license_types() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let types = ["basic", "pro", "enterprise"];
        
        for license_type in &types {
            let key = manager.generate_license(license_type, 30, Some(10)).await;
            assert!(key.is_ok(), "Failed to generate {} license", license_type);
            
            let validate_result = manager.validate_license(&key.unwrap()).await;
            assert!(validate_result.is_ok(), "Failed to validate {} license", license_type);
            
            let info = validate_result.unwrap();
            assert_eq!(info.license_type, *license_type);
        }
    }

    #[tokio::test]
    async fn test_license_max_devices() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        // Test with max_devices
        let key1 = manager.generate_license("pro", 30, Some(50)).await.unwrap();
        let info1 = manager.validate_license(&key1).await.unwrap();
        assert_eq!(info1.max_devices, Some(50));
        
        // Test without max_devices (unlimited)
        let key2 = manager.generate_license("enterprise", 365, None).await.unwrap();
        let info2 = manager.validate_license(&key2).await.unwrap();
        assert_eq!(info2.max_devices, None);
    }

    #[tokio::test]
    async fn test_license_format() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
        init_database().unwrap();
        
        let manager = LicenseManager::new().await;
        
        let key = manager.generate_license("pro", 30, Some(10)).await.unwrap();
        
        // License key should be base64 encoded
        let decoded = base64::engine::general_purpose::STANDARD.decode(&key);
        assert!(decoded.is_ok());
        
        // Decoded data should contain valid JSON
        let decoded_str = String::from_utf8(decoded.unwrap());
        assert!(decoded_str.is_ok());
        
        let json_result: serde_json::Value = serde_json::from_str(&decoded_str.unwrap());
        assert!(json_result.is_ok());
    }
}
