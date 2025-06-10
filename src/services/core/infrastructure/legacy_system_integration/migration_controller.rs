//! Migration Controller
//!
//! Orchestrates legacy system migration with dual-write strategies, gradual migration phases,
//! and comprehensive rollback capabilities. Integrates with feature flags for zero-downtime migrations.

use super::{
    feature_flag_migration_manager::{FeatureFlagMigrationManager, MigrationPhase, RolloutConfig},
    shared_types::{
        EventSeverity, LegacySystemType, MigrationEvent, MigrationEventType, PerformanceMetrics,
        SystemIdentifier,
    },
};
use crate::services::core::infrastructure::shared_types::ComponentHealth;
use crate::utils::error::ErrorKind;
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use worker::Env;

/// Migration strategies supported by the controller
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    /// Immediate full migration
    Immediate,
    /// Gradual percentage-based migration
    Gradual {
        initial_percentage: f64,
        increment_percentage: f64,
        interval_minutes: u64,
    },
    /// Blue-green deployment style migration
    BlueGreen,
    /// Canary deployment with automated rollout
    Canary {
        canary_percentage: f64,
        success_threshold: f64,
        observation_period_minutes: u64,
    },
}

impl Default for MigrationStrategy {
    fn default() -> Self {
        MigrationStrategy::Gradual {
            initial_percentage: 5.0,
            increment_percentage: 10.0,
            interval_minutes: 15,
        }
    }
}

/// Migration status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// Migration not started
    NotStarted,
    /// Migration in progress
    InProgress,
    /// Migration paused
    Paused,
    /// Migration completed successfully
    Completed,
    /// Migration failed
    Failed,
    /// Migration rolled back
    RolledBack,
    /// Migration validation phase
    Validating,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStatus::NotStarted => write!(f, "not_started"),
            MigrationStatus::InProgress => write!(f, "in_progress"),
            MigrationStatus::Paused => write!(f, "paused"),
            MigrationStatus::Completed => write!(f, "completed"),
            MigrationStatus::Failed => write!(f, "failed"),
            MigrationStatus::RolledBack => write!(f, "rolled_back"),
            MigrationStatus::Validating => write!(f, "validating"),
        }
    }
}

/// Comparison operators for triggers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    EqualTo,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Rollback strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackStrategy {
    /// Immediate rollback (fastest)
    Immediate,
    /// Gradual rollback (safer)
    Gradual,
    /// Blue-green switch
    BlueGreenSwitch,
}

/// Automatic rollback triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRollbackTrigger {
    /// Trigger name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Grace period before triggering
    pub grace_period_seconds: u64,
}

/// Rollback plan definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Rollback strategy
    pub strategy: RollbackStrategy,
    /// Maximum rollback time in minutes
    pub max_rollback_time_minutes: u64,
    /// Automatic rollback triggers
    pub auto_triggers: Vec<AutoRollbackTrigger>,
    /// Manual rollback steps
    pub manual_steps: Vec<String>,
    /// Validation steps after rollback
    pub post_rollback_validation: Vec<String>,
}

/// Migration plan definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Plan identifier
    pub plan_id: String,
    /// Plan name
    pub name: String,
    /// Description of the migration
    pub description: String,
    /// Source system identifier
    pub source_system: SystemIdentifier,
    /// Target system identifier
    pub target_system: SystemIdentifier,
    /// Migration strategy
    pub strategy: MigrationStrategy,
    /// Feature flag rollout configuration
    pub rollout_config: RolloutConfig,
    /// Validation rules
    pub validation_rules: Vec<String>,
    /// Rollback plan
    pub rollback_plan: Option<RollbackPlan>,
    /// Estimated duration in minutes
    pub estimated_duration_minutes: u64,
    /// Prerequisites that must be met
    pub prerequisites: Vec<String>,
    /// Created timestamp
    pub created_at: u64,
    /// Created by user/system
    pub created_by: String,
}

/// Migration execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationExecution {
    /// Execution identifier
    pub execution_id: String,
    /// Associated plan ID
    pub plan_id: String,
    /// Current status
    pub status: MigrationStatus,
    /// Current phase
    pub current_phase: MigrationPhase,
    /// Progress percentage (0.0 to 100.0)
    pub progress_percentage: f64,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp (if completed)
    pub end_time: Option<u64>,
    /// Last update timestamp
    pub last_update: u64,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Error messages
    pub error_messages: Vec<String>,
    /// Rollback count
    pub rollback_count: u32,
    /// Executed by user/system
    pub executed_by: String,
}

/// Migration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Execution ID
    pub execution_id: String,
    /// Success status
    pub success: bool,
    /// Final status
    pub final_status: MigrationStatus,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error messages
    pub error_messages: Vec<String>,
    /// Created timestamp
    pub created_at: u64,
}

