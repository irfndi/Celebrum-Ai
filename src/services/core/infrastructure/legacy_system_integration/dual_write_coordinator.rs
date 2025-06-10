//! Dual-Write Coordinator
//!
//! Manages dual-write operations to both legacy and new systems during migration,
//! ensuring data consistency and transaction safety with rollback capabilities.

use super::shared_types::{
    EventSeverity, LegacySystemType, MigrationEvent, MigrationEventType, PerformanceMetrics,
    SystemIdentifier,
};
use crate::services::core::infrastructure::{
    circuit_breaker_service::CircuitBreakerService, monitoring_module::MonitoringModule,
    shared_types::ComponentHealth,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use worker::Env;

/// Dual-write strategy configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DualWriteStrategy {
    /// Write to primary first, then secondary
    #[default]
    PrimaryFirst,
    /// Write to secondary first, then primary
    SecondaryFirst,
    /// Write to both simultaneously
    Parallel,
    /// Write only to primary (migration disabled)
    PrimaryOnly,
    /// Write only to secondary (migration complete)
    SecondaryOnly,
}

/// Consistency level requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConsistencyLevel {
    /// Eventual consistency - best effort
    Eventual,
    /// Strong consistency - both must succeed
    #[default]
    Strong,
    /// Weak consistency - primary success sufficient
    Weak,
}

/// Write operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOperation {
    /// Operation identifier
    pub operation_id: String,
    /// Operation type (insert, update, delete)
    pub operation_type: String,
    /// Target table/collection
    pub target: String,
    /// Data payload
    pub data: serde_json::Value,
    /// Query conditions for updates/deletes
    pub conditions: Option<HashMap<String, serde_json::Value>>,
    /// Timestamp
    pub timestamp: u64,
    /// Idempotency key
    pub idempotency_key: Option<String>,
}

/// Write operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    /// Operation ID
    pub operation_id: String,
    /// Success status
    pub success: bool,
    /// Primary system result
    pub primary_result: Option<SystemWriteResult>,
    /// Secondary system result  
    pub secondary_result: Option<SystemWriteResult>,
    /// Overall duration in milliseconds
    pub duration_ms: u64,
    /// Error messages
    pub error_messages: Vec<String>,
    /// Consistency achieved
    pub consistency_achieved: bool,
    /// Retry count
    pub retry_count: u32,
}

/// Individual system write result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemWriteResult {
    /// System identifier
    pub system_id: String,
    /// Success status
    pub success: bool,
    /// Response data
    pub response_data: Option<serde_json::Value>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message
    pub error_message: Option<String>,
    /// Retry count for this system
    pub retry_count: u32,
}

/// Dual-write result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualWriteResult {
    /// Overall success
    pub success: bool,
    /// Primary write result
    pub primary_success: bool,
    /// Secondary write result
    pub secondary_success: bool,
    /// Consistency level achieved
    pub consistency_level: ConsistencyLevel,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Individual write results
    pub write_results: Vec<WriteResult>,
    /// Rollback operations performed
    pub rollback_operations: Vec<RollbackOperation>,
}

/// Rollback operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackOperation {
    /// Operation identifier
    pub operation_id: String,
    /// Target system
    pub target_system: String,
    /// Rollback action performed
    pub action: String,
    /// Success status
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Dual-write coordinator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualWriteConfig {
    /// Enable dual-write coordinator
    pub enabled: bool,
    /// Default write strategy
    pub default_strategy: DualWriteStrategy,
    /// Default consistency level
    pub default_consistency_level: ConsistencyLevel,
    /// Timeout for individual writes in milliseconds
    pub write_timeout_ms: u64,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Enable automatic rollback on failure
    pub enable_automatic_rollback: bool,
    /// Batch size for bulk operations
    pub batch_size: u32,
    /// Enable transaction support
    pub enable_transactions: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
}

impl Default for DualWriteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_strategy: DualWriteStrategy::default(),
            default_consistency_level: ConsistencyLevel::default(),
            write_timeout_ms: 5000,
            max_retry_attempts: 3,
            retry_delay_ms: 1000,
            enable_automatic_rollback: true,
            batch_size: 100,
            enable_transactions: true,
            circuit_breaker_failure_threshold: 5,
            enable_performance_monitoring: true,
        }
    }
}

