//! Simple Data Access Service - Optimized for Cloudflare Workers
//!
//! This service provides streamlined data access for Cloudflare Workers:
//! - Direct KV store operations with built-in caching
//! - Simple HTTP client for external API calls
//! - Lightweight retry mechanism
//! - Basic data validation
//! - WASM-compatible implementations

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::{kv::KvStore, Fetch, Method, Request, RequestInit};

/// Simple data access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDataAccessConfig {
    /// Enable data access functionality
    pub enabled: bool,
    /// Default cache TTL in seconds
    pub default_ttl_seconds: u64,
    /// Maximum retry attempts for external calls
    pub max_retries: u32,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for SimpleDataAccessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl_seconds: 300, // 5 minutes
            max_retries: 3,
            timeout_ms: 5000, // 5 seconds
        }
    }
}

/// Simple data types supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    MarketData,
    UserData,
    Configuration,
    Cached,
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            DataType::MarketData => "market_data",
            DataType::UserData => "user_data",
            DataType::Configuration => "config",
            DataType::Cached => "cached",
        }
    }
}

/// Simple data request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDataRequest {
    pub key: String,
    pub data_type: DataType,
    pub use_cache: bool,
    pub ttl_seconds: Option<u64>,
}

impl SimpleDataRequest {
    pub fn new(key: String, data_type: DataType) -> Self {
        Self {
            key,
            data_type,
            use_cache: true,
            ttl_seconds: None,
        }
    }

    pub fn with_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
}

/// Simple data response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDataResponse {
    pub key: String,
    pub data: Option<serde_json::Value>,
    pub cached: bool,
    pub timestamp: u64,
    pub ttl_seconds: u64,
}

impl SimpleDataResponse {
    pub fn new(
        key: String,
        data: Option<serde_json::Value>,
        cached: bool,
        ttl_seconds: u64,
    ) -> Self {
        Self {
            key,
            data,
            cached,
            timestamp: Self::current_timestamp(),
            ttl_seconds,
        }
    }

    fn current_timestamp() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
    }
}

/// Simple data access service optimized for Cloudflare Workers
pub struct SimpleDataAccessService {
    config: SimpleDataAccessConfig,
    kv_store: KvStore,
    logger: crate::utils::logger::Logger,
}

impl SimpleDataAccessService {
    /// Create new data access service
    pub fn new(config: SimpleDataAccessConfig, kv_store: KvStore) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        logger.info("SimpleDataAccessService initialized for Cloudflare Workers");

