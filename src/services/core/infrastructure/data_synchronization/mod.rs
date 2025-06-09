//! Data Synchronization Engine
//!
//! Comprehensive distributed data synchronization framework for ensuring data consistency
//! across KV, D1, and R2 storage systems with multiple sync strategies, conflict resolution,
//! and operator tools.
//!
//! ## Features
//! - **Multiple Sync Strategies**: Write-through, write-behind, read-repair, periodic reconciliation
//! - **Conflict Resolution**: Vector clock-based conflict detection with multiple resolution strategies
//! - **Diff-based Sync**: Efficient delta synchronization with compression and merkle trees
//! - **Operator Tools**: Manual sync controls, dashboards, and administrative APIs
//! - **Comprehensive Testing**: Integration testing with consistency validation
//!
//! ## Architecture
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   KV Storage    │    │   D1 Database   │    │   R2 Storage    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!          │                       │                       │
//!          └───────────────────────┼───────────────────────┘
//!                                  │
//!                    ┌─────────────────┐
//!                    │ Sync Coordinator│
//!                    └─────────────────┘
//!                              │
//!            ┌─────────────────┼─────────────────┐
//!            │                 │                 │
//!   ┌────────────────┐ ┌──────────────┐ ┌──────────────┐
//!   │ Conflict       │ │ Diff Engine  │ │ Operator     │
//!   │ Resolution     │ │              │ │ Tools        │
//!   └────────────────┘ └──────────────┘ └──────────────┘
//! ```

pub mod sync_coordinator;
pub mod conflict_resolver;
pub mod diff_engine;
pub mod operator_tools;
pub mod sync_validation;

// Re-export main components
pub use sync_coordinator::{
    SyncCoordinator, SyncCoordinatorConfig, SyncCoordinatorMetrics, SyncStrategy, SyncStrategies,
    SyncConfig, SyncStats, SyncEvent, SyncEventType, SyncStatus, SyncOperation,
    WriteMode, ReadRepairConfig, ReconciliationConfig, ReconciliationSchedule
};

pub use conflict_resolver::{
    ConflictResolver, ConflictResolverConfig, ConflictDetector, ConflictResolutionStrategy,
    VectorClock, ConflictEvent, ConflictResolutionResult, ConflictMetrics, ConflictNotification,
    ResolutionPolicy, MergeStrategy, ConflictAuditLog
};

pub use diff_engine::{
    DiffEngine, DiffEngineConfig, DiffCalculator, DeltaSync, DiffResult, DiffMetrics,
    MerkleTree, RollingHash, CompressionEngine, DataDiff, DiffOperation, DiffType,
    SyncPayload, PayloadCompression
};

pub use operator_tools::{
    OperatorToolsService, ManualSyncTrigger, SyncDashboardService, OperatorToolsConfig,
    ManualSyncRequest, SyncTriggerType, SyncPriority, SyncQueueStatus, SyncDashboard,
    ActiveSyncOperation, StorageSystemStatus, SyncEvent, EventSeverity
};

pub use sync_validation::{
    SyncValidator, SyncValidationConfig, ConsistencyChecker, ValidationResult, ValidationMetrics,
    IntegrityValidator, ConsistencyReport, ValidationRule, ValidationSeverity, SyncTestSuite
};

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worker::Env;

/// Main data synchronization engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSynchronizationConfig {
    /// Sync coordinator configuration
    pub sync_coordinator: SyncCoordinatorConfig,
    /// Conflict resolution configuration
    pub conflict_resolver: ConflictResolverConfig,
    /// Diff engine configuration
    pub diff_engine: DiffEngineConfig,
    /// Operator tools configuration
    pub operator_tools: OperatorToolsConfig,
    /// Validation configuration
    pub validation: SyncValidationConfig,
    /// Feature flags for sync capabilities
    pub feature_flags: SyncFeatureFlags,
}