/// Migration controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationControllerConfig {
    /// Enable migration controller
    pub enabled: bool,
    /// Maximum concurrent migrations
    pub max_concurrent_migrations: u32,
    /// Default migration timeout in minutes
    pub default_timeout_minutes: u64,
    /// Rollback timeout in minutes
    pub rollback_timeout_minutes: u64,
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
    /// Enable automatic rollback
    pub enable_automatic_rollback: bool,
    /// Default rollback triggers
    pub default_rollback_triggers: Vec<AutoRollbackTrigger>,
}

impl Default for MigrationControllerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_concurrent_migrations: 3,
            default_timeout_minutes: 180, // 3 hours
            rollback_timeout_minutes: 30,
            health_check_interval_seconds: 60,
            enable_automatic_rollback: true,
            default_rollback_triggers: vec![
                AutoRollbackTrigger {
                    name: "high_error_rate".to_string(),
                    metric: "error_rate".to_string(),
                    threshold: 0.05, // 5%
                    operator: ComparisonOperator::GreaterThan,
                    grace_period_seconds: 300, // 5 minutes
                },
                AutoRollbackTrigger {
                    name: "high_latency".to_string(),
                    metric: "avg_latency_ms".to_string(),
                    threshold: 2000.0, // 2 seconds
                    operator: ComparisonOperator::GreaterThan,
                    grace_period_seconds: 180, // 3 minutes
                },
            ],
        }
    }
}

/// Migration Controller main implementation
pub struct MigrationController {
    /// Configuration
    config: MigrationControllerConfig,
    /// Feature flag migration manager
    #[allow(dead_code)]
    feature_flag_manager: Arc<FeatureFlagMigrationManager>,
    /// Migration plans
    migration_plans: Arc<Mutex<HashMap<String, MigrationPlan>>>,
    /// Active executions
    active_executions: Arc<Mutex<HashMap<String, MigrationExecution>>>,
    /// Execution history
    #[allow(dead_code)]
    execution_history: Arc<Mutex<Vec<MigrationResult>>>,
    /// Event history
    event_history: Arc<Mutex<Vec<MigrationEvent>>>,
    /// Logger instance
    logger: crate::utils::logger::Logger,
}

impl MigrationController {
    /// Create new migration controller
    pub async fn new(
        config: MigrationControllerConfig,
        feature_flag_manager: Arc<FeatureFlagMigrationManager>,
    ) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        let controller = Self {
            config,
            feature_flag_manager,
            migration_plans: Arc::new(Mutex::new(HashMap::new())),
            active_executions: Arc::new(Mutex::new(HashMap::new())),
            execution_history: Arc::new(Mutex::new(Vec::new())),
            event_history: Arc::new(Mutex::new(Vec::new())),
            logger,
        };

