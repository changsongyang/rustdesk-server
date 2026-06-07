use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, Row};

use super::{errors::DeviceError, models::*};

pub struct DeviceManager {
    pool: SqlitePool,
}

impl DeviceManager {
    pub async fn new() -> Self {
        let db_path = std::env::var("PRO_DB_URL")
            .unwrap_or_else(|_| "./data/rustdesk_pro.db".to_string());
        
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path))
            .await
            .expect("Failed to connect to database");
        
        Self::create_tables(&pool).await;
        
        Self { pool }
    }
    
    async fn create_tables(pool: &SqlitePool) {
        let _ = sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY NOT NULL,
                device_id TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                hostname TEXT,
                os_type TEXT,
                os_version TEXT,
                ip_address TEXT,
                status TEXT NOT NULL DEFAULT 'offline',
                user_id TEXT,
                organization_id TEXT,
                approved INTEGER NOT NULL DEFAULT 0,
                approved_by TEXT,
                approved_at TEXT,
                last_online TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices (device_id);
            CREATE INDEX IF NOT EXISTS idx_devices_organization ON devices (organization_id);
            CREATE INDEX IF NOT EXISTS idx_devices_status ON devices (status);
            "#,
        )
        .execute(pool)
        .await;
    }
    
    pub async fn create_device(
        &self,
        request: DeviceCreateRequest,
        user_id: Option<String>,
    ) -> Result<Device, DeviceError> {
        let existing = sqlx::query(
            "SELECT id FROM devices WHERE device_id = ?"
        )
        .bind(&request.device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DeviceError::DatabaseError(e.to_string()))?;
        
        if existing.is_some() {
            return Err(DeviceError::AlreadyExists);
        }
        
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO devices (
                id, device_id, name, hostname, os_type, os_version, 
                ip_address, status, user_id, organization_id, 
                approved, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&request.device_id)
        .bind(&request.name)
        .bind(&request.hostname)
        .bind(&request.os_type)
        .bind(&request.os_version)
        .bind(&request.ip_address)
        .bind("offline")
        .bind(&user_id)
        .bind(&request.organization_id)
        .bind(0)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DeviceError::DatabaseError(e.to_string()))?;
        
        self.get_device(&id).await
    }
    
    pub async fn get_device(&self, id: &str) -> Result<Device, DeviceError> {
        let row = sqlx::query(
            "SELECT * FROM devices WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DeviceError::DatabaseError(e.to_string()))?
        .ok_or(DeviceError::NotFound)?;
        
        Ok(Self::row_to_device(row))
    }
    
    pub async fn get_device_by_device_id(&self, device_id: &str) -> Result<Device, DeviceError> {
        let row = sqlx::query(
            "SELECT * FROM devices WHERE device_id = ?"
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DeviceError::DatabaseError(e.to_string()))?
        .ok_or(DeviceError::NotFound)?;
        
        Ok(Self::row_to_device(row))
    }
    
    pub async fn update_device(
        &self,
        id: &str,
        request: DeviceUpdateRequest,
        approved_by: Option<String>,
    ) -> Result<Device, DeviceError> {
        let now = Utc::now();
        let mut updates = Vec::new();
        let mut params: Vec<String> = Vec::new();
        
        if let Some(name) = &request.name {
            updates.push("name = ?");
            params.push(name.clone());
        }
        
        if let Some(hostname) = &request.hostname {
            updates.push("hostname = ?");
            params.push(hostname.clone());
        }
        
        if let Some(os_type) = &request.os_type {
            updates.push("os_type = ?");
            params.push(os_type.clone());
        }
        
        if let Some(os_version) = &request.os_version {
            updates.push("os_version = ?");
            params.push(os_version.clone());
        }
        
        if let Some(ip_address) = &request.ip_address {
            updates.push("ip_address = ?");
            params.push(ip_address.clone());
        }
        
        if let Some(status) = &request.status {
            updates.push("status = ?");
            params.push(status.to_string());
            
            if *status == DeviceStatus::Online {
                updates.push("last_online = ?");
                params.push(now.to_rfc3339());
            }
        }
        
        if let Some(approved) = request.approved {
            updates.push("approved = ?");
            params.push(if approved { "1" } else { "0" }.to_string());
            
            if approved {
                updates.push("approved_by = ?");
                updates.push("approved_at = ?");
                params.push(approved_by.unwrap_or_default());
                params.push(now.to_rfc3339());
            } else {
                updates.push("approved_by = NULL");
                updates.push("approved_at = NULL");
            }
        }
        
        updates.push("updated_at = ?");
        params.push(now.to_rfc3339());
        
        if updates.is_empty() {
            return self.get_device(id).await;
        }
        
        let query = format!(
            "UPDATE devices SET {} WHERE id = ?",
            updates.join(", ")
        );
        
        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(id);
        
        let result = query_builder.execute(&self.pool)
            .await
            .map_err(|e| DeviceError::DatabaseError(e.to_string()))?;
        
        if result.rows_affected() == 0 {
            return Err(DeviceError::NotFound);
        }
        
        self.get_device(id).await
    }
    
    pub async fn delete_device(&self, id: &str) -> Result<(), DeviceError> {
        let result = sqlx::query("DELETE FROM devices WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DeviceError::DatabaseError(e.to_string()))?;
        
        if result.rows_affected() == 0 {
            return Err(DeviceError::NotFound);
        }
        
        Ok(())
    }
    
    pub async fn list_devices(
        &self,
        organization_id: Option<&str>,
        status: Option<DeviceStatus>,
        approved: Option<bool>,
    ) -> Result<Vec<Device>, DeviceError> {
        let mut query = "SELECT * FROM devices WHERE 1=1".to_string();
        let mut params: Vec<String> = Vec::new();
        
        if let Some(org_id) = organization_id {
            query.push_str(" AND organization_id = ?");
            params.push(org_id.to_string());
        }
        
        if let Some(s) = &status {
            query.push_str(" AND status = ?");
            params.push(s.to_string());
        }
        
        if let Some(a) = approved {
            query.push_str(" AND approved = ?");
            params.push(if a { "1" } else { "0" }.to_string());
        }
        
        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        
        let rows = query_builder.fetch_all(&self.pool)
            .await
            .map_err(|e| DeviceError::DatabaseError(e.to_string()))?;
        
        Ok(rows.into_iter().map(Self::row_to_device).collect())
    }
    
    pub async fn approve_device(
        &self,
        id: &str,
        approved_by: String,
    ) -> Result<Device, DeviceError> {
        self.update_device(
            id,
            DeviceUpdateRequest {
                name: None,
                hostname: None,
                os_type: None,
                os_version: None,
                ip_address: None,
                status: None,
                approved: Some(true),
            },
            Some(approved_by),
        )
        .await
    }
    
    pub async fn update_device_status(
        &self,
        device_id: &str,
        status: DeviceStatus,
        ip_address: Option<String>,
    ) -> Result<Device, DeviceError> {
        let device = self.get_device_by_device_id(device_id).await?;
        
        self.update_device(
            &device.id,
            DeviceUpdateRequest {
                name: None,
                hostname: None,
                os_type: None,
                os_version: None,
                ip_address,
                status: Some(status),
                approved: None,
            },
            None,
        )
        .await
    }
    
    fn row_to_device(row: sqlx::sqlite::SqliteRow) -> Device {
        let status = match row.try_get::<String, _>("status").unwrap_or_default().to_lowercase().as_str() {
            "online" => DeviceStatus::Online,
            "offline" => DeviceStatus::Offline,
            "away" => DeviceStatus::Away,
            _ => DeviceStatus::Unknown,
        };
        
        Device {
            id: row.try_get("id").unwrap_or_default(),
            device_id: row.try_get("device_id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            hostname: row.try_get("hostname").ok(),
            os_type: row.try_get("os_type").ok(),
            os_version: row.try_get("os_version").ok(),
            ip_address: row.try_get("ip_address").ok(),
            status,
            user_id: row.try_get("user_id").ok(),
            organization_id: row.try_get("organization_id").ok(),
            approved: row.try_get::<i32, _>("approved").unwrap_or(0) != 0,
            approved_by: row.try_get("approved_by").ok(),
            approved_at: row.try_get("approved_at").ok().and_then(|s: String| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            last_online: row.try_get("last_online").ok().and_then(|s: String| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            created_at: row.try_get("created_at").ok().and_then(|s: String| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))).unwrap_or(Utc::now()),
            updated_at: row.try_get("updated_at").ok().and_then(|s: String| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))).unwrap_or(Utc::now()),
        }
    }
}

