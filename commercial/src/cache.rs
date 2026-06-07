use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// 简单的内存缓存实现
#[derive(Debug)]
pub struct CacheEntry<V> {
    pub value: V,
    pub expires_at: Option<Instant>,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, ttl_secs: Option<u64>) -> Self {
        let expires_at = ttl_secs.map(|secs| Instant::now() + Duration::from_secs(secs));
        Self { value, expires_at }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Instant::now() > exp)
            .unwrap_or(false)
    }
}

/// 线程安全的缓存管理器
#[derive(Debug)]
pub struct CacheManager<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    max_size: usize,
}

impl<K, V> Default for CacheManager<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(1000)
    }
}

impl<K, V> CacheManager<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// 创建新的缓存管理器
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// 获取缓存值
    pub async fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().await;
        
        if let Some(entry) = entries.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        
        None
    }

    /// 设置缓存值（无过期时间）
    pub async fn set(&self, key: K, value: V) {
        self.set_with_ttl(key, value, None).await;
    }

    /// 设置缓存值（带过期时间）
    pub async fn set_with_ttl(&self, key: K, value: V, ttl_secs: Option<u64>) {
        // 如果缓存已满，删除最旧的条目
        let mut entries = self.entries.write().await;
        if entries.len() >= self.max_size {
            // 简单策略：删除第一个过期的条目
            entries.retain(|_, v| !v.is_expired());
            
            // 如果仍然满了，随机删除一些条目
            if entries.len() >= self.max_size {
                let to_remove = entries.len() - self.max_size / 2;
                let keys: Vec<_> = entries.keys().take(to_remove).cloned().collect();
                for key in keys {
                    entries.remove(&key);
                }
            }
        }
        
        entries.insert(key, CacheEntry::new(value, ttl_secs));
    }

    /// 删除缓存值
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|e| e.value)
    }

    /// 清除所有缓存
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// 清理过期条目
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, v| !v.is_expired());
    }

    /// 获取缓存大小
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// 检查缓存是否为空
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = CacheManager::<i32, String>::new(10);
        
        cache.set(1, "one".to_string()).await;
        
        assert_eq!(cache.get(&1).await, Some("one".to_string()));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = CacheManager::<i32, String>::new(10);
        
        cache.set_with_ttl(1, "one".to_string(), Some(1)).await;
        
        assert_eq!(cache.get(&1).await, Some("one".to_string()));
        
        // Wait for expiration
        sleep(Duration::from_secs(2)).await;
        
        assert_eq!(cache.get(&1).await, None);
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache = CacheManager::<i32, String>::new(10);
        
        cache.set(1, "one".to_string()).await;
        cache.remove(&1).await;
        
        assert_eq!(cache.get(&1).await, None);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = CacheManager::<i32, String>::new(10);
        
        cache.set(1, "one".to_string()).await;
        cache.set(2, "two".to_string()).await;
        cache.clear().await;
        
        assert!(cache.is_empty().await);
    }
}
