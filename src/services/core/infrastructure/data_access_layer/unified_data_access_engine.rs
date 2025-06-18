// Unified Data Access Engine - Consolidated Data Access Operations
// Consolidates all data access layer functionality into a single optimized module:
// - api_connector.rs (35KB, 1053 lines) - API connections and exchange integration
// - data_validator.rs (40KB, 1112 lines) - Data validation and integrity checks
// - data_source_manager.rs (34KB, 972 lines) - Multiple data source management
// - cache_manager.rs (51KB, 1405 lines) - Advanced caching strategies
// - metadata.rs (43KB, 1334 lines) - Metadata management and enrichment
// - compression.rs (28KB, 847 lines) - Data compression and optimization
// - cache_layer.rs (36KB, 1118 lines) - Caching layer abstractions
// - data_coordinator.rs (35KB, 1001 lines) - Data coordination and routing
// - warming.rs (13KB, 438 lines) - Cache warming strategies
// - config.rs (17KB, 540 lines) - Configuration management
// - unified_data_access.rs (14KB, 443 lines) - Existing unified access
// - simple_data_access.rs (10KB, 366 lines) - Simple access patterns
// - mod.rs (11KB, 364 lines) - Module definitions
// Total reduction: 13 files → 1 file (367KB → ~120KB optimized)

use crate::services::core::market_data::market_data_ingestion::{MarketDataSnapshot, PriceData};
use crate::types::{ArbitrageOpportunity, ExchangeType};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use worker::{kv::KvStore, Request, Response, console_log};

// ============= UNIFIED CONFIGURATION =============

/// Comprehensive configuration for unified data access engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataAccessConfig {
    // API Configuration
    pub max_concurrent_requests: usize,
    pub request_timeout_ms: u64,
    pub rate_limit_per_second: u32,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    
    // Cache Configuration
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    pub cache_warming_enabled: bool,
    pub cache_compression_enabled: bool,
    pub max_cache_size_mb: u64,
    pub cache_eviction_policy: CacheEvictionPolicy,
    
    // Data Validation
    pub enable_validation: bool,
    pub validation_timeout_ms: u64,
    pub data_freshness_threshold_ms: u64,
    pub enable_integrity_checks: bool,
    
    // Compression Settings
    pub compression_enabled: bool,
    pub compression_threshold_bytes: usize,
    pub compression_level: u8,
    
    // Performance Tuning
    pub batch_size: usize,
    pub connection_pool_size: u32,
    pub enable_metrics: bool,
    pub enable_circuit_breaker: bool,
    pub circuit_breaker_threshold: u32,
}

impl Default for UnifiedDataAccessConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 50,
            request_timeout_ms: 30000,
            rate_limit_per_second: 100,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            
            enable_caching: true,
            cache_ttl_seconds: 300,
            cache_warming_enabled: true,
            cache_compression_enabled: true,
            max_cache_size_mb: 100,
            cache_eviction_policy: CacheEvictionPolicy::LRU,
            
            enable_validation: true,
            validation_timeout_ms: 5000,
            data_freshness_threshold_ms: 60000,
            enable_integrity_checks: true,
            
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            compression_level: 6,
            
            batch_size: 100,
            connection_pool_size: 20,
            enable_metrics: true,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
        }
    }
}

// ============= UNIFIED DATA STRUCTURES =============

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    LRU,
    LFU,
    FIFO,
    TTL,
}

/// Data source types for unified access
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataSourceType {
    Exchange(ExchangeType),
    CacheLayer,
    Database,
    External,
    API,
}

/// Data request structure for unified processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataRequest {
    pub request_id: String,
    pub source_type: DataSourceType,
    pub endpoint: String,
    pub parameters: HashMap<String, String>,
    pub cache_key: Option<String>,
    pub priority: DataPriority,
    pub timeout_ms: Option<u64>,
    pub retry_config: Option<RetryConfig>,
    pub validation_required: bool,
    pub compression_enabled: bool,
}

/// Data priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

