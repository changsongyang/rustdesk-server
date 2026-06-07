use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Prometheus 指标收集器
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    // HTTP 请求指标
    pub http_requests_total: Arc<RwLock<Counter>>,
    pub http_request_duration: Arc<RwLock<Histogram>>,
    pub http_requests_in_flight: Arc<RwLock<Gauge>>,
    
    // 业务指标
    pub active_users: Arc<RwLock<Gauge>>,
    pub active_devices: Arc<RwLock<Gauge>>,
    pub active_sessions: Arc<RwLock<Gauge>>,
    pub license_validations_total: Arc<RwLock<Counter>>,
    
    // 数据库指标
    pub db_query_total: Arc<RwLock<Counter>>,
    pub db_query_duration: Arc<RwLock<Histogram>>,
    
    // 缓存指标
    pub cache_hits_total: Arc<RwLock<Counter>>,
    pub cache_misses_total: Arc<RwLock<Counter>>,
    
    // 审计日志指标
    pub audit_logs_total: Arc<RwLock<Counter>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        let http_buckets = vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
        let db_buckets = vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];
        
        Self {
            http_requests_total: Arc::new(RwLock::new(Counter::new())),
            http_request_duration: Arc::new(RwLock::new(Histogram::new(http_buckets))),
            http_requests_in_flight: Arc::new(RwLock::new(Gauge::new())),
            
            active_users: Arc::new(RwLock::new(Gauge::new())),
            active_devices: Arc::new(RwLock::new(Gauge::new())),
            active_sessions: Arc::new(RwLock::new(Gauge::new())),
            license_validations_total: Arc::new(RwLock::new(Counter::new())),
            
            db_query_total: Arc::new(RwLock::new(Counter::new())),
            db_query_duration: Arc::new(RwLock::new(Histogram::new(db_buckets))),
            
            cache_hits_total: Arc::new(RwLock::new(Counter::new())),
            cache_misses_total: Arc::new(RwLock::new(Counter::new())),
            
            audit_logs_total: Arc::new(RwLock::new(Counter::new())),
        }
    }
    
    // HTTP 请求指标
    pub async fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        let label = format!("{}:{}", method, path);
        self.http_requests_total.write().await.inc_by(&label, 1);
        self.http_request_duration.write().await.observe(&label, duration.as_secs_f64());
    }
    
    pub async fn inc_http_requests_in_flight(&self) {
        self.http_requests_in_flight.write().await.inc();
    }
    
    pub async fn dec_http_requests_in_flight(&self) {
        self.http_requests_in_flight.write().await.dec();
    }
    
    // 业务指标
    pub async fn set_active_users(&self, count: usize) {
        self.active_users.write().await.set(count as i64);
    }
    
    pub async fn set_active_devices(&self, count: usize) {
        self.active_devices.write().await.set(count as i64);
    }
    
    pub async fn set_active_sessions(&self, count: usize) {
        self.active_sessions.write().await.set(count as i64);
    }
    
    pub async fn record_license_validation(&self) {
        self.license_validations_total.write().await.inc();
    }
    
    // 数据库指标
    pub async fn record_db_query(&self, table: &str, duration: Duration) {
        self.db_query_total.write().await.inc_by(table, 1);
        self.db_query_duration.write().await.observe(table, duration.as_secs_f64());
    }
    
    // 缓存指标
    pub async fn record_cache_hit(&self) {
        self.cache_hits_total.write().await.inc();
    }
    
    pub async fn record_cache_miss(&self) {
        self.cache_misses_total.write().await.inc();
    }
    
    // 审计日志指标
    pub async fn record_audit_log(&self, log_type: &str) {
        self.audit_logs_total.write().await.inc_by(log_type, 1);
    }
}

/// 计数器
#[derive(Debug)]
pub struct Counter {
    values: std::collections::HashMap<String, u64>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
        }
    }
    
    pub fn inc(&mut self) {
        self.inc_by("", 1);
    }
    
    pub fn inc_by(&mut self, label: &str, value: u64) {
        *self.values.entry(label.to_string()).or_insert(0) += value;
    }
    
    pub fn get(&self, label: &str) -> u64 {
        self.values.get(label).copied().unwrap_or(0)
    }
}

/// 仪表盘（当前值）
#[derive(Debug)]
pub struct Gauge {
    value: i64,
}

impl Gauge {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn inc(&mut self) {
        self.value += 1;
    }
    
    pub fn dec(&mut self) {
        self.value -= 1;
    }
    
    pub fn set(&mut self, value: i64) {
        self.value = value;
    }
    
    pub fn get(&self) -> i64 {
        self.value
    }
}

/// 直方图
#[derive(Debug)]
pub struct Histogram {
    buckets: Vec<f64>,
    values: std::collections::HashMap<String, Vec<f64>>,
    sum: std::collections::HashMap<String, f64>,
    count: std::collections::HashMap<String, u64>,
}

impl Histogram {
    pub fn new(buckets: impl Into<Vec<f64>>) -> Self {
        Self {
            buckets: buckets.into(),
            values: std::collections::HashMap::new(),
            sum: std::collections::HashMap::new(),
            count: std::collections::HashMap::new(),
        }
    }
    
