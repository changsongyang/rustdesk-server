use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, Row};
use serde_json;

use super::{errors::AuditError, models::*};

pub struct AuditLogger {
    pool: SqlitePool,
}

impl AuditLogger {
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
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY NOT NULL,
                log_type TEXT NOT NULL,
                action TEXT NOT NULL,
                user_id TEXT,
                username TEXT,
                device_id TEXT,
                device_name TEXT,
                ip_address TEXT,
                user_agent TEXT,
                details TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_logs_type ON audit_logs (log_type);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs (user_id);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_device ON audit_logs (device_id);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs (created_at);
            "#,
        )
        .execute(pool)
        .await;
    }
    
    pub async fn log(&self, request: AuditLogRequest) -> Result<AuditLog, AuditError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let details_str = request.details
            .map(|d| serde_json::to_string(&d))
            .transpose()
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, log_type, action, user_id, username, 
                device_id, device_name, 
                ip_address, user_agent, 
                details, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&request.log_type.to_string())
        .bind(&request.action)
        .bind(&request.user_id)
        .bind(&request.username)
        .bind(&request.device_id)
        .bind(&request.device_name)
        .bind(&request.ip_address)
        .bind(&request.user_agent)
        .bind(&details_str)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        self.get_log(&id).await
    }
    
    pub async fn get_log(&self, id: &str) -> Result<AuditLog, AuditError> {
        let row = sqlx::query("SELECT * FROM audit_logs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?
            .ok_or(AuditError::NotFound)?;
        
        Ok(Self::row_to_log(row))
    }
    
    pub async fn list_logs(
        &self,
        log_type: Option<AuditLogType>,
        user_id: Option<&str>,
        device_id: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<AuditLog>, AuditError> {
        let mut query = String::from("SELECT * FROM audit_logs WHERE 1=1");
        let mut params: Vec<String> = Vec::new();
        
        if let Some(t) = &log_type {
            query.push_str(" AND log_type = ?");
            params.push(t.to_string());
        }
        
        if let Some(uid) = user_id {
            query.push_str(" AND user_id = ?");
            params.push(uid.to_string());
        }
        
        if let Some(did) = device_id {
            query.push_str(" AND device_id = ?");
            params.push(did.to_string());
        }
        
        if let Some(start) = start_time {
            query.push_str(" AND created_at >= ?");
            params.push(start.to_rfc3339());
        }
        
        if let Some(end) = end_time {
            query.push_str(" AND created_at <= ?");
            params.push(end.to_rfc3339());
        }
        
        query.push_str(" ORDER BY created_at DESC");
        
        if let Some(l) = limit {
            query.push_str(" LIMIT ?");
            params.push(l.to_string());
        }
        
        if let Some(o) = offset {
            query.push_str(" OFFSET ?");
            params.push(o.to_string());
        }
        
        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        
        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        Ok(rows.into_iter().map(Self::row_to_log).collect())
    }
    
    pub async fn log_authentication(
        &self,
        user_id: Option<String>,
        username: Option<String>,
        action: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), AuditError> {
        self.log(AuditLogRequest {
            log_type: AuditLogType::Authentication,
            action: action.to_string(),
            user_id,
            username,
            device_id: None,
            device_name: None,
            ip_address,
            user_agent,
            details: None,
        })
        .await?;
        
        Ok(())
    }
    
    pub async fn log_device(
        &self,
        user_id: Option<String>,
        username: Option<String>,
        device_id: Option<String>,
        device_name: Option<String>,
        action: &str,
        ip_address: Option<String>,
        details: Option<serde_json::Value>,
    ) -> Result<(), AuditError> {
        self.log(AuditLogRequest {
            log_type: AuditLogType::Device,
            action: action.to_string(),
            user_id,
            username,
            device_id,
            device_name,
            ip_address,
            user_agent: None,
            details,
        })
        .await?;
        
        Ok(())
    }
    
    pub async fn log_session(
        &self,
        user_id: Option<String>,
        username: Option<String>,
        device_id: Option<String>,
        device_name: Option<String>,
        action: &str,
        ip_address: Option<String>,
    ) -> Result<(), AuditError> {
        self.log(AuditLogRequest {
            log_type: AuditLogType::Session,
            action: action.to_string(),
            user_id,
            username,
            device_id,
            device_name,
            ip_address,
            user_agent: None,
            details: None,
        })
        .await?;
        
        Ok(())
    }
    
    pub async fn log_configuration(
        &self,
        user_id: Option<String>,
        username: Option<String>,
        action: &str,
        details: Option<serde_json::Value>,
    ) -> Result<(), AuditError> {
        self.log(AuditLogRequest {
            log_type: AuditLogType::Configuration,
            action: action.to_string(),
            user_id,
            username,
            device_id: None,
            device_name: None,
            ip_address: None,
            user_agent: None,
            details,
        })
        .await?;
        
        Ok(())
    }
    
    fn row_to_log(row: sqlx::sqlite::SqliteRow) -> AuditLog {
        let log_type = match row.try_get::<String, _>("log_type").unwrap_or_default().to_lowercase().as_str() {
            "authentication" => AuditLogType::Authentication,
            "authorization" => AuditLogType::Authorization,
            "device" => AuditLogType::Device,
            "user" => AuditLogType::User,
            "license" => AuditLogType::License,
            "system" => AuditLogType::System,
            "session" => AuditLogType::Session,
            "configuration" => AuditLogType::Configuration,
            _ => AuditLogType::System,
        };
        
        let details = row.try_get("details").ok().and_then(|s: String| serde_json::from_str(&s).ok());
        
        AuditLog {
            id: row.try_get("id").unwrap_or_default(),
            log_type,
            action: row.try_get("action").unwrap_or_default(),
            user_id: row.try_get("user_id").ok(),
            username: row.try_get("username").ok(),
            device_id: row.try_get("device_id").ok(),
            device_name: row.try_get("device_name").ok(),
            ip_address: row.try_get("ip_address").ok(),
            user_agent: row.try_get("user_agent").ok(),
            details,
            created_at: row.try_get("created_at").ok().and_then(|s: String| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))).unwrap_or(Utc::now()),
        }
    }
}