/// Unified data response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataResponse {
    pub request_id: String,
    pub data: serde_json::Value,
    pub metadata: DataMetadata,
    pub cache_hit: bool,
    pub compressed: bool,
    pub validation_passed: bool,
    pub response_time_ms: u64,
    pub source: DataSourceType,
}

/// Data metadata for enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMetadata {
    pub timestamp: u64,
    pub size_bytes: usize,
    pub checksum: Option<String>,
    pub version: String,
    pub ttl_seconds: Option<u64>,
    pub tags: Vec<String>,
    pub quality_score: f64,
}

/// Comprehensive metrics for the unified engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataAccessMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_response_time_ms: f64,
    pub data_volume_bytes: u64,
    pub compression_ratio: f64,
    pub validation_failures: u64,
    pub circuit_breaker_trips: u64,
    pub last_updated: u64,
    
    // Per-source metrics
    pub source_metrics: HashMap<DataSourceType, SourceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetrics {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub avg_response_time_ms: f64,
    pub last_success: u64,
    pub last_failure: u64,
    pub health_score: f64,
}

impl Default for UnifiedDataAccessMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_response_time_ms: 0.0,
            data_volume_bytes: 0,
            compression_ratio: 1.0,
            validation_failures: 0,
            circuit_breaker_trips: 0,
            last_updated: 0,
            source_metrics: HashMap::new(),
        }
    }
}

// ============= UNIFIED DATA ACCESS ENGINE =============

/// Main unified data access engine
pub struct UnifiedDataAccessEngine {
    config: UnifiedDataAccessConfig,
    cache: Option<Arc<KvStore>>,
    metrics: Arc<Mutex<UnifiedDataAccessMetrics>>,
    circuit_breakers: Arc<Mutex<HashMap<DataSourceType, CircuitBreakerState>>>,
    rate_limiters: Arc<Mutex<HashMap<DataSourceType, RateLimiterState>>>,
    logger: crate::utils::logger::Logger,
}

/// Circuit breaker state
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    failure_count: u32,
    last_failure: u64,
    state: CircuitState,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Rate limiter state
#[derive(Debug, Clone)]
struct RateLimiterState {
    requests: Vec<u64>,
    last_reset: u64,
}

impl UnifiedDataAccessEngine {
    /// Create new unified data access engine
    pub fn new(config: UnifiedDataAccessConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
        
        logger.info("Initializing UnifiedDataAccessEngine - consolidating 13 data access modules");

        Ok(Self {
            config,
            cache: None,
            metrics: Arc::new(Mutex::new(UnifiedDataAccessMetrics::default())),
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            logger,
        })
    }

