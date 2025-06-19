// src/services/core/infrastructure/unified_cloudflare_services.rs

//! Unified Cloudflare Services Module
//!
//! This module consolidates all Cloudflare-specific services including:
//! - D1 Database operations
//! - KV Store operations  
//! - R2 Object Storage operations
//! - Cloudflare Pipelines
//! - Health monitoring
//!
//! Designed for maximum efficiency, zero duplication, and high concurrency

use crate::utils::error::{ArbitrageError, ArbitrageResult, ErrorKind};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worker::wasm_bindgen;
use worker::{kv::KvStore, D1Database, Env};

// ============= UNIFIED CONFIGURATION =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCloudflareConfig {
    pub d1: D1Config,
    pub kv: KvConfig,
    pub r2: R2Config,
    pub pipelines: PipelinesConfig,
    pub health: HealthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1Config {
    pub database_name: String,
    pub connection_timeout_ms: u64,
    pub query_timeout_ms: u64,
    pub max_batch_size: usize,
    pub enable_performance_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvConfig {
    pub namespace: String,
    pub default_ttl_seconds: u64,
    pub max_key_size_bytes: usize,
    pub max_value_size_bytes: usize,
    pub compression_enabled: bool,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Config {
    pub bucket_name: String,
    pub max_object_size_bytes: usize,
    pub default_storage_class: String,
    pub enable_multipart_upload: bool,
    pub multipart_threshold_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelinesConfig {
    pub max_concurrent_pipelines: u32,
    pub pipeline_timeout_ms: u64,
    pub batch_processing_size: usize,
    pub enable_priority_queuing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub check_interval_ms: u64,
    pub timeout_ms: u64,
    pub enable_detailed_monitoring: bool,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub error_rate_percent: f64,
    pub response_time_ms: u64,
    pub availability_percent: f64,
}

// ============= SERVICE METRICS =============

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudflareMetrics {
    pub d1_metrics: D1Metrics,
    pub kv_metrics: KvMetrics,
    pub r2_metrics: R2Metrics,
    pub pipeline_metrics: PipelineMetrics,
    pub health_metrics: HealthMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct D1Metrics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_query_time_ms: f64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KvMetrics {
    pub total_operations: u64,
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_response_time_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct R2Metrics {
    pub total_operations: u64,
    pub uploads: u64,
    pub downloads: u64,
    pub deletes: u64,
    pub total_bytes_transferred: u64,
    pub average_transfer_time_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineMetrics {
    pub active_pipelines: u32,
    pub completed_pipelines: u64,
    pub failed_pipelines: u64,
    pub average_processing_time_ms: f64,
    pub total_items_processed: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub uptime_percent: f64,
    pub error_rate_percent: f64,
    pub average_response_time_ms: f64,
    pub last_health_check: Option<u64>,
    pub consecutive_failures: u32,
}

// ============= SERVICE STATUS =============

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ServiceStatus {
    #[default]
    Healthy,
    Degraded,
    Unhealthy,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub d1_status: ServiceStatus,
    pub kv_status: ServiceStatus,
    pub r2_status: ServiceStatus,
    pub pipeline_status: ServiceStatus,
    pub overall_status: ServiceStatus,
}

// ============= MAIN UNIFIED SERVICE =============

pub struct UnifiedCloudflareServices {
    config: UnifiedCloudflareConfig,
    d1_database: Option<D1Database>,
    kv_store: Option<KvStore>,
    metrics: Arc<RwLock<CloudflareMetrics>>,
    health: Arc<RwLock<ServiceHealth>>,
}

impl UnifiedCloudflareServices {
    pub fn new(config: UnifiedCloudflareConfig) -> Self {
        Self {
            config,
            d1_database: None,
            kv_store: None,
            metrics: Arc::new(RwLock::new(CloudflareMetrics::default())),
            health: Arc::new(RwLock::new(ServiceHealth::default())),
        }
    }

    pub async fn initialize(&mut self, env: &Env) -> ArbitrageResult<()> {
        // Initialize D1 Database
        if let Ok(database) = env.d1(&self.config.d1.database_name) {
            self.d1_database = Some(database);
            self.update_service_status("d1", ServiceStatus::Healthy)
                .await;
        } else {
            self.update_service_status("d1", ServiceStatus::Unhealthy)
                .await;
        }

        // Initialize KV Store
        if let Ok(kv) = env.kv(&self.config.kv.namespace) {
            self.kv_store = Some(kv);
            self.update_service_status("kv", ServiceStatus::Healthy)
                .await;
        } else {
            self.update_service_status("kv", ServiceStatus::Unhealthy)
                .await;
        }

        self.update_overall_health_status().await;
        Ok(())
    }

    // ============= D1 DATABASE OPERATIONS =============

    pub async fn execute_d1_query(
        &self,
        query: &str,
        params: &[&str],
    ) -> ArbitrageResult<worker::d1::D1Result> {
        let start_time = std::time::Instant::now();

        let database = self.d1_database.as_ref().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::InfrastructureError,
                "D1 database not initialized",
            )
        })?;

        // Convert string parameters to JsValue array
        let js_params: Vec<wasm_bindgen::JsValue> = params
            .iter()
            .map(|param| wasm_bindgen::JsValue::from_str(param))
            .collect();

        let result = database.prepare(query).bind(&js_params)?.all().await;

        let duration = start_time.elapsed();
        self.update_d1_metrics(duration.as_millis() as f64, result.is_ok())
            .await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => {
                self.update_service_status("d1", ServiceStatus::Degraded)
                    .await;
                Err(ArbitrageError::new(
                    ErrorKind::DatabaseError,
                    format!("D1 query failed: {:?}", error),
                ))
            }
        }
    }

    pub async fn execute_d1_batch(
        &self,
        queries: Vec<(&str, Vec<&str>)>,
    ) -> ArbitrageResult<Vec<worker::d1::D1Result>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();

        for (query, params) in queries {
            match self.execute_d1_query(query, &params).await {
                Ok(result) => results.push(result),
                Err(error) => return Err(error),
            }
        }

        let duration = start_time.elapsed();
        self.update_d1_metrics(duration.as_millis() as f64, true)
            .await;

        Ok(results)
    }

    // ============= KV STORE OPERATIONS =============

    pub async fn kv_get(&self, key: &str) -> ArbitrageResult<Option<String>> {
        let start_time = std::time::Instant::now();

        let kv = self.kv_store.as_ref().ok_or_else(|| {
            ArbitrageError::new(ErrorKind::InfrastructureError, "KV store not initialized")
        })?;

        let result = kv.get(key).text().await;

        let duration = start_time.elapsed();
        let is_hit = result.is_ok() && result.as_ref().unwrap().is_some();
        self.update_kv_metrics("read", duration.as_millis() as f64, is_hit)
            .await;

        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                self.update_service_status("kv", ServiceStatus::Degraded)
                    .await;
                Err(ArbitrageError::new(
                    ErrorKind::Cache,
                    format!("KV get failed: {:?}", error),
                ))
            }
        }
    }

    pub async fn kv_put(&self, key: &str, value: &str, ttl: Option<u64>) -> ArbitrageResult<()> {
        let start_time = std::time::Instant::now();

        let kv = self.kv_store.as_ref().ok_or_else(|| {
            ArbitrageError::new(ErrorKind::InfrastructureError, "KV store not initialized")
        })?;

        let mut put_request = kv.put(key, value)?;
        if let Some(ttl_seconds) = ttl {
            put_request = put_request.expiration_ttl(ttl_seconds);
        }

        let result = put_request.execute().await;

        let duration = start_time.elapsed();
        self.update_kv_metrics("write", duration.as_millis() as f64, result.is_ok())
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                self.update_service_status("kv", ServiceStatus::Degraded)
                    .await;
                Err(ArbitrageError::new(
                    ErrorKind::Cache,
                    format!("KV put failed: {:?}", error),
                ))
            }
        }
    }

    pub async fn kv_delete(&self, key: &str) -> ArbitrageResult<()> {
        let start_time = std::time::Instant::now();

        let kv = self.kv_store.as_ref().ok_or_else(|| {
            ArbitrageError::new(ErrorKind::InfrastructureError, "KV store not initialized")
        })?;

        let result = kv.delete(key).await;

        let duration = start_time.elapsed();
        self.update_kv_metrics("delete", duration.as_millis() as f64, result.is_ok())
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                self.update_service_status("kv", ServiceStatus::Degraded)
                    .await;
                Err(ArbitrageError::new(
                    ErrorKind::Cache,
                    format!("KV delete failed: {:?}", error),
                ))
            }
        }
    }

    // ============= R2 OBJECT STORAGE OPERATIONS =============

    /// Get object from R2 storage (placeholder - R2 not available in current WASM context)
    pub async fn r2_get(&self, _key: &str) -> ArbitrageResult<Option<Vec<u8>>> {
        // R2 operations removed due to Send/Sync compatibility issues in WASM
        Err(ArbitrageError::storage_error(
            "R2 operations not available in WASM context",
        ))
    }

    /// Put object to R2 storage (placeholder - R2 not available in current WASM context)
    pub async fn r2_put(&self, _key: &str, _data: Vec<u8>) -> ArbitrageResult<()> {
        // R2 operations removed due to Send/Sync compatibility issues in WASM
        Err(ArbitrageError::storage_error(
            "R2 operations not available in WASM context",
        ))
    }

    /// Delete object from R2 storage (placeholder - R2 not available in current WASM context)
    pub async fn r2_delete(&self, _key: &str) -> ArbitrageResult<()> {
        // R2 operations removed due to Send/Sync compatibility issues in WASM
        Err(ArbitrageError::storage_error(
            "R2 operations not available in WASM context",
        ))
    }

    // ============= PIPELINE OPERATIONS =============

    pub async fn execute_pipeline<T, F, Fut>(
        &self,
        name: &str,
        items: Vec<T>,
        processor: F,
    ) -> ArbitrageResult<Vec<T>>
    where
        T: Send + Sync + 'static + Clone,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = ArbitrageResult<T>> + Send,
    {
        let start_time = std::time::Instant::now();
        let total_items = items.len();

        // Process in batches for better performance
        let batch_size = self.config.pipelines.batch_processing_size;
        let mut results = Vec::new();
        let mut failed_count = 0;

        for chunk in items.chunks(batch_size) {
            let futures: Vec<_> = chunk
                .iter()
                .cloned()
                .map(|item| {
                    let processor = processor.clone();
                    async move { processor(item).await }
                })
                .collect();

            let batch_results = futures::future::join_all(futures).await;

            for result in batch_results {
                match result {
                    Ok(processed_item) => results.push(processed_item),
                    Err(_) => failed_count += 1,
                }
            }
        }

        let duration = start_time.elapsed();
        let success = failed_count == 0;

        self.update_pipeline_metrics(
            name,
            duration.as_millis() as f64,
            total_items as u64,
            success,
        )
        .await;

        if success {
            Ok(results)
        } else {
            Err(ArbitrageError::new(
                ErrorKind::Internal,
                format!(
                    "Pipeline {} failed {} out of {} items",
                    name, failed_count, total_items
                ),
            ))
        }
    }

    // ============= HEALTH CHECK OPERATIONS =============

    pub async fn perform_health_checks(&self) -> ArbitrageResult<ServiceHealth> {
        let mut health_status = ServiceHealth::default();

        // Check D1 health
        health_status.d1_status = self.check_d1_health().await;

        // Check KV health
        health_status.kv_status = self.check_kv_health().await;

        // Check R2 health
        health_status.r2_status = self.check_r2_health().await;

        // Check pipeline health
        health_status.pipeline_status = self.check_pipeline_health().await;

        // Determine overall status
        health_status.overall_status = self.calculate_overall_status(&health_status);

        // Update stored health status
        {
            let mut health = self.health.write();
            *health = health_status.clone();
        }

        self.update_health_metrics().await;

        Ok(health_status)
    }

    // ============= PRIVATE HELPER METHODS =============

    async fn update_d1_metrics(&self, duration_ms: f64, success: bool) {
        let mut metrics = self.metrics.write();
        metrics.d1_metrics.total_queries += 1;

        if success {
            metrics.d1_metrics.successful_queries += 1;
        } else {
            metrics.d1_metrics.failed_queries += 1;
        }

        // Update rolling average
        let total = metrics.d1_metrics.total_queries as f64;
        let current_avg = metrics.d1_metrics.average_query_time_ms;
        metrics.d1_metrics.average_query_time_ms =
            (current_avg * (total - 1.0) + duration_ms) / total;
    }

    async fn update_kv_metrics(&self, operation: &str, duration_ms: f64, is_hit: bool) {
        let mut metrics = self.metrics.write();
        metrics.kv_metrics.total_operations += 1;

        match operation {
            "read" => {
                metrics.kv_metrics.reads += 1;
                if is_hit {
                    metrics.kv_metrics.cache_hits += 1;
                } else {
                    metrics.kv_metrics.cache_misses += 1;
                }
            }
            "write" => metrics.kv_metrics.writes += 1,
            "delete" => metrics.kv_metrics.deletes += 1,
            _ => {}
        }

        // Update rolling average
        let total = metrics.kv_metrics.total_operations as f64;
        let current_avg = metrics.kv_metrics.average_response_time_ms;
        metrics.kv_metrics.average_response_time_ms =
            (current_avg * (total - 1.0) + duration_ms) / total;
    }

    #[allow(dead_code)]
    async fn update_r2_metrics(
        &self,
        operation: &str,
        duration_ms: f64,
        bytes: u64,
        success: bool,
    ) {
        let mut metrics = self.metrics.write();
        metrics.r2_metrics.total_operations += 1;

        if success {
            match operation {
                "upload" => metrics.r2_metrics.uploads += 1,
                "download" => metrics.r2_metrics.downloads += 1,
                "delete" => metrics.r2_metrics.deletes += 1,
                _ => {}
            }
            metrics.r2_metrics.total_bytes_transferred += bytes;
        }

        // Update rolling average
        let total = metrics.r2_metrics.total_operations as f64;
        let current_avg = metrics.r2_metrics.average_transfer_time_ms;
        metrics.r2_metrics.average_transfer_time_ms =
            (current_avg * (total - 1.0) + duration_ms) / total;
    }

    async fn update_pipeline_metrics(
        &self,
        _name: &str,
        duration_ms: f64,
        items_processed: u64,
        success: bool,
    ) {
        let mut metrics = self.metrics.write();

        if success {
            metrics.pipeline_metrics.completed_pipelines += 1;
        } else {
            metrics.pipeline_metrics.failed_pipelines += 1;
        }

        metrics.pipeline_metrics.total_items_processed += items_processed;

        // Update rolling average
        let total_pipelines = metrics.pipeline_metrics.completed_pipelines
            + metrics.pipeline_metrics.failed_pipelines;
        if total_pipelines > 0 {
            let current_avg = metrics.pipeline_metrics.average_processing_time_ms;
            metrics.pipeline_metrics.average_processing_time_ms =
                (current_avg * (total_pipelines - 1) as f64 + duration_ms) / total_pipelines as f64;
        }
    }

    async fn update_service_status(&self, service: &str, status: ServiceStatus) {
        let mut health = self.health.write();
        match service {
            "d1" => health.d1_status = status,
            "kv" => health.kv_status = status,
            "r2" => health.r2_status = status,
            "pipeline" => health.pipeline_status = status,
            _ => {}
        }
    }

    async fn update_overall_health_status(&self) {
        let health = self.health.read();
        let overall = self.calculate_overall_status(&health);
        drop(health);

        let mut health = self.health.write();
        health.overall_status = overall;
    }

    fn calculate_overall_status(&self, health: &ServiceHealth) -> ServiceStatus {
        let statuses = [
            &health.d1_status,
            &health.kv_status,
            &health.r2_status,
            &health.pipeline_status,
        ];

        let unhealthy_count = statuses
            .iter()
            .filter(|&s| matches!(s, ServiceStatus::Unhealthy))
            .count();
        let degraded_count = statuses
            .iter()
            .filter(|&s| matches!(s, ServiceStatus::Degraded))
            .count();

        if unhealthy_count > statuses.len() / 2 {
            ServiceStatus::Unhealthy
        } else if unhealthy_count > 0 || degraded_count > statuses.len() / 2 {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        }
    }

    async fn check_d1_health(&self) -> ServiceStatus {
        if self.d1_database.is_none() {
            return ServiceStatus::Unhealthy;
        }

        // Simple health check query
        match self.execute_d1_query("SELECT 1", &[]).await {
            Ok(_) => ServiceStatus::Healthy,
            Err(_) => ServiceStatus::Unhealthy,
        }
    }

    async fn check_kv_health(&self) -> ServiceStatus {
        if self.kv_store.is_none() {
            return ServiceStatus::Unhealthy;
        }

        // Try a simple KV operation
        match self.kv_get("__health_check__").await {
            Ok(_) => ServiceStatus::Healthy,
            Err(_) => ServiceStatus::Degraded, // KV might be available but slow
        }
    }

    async fn check_r2_health(&self) -> ServiceStatus {
        // R2 health check disabled due to WASM compatibility
        ServiceStatus::Maintenance
    }

    async fn check_pipeline_health(&self) -> ServiceStatus {
        // Pipeline health is based on recent metrics
        let metrics = self.metrics.read();
        let total_pipelines = metrics.pipeline_metrics.completed_pipelines
            + metrics.pipeline_metrics.failed_pipelines;

        if total_pipelines == 0 {
            return ServiceStatus::Healthy; // No pipelines run yet
        }

        let failure_rate =
            metrics.pipeline_metrics.failed_pipelines as f64 / total_pipelines as f64;

        if failure_rate > 0.5 {
            ServiceStatus::Unhealthy
        } else if failure_rate > 0.1 {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        }
    }

    async fn update_health_metrics(&self) {
        let mut metrics = self.metrics.write();
        let health = self.health.read();

        // Calculate uptime percentage based on service statuses
        let healthy_services = [
            &health.d1_status,
            &health.kv_status,
            &health.r2_status,
            &health.pipeline_status,
        ]
        .iter()
        .filter(|&s| matches!(s, ServiceStatus::Healthy))
        .count();

        metrics.health_metrics.uptime_percent = (healthy_services as f64 / 4.0) * 100.0;

        // Calculate error rate
        let total_operations = metrics.d1_metrics.total_queries
            + metrics.kv_metrics.total_operations
            + metrics.r2_metrics.total_operations;
        let failed_operations = metrics.d1_metrics.failed_queries
            + (metrics.kv_metrics.total_operations - metrics.kv_metrics.cache_hits)
            + (metrics.r2_metrics.total_operations
                - metrics.r2_metrics.uploads
                - metrics.r2_metrics.downloads
                - metrics.r2_metrics.deletes);

        if total_operations > 0 {
            metrics.health_metrics.error_rate_percent =
                (failed_operations as f64 / total_operations as f64) * 100.0;
        }

        // Calculate average response time across all services
        let response_times = [
            metrics.d1_metrics.average_query_time_ms,
            metrics.kv_metrics.average_response_time_ms,
            metrics.r2_metrics.average_transfer_time_ms,
        ];

        let valid_times: Vec<f64> = response_times
            .iter()
            .filter(|&&t| t > 0.0)
            .cloned()
            .collect();
        if !valid_times.is_empty() {
            metrics.health_metrics.average_response_time_ms =
                valid_times.iter().sum::<f64>() / valid_times.len() as f64;
        }

        metrics.health_metrics.last_health_check = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    // ============= PUBLIC INTERFACE METHODS =============

    pub async fn get_metrics(&self) -> CloudflareMetrics {
        self.metrics.read().clone()
    }

    pub async fn get_health(&self) -> ServiceHealth {
        self.health.read().clone()
    }

    pub fn is_initialized(&self) -> bool {
        self.d1_database.is_some() || self.kv_store.is_some()
    }
}