/// Feature flags for data synchronization capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFeatureFlags {
    /// Enable write-through synchronization
    pub enable_write_through: bool,
    /// Enable write-behind synchronization
    pub enable_write_behind: bool,
    /// Enable read-repair synchronization
    pub enable_read_repair: bool,
    /// Enable periodic reconciliation
    pub enable_periodic_reconciliation: bool,
    /// Enable vector clock conflict detection
    pub enable_vector_clocks: bool,
    /// Enable automatic conflict resolution
    pub enable_auto_conflict_resolution: bool,
    /// Enable diff-based synchronization
    pub enable_diff_sync: bool,
    /// Enable operator dashboard
    pub enable_operator_dashboard: bool,
    /// Enable manual sync triggers
    pub enable_manual_sync: bool,
    /// Enable compression for sync payloads
    pub enable_compression: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
}

impl Default for SyncFeatureFlags {
    fn default() -> Self {
        Self {
            enable_write_through: true,
            enable_write_behind: true,
            enable_read_repair: true,
            enable_periodic_reconciliation: true,
            enable_vector_clocks: true,
            enable_auto_conflict_resolution: true,
            enable_diff_sync: true,
            enable_operator_dashboard: true,
            enable_manual_sync: true,
            enable_compression: true,
            enable_metrics: true,
            enable_audit_logging: true,
        }
    }
}

impl Default for DataSynchronizationConfig {
    fn default() -> Self {
        Self {
            sync_coordinator: SyncCoordinatorConfig::default(),
            conflict_resolver: ConflictResolverConfig::default(),
            diff_engine: DiffEngineConfig::default(),
            operator_tools: OperatorToolsConfig::default(),
            validation: SyncValidationConfig::default(),
            feature_flags: SyncFeatureFlags::default(),
        }
    }
}

/// Main data synchronization engine
pub struct DataSynchronizationEngine {
    /// Sync coordinator
    sync_coordinator: Arc<SyncCoordinator>,
    /// Conflict resolver
    conflict_resolver: Arc<ConflictResolver>,
    /// Diff engine
    diff_engine: Arc<DiffEngine>,
    /// Operator tools
    operator_tools: Arc<OperatorToolsService>,
    /// Sync validator
    sync_validator: Arc<SyncValidator>,
    /// Configuration
    config: DataSynchronizationConfig,
    /// Initialization status
    is_initialized: bool,
}

impl DataSynchronizationEngine {
    /// Create new data synchronization engine
    pub async fn new(env: &Env, config: DataSynchronizationConfig) -> ArbitrageResult<Self> {
        // Initialize sync coordinator
        let sync_coordinator = Arc::new(
            SyncCoordinator::new(env, &config.sync_coordinator, &config.feature_flags).await?
        );

        // Initialize conflict resolver
        let conflict_resolver = Arc::new(
            ConflictResolver::new(&config.conflict_resolver, &config.feature_flags).await?
        );

        // Initialize diff engine
        let diff_engine = Arc::new(
            DiffEngine::new(&config.diff_engine, &config.feature_flags).await?
        );

        // Initialize operator tools
        let operator_tools = Arc::new(
            OperatorToolsService::new(&config.operator_tools, &config.feature_flags, Arc::clone(&sync_coordinator)).await?
        );

        // Initialize sync validator
        let sync_validator = Arc::new(
            SyncValidator::new(&config.validation, &config.feature_flags).await?
        );

        Ok(Self {
            sync_coordinator,
            conflict_resolver,
            diff_engine,
            operator_tools,
            sync_validator,
            config,
            is_initialized: false,
        })
    }

    /// Initialize the data synchronization engine
    pub async fn initialize(&mut self) -> ArbitrageResult<()> {
        if self.is_initialized {
            return Ok(());
        }

        // Initialize all components
        self.sync_coordinator.initialize().await?;
        self.conflict_resolver.initialize().await?;
        self.diff_engine.initialize().await?;
        self.operator_tools.initialize().await?;
        self.sync_validator.initialize().await?;

        self.is_initialized = true;
        Ok(())
    }

    /// Get sync coordinator
    pub fn sync_coordinator(&self) -> Arc<SyncCoordinator> {
        Arc::clone(&self.sync_coordinator)
    }

    /// Get conflict resolver
    pub fn conflict_resolver(&self) -> Arc<ConflictResolver> {
        Arc::clone(&self.conflict_resolver)
    }

