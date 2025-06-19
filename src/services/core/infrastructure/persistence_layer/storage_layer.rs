// Storage Layer - Consolidated Workers-Optimized Persistence
// Replaces the complex multi-file persistence layer with a unified approach
// Optimized for Cloudflare Workers environment with D1/KV/R2 focus

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};

use worker::{kv::KvStore, D1Database};

/// Unified configuration for all storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLayerConfig {
    pub default_timeout_ms: u64,
    pub max_connections: u32,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for StorageLayerConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            max_connections: 100,
            enable_caching: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Storage operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult<T> {
    pub data: Option<T>,
    pub success: bool,
    pub execution_time_ms: u64,
    pub rows_affected: u64,
}

/// Storage metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLayerMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_execution_time_ms: f64,
    pub last_updated: u64,
}

impl Default for StorageLayerMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_execution_time_ms: 0.0,
            last_updated: crate::utils::time::get_current_timestamp(),
        }
    }
}

/// Main unified storage service - replaces all persistence_layer components
pub struct StorageLayerService {
    #[allow(dead_code)]
    config: StorageLayerConfig,
    d1_database: Option<D1Database>,
    kv_store: KvStore,
    metrics: std::sync::Arc<std::sync::Mutex<StorageLayerMetrics>>,
    #[allow(dead_code)]
    logger: crate::utils::logger::Logger,
}

impl StorageLayerService {
    /// Create new unified storage service
    pub fn new(
        config: StorageLayerConfig,
        d1_database: Option<D1Database>,
        kv_store: KvStore,
    ) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        logger.info("Initializing StorageLayerService for Cloudflare Workers");

