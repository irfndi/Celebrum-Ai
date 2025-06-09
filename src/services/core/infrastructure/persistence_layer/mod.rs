//! D1/R2 Persistence Layer
//!
//! Comprehensive database schema and blob storage architecture for ArbEdge platform
//! providing unified persistence layer with connection pooling, transactions, and migrations

pub mod connection_pool;
pub mod migration_engine;
pub mod migration_utilities;
pub mod performance_monitor;
pub mod query_profiler;
pub mod schema_manager;
pub mod transaction_coordinator;
// pub mod r2_storage_manager;

// Re-export main components
pub use connection_pool::{
    ConnectionHealth, ConnectionManager, ConnectionMetrics, ConnectionPool, ConnectionStats,
    PoolConfig, ServiceHealth,
};
pub use migration_engine::{
    Migration, MigrationConfig, MigrationDirection, MigrationEngine, MigrationOperation,
    MigrationPlan, MigrationRecord, MigrationState, MigrationStats, MigrationTarget,
    MigrationVersion, R2MigrationOperation, SchemaChangeType, ValidationCheck,
    ValidationCheckResult, ValidationResult,
};
pub use migration_utilities::{
    ColumnDefinition, CommonMigrations, DataTransformer, ForeignKeyDefinition, MigrationBuilder,
    MigrationDiscovery, SchemaSnapshot, SqlGenerator, TableSnapshot,
};
pub use performance_monitor::{
    AlertSeverity, AlertType, ConnectionPoolHealth, DatabaseHealth, DatabaseType, ErrorCategory,
    ErrorRateHealth, IndexEfficiencyHealth, IndexUsage, OptimizationRecommendation,
    PerformanceAlert, PerformanceConfig, PerformanceDashboard, PerformanceMonitor,
    PerformanceTrends, QueryError, QueryMetrics, QueryOperationType, QueryPerformanceHealth,
    QueryPlan, RecommendationPriority, RecommendationType, ScanType, TrendPoint,
};
pub use query_profiler::{ProfilerConfig, QueryContext, QueryParameter, QueryProfiler};
pub use schema_manager::{
    ConstraintDefinition, DataType, IndexDefinition, SchemaManager, SchemaType, SchemaVersion,
    TableDefinition,
};
pub use transaction_coordinator::{
    D1OperationType, IsolationLevel, R2OperationType, TransactionConfig, TransactionCoordinator,
    TransactionEvent, TransactionInfo, TransactionLogEntry, TransactionOperation, TransactionState,
    TransactionStats,
};

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worker::Env;

/// Unified persistence layer configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// D1 Database configuration
    pub d1_config: D1Config,
    /// R2 Storage configuration
    pub r2_config: R2Config,
}

/// D1 Database specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1Config {
    pub database_name: String,
    pub max_connections: u32,
    pub query_timeout_ms: u64,
    pub enable_prepared_statements: bool,
    pub enable_query_logging: bool,
    pub enable_connection_pooling: bool,
}

/// R2 Storage specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Config {
    pub bucket_name: String,
    pub max_object_size_mb: u64,
    pub default_storage_class: String,
    pub enable_compression: bool,
    pub compression_threshold_kb: u64,
    pub enable_lifecycle_management: bool,
}

impl Default for D1Config {
    fn default() -> Self {
        Self {
            database_name: "ArbEdgeDB".to_string(),
            max_connections: 50,
            query_timeout_ms: 30000,
            enable_prepared_statements: true,
            enable_query_logging: true,
            enable_connection_pooling: true,
        }
    }
}

impl Default for R2Config {
    fn default() -> Self {
        Self {
            bucket_name: "arb-edge-storage".to_string(),
            max_object_size_mb: 100,
            default_storage_class: "Standard".to_string(),
            enable_compression: true,
            compression_threshold_kb: 1024,
            enable_lifecycle_management: true,
        }
    }
}

/// Main persistence layer manager
pub struct PersistenceLayer {
    /// Schema management
    schema_manager: Arc<SchemaManager>,
    /// Connection management
    connection_manager: Arc<ConnectionManager>,
    /// Configuration
    config: PersistenceConfig,
}

impl PersistenceLayer {
    /// Create new persistence layer instance
    pub async fn new(env: &Env, config: PersistenceConfig) -> ArbitrageResult<Self> {
        // Initialize schema manager
        let schema_manager =
            Arc::new(SchemaManager::new(&config.d1_config, &config.r2_config).await?);

        // Initialize connection manager with default pool config
        let pool_config = PoolConfig::default();
        #[allow(clippy::arc_with_non_send_sync)]
        let connection_manager = Arc::new(ConnectionManager::new(env, &pool_config).await?);

        Ok(Self {
            schema_manager,
            connection_manager,
            config,
        })
    }

    /// Initialize the persistence layer
    pub async fn initialize(&self) -> ArbitrageResult<()> {
        // Validate configuration
        self.validate_configuration().await?;

        // Initialize schema if needed
        self.schema_manager.initialize_schema().await?;

        Ok(())
    }

    /// Get schema manager
    pub fn schema_manager(&self) -> Arc<SchemaManager> {
        Arc::clone(&self.schema_manager)
    }

    /// Get connection manager
    pub fn connection_manager(&self) -> Arc<ConnectionManager> {
        Arc::clone(&self.connection_manager)
    }

    /// Health check for persistence layer
    pub async fn health_check(&self) -> ArbitrageResult<PersistenceHealth> {
        let schema_issues = self.schema_manager.validate_schema().await?;
        let connection_health = self.connection_manager.health_check().await?;

        Ok(PersistenceHealth {
            overall_healthy: schema_issues.is_empty() && connection_health.is_healthy,
            schema_status: SchemaHealth {
                is_healthy: schema_issues.is_empty(),
                validation_issues: schema_issues,
                table_count: self.schema_manager.get_all_tables().len() as u32,
            },
            connection_status: connection_health,
            last_check: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Validate configuration
    async fn validate_configuration(&self) -> ArbitrageResult<()> {
        // Validate D1 configuration
        if self.config.d1_config.database_name.is_empty() {
            return Err(ArbitrageError::database_error(
                "D1 database name cannot be empty",
            ));
        }

        // Validate R2 configuration
        if self.config.r2_config.bucket_name.is_empty() {
            return Err(ArbitrageError::database_error(
                "R2 bucket name cannot be empty",
            ));
        }

        Ok(())
    }
}

/// Overall persistence layer health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceHealth {
    pub overall_healthy: bool,
    pub schema_status: SchemaHealth,
    pub connection_status: ConnectionHealth,
    pub last_check: u64,
}

/// Schema health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaHealth {
    pub is_healthy: bool,
    pub validation_issues: Vec<String>,
    pub table_count: u32,
}

/// Persistence metrics (simplified for Task 15.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceMetrics {
    pub schema_metrics: SchemaMetrics,
    pub collected_at: u64,
}

/// Schema metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetrics {
    pub total_tables: u32,
    pub total_indexes: u32,
    pub total_constraints: u32,
    pub schema_version: String,
}