        controller.logger.info("Migration Controller initialized");
        Ok(controller)
    }

    /// Create a new migration plan
    pub async fn create_migration_plan(&self, mut plan: MigrationPlan) -> ArbitrageResult<String> {
        if !self.config.enabled {
            return Err(ArbitrageError::new(
                ErrorKind::ConfigError,
                "Migration controller is disabled".to_string(),
            ));
        }

        // Validate the plan
        self.validate_migration_plan(&plan)?;

        // Generate unique plan ID if not provided
        if plan.plan_id.is_empty() {
            plan.plan_id = Uuid::new_v4().to_string();
        }

        // Set creation timestamp
        plan.created_at = chrono::Utc::now().timestamp_millis() as u64;

        // Store the plan
        {
            let mut plans = self.migration_plans.lock().unwrap();
            plans.insert(plan.plan_id.clone(), plan.clone());
        }

        // Log event
        self.log_migration_event(
            plan.plan_id.clone(),
            MigrationEventType::MigrationStarted,
            format!("Created migration plan: {}", plan.name),
            EventSeverity::Info,
            HashMap::new(),
        )
        .await?;

        self.logger.info(&format!(
            "Created migration plan: {} ({})",
            plan.name, plan.plan_id
        ));
        Ok(plan.plan_id)
    }

    /// Execute a migration plan
    pub async fn execute_migration(
        &self,
        _env: &Env,
        plan_id: &str,
        executed_by: &str,
    ) -> ArbitrageResult<String> {
        // Get the plan
        let plan = {
            let plans = self.migration_plans.lock().unwrap();
            plans
                .get(plan_id)
                .ok_or_else(|| {
                    ArbitrageError::new(
                        ErrorKind::NotFound,
                        format!("Migration plan not found: {}", plan_id),
                    )
                })?
                .clone()
        };

        // Check concurrent execution limits
        self.enforce_execution_limits()?;

        // Check prerequisites
        self.check_prerequisites(&plan).await?;

        // Create execution tracking
        let execution_id = Uuid::new_v4().to_string();
        let execution = MigrationExecution {
            execution_id: execution_id.clone(),
            plan_id: plan_id.to_string(),
            status: MigrationStatus::InProgress,
            current_phase: MigrationPhase::Disabled,
            progress_percentage: 0.0,
            start_time: chrono::Utc::now().timestamp_millis() as u64,
            end_time: None,
            last_update: chrono::Utc::now().timestamp_millis() as u64,
            performance_metrics: PerformanceMetrics::default(),
            error_messages: Vec::new(),
            rollback_count: 0,
            executed_by: executed_by.to_string(),
        };

        // Store execution
        {
            let mut executions = self.active_executions.lock().unwrap();
            executions.insert(execution_id.clone(), execution);
        }

        self.logger.info(&format!(
            "Started migration execution: {} for plan: {}",
            execution_id, plan_id
        ));

        Ok(execution_id)
    }

    /// Get migration status
    pub async fn get_migration_status(
        &self,
        execution_id: &str,
    ) -> ArbitrageResult<MigrationExecution> {
        let executions = self.active_executions.lock().unwrap();
        executions.get(execution_id).cloned().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::NotFound,
                format!("Execution not found: {}", execution_id),
            )
        })
    }

    /// Get all active migrations
    pub async fn get_active_migrations(&self) -> Vec<MigrationExecution> {
        let executions = self.active_executions.lock().unwrap();
        executions.values().cloned().collect()
    }

    /// Get migration plan
    pub async fn get_migration_plan(&self, plan_id: &str) -> ArbitrageResult<MigrationPlan> {
        let plans = self.migration_plans.lock().unwrap();
        plans.get(plan_id).cloned().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::NotFound,
                format!("Migration plan not found: {}", plan_id),
            )
        })
    }

    /// Get health status
    pub async fn get_health(&self) -> ComponentHealth {
        let active_count = {
            let executions = self.active_executions.lock().unwrap();
            executions.len()
        };

        let _total_plans = {
            let plans = self.migration_plans.lock().unwrap();
            plans.len()
        };

        let status = if self.config.enabled
            && active_count <= self.config.max_concurrent_migrations as usize
        {
            "healthy"
        } else if !self.config.enabled {
            "disabled"
        } else {
            "degraded"
        };

        let uptime = chrono::Utc::now().timestamp_millis() as u64
            - self.config.health_check_interval_seconds;
        let performance_score = if self.config.enabled
            && active_count <= self.config.max_concurrent_migrations as usize
        {
            100.0
        } else {
            0.0
        };
        let error_count = if self.config.enabled {
            active_count as u64
        } else {
            0
        };
        let warning_count = 0;
        let is_healthy = status == "healthy";

        ComponentHealth::new(
            is_healthy,
            "migration_controller".to_string(),
            uptime,
            performance_score,
            error_count as u32,
            warning_count,
        )
    }

    // ============= PRIVATE HELPER METHODS =============

    /// Validate migration plan
    fn validate_migration_plan(&self, plan: &MigrationPlan) -> ArbitrageResult<()> {
        if plan.name.is_empty() {
            return Err(ArbitrageError::validation_error(
                "Migration plan name cannot be empty".to_string(),
            ));
        }

        if plan.source_system == plan.target_system {
            return Err(ArbitrageError::validation_error(
                "Source and target systems cannot be the same".to_string(),
            ));
        }

        if plan.estimated_duration_minutes == 0 {
            return Err(ArbitrageError::validation_error(
                "Estimated duration must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Enforce execution limits
    fn enforce_execution_limits(&self) -> ArbitrageResult<()> {
        let active_count = {
            let executions = self.active_executions.lock().unwrap();
            executions.len()
        };

        if active_count >= self.config.max_concurrent_migrations as usize {
            return Err(ArbitrageError::new(
                ErrorKind::RateLimit,
                format!(
                    "Maximum concurrent migrations ({}) exceeded",
                    self.config.max_concurrent_migrations
                ),
            ));
        }

        Ok(())
    }

    /// Check prerequisites
    async fn check_prerequisites(&self, plan: &MigrationPlan) -> ArbitrageResult<()> {
        for prerequisite in &plan.prerequisites {
            self.logger
                .info(&format!("Checking prerequisite: {}", prerequisite));
            // In a real implementation, this would check actual prerequisites
            // For now, we'll assume all prerequisites are met
        }
        Ok(())
    }

    /// Log migration event
    async fn log_migration_event(
        &self,
        execution_id: String,
        event_type: MigrationEventType,
        message: String,
        severity: EventSeverity,
        data: HashMap<String, serde_json::Value>,
    ) -> ArbitrageResult<()> {
        let event = MigrationEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            event_type,
            system_id: SystemIdentifier::new(
                LegacySystemType::Custom("migration_controller".to_string()),
                execution_id,
                "1.0.0".to_string(),
            ),
            message,
            severity,
            data,
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
