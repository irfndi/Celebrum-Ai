//! KV Store Standardization Utilities
//! 
//! Provides consistent patterns for KV store usage across all services:
//! - Standardized key naming conventions
//! - TTL policies and cache invalidation
//! - Performance monitoring and metrics
//! - Cache-aside pattern implementations

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use worker::kv::KvStore;

/// Standard TTL policies for different data types
#[derive(Debug, Clone, Copy)]
pub enum CacheTTL {
    /// Very short-lived data (30 seconds) - real-time market data
    RealTime = 30,
    /// Short-lived data (5 minutes) - user sessions, temporary state
    Short = 300,
    /// Medium-lived data (1 hour) - user profiles, preferences
    Medium = 3600,
    /// Long-lived data (24 hours) - configuration, static data
    Long = 86400,
    /// Very long-lived data (7 days) - historical data, analytics
    VeryLong = 604800,
}

impl CacheTTL {
    pub fn as_seconds(&self) -> u64 {
        *self as u64
    }
}

/// Standard key prefixes for different service domains
#[derive(Debug, Clone)]
pub enum KeyPrefix {
    UserProfile,
    UserSession,
    UserPreferences,
    Position,
    Opportunity,
    MarketData,
    Exchange,
    Analytics,
    Notification,
    Configuration,
    Cache,
    Metrics,
}

impl KeyPrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyPrefix::UserProfile => "user_profile",
            KeyPrefix::UserSession => "user_session",
            KeyPrefix::UserPreferences => "user_prefs",
            KeyPrefix::Position => "positions",
            KeyPrefix::Opportunity => "opportunity",
            KeyPrefix::MarketData => "market_data",
            KeyPrefix::Exchange => "exchange",
            KeyPrefix::Analytics => "analytics",
            KeyPrefix::Notification => "notification",
            KeyPrefix::Configuration => "config",
            KeyPrefix::Cache => "cache",
            KeyPrefix::Metrics => "metrics",
        }
    }
}

/// Standardized KV key builder
#[derive(Debug, Clone)]
pub struct KvKeyBuilder {
    prefix: KeyPrefix,
    components: Vec<String>,
}

impl KvKeyBuilder {
    pub fn new(prefix: KeyPrefix) -> Self {
        Self {
            prefix,
            components: Vec::new(),
        }
    }

    pub fn add_component<T: ToString>(mut self, component: T) -> Self {
        self.components.push(component.to_string());
        self
    }

    pub fn build(self) -> String {
        let mut key = self.prefix.as_str().to_string();
        for component in self.components {
            key.push(':');
            key.push_str(&component);
        }
        key
    }
}

/// Cache metadata for tracking and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub created_at: u64,
    pub expires_at: u64,
    pub access_count: u32,
    pub last_accessed: u64,
    pub data_size: usize,
    pub service_name: String,
}

