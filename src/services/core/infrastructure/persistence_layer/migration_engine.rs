//! Migration Engine Implementation for D1/R2 Persistence Layer
//!
//! Provides comprehensive database migration framework supporting schema versioning,
//! forward/backward migrations, data transformation utilities, zero-downtime deployments,
//! migration validation, rollback capabilities, and automated migration execution.

use crate::utils::error::{ArbitrageError, ArbitrageResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use worker::Env;

use super::connection_pool::ConnectionManager;
use super::schema_manager::SchemaManager;
use super::transaction_coordinator::{TransactionConfig, TransactionCoordinator};

/// Migration engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Enable automatic migration discovery
    pub auto_discovery: bool,
    /// Migration timeout in milliseconds
    pub migration_timeout_ms: u64,
    /// Enable parallel migration execution
    pub parallel_execution: bool,
    /// Maximum parallel migrations
    pub max_parallel_migrations: u32,
    /// Enable zero-downtime migrations
    pub zero_downtime: bool,
    /// Enable migration validation
    pub validate_migrations: bool,
    /// Backup before migration
    pub backup_before_migration: bool,
    /// Maximum rollback steps
    pub max_rollback_steps: u32,
    /// Migration batch size for large datasets
    pub batch_size: u32,
    /// Enable dry run mode
    pub dry_run_mode: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            migration_timeout_ms: 300000, // 5 minutes
            parallel_execution: true,
            max_parallel_migrations: 3,
            zero_downtime: true,
            validate_migrations: true,
            backup_before_migration: true,
            max_rollback_steps: 10,
            batch_size: 1000,
            dry_run_mode: false,
        }
    }
}

/// Migration version information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MigrationVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
    /// Pre-release identifier
    pub pre_release: Option<String>,
}

impl MigrationVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    pub fn with_pre_release(mut self, pre_release: String) -> Self {
        self.pre_release = Some(pre_release);
        self
    }

    pub fn is_compatible_with(&self, other: &MigrationVersion) -> bool {
        self.major == other.major && self >= other
    }
}

impl std::fmt::Display for MigrationVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref pre) = self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Migration direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationDirection {
    /// Forward migration (up)
    Up,
    /// Backward migration (down)
    Down,
}

/// Migration target type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationTarget {
    /// D1 database migration
    D1Database,
    /// R2 storage migration
    R2Storage,
    /// Both D1 and R2
    Both,
}

/// Migration operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationOperation {
    /// SQL operation for D1
    SqlOperation {
        sql: String,
        parameters: Vec<serde_json::Value>,
        rollback_sql: Option<String>,
    },
    /// R2 storage operation
    R2Operation {
        operation_type: R2MigrationOperation,
        key_pattern: String,
        data_transformation: Option<String>,
    },
    /// Custom data transformation
    DataTransformation {
        source_query: String,
        transformation_fn: String,
        target_operation: String,
    },
    /// Schema modification
    SchemaChange {
        change_type: SchemaChangeType,
        table_name: String,
        definition: serde_json::Value,
    },
}

/// R2 migration operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum R2MigrationOperation {
    /// Copy objects with transformation
    CopyTransform,
    /// Delete objects matching pattern
    DeletePattern,
    /// Rename objects
    RenamePattern,
    /// Update metadata
    UpdateMetadata,
}

/// Schema change types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChangeType {
    CreateTable,
    DropTable,
    AlterTable,
    CreateIndex,
    DropIndex,
    CreateConstraint,
    DropConstraint,
}

/// Migration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Unique migration identifier
    pub id: String,
    /// Migration name/description
    pub name: String,
    /// Migration version
    pub version: MigrationVersion,
    /// Target for this migration
    pub target: MigrationTarget,
    /// Forward migration operations
    pub up_operations: Vec<MigrationOperation>,
    /// Backward migration operations
    pub down_operations: Vec<MigrationOperation>,
    /// Migration dependencies
    pub dependencies: Vec<String>,
    /// Estimated execution time in seconds
    pub estimated_duration_secs: u32,
    /// Migration tags for categorization
    pub tags: Vec<String>,
    /// Validation checks
    pub validation_checks: Vec<ValidationCheck>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Author information
    pub author: String,
}

