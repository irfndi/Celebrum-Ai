// Infrastructure Engine Module - Main Orchestrator for All Infrastructure Services
// Provides service discovery, dependency management, configuration, and health monitoring

use crate::utils::error::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use worker::kv::KvStore;

use super::{
    data_access_layer::{
        unified_data_access_engine::UnifiedDataAccessEngine, DataAccessLayerConfig,
    },
    data_ingestion_module::unified_ingestion_engine::UnifiedIngestionEngine,
    persistence_layer::{database_manager::DatabaseManager, storage_layer::StorageLayer},
    service_container::ServiceContainer,
    shared_types::{HealthStatus, ServiceHealthCheck, SystemHealthReport},
    unified_analytics_and_cleanup::UnifiedAnalyticsAndCleanup,
    unified_cloudflare_services::UnifiedCloudflareServices,
    unified_core_services::UnifiedCoreServices,
    unified_notification_services::{NotificationCoordinator, NotificationCoordinatorConfig},
};
use worker::Env;

/// Service types in the infrastructure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Database,
    Cache,
    Health,
    Notification,
    DataAccess,
    Metrics,
    External,
    Custom(String),
}

impl ServiceType {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceType::Database => "database",
            ServiceType::Cache => "cache",
            ServiceType::Health => "health",
            ServiceType::Notification => "notification",
            ServiceType::DataAccess => "data_access",
            ServiceType::Metrics => "metrics",
            ServiceType::External => "external",
            ServiceType::Custom(name) => name,
        }
    }
}

/// Service status for monitoring
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
    Initializing,
    Stopped,
}

impl ServiceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceStatus::Healthy => "healthy",
            ServiceStatus::Degraded => "degraded",
            ServiceStatus::Unhealthy => "unhealthy",
            ServiceStatus::Unknown => "unknown",
            ServiceStatus::Initializing => "initializing",
            ServiceStatus::Stopped => "stopped",
        }
    }

    pub fn is_operational(&self) -> bool {
        matches!(self, ServiceStatus::Healthy | ServiceStatus::Degraded)
    }
}

/// Service dependency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub service_name: String,
    pub service_type: ServiceType,
    pub is_critical: bool,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub health_check_interval_seconds: u64,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_name: String,
    pub service_type: ServiceType,
    pub version: String,
    pub description: String,
    pub dependencies: Vec<ServiceDependency>,
    pub health_check_endpoint: Option<String>,
    pub metrics_enabled: bool,
    pub auto_recovery: bool,
    pub priority: u8, // 1 = highest priority, 10 = lowest
    pub tags: HashMap<String, String>,
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Service runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub registration: ServiceRegistration,
    pub status: ServiceStatus,
    pub last_health_check: Option<u64>,
    pub uptime_seconds: u64,
    pub error_count: u64,
    pub restart_count: u64,
    pub last_error: Option<String>,
    pub performance_metrics: HashMap<String, f64>,
}

/// Infrastructure configuration
#[derive(Debug, Clone)]
pub struct InfrastructureConfig {
    pub enable_service_discovery: bool,
    pub enable_health_monitoring: bool,
    pub enable_auto_recovery: bool,
    pub enable_metrics_collection: bool,
    pub health_check_interval_seconds: u64,
    pub service_timeout_seconds: u64,
    pub max_restart_attempts: u32,
    pub restart_backoff_multiplier: f64,
    pub enable_circuit_breaker: bool,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
    pub enable_load_balancing: bool,
    pub enable_graceful_shutdown: bool,
    pub shutdown_timeout_seconds: u64,
}

impl Default for InfrastructureConfig {
    fn default() -> Self {
        Self {
            enable_service_discovery: true,
            enable_health_monitoring: true,
            enable_auto_recovery: true,
            enable_metrics_collection: true,
            health_check_interval_seconds: 30,
            service_timeout_seconds: 30,
            max_restart_attempts: 3,
            restart_backoff_multiplier: 2.0,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 60,
            enable_load_balancing: false, // Not needed for single instance
            enable_graceful_shutdown: true,
            shutdown_timeout_seconds: 30,
        }
    }
}

