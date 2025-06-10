//! Legacy Adapter Layer
//!
//! Provides seamless integration between legacy components and new modular architecture,
//! ensuring backward compatibility and smooth transition during migration phases.

use super::shared_types::{LegacySystemType, MigrationEvent, SystemIdentifier};
use crate::services::core::infrastructure::shared_types::{
    CircuitBreaker, CircuitBreakerState, ComponentHealth,
};
use crate::utils::ArbitrageError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Adapter configuration for mapping legacy systems
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterConfig {
    /// Enable adapter layer
    pub enabled: bool,
    /// Legacy system mappings
    pub system_mappings: HashMap<String, ServiceMapping>,
    /// Request transformation rules
    pub request_transformations: Vec<RequestTransformation>,
    /// Response transformation rules
    pub response_transformations: Vec<ResponseTransformation>,
    /// Error handling configuration
    pub error_handling_config: ErrorHandlingConfig,
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
    /// Health check configuration
    pub health_check_config: HealthCheckConfig,
    /// Circuit breaker settings
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            system_mappings: HashMap::new(),
            request_transformations: Vec::new(),
            response_transformations: Vec::new(),
            error_handling_config: ErrorHandlingConfig::default(),
            performance_thresholds: PerformanceThresholds::default(),
            health_check_config: HealthCheckConfig::default(),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 60,
        }
    }
}

/// Service mapping configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceMapping {
    /// Legacy service identifier
    pub legacy_service_id: String,
    /// New service identifier
    pub new_service_id: String,
    /// Service type
    pub service_type: LegacySystemType,
    /// Mapping strategy
    pub mapping_strategy: MappingStrategy,
    /// Priority (higher values take precedence)
    pub priority: u32,
    /// Enable fallback to legacy system
    pub enable_fallback: bool,
    /// Service-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Mapping strategies for service adaptation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MappingStrategy {
    /// Direct one-to-one mapping
    Direct,
    /// Aggregate multiple legacy calls into one new call
    Aggregate,
    /// Split one legacy call into multiple new calls
    Split,
    /// Transform data structure between systems
    Transform,
    /// Route based on conditions
    Conditional,
    /// Proxy with minimal changes
    Proxy,
}

/// Request transformation configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestTransformation {
    /// Transformation ID
    pub id: String,
    /// Source service pattern
    pub source_pattern: String,
    /// Target service pattern
    pub target_pattern: String,
    /// Transformation type
    pub transformation_type: TransformationType,
    /// Field mappings
    pub field_mappings: HashMap<String, String>,
    /// Custom transformation logic
    pub custom_logic: Option<String>,
    /// Priority
    pub priority: u32,
}

/// Response transformation configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseTransformation {
    /// Transformation ID
    pub id: String,
    /// Source service pattern
    pub source_pattern: String,
    /// Target service pattern
    pub target_pattern: String,
    /// Transformation type
    pub transformation_type: TransformationType,
    /// Field mappings
    pub field_mappings: HashMap<String, String>,
    /// Custom transformation logic
    pub custom_logic: Option<String>,
    /// Priority
    pub priority: u32,
}

/// Transformation types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransformationType {
    /// Field renaming and restructuring
    FieldMapping,
    /// Data type conversions
    TypeConversion,
    /// Format transformations (JSON <-> XML, etc.)
    FormatConversion,
    /// Protocol adaptations (REST <-> GraphQL, etc.)
    ProtocolAdaptation,
    /// Custom transformation logic
    Custom,
}

/// Error handling configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorHandlingConfig {
    /// Enable error handling
    pub enabled: bool,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Fallback strategies
    pub fallback_strategies: Vec<FallbackStrategy>,
    /// Error mapping rules
    pub error_mappings: HashMap<String, String>,
    /// Circuit breaker on error threshold
    pub circuit_breaker_on_errors: bool,
    /// Error threshold percentage
    pub error_threshold_percent: f64,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retry_config: RetryConfig::default(),
            fallback_strategies: Vec::new(),
            error_mappings: HashMap::new(),
            circuit_breaker_on_errors: true,
            error_threshold_percent: 10.0,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter enabled
    pub enable_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            enable_jitter: true,
        }
    }
}