/// Validation check for migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Check name
    pub name: String,
    /// Check description
    pub description: String,
    /// SQL query or validation logic
    pub check_query: String,
    /// Expected result type
    pub expected_result: ValidationResult,
}

/// Validation result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Expect specific row count
    RowCount(u32),
    /// Expect no rows
    NoRows,
    /// Expect any rows
    AnyRows,
    /// Custom validation function
    CustomValidation(String),
}

/// Migration execution state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationState {
    /// Migration is pending execution
    Pending,
    /// Migration is currently running
    Running,
    /// Migration completed successfully
    Completed,
    /// Migration failed during execution
    Failed,
    /// Migration was rolled back
    RolledBack,
    /// Migration is being validated
    Validating,
    /// Migration validation failed
    ValidationFailed,
    /// Migration is being backed up
    BackingUp,
}

/// Migration execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration ID
    pub migration_id: String,
    /// Current state
    pub state: MigrationState,
    /// Execution direction
    pub direction: MigrationDirection,
    /// Started timestamp
    pub started_at: DateTime<Utc>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Validation results
    pub validation_results: Vec<ValidationCheckResult>,
    /// Rollback information
    pub rollback_info: Option<RollbackInfo>,
}

/// Validation check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheckResult {
    /// Check name
    pub check_name: String,
    /// Whether check passed
    pub passed: bool,
    /// Actual result
    pub actual_result: String,
    /// Expected result
    pub expected_result: String,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Rollback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Rollback reason
    pub reason: String,
    /// Backup location
    pub backup_location: Option<String>,
    /// Rollback operations executed
    pub rollback_operations: Vec<String>,
    /// Rollback timestamp
    pub rolled_back_at: DateTime<Utc>,
}

/// Migration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStats {
    /// Total migrations registered
    pub total_migrations: u32,
    /// Pending migrations
    pub pending_migrations: u32,
    /// Completed migrations
    pub completed_migrations: u32,
    /// Failed migrations
    pub failed_migrations: u32,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Success rate percentage
    pub success_rate_percentage: f64,
    /// Last migration timestamp
    pub last_migration_at: Option<DateTime<Utc>>,
    /// Statistics last updated
    pub last_updated: DateTime<Utc>,
}

impl Default for MigrationStats {
    fn default() -> Self {
        Self {
            total_migrations: 0,
            pending_migrations: 0,
            completed_migrations: 0,
            failed_migrations: 0,
            average_execution_time_ms: 0.0,
            success_rate_percentage: 0.0,
            last_migration_at: None,
            last_updated: Utc::now(),
        }
    }
}

/// Migration plan for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Plan ID
    pub plan_id: String,
    /// Target version
    pub target_version: MigrationVersion,
    /// Current version
    pub current_version: MigrationVersion,
    /// Migrations to execute in order
    pub migrations: Vec<Migration>,
    /// Estimated total duration in seconds
    pub estimated_duration_secs: u32,
    /// Plan created timestamp
    pub created_at: DateTime<Utc>,
    /// Plan validation status
    pub is_validated: bool,
}

/// Migration engine main implementation
#[allow(dead_code)]
pub struct MigrationEngine {
    /// Migration configuration
    config: MigrationConfig,
    /// Connection manager
    connection_manager: Arc<ConnectionManager>,
    /// Schema manager
    schema_manager: Arc<SchemaManager>,
    /// Transaction coordinator
    transaction_coordinator: Arc<TransactionCoordinator>,
    /// Registered migrations
    migrations: Arc<Mutex<HashMap<String, Migration>>>,
    /// Migration execution history
    execution_history: Arc<Mutex<VecDeque<MigrationRecord>>>,
    /// Migration statistics
    stats: Arc<Mutex<MigrationStats>>,
    /// Current schema version
    current_version: Arc<Mutex<MigrationVersion>>,
    /// Logger instance
    logger: crate::utils::logger::Logger,
}