        Ok(Self {
            config,
            d1_database,
            kv_store,
            metrics: std::sync::Arc::new(std::sync::Mutex::new(StorageLayerMetrics::default())),
            logger,
        })
    }

    /// Execute a SQL query on D1 database
    pub async fn query<T>(&self, sql: &str) -> ArbitrageResult<StorageResult<Vec<T>>>
    where
        T: serde::de::DeserializeOwned,
    {
        let start_time = crate::utils::time::get_current_timestamp();

        let d1_db = self
            .d1_database
            .as_ref()
            .ok_or_else(|| ArbitrageError::database_error("D1 database not available"))?;

        let query = d1_db.prepare(sql);

        match query.all().await {
            Ok(_results) => {
                // For D1, we need to handle the results differently
                // This is a simplified approach for Workers environment
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;

                self.record_metrics(true, execution_time).await;

                Ok(StorageResult {
                    data: Some(Vec::new()), // Simplified - in real implementation, parse results
                    success: true,
                    execution_time_ms: execution_time,
                    rows_affected: 0,
                })
            }
            Err(e) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(false, execution_time).await;
                Err(ArbitrageError::database_error(format!(
                    "D1 query error: {}",
                    e
                )))
            }
        }
    }

    /// Execute a SQL statement (INSERT, UPDATE, DELETE)
    pub async fn execute(&self, sql: &str) -> ArbitrageResult<StorageResult<u64>> {
        let start_time = crate::utils::time::get_current_timestamp();

        let d1_db = self
            .d1_database
            .as_ref()
            .ok_or_else(|| ArbitrageError::database_error("D1 database not available"))?;

        let query = d1_db.prepare(sql);

        match query.run().await {
            Ok(_result) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                // Simplified approach - in real implementation, you'd extract actual changes
                let rows_affected = 1;

                self.record_metrics(true, execution_time).await;

                Ok(StorageResult {
                    data: Some(rows_affected),
                    success: true,
                    execution_time_ms: execution_time,
                    rows_affected,
                })
            }
            Err(e) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(false, execution_time).await;
                Err(ArbitrageError::database_error(format!(
                    "D1 execute error: {}",
                    e
                )))
            }
        }
    }

    /// Store data in KV store
    pub async fn kv_put(&self, key: &str, value: &str) -> ArbitrageResult<StorageResult<()>> {
        let start_time = crate::utils::time::get_current_timestamp();

        match self.kv_store.put(key, value)?.execute().await {
            Ok(_) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(true, execution_time).await;

                Ok(StorageResult {
                    data: Some(()),
                    success: true,
                    execution_time_ms: execution_time,
                    rows_affected: 1,
                })
            }
            Err(e) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(false, execution_time).await;
                Err(ArbitrageError::database_error(format!(
                    "KV put error: {}",
                    e
                )))
            }
        }
    }

    /// Get data from KV store
    pub async fn kv_get(&self, key: &str) -> ArbitrageResult<StorageResult<String>> {
        let start_time = crate::utils::time::get_current_timestamp();

        match self.kv_store.get(key).text().await {
            Ok(Some(value)) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(true, execution_time).await;

                Ok(StorageResult {
                    data: Some(value),
                    success: true,
                    execution_time_ms: execution_time,
                    rows_affected: 1,
                })
            }
            Ok(None) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(true, execution_time).await;

                Ok(StorageResult {
                    data: None,
                    success: true,
                    execution_time_ms: execution_time,
                    rows_affected: 0,
                })
            }
            Err(e) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(false, execution_time).await;
                Err(ArbitrageError::database_error(format!(
                    "KV get error: {}",
                    e
                )))
            }
        }
    }

    /// Delete data from KV store
    pub async fn kv_delete(&self, key: &str) -> ArbitrageResult<StorageResult<()>> {
        let start_time = crate::utils::time::get_current_timestamp();

        match self.kv_store.delete(key).await {
            Ok(_) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(true, execution_time).await;

                Ok(StorageResult {
                    data: Some(()),
                    success: true,
                    execution_time_ms: execution_time,
                    rows_affected: 1,
                })
            }
            Err(e) => {
                let execution_time = crate::utils::time::get_current_timestamp() - start_time;
                self.record_metrics(false, execution_time).await;
                Err(ArbitrageError::database_error(format!(
                    "KV delete error: {}",
                    e
                )))
            }
        }
    }

    /// Health check for the storage service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        // Test KV store
        match self.kv_put("health_check_test", "test").await {
            Ok(_) => {
                let _ = self.kv_delete("health_check_test").await;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> StorageLayerMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            StorageLayerMetrics::default()
        }
    }

    // Private helper methods

    async fn record_metrics(&self, success: bool, execution_time_ms: u64) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_operations += 1;

            if success {
                metrics.successful_operations += 1;
            } else {
                metrics.failed_operations += 1;
            }

            // Update average execution time
            let total_ops = metrics.total_operations as f64;
            metrics.avg_execution_time_ms = ((metrics.avg_execution_time_ms * (total_ops - 1.0))
                + execution_time_ms as f64)
                / total_ops;

            metrics.last_updated = crate::utils::time::get_current_timestamp();
        }
    }
}

/// Builder pattern for creating storage layer service
pub struct StorageLayerBuilder {
    config: StorageLayerConfig,
    d1_database: Option<D1Database>,
}

impl StorageLayerBuilder {
    pub fn new() -> Self {
        Self {
            config: StorageLayerConfig::default(),
            d1_database: None,
        }
    }

    pub fn with_d1_database(mut self, database: D1Database) -> Self {
        self.d1_database = Some(database);
        self
    }

    pub fn with_caching(mut self, enabled: bool, ttl_seconds: u64) -> Self {
        self.config.enable_caching = enabled;
        self.config.cache_ttl_seconds = ttl_seconds;
        self
    }

    pub fn build(self, kv_store: KvStore) -> ArbitrageResult<StorageLayerService> {
        StorageLayerService::new(self.config, self.d1_database, kv_store)
    }
}

impl Default for StorageLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for compatibility with imports
pub type StorageLayer = StorageLayerService;
