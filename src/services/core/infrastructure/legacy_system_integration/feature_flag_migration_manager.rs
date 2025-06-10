//! Feature Flag Migration Manager - Part 1: Core Types and Enums

use super::shared_types::{
    EventSeverity, LegacySystemType, MigrationEvent, MigrationEventType, PerformanceMetrics,
    SystemIdentifier,
};
use crate::services::core::infrastructure::shared_types::ComponentHealth;
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use worker::Env;

/// Migration phases for progressive rollouts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationPhase {
    /// Initial disabled state
    Disabled,
    /// Canary rollout (small percentage)
    Canary,
    /// Gradual rollout (increasing percentage)
    Gradual,
    /// Full rollout (100%)
    Full,
    /// Rollback phase
    Rollback,
    /// Migration completed
    Completed,
}

impl std::fmt::Display for MigrationPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationPhase::Disabled => write!(f, "disabled"),
            MigrationPhase::Canary => write!(f, "canary"),
            MigrationPhase::Gradual => write!(f, "gradual"),
            MigrationPhase::Full => write!(f, "full"),
            MigrationPhase::Rollback => write!(f, "rollback"),
            MigrationPhase::Completed => write!(f, "completed"),
        }
    }
}

/// Rollout strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RolloutStrategy {
    /// Manual progression - requires explicit progression commands
    Manual,
    /// Time-based progression - automatic progression based on time intervals
    TimeBased {
        interval_minutes: u64,
        increment_percentage: f64,
    },
    /// Metrics-based progression - automatic progression based on performance metrics
    MetricsBased {
        success_rate_threshold: f64,
        latency_threshold_ms: f64,
        error_rate_threshold: f64,
        observation_window_minutes: u64,
    },
}

impl Default for RolloutStrategy {
    fn default() -> Self {
        RolloutStrategy::MetricsBased {
            success_rate_threshold: 0.99,
            latency_threshold_ms: 1000.0,
            error_rate_threshold: 0.01,
            observation_window_minutes: 15,
        }
    }
}

/// Safety threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyThreshold {
    /// Maximum allowed error rate (0.0 to 1.0)
    pub max_error_rate: f64,
    /// Maximum allowed latency in milliseconds
    pub max_latency_ms: f64,
    /// Minimum required success rate (0.0 to 1.0)
    pub min_success_rate: f64,
    /// Enable automatic rollback on threshold breach
    pub auto_rollback_enabled: bool,
    /// Grace period before triggering rollback (seconds)
    pub rollback_grace_period_seconds: u64,
}

impl Default for SafetyThreshold {
    fn default() -> Self {
        Self {
            max_error_rate: 0.05,   // 5% max error rate
            max_latency_ms: 1000.0, // 1 second max latency
            min_success_rate: 0.95, // 95% min success rate
            auto_rollback_enabled: true,
            rollback_grace_period_seconds: 60,
        }
    }
}

/// Rollout configuration for a specific migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutConfig {
    /// Migration identifier
    pub migration_id: String,
    /// Target system
    pub target_system: SystemIdentifier,
    /// Rollout strategy
    pub strategy: RolloutStrategy,
    /// Safety thresholds
    pub safety_threshold: SafetyThreshold,
    /// Initial rollout percentage
    pub initial_percentage: f64,
    /// Maximum rollout percentage
    pub max_percentage: f64,
    /// Increment size for gradual rollouts
    pub increment_percentage: f64,
    /// Enable feature flags
    pub feature_flags: HashMap<String, bool>,
    /// Custom configuration parameters
    pub custom_config: HashMap<String, serde_json::Value>,
}

impl Default for RolloutConfig {
    fn default() -> Self {
        Self {
            migration_id: Uuid::new_v4().to_string(),
            target_system: SystemIdentifier::new(
                LegacySystemType::Custom("default".to_string()),
                "default".to_string(),
                "1.0.0".to_string(),
            ),
            strategy: RolloutStrategy::default(),
            safety_threshold: SafetyThreshold::default(),
            initial_percentage: 5.0,
            max_percentage: 100.0,
            increment_percentage: 10.0,
            feature_flags: HashMap::new(),
            custom_config: HashMap::new(),
        }
    }
}