    pub fn observe(&mut self, label: &str, value: f64) {
        self.values.entry(label.to_string()).or_insert_with(Vec::new).push(value);
        *self.sum.entry(label.to_string()).or_insert(0.0) += value;
        *self.count.entry(label.to_string()).or_insert(0) += 1;
    }
}

/// Prometheus 导出格式
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusMetrics {
    pub http_requests_total: Vec<MetricFamily>,
    pub http_request_duration_seconds: Vec<MetricFamily>,
    pub http_requests_in_flight: Vec<MetricFamily>,
    pub active_users: Vec<MetricFamily>,
    pub active_devices: Vec<MetricFamily>,
    pub active_sessions: Vec<MetricFamily>,
    pub license_validations_total: Vec<MetricFamily>,
    pub db_query_total: Vec<MetricFamily>,
    pub db_query_duration_seconds: Vec<MetricFamily>,
    pub cache_hits_total: Vec<MetricFamily>,
    pub cache_misses_total: Vec<MetricFamily>,
    pub audit_logs_total: Vec<MetricFamily>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricFamily {
    pub name: String,
    pub help: String,
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metric {
    pub labels: std::collections::HashMap<String, String>,
    pub value: f64,
}

impl MetricsCollector {
    pub async fn export(&self) -> PrometheusMetrics {
        let http_requests_total = self.export_counter(&self.http_requests_total, "http_requests_total", "Total HTTP requests").await;
        let http_request_duration_seconds = self.export_histogram(&self.http_request_duration, "http_request_duration_seconds", "HTTP request duration in seconds").await;
        let http_requests_in_flight = self.export_gauge(&self.http_requests_in_flight, "http_requests_in_flight", "HTTP requests currently in flight").await;
        let active_users = self.export_gauge(&self.active_users, "active_users", "Number of active users").await;
        let active_devices = self.export_gauge(&self.active_devices, "active_devices", "Number of active devices").await;
        let active_sessions = self.export_gauge(&self.active_sessions, "active_sessions", "Number of active sessions").await;
        let license_validations_total = self.export_counter(&self.license_validations_total, "license_validations_total", "Total license validations").await;
        let db_query_total = self.export_counter(&self.db_query_total, "db_query_total", "Total database queries").await;
        let db_query_duration_seconds = self.export_histogram(&self.db_query_duration, "db_query_duration_seconds", "Database query duration in seconds").await;
        let cache_hits_total = self.export_counter(&self.cache_hits_total, "cache_hits_total", "Total cache hits").await;
        let cache_misses_total = self.export_counter(&self.cache_misses_total, "cache_misses_total", "Total cache misses").await;
        let audit_logs_total = self.export_counter(&self.audit_logs_total, "audit_logs_total", "Total audit logs").await;
        
        PrometheusMetrics {
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            active_users,
            active_devices,
            active_sessions,
            license_validations_total,
            db_query_total,
            db_query_duration_seconds,
            cache_hits_total,
            cache_misses_total,
            audit_logs_total,
        }
    }
    
    async fn export_counter(&self, counter: &Arc<RwLock<Counter>>, name: &str, help: &str) -> Vec<MetricFamily> {
        let counter = counter.read().await;
        if counter.values.is_empty() {
            return vec![];
        }
        
        vec![MetricFamily {
            name: name.to_string(),
            help: help.to_string(),
            metrics: counter.values.iter().map(|(label, value)| {
                let mut labels = std::collections::HashMap::new();
                if !label.is_empty() {
                    let parts: Vec<&str> = label.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        labels.insert("method".to_string(), parts[0].to_string());
                        labels.insert("path".to_string(), parts[1].to_string());
                    } else {
                        labels.insert("type".to_string(), label.clone());
                    }
                }
                Metric {
                    labels,
                    value: *value as f64,
                }
            }).collect(),
        }]
    }
    
    async fn export_gauge(&self, gauge: &Arc<RwLock<Gauge>>, name: &str, help: &str) -> Vec<MetricFamily> {
        let gauge = gauge.read().await;
        vec![MetricFamily {
            name: name.to_string(),
            help: help.to_string(),
            metrics: vec![Metric {
                labels: std::collections::HashMap::new(),
                value: gauge.get() as f64,
            }],
        }]
    }
    
    async fn export_histogram(&self, histogram: &Arc<RwLock<Histogram>>, name: &str, help: &str) -> Vec<MetricFamily> {
        let histogram = histogram.read().await;
        if histogram.count.is_empty() {
            return vec![];
        }
        
        let mut metrics = Vec::new();
        for (label, count) in &histogram.count {
            let mut labels = std::collections::HashMap::new();
            if !label.is_empty() {
                labels.insert("table".to_string(), label.clone());
            }
            metrics.push(Metric {
                labels,
                value: *count as f64,
            });
        }
        
        vec![MetricFamily {
            name: name.to_string(),
            help: help.to_string(),
            metrics,
        }]
    }
}

/// HTTP 请求计时中间件
pub struct RequestTimer {
    start: Instant,
}

impl RequestTimer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for RequestTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let metrics = MetricsCollector::new();
        
        metrics.inc_http_requests_in_flight().await;
        metrics.record_http_request("GET", "/api/users", 200, Duration::from_millis(50)).await;
        metrics.set_active_users(10).await;
        metrics.record_license_validation().await;
        
        let exported = metrics.export().await;
        
        assert!(!exported.http_requests_in_flight.is_empty());
    }
}