impl CacheMetadata {
    pub fn new(ttl: CacheTTL, data_size: usize, service_name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            created_at: now,
            expires_at: now + ttl.as_seconds(),
            access_count: 0,
            last_accessed: now,
            data_size,
            service_name,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Wrapper for cached data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData<T> {
    pub data: T,
    pub metadata: CacheMetadata,
}

impl<T> CachedData<T> {
    pub fn new(data: T, ttl: CacheTTL, service_name: String) -> Self {
        let data_size = std::mem::size_of_val(&data);
        Self {
            data,
            metadata: CacheMetadata::new(ttl, data_size, service_name),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.metadata.is_expired()
    }

    pub fn access_data(&mut self) -> &T {
        self.metadata.record_access();
        &self.data
    }
}

/// Standardized KV operations with monitoring
pub struct StandardKvOperations {
    kv_store: KvStore,
    service_name: String,
}

impl StandardKvOperations {
    pub fn new(kv_store: KvStore, service_name: String) -> Self {
        Self {
            kv_store,
            service_name,
        }
    }

    /// Store data with standardized key and TTL
    pub async fn put_with_ttl<T: Serialize>(
        &self,
        key_builder: KvKeyBuilder,
        data: T,
        ttl: CacheTTL,
    ) -> Result<(), worker::Error> {
        let key = key_builder.build();
        let cached_data = CachedData::new(data, ttl, self.service_name.clone());
        
        let serialized = serde_json::to_string(&cached_data)
            .map_err(|e| worker::Error::RustError(format!("Serialization error: {}", e)))?;

        self.kv_store
            .put(&key, serialized)?
            .expiration_ttl(ttl.as_seconds())
            .execute()
            .await?;

        // Record metrics
        self.record_cache_operation("put", &key, cached_data.metadata.data_size).await;

        Ok(())
    }

    /// Get data with automatic metadata tracking
    pub async fn get_with_metadata<T: for<'de> Deserialize<'de> + Serialize + Clone>(
        &self,
        key_builder: KvKeyBuilder,
    ) -> Result<Option<T>, worker::Error> {
        let key = key_builder.build();
        
        match self.kv_store.get(&key).text().await? {
            Some(serialized) => {
                let mut cached_data: CachedData<T> = serde_json::from_str(&serialized)
                    .map_err(|e| worker::Error::RustError(format!("Deserialization error: {}", e)))?;

                if cached_data.is_valid() {
                    let data = cached_data.access_data().clone();
                    
                    // Update metadata in cache
                    let updated_serialized = serde_json::to_string(&cached_data)
                        .map_err(|e| worker::Error::RustError(format!("Serialization error: {}", e)))?;
                    
                    // Update cache with new metadata (fire and forget)
                    let _ = self.kv_store.put(&key, updated_serialized)?.execute().await;
                    
                    // Record metrics
                    self.record_cache_operation("hit", &key, cached_data.metadata.data_size).await;
                    
                    Ok(Some(data))
                } else {
                    // Data expired, remove from cache
                    let _ = self.kv_store.delete(&key).await;
                    self.record_cache_operation("expired", &key, 0).await;
                    Ok(None)
                }
            }
            None => {
                self.record_cache_operation("miss", &key, 0).await;
                Ok(None)
            }
        }
    }

    /// Cache-aside pattern: get from cache or compute and store
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key_builder: KvKeyBuilder,
        ttl: CacheTTL,
        compute_fn: F,
    ) -> Result<T, worker::Error>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, worker::Error>>,
    {
        // Try to get from cache first
        if let Some(cached_data) = self.get_with_metadata::<T>(key_builder.clone()).await? {
            return Ok(cached_data);
        }

        // Cache miss, compute the value
        let computed_data = compute_fn().await?;

        // Store in cache
        self.put_with_ttl(key_builder, computed_data.clone(), ttl).await?;

        Ok(computed_data)
    }

    /// Invalidate cache entries by pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<u32, worker::Error> {
        // Note: KV doesn't support pattern deletion, so this would need to be implemented
        // by maintaining an index of keys or using a different approach
        // For now, we'll just record the invalidation attempt
        self.record_cache_operation("invalidate_pattern", pattern, 0).await;
        Ok(0)
    }

    /// Record cache operation metrics
    async fn record_cache_operation(&self, operation: &str, key: &str, size: usize) {
        let metrics_key = KvKeyBuilder::new(KeyPrefix::Metrics)
            .add_component("cache")
            .add_component(&self.service_name)
            .add_component(operation)
            .build();

        let metric_data = serde_json::json!({
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "operation": operation,
            "key": key,
            "size": size,
            "service": self.service_name
        });

        // Store metrics (fire and forget)
        let _ = self.kv_store
            .put(&metrics_key, metric_data.to_string())
            .unwrap()
            .expiration_ttl(CacheTTL::Long.as_seconds())
            .execute()
            .await;
    }
}

/// Service-specific KV helpers
pub mod service_helpers {
    use super::*;

    /// User profile cache operations
    pub struct UserProfileCache {
        kv_ops: StandardKvOperations,
    }

    impl UserProfileCache {
        pub fn new(kv_store: KvStore) -> Self {
            Self {
                kv_ops: StandardKvOperations::new(kv_store, "user_profile".to_string()),
            }
        }

        pub async fn get_profile<T: for<'de> Deserialize<'de> + Serialize + Clone>(
            &self,
            user_id: &str,
        ) -> Result<Option<T>, worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::UserProfile)
                .add_component(user_id);
            
            self.kv_ops.get_with_metadata(key).await
        }