/// Fallback strategies
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FallbackStrategy {
    /// Use legacy system as fallback
    UseLegacySystem,
    /// Use cached response
    UseCachedResponse,
    /// Return default response
    ReturnDefaultResponse,
    /// Graceful degradation
    GracefulDegradation,
    /// Custom fallback logic
    Custom(String),
}

/// Performance thresholds
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Maximum memory usage in MB
    pub max_memory_usage_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_usage_percent: f64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Request rate limit per second
    pub rate_limit_per_second: u32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000,
            max_memory_usage_mb: 256,
            max_cpu_usage_percent: 80.0,
            max_concurrent_requests: 100,
            rate_limit_per_second: 1000,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval in seconds
    pub interval_seconds: u64,
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
    /// Health endpoints to check
    pub endpoints: Vec<String>,
    /// Unhealthy threshold
    pub unhealthy_threshold: u32,
    /// Healthy threshold
    pub healthy_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            timeout_seconds: 5,
            endpoints: Vec::new(),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        }
    }
}

/// Request translation result
#[derive(Debug, Clone)]
pub struct RequestTranslation {
    /// Translated request data
    pub translated_request: serde_json::Value,
    /// Target service identifier
    pub target_service: String,
    /// Translation metadata
    pub metadata: HashMap<String, String>,
    /// Translation timestamp
    pub timestamp: u64,
}

/// Response translation result
#[derive(Debug, Clone)]
pub struct ResponseTranslation {
    /// Translated response data
    pub translated_response: serde_json::Value,
    /// Source service identifier
    pub source_service: String,
    /// Translation metadata
    pub metadata: HashMap<String, String>,
    /// Translation timestamp
    pub timestamp: u64,
}

/// Compatibility layer for legacy integration
#[derive(Debug, Clone, Default)]
pub struct CompatibilityLayer {
    /// API version compatibility
    pub api_versions: HashMap<String, String>,
    /// Data format compatibility
    pub data_formats: HashMap<String, String>,
    /// Protocol compatibility
    pub protocols: HashMap<String, String>,
    /// Error code mappings
    pub error_codes: HashMap<String, String>,
}

/// Adapter metrics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful adaptations
    pub successful_adaptations: u64,
    /// Failed adaptations
    pub failed_adaptations: u64,
    /// Fallback activations
    pub fallback_activations: u64,
    /// Average adaptation time in milliseconds
    pub avg_adaptation_time_ms: f64,
    /// Requests by service type
    pub requests_by_service: HashMap<String, u64>,
    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,
    /// Last updated timestamp
    pub last_updated: u64,
}

impl Default for AdapterMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_adaptations: 0,
            failed_adaptations: 0,
            fallback_activations: 0,
            avg_adaptation_time_ms: 0.0,
            requests_by_service: HashMap::new(),
            errors_by_type: HashMap::new(),
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Legacy Adapter Layer main implementation
pub struct LegacyAdapterLayer {
    config: AdapterConfig,
    circuit_breakers: Arc<Mutex<HashMap<String, CircuitBreaker>>>,
    service_mappings: Arc<Mutex<HashMap<String, ServiceMapping>>>,
    compatibility_layer: Arc<Mutex<CompatibilityLayer>>,
    adapter_metrics: Arc<Mutex<AdapterMetrics>>,
    health_status: Arc<Mutex<ComponentHealth>>,
    migration_events: Arc<Mutex<Vec<MigrationEvent>>>,
    #[allow(dead_code)]
    start_time: Instant,
}

impl LegacyAdapterLayer {
    /// Create new Legacy Adapter Layer
    pub fn new(config: AdapterConfig) -> Result<Self, ArbitrageError> {
        let mut circuit_breakers = HashMap::new();
        let mut service_mappings = HashMap::new();

        // Initialize circuit breakers for each service
        for (service_id, mapping) in &config.system_mappings {
            let cb = CircuitBreaker::new(
                config.circuit_breaker_threshold,
                config.circuit_breaker_timeout_seconds,
            );
            circuit_breakers.insert(service_id.clone(), cb);
            service_mappings.insert(service_id.clone(), mapping.clone());
        }

        Ok(Self {
            config,
            circuit_breakers: Arc::new(Mutex::new(circuit_breakers)),
            service_mappings: Arc::new(Mutex::new(service_mappings)),
            compatibility_layer: Arc::new(Mutex::new(CompatibilityLayer::default())),
            adapter_metrics: Arc::new(Mutex::new(AdapterMetrics::default())),
            health_status: Arc::new(Mutex::new(ComponentHealth::new(
                true,                             // is_healthy
                "LegacyAdapterLayer".to_string(), // component_name
                0,                                // uptime_seconds
                1.0,                              // performance_score
                0,                                // error_count
                0,                                // warning_count
            ))),
            migration_events: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        })
    }

