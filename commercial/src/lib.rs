pub mod audit;
pub mod cache;
pub mod db;
pub mod device;
pub mod license;
pub mod metrics;
pub mod user;
pub mod web;

use sqlx::SqlitePool;
use std::sync::Arc;

pub use audit::AuditLogger;
pub use cache::CacheManager;
pub use device::DeviceManager;
pub use license::LicenseManager;
pub use metrics::MetricsCollector;
pub use user::UserManager;

#[derive(Clone)]
pub struct AppState {
    pub license_manager: Arc<LicenseManager>,
    pub user_manager: Arc<UserManager>,
    pub device_manager: Arc<DeviceManager>,
    pub audit_logger: Arc<AuditLogger>,
    pub cache: Arc<CacheManager<String, String>>,
    pub metrics: Arc<MetricsCollector>,
    pub db_pool: Arc<SqlitePool>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

impl AppState {
    pub async fn new() -> Self {
        let db_pool = Arc::new(db::init_database().await.expect("Failed to initialize database"));

        Self {
            license_manager: Arc::new(LicenseManager::new().await),
            user_manager: Arc::new(UserManager::new(db_pool.clone()).await),
            device_manager: Arc::new(DeviceManager::new(db_pool.clone()).await),
            audit_logger: Arc::new(AuditLogger::new(db_pool.clone()).await),
            cache: Arc::new(CacheManager::new(1000)),
            metrics: Arc::new(MetricsCollector::new()),
            db_pool,
        }
    }
}