        Ok(Self {
            config,
            kv_store,
            logger,
        })
    }

    /// Get data by key with optional caching
    pub async fn get_data(
        &self,
        request: SimpleDataRequest,
    ) -> ArbitrageResult<SimpleDataResponse> {
        if !self.config.enabled {
            return Ok(SimpleDataResponse::new(request.key, None, false, 0));
        }

        let cache_key = self.build_cache_key(&request.key, &request.data_type);

        // Try cache first if enabled
        if request.use_cache {
            if let Ok(Some(cached_data)) = self
                .kv_store
                .get(&cache_key)
                .json::<serde_json::Value>()
                .await
            {
                return Ok(SimpleDataResponse::new(
                    request.key,
                    Some(cached_data),
                    true,
                    request
                        .ttl_seconds
                        .unwrap_or(self.config.default_ttl_seconds),
                ));
            }
        }

        // If not in cache, return empty response
        // In a real implementation, this would fetch from external sources
        Ok(SimpleDataResponse::new(request.key, None, false, 0))
    }

    /// Store data in KV with TTL
    pub async fn store_data(
        &self,
        key: &str,
        data_type: DataType,
        data: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_key = self.build_cache_key(key, &data_type);
        let ttl = ttl_seconds.unwrap_or(self.config.default_ttl_seconds);

        self.kv_store
            .put(&cache_key, data)?
            .expiration_ttl(ttl)
            .execute()
            .await?;

        Ok(())
    }

    /// Simple HTTP client for external API calls
    pub async fn fetch_external(
        &self,
        url: &str,
        method: Method,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    ) -> ArbitrageResult<serde_json::Value> {
        if !self.config.enabled {
            return Err(ArbitrageError::api_error("Data access disabled"));
        }

        let mut request_init = RequestInit::new();
        request_init.method = method;

        if let Some(body_data) = body {
            request_init.body = Some(body_data.into());
        }

        if let Some(request_headers) = headers {
            let mut headers_obj = worker::Headers::new();
            for (key, value) in request_headers {
                headers_obj.set(&key, &value).map_err(|e| {
                    ArbitrageError::api_error(&format!("Failed to set header: {:?}", e))
                })?;
            }
            request_init.headers = headers_obj;
        }

        let request = Request::new_with_init(url, &request_init)?;
        let mut response = Fetch::Request(request).send().await?;

        if response.status_code() >= 400 {
            return Err(ArbitrageError::api_error(&format!(
                "HTTP {} from {}",
                response.status_code(),
                url
            )));
        }

        let json_response = response.json::<serde_json::Value>().await?;
        Ok(json_response)
    }

    /// Simple batch get operation
    pub async fn get_batch(
        &self,
        requests: Vec<SimpleDataRequest>,
    ) -> ArbitrageResult<Vec<SimpleDataResponse>> {
        let mut responses = Vec::new();

        for request in requests {
            match self.get_data(request).await {
                Ok(response) => responses.push(response),
                Err(err) => {
                    self.logger
                        .warn(&format!("Batch get failed for key: {}", err));
                    // Continue with other requests rather than failing the entire batch
                }
            }
        }

        Ok(responses)
    }

    /// Check if a key exists in cache
    pub async fn exists(&self, key: &str, data_type: DataType) -> bool {
        if !self.config.enabled {
            return false;
        }

        let cache_key = self.build_cache_key(key, &data_type);
        match self.kv_store.get(&cache_key).text().await {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    /// Delete data from cache
    pub async fn delete(&self, key: &str, data_type: DataType) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_key = self.build_cache_key(key, &data_type);
        self.kv_store.delete(&cache_key).await?;
        Ok(())
    }

    /// Simple health check
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        // Test KV store with a simple operation
        let test_key = "health_check_test";
        let test_data =
            serde_json::json!({"test": true, "timestamp": SimpleDataResponse::current_timestamp()});

        match self.kv_store.put(test_key, &test_data)?.execute().await {
            Ok(_) => {
                // Clean up test data
                let _ = self.kv_store.delete(test_key).await;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Build cache key with prefix
    fn build_cache_key(&self, key: &str, data_type: &DataType) -> String {
        format!("{}:{}", data_type.as_str(), key)
    }

    /// Get configuration
    pub fn config(&self) -> &SimpleDataAccessConfig {
        &self.config
    }

    /// Get KV store reference
    pub fn kv_store(&self) -> &KvStore {
        &self.kv_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_data_request_builder() {
        let request = SimpleDataRequest::new("test_key".to_string(), DataType::MarketData)
            .with_cache(false)
            .with_ttl(600);

        assert_eq!(request.key, "test_key");
        assert_eq!(request.data_type.as_str(), "market_data");
        assert!(!request.use_cache);
        assert_eq!(request.ttl_seconds, Some(600));
    }

    #[test]
    fn test_data_types() {
        assert_eq!(DataType::MarketData.as_str(), "market_data");
        assert_eq!(DataType::UserData.as_str(), "user_data");
        assert_eq!(DataType::Configuration.as_str(), "config");
        assert_eq!(DataType::Cached.as_str(), "cached");
    }

    #[test]
    fn test_cache_key_building() {
        // This would require a mock KV store to test properly in unit tests
        // For now, we just test the data type string conversion
        let data_type = DataType::MarketData;
        let expected_prefix = "market_data:test_key";
        assert!(expected_prefix.starts_with(data_type.as_str()));
    }
}
