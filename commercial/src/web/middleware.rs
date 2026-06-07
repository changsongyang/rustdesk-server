use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};

use crate::user::models::UserInfo;
use crate::AppState;

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get("Authorization");

    // Get state from extensions
    let state = req
        .extensions()
        .get::<Extension<AppState>>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(header) = auth_header {
        let token = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;

        match state.user_manager.validate_token(token).await {
            Ok(user) => {
                // Add user to request extensions for use in handlers
                let mut req = req;
                req.extensions_mut().insert(user);
                Ok(next.run(req).await)
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn admin_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Check if user is admin
    let user = req
        .extensions()
        .get::<UserInfo>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if user.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

pub async fn optional_auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get("Authorization");

    if let Some(header) = auth_header {
        let state = req
            .extensions()
            .get::<Extension<AppState>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let token = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;

        if let Ok(user) = state.user_manager.validate_token(token).await {
            req.extensions_mut().insert(user);
        }
    }

    Ok(next.run(req).await)
}
