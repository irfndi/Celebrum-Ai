// Data Source Manager - Multi-source Data Coordination Component
// Provides hierarchical data access with Pipeline → KV → Database → API fallback

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use worker::kv::KvStore;

/// Data source types in priority order
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceType {
    Pipeline, // Cloudflare Pipelines (highest priority)
    Cache,    // KV Store (second priority)
    Database, // D1 Database (third priority)
    API,      // External APIs (lowest priority)
}

impl DataSourceType {
    pub fn as_str(&self) -> &str {
        match self {
            DataSourceType::Pipeline => "pipeline",
            DataSourceType::Cache => "cache",
            DataSourceType::Database => "database",
            DataSourceType::API => "api",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            DataSourceType::Pipeline => 1,
            DataSourceType::Cache => 2,
            DataSourceType::Database => 3,
            DataSourceType::API => 4,
        }
    }
}

/// Data source health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceHealth {
    pub source_type: DataSourceType,
    pub is_healthy: bool,
    pub last_success_timestamp: u64,
    pub last_error: Option<String>,
    pub success_rate_percent: f32,
    pub average_latency_ms: f64,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub circuit_breaker_open: bool,
    pub last_health_check: u64,
}

impl Default for DataSourceHealth {
    fn default() -> Self {
        Self {
            source_type: DataSourceType::Cache,
            is_healthy: false,
            last_success_timestamp: 0,
            last_error: None,
            success_rate_percent: 0.0,
            average_latency_ms: 0.0,
            total_requests: 0,
            failed_requests: 0,
            circuit_breaker_open: false,
            last_health_check: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Circuit breaker state for data sources
#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Circuit breaker is open, requests fail fast
    HalfOpen, // Testing if service is back
}

/// Circuit breaker for individual data sources
#[derive(Debug, Clone)]
struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    failure_threshold: u32,
    timeout_ms: u64,
    last_failure_time: Option<u64>,
    success_count: u32,
    half_open_max_calls: u32,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout_ms: u64) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout_ms,
            last_failure_time: None,
            success_count: 0,
            half_open_max_calls: 3,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    let now = chrono::Utc::now().timestamp_millis() as u64;
                    if now - last_failure > self.timeout_ms {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => self.success_count < self.half_open_max_calls,
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.half_open_max_calls {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitBreakerState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(chrono::Utc::now().timestamp_millis() as u64);

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
            }
            CircuitBreakerState::Open => {}
        }
    }

    fn is_open(&self) -> bool {
        matches!(self.state, CircuitBreakerState::Open)
    }
}

/// Configuration for the data source manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceManagerConfig {
    pub enable_kv_store: bool,
    pub enable_d1_database: bool,
    pub enable_external_apis: bool,
    pub enable_caching: bool,
    pub enable_fallback_chain: bool,
    pub enable_automatic_failover: bool,
    pub enable_circuit_breakers: bool,
    pub enable_performance_tracking: bool,
    pub connection_pool_size: u32,
    pub default_timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub cache_ttl_seconds: u64,
    pub enable_compression: bool,
    pub compression_threshold_bytes: usize,
    pub enable_metrics: bool,
    pub enable_health_checks: bool,
    pub health_check_interval_seconds: u64,
}

impl Default for DataSourceManagerConfig {
    fn default() -> Self {
        Self {
            enable_kv_store: true,
            enable_d1_database: true,
            enable_external_apis: true,
            enable_caching: true,
            enable_fallback_chain: true,
            enable_automatic_failover: true,
            enable_circuit_breakers: true,
            enable_performance_tracking: true,
            connection_pool_size: 15,
            default_timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            cache_ttl_seconds: 300,
            enable_compression: true,
            compression_threshold_bytes: 1024,
            enable_metrics: true,
            enable_health_checks: true,
            health_check_interval_seconds: 30,
        }
    }
}

impl DataSourceManagerConfig {
    /// Create configuration optimized for high concurrency
    pub fn high_concurrency() -> Self {
        Self {
            default_timeout_seconds: 15,
            max_retries: 2,
            health_check_interval_seconds: 15,
            retry_delay_ms: 500,
            cache_ttl_seconds: 180, // 3 minutes
            ..Default::default()
        }
    }