// ============= DEFAULT IMPLEMENTATIONS =============

impl Default for UnifiedCloudflareConfig {
    fn default() -> Self {
        Self {
            d1: D1Config {
                database_name: "main".to_string(),
                connection_timeout_ms: 5000,
                query_timeout_ms: 10000,
                max_batch_size: 100,
                enable_performance_monitoring: true,
            },
            kv: KvConfig {
                namespace: "main".to_string(),
                default_ttl_seconds: 3600,
                max_key_size_bytes: 512,
                max_value_size_bytes: 25 * 1024 * 1024, // 25MB
                compression_enabled: true,
                batch_size: 50,
            },
            r2: R2Config {
                bucket_name: "main".to_string(),
                max_object_size_bytes: 100 * 1024 * 1024, // 100MB
                default_storage_class: "Standard".to_string(),
                enable_multipart_upload: true,
                multipart_threshold_bytes: 100 * 1024 * 1024, // 100MB
            },
            pipelines: PipelinesConfig {
                max_concurrent_pipelines: 10,
                pipeline_timeout_ms: 30000,
                batch_processing_size: 100,
                enable_priority_queuing: true,
            },
            health: HealthConfig {
                check_interval_ms: 30000,
                timeout_ms: 5000,
                enable_detailed_monitoring: true,
                alert_thresholds: AlertThresholds {
                    error_rate_percent: 5.0,
                    response_time_ms: 1000,
                    availability_percent: 99.0,
                },
            },
        }
    }
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            d1_status: ServiceStatus::Healthy,
            kv_status: ServiceStatus::Healthy,
            r2_status: ServiceStatus::Healthy,
            pipeline_status: ServiceStatus::Healthy,
            overall_status: ServiceStatus::Healthy,
        }
    }
}