        pub async fn put_profile<T: Serialize>(
            &self,
            user_id: &str,
            profile: T,
        ) -> Result<(), worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::UserProfile)
                .add_component(user_id);
            
            self.kv_ops.put_with_ttl(key, profile, CacheTTL::Medium).await
        }
    }

    /// Position cache operations
    pub struct PositionCache {
        kv_ops: StandardKvOperations,
    }

    impl PositionCache {
        pub fn new(kv_store: KvStore) -> Self {
            Self {
                kv_ops: StandardKvOperations::new(kv_store, "positions".to_string()),
            }
        }

        pub async fn get_user_positions<T: for<'de> Deserialize<'de> + Serialize + Clone>(
            &self,
            user_id: &str,
        ) -> Result<Option<T>, worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::Position)
                .add_component(user_id);
            
            self.kv_ops.get_with_metadata(key).await
        }

        pub async fn put_user_positions<T: Serialize>(
            &self,
            user_id: &str,
            positions: T,
        ) -> Result<(), worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::Position)
                .add_component(user_id);
            
            self.kv_ops.put_with_ttl(key, positions, CacheTTL::Short).await
        }

        pub async fn get_position_index<T: for<'de> Deserialize<'de> + Serialize + Clone>(
            &self,
        ) -> Result<Option<T>, worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::Position)
                .add_component("index");
            
            self.kv_ops.get_with_metadata(key).await
        }

        pub async fn put_position_index<T: Serialize>(
            &self,
            index: T,
        ) -> Result<(), worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::Position)
                .add_component("index");
            
            self.kv_ops.put_with_ttl(key, index, CacheTTL::Short).await
        }
    }

    /// Market data cache operations
    pub struct MarketDataCache {
        kv_ops: StandardKvOperations,
    }

    impl MarketDataCache {
        pub fn new(kv_store: KvStore) -> Self {
            Self {
                kv_ops: StandardKvOperations::new(kv_store, "market_data".to_string()),
            }
        }

        pub async fn get_ticker<T: for<'de> Deserialize<'de> + Serialize + Clone>(
            &self,
            exchange: &str,
            symbol: &str,
        ) -> Result<Option<T>, worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::MarketData)
                .add_component("ticker")
                .add_component(exchange)
                .add_component(symbol);
            
            self.kv_ops.get_with_metadata(key).await
        }

        pub async fn put_ticker<T: Serialize>(
            &self,
            exchange: &str,
            symbol: &str,
            ticker: T,
        ) -> Result<(), worker::Error> {
            let key = KvKeyBuilder::new(KeyPrefix::MarketData)
                .add_component("ticker")
                .add_component(exchange)
                .add_component(symbol);
            
            self.kv_ops.put_with_ttl(key, ticker, CacheTTL::RealTime).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_builder() {
        let key = KvKeyBuilder::new(KeyPrefix::UserProfile)
            .add_component("user123")
            .add_component("preferences")
            .build();
        
        assert_eq!(key, "user_profile:user123:preferences");
    }

    #[test]
    fn test_cache_metadata_expiration() {
        let metadata = CacheMetadata::new(CacheTTL::Short, 100, "test_service".to_string());
        assert!(!metadata.is_expired());
        
        // Test with expired metadata
        let mut expired_metadata = metadata.clone();
        expired_metadata.expires_at = 0; // Set to past
        assert!(expired_metadata.is_expired());
    }

    #[test]
    fn test_cached_data_validity() {
        let data = "test_data".to_string();
        let cached = CachedData::new(data, CacheTTL::Short, "test_service".to_string());
        assert!(cached.is_valid());
    }

    #[test]
    fn test_ttl_values() {
        assert_eq!(CacheTTL::RealTime.as_seconds(), 30);
        assert_eq!(CacheTTL::Short.as_seconds(), 300);
        assert_eq!(CacheTTL::Medium.as_seconds(), 3600);
        assert_eq!(CacheTTL::Long.as_seconds(), 86400);
        assert_eq!(CacheTTL::VeryLong.as_seconds(), 604800);
    }

    #[test]
    fn test_key_prefixes() {
        assert_eq!(KeyPrefix::UserProfile.as_str(), "user_profile");
        assert_eq!(KeyPrefix::Position.as_str(), "positions");
        assert_eq!(KeyPrefix::MarketData.as_str(), "market_data");
    }
} 