    /// Translate incoming request from legacy system
    pub async fn translate_request(
        &self,
        source_service: &str,
        request_data: serde_json::Value,
    ) -> Result<RequestTranslation, ArbitrageError> {
        let _start_time = Instant::now();

        // Find matching service mapping
        let service_mapping = {
            let mappings = self.service_mappings.lock().unwrap();
            mappings.get(source_service).cloned()
        };

        let mapping = service_mapping.ok_or_else(|| {
            ArbitrageError::infrastructure_error(format!(
                "No service mapping found for: {}",
                source_service
            ))
        })?;

        // Check circuit breaker
        let circuit_breaker_open = {
            let circuit_breakers = self.circuit_breakers.lock().unwrap();
            if let Some(cb) = circuit_breakers.get(source_service) {
                cb.state != crate::services::core::infrastructure::shared_types::CircuitBreakerState::Closed
            } else {
                false
            }
        };

        if circuit_breaker_open {
            return Err(ArbitrageError::infrastructure_error(format!(
                "Circuit breaker open for service: {}",
                source_service
            )));
        }

        // Apply request transformations
        let translated_request = self
            .apply_request_transformations(&mapping, request_data)
            .await?;

        // Update metrics
        {
            let mut metrics = self.adapter_metrics.lock().unwrap();
            metrics.total_requests += 1;
            metrics.successful_adaptations += 1;

            let duration_ms = _start_time.elapsed().as_millis() as f64;
            metrics.avg_adaptation_time_ms = (metrics.avg_adaptation_time_ms
                * (metrics.total_requests - 1) as f64
                + duration_ms)
                / metrics.total_requests as f64;

            *metrics
                .requests_by_service
                .entry(source_service.to_string())
                .or_insert(0) += 1;
            metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
        }

        // Record circuit breaker success
        {
            let mut circuit_breakers = self.circuit_breakers.lock().unwrap();
            if let Some(cb) = circuit_breakers.get_mut(source_service) {
                cb.record_success();
            }
        }

        Ok(RequestTranslation {
            translated_request,
            target_service: mapping.new_service_id,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Translate outgoing response to legacy system
    pub async fn translate_response(
        &self,
        target_service: &str,
        response_data: serde_json::Value,
    ) -> Result<ResponseTranslation, ArbitrageError> {
        let _start_time = Instant::now();

        // Find matching service mapping (reverse lookup)
        let service_mapping = {
            let mappings = self.service_mappings.lock().unwrap();
            mappings
                .values()
                .find(|mapping| mapping.new_service_id == target_service)
                .cloned()
        };

        let mapping = service_mapping.ok_or_else(|| {
            ArbitrageError::infrastructure_error(format!(
                "No service mapping found for target: {}",
                target_service
            ))
        })?;

        // Apply response transformations
        let translated_response = self
            .apply_response_transformations(&mapping, response_data)
            .await?;

        Ok(ResponseTranslation {
            translated_response,
            source_service: mapping.legacy_service_id,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Apply request transformations based on mapping strategy
    async fn apply_request_transformations(
        &self,
        mapping: &ServiceMapping,
        request_data: serde_json::Value,
    ) -> Result<serde_json::Value, ArbitrageError> {
        match mapping.mapping_strategy {
            MappingStrategy::Direct => Ok(request_data),
            MappingStrategy::Proxy => Ok(request_data),
            MappingStrategy::Transform => {
                // Apply field mappings and transformations
                self.transform_data(request_data, &self.config.request_transformations)
                    .await
            }
            MappingStrategy::Aggregate => {
                // For now, return as-is - would need specific aggregation logic
                Ok(request_data)
            }
            MappingStrategy::Split => {
                // For now, return as-is - would need specific splitting logic
                Ok(request_data)
            }
            MappingStrategy::Conditional => {
                // Apply conditional transformation logic
                Ok(request_data)
            }
        }
    }

    /// Apply response transformations based on mapping strategy
    async fn apply_response_transformations(
        &self,
        mapping: &ServiceMapping,
        response_data: serde_json::Value,
    ) -> Result<serde_json::Value, ArbitrageError> {
        match mapping.mapping_strategy {
            MappingStrategy::Direct => Ok(response_data),
            MappingStrategy::Proxy => Ok(response_data),
            MappingStrategy::Transform => {
                // Apply field mappings and transformations
                self.transform_response_data(response_data, &self.config.response_transformations)
                    .await
            }
            MappingStrategy::Aggregate => {
                // For now, return as-is - would need specific aggregation logic
                Ok(response_data)
            }
            MappingStrategy::Split => {
                // For now, return as-is - would need specific splitting logic
                Ok(response_data)
            }
            MappingStrategy::Conditional => {
                // Apply conditional transformation logic
                Ok(response_data)
            }
        }
    }

    /// Transform data using transformation rules
    async fn transform_data(
        &self,
        mut data: serde_json::Value,
        transformations: &[RequestTransformation],
    ) -> Result<serde_json::Value, ArbitrageError> {
        // Sort transformations by priority
        let mut sorted_transformations = transformations.to_vec();
        sorted_transformations.sort_by_key(|t| t.priority);

        for transformation in sorted_transformations {
            // Apply field mappings
            for (source_field, target_field) in &transformation.field_mappings {
                if let Some(value) = data.get(source_field).cloned() {
                    // Remove source field and add target field
                    if let serde_json::Value::Object(ref mut map) = data {
                        map.remove(source_field);
                        map.insert(target_field.clone(), value);
                    }
                }
            }

            // Apply custom transformation logic if available
            if let Some(_custom_logic) = &transformation.custom_logic {
                // TODO: Implement custom transformation logic
                // This would involve parsing and executing custom transformation rules
            }
        }

        Ok(data)
    }

    async fn transform_response_data(
        &self,
        mut data: serde_json::Value,
        transformations: &[ResponseTransformation],
    ) -> Result<serde_json::Value, ArbitrageError> {
        // Sort transformations by priority
        let mut sorted_transformations = transformations.to_vec();
        sorted_transformations.sort_by_key(|t| t.priority);

        for transformation in sorted_transformations {
            // Apply field mappings
            for (source_field, target_field) in &transformation.field_mappings {
                if let Some(value) = data.get(source_field).cloned() {
                    // Remove source field and add target field
                    if let serde_json::Value::Object(ref mut map) = data {
                        map.remove(source_field);
                        map.insert(target_field.clone(), value);
                    }
                }
            }

            // Apply custom transformation logic if available
            if let Some(_custom_logic) = &transformation.custom_logic {
                // TODO: Implement custom transformation logic
                // This would involve parsing and executing custom transformation rules
            }
        }

        for transformation in transformations {
            match transformation.transformation_type {
                TransformationType::FieldMapping => {
                    if let serde_json::Value::Object(ref mut obj) = data {
                        for (source_field, target_field) in &transformation.field_mappings {
                            if let Some(value) = obj.remove(source_field) {
                                obj.insert(target_field.clone(), value);
                            }
                        }
                    }
                }
                TransformationType::TypeConversion => {
                    // Apply type conversions as needed
                    // For now, keep data as-is
                }
                TransformationType::FormatConversion => {
                    // Apply format conversions as needed
                    // For now, keep data as-is
                }
                TransformationType::ProtocolAdaptation => {
                    // Apply protocol adaptations as needed
                    // For now, keep data as-is
                }
                TransformationType::Custom => {
                    // Apply custom transformation logic
                    // For now, keep data as-is
                }
            }
        }

        Ok(data)
    }

    /// Handle adaptation errors with fallback strategies
    pub async fn handle_adaptation_error(
        &self,
        service_id: &str,
        error: &ArbitrageError,
    ) -> Result<serde_json::Value, ArbitrageError> {
        // Record circuit breaker failure
        {
            let mut circuit_breakers = self.circuit_breakers.lock().unwrap();
            if let Some(cb) = circuit_breakers.get_mut(service_id) {
                cb.record_failure();
            }
        }

        // Update error metrics
        {
            let mut metrics = self.adapter_metrics.lock().unwrap();
            metrics.failed_adaptations += 1;
            *metrics.errors_by_type.entry(error.to_string()).or_insert(0) += 1;
        }

        // Apply fallback strategies
        if let Some(strategy) = self
            .config
            .error_handling_config
            .fallback_strategies
            .first()
        {
            match strategy {
                FallbackStrategy::UseLegacySystem => {
                    // Increment fallback counter
                    {
                        let mut metrics = self.adapter_metrics.lock().unwrap();
                        metrics.fallback_activations += 1;
                    }

                    // Return indication to use legacy system
                    return Ok(serde_json::json!({
                        "fallback": true,
                        "strategy": "use_legacy_system",
                        "reason": error.to_string()
                    }));
                }
                FallbackStrategy::UseCachedResponse => {
                    // Would implement cache lookup here
                    return Ok(serde_json::json!({
                        "fallback": true,
                        "strategy": "use_cached_response",
                        "reason": error.to_string()
                    }));
                }
                FallbackStrategy::ReturnDefaultResponse => {
                    return Ok(serde_json::json!({
                        "fallback": true,
                        "strategy": "default_response",
                        "reason": error.to_string()
                    }));
                }
                FallbackStrategy::GracefulDegradation => {
                    return Ok(serde_json::json!({
                        "fallback": true,
                        "strategy": "graceful_degradation",
                        "reason": error.to_string()
                    }));
                }
                FallbackStrategy::Custom(_logic) => {
                    // Would implement custom logic here
                    return Ok(serde_json::json!({
                        "fallback": true,
                        "strategy": "custom",
                        "reason": error.to_string()
                    }));
                }
            }
        }

        // If no fallback strategy worked, return the original error
        Err(error.clone())
    }

    /// Add new service mapping
    pub async fn add_service_mapping(
        &self,
        service_id: String,
        mapping: ServiceMapping,
    ) -> Result<(), ArbitrageError> {
        // Create circuit breaker for new service
        let cb = CircuitBreaker::new(
            self.config.circuit_breaker_threshold,
            self.config.circuit_breaker_timeout_seconds,
        );

        {
            let mut circuit_breakers = self.circuit_breakers.lock().unwrap();
            circuit_breakers.insert(service_id.clone(), cb);
        }

        {
            let mut mappings = self.service_mappings.lock().unwrap();
            mappings.insert(service_id.clone(), mapping);
        }

        // Record event
        self.record_migration_event(MigrationEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: crate::services::core::infrastructure::legacy_system_integration::shared_types::MigrationEventType::FeatureFlagToggled,
            system_id: SystemIdentifier::new(
                LegacySystemType::Custom("legacy_adapter_layer".to_string()),
                "legacy_adapter_layer".to_string(),
                "1.0.0".to_string(),
            ),
            message: format!("Added service mapping: {}", service_id),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            severity: crate::services::core::infrastructure::legacy_system_integration::shared_types::EventSeverity::Info,
            data: HashMap::new(),
        }).await?;

        Ok(())
    }

    /// Remove service mapping
    pub async fn remove_service_mapping(&self, service_id: &str) -> Result<(), ArbitrageError> {
        {
            let mut circuit_breakers = self.circuit_breakers.lock().unwrap();
            circuit_breakers.remove(service_id);
        }

        {
            let mut mappings = self.service_mappings.lock().unwrap();
            mappings.remove(service_id);
        }

        // Record event
        self.record_migration_event(MigrationEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: crate::services::core::infrastructure::legacy_system_integration::shared_types::MigrationEventType::FeatureFlagToggled,
            system_id: SystemIdentifier::new(
                LegacySystemType::Custom("legacy_adapter_layer".to_string()),
                "legacy_adapter_layer".to_string(),
                "1.0.0".to_string(),
            ),
            message: format!("Removed service mapping: {}", service_id),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            severity: crate::services::core::infrastructure::legacy_system_integration::shared_types::EventSeverity::Info,
            data: HashMap::new(),
        }).await?;

        Ok(())
    }

    /// Get adapter metrics
    pub fn get_metrics(&self) -> AdapterMetrics {
        let metrics = self.adapter_metrics.lock().unwrap();
        metrics.clone()
    }

    /// Get system health
    pub async fn get_health(&self) -> ComponentHealth {
        let circuit_breaker_health = {
            let circuit_breakers = self.circuit_breakers.lock().unwrap();
            circuit_breakers
                .values()
                .all(|cb| cb.state == CircuitBreakerState::Closed)
        };

        let metrics = self.get_metrics();
        let error_rate = if metrics.total_requests > 0 {
            (metrics.failed_adaptations as f64 / metrics.total_requests as f64) * 100.0
        } else {
            0.0
        };

        let overall_healthy = circuit_breaker_health
            && error_rate < self.config.error_handling_config.error_threshold_percent;

        let mut health = self.health_status.lock().unwrap().clone();

        health.is_healthy = overall_healthy;
        health.last_check = chrono::Utc::now().timestamp_millis() as u64;

        // Clean up old events (keep last 1000)
        {
            let mut events = self.migration_events.lock().unwrap();
            let len = events.len();
            if len > 1000 {
                events.drain(0..len - 1000);
            }
        }

        health
    }

    /// Record migration event
    async fn record_migration_event(&self, event: MigrationEvent) -> Result<(), ArbitrageError> {
        let mut events = self.migration_events.lock().unwrap();
        events.push(event);

        // Keep only last 1000 events
        if events.len() > 1000 {
            let len = events.len();
            if len > 1000 {
                events.drain(0..len - 1000);
            }
        }

        Ok(())
    }

    /// Get migration events
    pub fn get_migration_events(&self) -> Vec<MigrationEvent> {
        let events = self.migration_events.lock().unwrap();
        events.clone()
    }

    /// Update compatibility layer
    pub async fn update_compatibility_layer(
        &self,
        compatibility: CompatibilityLayer,
    ) -> Result<(), ArbitrageError> {
        {
            let mut layer = self.compatibility_layer.lock().unwrap();
            *layer = compatibility;
        }

        // Record event
        self.record_migration_event(MigrationEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: crate::services::core::infrastructure::legacy_system_integration::shared_types::MigrationEventType::MigrationStarted,
            system_id: SystemIdentifier::new(
                LegacySystemType::Custom("legacy_adapter_layer".to_string()),
                "legacy_adapter_layer".to_string(),
                "1.0.0".to_string(),
            ),
            message: "Updated compatibility layer configuration".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            severity: crate::services::core::infrastructure::legacy_system_integration::shared_types::EventSeverity::Info,
            data: HashMap::new(),
        }).await?;

        Ok(())
    }

    /// Validate adapter configuration
    pub fn validate_config(&self) -> Result<(), ArbitrageError> {
        if !self.config.enabled {
            return Ok(());
        }

        if self.config.system_mappings.is_empty() {
            return Err(ArbitrageError::infrastructure_error(
                "No service mappings configured",
            ));
        }

        // Validate performance thresholds
        let thresholds = &self.config.performance_thresholds;
        if thresholds.max_response_time_ms == 0 {
            return Err(ArbitrageError::infrastructure_error(
                "Invalid max response time",
            ));
        }

        if thresholds.max_concurrent_requests == 0 {
            return Err(ArbitrageError::infrastructure_error(
                "Invalid max concurrent requests",
            ));
        }

        Ok(())
    }
}

impl Default for LegacyAdapterLayer {
    fn default() -> Self {
        Self::new(AdapterConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_config_creation() {
        let config = AdapterConfig::default();
        assert!(config.enabled);
        assert!(config.system_mappings.is_empty());
    }

    #[test]
    fn test_service_mapping_creation() {
        let mapping = ServiceMapping {
            legacy_service_id: "legacy_telegram".to_string(),
            new_service_id: "new_telegram".to_string(),
            service_type: LegacySystemType::TelegramService,
            mapping_strategy: MappingStrategy::Direct,
            priority: 1,
            enable_fallback: true,
            config: HashMap::new(),
        };

        assert_eq!(mapping.legacy_service_id, "legacy_telegram");
        assert_eq!(mapping.new_service_id, "new_telegram");
    }

    #[tokio::test]
    async fn test_adapter_layer_creation() {
        let config = AdapterConfig::default();
        let adapter = LegacyAdapterLayer::new(config);
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::default();
        let adapter = LegacyAdapterLayer::new(config).unwrap();

        let health = adapter.get_health().await;
        assert_eq!(health.component_name, "LegacyAdapterLayer");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AdapterConfig::default();
        config.performance_thresholds.max_response_time_ms = 0;

        let adapter = LegacyAdapterLayer::new(config).unwrap();
        let result = adapter.validate_config();
        assert!(result.is_err());
    }
}
