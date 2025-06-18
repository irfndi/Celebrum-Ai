// Unified Data Access Layer - Consolidated Workers-Optimized Data Access
// Replaces the complex multi-file data access layer with a simplified, unified approach
// Optimized for Cloudflare Workers environment with KV/D1/R2 focus

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::kv::KvStore;

/// Unified configuration for all data access operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataAccessConfig {
    // Core access settings
    pub default_timeout_ms: u64,
    pub max_concurrent_operations: u32,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    
    // Performance settings
    pub enable_compression: bool,
    pub enable_warming: bool,
    pub enable_validation: bool,
    pub enable_metrics: bool,
    
    // Batch operations
    pub batch_size: usize,
    pub max_retry_attempts: u32,
    pub retry_backoff_ms: u64,
}

impl Default for UnifiedDataAccessConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            max_concurrent_operations: 100,
            enable_caching: true,
            cache_ttl_seconds: 3600,
            enable_compression: true,
            enable_warming: false,
            enable_validation: true,
            enable_metrics: true,
            batch_size: 50,
            max_retry_attempts: 3,
            retry_backoff_ms: 1000,
        }
    }
}

/// Unified data access operations result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessResult<T> {
    pub data: Option<T>,
    pub cached: bool,
    pub latency_ms: u64,
    pub source: DataSource,
    pub validation_passed: bool,
}

/// Data source identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    KV,
    D1,
    R2,
    External(String),
    Cache,
}

/// Unified metrics for data access operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataAccessMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_latency_ms: f64,
    pub compression_ratio: f64,
    pub validation_errors: u64,
    pub last_updated: u64,
}

impl Default for UnifiedDataAccessMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_latency_ms: 0.0,
            compression_ratio: 1.0,
            validation_errors: 0,
            last_updated: crate::utils::time::get_current_timestamp(),
        }
    }
}

/// Main unified data access service - replaces all data_access_layer components
pub struct UnifiedDataAccessService {
    config: UnifiedDataAccessConfig,
    kv_store: KvStore,
    metrics: std::sync::Arc<std::sync::Mutex<UnifiedDataAccessMetrics>>,
    cache: std::sync::Arc<std::sync::Mutex<HashMap<String, CacheEntry>>>,
    logger: crate::utils::logger::Logger,
}

/// Simplified cache entry for unified access
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    pub data: String,
    pub expires_at: u64,
    pub created_at: u64,
    pub access_count: u64,
    pub compressed: bool,
}

impl UnifiedDataAccessService {
    /// Create new unified data access service
    pub fn new(config: UnifiedDataAccessConfig, kv_store: KvStore) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
        
        logger.info("Initializing UnifiedDataAccessService for Cloudflare Workers");
        