/// Current rollout progress and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutProgress {
    /// Migration identifier
    pub migration_id: String,
    /// Current phase
    pub phase: MigrationPhase,
    /// Current rollout percentage
    pub current_percentage: f64,
    /// Start timestamp
    pub start_time: u64,
    /// Last update timestamp
    pub last_update: u64,
    /// Next scheduled update (for time-based strategies)
    pub next_update: Option<u64>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Safety threshold violations
    pub threshold_violations: Vec<String>,
    /// Rollback count
    pub rollback_count: u32,
    /// Success status
    pub is_successful: bool,
    /// Error messages
    pub error_messages: Vec<String>,
}

/// Migration feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationFeatureFlags {
    /// Enable feature flag migration manager
    pub feature_flag_migration_manager: bool,
    /// Enable percentage-based rollouts
    pub percentage_based_rollouts: bool,
    /// Enable automated progression
    pub automated_progression: bool,
    /// Enable safety thresholds
    pub safety_thresholds: bool,
    /// Enable rollback automation
    pub rollback_automation: bool,
    /// Enable performance monitoring
    pub performance_monitoring: bool,
    /// Enable migration metrics
    pub migration_metrics: bool,
    /// Maximum concurrent migrations
    pub max_concurrent_migrations: u32,
    /// Default rollout increment percentage
    pub rollout_increment_percentage: f64,
    /// Safety threshold error rate
    pub safety_threshold_error_rate: f64,
    /// Safety threshold latency in milliseconds
    pub safety_threshold_latency_ms: f64,
    /// Automatic rollback on threshold breach
    pub automatic_rollback_on_threshold: bool,
}

impl Default for MigrationFeatureFlags {
    fn default() -> Self {
        Self {
            feature_flag_migration_manager: false,
            percentage_based_rollouts: false,
            automated_progression: false,
            safety_thresholds: false,
            rollback_automation: false,
            performance_monitoring: false,
            migration_metrics: false,
            max_concurrent_migrations: 3,
            rollout_increment_percentage: 10.0,
            safety_threshold_error_rate: 0.05,
            safety_threshold_latency_ms: 1000.0,
            automatic_rollback_on_threshold: true,
        }
    }
}

/// Migration feature configuration for a specific feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationFeatureConfig {
    /// Feature name
    pub feature_name: String,
    /// Enable feature
    pub enabled: bool,
    /// Rollout percentage for this feature
    pub rollout_percentage: f64,
    /// Target users or systems (empty for all)
    pub target_identifiers: Vec<String>,
    /// Feature-specific configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
}

/// Feature Flag Migration Manager main implementation
pub struct FeatureFlagMigrationManager {
    /// Configuration
    config: MigrationFeatureFlags,
    /// Active rollout configurations
    rollout_configs: Arc<Mutex<HashMap<String, RolloutConfig>>>,
    /// Rollout progress tracking
    rollout_progress: Arc<Mutex<HashMap<String, RolloutProgress>>>,
    /// Feature configurations
    #[allow(dead_code)]
    feature_configs: Arc<Mutex<HashMap<String, MigrationFeatureConfig>>>,
    /// Performance metrics cache
    metrics_cache: Arc<Mutex<HashMap<String, PerformanceMetrics>>>,
    /// Event history
    event_history: Arc<Mutex<Vec<MigrationEvent>>>,
    /// Logger instance
    logger: crate::utils::logger::Logger,
}

impl FeatureFlagMigrationManager {
    /// Create new feature flag migration manager
    pub async fn new(config: MigrationFeatureFlags) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        let manager = Self {
            config,
            rollout_configs: Arc::new(Mutex::new(HashMap::new())),
            rollout_progress: Arc::new(Mutex::new(HashMap::new())),
            feature_configs: Arc::new(Mutex::new(HashMap::new())),
            metrics_cache: Arc::new(Mutex::new(HashMap::new())),
            event_history: Arc::new(Mutex::new(Vec::new())),
            logger,
        };