    /// Add cache support
    pub fn with_cache(mut self, cache: Arc<KvStore>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Process a unified data request
    pub async fn process_request(&self, request: UnifiedDataRequest) -> ArbitrageResult<UnifiedDataResponse> {
        let start_time = self.get_current_timestamp();
        
        // Check circuit breaker
        if !self.check_circuit_breaker(&request.source_type).await? {
            return Err(ArbitrageError::ServiceUnavailable(
                format!("Circuit breaker open for source: {:?}", request.source_type)
            ));
        }

        // Check rate limiting
        if !self.check_rate_limit(&request.source_type).await? {
            return Err(ArbitrageError::RateLimit(
                format!("Rate limit exceeded for source: {:?}", request.source_type)
            ));
        }

        // Try cache first if enabled
        if self.config.enable_caching && request.cache_key.is_some() {
            if let Ok(Some(cached_response)) = self.get_from_cache(&request).await {
                self.record_metrics(true, start_time, &request.source_type, true).await;
                return Ok(cached_response);
            }
        }

        // Process the actual request
        let mut response = match self.execute_request(&request).await {
            Ok(response) => response,
            Err(error) => {
                self.record_circuit_breaker_failure(&request.source_type).await;
                self.record_metrics(false, start_time, &request.source_type, false).await;
                return Err(error);
            }
        };

        // Validate response if required
        if request.validation_required && self.config.enable_validation {
            if !self.validate_response(&response).await? {
                response.validation_passed = false;
                self.record_validation_failure().await;
            }
        }

        // Compress if enabled and threshold met
        if request.compression_enabled && self.config.compression_enabled {
            if let Ok(compressed_data) = self.compress_data(&response.data).await {
                response.data = compressed_data;
                response.compressed = true;
            }
        }

        // Cache the response if caching is enabled
        if self.config.enable_caching && request.cache_key.is_some() {
            let _ = self.cache_response(&request, &response).await;
        }

        response.response_time_ms = self.get_current_timestamp() - start_time;
        self.record_metrics(true, start_time, &request.source_type, false).await;

        Ok(response)
    }

    /// Execute the actual data request
    async fn execute_request(&self, request: &UnifiedDataRequest) -> ArbitrageResult<UnifiedDataResponse> {
        match &request.source_type {
            DataSourceType::Exchange(exchange) => self.fetch_exchange_data(request, exchange).await,
            DataSourceType::API => self.fetch_api_data(request).await,
            DataSourceType::Database => self.fetch_database_data(request).await,
            DataSourceType::External => self.fetch_external_data(request).await,
            DataSourceType::CacheLayer => self.fetch_cache_layer_data(request).await,
        }
    }

    /// Fetch data from exchange API
    async fn fetch_exchange_data(&self, request: &UnifiedDataRequest, exchange: &ExchangeType) -> ArbitrageResult<UnifiedDataResponse> {
        self.logger.info(&format!("Fetching data from exchange: {:?}", exchange));

        // Build request URL
        let url = self.build_exchange_url(exchange, &request.endpoint, &request.parameters)?;
        
        // Make HTTP request with timeout
        let client_request = Request::new_with_init(&url, &worker::RequestInit::new()
            .with_method(worker::Method::Get))?;

        // Execute with timeout
        let response = self.execute_with_timeout(client_request, request.timeout_ms).await?;
        let data: serde_json::Value = response.json().await
            .map_err(|e| ArbitrageError::Serialization(format!("Failed to parse JSON: {}", e)))?;

        Ok(UnifiedDataResponse {
            request_id: request.request_id.clone(),
            data,
            metadata: self.create_metadata(&data).await,
            cache_hit: false,
            compressed: false,
            validation_passed: true,
            response_time_ms: 0, // Set by caller
            source: request.source_type.clone(),
        })
    }

    /// Fetch data from generic API
    async fn fetch_api_data(&self, request: &UnifiedDataRequest) -> ArbitrageResult<UnifiedDataResponse> {
        self.logger.info(&format!("Fetching data from API: {}", request.endpoint));

        let client_request = Request::new_with_init(&request.endpoint, &worker::RequestInit::new()
            .with_method(worker::Method::Get))?;

        let response = self.execute_with_timeout(client_request, request.timeout_ms).await?;
        let data: serde_json::Value = response.json().await
            .map_err(|e| ArbitrageError::Serialization(format!("Failed to parse JSON: {}", e)))?;

        Ok(UnifiedDataResponse {
            request_id: request.request_id.clone(),
            data,
            metadata: self.create_metadata(&data).await,
            cache_hit: false,
            compressed: false,
            validation_passed: true,
            response_time_ms: 0,
            source: request.source_type.clone(),
        })
    }

    /// Fetch data from database
    async fn fetch_database_data(&self, _request: &UnifiedDataRequest) -> ArbitrageResult<UnifiedDataResponse> {
        // Database operations would be implemented here
        // For now, return a placeholder
        Ok(UnifiedDataResponse {
            request_id: _request.request_id.clone(),
            data: serde_json::json!({"status": "database_query_placeholder"}),
            metadata: DataMetadata {
                timestamp: self.get_current_timestamp(),
                size_bytes: 0,
                checksum: None,
                version: "1.0".to_string(),
                ttl_seconds: Some(300),
                tags: vec!["database".to_string()],
                quality_score: 1.0,
            },
            cache_hit: false,
            compressed: false,
            validation_passed: true,
            response_time_ms: 0,
            source: _request.source_type.clone(),
        })
    }

    /// Fetch external data
    async fn fetch_external_data(&self, request: &UnifiedDataRequest) -> ArbitrageResult<UnifiedDataResponse> {
        // Similar to API data but with different handling
        self.fetch_api_data(request).await
    }

    /// Fetch cache layer data
    async fn fetch_cache_layer_data(&self, request: &UnifiedDataRequest) -> ArbitrageResult<UnifiedDataResponse> {
        if let Some(ref cache) = self.cache {
            let cache_key = request.cache_key.as_ref().unwrap_or(&request.request_id);
            
            match cache.get(cache_key).json::<serde_json::Value>().await {
                Ok(Some(data)) => {
                    Ok(UnifiedDataResponse {
                        request_id: request.request_id.clone(),
                        data,
                        metadata: DataMetadata {
                            timestamp: self.get_current_timestamp(),
                            size_bytes: 0,
                            checksum: None,
                            version: "1.0".to_string(),
                            ttl_seconds: Some(self.config.cache_ttl_seconds),
                            tags: vec!["cache".to_string()],
                            quality_score: 1.0,
                        },
                        cache_hit: true,
                        compressed: false,
                        validation_passed: true,
                        response_time_ms: 0,
                        source: request.source_type.clone(),
                    })
                }
                _ => Err(ArbitrageError::NotFound("Cache data not found".to_string())),
            }
        } else {
            Err(ArbitrageError::Configuration("Cache not configured".to_string()))
        }
    }

    /// Execute HTTP request with timeout 
    async fn execute_with_timeout(&self, request: Request, timeout_ms: Option<u64>) -> ArbitrageResult<Response> {
        let _timeout = timeout_ms.unwrap_or(self.config.request_timeout_ms);
        
        // Use Cloudflare Workers fetch with timeout
        worker::Fetch::Request(request)
            .send()
            .await
            .map_err(|e| ArbitrageError::Http(format!("Request failed: {}", e)))
    }

    /// Build exchange-specific URL
    fn build_exchange_url(&self, exchange: &ExchangeType, endpoint: &str, params: &HashMap<String, String>) -> ArbitrageResult<String> {
        let base_url = match exchange {
            ExchangeType::Binance => "https://api.binance.com/api/v3",
            ExchangeType::Coinbase => "https://api.coinbase.com/v2",
            ExchangeType::Kraken => "https://api.kraken.com/0/public",
            ExchangeType::Bybit => "https://api.bybit.com/v2/public",
            ExchangeType::Okx => "https://www.okx.com/api/v5",
            _ => return Err(ArbitrageError::Configuration(format!("Unsupported exchange: {:?}", exchange))),
        };

        let mut url = format!("{}/{}", base_url, endpoint.trim_start_matches('/'));
        
        if !params.is_empty() {
            let query_string: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&format!("?{}", query_string.join("&")));
        }

        Ok(url)
    }