    /// Get diff engine
    pub fn diff_engine(&self) -> Arc<DiffEngine> {
        Arc::clone(&self.diff_engine)
    }

    /// Get operator tools
    pub fn operator_tools(&self) -> Arc<OperatorToolsService> {
        Arc::clone(&self.operator_tools)
    }

    /// Get sync validator
    pub fn sync_validator(&self) -> Arc<SyncValidator> {
        Arc::clone(&self.sync_validator)
    }

    /// Perform health check
    pub async fn health_check(&self) -> ArbitrageResult<DataSyncHealth> {
        let coordinator_health = self.sync_coordinator.health_check().await?;
        let resolver_health = self.conflict_resolver.health_check().await?;
        let diff_health = self.diff_engine.health_check().await?;
        let operator_health = self.operator_tools.health_check().await?;
        let validator_health = self.sync_validator.health_check().await?;

        let overall_healthy = coordinator_health.is_healthy
            && resolver_health.is_healthy
            && diff_health.is_healthy
            && operator_health.is_healthy
            && validator_health.is_healthy;

        Ok(DataSyncHealth {
            overall_healthy,
            sync_coordinator_health: coordinator_health,
            conflict_resolver_health: resolver_health,
            diff_engine_health: diff_health,
            operator_tools_health: operator_health,
            sync_validator_health: validator_health,
            last_check: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Get comprehensive metrics
    pub async fn get_metrics(&self) -> ArbitrageResult<DataSyncMetrics> {
        let coordinator_metrics = self.sync_coordinator.get_metrics().await?;
        let resolver_metrics = self.conflict_resolver.get_metrics().await?;
        let diff_metrics = self.diff_engine.get_metrics().await?;
        let operator_metrics = self.operator_tools.get_metrics().await?;
        let validator_metrics = self.sync_validator.get_metrics().await?;

        Ok(DataSyncMetrics {
            sync_coordinator_metrics: coordinator_metrics,
            conflict_resolver_metrics: resolver_metrics,
            diff_engine_metrics: diff_metrics,
            operator_tools_metrics: operator_metrics,
            sync_validator_metrics: validator_metrics,
            collected_at: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Check if engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get configuration
    pub fn config(&self) -> &DataSynchronizationConfig {
        &self.config
    }

    /// Shutdown the engine gracefully
    pub async fn shutdown(&mut self) -> ArbitrageResult<()> {
        if !self.is_initialized {
            return Ok(());
        }

        // Shutdown all components
        self.sync_coordinator.shutdown().await?;
        self.conflict_resolver.shutdown().await?;
        self.diff_engine.shutdown().await?;
        self.operator_tools.shutdown().await?;
        self.sync_validator.shutdown().await?;

        self.is_initialized = false;
        Ok(())
    }
}

/// Overall data synchronization health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSyncHealth {
    pub overall_healthy: bool,
    pub sync_coordinator_health: SyncCoordinatorHealth,
    pub conflict_resolver_health: ConflictResolverHealth,
    pub diff_engine_health: DiffEngineHealth,
    pub operator_tools_health: OperatorToolsHealth,
    pub sync_validator_health: SyncValidatorHealth,
    pub last_check: u64,
}

/// Overall data synchronization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSyncMetrics {
    pub sync_coordinator_metrics: SyncCoordinatorMetrics,
    pub conflict_resolver_metrics: ConflictMetrics,
    pub diff_engine_metrics: DiffMetrics,
    pub operator_tools_metrics: OperatorMetrics,
    pub sync_validator_metrics: ValidationMetrics,
    pub collected_at: u64,
}

// Health status type aliases for consistency
pub type SyncCoordinatorHealth = crate::services::core::infrastructure::shared_types::ComponentHealth;
pub type ConflictResolverHealth = crate::services::core::infrastructure::shared_types::ComponentHealth;
pub type DiffEngineHealth = crate::services::core::infrastructure::shared_types::ComponentHealth;
pub type OperatorToolsHealth = crate::services::core::infrastructure::shared_types::ComponentHealth;
pub type SyncValidatorHealth = crate::services::core::infrastructure::shared_types::ComponentHealth; 