use axum::{
    http::Method,
    routing::{get, post, put, delete},
    Router,
    Extension,
    response::IntoResponse,
};
use tower_http::cors::{Any, CorsLayer};

use crate::AppState;
use super::handlers::*;
use super::middleware::auth_middleware;

/// Swagger UI API 文档路由
pub async fn swagger_ui() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>RustDesk Pro Server API</title>
        <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
    </head>
    <body>
        <div id="swagger-ui"></div>
        <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
        <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
        <script>
            window.onload = function() {
                window.ui = SwaggerUIBundle({
                    url: "/openapi.json",
                    dom_id: '#swagger-ui',
                    presets: [
                        SwaggerUIBundle.presets.apis,
                        SwaggerUIStandalonePreset
                    ],
                    layout: "StandaloneLayout"
                });
            };
        </script>
    </body>
    </html>
    "#;
    axum::response::Html(html)
}

/// OpenAPI JSON 文档
pub async fn openapi_json() -> impl IntoResponse {
    let spec = include_str!("../../openapi.json");
    let json_value: serde_json::Value = serde_json::from_str(spec).unwrap();
    axum::response::Json(json_value)
}

/// Prometheus 指标端点
pub async fn prometheus_metrics(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let metrics = state.metrics.export().await;
    
    // 转换为 Prometheus 文本格式
    let mut output = String::new();
    
    // HTTP 请求总数
    for family in &metrics.http_requests_total {
        for metric in &family.metrics {
            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", metric.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(","))
            };
            output.push_str(&format!("{} {} {}\n", family.name, labels, metric.value));
        }
    }
    
    // HTTP 请求持续时间
    for family in &metrics.http_request_duration_seconds {
        for metric in &family.metrics {
            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", metric.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(","))
            };
            output.push_str(&format!("{}_sum {} {}\n", family.name.trim_end_matches("_total"), labels, metric.value));
        }
    }
    
    // HTTP 飞行中请求
    for family in &metrics.http_requests_in_flight {
        for metric in &family.metrics {
            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", metric.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(","))
            };
            output.push_str(&format!("{} {} {}\n", family.name, labels, metric.value));
        }
    }
    
    // 活跃用户
    for family in &metrics.active_users {
        for metric in &family.metrics {
            output.push_str(&format!("{} {}\n", family.name, metric.value));
        }
    }
    
    // 活跃设备
    for family in &metrics.active_devices {
        for metric in &family.metrics {
            output.push_str(&format!("{} {}\n", family.name, metric.value));
        }
    }
    
    // 活跃会话
    for family in &metrics.active_sessions {
        for metric in &family.metrics {
            output.push_str(&format!("{} {}\n", family.name, metric.value));
        }
    }
    
    // 许可证验证总数
    for family in &metrics.license_validations_total {
        for metric in &family.metrics {
            output.push_str(&format!("{} {}\n", family.name, metric.value));
        }
    }
    
    // 数据库查询总数
    for family in &metrics.db_query_total {
        for metric in &family.metrics {
            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", metric.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(","))
            };
            output.push_str(&format!("{} {} {}\n", family.name, labels, metric.value));
        }
    }
    
    // 缓存命中
    for family in &metrics.cache_hits_total {
        for metric in &family.metrics {
            output.push_str(&format!("{} {}\n", family.name, metric.value));
        }
    }
    
    // 缓存未命中
    for family in &metrics.cache_misses_total {
        for metric in &family.metrics {
            output.push_str(&format!("{} {}\n", family.name, metric.value));
        }
    }
    
    // 审计日志总数
    for family in &metrics.audit_logs_total {
        for metric in &family.metrics {
            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", metric.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(","))
            };
            output.push_str(&format!("{} {} {}\n", family.name, labels, metric.value));
        }
    }
    
    axum::response::Html(output)
}

pub async fn start_server(state: AppState, port: u16) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);
    
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login))
        .route("/api/auth/validate", post(validate_token))
        .route("/api/license/validate", post(validate_license))
        // API Documentation
        .route("/docs", get(swagger_ui))
        .route("/openapi.json", get(openapi_json))
        // Monitoring
        .route("/metrics", get(prometheus_metrics));
    
    // Protected routes (authentication required)
    let protected_routes = Router::new()
        // User management
        .route("/api/users", post(create_user).get(list_users))
        .route("/api/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/api/users/me", get(get_current_user))
        .route("/api/users/:id/password", put(change_password))
        
        // Device management
        .route("/api/devices", post(create_device).get(list_devices))
        .route("/api/devices/:id", get(get_device).put(update_device).delete(delete_device))
        .route("/api/devices/:id/approve", post(approve_device))
        .route("/api/devices/:id/status", put(update_device_status))
        .route("/api/devices/pending", get(list_pending_devices))
        
        // Audit logs
        .route("/api/audit", get(list_audit_logs))
        .route("/api/audit/:id", get(get_audit_log))
        .route("/api/audit/stats", get(get_audit_stats))
        
        // License management
        .route("/api/license/generate", post(generate_license))
        .route("/api/license", get(get_active_license))
        
        // Organization management
        .route("/api/organizations", post(create_organization).get(list_organizations))
        .route("/api/organizations/:id", get(get_organization).put(update_organization).delete(delete_organization))
        .layer(axum::middleware::from_fn(auth_middleware));
    
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(Extension(state))
        .layer(cors);
    
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    
    log::info!("RustDesk Pro Server listening on http://{}", addr);
    log::info!("API Documentation:");
    log::info!("  Swagger UI: http://localhost:{}/docs", port);
    log::info!("  OpenAPI JSON: http://localhost:{}/openapi.json", port);
    log::info!("  Health Check: GET /health");
    log::info!("  Authentication:");
    log::info!("    POST /api/auth/login");
    log::info!("    POST /api/auth/validate");
    log::info!("  Users:");
    log::info!("    GET|POST /api/users");
    log::info!("    GET|PUT|DELETE /api/users/:id");
    log::info!("  Devices:");
    log::info!("    GET|POST /api/devices");
    log::info!("    GET|PUT|DELETE /api/devices/:id");
    log::info!("    POST /api/devices/:id/approve");
    log::info!("  Audit Logs:");
    log::info!("    GET /api/audit");
    log::info!("  License:");
    log::info!("    POST /api/license/generate");
    log::info!("    POST /api/license/validate");
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
