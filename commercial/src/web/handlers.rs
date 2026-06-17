use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::audit::models::*;
use crate::device::models::*;
use crate::user::models::*;
use crate::AppState;

// ==================== Root Handler ====================

pub async fn root_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "RustDesk Pro Server API",
            "version": "1.0.0",
            "docs": "/docs",
            "health": "/health",
            "auth": {
                "login": "POST /api/auth/login",
                "validate": "POST /api/auth/validate"
            }
        })),
    )
}

// ==================== Health Check ====================

pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "version": "1.0.0",
            "service": "rustdesk-pro-server"
        })),
    )
}

// ==================== Authentication ====================

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    Extension(state): Extension<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let login_request = UserLoginRequest {
        username: request.username,
        password: request.password,
    };

    match state.user_manager.login(login_request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap())),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

pub async fn validate_token(
    Extension(state): Extension<AppState>,
    Json(request): Json<ValidateTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.user_manager.validate_token(&request.token).await {
        Ok(user) => Ok(Json(serde_json::to_value(user).unwrap())),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

// ==================== User Management ====================

pub async fn create_user(
    Extension(state): Extension<AppState>,
    Json(request): Json<UserCreateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.user_manager.create_user(request).await {
        Ok(user) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(user).unwrap()),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_user(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.user_manager.get_user_by_id(&id).await {
        Ok(user) => Ok(Json(serde_json::to_value(user).unwrap())),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_current_user(
    Extension(_state): Extension<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Get user from auth middleware
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Not implemented"})),
    ))
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub organization_id: Option<String>,
}

pub async fn list_users(
    Extension(state): Extension<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state
        .user_manager
        .list_users(query.organization_id.as_deref())
        .await
    {
        Ok(users) => Ok(Json(serde_json::to_value(users).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn update_user(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UserUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.user_manager.update_user(&id, request).await {
        Ok(user) => Ok(Json(serde_json::to_value(user).unwrap())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn delete_user(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.user_manager.delete_user(&id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, Json(serde_json::json!({})))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    Extension(_state): Extension<AppState>,
    Path(_id): Path<String>,
    Json(_request): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement password change
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Not implemented"})),
    ))
}

// ==================== Device Management ====================

pub async fn create_device(
    Extension(state): Extension<AppState>,
    Json(request): Json<DeviceCreateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.device_manager.create_device(request, None).await {
        Ok(device) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(device).unwrap()),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_device(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.device_manager.get_device(&id).await {
        Ok(device) => Ok(Json(serde_json::to_value(device).unwrap())),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDevicesQuery {
    pub organization_id: Option<String>,
    pub status: Option<String>,
    pub approved: Option<bool>,
}

pub async fn list_devices(
    Extension(state): Extension<AppState>,
    Query(query): Query<ListDevicesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let status = query.status.and_then(|s| match s.to_lowercase().as_str() {
        "online" => Some(DeviceStatus::Online),
        "offline" => Some(DeviceStatus::Offline),
        "away" => Some(DeviceStatus::Away),
        _ => None,
    });

    match state
        .device_manager
        .list_devices(query.organization_id.as_deref(), status, query.approved)
        .await
    {
        Ok(devices) => Ok(Json(serde_json::to_value(devices).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn update_device(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(request): Json<DeviceUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.device_manager.update_device(&id, request, None).await {
        Ok(device) => Ok(Json(serde_json::to_value(device).unwrap())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn delete_device(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.device_manager.delete_device(&id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, Json(serde_json::json!({})))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn approve_device(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Get user from auth middleware
    match state
        .device_manager
        .approve_device(&id, "admin".to_string())
        .await
    {
        Ok(device) => Ok(Json(serde_json::to_value(device).unwrap())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceStatusRequest {
    pub status: String,
    pub ip_address: Option<String>,
}

pub async fn update_device_status(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateDeviceStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let status = match request.status.to_lowercase().as_str() {
        "online" => DeviceStatus::Online,
        "offline" => DeviceStatus::Offline,
        "away" => DeviceStatus::Away,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid status"})),
            ))
        }
    };

    match state
        .device_manager
        .update_device_status(&id, status, request.ip_address)
        .await
    {
        Ok(device) => Ok(Json(serde_json::to_value(device).unwrap())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn list_pending_devices(
    Extension(state): Extension<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state
        .device_manager
        .list_devices(None, None, Some(false))
        .await
    {
        Ok(devices) => Ok(Json(serde_json::to_value(devices).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

// ==================== Audit Logs ====================

#[derive(Debug, Deserialize)]
pub struct ListAuditLogsQuery {
    pub log_type: Option<String>,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn list_audit_logs(
    Extension(state): Extension<AppState>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let log_type = query
        .log_type
        .and_then(|t| match t.to_lowercase().as_str() {
            "authentication" => Some(AuditLogType::Authentication),
            "authorization" => Some(AuditLogType::Authorization),
            "device" => Some(AuditLogType::Device),
            "user" => Some(AuditLogType::User),
            "license" => Some(AuditLogType::License),
            "system" => Some(AuditLogType::System),
            "session" => Some(AuditLogType::Session),
            "configuration" => Some(AuditLogType::Configuration),
            _ => None,
        });

    match state
        .audit_logger
        .list_logs(
            log_type,
            query.user_id.as_deref(),
            query.device_id.as_deref(),
            None,
            None,
            query.limit,
            query.offset,
        )
        .await
    {
        Ok(logs) => Ok(Json(serde_json::to_value(logs).unwrap())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_audit_log(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.audit_logger.get_log(&id).await {
        Ok(log) => Ok(Json(serde_json::to_value(log).unwrap())),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_audit_stats(
    Extension(_state): Extension<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement audit stats
    Ok(Json(serde_json::json!({
        "total": 0,
        "by_type": {},
        "recent": []
    })))
}

// ==================== License Management ====================

#[derive(Debug, Deserialize)]
pub struct LicenseGenerateRequest {
    pub license_type: String,
    pub duration_days: i64,
    pub max_devices: Option<i32>,
}

pub async fn generate_license(
    Extension(state): Extension<AppState>,
    Json(request): Json<LicenseGenerateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state
        .license_manager
        .generate_license(
            &request.license_type,
            request.duration_days,
            request.max_devices,
        )
        .await
    {
        Ok(key) => Ok(Json(serde_json::json!({"key": key}))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct LicenseValidateRequest {
    pub key: String,
}

pub async fn validate_license(
    Extension(state): Extension<AppState>,
    Json(request): Json<LicenseValidateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.license_manager.validate_license(&request.key).await {
        Ok(info) => Ok(Json(serde_json::to_value(info).unwrap())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_active_license(
    Extension(state): Extension<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.license_manager.get_active_license().await {
        Some(info) => Ok(Json(serde_json::to_value(info).unwrap())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "No active license"})),
        )),
    }
}

// ==================== Organization Management ====================

#[derive(Debug, Deserialize)]
pub struct OrganizationCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_devices: Option<i32>,
}

pub async fn create_organization(
    Extension(_state): Extension<AppState>,
    Json(_request): Json<OrganizationCreateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement organization creation
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Not implemented"})),
    ))
}

pub async fn list_organizations(
    Extension(_state): Extension<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement organization listing
    Ok(Json(serde_json::json!([])))
}

pub async fn get_organization(
    Extension(_state): Extension<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement organization retrieval
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Not implemented"})),
    ))
}

pub async fn update_organization(
    Extension(_state): Extension<AppState>,
    Path(_id): Path<String>,
    Json(_request): Json<OrganizationCreateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement organization update
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Not implemented"})),
    ))
}

pub async fn delete_organization(
    Extension(_state): Extension<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement organization deletion
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Not implemented"})),
    ))
}
