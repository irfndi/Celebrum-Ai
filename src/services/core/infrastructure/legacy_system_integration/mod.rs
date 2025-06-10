//! Legacy System Integration Module
//!
//! Comprehensive legacy system integration framework providing production-ready
//! migration capabilities with zero-downtime, gradual rollouts, and automated rollback.
//!
//! ## Core Components
//!
//! - **MigrationController**: Orchestrates migration phases and strategies
//! - **DualWriteCoordinator**: Manages dual-write operations with transaction safety
//! - **ReadMigrationManager**: Gradual read migration with intelligent routing
//! - **ValidationEngine**: Data consistency and integrity validation
//! - **FeatureFlagMigrationManager**: Advanced feature flag management for migrations
//! - **LegacyAdapterLayer**: Backward compatibility and service mapping
//!
//! ## Architectural Principles
//!
//! - Zero duplication with modular design
//! - High efficiency and concurrency
//! - Circuit breaker integration for fault tolerance
//! - Comprehensive monitoring and observability
//! - Feature flag-driven rollouts and rollbacks
//! - Production-ready with no mock implementations

pub mod dual_write_coordinator;
pub mod feature_flag_migration_manager;
pub mod legacy_adapter_layer;
pub mod migration_controller;
pub mod read_migration_manager;
pub mod shared_types;
pub mod validation_engine;

// Core exports
pub use feature_flag_migration_manager::{
    FeatureFlagMigrationManager, MigrationFeatureConfig, MigrationFeatureFlags, MigrationPhase,
    RolloutConfig, RolloutProgress, RolloutStrategy, SafetyThreshold,
};

pub use migration_controller::{
    MigrationController, MigrationControllerConfig, MigrationExecution, MigrationPlan,
    MigrationResult, MigrationStatus, MigrationStrategy,
};

pub use dual_write_coordinator::{
    ConsistencyLevel, DualWriteConfig, DualWriteCoordinator, DualWriteResult, DualWriteStrategy,
    WriteOperation, WriteResult,
};

pub use read_migration_manager::{
    ReadMigrationConfig, ReadMigrationManager, ReadMigrationPhase, ReadRoutingStrategy,
    RoutingDecision,
};

pub use validation_engine::{
    ConsistencyCheck, DataComparison, ValidationConfig, ValidationEngine, ValidationMetrics,
    ValidationResult, ValidationRule,
};

pub use legacy_adapter_layer::{
    AdapterConfig, CompatibilityLayer, LegacyAdapterLayer, RequestTranslation, ResponseTranslation,
    ServiceMapping,
};

pub use shared_types::{
    LegacySystemType, MigrationError, MigrationEvent, MigrationMetrics, MigrationSystemHealth,
    SystemIdentifier,
};

/// Main legacy system integration configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LegacySystemIntegrationConfig {
    /// Enable legacy system integration
    pub enabled: bool,
    /// Maximum concurrent migration operations
    pub max_concurrent_migrations: u32,
    /// Migration batch size for data operations
    pub migration_batch_size: u32,
    /// Validation sample rate (0.0 to 1.0)
    pub validation_sample_rate: f64,
    /// Automatic rollback on safety threshold breach
    pub automatic_rollback_on_threshold: bool,
    /// Migration timeout in seconds
    pub migration_timeout_seconds: u64,
    /// Validation timeout in seconds
    pub validation_timeout_seconds: u64,
    /// Dual-write timeout in milliseconds
    pub dual_write_timeout_ms: u64,
    /// Read migration timeout in milliseconds
    pub read_migration_timeout_ms: u64,
}

impl Default for LegacySystemIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_concurrent_migrations: 3,
            migration_batch_size: 1000,
            validation_sample_rate: 0.1,
            automatic_rollback_on_threshold: true,
            migration_timeout_seconds: 300,
            validation_timeout_seconds: 30,
            dual_write_timeout_ms: 5000,
            read_migration_timeout_ms: 3000,
        }
    }
}

/// Legacy System Integration Health Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LegacySystemIntegrationHealth {
    /// Overall system health status
    pub status: String,
    /// Migration controller health
    pub migration_controller_healthy: bool,
    /// Dual-write coordinator health
    pub dual_write_coordinator_healthy: bool,
    /// Read migration manager health
    pub read_migration_manager_healthy: bool,
    /// Validation engine health
    pub validation_engine_healthy: bool,
    /// Feature flag manager health
    pub feature_flag_manager_healthy: bool,
    /// Legacy adapter layer health
    pub legacy_adapter_layer_healthy: bool,
    /// Active migrations count
    pub active_migrations: u32,
    /// Last health check timestamp
    pub last_check: u64,
}

impl Default for LegacySystemIntegrationHealth {
    fn default() -> Self {
        Self {
            status: "initializing".to_string(),
            migration_controller_healthy: false,
            dual_write_coordinator_healthy: false,
            read_migration_manager_healthy: false,
            validation_engine_healthy: false,
            feature_flag_manager_healthy: false,
            legacy_adapter_layer_healthy: false,
            active_migrations: 0,
            last_check: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Legacy System Integration Metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LegacySystemIntegrationMetrics {
    /// Total migrations executed
    pub total_migrations: u64,
    /// Successful migrations
    pub successful_migrations: u64,
    /// Failed migrations
    pub failed_migrations: u64,
    /// Rollbacks executed
    pub rollbacks_executed: u64,
    /// Average migration duration in milliseconds
    pub average_migration_duration_ms: f64,
    /// Dual-write operations count
    pub dual_write_operations: u64,
    /// Read migrations performed
    pub read_migrations_performed: u64,
    /// Validation operations performed
    pub validation_operations: u64,
    /// Validation failures detected
    pub validation_failures: u64,
    /// Feature flag toggles performed
    pub feature_flag_toggles: u64,
    /// Last metrics update timestamp
    pub last_updated: u64,
}

impl Default for LegacySystemIntegrationMetrics {
    fn default() -> Self {
        Self {
            total_migrations: 0,
            successful_migrations: 0,
            failed_migrations: 0,
            rollbacks_executed: 0,
            average_migration_duration_ms: 0.0,
            dual_write_operations: 0,
            read_migrations_performed: 0,
            validation_operations: 0,
            validation_failures: 0,
            feature_flag_toggles: 0,
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}