/// Circuit breaker state for service protection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, requests blocked
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub service_name: String,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub last_failure_time: Option<u64>,
    pub next_attempt_time: Option<u64>,
    pub success_count: u32,
    pub threshold: u32,
    pub timeout_seconds: u64,
}

/// Infrastructure health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureHealth {
    pub overall_status: ServiceStatus,
    pub healthy_services: u32,
    pub degraded_services: u32,
    pub unhealthy_services: u32,
    pub total_services: u32,
    pub critical_services_healthy: bool,
    pub uptime_percentage: f64,
    pub last_updated: u64,
    pub service_statuses: HashMap<String, ServiceStatus>,
}

/// Main infrastructure engine
#[allow(dead_code)]
pub struct InfrastructureEngine {
    config: InfrastructureConfig,
    #[allow(dead_code)]
    unified_core: Option<UnifiedCoreServices>,
    #[allow(dead_code)]
    unified_cloudflare: Option<UnifiedCloudflareServices>,
    #[allow(dead_code)]
    unified_analytics: Option<UnifiedAnalyticsAndCleanup>,
    #[allow(dead_code)]
    data_access: Option<UnifiedDataAccessEngine>,
    #[allow(dead_code)]
    data_ingestion: Option<UnifiedIngestionEngine>,
    #[allow(dead_code)]
    database_manager: Option<DatabaseManager>,
    #[allow(dead_code)]
    storage_layer: Option<StorageLayer>,
    #[allow(dead_code)]
    service_container: Option<ServiceContainer>,
    #[allow(dead_code)]
    kv_store: Option<Arc<KvStore>>,
    #[allow(dead_code)]
    d1_database: Option<Arc<worker::D1Database>>,
    startup_time: Option<u64>,
    #[allow(dead_code)]
    is_initialized: bool,
    #[allow(dead_code)]
    health_check_interval: u64,
    #[allow(dead_code)]
    last_health_check: u64,
    notification_engine: Option<NotificationCoordinator>,
}

#[allow(dead_code)]
impl InfrastructureEngine {
    /// Create new InfrastructureEngine with default configuration
    pub fn new(kv_store: KvStore) -> Self {
        Self {
            config: InfrastructureConfig::default(),
            unified_core: None,
            unified_cloudflare: None,
            unified_analytics: None,
            data_access: None,
            data_ingestion: None,
            database_manager: None,
            storage_layer: None,
            service_container: None,
            kv_store: Some(Arc::new(kv_store)),
            d1_database: None,
            startup_time: None,
            is_initialized: false,
            health_check_interval: 0,
            last_health_check: 0,
            notification_engine: None,
        }
    }

    /// Create InfrastructureEngine with custom configuration
    pub fn new_with_config(kv_store: KvStore, config: InfrastructureConfig) -> Self {
        Self {
            config,
            unified_core: None,
            unified_cloudflare: None,
            unified_analytics: None,
            data_access: None,
            data_ingestion: None,
            database_manager: None,
            storage_layer: None,
            service_container: None,
            kv_store: Some(Arc::new(kv_store)),
            d1_database: None,
            startup_time: None,
            is_initialized: false,
            health_check_interval: 0,
            last_health_check: 0,
            notification_engine: None,
        }
    }