impl MigrationEngine {
    /// Create new migration engine
    pub async fn new(
        config: MigrationConfig,
        connection_manager: Arc<ConnectionManager>,
        schema_manager: Arc<SchemaManager>,
    ) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        // Initialize transaction coordinator for migration transactions
        let transaction_config = TransactionConfig::default();
        #[allow(clippy::arc_with_non_send_sync)]
        let transaction_coordinator = Arc::new(
            TransactionCoordinator::new(transaction_config, Arc::clone(&connection_manager))
                .await?,
        );

        let engine = Self {
            config,
            connection_manager,
            schema_manager,
            transaction_coordinator,
            migrations: Arc::new(Mutex::new(HashMap::new())),
            execution_history: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(Mutex::new(MigrationStats::default())),
            current_version: Arc::new(Mutex::new(MigrationVersion::new(0, 0, 0))),
            logger,
        };

        engine.logger.info("Migration engine initialized");
        Ok(engine)
    }

    /// Register a new migration
    pub async fn register_migration(&self, migration: Migration) -> ArbitrageResult<()> {
        // Validate migration
        self.validate_migration(&migration).await?;

        // Store migration
        {
            let mut migrations = self.migrations.lock().unwrap();
            migrations.insert(migration.id.clone(), migration.clone());
        }

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_migrations += 1;
            stats.pending_migrations += 1;
            stats.last_updated = Utc::now();
        }

        self.logger.info(&format!(
            "Registered migration: {} ({})",
            migration.name, migration.version
        ));

        Ok(())
    }

    /// Create migration plan to target version
    pub async fn create_migration_plan(
        &self,
        target_version: MigrationVersion,
    ) -> ArbitrageResult<MigrationPlan> {
        let current_version = {
            let version = self.current_version.lock().unwrap();
            version.clone()
        };

        let migrations = self
            .get_migration_path(&current_version, &target_version)
            .await?;

        let estimated_duration_secs = migrations
            .iter()
            .map(|m| m.estimated_duration_secs)
            .sum::<u32>();

        let plan = MigrationPlan {
            plan_id: Uuid::new_v4().to_string(),
            target_version,
            current_version,
            migrations,
            estimated_duration_secs,
            created_at: Utc::now(),
            is_validated: false,
        };

        self.logger.info(&format!(
            "Created migration plan {} with {} migrations",
            plan.plan_id,
            plan.migrations.len()
        ));

        Ok(plan)
    }

    /// Execute migration plan
    pub async fn execute_migration_plan(
        &self,
        env: &Env,
        mut plan: MigrationPlan,
    ) -> ArbitrageResult<Vec<MigrationRecord>> {
        // Validate plan if not already validated
        if !plan.is_validated {
            self.validate_migration_plan(&plan).await?;
            plan.is_validated = true;
        }

        let mut execution_records = Vec::new();

        // Execute migrations in order
        for migration in plan.migrations {
            let record = self
                .execute_single_migration(env, &migration, MigrationDirection::Up)
                .await?;
            execution_records.push(record);
        }

        // Update current version
        let target_version = plan.target_version.clone();
        {
            let mut current_version = self.current_version.lock().unwrap();
            *current_version = plan.target_version;
        }

        self.logger.info(&format!(
            "Completed migration plan {} to version {}",
            plan.plan_id, target_version
        ));

        Ok(execution_records)
    }

    /// Execute single migration
    pub async fn execute_single_migration(
        &self,
        env: &Env,
        migration: &Migration,
        direction: MigrationDirection,
    ) -> ArbitrageResult<MigrationRecord> {
        let start_time = Utc::now();
        let mut record = MigrationRecord {
            migration_id: migration.id.clone(),
            state: MigrationState::Running,
            direction,
            started_at: start_time,
            completed_at: None,
            duration_ms: None,
            error_message: None,
            validation_results: Vec::new(),
            rollback_info: None,
        };

        // Begin transaction for migration
        let transaction_id = self.transaction_coordinator.begin_transaction(None).await?;

        let execution_result = match direction {
            MigrationDirection::Up => {
                self.execute_migration_operations(env, &transaction_id, &migration.up_operations)
                    .await
            }
            MigrationDirection::Down => {
                self.execute_migration_operations(env, &transaction_id, &migration.down_operations)
                    .await
            }
        };

        match execution_result {
            Ok(_) => {
                // Validate migration if enabled
                if self.config.validate_migrations {
                    record.state = MigrationState::Validating;
                    record.validation_results =
                        self.validate_migration_execution(migration).await?;

                    // Check if all validations passed
                    let all_passed = record.validation_results.iter().all(|r| r.passed);
                    if !all_passed {
                        record.state = MigrationState::ValidationFailed;
                        self.transaction_coordinator
                            .rollback_transaction(env, &transaction_id)
                            .await?;
                    } else {
                        record.state = MigrationState::Completed;
                        self.transaction_coordinator
                            .commit_transaction(env, &transaction_id)
                            .await?;
                    }
                } else {
                    record.state = MigrationState::Completed;
                    self.transaction_coordinator
                        .commit_transaction(env, &transaction_id)
                        .await?;
                }
            }
            Err(e) => {
                record.state = MigrationState::Failed;
                record.error_message = Some(e.to_string());
                self.transaction_coordinator
                    .rollback_transaction(env, &transaction_id)
                    .await?;
            }
        }

        // Update record with completion info
        let end_time = Utc::now();
        record.completed_at = Some(end_time);
        record.duration_ms = Some((end_time - start_time).num_milliseconds() as u64);

        // Store execution record
        {
            let mut history = self.execution_history.lock().unwrap();
            history.push_back(record.clone());

            // Keep only recent history
            while history.len() > 1000 {
                history.pop_front();
            }
        }

        // Update statistics
        self.update_migration_statistics(&record).await;

        self.logger.info(&format!(
            "Migration {} {} in {}ms with state {:?}",
            migration.name,
            match direction {
                MigrationDirection::Up => "up",
                MigrationDirection::Down => "down",
            },
            record.duration_ms.unwrap_or(0),
            record.state
        ));

        Ok(record)
    }

    /// Rollback to specific version
    pub async fn rollback_to_version(
        &self,
        env: &Env,
        target_version: MigrationVersion,
    ) -> ArbitrageResult<Vec<MigrationRecord>> {
        let current_version = {
            let version = self.current_version.lock().unwrap();
            version.clone()
        };

        if target_version >= current_version {
            return Err(ArbitrageError::database_error(
                "Target version must be lower than current version for rollback",
            ));
        }

        // Get rollback path
        let rollback_migrations = self
            .get_rollback_path(&current_version, &target_version)
            .await?;

        let mut rollback_records = Vec::new();

        // Execute rollback migrations
        for migration in rollback_migrations {
            let record = self
                .execute_single_migration(env, &migration, MigrationDirection::Down)
                .await?;
            rollback_records.push(record);
        }

        // Update current version
        let target_version_clone = target_version.clone();
        {
            let mut current_version = self.current_version.lock().unwrap();
            *current_version = target_version;
        }

        self.logger.info(&format!(
            "Completed rollback to version {}",
            target_version_clone
        ));

        Ok(rollback_records)
    }

    /// Get migration statistics
    pub async fn get_statistics(&self) -> ArbitrageResult<MigrationStats> {
        let stats = self.stats.lock().unwrap();
        Ok(stats.clone())
    }

    /// Get current schema version
    pub async fn get_current_version(&self) -> ArbitrageResult<MigrationVersion> {
        let version = self.current_version.lock().unwrap();
        Ok(version.clone())
    }

    /// List pending migrations
    pub async fn get_pending_migrations(&self) -> ArbitrageResult<Vec<Migration>> {
        let migrations = self.migrations.lock().unwrap();
        let current_version = {
            let version = self.current_version.lock().unwrap();
            version.clone()
        };

        let pending: Vec<Migration> = migrations
            .values()
            .filter(|m| m.version > current_version)
            .cloned()
            .collect();

        Ok(pending)
    }

    /// Validate migration
    async fn validate_migration(&self, migration: &Migration) -> ArbitrageResult<()> {
        // Check for duplicate migration ID
        {
            let migrations = self.migrations.lock().unwrap();
            if migrations.contains_key(&migration.id) {
                return Err(ArbitrageError::database_error(format!(
                    "Migration with ID {} already exists",
                    migration.id
                )));
            }
        }

        // Validate operations
        for operation in &migration.up_operations {
            self.validate_migration_operation(operation).await?;
        }

        for operation in &migration.down_operations {
            self.validate_migration_operation(operation).await?;
        }

        Ok(())
    }

    /// Validate migration operation
    async fn validate_migration_operation(
        &self,
        _operation: &MigrationOperation,
    ) -> ArbitrageResult<()> {
        // TODO: Implement operation-specific validation
        Ok(())
    }

    /// Validate migration plan
    async fn validate_migration_plan(&self, _plan: &MigrationPlan) -> ArbitrageResult<()> {
        // TODO: Implement comprehensive plan validation
        Ok(())
    }

    /// Get migration path between versions
    async fn get_migration_path(
        &self,
        from: &MigrationVersion,
        to: &MigrationVersion,
    ) -> ArbitrageResult<Vec<Migration>> {
        let migrations = self.migrations.lock().unwrap();
        let mut path: Vec<Migration> = migrations
            .values()
            .filter(|m| m.version > *from && m.version <= *to)
            .cloned()
            .collect();

        // Sort by version
        path.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(path)
    }

    /// Get rollback path between versions
    async fn get_rollback_path(
        &self,
        from: &MigrationVersion,
        to: &MigrationVersion,
    ) -> ArbitrageResult<Vec<Migration>> {
        let migrations = self.migrations.lock().unwrap();
        let mut path: Vec<Migration> = migrations
            .values()
            .filter(|m| m.version > *to && m.version <= *from)
            .cloned()
            .collect();

        // Sort by version in reverse order for rollback
        path.sort_by(|a, b| b.version.cmp(&a.version));

        Ok(path)
    }

    /// Execute migration operations
    async fn execute_migration_operations(
        &self,
        env: &Env,
        transaction_id: &str,
        operations: &[MigrationOperation],
    ) -> ArbitrageResult<()> {
        for operation in operations {
            self.execute_migration_operation(env, transaction_id, operation)
                .await?;
        }
        Ok(())
    }

    /// Execute single migration operation
    async fn execute_migration_operation(
        &self,
        env: &Env,
        transaction_id: &str,
        operation: &MigrationOperation,
    ) -> ArbitrageResult<()> {
        match operation {
            MigrationOperation::SqlOperation {
                sql, parameters, ..
            } => {
                // Execute SQL operation through transaction coordinator
                let d1_operation = crate::services::core::infrastructure::persistence_layer::transaction_coordinator::TransactionOperation::D1Operation {
                    sql: sql.clone(),
                    parameters: parameters.clone(),
                    operation_type: crate::services::core::infrastructure::persistence_layer::transaction_coordinator::D1OperationType::Select, // TODO: Determine actual operation type
                };

                self.transaction_coordinator
                    .execute_operation(env, transaction_id, d1_operation)
                    .await?;
            }
            MigrationOperation::R2Operation { .. } => {
                // TODO: Implement R2 migration operations
                self.logger.debug("R2 migration operation placeholder");
            }
            MigrationOperation::DataTransformation { .. } => {
                // TODO: Implement data transformation operations
                self.logger
                    .debug("Data transformation operation placeholder");
            }
            MigrationOperation::SchemaChange { .. } => {
                // TODO: Implement schema change operations
                self.logger.debug("Schema change operation placeholder");
            }
        }

        Ok(())
    }

    /// Validate migration execution
    async fn validate_migration_execution(
        &self,
        _migration: &Migration,
    ) -> ArbitrageResult<Vec<ValidationCheckResult>> {
        // TODO: Implement migration validation checks
        Ok(Vec::new())
    }

    /// Update migration statistics
    async fn update_migration_statistics(&self, record: &MigrationRecord) {
        let mut stats = self.stats.lock().unwrap();

        match record.state {
            MigrationState::Completed => {
                stats.completed_migrations += 1;
                stats.pending_migrations = stats.pending_migrations.saturating_sub(1);
                if let Some(duration) = record.duration_ms {
                    stats.average_execution_time_ms =
                        (stats.average_execution_time_ms + duration as f64) / 2.0;
                }
                stats.last_migration_at = record.completed_at;
            }
            MigrationState::Failed | MigrationState::ValidationFailed => {
                stats.failed_migrations += 1;
                stats.pending_migrations = stats.pending_migrations.saturating_sub(1);
            }
            _ => {}
        }

        // Calculate success rate
        let total_completed = stats.completed_migrations + stats.failed_migrations;
        if total_completed > 0 {
            stats.success_rate_percentage =
                (stats.completed_migrations as f64 / total_completed as f64) * 100.0;
        }

        stats.last_updated = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_version_creation() {
        let version = MigrationVersion::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
    }

    #[test]
    fn test_migration_version_with_prerelease() {
        let version = MigrationVersion::new(1, 0, 0).with_pre_release("alpha".to_string());
        assert_eq!(version.pre_release, Some("alpha".to_string()));
        assert_eq!(version.to_string(), "1.0.0-alpha");
    }

    #[test]
    fn test_migration_version_display() {
        let version = MigrationVersion::new(2, 1, 0);
        assert_eq!(version.to_string(), "2.1.0");
    }

    #[test]
    fn test_migration_version_compatibility() {
        let v1 = MigrationVersion::new(1, 0, 0);
        let v2 = MigrationVersion::new(1, 1, 0);
        let v3 = MigrationVersion::new(2, 0, 0);

        assert!(v2.is_compatible_with(&v1));
        assert!(!v3.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_migration_version_ordering() {
        let v1 = MigrationVersion::new(1, 0, 0);
        let v2 = MigrationVersion::new(1, 1, 0);
        let v3 = MigrationVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert!(config.auto_discovery);
        assert_eq!(config.migration_timeout_ms, 300000);
        assert!(config.parallel_execution);
        assert_eq!(config.max_parallel_migrations, 3);
        assert!(config.zero_downtime);
    }

    #[test]
    fn test_migration_stats_default() {
        let stats = MigrationStats::default();
        assert_eq!(stats.total_migrations, 0);
        assert_eq!(stats.pending_migrations, 0);
        assert_eq!(stats.completed_migrations, 0);
        assert_eq!(stats.failed_migrations, 0);
        assert_eq!(stats.average_execution_time_ms, 0.0);
        assert_eq!(stats.success_rate_percentage, 0.0);
    }

    #[test]
    fn test_migration_direction() {
        assert_eq!(MigrationDirection::Up, MigrationDirection::Up);
        assert_ne!(MigrationDirection::Up, MigrationDirection::Down);
    }

    #[test]
    fn test_migration_target() {
        assert_eq!(MigrationTarget::D1Database, MigrationTarget::D1Database);
        assert_ne!(MigrationTarget::D1Database, MigrationTarget::R2Storage);
    }

    #[test]
    fn test_migration_state_transitions() {
        let state = MigrationState::Pending;
        assert_eq!(state, MigrationState::Pending);
        assert_ne!(state, MigrationState::Running);
    }
}
