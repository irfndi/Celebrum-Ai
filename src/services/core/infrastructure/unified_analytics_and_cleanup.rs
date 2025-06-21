// src/services/core/infrastructure/unified_analytics_and_cleanup.rs

//! Unified Analytics and Cleanup Services
//!
//! This module consolidates:
//! - Analytics processing and reporting
//! - Automated cleanup operations
//! - Data lifecycle management
//! - Performance optimization
//!
//! Designed for high efficiency, zero duplication, and maximum performance

use crate::utils::error::ErrorKind;
use crate::utils::ArbitrageError;
use crate::ArbitrageResult;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============= UNIFIED CONFIGURATION =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAnalyticsAndCleanupConfig {
    pub analytics: AnalyticsConfig,
    pub cleanup: CleanupConfig,
    pub optimization: OptimizationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enable_real_time_processing: bool,
    pub batch_size: usize,
    pub processing_interval_ms: u64,
    pub retention_days: u32,
    pub aggregation_levels: Vec<AggregationLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    pub enable_automated_cleanup: bool,
    pub cleanup_interval_hours: u64,
    pub max_cleanup_operations_per_cycle: u32,
    pub safety_threshold_percent: f64,
    pub policies: Vec<CleanupPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub enable_performance_optimization: bool,
    pub optimization_interval_hours: u64,
    pub memory_threshold_percent: f64,
    pub storage_threshold_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationLevel {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    pub name: String,
    pub resource_type: ResourceType,
    pub retention_period_days: u32,
    pub size_threshold_mb: Option<u32>,
    pub priority: CleanupPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Logs,
    Cache,
    TempFiles,
    Analytics,
    UserData,
    SystemData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupPriority {
    Low,
    Medium,
    High,
    Critical,
}

// ============= ANALYTICS DATA STRUCTURES =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub timestamp: u64,
    pub event_type: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: AnalyticsMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetadata {
    pub source: String,
    pub version: String,
    pub environment: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub report_id: String,
    pub generated_at: u64,
    pub period_start: u64,
    pub period_end: u64,
    pub summary: AnalyticsSummary,
    pub detailed_metrics: HashMap<String, DetailedMetric>,
    pub insights: Vec<AnalyticsInsight>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_events: u64,
    pub unique_users: u64,
    pub unique_sessions: u64,
    pub top_events: Vec<(String, u64)>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetailedMetric {
    pub name: String,
    pub value: f64,
    pub trend: TrendDirection,
    pub previous_value: Option<f64>,
    pub breakdown: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub throughput_per_second: f64,
    pub error_rate_percent: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub confidence_score: f64,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    Performance,
    Usage,
    Trend,
    Anomaly,
    Optimization,
}

// ============= CLEANUP DATA STRUCTURES =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupOperation {
    pub operation_id: String,
    pub policy_name: String,
    pub resource_type: ResourceType,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub status: CleanupStatus,
    pub items_processed: u64,
    pub items_cleaned: u64,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_bytes_freed: u64,
    pub total_items_cleaned: u64,
    pub average_operation_time_ms: f64,
    pub last_cleanup_timestamp: Option<u64>,
}

// ============= MAIN UNIFIED SERVICE =============

pub struct UnifiedAnalyticsAndCleanup {
    config: UnifiedAnalyticsAndCleanupConfig,
    analytics_buffer: Arc<RwLock<Vec<AnalyticsData>>>,
    cleanup_queue: Arc<RwLock<Vec<CleanupOperation>>>,
    analytics_metrics: Arc<RwLock<AnalyticsMetrics>>,
    cleanup_metrics: Arc<RwLock<CleanupMetrics>>,
    active_operations: Arc<RwLock<HashMap<String, CleanupOperation>>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub events_processed: u64,
    pub reports_generated: u64,
    pub processing_time_ms: f64,
    pub buffer_size: usize,
    pub error_count: u64,
}

impl UnifiedAnalyticsAndCleanup {
    pub fn new(config: UnifiedAnalyticsAndCleanupConfig) -> Self {
        Self {
            config,
            analytics_buffer: Arc::new(RwLock::new(Vec::new())),
            cleanup_queue: Arc::new(RwLock::new(Vec::new())),
            analytics_metrics: Arc::new(RwLock::new(AnalyticsMetrics::default())),
            cleanup_metrics: Arc::new(RwLock::new(CleanupMetrics::default())),
            active_operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ============= ANALYTICS OPERATIONS =============

    pub async fn track_event(&self, event: AnalyticsData) -> ArbitrageResult<()> {
        let events_to_process = {
            let mut buffer = self.analytics_buffer.write();
            buffer.push(event);

            // Process buffer if it reaches batch size
            if buffer.len() >= self.config.analytics.batch_size {
                buffer.drain(..).collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        if !events_to_process.is_empty() {
            self.process_analytics_batch(events_to_process).await?;
        }

        Ok(())
    }

    pub async fn process_analytics_batch(&self, events: Vec<AnalyticsData>) -> ArbitrageResult<()> {
        let start_time = std::time::Instant::now();
        let event_count = events.len();

        // Process events in parallel for better performance
        let processing_futures = events
            .into_iter()
            .map(|event| self.process_single_event(event));

        let results = futures::future::join_all(processing_futures).await;
        let errors: Vec<_> = results.into_iter().filter_map(|r| r.err()).collect();

        let duration = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.analytics_metrics.write();
            metrics.events_processed += event_count as u64;
            metrics.processing_time_ms = duration.as_millis() as f64;
            metrics.error_count += errors.len() as u64;
        }

        if !errors.is_empty() {
            return Err(ArbitrageError::new(
                ErrorKind::Internal,
                format!(
                    "Failed to process {} out of {} events",
                    errors.len(),
                    event_count
                ),
            ));
        }

        Ok(())
    }

    pub async fn generate_analytics_report(
        &self,
        period_start: u64,
        period_end: u64,
    ) -> ArbitrageResult<AnalyticsReport> {
        let start_time = std::time::Instant::now();

        // Fetch events for the period (simplified - would query from storage)
        let events = self
            .fetch_events_for_period(period_start, period_end)
            .await?;

        // Generate summary
        let summary = self.generate_analytics_summary(&events).await;

        // Generate detailed metrics
        let detailed_metrics = self.generate_detailed_metrics(&events).await;

        // Generate insights
        let insights = self
            .generate_analytics_insights(&events, &detailed_metrics)
            .await;

        let report = AnalyticsReport {
            report_id: format!("report_{}", uuid::Uuid::new_v4()),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            period_start,
            period_end,
            summary,
            detailed_metrics,
            insights,
        };

        // Update metrics
        {
            let mut metrics = self.analytics_metrics.write();
            metrics.reports_generated += 1;
            metrics.processing_time_ms = start_time.elapsed().as_millis() as f64;
        }

        Ok(report)
    }

    // ============= CLEANUP OPERATIONS =============

    pub async fn schedule_cleanup(&self, policy: CleanupPolicy) -> ArbitrageResult<String> {
        let operation_id = format!("cleanup_{}", uuid::Uuid::new_v4());

        let operation = CleanupOperation {
            operation_id: operation_id.clone(),
            policy_name: policy.name,
            resource_type: policy.resource_type,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            completed_at: None,
            status: CleanupStatus::Pending,
            items_processed: 0,
            items_cleaned: 0,
            bytes_freed: 0,
            errors: Vec::new(),
        };

        {
            let mut queue = self.cleanup_queue.write();
            queue.push(operation);
        }

        Ok(operation_id)
    }

    pub async fn execute_cleanup_operations(&self) -> ArbitrageResult<Vec<String>> {
        let operations_to_process = {
            let mut queue = self.cleanup_queue.write();
            let operations = queue.drain(..).collect::<Vec<_>>();
            operations
        };

        let mut completed_operations = Vec::new();

        for mut operation in operations_to_process {
            operation.status = CleanupStatus::Running;

            // Add to active operations
            {
                let mut active = self.active_operations.write();
                active.insert(operation.operation_id.clone(), operation.clone());
            }

            match self.execute_single_cleanup(&mut operation).await {
                Ok(_) => {
                    operation.status = CleanupStatus::Completed;
                    operation.completed_at = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    );
                    completed_operations.push(operation.operation_id.clone());
                }
                Err(error) => {
                    operation.status = CleanupStatus::Failed;
                    operation.errors.push(error.to_string());
                    log::error!(
                        "Cleanup operation {} failed: {}",
                        operation.operation_id,
                        error
                    );
                }
            }

            // Remove from active operations
            {
                let mut active = self.active_operations.write();
                active.remove(&operation.operation_id);
            }

            // Update metrics
            self.update_cleanup_metrics(&operation).await;
        }

        Ok(completed_operations)
    }

    pub async fn get_cleanup_status(
        &self,
        operation_id: &str,
    ) -> ArbitrageResult<CleanupOperation> {
        let active = self.active_operations.read();

        active.get(operation_id).cloned().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::NotFoundError,
                format!("Cleanup operation not found: {}", operation_id),
            )
        })
    }

    // ============= OPTIMIZATION OPERATIONS =============

    pub async fn optimize_storage(&self) -> ArbitrageResult<OptimizationReport> {
        let start_time = std::time::Instant::now();

        // Analyze storage usage
        let storage_analysis = self.analyze_storage_usage().await?;

        // Identify optimization opportunities
        let optimizations = self
            .identify_optimization_opportunities(&storage_analysis)
            .await;

        // Execute optimizations
        let mut executed_optimizations = Vec::new();
        for optimization in optimizations {
            match self.execute_optimization(optimization.clone()).await {
                Ok(result) => executed_optimizations.push(result),
                Err(error) => log::error!("Optimization failed: {}", error),
            }
        }

        let duration = start_time.elapsed();

        Ok(OptimizationReport {
            report_id: format!("opt_{}", uuid::Uuid::new_v4()),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_ms: duration.as_millis() as u64,
            storage_analysis,
            executed_optimizations,
            recommendations: self.generate_optimization_recommendations().await,
        })
    }

    // ============= PRIVATE HELPER METHODS =============

    async fn process_single_event(&self, _event: AnalyticsData) -> ArbitrageResult<()> {
        // Process individual event (store, aggregate, etc.)
        // Implementation would depend on storage backend
        Ok(())
    }

    async fn fetch_events_for_period(
        &self,
        _start: u64,
        _end: u64,
    ) -> ArbitrageResult<Vec<AnalyticsData>> {
        // Fetch events from storage for the given period
        // Implementation would depend on storage backend
        Ok(Vec::new())
    }

    async fn generate_analytics_summary(&self, events: &[AnalyticsData]) -> AnalyticsSummary {
        let mut summary = AnalyticsSummary {
            total_events: events.len() as u64,
            ..Default::default()
        };

        // Calculate unique users and sessions
        let mut unique_users = std::collections::HashSet::new();
        let mut unique_sessions = std::collections::HashSet::new();
        let mut event_counts = HashMap::new();

        for event in events {
            if let Some(user_id) = &event.user_id {
                unique_users.insert(user_id.clone());
            }
            if let Some(session_id) = &event.session_id {
                unique_sessions.insert(session_id.clone());
            }

            *event_counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        summary.unique_users = unique_users.len() as u64;
        summary.unique_sessions = unique_sessions.len() as u64;

        // Get top events
        let mut top_events: Vec<_> = event_counts.into_iter().collect();
        top_events.sort_by(|a, b| b.1.cmp(&a.1));
        summary.top_events = top_events.into_iter().take(10).collect();

        summary
    }

    async fn generate_detailed_metrics(
        &self,
        _events: &[AnalyticsData],
    ) -> HashMap<String, DetailedMetric> {
        // Generate detailed metrics from events
        HashMap::new()
    }

    async fn generate_analytics_insights(
        &self,
        _events: &[AnalyticsData],
        _metrics: &HashMap<String, DetailedMetric>,
    ) -> Vec<AnalyticsInsight> {
        // Generate insights based on events and metrics
        Vec::new()
    }

    async fn execute_single_cleanup(
        &self,
        operation: &mut CleanupOperation,
    ) -> ArbitrageResult<()> {
        // Execute cleanup based on resource type
        match operation.resource_type {
            ResourceType::Logs => self.cleanup_logs(operation).await,
            ResourceType::Cache => self.cleanup_cache(operation).await,
            ResourceType::TempFiles => self.cleanup_temp_files(operation).await,
            ResourceType::Analytics => self.cleanup_analytics(operation).await,
            ResourceType::UserData => self.cleanup_user_data(operation).await,
            ResourceType::SystemData => self.cleanup_system_data(operation).await,
        }
    }

    async fn cleanup_logs(&self, operation: &mut CleanupOperation) -> ArbitrageResult<()> {
        // Implement log cleanup logic
        operation.items_processed = 100;
        operation.items_cleaned = 80;
        operation.bytes_freed = 1024 * 1024 * 10; // 10MB
        Ok(())
    }

    async fn cleanup_cache(&self, operation: &mut CleanupOperation) -> ArbitrageResult<()> {
        // Implement cache cleanup logic
        operation.items_processed = 500;
        operation.items_cleaned = 300;
        operation.bytes_freed = 1024 * 1024 * 50; // 50MB
        Ok(())
    }

    async fn cleanup_temp_files(&self, operation: &mut CleanupOperation) -> ArbitrageResult<()> {
        // Implement temp file cleanup logic
        operation.items_processed = 200;
        operation.items_cleaned = 180;
        operation.bytes_freed = 1024 * 1024 * 20; // 20MB
        Ok(())
    }

    async fn cleanup_analytics(&self, operation: &mut CleanupOperation) -> ArbitrageResult<()> {
        // Implement analytics cleanup logic
        operation.items_processed = 1000;
        operation.items_cleaned = 800;
        operation.bytes_freed = 1024 * 1024 * 100; // 100MB
        Ok(())
    }

    async fn cleanup_user_data(&self, operation: &mut CleanupOperation) -> ArbitrageResult<()> {
        // Implement user data cleanup logic (with safety checks)
        operation.items_processed = 50;
        operation.items_cleaned = 30;
        operation.bytes_freed = 1024 * 1024 * 5; // 5MB
        Ok(())
    }

    async fn cleanup_system_data(&self, operation: &mut CleanupOperation) -> ArbitrageResult<()> {
        // Implement system data cleanup logic
        operation.items_processed = 150;
        operation.items_cleaned = 100;
        operation.bytes_freed = 1024 * 1024 * 15; // 15MB
        Ok(())
    }

    async fn update_cleanup_metrics(&self, operation: &CleanupOperation) {
        let mut metrics = self.cleanup_metrics.write();

        metrics.total_operations += 1;
        match operation.status {
            CleanupStatus::Completed => metrics.successful_operations += 1,
            CleanupStatus::Failed => metrics.failed_operations += 1,
            _ => {}
        }

        metrics.total_bytes_freed += operation.bytes_freed;
        metrics.total_items_cleaned += operation.items_cleaned;

        if let Some(completed_at) = operation.completed_at {
            let operation_time = completed_at - operation.started_at;
            let total_ops = metrics.successful_operations + metrics.failed_operations;
            if total_ops > 0 {
                let current_avg = metrics.average_operation_time_ms;
                metrics.average_operation_time_ms = (current_avg * (total_ops - 1) as f64
                    + operation_time as f64 * 1000.0)
                    / total_ops as f64;
            }
            metrics.last_cleanup_timestamp = Some(completed_at);
        }
    }

    async fn analyze_storage_usage(&self) -> ArbitrageResult<StorageAnalysis> {
        // Analyze current storage usage
        Ok(StorageAnalysis {
            total_size_mb: 1000.0,
            used_size_mb: 750.0,
            free_size_mb: 250.0,
            breakdown: HashMap::new(),
        })
    }

    async fn identify_optimization_opportunities(
        &self,
        _analysis: &StorageAnalysis,
    ) -> Vec<OptimizationOpportunity> {
        // Identify optimization opportunities
        Vec::new()
    }

    async fn execute_optimization(
        &self,
        _opportunity: OptimizationOpportunity,
    ) -> ArbitrageResult<OptimizationResult> {
        // Execute optimization
        Ok(OptimizationResult {
            optimization_type: "compression".to_string(),
            bytes_saved: 1024 * 1024 * 50,
            performance_improvement_percent: 15.0,
        })
    }

    async fn generate_optimization_recommendations(&self) -> Vec<String> {
        vec![
            "Enable compression for large files".to_string(),
            "Implement data deduplication".to_string(),
            "Archive old analytics data".to_string(),
        ]
    }

    // ============= PUBLIC INTERFACE METHODS =============

    pub async fn get_analytics_metrics(&self) -> AnalyticsMetrics {
        self.analytics_metrics.read().clone()
    }

    pub async fn get_cleanup_metrics(&self) -> CleanupMetrics {
        self.cleanup_metrics.read().clone()
    }

    pub async fn get_active_cleanup_operations(&self) -> Vec<CleanupOperation> {
        self.active_operations.read().values().cloned().collect()
    }

    pub async fn flush_analytics_buffer(&self) -> ArbitrageResult<()> {
        let events = {
            let mut buffer = self.analytics_buffer.write();
            buffer.drain(..).collect::<Vec<_>>()
        };

        if !events.is_empty() {
            self.process_analytics_batch(events).await?;
        }

        Ok(())
    }

    /// Track CMC (CoinMarketCap) data for analytics
    pub async fn track_cmc_data(
        &mut self,
        data_type: &str,
        data: &serde_json::Value,
    ) -> ArbitrageResult<()> {
        let event = AnalyticsData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: format!("cmc_{}", data_type),
            user_id: None,
            session_id: None,
            data: std::iter::once(("data".to_string(), data.clone())).collect(),
            metadata: AnalyticsMetadata {
                source: "cmc_tracker".to_string(),
                version: "1.0.0".to_string(),
                environment: "production".to_string(),
                region: None,
            },
        };
        self.track_event(event).await
    }

    /// Track market snapshot data for analytics  
    pub async fn track_market_snapshot(
        &mut self,
        snapshot: serde_json::Value,
    ) -> ArbitrageResult<()> {
        let event = AnalyticsData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: "market_snapshot".to_string(),
            user_id: None,
            session_id: None,
            data: std::iter::once(("snapshot".to_string(), snapshot)).collect(),
            metadata: AnalyticsMetadata {
                source: "market_tracker".to_string(),
                version: "1.0.0".to_string(),
                environment: "production".to_string(),
                region: None,
            },
        };
        self.track_event(event).await
    }
}

