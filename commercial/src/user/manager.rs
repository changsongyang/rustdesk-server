use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;
use std::sync::Arc;

use super::{errors::UserError, models::*};

const JWT_SECRET: &str = "rustdesk-pro-jwt-secret-key-change-in-production";
const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    role: String,
}

pub struct UserManager {
    pool: Arc<SqlitePool>,
}

impl UserManager {
    pub async fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, request: UserCreateRequest) -> Result<UserInfo, UserError> {
        if request.password.len() < 8 {
            return Err(UserError::PasswordTooShort);
        }

        let role = UserRole::from_str(&request.role).map_err(|_| UserError::InvalidRole)?;

        // Check if username or email already exists
        let existing =
            sqlx::query("SELECT username, email FROM users WHERE username = ? OR email = ?")
                .bind(&request.username)
                .bind(&request.email)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = existing {
            let username: String = row.try_get("username").unwrap_or_default();
            let email: String = row.try_get("email").unwrap_or_default();

            if username == request.username {
                return Err(UserError::UsernameExists);
            }
            if email == request.email {
                return Err(UserError::EmailExists);
            }
        }

        let id = uuid::Uuid::new_v4().to_string();
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;
        let created_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, organization_id, created_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, 1)
            "#
        )
        .bind(&id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(role.to_str())
        .bind(&request.organization_id)
        .bind(&created_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(UserInfo {
            id,
            username: request.username,
            email: request.email,
            role: role.to_str().to_string(),
            organization_id: request.organization_id,
        })
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<UserInfo, UserError> {
        let row = sqlx::query(
            "SELECT id, username, email, role, organization_id FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|_| UserError::NotFound)?;

        Ok(UserInfo {
            id: row.try_get("id").map_err(|_| UserError::NotFound)?,
            username: row.try_get("username").map_err(|_| UserError::NotFound)?,
            email: row.try_get("email").map_err(|_| UserError::NotFound)?,
            role: row.try_get("role").map_err(|_| UserError::NotFound)?,
            organization_id: row.try_get("organization_id").ok(),
        })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<UserInfo, UserError> {
        let row = sqlx::query(
            "SELECT id, username, email, role, organization_id FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_one(&*self.pool)
        .await
        .map_err(|_| UserError::NotFound)?;

        Ok(UserInfo {
            id: row.try_get("id").map_err(|_| UserError::NotFound)?,
            username: row.try_get("username").map_err(|_| UserError::NotFound)?,
            email: row.try_get("email").map_err(|_| UserError::NotFound)?,
            role: row.try_get("role").map_err(|_| UserError::NotFound)?,
            organization_id: row.try_get("organization_id").ok(),
        })
    }

    pub async fn update_user(
        &self,
        id: &str,
        request: UserUpdateRequest,
    ) -> Result<UserInfo, UserError> {
        // Get current user
        let current = self.get_user_by_id(id).await?;

        // Build update query dynamically
        let email = request.email.unwrap_or(current.email);
        let role = request.role.unwrap_or(current.role);
        let organization_id = request.organization_id.or(current.organization_id);
        let is_active = request.is_active.unwrap_or(true);

        // Validate role
        if UserRole::from_str(&role).is_err() {
            return Err(UserError::InvalidRole);
        }

        sqlx::query(
            r#"
            UPDATE users 
            SET email = ?, role = ?, organization_id = ?, is_active = ?
            WHERE id = ?
            "#,
        )
        .bind(&email)
        .bind(&role)
        .bind(&organization_id)
        .bind(is_active as i32)
        .bind(id)
        .execute(&*self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        self.get_user_by_id(id).await
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), UserError> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::NotFound);
        }

        Ok(())
    }

    pub async fn list_users(
        &self,
        organization_id: Option<&str>,
    ) -> Result<Vec<UserInfo>, UserError> {
        let rows = if let Some(org_id) = organization_id {
            sqlx::query(
                "SELECT id, username, email, role, organization_id FROM users WHERE organization_id = ?"
            )
            .bind(org_id)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query("SELECT id, username, email, role, organization_id FROM users")
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| UserError::DatabaseError(e.to_string()))?
        };

        let users = rows
            .into_iter()
            .map(|row| UserInfo {
                id: row.try_get("id").unwrap_or_default(),
                username: row.try_get("username").unwrap_or_default(),
                email: row.try_get("email").unwrap_or_default(),
                role: row.try_get("role").unwrap_or_default(),
                organization_id: row.try_get("organization_id").ok(),
            })
            .collect();

        Ok(users)
    }

    pub async fn login(&self, request: UserLoginRequest) -> Result<UserLoginResponse, UserError> {
        // Get user by username
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, role, organization_id, is_active FROM users WHERE username = ?"
        )
        .bind(&request.username)
        .fetch_one(&*self.pool)
        .await
        .map_err(|_| UserError::InvalidCredentials)?;

        let password_hash: String = row
            .try_get("password_hash")
            .map_err(|_| UserError::InvalidCredentials)?;
        let is_active: i32 = row.try_get("is_active").unwrap_or(0);

        // Verify password
        let valid =
            verify(&request.password, &password_hash).map_err(|_| UserError::InvalidCredentials)?;

        if !valid {
            return Err(UserError::InvalidCredentials);
        }

        if is_active == 0 {
            return Err(UserError::UserNotActive);
        }

        let user = UserInfo {
            id: row
                .try_get("id")
                .map_err(|_| UserError::InvalidCredentials)?,
            username: row
                .try_get("username")
                .map_err(|_| UserError::InvalidCredentials)?,
            email: row
                .try_get("email")
                .map_err(|_| UserError::InvalidCredentials)?,
            role: row
                .try_get("role")
                .map_err(|_| UserError::InvalidCredentials)?,
            organization_id: row.try_get("organization_id").ok(),
        };

        // Generate JWT token
        let claims = Claims {
            sub: user.id.clone(),
            exp: (Utc::now() + chrono::Duration::hours(JWT_EXPIRATION_HOURS)).timestamp() as usize,
            role: user.role.clone(),
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .map_err(|_| UserError::InvalidCredentials)?;

        // Update last login
        let _ = sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(&user.id)
            .execute(&*self.pool)
            .await;

        Ok(UserLoginResponse { token, user })
    }

    pub async fn validate_token(&self, token: &str) -> Result<UserInfo, UserError> {
        let token = token.strip_prefix("Bearer ").unwrap_or(token);

        let decoding_key = DecodingKey::from_secret(JWT_SECRET.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| UserError::InvalidCredentials)?;

        let user = self.get_user_by_id(&token_data.claims.sub).await?;

        Ok(user)
    }

    pub async fn update_last_login(&self, user_id: &str) -> Result<(), UserError> {
        sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(user_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