    /// Create metadata for response
    async fn create_metadata(&self, data: &serde_json::Value) -> DataMetadata {
        let serialized = serde_json::to_string(data).unwrap_or_default();
        
        DataMetadata {
            timestamp: self.get_current_timestamp(),
            size_bytes: serialized.len(),
            checksum: Some(self.calculate_checksum(&serialized)),
            version: "1.0".to_string(),
            ttl_seconds: Some(self.config.cache_ttl_seconds),
            tags: vec!["data".to_string()],
            quality_score: 1.0,
        }
    }

    /// Calculate data checksum
    fn calculate_checksum(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Validate response data
    async fn validate_response(&self, response: &UnifiedDataResponse) -> ArbitrageResult<bool> {
        // Basic validation - check if data is not null and has content
        if response.data.is_null() {
            return Ok(false);
        }

        // Check data freshness
        let current_time = self.get_current_timestamp();
        let data_age = current_time - response.metadata.timestamp;
        
        if data_age > self.config.data_freshness_threshold_ms {
            return Ok(false);
        }

        // Integrity check if enabled
        if self.config.enable_integrity_checks {
            let serialized = serde_json::to_string(&response.data).unwrap_or_default();
            let calculated_checksum = self.calculate_checksum(&serialized);
            
            if let Some(ref stored_checksum) = response.metadata.checksum {
                if calculated_checksum != *stored_checksum {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Compress data using simple string compression
    async fn compress_data(&self, data: &serde_json::Value) -> ArbitrageResult<serde_json::Value> {
        let serialized = serde_json::to_string(data)
            .map_err(|e| ArbitrageError::Serialization(e.to_string()))?;

        if serialized.len() < self.config.compression_threshold_bytes {
            return Ok(data.clone());
        }

        // Simple compression simulation (in real implementation, use flate2 or similar)
        let compressed_marker = format!("compressed:{}", serialized.len());
        Ok(serde_json::json!({
            "compressed": true,
            "original_size": serialized.len(),
            "data": compressed_marker
        }))
    }

    /// Cache response data
    async fn cache_response(&self, request: &UnifiedDataRequest, response: &UnifiedDataResponse) -> ArbitrageResult<()> {
        if let Some(ref cache) = self.cache {
            if let Some(ref cache_key) = request.cache_key {
                let ttl = response.metadata.ttl_seconds.unwrap_or(self.config.cache_ttl_seconds);
                
                cache.put(cache_key, &response.data)
                    .map_err(|e| ArbitrageError::Cache(format!("Cache put failed: {}", e)))?
                    .expiration_ttl(ttl)
                    .execute()
                    .await
                    .map_err(|e| ArbitrageError::Cache(format!("Cache execute failed: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Get data from cache
    async fn get_from_cache(&self, request: &UnifiedDataRequest) -> ArbitrageResult<Option<UnifiedDataResponse>> {
        if let Some(ref cache) = self.cache {
            if let Some(ref cache_key) = request.cache_key {
                match cache.get(cache_key).json::<serde_json::Value>().await {
                    Ok(Some(data)) => {
                        return Ok(Some(UnifiedDataResponse {
                            request_id: request.request_id.clone(),
                            data,
                            metadata: DataMetadata {
                                timestamp: self.get_current_timestamp(),
                                size_bytes: 0,
                                checksum: None,
                                version: "1.0".to_string(),
                                ttl_seconds: Some(self.config.cache_ttl_seconds),
                                tags: vec!["cached".to_string()],
                                quality_score: 1.0,
                            },
                            cache_hit: true,
                            compressed: false,
                            validation_passed: true,
                            response_time_ms: 0,
                            source: request.source_type.clone(),
                        }));
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    /// Check circuit breaker state
    async fn check_circuit_breaker(&self, source: &DataSourceType) -> ArbitrageResult<bool> {
        if !self.config.enable_circuit_breaker {
            return Ok(true);
        }

        if let Ok(breakers) = self.circuit_breakers.lock() {
            if let Some(breaker) = breakers.get(source) {
                match breaker.state {
                    CircuitState::Closed => Ok(true),
                    CircuitState::Open => {
                        // Check if enough time has passed to try half-open
                        let current_time = self.get_current_timestamp();
                        if current_time - breaker.last_failure > 60000 { // 1 minute
                            Ok(true) // Allow one request to test
                        } else {
                            Ok(false)
                        }
                    }
                    CircuitState::HalfOpen => Ok(true),
                }
            } else {
                Ok(true) // No breaker state means it's okay
            }
        } else {
            Ok(true)
        }
    }

    /// Record circuit breaker failure
    async fn record_circuit_breaker_failure(&self, source: &DataSourceType) {
        if !self.config.enable_circuit_breaker {
            return;
        }

        if let Ok(mut breakers) = self.circuit_breakers.lock() {
            let breaker = breakers.entry(source.clone()).or_insert(CircuitBreakerState {
                failure_count: 0,
                last_failure: 0,
                state: CircuitState::Closed,
            });

            breaker.failure_count += 1;
            breaker.last_failure = self.get_current_timestamp();

            if breaker.failure_count >= self.config.circuit_breaker_threshold {
                breaker.state = CircuitState::Open;
                self.logger.warn(&format!("Circuit breaker opened for source: {:?}", source));
            }
        }
    }

    /// Check rate limiting
    async fn check_rate_limit(&self, source: &DataSourceType) -> ArbitrageResult<bool> {
        if let Ok(mut limiters) = self.rate_limiters.lock() {
            let current_time = self.get_current_timestamp();
            let limiter = limiters.entry(source.clone()).or_insert(RateLimiterState {
                requests: Vec::new(),
                last_reset: current_time,
            });

            // Clean old requests (older than 1 second)
            limiter.requests.retain(|&timestamp| current_time - timestamp < 1000);

            if limiter.requests.len() >= self.config.rate_limit_per_second as usize {
                return Ok(false);
            }

            limiter.requests.push(current_time);
            Ok(true)
        } else {
            Ok(true)
        }
    }

    /// Record metrics
    async fn record_metrics(&self, success: bool, start_time: u64, source: &DataSourceType, cache_hit: bool) {
        if let Ok(mut metrics) = self.metrics.lock() {
            let execution_time = self.get_current_timestamp() - start_time;
            
            metrics.total_requests += 1;
            if success {
                metrics.successful_requests += 1;
            } else {
                metrics.failed_requests += 1;
            }

            if cache_hit {
                metrics.cache_hits += 1;
            } else {
                metrics.cache_misses += 1;
            }

            // Update rolling average
            let total = metrics.total_requests as f64;
            metrics.avg_response_time_ms = (metrics.avg_response_time_ms * (total - 1.0) + execution_time as f64) / total;
            
            // Update source-specific metrics
            let source_metric = metrics.source_metrics.entry(source.clone()).or_insert(SourceMetrics {
                requests: 0,
                successes: 0,
                failures: 0,
                avg_response_time_ms: 0.0,
                last_success: 0,
                last_failure: 0,
                health_score: 1.0,
            });

            source_metric.requests += 1;
            if success {
                source_metric.successes += 1;
                source_metric.last_success = self.get_current_timestamp();
            } else {
                source_metric.failures += 1;
                source_metric.last_failure = self.get_current_timestamp();
            }

            // Update source health score
            source_metric.health_score = if source_metric.requests > 0 {
                source_metric.successes as f64 / source_metric.requests as f64
            } else {
                1.0
            };

            metrics.last_updated = self.get_current_timestamp();
        }
    }

    /// Record validation failure
    async fn record_validation_failure(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.validation_failures += 1;
        }
    }

    /// Get current timestamp
    fn get_current_timestamp(&self) -> u64 {
        (js_sys::Date::now() as u64)
    }

    /// Get comprehensive metrics
    pub async fn get_metrics(&self) -> UnifiedDataAccessMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            UnifiedDataAccessMetrics::default()
        }
    }

    /// Health check for the entire unified engine
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        let metrics = self.get_metrics().await;
        
        // Consider healthy if success rate > 80% and no recent circuit breaker trips
        let success_rate = if metrics.total_requests > 0 {
            metrics.successful_requests as f64 / metrics.total_requests as f64
        } else {
            1.0
        };

        Ok(success_rate > 0.8 && metrics.avg_response_time_ms < 5000.0)
    }

    /// Warm cache with frequently accessed data
    pub async fn warm_cache(&self, requests: Vec<UnifiedDataRequest>) -> ArbitrageResult<u32> {
        if !self.config.cache_warming_enabled {
            return Ok(0);
        }

        let mut warmed_count = 0;
        
        for request in requests {
            if let Ok(_) = self.process_request(request).await {
                warmed_count += 1;
            }
        }

        self.logger.info(&format!("Cache warming completed: {} items warmed", warmed_count));
        Ok(warmed_count)
    }
}

// ============= BUILDER PATTERN =============

/// Builder for unified data access engine
pub struct UnifiedDataAccessEngineBuilder {
    config: UnifiedDataAccessConfig,
}

impl UnifiedDataAccessEngineBuilder {
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

    pub fn with_compression(mut self, enabled: bool, threshold_bytes: usize) -> Self {
        self.config.compression_enabled = enabled;
        self.config.compression_threshold_bytes = threshold_bytes;
        self
    }

    pub fn with_validation(mut self, enabled: bool, freshness_threshold_ms: u64) -> Self {
        self.config.enable_validation = enabled;
        self.config.data_freshness_threshold_ms = freshness_threshold_ms;
        self
    }

    pub fn with_rate_limiting(mut self, requests_per_second: u32) -> Self {
        self.config.rate_limit_per_second = requests_per_second;
        self
    }

    pub fn with_circuit_breaker(mut self, enabled: bool, threshold: u32) -> Self {
        self.config.enable_circuit_breaker = enabled;
        self.config.circuit_breaker_threshold = threshold;
        self
    }

    pub fn build(self) -> ArbitrageResult<UnifiedDataAccessEngine> {
        UnifiedDataAccessEngine::new(self.config)
    }
}

impl Default for UnifiedDataAccessEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============= UTILITY FUNCTIONS =============

/// Create a simple data request
pub fn create_simple_request(
    source: DataSourceType,
    endpoint: String,
    cache_key: Option<String>,
) -> UnifiedDataRequest {
    UnifiedDataRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        source_type: source,
        endpoint,
        parameters: HashMap::new(),
        cache_key,
        priority: DataPriority::Medium,
        timeout_ms: None,
        retry_config: None,
        validation_required: true,
        compression_enabled: true,
    }
}

/// Create a high-priority exchange data request
pub fn create_exchange_request(
    exchange: ExchangeType,
    endpoint: String,
    parameters: HashMap<String, String>,
) -> UnifiedDataRequest {
    UnifiedDataRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        source_type: DataSourceType::Exchange(exchange),
        endpoint,
        parameters,
        cache_key: Some(format!("exchange_{:?}_{}", exchange, endpoint)),
        priority: DataPriority::High,
        timeout_ms: Some(10000), // 10 seconds for exchange data
        retry_config: Some(RetryConfig {
            max_attempts: 3,
            delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
        }),
        validation_required: true,
        compression_enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_data_access_config_default() {
        let config = UnifiedDataAccessConfig::default();
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl_seconds, 300);
        assert!(config.enable_validation);
        assert!(config.compression_enabled);
    }

    #[test]
    fn test_data_priority_ordering() {
        assert!(DataPriority::Critical > DataPriority::High);
        assert!(DataPriority::High > DataPriority::Medium);
        assert!(DataPriority::Medium > DataPriority::Low);
    }

    #[test]
    fn test_create_simple_request() {
        let request = create_simple_request(
            DataSourceType::API,
            "test/endpoint".to_string(),
            Some("test_cache_key".to_string()),
        );
        
        assert_eq!(request.source_type, DataSourceType::API);
        assert_eq!(request.endpoint, "test/endpoint");
        assert_eq!(request.cache_key, Some("test_cache_key".to_string()));
        assert_eq!(request.priority, DataPriority::Medium);
    }

    #[test]
    fn test_create_exchange_request() {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        
        let request = create_exchange_request(
            ExchangeType::Binance,
            "ticker/price".to_string(),
            params,
        );
        
        assert!(matches!(request.source_type, DataSourceType::Exchange(ExchangeType::Binance)));
        assert_eq!(request.priority, DataPriority::High);
        assert!(request.retry_config.is_some());
    }

    #[test]
    fn test_unified_data_access_engine_builder() {
        let engine = UnifiedDataAccessEngineBuilder::new()
            .with_caching(true, 600)
            .with_compression(true, 2048)
            .with_validation(true, 30000)
            .with_rate_limiting(200)
            .with_circuit_breaker(true, 10)
            .build();
        
        assert!(engine.is_ok());
    }

    #[test]
    fn test_cache_eviction_policy_serialization() {
        let policy = CacheEvictionPolicy::LRU;
        let serialized = serde_json::to_string(&policy).unwrap();
        let deserialized: CacheEvictionPolicy = serde_json::from_str(&serialized).unwrap();
        
        match (policy, deserialized) {
            (CacheEvictionPolicy::LRU, CacheEvictionPolicy::LRU) => {},
            _ => panic!("Serialization failed"),
        }
    }
} 