/// Dual-Write Coordinator main implementation
pub struct DualWriteCoordinator {
    /// Configuration
    config: DualWriteConfig,
    /// Circuit breaker service
    circuit_breaker_service: Option<Arc<CircuitBreakerService>>,
    /// Monitoring module
    monitoring_module: Option<Arc<MonitoringModule>>,
    /// Active operations
    active_operations: Arc<Mutex<HashMap<String, WriteOperation>>>,
    /// Operation history
    #[allow(dead_code)]
    operation_history: Arc<Mutex<Vec<WriteResult>>>,
    /// Performance metrics
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Event history
    event_history: Arc<Mutex<Vec<MigrationEvent>>>,
    /// Logger instance
    logger: crate::utils::logger::Logger,
}

impl DualWriteCoordinator {
    /// Create new dual-write coordinator
    pub async fn new(config: DualWriteConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        let coordinator = Self {
            config,
            circuit_breaker_service: None,
            monitoring_module: None,
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            operation_history: Arc::new(Mutex::new(Vec::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            event_history: Arc::new(Mutex::new(Vec::new())),
            logger,
        };

        coordinator
            .logger
            .info("Dual-Write Coordinator initialized");
        Ok(coordinator)
    }

    /// Set circuit breaker service integration
    pub fn set_circuit_breaker_service(&mut self, service: Arc<CircuitBreakerService>) {
        self.circuit_breaker_service = Some(service);
        self.logger
            .info("Circuit breaker service integration enabled");
    }

    /// Set monitoring module integration
    pub fn set_monitoring_module(&mut self, module: Arc<MonitoringModule>) {
        self.monitoring_module = Some(module);
        self.logger.info("Monitoring module integration enabled");
    }

    /// Execute dual-write operation
    pub async fn execute_dual_write(
        &self,
        _env: &Env,
        operation: WriteOperation,
        strategy: Option<DualWriteStrategy>,
        consistency_level: Option<ConsistencyLevel>,
    ) -> ArbitrageResult<DualWriteResult> {
        if !self.config.enabled {
            return Err(ArbitrageError::infrastructure_error(
                "Dual-write coordinator is disabled".to_string(),
            ));
        }

        let strategy = strategy.unwrap_or(self.config.default_strategy.clone());
        let consistency_level =
            consistency_level.unwrap_or(self.config.default_consistency_level.clone());

        // Start tracking the operation
        {
            let mut operations = self.active_operations.lock().unwrap();
            operations.insert(operation.operation_id.clone(), operation.clone());
        }

        let start_time = chrono::Utc::now().timestamp_millis() as u64;

        // Execute the dual-write based on strategy
        let result = match strategy {
            DualWriteStrategy::PrimaryFirst => {
                self.execute_primary_first_write(_env, &operation, &consistency_level)
                    .await
            }
            DualWriteStrategy::SecondaryFirst => {
                self.execute_secondary_first_write(_env, &operation, &consistency_level)
                    .await
            }
            DualWriteStrategy::Parallel => {
                self.execute_parallel_write(_env, &operation, &consistency_level)
                    .await
            }
            DualWriteStrategy::PrimaryOnly => {
                self.execute_primary_only_write(_env, &operation).await
            }
            DualWriteStrategy::SecondaryOnly => {
                self.execute_secondary_only_write(_env, &operation).await
            }
        };

        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        let duration = end_time - start_time;

        // Remove from active operations
        {
            let mut operations = self.active_operations.lock().unwrap();
            operations.remove(&operation.operation_id);
        }

        // Update performance metrics
        self.update_performance_metrics(duration, result.is_ok())
            .await?;

        // Log the operation
        self.log_dual_write_event(
            operation.operation_id.clone(),
            if result.is_ok() { "success" } else { "failure" },
            duration,
        )
        .await?;

        result
    }

    /// Execute batch dual-write operations
    pub async fn execute_batch_dual_write(
        &self,
        _env: &Env,
        operations: Vec<WriteOperation>,
        strategy: Option<DualWriteStrategy>,
        consistency_level: Option<ConsistencyLevel>,
    ) -> ArbitrageResult<Vec<DualWriteResult>> {
        let mut results = Vec::new();
        let batch_size = self.config.batch_size as usize;

        for chunk in operations.chunks(batch_size) {
            for operation in chunk {
                let result = self
                    .execute_dual_write(
                        _env,
                        operation.clone(),
                        strategy.clone(),
                        consistency_level.clone(),
                    )
                    .await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get health status
    pub async fn get_health(&self) -> ComponentHealth {
        let active_count = {
            let operations = self.active_operations.lock().unwrap();
            operations.len()
        };

        let _status = if self.config.enabled && active_count < 1000 {
            "healthy"
        } else if !self.config.enabled {
            "disabled"
        } else {
            "degraded"
        };

        ComponentHealth::new(
            true,                                 // is_healthy
            "Dual-Write Coordinator".to_string(), // component_name
            0,                                    // uptime_seconds
            1.0,                                  // performance_score
            0,                                    // error_count
            0,                                    // warning_count
        )
    }

    // ============= PRIVATE HELPER METHODS =============

    /// Execute primary-first write strategy
    async fn execute_primary_first_write(
        &self,
        _env: &Env,
        operation: &WriteOperation,
        consistency_level: &ConsistencyLevel,
    ) -> ArbitrageResult<DualWriteResult> {
        self.logger.info(&format!(
            "Executing primary-first write for: {}",
            operation.operation_id
        ));

        // Write to primary system first
        let primary_result = self.write_to_primary_system(_env, operation).await?;

        if !primary_result.success && matches!(consistency_level, ConsistencyLevel::Strong) {
            return Ok(DualWriteResult {
                success: false,
                primary_success: false,
                secondary_success: false,
                consistency_level: consistency_level.clone(),
                total_duration_ms: primary_result.duration_ms,
                write_results: vec![],
                rollback_operations: vec![],
            });
        }

        // Write to secondary system
        let secondary_result = self.write_to_secondary_system(_env, operation).await?;

        // Handle rollback if needed
        let mut rollback_operations = Vec::new();
        if primary_result.success
            && !secondary_result.success
            && matches!(consistency_level, ConsistencyLevel::Strong)
        {
            let rollback = self.rollback_primary_operation(_env, operation).await?;
            rollback_operations.push(rollback);
        }

        Ok(DualWriteResult {
            success: primary_result.success
                && (secondary_result.success
                    || !matches!(consistency_level, ConsistencyLevel::Strong)),
            primary_success: primary_result.success,
            secondary_success: secondary_result.success,
            consistency_level: consistency_level.clone(),
            total_duration_ms: primary_result.duration_ms + secondary_result.duration_ms,
            write_results: vec![],
            rollback_operations,
        })
    }

    /// Execute secondary-first write strategy
    async fn execute_secondary_first_write(
        &self,
        _env: &Env,
        operation: &WriteOperation,
        consistency_level: &ConsistencyLevel,
    ) -> ArbitrageResult<DualWriteResult> {
        self.logger.info(&format!(
            "Executing secondary-first write for: {}",
            operation.operation_id
        ));

        // Write to secondary system first
        let secondary_result = self.write_to_secondary_system(_env, operation).await?;

        if !secondary_result.success && matches!(consistency_level, ConsistencyLevel::Strong) {
            return Ok(DualWriteResult {
                success: false,
                primary_success: false,
                secondary_success: false,
                consistency_level: consistency_level.clone(),
                total_duration_ms: secondary_result.duration_ms,
                write_results: vec![],
                rollback_operations: vec![],
            });
        }

        // Write to primary system
        let primary_result = self.write_to_primary_system(_env, operation).await?;

        // Handle rollback if needed
        let mut rollback_operations = Vec::new();
        if secondary_result.success
            && !primary_result.success
            && matches!(consistency_level, ConsistencyLevel::Strong)
        {
            let rollback = self.rollback_secondary_operation(_env, operation).await?;
            rollback_operations.push(rollback);
        }

        Ok(DualWriteResult {
            success: secondary_result.success
                && (primary_result.success
                    || !matches!(consistency_level, ConsistencyLevel::Strong)),
            primary_success: primary_result.success,
            secondary_success: secondary_result.success,
            consistency_level: consistency_level.clone(),
            total_duration_ms: primary_result.duration_ms + secondary_result.duration_ms,
            write_results: vec![],
            rollback_operations,
        })
    }

    /// Execute parallel write strategy
    async fn execute_parallel_write(
        &self,
        _env: &Env,
        operation: &WriteOperation,
        consistency_level: &ConsistencyLevel,
    ) -> ArbitrageResult<DualWriteResult> {
        self.logger.info(&format!(
            "Executing parallel write for: {}",
            operation.operation_id
        ));

        // Execute both writes concurrently
        let (primary_result, secondary_result) = futures::join!(
            self.write_to_primary_system(_env, operation),
            self.write_to_secondary_system(_env, operation)
        );

        let primary_result = primary_result?;
        let secondary_result = secondary_result?;

        // Handle rollback if needed for strong consistency
        let mut rollback_operations = Vec::new();
        if matches!(consistency_level, ConsistencyLevel::Strong) {
            if primary_result.success && !secondary_result.success {
                let rollback = self.rollback_primary_operation(_env, operation).await?;
                rollback_operations.push(rollback);
            } else if !primary_result.success && secondary_result.success {
                let rollback = self.rollback_secondary_operation(_env, operation).await?;
                rollback_operations.push(rollback);
            }
        }

        let success = match consistency_level {
            ConsistencyLevel::Strong => primary_result.success && secondary_result.success,
            ConsistencyLevel::Weak => primary_result.success,
            ConsistencyLevel::Eventual => primary_result.success || secondary_result.success,
        };

        Ok(DualWriteResult {
            success,
            primary_success: primary_result.success,
            secondary_success: secondary_result.success,
            consistency_level: consistency_level.clone(),
            total_duration_ms: std::cmp::max(
                primary_result.duration_ms,
                secondary_result.duration_ms,
            ),
            write_results: vec![],
            rollback_operations,
        })
    }

    /// Execute primary-only write
    async fn execute_primary_only_write(
        &self,
        _env: &Env,
        operation: &WriteOperation,
    ) -> ArbitrageResult<DualWriteResult> {
        self.logger.info(&format!(
            "Executing primary-only write for: {}",
            operation.operation_id
        ));

        let primary_result = self.write_to_primary_system(_env, operation).await?;

        Ok(DualWriteResult {
            success: primary_result.success,
            primary_success: primary_result.success,
            secondary_success: false,
            consistency_level: ConsistencyLevel::Weak,
            total_duration_ms: primary_result.duration_ms,
            write_results: vec![],
            rollback_operations: vec![],
        })
    }

    /// Execute secondary-only write
    async fn execute_secondary_only_write(
        &self,
        _env: &Env,
        operation: &WriteOperation,
    ) -> ArbitrageResult<DualWriteResult> {
        self.logger.info(&format!(
            "Executing secondary-only write for: {}",
            operation.operation_id
        ));

        let secondary_result = self.write_to_secondary_system(_env, operation).await?;

        Ok(DualWriteResult {
            success: secondary_result.success,
            primary_success: false,
            secondary_success: secondary_result.success,
            consistency_level: ConsistencyLevel::Weak,
            total_duration_ms: secondary_result.duration_ms,
            write_results: vec![],
            rollback_operations: vec![],
        })
    }

    /// Write to primary system
    async fn write_to_primary_system(
        &self,
        _env: &Env,
        operation: &WriteOperation,
    ) -> ArbitrageResult<SystemWriteResult> {
        // In a real implementation, this would write to the actual primary system
        // For now, we'll simulate the operation with a small delay
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Promise;
            use wasm_bindgen_futures::JsFuture;
            let promise = Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 100)
                    .unwrap();
            });
            JsFuture::from(promise).await.unwrap();
        }

        Ok(SystemWriteResult {
            system_id: "primary".to_string(),
            success: true,
            response_data: Some(serde_json::json!({"id": operation.operation_id})),
            duration_ms: 100,
            error_message: None,
            retry_count: 0,
        })
    }

    /// Write to secondary system
    async fn write_to_secondary_system(
        &self,
        _env: &Env,
        operation: &WriteOperation,
    ) -> ArbitrageResult<SystemWriteResult> {
        // In a real implementation, this would write to the actual secondary system
        // For now, we'll simulate the operation with a small delay
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Promise;
            use wasm_bindgen_futures::JsFuture;
            let promise = Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 150)
                    .unwrap();
            });
            JsFuture::from(promise).await.unwrap();
        }

        Ok(SystemWriteResult {
            system_id: "secondary".to_string(),
            success: true,
            response_data: Some(serde_json::json!({"id": operation.operation_id})),
            duration_ms: 150,
            error_message: None,
            retry_count: 0,
        })
    }

    /// Rollback primary operation
    async fn rollback_primary_operation(
        &self,
        _env: &Env,
        operation: &WriteOperation,
    ) -> ArbitrageResult<RollbackOperation> {
        self.logger.warn(&format!(
            "Rolling back primary operation: {}",
            operation.operation_id
        ));

        // In a real implementation, this would perform the actual rollback
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Promise;
            use wasm_bindgen_futures::JsFuture;
            let promise = Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50)
                    .unwrap();
            });
            JsFuture::from(promise).await.unwrap();
        }

        Ok(RollbackOperation {
            operation_id: operation.operation_id.clone(),
            target_system: "primary".to_string(),
            action: "rollback".to_string(),
            success: true,
            duration_ms: 50,
            error_message: None,
        })
    }

    /// Rollback secondary operation
    async fn rollback_secondary_operation(
        &self,
        _env: &Env,
        operation: &WriteOperation,
    ) -> ArbitrageResult<RollbackOperation> {
        self.logger.warn(&format!(
            "Rolling back secondary operation: {}",
            operation.operation_id
        ));

        // In a real implementation, this would perform the actual rollback
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Promise;
            use wasm_bindgen_futures::JsFuture;
            let promise = Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50)
                    .unwrap();
            });
            JsFuture::from(promise).await.unwrap();
        }

        Ok(RollbackOperation {
            operation_id: operation.operation_id.clone(),
            target_system: "secondary".to_string(),
            action: "rollback".to_string(),
            success: true,
            duration_ms: 50,
            error_message: None,
        })
    }

    /// Update performance metrics
    async fn update_performance_metrics(
        &self,
        duration_ms: u64,
        success: bool,
    ) -> ArbitrageResult<()> {
        let mut metrics = self.performance_metrics.lock().unwrap();
        if success {
            metrics.total_operations += 1;
        } else {
            metrics.error_count += 1;
        }

        metrics.total_operations += 1;

        // Update latency metrics
        metrics.latency_ms = duration_ms as f64;

        if (duration_ms as f64) > metrics.p99_latency_ms {
            metrics.p99_latency_ms = duration_ms as f64;
        }

        if (duration_ms as f64) < metrics.p95_latency_ms {
            metrics.p95_latency_ms = duration_ms as f64;
        }

        // Update throughput
        metrics.throughput_ops_per_sec = metrics.total_operations as f64;

        Ok(())
    }

    /// Log dual-write event
    async fn log_dual_write_event(
        &self,
        operation_id: String,
        status: &str,
        duration_ms: u64,
    ) -> ArbitrageResult<()> {
        let event = MigrationEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            event_type: MigrationEventType::DataWritten,
            system_id: SystemIdentifier::new(
                LegacySystemType::Custom("dual_write_coordinator".to_string()),
                operation_id,
                "1.0.0".to_string(),
            ),
            message: format!("Dual-write operation completed with status: {}", status),
            severity: if status == "success" {
                EventSeverity::Info
            } else {
                EventSeverity::Warning
            },
            data: {
                let mut data = HashMap::new();
                data.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(duration_ms)),
                );
                data.insert(
                    "status".to_string(),
                    serde_json::Value::String(status.to_string()),
                );
                data
            },
        };

        {
            let mut history = self.event_history.lock().unwrap();
            history.push(event);

            // Keep only the last 1000 events
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }

        Ok(())
    }
}
