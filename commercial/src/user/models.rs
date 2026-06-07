use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub organization_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "operator")]
    Operator,
    #[serde(rename = "viewer")]
    Viewer,
}

impl UserRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "operator" => Some(Self::Operator),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Operator => "operator",
            Self::Viewer => "viewer",
        }
    }
    
    pub fn has_permission(&self, permission: Permission) -> bool {
        match (self, permission) {
            (_, Permission::View) => true,
            (UserRole::Admin, _) => true,
            (UserRole::Operator, Permission::ManageDevices) => true,
            (UserRole::Operator, Permission::Connect) => true,
            (UserRole::Operator, Permission::ManageUsers) => false,
            (UserRole::Viewer, _) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    View,
    ManageUsers,
    ManageDevices,
    Connect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreateRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub organization_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateRequest {
    pub email: Option<String>,
    pub role: Option<String>,
    pub organization_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub organization_id: Option<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role.to_str().to_string(),
            organization_id: user.organization_id,
        }
    }
}
