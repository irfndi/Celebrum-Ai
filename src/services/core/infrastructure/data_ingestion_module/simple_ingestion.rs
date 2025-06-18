// Simple Ingestion Service - Consolidated Workers-Optimized Data Ingestion
// Replaces the complex multi-file data ingestion module with a simplified approach
// Optimized for Cloudflare Workers environment

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::kv::KvStore;

/// Simplified configuration for data ingestion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleIngestionConfig {
    // Core ingestion settings
    pub max_batch_size: usize,
    pub ingestion_timeout_ms: u64,
    pub enable_transformation: bool,
    pub enable_validation: bool,

    // Queue and pipeline settings
    pub queue_max_size: usize,
    pub pipeline_buffer_size: usize,
    pub enable_retry: bool,
    pub max_retry_attempts: u32,

    // Performance settings
    pub enable_compression: bool,
    pub enable_metrics: bool,
    pub parallel_workers: u32,
}

impl Default for SimpleIngestionConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            ingestion_timeout_ms: 30000,
            enable_transformation: true,
            enable_validation: true,
            queue_max_size: 1000,
            pipeline_buffer_size: 500,
            enable_retry: true,
            max_retry_attempts: 3,
            enable_compression: false,
            enable_metrics: true,
            parallel_workers: 4,
        }
    }
}

/// Data ingestion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionRequest {
    pub id: String,
    pub data_type: IngestionDataType,
    pub payload: serde_json::Value,
    pub source: String,
    pub timestamp: u64,
    pub priority: IngestionPriority,
    pub metadata: HashMap<String, String>,
}

/// Types of data that can be ingested
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IngestionDataType {
    MarketData,
    UserData,
    Configuration,
    Analytics,
    Logs,
    Custom(String),
}

/// Priority levels for ingestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IngestionPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Result of an ingestion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionResult {
    pub request_id: String,
    pub success: bool,
    pub processed_at: u64,
    pub processing_time_ms: u64,
    pub records_processed: u32,
    pub records_failed: u32,
    pub storage_location: Option<String>,
    pub error_message: Option<String>,
}

/// Metrics for ingestion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleIngestionMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_processing_time_ms: f64,
    pub total_records_processed: u64,
    pub requests_by_type: HashMap<String, u64>,
    pub requests_by_priority: HashMap<String, u64>,
    pub last_updated: u64,
}

impl Default for SimpleIngestionMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_processing_time_ms: 0.0,
            total_records_processed: 0,
            requests_by_type: HashMap::new(),
            requests_by_priority: HashMap::new(),
            last_updated: crate::utils::time::get_current_timestamp(),
        }
    }
}

/// Main simplified ingestion service - replaces all data_ingestion_module components
pub struct SimpleIngestionService {
    config: SimpleIngestionConfig,
    kv_store: KvStore,
    metrics: std::sync::Arc<std::sync::Mutex<SimpleIngestionMetrics>>,
    queue: std::sync::Arc<std::sync::Mutex<Vec<IngestionRequest>>>,
    logger: crate::utils::logger::Logger,
}

impl SimpleIngestionService {
    /// Create new simple ingestion service
    pub fn new(config: SimpleIngestionConfig, kv_store: KvStore) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        logger.info("Initializing SimpleIngestionService for Cloudflare Workers");