        Ok(Self {
            config,
            kv_store,
            metrics: std::sync::Arc::new(std::sync::Mutex::new(UnifiedDataAccessMetrics::default())),
            cache: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            logger,
        })
    }

    /// Get data with automatic caching, validation, and compression
    pub async fn get<T>(&self, key: &str) -> ArbitrageResult<DataAccessResult<T>>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let start_time = crate::utils::time::get_current_timestamp();
        
        // Check cache first if enabled
        if self.config.enable_caching {
            if let Some(cached_data) = self.get_from_cache(key).await? {
                return Ok(DataAccessResult {
                    data: Some(serde_json::from_str(&cached_data)?),
                    cached: true,
                    latency_ms: crate::utils::time::get_current_timestamp() - start_time,
                    source: DataSource::Cache,
                    validation_passed: true,
                });
            }
        }

        // Get from KV store
        let kv_result = self.kv_store.get(key).text().await;
        
        match kv_result {
            Ok(Some(data)) => {
                // Validate if enabled
                let validation_passed = if self.config.enable_validation {
                    self.validate_data(&data).await
                } else {
                    true
                };

                // Cache if enabled and validation passed
                if self.config.enable_caching && validation_passed {
                    self.set_cache(key, &data).await?;
                }

                let parsed_data: T = serde_json::from_str(&data)?;
                
                self.record_metrics(true, start_time, false).await;
                
                Ok(DataAccessResult {
                    data: Some(parsed_data),
                    cached: false,
                    latency_ms: crate::utils::time::get_current_timestamp() - start_time,
                    source: DataSource::KV,
                    validation_passed,
                })
            }
            Ok(None) => {
                self.record_metrics(true, start_time, true).await;
                Ok(DataAccessResult {
                    data: None,
                    cached: false,
                    latency_ms: crate::utils::time::get_current_timestamp() - start_time,
                    source: DataSource::KV,
                    validation_passed: true,
                })
            }
            Err(e) => {
                self.record_metrics(false, start_time, true).await;
                Err(ArbitrageError::database_error(format!("KV get error: {}", e)))
            }
        }
    }

    /// Set data with automatic caching, validation, and compression
    pub async fn set<T>(&self, key: &str, value: &T) -> ArbitrageResult<()>
    where
        T: serde::Serialize,
    {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let serialized = serde_json::to_string(value)?;
        
        // Validate if enabled
        if self.config.enable_validation && !self.validate_data(&serialized).await {
            self.record_metrics(false, start_time, false).await;
            return Err(ArbitrageError::validation_error("Data validation failed"));
        }

        // Compress if enabled and data is large enough
        let final_data = if self.config.enable_compression && serialized.len() > 1024 {
            self.compress_data(&serialized)?
        } else {
            serialized.clone()
        };

        // Set in KV store
        match self.kv_store.put(key, final_data)?.execute().await {
            Ok(_) => {
                // Update cache if enabled
                if self.config.enable_caching {
                    self.set_cache(key, &serialized).await?;
                }
                
                self.record_metrics(true, start_time, false).await;
                Ok(())
            }
            Err(e) => {
                self.record_metrics(false, start_time, false).await;
                Err(ArbitrageError::database_error(format!("KV set error: {}", e)))
            }
        }
    }

    /// Delete data and clear from cache
    pub async fn delete(&self, key: &str) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        // Delete from KV store
        match self.kv_store.delete(key).await {
            Ok(_) => {
                // Clear from cache
                if self.config.enable_caching {
                    self.clear_cache(key).await;
                }
                
                self.record_metrics(true, start_time, false).await;
                Ok(())
            }
            Err(e) => {
                self.record_metrics(false, start_time, false).await;
                Err(ArbitrageError::database_error(format!("KV delete error: {}", e)))
            }
        }
    }

    /// Batch operations for improved performance
    pub async fn batch_get<T>(&self, keys: &[String]) -> ArbitrageResult<Vec<DataAccessResult<T>>>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let mut results = Vec::new();
        
        // Process in batches to avoid overwhelming the system
        for chunk in keys.chunks(self.config.batch_size) {
            for key in chunk {
                match self.get(key).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        self.logger.error(&format!("Batch get error for key {}: {}", key, e));
                        results.push(DataAccessResult {
                            data: None,
                            cached: false,
                            latency_ms: 0,
                            source: DataSource::KV,
                            validation_passed: false,
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Health check for the unified data access service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        // Test KV store with a simple operation
        let test_key = "health_check_test";
        let test_data = "test";
        
        match self.kv_store.put(test_key, test_data)?.execute().await {
            Ok(_) => {
                // Clean up test data
                let _ = self.kv_store.delete(test_key).await;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> UnifiedDataAccessMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            UnifiedDataAccessMetrics::default()
        }
    }

    // Private helper methods

    async fn get_from_cache(&self, key: &str) -> ArbitrageResult<Option<String>> {
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(key) {
                let now = crate::utils::time::get_current_timestamp();
                if now < entry.expires_at {
                    return Ok(Some(entry.data.clone()));
                }
            }
        }
        Ok(None)
    }

    async fn set_cache(&self, key: &str, data: &str) -> ArbitrageResult<()> {
        if let Ok(mut cache) = self.cache.lock() {
            let now = crate::utils::time::get_current_timestamp();
            let entry = CacheEntry {
                data: data.to_string(),
                expires_at: now + (self.config.cache_ttl_seconds * 1000),
                created_at: now,
                access_count: 1,
                compressed: false,
            };
            cache.insert(key.to_string(), entry);
        }
        Ok(())
    }

    async fn clear_cache(&self, key: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(key);
        }
    }

    async fn validate_data(&self, _data: &str) -> bool {
        // Basic validation - can be extended based on requirements
        // For now, just check if it's valid JSON
        serde_json::from_str::<serde_json::Value>(_data).is_ok()
    }

    fn compress_data(&self, data: &str) -> ArbitrageResult<String> {
        // Simple compression placeholder - in real implementation could use gzip
        // For now, just return the original data
        Ok(data.to_string())
    }

    async fn record_metrics(&self, success: bool, start_time: u64, cache_miss: bool) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_operations += 1;
            
            if success {
                metrics.successful_operations += 1;
            } else {
                metrics.failed_operations += 1;
            }
            
            if cache_miss {
                metrics.cache_misses += 1;
            } else {
                metrics.cache_hits += 1;
            }
            
            let latency = crate::utils::time::get_current_timestamp() - start_time;
            metrics.avg_latency_ms = ((metrics.avg_latency_ms * (metrics.total_operations - 1) as f64) + latency as f64) / metrics.total_operations as f64;
            metrics.last_updated = crate::utils::time::get_current_timestamp();
        }
    }
}

/// Builder pattern for creating unified data access service
pub struct UnifiedDataAccessBuilder {
    config: UnifiedDataAccessConfig,
}

impl UnifiedDataAccessBuilder {
    pub fn new() -> Self {
        Self {
            config: UnifiedDataAccessConfig::default(),
        }
    }
    
    pub fn with_caching(mut self, enabled: bool, ttl_seconds: u64) -> Self {
        self.config.enable_caching = enabled;
        self.config.cache_ttl_seconds = ttl_seconds;
        self
    }
    
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.config.enable_compression = enabled;
        self
    }
    
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.config.enable_validation = enabled;
        self
    }
    
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }
    
    pub fn build(self, kv_store: KvStore) -> ArbitrageResult<UnifiedDataAccessService> {
        UnifiedDataAccessService::new(self.config, kv_store)
    }
}

impl Default for UnifiedDataAccessBuilder {
    fn default() -> Self {
        Self::new()
    }
} 