        manager
            .logger
            .info("Feature Flag Migration Manager initialized");
        Ok(manager)
    }

    /// Load configuration from feature flags JSON
    pub async fn load_from_feature_flags(&mut self, __env: &Env) -> ArbitrageResult<()> {
        if !self.config.feature_flag_migration_manager {
            return Ok(());
        }

        // In a real implementation, this would load from a KV store or external source
        // For now, we'll use the default configuration
        self.logger
            .info("Loading feature flag configuration from external source");

        // This would typically load from:
        // - Cloudflare KV store
        // - External configuration service
        // - Environment variables
        // - Database

        Ok(())
    }

    /// Create a new migration rollout
    pub async fn create_rollout(&self, rollout_config: RolloutConfig) -> ArbitrageResult<String> {
        if !self.config.feature_flag_migration_manager {
            return Err(ArbitrageError::config_error(
                "Feature flag migration manager is disabled",
            ));
        }

        let migration_id = rollout_config.migration_id.clone();

        // Validate configuration
        self.validate_rollout_config(&rollout_config)?;

        // Check concurrent migration limits
        self.enforce_migration_limits()?;

        // Store rollout configuration
        {
            let mut configs = self.rollout_configs.lock().unwrap();
            configs.insert(migration_id.clone(), rollout_config.clone());
        }

        // Initialize progress tracking
        let progress = RolloutProgress {
            migration_id: migration_id.clone(),
            phase: MigrationPhase::Disabled,
            current_percentage: 0.0,
            start_time: chrono::Utc::now().timestamp_millis() as u64,
            last_update: chrono::Utc::now().timestamp_millis() as u64,
            next_update: None,
            performance_metrics: PerformanceMetrics::default(),
            threshold_violations: Vec::new(),
            rollback_count: 0,
            is_successful: false,
            error_messages: Vec::new(),
        };

        {
            let mut progress_map = self.rollout_progress.lock().unwrap();
            progress_map.insert(migration_id.clone(), progress);
        }

        // Log event
        self.log_migration_event(
            migration_id.clone(),
            MigrationEventType::MigrationStarted,
            format!("Created rollout for migration: {}", migration_id),
            EventSeverity::Info,
            HashMap::new(),
        )
        .await?;

        self.logger
            .info(&format!("Created migration rollout: {}", migration_id));
        Ok(migration_id)
    }

    /// Start a migration rollout
    pub async fn start_rollout(&self, migration_id: &str) -> ArbitrageResult<()> {
        let rollout_config = {
            let configs = self.rollout_configs.lock().unwrap();
            configs
                .get(migration_id)
                .ok_or_else(|| {
                    ArbitrageError::infrastructure_error(format!(
                        "Migration not found: {}",
                        migration_id
                    ))
                })?
                .clone()
        };

        // Update progress to canary phase
        {
            let mut progress_map = self.rollout_progress.lock().unwrap();
            if let Some(progress) = progress_map.get_mut(migration_id) {
                progress.phase = MigrationPhase::Canary;
                progress.current_percentage = rollout_config.initial_percentage;
                progress.last_update = chrono::Utc::now().timestamp_millis() as u64;

                // Set next update time for time-based strategies
                if let RolloutStrategy::TimeBased {
                    interval_minutes, ..
                } = &rollout_config.strategy
                {
                    progress.next_update =
                        Some(progress.last_update + (interval_minutes * 60 * 1000));
                }
            }
        }

        // Apply feature flags for initial percentage
        self.apply_feature_flags(migration_id, rollout_config.initial_percentage)
            .await?;

        // Log event
        self.log_migration_event(
            migration_id.to_string(),
            MigrationEventType::MigrationStarted,
            format!(
                "Started rollout with {}% traffic",
                rollout_config.initial_percentage
            ),
            EventSeverity::Info,
            HashMap::new(),
        )
        .await?;

        self.logger.info(&format!(
            "Started migration rollout: {} at {}%",
            migration_id, rollout_config.initial_percentage
        ));

        Ok(())
    }

    /// Progress to next rollout phase
    pub async fn progress_rollout(&self, migration_id: &str) -> ArbitrageResult<MigrationPhase> {
        let rollout_config = {
            let configs = self.rollout_configs.lock().unwrap();
            configs
                .get(migration_id)
                .ok_or_else(|| {
                    ArbitrageError::infrastructure_error(format!(
                        "Migration not found: {}",
                        migration_id
                    ))
                })?
                .clone()
        };

        let (current_phase, current_percentage) = {
            let progress_map = self.rollout_progress.lock().unwrap();
            let progress = progress_map.get(migration_id).ok_or_else(|| {
                ArbitrageError::infrastructure_error(format!(
                    "Migration progress not found: {}",
                    migration_id
                ))
            })?;
            (progress.phase.clone(), progress.current_percentage)
        };

        // Calculate next percentage and phase
        let next_percentage = (current_percentage + rollout_config.increment_percentage)
            .min(rollout_config.max_percentage);

        let next_phase = match current_phase {
            MigrationPhase::Disabled => MigrationPhase::Canary,
            MigrationPhase::Canary => {
                if next_percentage >= rollout_config.max_percentage {
                    MigrationPhase::Full
                } else {
                    MigrationPhase::Gradual
                }
            }
            MigrationPhase::Gradual => {
                if next_percentage >= rollout_config.max_percentage {
                    MigrationPhase::Full
                } else {
                    MigrationPhase::Gradual
                }
            }
            MigrationPhase::Full => MigrationPhase::Completed,
            _ => {
                return Err(ArbitrageError::infrastructure_error(format!(
                    "Cannot progress from phase: {}",
                    current_phase
                )))
            }
        };

        // Check safety thresholds before progressing
        if self.config.safety_thresholds {
            self.check_safety_thresholds(migration_id, &rollout_config.safety_threshold)
                .await?;
        }

        // Update progress
        {
            let mut progress_map = self.rollout_progress.lock().unwrap();
            if let Some(progress) = progress_map.get_mut(migration_id) {
                progress.phase = next_phase.clone();
                progress.current_percentage = next_percentage;
                progress.last_update = chrono::Utc::now().timestamp_millis() as u64;

                if next_phase == MigrationPhase::Completed {
                    progress.is_successful = true;
                }
            }
        }

        // Apply new feature flags
        self.apply_feature_flags(migration_id, next_percentage)
            .await?;

        // Log event
        self.log_migration_event(
            migration_id.to_string(),
            MigrationEventType::MigrationCompleted,
            format!(
                "Progressed to phase {} with {}% traffic",
                next_phase, next_percentage
            ),
            EventSeverity::Info,
            HashMap::new(),
        )
        .await?;

        self.logger.info(&format!(
            "Progressed migration {} to phase {} at {}%",
            migration_id, next_phase, next_percentage
        ));

        Ok(next_phase)
    }

    /// Rollback a migration
    pub async fn rollback_migration(
        &self,
        migration_id: &str,
        reason: &str,
    ) -> ArbitrageResult<()> {
        // Update progress to rollback phase
        {
            let mut progress_map = self.rollout_progress.lock().unwrap();
            if let Some(progress) = progress_map.get_mut(migration_id) {
                progress.phase = MigrationPhase::Rollback;
                progress.current_percentage = 0.0;
                progress.last_update = chrono::Utc::now().timestamp_millis() as u64;
                progress.rollback_count += 1;
                progress.error_messages.push(reason.to_string());
            }
        }

        // Disable all feature flags for this migration
        self.apply_feature_flags(migration_id, 0.0).await?;

        // Log event
        let mut event_data = HashMap::new();
        event_data.insert(
            "reason".to_string(),
            serde_json::Value::String(reason.to_string()),
        );

        self.log_migration_event(
            migration_id.to_string(),
            MigrationEventType::RollbackStarted,
            format!("Rolled back migration: {}", reason),
            EventSeverity::Warning,
            event_data,
        )
        .await?;

        self.logger.warn(&format!(
            "Rolled back migration {}: {}",
            migration_id, reason
        ));

        Ok(())
    }

    /// Check if a feature is enabled for a given identifier
    pub async fn is_feature_enabled(
        &self,
        migration_id: &str,
        feature_name: &str,
        identifier: &str,
    ) -> ArbitrageResult<bool> {
        let progress = {
            let progress_map = self.rollout_progress.lock().unwrap();
            progress_map.get(migration_id).cloned()
        };

        let progress = match progress {
            Some(p) => p,
            None => return Ok(false), // Migration not found, feature disabled
        };

        // Check if migration is in a state where features should be enabled
        match progress.phase {
            MigrationPhase::Disabled | MigrationPhase::Rollback => return Ok(false),
            MigrationPhase::Completed => return Ok(true),
            _ => {} // Continue with percentage-based check
        }

        // For percentage-based rollouts, use a hash-based approach for consistent results
        let hash_input = format!("{}:{}:{}", migration_id, feature_name, identifier);
        let hash = self.calculate_hash(&hash_input);
        let percentage_threshold = (hash % 100) as f64;

        Ok(percentage_threshold < progress.current_percentage)
    }

    /// Update performance metrics for a migration
    pub async fn update_performance_metrics(
        &self,
        migration_id: &str,
        metrics: PerformanceMetrics,
    ) -> ArbitrageResult<()> {
        // Update metrics cache
        {
            let mut cache = self.metrics_cache.lock().unwrap();
            cache.insert(migration_id.to_string(), metrics.clone());
        }

        // Update progress metrics
        {
            let mut progress_map = self.rollout_progress.lock().unwrap();
            if let Some(progress) = progress_map.get_mut(migration_id) {
                progress.performance_metrics = metrics;
                progress.last_update = chrono::Utc::now().timestamp_millis() as u64;
            }
        }

        // Check safety thresholds if enabled
        if self.config.safety_thresholds {
            let rollout_config = {
                let configs = self.rollout_configs.lock().unwrap();
                configs.get(migration_id).cloned()
            };

            if let Some(config) = rollout_config {
                if let Err(e) = self
                    .check_safety_thresholds(migration_id, &config.safety_threshold)
                    .await
                {
                    self.logger.warn(&format!(
                        "Safety threshold check failed for migration {}: {}",
                        migration_id, e
                    ));

                    if config.safety_threshold.auto_rollback_enabled {
                        self.rollback_migration(migration_id, &e.to_string())
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get migration status
    pub async fn get_migration_status(
        &self,
        migration_id: &str,
    ) -> ArbitrageResult<RolloutProgress> {
        let progress_map = self.rollout_progress.lock().unwrap();
        progress_map.get(migration_id).cloned().ok_or_else(|| {
            ArbitrageError::infrastructure_error(format!("Migration not found: {}", migration_id))
        })
    }

    /// Get health status
    pub async fn get_health(&self) -> ComponentHealth {
        let _total_migrations = {
            let progress_map = self.rollout_progress.lock().unwrap();
            progress_map.len()
        };

        let active_migrations = {
            let progress_map = self.rollout_progress.lock().unwrap();
            progress_map
                .values()
                .filter(|p| {
                    !matches!(
                        p.phase,
                        MigrationPhase::Completed | MigrationPhase::Disabled
                    )
                })
                .count()
        };

        let _status = if active_migrations <= self.config.max_concurrent_migrations as usize {
            "healthy"
        } else {
            "degraded"
        };

        ComponentHealth::new(
            true, // is_healthy
            "Feature Flag Migration Manager".to_string(),
            0,   // uptime_seconds
            1.0, // performance_score
            0,   // error_count
            0,   // warning_count
        )
    }

    // ============= PRIVATE HELPER METHODS =============

    /// Validate rollout configuration
    fn validate_rollout_config(&self, config: &RolloutConfig) -> ArbitrageResult<()> {
        if config.initial_percentage < 0.0 || config.initial_percentage > 100.0 {
            return Err(ArbitrageError::validation_error(
                "Initial percentage must be between 0 and 100".to_string(),
            ));
        }

        if config.max_percentage < config.initial_percentage {
            return Err(ArbitrageError::validation_error(
                "Max percentage must be >= initial percentage".to_string(),
            ));
        }

        if config.increment_percentage <= 0.0 {
            return Err(ArbitrageError::validation_error(
                "Increment percentage must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Enforce concurrent migration limits
    fn enforce_migration_limits(&self) -> ArbitrageResult<()> {
        let active_count = {
            let progress_map = self.rollout_progress.lock().unwrap();
            progress_map
                .values()
                .filter(|p| {
                    !matches!(
                        p.phase,
                        MigrationPhase::Completed | MigrationPhase::Disabled
                    )
                })
                .count()
        };

        if active_count >= self.config.max_concurrent_migrations as usize {
            return Err(ArbitrageError::rate_limit_error(format!(
                "Maximum concurrent migrations ({}) exceeded",
                self.config.max_concurrent_migrations
            )));
        }

        Ok(())
    }

    /// Apply feature flags for a given percentage
    async fn apply_feature_flags(
        &self,
        migration_id: &str,
        percentage: f64,
    ) -> ArbitrageResult<()> {
        // In a real implementation, this would update the actual feature flag store
        // For now, we'll just log the action
        self.logger.info(&format!(
            "Applied feature flags for migration {} at {}% rollout",
            migration_id, percentage
        ));
        Ok(())
    }

    /// Check safety thresholds
    async fn check_safety_thresholds(
        &self,
        migration_id: &str,
        threshold: &SafetyThreshold,
    ) -> ArbitrageResult<()> {
        let metrics = {
            let cache = self.metrics_cache.lock().unwrap();
            cache.get(migration_id).cloned()
        };

        let metrics = match metrics {
            Some(m) => m,
            None => return Ok(()), // No metrics available, skip check
        };

        let mut violations = Vec::new();

        if metrics.error_count > 0 && metrics.total_operations > 0 {
            let error_rate = metrics.error_count as f64 / metrics.total_operations as f64;
            if error_rate > threshold.max_error_rate {
                violations.push(format!(
                    "Error rate {} exceeds threshold {}",
                    error_rate, threshold.max_error_rate
                ));
            }
        }

        if metrics.latency_ms > threshold.max_latency_ms {
            violations.push(format!(
                "Latency {} ms exceeds threshold {} ms",
                metrics.latency_ms, threshold.max_latency_ms
            ));
        }

        let success_rate = if metrics.total_operations > 0 {
            (metrics.total_operations - metrics.error_count) as f64
                / metrics.total_operations as f64
        } else {
            1.0
        };

        if success_rate < threshold.min_success_rate {
            violations.push(format!(
                "Success rate {} below threshold {}",
                success_rate, threshold.min_success_rate
            ));
        }

        if !violations.is_empty() {
            // Store violations in progress
            {
                let mut progress_map = self.rollout_progress.lock().unwrap();
                if let Some(progress) = progress_map.get_mut(migration_id) {
                    progress.threshold_violations.extend(violations.clone());
                }
            }

            return Err(ArbitrageError::infrastructure_error(format!(
                "Safety threshold violations: {}",
                violations.join(", ")
            )));
        }

        Ok(())
    }

    /// Log migration event
    async fn log_migration_event(
        &self,
        migration_id: String,
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
                LegacySystemType::Custom("feature_flag_manager".to_string()),
                migration_id,
                "1.0.0".to_string(),
            ),
            message,
            severity,
            data,
        };

        {
            let mut history = self.event_history.lock().unwrap();
            history.push(event);

            // Keep only the last 1000 events to prevent memory leaks
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }

        Ok(())
    }

    /// Calculate hash for consistent percentage-based decisions
    fn calculate_hash(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}