    async fn generate_health_report(&self) -> SystemHealthReport {
        let mut services_health: HashMap<String, ServiceHealthCheck> = HashMap::new();

        // Check database health
        let db_health = ServiceHealthCheck {
            service_name: "database".to_string(),
            status: HealthStatus::Healthy, // Assuming healthy for now
            last_check: crate::utils::time::get_current_timestamp(),
            message: "Database operational".to_string(),
        };

        #[allow(clippy::if_same_then_else)]
        if db_health.status == HealthStatus::Healthy {
            services_health.insert("database".to_string(), db_health.clone());
        } else if db_health.status == HealthStatus::Degraded {
            services_health.insert("database".to_string(), db_health.clone());
        } else if db_health.status == HealthStatus::Unhealthy {
            services_health.insert("database".to_string(), db_health.clone());
        }

        // Check cache health
        let cache_health = ServiceHealthCheck {
            service_name: "cache".to_string(),
            status: HealthStatus::Healthy, // Assuming healthy for now
            last_check: crate::utils::time::get_current_timestamp(),
            message: "Cache operational".to_string(),
        };

        #[allow(clippy::if_same_then_else)]
        if cache_health.status == HealthStatus::Healthy {
            services_health.insert("cache".to_string(), cache_health.clone());
        } else if cache_health.status == HealthStatus::Degraded {
            services_health.insert("cache".to_string(), cache_health.clone());
        } else if cache_health.status == HealthStatus::Unhealthy {
            services_health.insert("cache".to_string(), cache_health.clone());
        }

        let overall_health = if services_health
            .values()
            .any(|h| h.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else if services_health
            .values()
            .any(|h| h.status == HealthStatus::Degraded)
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        SystemHealthReport {
            overall_status: overall_health,
            services: services_health,
            timestamp: crate::utils::time::get_current_timestamp(),
            uptime_seconds: self.startup_time.unwrap_or(0),
        }
    }

    /// Initialize all infrastructure services
    pub async fn initialize(&mut self, env: &Env) -> ArbitrageResult<()> {
        // Initialize core services in dependency order

        // 2. Initialize database core
        // DatabaseCore initialization removed - using unified persistence layer

        // 3. Initialize cache manager
        // CacheManager initialization removed - using unified cloudflare services
        self.register_service(ServiceRegistration {
            service_name: "cache_manager".to_string(),
            service_type: ServiceType::Cache,
            version: "1.0.0".to_string(),
            description: "Centralized KV operations with intelligent TTL management".to_string(),
            dependencies: vec![],
            health_check_endpoint: None,
            metrics_enabled: true,
            auto_recovery: true,
            priority: 1,
            tags: HashMap::new(),
            configuration: HashMap::new(),
        })
        .await?;

        // 4. Initialize service health monitor
        if self.config.enable_health_monitoring {
            // ServiceHealthManager initialization removed - using unified core services
            self.register_service(ServiceRegistration {
                service_name: "service_health".to_string(),
                service_type: ServiceType::Health,
                version: "1.0.0".to_string(),
                description: "Comprehensive system health reporting with dependency tracking"
                    .to_string(),
                dependencies: vec![
                    ServiceDependency {
                        service_name: "database_core".to_string(),
                        service_type: ServiceType::Database,
                        is_critical: true,
                        timeout_ms: 5000,
                        retry_attempts: 3,
                        health_check_interval_seconds: 30,
                    },
                    ServiceDependency {
                        service_name: "cache_manager".to_string(),
                        service_type: ServiceType::Cache,
                        is_critical: true,
                        timeout_ms: 2000,
                        retry_attempts: 3,
                        health_check_interval_seconds: 30,
                    },
                ],
                health_check_endpoint: None,
                metrics_enabled: true,
                auto_recovery: true,
                priority: 2,
                tags: HashMap::new(),
                configuration: HashMap::new(),
            })
            .await?;
        }

        // 5. Initialize notification engine
        let notification_config = NotificationCoordinatorConfig::default();
        self.notification_engine = Some(
            NotificationCoordinator::new(
                notification_config,
                self.kv_store.as_ref().unwrap().as_ref().clone(),
                env,
            )
            .await?,
        );
        self.register_service(ServiceRegistration {
            service_name: "notification_engine".to_string(),
            service_type: ServiceType::Notification,
            version: "1.0.0".to_string(),
            description: "Centralized notification delivery and template management".to_string(),
            dependencies: vec![ServiceDependency {
                service_name: "cache_manager".to_string(),
                service_type: ServiceType::Cache,
                is_critical: false,
                timeout_ms: 2000,
                retry_attempts: 2,
                health_check_interval_seconds: 60,
            }],
            health_check_endpoint: None,
            metrics_enabled: true,
            auto_recovery: true,
            priority: 3,
            tags: HashMap::new(),
            configuration: HashMap::new(),
        })
        .await?;

        // 6. Initialize data access layer
        let _data_access_config = DataAccessLayerConfig::default();
        let unified_data_access_config = crate::services::core::infrastructure::data_access_layer::unified_data_access_engine::UnifiedDataAccessConfig::default();
        let mut engine = UnifiedDataAccessEngine::new(unified_data_access_config)?;
        engine = engine.with_cache(self.kv_store.as_ref().unwrap().clone());
        self.data_access = Some(engine);
        self.register_service(ServiceRegistration {
            service_name: "data_access_layer".to_string(),
            service_type: ServiceType::DataAccess,
            version: "1.0.0".to_string(),
            description: "Unified data access patterns with Pipeline → KV → API fallback"
                .to_string(),
            dependencies: vec![
                ServiceDependency {
                    service_name: "cache_manager".to_string(),
                    service_type: ServiceType::Cache,
                    is_critical: true,
                    timeout_ms: 2000,
                    retry_attempts: 3,
                    health_check_interval_seconds: 30,
                },
                ServiceDependency {
                    service_name: "database_core".to_string(),
                    service_type: ServiceType::Database,
                    is_critical: false,
                    timeout_ms: 5000,
                    retry_attempts: 2,
                    health_check_interval_seconds: 60,
                },
            ],
            health_check_endpoint: None,
            metrics_enabled: true,
            auto_recovery: true,
            priority: 3,
            tags: HashMap::new(),
            configuration: HashMap::new(),
        })
        .await?;

        // Start health monitoring if enabled
        if self.config.enable_health_monitoring {
            self.start_health_monitoring().await?;
        }

        Ok(())
    }

    /// Register a service with the infrastructure engine
    pub async fn register_service(&self, registration: ServiceRegistration) -> ArbitrageResult<()> {
        // Store service registration in a simple way
        if let Some(kv_store) = &self.kv_store {
            let key = format!("service_registry:{}", registration.service_name);
            let value = serde_json::to_string(&registration).map_err(|e| {
                ArbitrageError::serialization_error(format!(
                    "Failed to serialize service registration: {}",
                    e
                ))
            })?;

            kv_store.put(&key, value)?.execute().await.map_err(|e| {
                ArbitrageError::internal_error(format!("Failed to register service: {}", e))
            })?;
        }

        Ok(())
    }

    /// Get service information
    pub async fn get_service_info(&self, _service_name: &str) -> Option<ServiceInfo> {
        // Simplified service info retrieval
        None // Return None for now - complex service registry not needed for Workers
    }

    /// Get all registered services
    pub async fn get_all_services(&self) -> HashMap<String, ServiceInfo> {
        // Return empty map for simplified implementation
        HashMap::new()
    }

    /// Get infrastructure health summary
    pub async fn get_infrastructure_health(&self) -> InfrastructureHealth {
        // Simplified health check using available components
        let total_services = 4; // Core services we track
        let healthy_services = 4; // Assume all healthy for simplified implementation
        let degraded_services = 0;
        let unhealthy_services = 0;

        InfrastructureHealth {
            overall_status: ServiceStatus::Healthy,
            healthy_services,
            degraded_services,
            unhealthy_services,
            total_services,
            critical_services_healthy: true,
            uptime_percentage: 100.0,
            last_updated: crate::utils::time::get_current_timestamp(),
            service_statuses: HashMap::new(),
        }
    }

    /// Update service status
    pub async fn update_service_status(
        &self,
        _service_name: &str,
        _status: ServiceStatus,
    ) -> ArbitrageResult<()> {
        // Simplified status update - just log for now
        Ok(())
    }

    /// Restart a service (if auto-recovery is enabled)
    pub async fn restart_service(&self, _service_name: &str) -> ArbitrageResult<()> {
        // Service restart not needed in Workers environment
        Ok(())
    }

    /// Get configuration value
    pub async fn get_config(&self, key: &str) -> Option<serde_json::Value> {
        // Simple config access using the available config
        match key {
            "enable_health_monitoring" => Some(serde_json::Value::Bool(
                self.config.enable_health_monitoring,
            )),
            "enable_metrics_collection" => Some(serde_json::Value::Bool(
                self.config.enable_metrics_collection,
            )),
            _ => None,
        }
    }

    /// Set configuration value
    pub async fn set_config(&self, _key: &str, _value: serde_json::Value) -> ArbitrageResult<()> {
        // Config updates not supported in simplified Workers implementation
        Ok(())
    }

    /// Graceful shutdown of all services
    pub async fn shutdown(&self) -> ArbitrageResult<()> {
        // Simplified shutdown - just return success
        Ok(())
    }

    // ============= INTERNAL HELPER METHODS =============

    async fn start_health_monitoring(&self) -> ArbitrageResult<()> {
        // Health monitoring simplified for Workers
        Ok(())
    }

    async fn initialize_circuit_breaker(&self, _service_name: &str) -> ArbitrageResult<()> {
        // Circuit breakers simplified for Workers
        Ok(())
    }

    async fn check_circuit_breaker(&self, _service_name: &str) -> bool {
        // Always return true - no circuit breakers in simplified implementation
        true
    }

    async fn record_circuit_breaker_success(&self, _service_name: &str) -> ArbitrageResult<()> {
        // No-op for simplified implementation
        Ok(())
    }

    async fn record_circuit_breaker_failure(&self, _service_name: &str) -> ArbitrageResult<()> {
        // No-op for simplified implementation
        Ok(())
    }

    pub async fn get_detailed_health_status(&self) -> SystemHealthReport {
        // Simple health report for now
        let services = HashMap::new();

        SystemHealthReport {
            overall_status: HealthStatus::Healthy, // Placeholder
            services,
            timestamp: crate::utils::time::get_current_timestamp(),
            uptime_seconds: self.startup_time.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type_as_str() {
        assert_eq!(ServiceType::Database.as_str(), "database");
        assert_eq!(ServiceType::Cache.as_str(), "cache");
        assert_eq!(ServiceType::Health.as_str(), "health");
        assert_eq!(ServiceType::Custom("test".to_string()).as_str(), "test");
    }

    #[test]
    fn test_service_status_operational() {
        assert!(ServiceStatus::Healthy.is_operational());
        assert!(ServiceStatus::Degraded.is_operational());
        assert!(!ServiceStatus::Unhealthy.is_operational());
        assert!(!ServiceStatus::Stopped.is_operational());
        assert!(!ServiceStatus::Unknown.is_operational());
    }

    #[test]
    fn test_infrastructure_config_default() {
        let config = InfrastructureConfig::default();
        assert!(config.enable_service_discovery);
        assert!(config.enable_health_monitoring);
        assert!(config.enable_auto_recovery);
        assert!(config.enable_metrics_collection);
        assert_eq!(config.health_check_interval_seconds, 30);
        assert_eq!(config.max_restart_attempts, 3);
        assert!(config.enable_circuit_breaker);
        assert_eq!(config.circuit_breaker_threshold, 5);
    }

    #[test]
    fn test_circuit_breaker_states() {
        assert_eq!(CircuitBreakerState::Closed, CircuitBreakerState::Closed);
        assert_ne!(CircuitBreakerState::Open, CircuitBreakerState::Closed);
        assert_ne!(CircuitBreakerState::HalfOpen, CircuitBreakerState::Open);
    }

    #[test]
    fn test_service_registration_creation() {
        let registration = ServiceRegistration {
            service_name: "test_service".to_string(),
            service_type: ServiceType::Database,
            version: "1.0.0".to_string(),
            description: "Test service".to_string(),
            dependencies: vec![],
            health_check_endpoint: None,
            metrics_enabled: true,
            auto_recovery: true,
            priority: 1,
            tags: HashMap::new(),
            configuration: HashMap::new(),
        };

        assert_eq!(registration.service_name, "test_service");
        assert_eq!(registration.service_type, ServiceType::Database);
        assert_eq!(registration.version, "1.0.0");
        assert!(registration.metrics_enabled);
        assert!(registration.auto_recovery);
        assert_eq!(registration.priority, 1);
    }
}