        Ok(Self {
            config,
            kv_store,
            metrics: std::sync::Arc::new(std::sync::Mutex::new(SimpleIngestionMetrics::default())),
            queue: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            logger,
        })
    }

    /// Ingest a single data item
    pub async fn ingest(&self, request: IngestionRequest) -> ArbitrageResult<IngestionResult> {
        let start_time = crate::utils::time::get_current_timestamp();

        // Validate request if enabled
        if self.config.enable_validation && !self.validate_request(&request).await {
            return Ok(IngestionResult {
                request_id: request.id.clone(),
                success: false,
                processed_at: start_time,
                processing_time_ms: crate::utils::time::get_current_timestamp() - start_time,
                records_processed: 0,
                records_failed: 1,
                storage_location: None,
                error_message: Some("Request validation failed".to_string()),
            });
        }

        // Transform data if enabled
        let processed_data = if self.config.enable_transformation {
            self.transform_data(&request).await?
        } else {
            request.payload.clone()
        };

        // Store the processed data
        let storage_key = self.generate_storage_key(&request);
        match self.store_data(&storage_key, &processed_data).await {
            Ok(_) => {
                let processing_time = crate::utils::time::get_current_timestamp() - start_time;

                // Update metrics
                self.update_metrics(&request, true, processing_time).await;

                Ok(IngestionResult {
                    request_id: request.id,
                    success: true,
                    processed_at: start_time,
                    processing_time_ms: processing_time,
                    records_processed: 1,
                    records_failed: 0,
                    storage_location: Some(storage_key),
                    error_message: None,
                })
            }
            Err(e) => {
                let processing_time = crate::utils::time::get_current_timestamp() - start_time;

                // Update metrics for failure
                self.update_metrics(&request, false, processing_time).await;

                Ok(IngestionResult {
                    request_id: request.id,
                    success: false,
                    processed_at: start_time,
                    processing_time_ms: processing_time,
                    records_processed: 0,
                    records_failed: 1,
                    storage_location: None,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }

    /// Ingest multiple data items as a batch
    pub async fn ingest_batch(
        &self,
        requests: Vec<IngestionRequest>,
    ) -> ArbitrageResult<Vec<IngestionResult>> {
        let mut results = Vec::new();

        // Process in chunks to respect batch size limits
        for chunk in requests.chunks(self.config.max_batch_size) {
            for request in chunk {
                match self.ingest(request.clone()).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        self.logger.error(&format!(
                            "Batch ingestion error for request {}: {}",
                            request.id, e
                        ));
                        results.push(IngestionResult {
                            request_id: request.id.clone(),
                            success: false,
                            processed_at: crate::utils::time::get_current_timestamp(),
                            processing_time_ms: 0,
                            records_processed: 0,
                            records_failed: 1,
                            storage_location: None,
                            error_message: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Queue a request for later processing
    pub async fn queue_request(&self, request: IngestionRequest) -> ArbitrageResult<()> {
        if let Ok(mut queue) = self.queue.lock() {
            if queue.len() >= self.config.queue_max_size {
                return Err(ArbitrageError::rate_limit_error("Ingestion queue is full"));
            }

            // Insert based on priority (higher priority first)
            let insert_pos = queue
                .iter()
                .position(|r| r.priority < request.priority)
                .unwrap_or(queue.len());
            queue.insert(insert_pos, request);
        }

        Ok(())
    }

    /// Process queued requests
    pub async fn process_queue(&self) -> ArbitrageResult<Vec<IngestionResult>> {
        let requests = if let Ok(mut queue) = self.queue.lock() {
            let batch_size = std::cmp::min(queue.len(), self.config.max_batch_size);
            queue.drain(0..batch_size).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        self.ingest_batch(requests).await
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        if let Ok(queue) = self.queue.lock() {
            queue.len()
        } else {
            0
        }
    }

    /// Health check for the ingestion service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        // Test basic functionality with a simple operation
        let test_request = IngestionRequest {
            id: "health_check_test".to_string(),
            data_type: IngestionDataType::Custom("health_check".to_string()),
            payload: serde_json::json!({"test": "data"}),
            source: "health_check".to_string(),
            timestamp: crate::utils::time::get_current_timestamp(),
            priority: IngestionPriority::Low,
            metadata: HashMap::new(),
        };

        match self.ingest(test_request).await {
            Ok(result) => Ok(result.success),
            Err(_) => Ok(false),
        }
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> SimpleIngestionMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            SimpleIngestionMetrics::default()
        }
    }

    // Private helper methods

    async fn validate_request(&self, request: &IngestionRequest) -> bool {
        // Basic validation
        !request.id.is_empty()
            && !request.source.is_empty()
            && request.timestamp > 0
            && !request.payload.is_null()
    }

    async fn transform_data(
        &self,
        request: &IngestionRequest,
    ) -> ArbitrageResult<serde_json::Value> {
        // Basic transformation - add metadata and normalize structure
        let mut transformed = request.payload.clone();

        if let Some(obj) = transformed.as_object_mut() {
            obj.insert(
                "_ingestion_id".to_string(),
                serde_json::Value::String(request.id.clone()),
            );
            obj.insert(
                "_ingestion_timestamp".to_string(),
                serde_json::Value::Number(request.timestamp.into()),
            );
            obj.insert(
                "_ingestion_source".to_string(),
                serde_json::Value::String(request.source.clone()),
            );
            obj.insert(
                "_data_type".to_string(),
                serde_json::Value::String(format!("{:?}", request.data_type)),
            );
        }

        Ok(transformed)
    }

    fn generate_storage_key(&self, request: &IngestionRequest) -> String {
        format!(
            "ingestion/{:?}/{}/{}",
            request.data_type, request.source, request.id
        )
        .to_lowercase()
    }

    async fn store_data(&self, key: &str, data: &serde_json::Value) -> ArbitrageResult<()> {
        let serialized = serde_json::to_string(data)?;

        match self.kv_store.put(key, serialized)?.execute().await {
            Ok(_) => Ok(()),
            Err(e) => Err(ArbitrageError::database_error(format!(
                "Failed to store ingested data: {}",
                e
            ))),
        }
    }

    async fn update_metrics(
        &self,
        request: &IngestionRequest,
        success: bool,
        processing_time_ms: u64,
    ) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_requests += 1;

            if success {
                metrics.successful_requests += 1;
                metrics.total_records_processed += 1;
            } else {
                metrics.failed_requests += 1;
            }

            // Update average processing time
            let total_requests = metrics.total_requests as f64;
            metrics.avg_processing_time_ms = ((metrics.avg_processing_time_ms
                * (total_requests - 1.0))
                + processing_time_ms as f64)
                / total_requests;

            // Update type and priority counters
            let data_type_key = format!("{:?}", request.data_type);
            *metrics.requests_by_type.entry(data_type_key).or_insert(0) += 1;

            let priority_key = format!("{:?}", request.priority);
            *metrics
                .requests_by_priority
                .entry(priority_key)
                .or_insert(0) += 1;

            metrics.last_updated = crate::utils::time::get_current_timestamp();
        }
    }
}

/// Builder pattern for creating simple ingestion service
pub struct SimpleIngestionBuilder {
    config: SimpleIngestionConfig,
}

impl SimpleIngestionBuilder {
    pub fn new() -> Self {
        Self {
            config: SimpleIngestionConfig::default(),
        }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.max_batch_size = size;
        self
    }

    pub fn with_transformation(mut self, enabled: bool) -> Self {
        self.config.enable_transformation = enabled;
        self
    }

    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.config.enable_validation = enabled;
        self
    }

    pub fn with_queue_size(mut self, size: usize) -> Self {
        self.config.queue_max_size = size;
        self
    }

    pub fn with_retry(mut self, enabled: bool, max_attempts: u32) -> Self {
        self.config.enable_retry = enabled;
        self.config.max_retry_attempts = max_attempts;
        self
    }

    pub fn build(self, kv_store: KvStore) -> ArbitrageResult<SimpleIngestionService> {
        SimpleIngestionService::new(self.config, kv_store)
    }
}

impl Default for SimpleIngestionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