// ============= TESTS =============

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let config = UnifiedCloudflareConfig::default();
        let service = UnifiedCloudflareServices::new(config);

        assert!(!service.is_initialized());
    }

    #[tokio::test]
    async fn test_metrics_initialization() {
        let config = UnifiedCloudflareConfig::default();
        let service = UnifiedCloudflareServices::new(config);

        let metrics = service.get_metrics().await;
        assert_eq!(metrics.d1_metrics.total_queries, 0);
        assert_eq!(metrics.kv_metrics.total_operations, 0);
        assert_eq!(metrics.r2_metrics.total_operations, 0);
    }

    #[tokio::test]
    async fn test_health_status_calculation() {
        let config = UnifiedCloudflareConfig::default();
        let service = UnifiedCloudflareServices::new(config);

        let health = ServiceHealth {
            d1_status: ServiceStatus::Healthy,
            kv_status: ServiceStatus::Healthy,
            r2_status: ServiceStatus::Degraded,
            pipeline_status: ServiceStatus::Healthy,
            overall_status: ServiceStatus::Healthy,
        };

        let overall = service.calculate_overall_status(&health);
        assert_eq!(overall, ServiceStatus::Healthy);
    }

    #[tokio::test]
    async fn test_pipeline_processing() {
        let config = UnifiedCloudflareConfig::default();
        let service = UnifiedCloudflareServices::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let result = service
            .execute_pipeline("test_pipeline", items, |item| async move { Ok(item * 2) })
            .await;

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed, vec![2, 4, 6, 8, 10]);
    }
}