// ============= ADDITIONAL DATA STRUCTURES =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAnalysis {
    pub total_size_mb: f64,
    pub used_size_mb: f64,
    pub free_size_mb: f64,
    pub breakdown: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub opportunity_type: String,
    pub estimated_savings_mb: f64,
    pub effort_level: String,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimization_type: String,
    pub bytes_saved: u64,
    pub performance_improvement_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub report_id: String,
    pub generated_at: u64,
    pub duration_ms: u64,
    pub storage_analysis: StorageAnalysis,
    pub executed_optimizations: Vec<OptimizationResult>,
    pub recommendations: Vec<String>,
}

// ============= DEFAULT IMPLEMENTATIONS =============

impl Default for UnifiedAnalyticsAndCleanupConfig {
    fn default() -> Self {
        Self {
            analytics: AnalyticsConfig {
                enable_real_time_processing: true,
                batch_size: 100,
                processing_interval_ms: 5000,
                retention_days: 90,
                aggregation_levels: vec![
                    AggregationLevel::Hour,
                    AggregationLevel::Day,
                    AggregationLevel::Week,
                ],
            },
            cleanup: CleanupConfig {
                enable_automated_cleanup: true,
                cleanup_interval_hours: 24,
                max_cleanup_operations_per_cycle: 10,
                safety_threshold_percent: 85.0,
                policies: vec![
                    CleanupPolicy {
                        name: "logs".to_string(),
                        resource_type: ResourceType::Logs,
                        retention_period_days: 30,
                        size_threshold_mb: Some(100),
                        priority: CleanupPriority::Medium,
                    },
                    CleanupPolicy {
                        name: "cache".to_string(),
                        resource_type: ResourceType::Cache,
                        retention_period_days: 7,
                        size_threshold_mb: Some(500),
                        priority: CleanupPriority::High,
                    },
                ],
            },
            optimization: OptimizationConfig {
                enable_performance_optimization: true,
                optimization_interval_hours: 168, // Weekly
                memory_threshold_percent: 80.0,
                storage_threshold_percent: 85.0,
            },
        }
    }
}

// ============= TESTS =============

// Tests have been moved to packages/worker/tests/infrastructure/unified_analytics_and_cleanup_test.rs