    /// Create configuration optimized for reliability
    pub fn high_reliability() -> Self {
        Self {
            max_retries: 5,
            health_check_interval_seconds: 60,
            default_timeout_seconds: 60,
            retry_delay_ms: 2000,
            cache_ttl_seconds: 600, // 10 minutes
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.default_timeout_seconds == 0 {
            return Err(ArbitrageError::validation_error(
                "default_timeout_seconds must be greater than 0",
            ));
        }
        if self.max_retries > 10 {
            return Err(ArbitrageError::validation_error(
                "max_retries must not exceed 10",
            ));
        }
        if self.cache_ttl_seconds == 0 {
            return Err(ArbitrageError::validation_error(
                "cache_ttl_seconds must be greater than 0",
            ));
        }
        Ok(())
    }
}

/// Data source performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceMetrics {
    pub source_type: DataSourceType,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub success_rate_percent: f32,
    pub circuit_breaker_trips: u64,
    pub failover_count: u64,
    pub last_updated: u64,
}

impl Default for DataSourceMetrics {
    fn default() -> Self {
        Self {
            source_type: DataSourceType::Cache,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_latency_ms: 0.0,
            min_latency_ms: f64::MAX,
            max_latency_ms: 0.0,
            success_rate_percent: 0.0,
            circuit_breaker_trips: 0,
            failover_count: 0,
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Data source manager for hierarchical data access
pub struct DataSourceManager {
    config: DataSourceManagerConfig,
    logger: crate::utils::logger::Logger,
    kv_store: KvStore,

    // Circuit breakers for each data source
    circuit_breakers: Arc<std::sync::Mutex<HashMap<DataSourceType, CircuitBreaker>>>,

    // Health status for each data source
    health_status: Arc<std::sync::Mutex<HashMap<DataSourceType, DataSourceHealth>>>,

    // Performance metrics for each data source
    metrics: Arc<std::sync::Mutex<HashMap<DataSourceType, DataSourceMetrics>>>,

    // Active connections count
    active_connections: Arc<std::sync::Mutex<u32>>,
}

impl DataSourceManager {
    /// Create new DataSourceManager instance
    pub fn new(kv_store: KvStore, mut config: DataSourceManagerConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        // Validate configuration
        config.validate()?;

        // Initialize circuit breakers
        let mut circuit_breakers = HashMap::new();
        for source_type in [
            DataSourceType::Pipeline,
            DataSourceType::Cache,
            DataSourceType::Database,
            DataSourceType::API,
        ] {
            circuit_breakers.insert(
                source_type.clone(),
                CircuitBreaker::new(5, 60000), // Default: 5 failures, 60s timeout
            );
        }

        // Initialize health status
        let mut health_status = HashMap::new();
        for source_type in [
            DataSourceType::Pipeline,
            DataSourceType::Cache,
            DataSourceType::Database,
            DataSourceType::API,
        ] {
            let mut health = DataSourceHealth::default();
            health.source_type = source_type.clone();
            health_status.insert(source_type, health);
        }

        // Initialize metrics
        let mut metrics = HashMap::new();
        for source_type in [
            DataSourceType::Pipeline,
            DataSourceType::Cache,
            DataSourceType::Database,
            DataSourceType::API,
        ] {
            let mut metric = DataSourceMetrics::default();
            metric.source_type = source_type.clone();
            metrics.insert(source_type, metric);
        }

        let manager = Self {
            config,
            logger,
            kv_store,
            circuit_breakers: Arc::new(std::sync::Mutex::new(circuit_breakers)),
            health_status: Arc::new(std::sync::Mutex::new(health_status)),
            metrics: Arc::new(std::sync::Mutex::new(metrics)),
            active_connections: Arc::new(std::sync::Mutex::new(0)),
        };

        manager.logger.info(&format!(
            "DataSourceManager initialized: caching={}, fallback={}, health_monitoring={}",
            manager.config.enable_caching,
            manager.config.enable_fallback_chain,
            manager.config.enable_health_checks
        ));

        Ok(manager)
    }

    /// Get data with automatic failover through the hierarchy
    pub async fn get_data<T>(&self, key: &str, data_type: &str) -> ArbitrageResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let sources = vec![
            DataSourceType::Pipeline,
            DataSourceType::Cache,
            DataSourceType::Database,
            DataSourceType::API,
        ];

        for source_type in sources {
            // Check circuit breaker
            if !self.can_use_source(&source_type).await {
                self.logger.warn(&format!(
                    "Skipping {} due to circuit breaker",
                    source_type.as_str()
                ));
                continue;
            }

            // Check connection limit
            if !self.check_connection_limit().await {
                self.logger
                    .warn("Connection limit reached, skipping request");
                continue;
            }

            let start_time = chrono::Utc::now().timestamp_millis() as u64;

            match self
                .get_from_source::<T>(&source_type, key, data_type)
                .await
            {
                Ok(Some(data)) => {
                    self.record_success(&source_type, start_time).await;
                    return Ok(Some(data));
                }
                Ok(None) => {
                    // Data not found in this source, try next
                    self.record_success(&source_type, start_time).await;
                    continue;
                }
                Err(e) => {
                    self.record_failure(&source_type, start_time, &e).await;
                    self.logger.warn(&format!(
                        "Failed to get data from {}: {}",
                        source_type.as_str(),
                        e
                    ));

                    if !self.config.enable_automatic_failover {
                        return Err(e);
                    }
                    continue;
                }
            }
        }

        // All sources failed or returned no data
        Ok(None)
    }

    /// Set data with write-through to appropriate sources
    pub async fn set_data<T>(
        &self,
        key: &str,
        data: &T,
        data_type: &str,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()>
    where
        T: serde::Serialize,
    {
        let mut write_success = false;
        let mut last_error = None;

        // Write to cache (primary write target)
        if self.can_use_source(&DataSourceType::Cache).await {
            let start_time = chrono::Utc::now().timestamp_millis() as u64;
            match self
                .set_to_source(&DataSourceType::Cache, key, data, data_type, ttl_seconds)
                .await
            {
                Ok(_) => {
                    self.record_success(&DataSourceType::Cache, start_time)
                        .await;
                    write_success = true;
                }
                Err(e) => {
                    self.record_failure(&DataSourceType::Cache, start_time, &e)
                        .await;
                    last_error = Some(e);
                }
            }
        }

        // Write to pipeline if available (for persistence)
        if self.can_use_source(&DataSourceType::Pipeline).await {
            let start_time = chrono::Utc::now().timestamp_millis() as u64;
            match self
                .set_to_source(&DataSourceType::Pipeline, key, data, data_type, ttl_seconds)
                .await
            {
                Ok(_) => {
                    self.record_success(&DataSourceType::Pipeline, start_time)
                        .await;
                    write_success = true;
                }
                Err(e) => {
                    self.record_failure(&DataSourceType::Pipeline, start_time, &e)
                        .await;
                    if last_error.is_none() {
                        last_error = Some(e);
                    }
                }
            }
        }

        if write_success {
            Ok(())
        } else {
            Err(last_error
                .unwrap_or_else(|| ArbitrageError::parse_error("All data sources unavailable")))
        }
    }

    /// Get data from specific source
    async fn get_from_source<T>(
        &self,
        source_type: &DataSourceType,
        key: &str,
        _data_type: &str,
    ) -> ArbitrageResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        match source_type {
            DataSourceType::Cache => match self.kv_store.get(key).text().await {
                Ok(Some(data_str)) => match serde_json::from_str::<T>(&data_str) {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => Err(ArbitrageError::parse_error(format!(
                        "Failed to deserialize data: {}",
                        e
                    ))),
                },
                Ok(None) => Ok(None),
                Err(e) => Err(ArbitrageError::cache_error(format!("KV get failed: {}", e))),
            },
            DataSourceType::Pipeline => {
                // Placeholder for pipeline data access
                // In a real implementation, this would query Cloudflare Pipelines
                Ok(None)
            }
            DataSourceType::Database => {
                // Placeholder for database data access
                // In a real implementation, this would query D1 database
                Ok(None)
            }
            DataSourceType::API => {
                // Placeholder for external API data access
                // In a real implementation, this would call external APIs
                Ok(None)
            }
        }
    }

    /// Set data to specific source
    async fn set_to_source<T>(
        &self,
        source_type: &DataSourceType,
        key: &str,
        data: &T,
        _data_type: &str,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()>
    where
        T: serde::Serialize,
    {
        match source_type {
            DataSourceType::Cache => {
                let data_str = serde_json::to_string(data).map_err(|e| {
                    ArbitrageError::parse_error(format!("Failed to serialize data: {}", e))
                })?;

                let mut put_request = self.kv_store.put(key, &data_str)?;
                if let Some(ttl) = ttl_seconds {
                    put_request = put_request.expiration_ttl(ttl);
                }

                put_request
                    .execute()
                    .await
                    .map_err(|e| ArbitrageError::cache_error(format!("KV put failed: {}", e)))?;

                Ok(())
            }
            DataSourceType::Pipeline => {
                // Placeholder for pipeline data storage
                // In a real implementation, this would write to Cloudflare Pipelines
                Ok(())
            }
            DataSourceType::Database => {
                // Placeholder for database data storage
                // In a real implementation, this would write to D1 database
                Ok(())
            }
            DataSourceType::API => {
                // APIs are typically read-only, so this is a no-op
                Ok(())
            }
        }
    }

    /// Check if a data source can be used (circuit breaker check)
    async fn can_use_source(&self, source_type: &DataSourceType) -> bool {
        if !self.config.enable_circuit_breakers {
            return true;
        }

        if let Ok(mut breakers) = self.circuit_breakers.lock() {
            if let Some(breaker) = breakers.get_mut(source_type) {
                breaker.can_execute()
            } else {
                true
            }
        } else {
            true
        }
    }

    /// Check connection limit
    async fn check_connection_limit(&self) -> bool {
        if let Ok(mut connections) = self.active_connections.lock() {
            if *connections >= self.config.connection_pool_size {
                false
            } else {
                *connections += 1;
                true
            }
        } else {
            true
        }
    }

    /// Record successful operation
    async fn record_success(&self, source_type: &DataSourceType, start_time: u64) {
        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        let latency = end_time - start_time;

        // Update circuit breaker
        if self.config.enable_circuit_breakers {
            if let Ok(mut breakers) = self.circuit_breakers.lock() {
                if let Some(breaker) = breakers.get_mut(source_type) {
                    breaker.record_success();
                }
            }
        }

        // Update health status
        if let Ok(mut health) = self.health_status.lock() {
            if let Some(status) = health.get_mut(source_type) {
                status.is_healthy = true;
                status.last_success_timestamp = end_time;
                status.total_requests += 1;
                status.success_rate_percent = (status.total_requests - status.failed_requests)
                    as f32
                    / status.total_requests as f32
                    * 100.0;
                status.average_latency_ms = (status.average_latency_ms
                    * (status.total_requests - 1) as f64
                    + latency as f64)
                    / status.total_requests as f64;
                status.last_health_check = end_time;
            }
        }

        // Update metrics
        if self.config.enable_performance_tracking {
            if let Ok(mut metrics) = self.metrics.lock() {
                if let Some(metric) = metrics.get_mut(source_type) {
                    metric.total_requests += 1;
                    metric.successful_requests += 1;
                    metric.average_latency_ms = (metric.average_latency_ms
                        * (metric.total_requests - 1) as f64
                        + latency as f64)
                        / metric.total_requests as f64;
                    metric.min_latency_ms = metric.min_latency_ms.min(latency as f64);
                    metric.max_latency_ms = metric.max_latency_ms.max(latency as f64);
                    metric.success_rate_percent =
                        metric.successful_requests as f32 / metric.total_requests as f32 * 100.0;
                    metric.last_updated = end_time;
                }
            }
        }

        // Decrement active connections
        if let Ok(mut connections) = self.active_connections.lock() {
            if *connections > 0 {
                *connections -= 1;
            }
        }
    }

    /// Record failed operation
    async fn record_failure(
        &self,
        source_type: &DataSourceType,
        start_time: u64,
        error: &ArbitrageError,
    ) {
        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        let latency = end_time - start_time;

        // Update circuit breaker
        if self.config.enable_circuit_breakers {
            if let Ok(mut breakers) = self.circuit_breakers.lock() {
                if let Some(breaker) = breakers.get_mut(source_type) {
                    breaker.record_failure();
                }
            }
        }

        // Update health status
        if let Ok(mut health) = self.health_status.lock() {
            if let Some(status) = health.get_mut(source_type) {
                status.is_healthy = false;
                status.last_error = Some(error.to_string());
                status.total_requests += 1;
                status.failed_requests += 1;
                status.success_rate_percent = (status.total_requests - status.failed_requests)
                    as f32
                    / status.total_requests as f32
                    * 100.0;
                status.average_latency_ms = (status.average_latency_ms
                    * (status.total_requests - 1) as f64
                    + latency as f64)
                    / status.total_requests as f64;
                status.last_health_check = end_time;

                // Check circuit breaker status
                if let Ok(breakers) = self.circuit_breakers.lock() {
                    if let Some(breaker) = breakers.get(source_type) {
                        status.circuit_breaker_open = breaker.is_open();
                    }
                }
            }
        }

        // Update metrics
        if self.config.enable_performance_tracking {
            if let Ok(mut metrics) = self.metrics.lock() {
                if let Some(metric) = metrics.get_mut(source_type) {
                    metric.total_requests += 1;
                    metric.failed_requests += 1;
                    metric.average_latency_ms = (metric.average_latency_ms
                        * (metric.total_requests - 1) as f64
                        + latency as f64)
                        / metric.total_requests as f64;
                    metric.min_latency_ms = metric.min_latency_ms.min(latency as f64);
                    metric.max_latency_ms = metric.max_latency_ms.max(latency as f64);
                    metric.success_rate_percent =
                        metric.successful_requests as f32 / metric.total_requests as f32 * 100.0;
                    metric.last_updated = end_time;

                    // Check if this was a circuit breaker trip
                    if let Ok(breakers) = self.circuit_breakers.lock() {
                        if let Some(breaker) = breakers.get(source_type) {
                            if breaker.is_open() {
                                metric.circuit_breaker_trips += 1;
                            }
                        }
                    }
                }
            }
        }

        // Decrement active connections
        if let Ok(mut connections) = self.active_connections.lock() {
            if *connections > 0 {
                *connections -= 1;
            }
        }
    }

    /// Get health status for all data sources
    pub async fn get_health_status(&self) -> HashMap<DataSourceType, DataSourceHealth> {
        if let Ok(health) = self.health_status.lock() {
            health.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get performance metrics for all data sources
    pub async fn get_metrics(&self) -> HashMap<DataSourceType, DataSourceMetrics> {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get overall health summary
    pub async fn get_health_summary(&self) -> ArbitrageResult<serde_json::Value> {
        let health_status = self.get_health_status().await;
        let metrics = self.get_metrics().await;

        let total_sources = health_status.len();
        let healthy_sources = health_status.values().filter(|h| h.is_healthy).count();
        let sources_with_circuit_breakers = health_status
            .values()
            .filter(|h| h.circuit_breaker_open)
            .count();

        let overall_success_rate = if !metrics.is_empty() {
            metrics
                .values()
                .map(|m| m.success_rate_percent)
                .sum::<f32>()
                / metrics.len() as f32
        } else {
            0.0
        };

        let overall_latency = if !metrics.is_empty() {
            metrics.values().map(|m| m.average_latency_ms).sum::<f64>() / metrics.len() as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "overall_health": healthy_sources as f32 / total_sources as f32 * 100.0,
            "healthy_sources": healthy_sources,
            "total_sources": total_sources,
            "circuit_breakers_open": sources_with_circuit_breakers,
            "overall_success_rate_percent": overall_success_rate,
            "overall_average_latency_ms": overall_latency,
            "source_details": health_status,
            "performance_metrics": metrics,
            "last_updated": chrono::Utc::now().timestamp_millis()
        }))
    }

    /// Health check for the data source manager
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        let health_status = self.get_health_status().await;

        // Consider healthy if at least cache is working
        if let Some(cache_health) = health_status.get(&DataSourceType::Cache) {
            Ok(cache_health.is_healthy)
        } else {
            Ok(false)
        }
    }

    /// Get the underlying KvStore for direct access when needed
    pub fn get_kv_store(&self) -> worker::kv::KvStore {
        self.kv_store.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_manager_config_default() {
        let config = DataSourceManagerConfig::default();
        assert!(config.enable_circuit_breakers);
        assert_eq!(config.circuit_breaker_failure_threshold, 5);
        assert_eq!(config.connection_pool_size, 15);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_data_source_manager_config_high_concurrency() {
        let config = DataSourceManagerConfig::high_concurrency();
        assert_eq!(config.connection_pool_size, 25);
        assert_eq!(config.request_timeout_seconds, 15);
        assert_eq!(config.circuit_breaker_failure_threshold, 3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_data_source_type_priority() {
        assert_eq!(DataSourceType::Pipeline.priority(), 1);
        assert_eq!(DataSourceType::Cache.priority(), 2);
        assert_eq!(DataSourceType::Database.priority(), 3);
        assert_eq!(DataSourceType::API.priority(), 4);
    }

    #[test]
    fn test_circuit_breaker_functionality() {
        let mut breaker = CircuitBreaker::new(3, 60000);

        // Initially closed
        assert!(breaker.can_execute());
        assert!(!breaker.is_open());

        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.can_execute());
        assert!(!breaker.is_open());

        // Third failure should open circuit
        breaker.record_failure();
        assert!(!breaker.can_execute());
        assert!(breaker.is_open());
    }

    #[test]
    fn test_data_source_health_default() {
        let health = DataSourceHealth::default();
        assert!(!health.is_healthy);
        assert_eq!(health.success_rate_percent, 0.0);
        assert_eq!(health.total_requests, 0);
        assert!(!health.circuit_breaker_open);
    }

    #[test]
    fn test_data_source_metrics_default() {
        let metrics = DataSourceMetrics::default();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 0);
        assert_eq!(metrics.success_rate_percent, 0.0);
    }
}
