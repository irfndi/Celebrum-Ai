//! Automated Cleanup System
//!
//! Comprehensive automated data cleanup system with TTL-based policies,
//! manual controls, storage monitoring, and impact analysis.

use serde::{Deserialize, Serialize};

// Core cleanup modules
pub mod cleanup_scheduler;
pub mod cleanup_validation;
pub mod impact_analysis;
pub mod policy_management;
pub mod storage_analytics;

// Re-export main types for external use
pub use cleanup_scheduler::{
    CleanupPolicy, CleanupResult, CleanupScheduler, CleanupSchedulerConfig, CleanupScope,
    CleanupStatus, CleanupStrategy,
};

pub use storage_analytics::{
    AlertSeverity, AlertThresholds, AlertType, CostSettings, OverallStorageMetrics, StorageAlert,
    StorageAnalyticsConfig, StorageAnalyticsDashboard, StorageAnalyticsService, StorageDataPoint,
    StorageUsageMetrics, StorageUsageTrend, TopConsumer, TrendDirection,
};

pub use impact_analysis::{
    ActionType, AnalysisStatus, CleanupImpactAnalysisEngine, DataDependency, DependencyGraph,
    DependencyType, ImpactAnalysisConfig, ImpactAnalysisRequest, ImpactAnalysisResult, ImpactLevel,
    RecommendedAction, RiskAssessment, SafetyCheck, SafetyCheckStatus, SafetyCheckType,
};

pub use policy_management::{
    AuditDetails, AuditEventType, BackupRequirement, CleanupAction,
    CleanupPolicy as PolicyCleanupPolicy, CleanupRule, CreatePolicyRequest, ExecutionError,
    ExecutionMetrics, ExecutionStatus, ListPoliciesRequest, ListPoliciesResponse, NetworkMetrics,
    NotificationChannel, PaginationRequest, PaginationResponse, ParameterType, ParameterValidation,
    PolicyAuditEntry, PolicyConfiguration, PolicyEvent, PolicyExecutionResult, PolicyFilter,
    PolicyManagementConfig, PolicyManagementInterface, PolicyMetadata, PolicyNotifications,
    PolicyRetryConfig, PolicySafetyConfig, PolicySchedule, PolicyStatus, PolicyTarget,
    PolicyTemplate, PolicyType, PolicyValidation, RateLimitState, RequestMetadata, RetryDelay,
    RetryInfo, RuleCondition, ScheduleType, SortDirection, SortOptions, StorageMetrics,
    TemplateMetadata, TemplateParameter, TestingRequirements, UpdatePolicyRequest, UserContext,
    ValidationRule, ValidationRuleType,
};

pub use cleanup_validation::{
    AuditDetailLevel, AuditTrailConfig, BackupIntegrityMethod, BackupVerificationConfig,
    ChaosTestingConfig, CleanupValidationConfig, CleanupValidator, ComparisonOperator,
    ComplianceValidationConfig, DriftDetectionConfig, DriftDetectionMethod, ExpectedResult,
    FailureHandlingStrategy, FailureScenario, LoadTestingConfig, NetworkIOMetrics,
    PerformanceTestConfig, PerformanceThresholds, PrivacyComplianceConfig, RollbackScenario,
    RollbackTestingConfig, SafetyCheckConfig, StorageIOMetrics, TestPerformanceMetrics,
    TestPriority, TestResult, TestStatus, ValidationError, ValidationMetrics, ValidationResult,
    ValidationRetryConfig, ValidationStatus, ValidationSuite, ValidationSuiteConfig,
    ValidationTestCase, ValidationTestType,
};

/// Automated cleanup system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedCleanupConfig {
    /// System enabled status
    pub enabled: bool,
    /// Cleanup scheduler configuration
    pub scheduler_config: cleanup_scheduler::CleanupSchedulerConfig,
    /// Storage analytics configuration
    pub analytics_config: storage_analytics::StorageAnalyticsConfig,
    /// Impact analysis configuration
    pub impact_analysis_config: impact_analysis::ImpactAnalysisConfig,
    /// Policy management configuration
    pub policy_management_config: policy_management::PolicyManagementConfig,
    /// Cleanup validation configuration
    pub validation_config: cleanup_validation::CleanupValidationConfig,
}

impl Default for AutomatedCleanupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scheduler_config: cleanup_scheduler::CleanupSchedulerConfig::default(),
            analytics_config: storage_analytics::StorageAnalyticsConfig::default(),
            impact_analysis_config: impact_analysis::ImpactAnalysisConfig::default(),
            policy_management_config: policy_management::PolicyManagementConfig::default(),
            validation_config: cleanup_validation::CleanupValidationConfig::default(),
        }
    }
}

/// Main automated cleanup system
#[derive(Debug)]
pub struct AutomatedCleanupSystem {
    config: AutomatedCleanupConfig,
    cleanup_scheduler: Option<CleanupScheduler>,
    storage_analytics: Option<StorageAnalyticsService>,
    impact_analyzer: Option<CleanupImpactAnalysisEngine>,
    policy_manager: Option<PolicyManagementInterface>,
    cleanup_validator: Option<CleanupValidator>,
    is_initialized: bool,
}

impl AutomatedCleanupSystem {
    /// Create a new automated cleanup system
    pub fn new(config: AutomatedCleanupConfig) -> Self {
        Self {
            config,
            cleanup_scheduler: None,
            storage_analytics: None,
            impact_analyzer: None,
            policy_manager: None,
            cleanup_validator: None,
            is_initialized: false,
        }
    }

    /// Initialize the automated cleanup system
    pub async fn initialize(
        &mut self,
        connection_manager: std::sync::Arc<
            crate::services::core::infrastructure::persistence::ConnectionManager,
        >,
        transaction_coordinator: std::sync::Arc<
            crate::services::core::infrastructure::persistence::TransactionCoordinator,
        >,
    ) -> crate::utils::error::ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Initialize cleanup scheduler
        if self.config.scheduler_config.enabled {
            let scheduler = CleanupScheduler::new(
                self.config.scheduler_config.clone(),
                connection_manager.clone(),
                transaction_coordinator.clone(),
            )
            .await?;
            self.cleanup_scheduler = Some(scheduler);
        }

        // Initialize storage analytics
        if self.config.analytics_config.enabled {
            let analytics = StorageAnalyticsService::new(
                self.config.analytics_config.clone(),
                connection_manager.clone(),
                transaction_coordinator.clone(),
            )
            .await?;
            self.storage_analytics = Some(analytics);
        }

        // Initialize impact analyzer
        let analyzer = CleanupImpactAnalysisEngine::new(
            self.config.impact_analysis_config.clone(),
            connection_manager.clone(),
            transaction_coordinator.clone(),
        );
        self.impact_analyzer = Some(analyzer);

        // Initialize policy manager
        if self.config.policy_management_config.enabled {
            let policy_manager = PolicyManagementInterface::new(
                self.config.policy_management_config.clone(),
                connection_manager.clone(),
                transaction_coordinator.clone(),
            )
            .await?;
            self.policy_manager = Some(policy_manager);
        }

        // Initialize cleanup validator
        if self.config.validation_config.enabled {
            let validator = CleanupValidator::new(
                self.config.validation_config.clone(),
                connection_manager.clone(),
                transaction_coordinator.clone(),
            )
            .await?;
            self.cleanup_validator = Some(validator);
        }

        self.is_initialized = true;
        Ok(())
    }

    /// Start the automated cleanup system
    pub async fn start(&self, env: &worker::Env) -> crate::utils::error::ArbitrageResult<()> {
        if !self.is_initialized {
            return Err(crate::utils::error::ArbitrageError::new(
                crate::utils::error::ErrorKind::Internal,
                "Automated cleanup system not initialized".to_string(),
            ));
        }

        // Start all components
        if let Some(scheduler) = &self.cleanup_scheduler {
            scheduler.start(env).await?;
        }

        if let Some(analytics) = &self.storage_analytics {
            analytics.start(env).await?;
        }

        // Impact analyzer doesn't need explicit start - it's stateless

        if let Some(policy_manager) = &self.policy_manager {
            policy_manager.start(env).await?;
        }

        if let Some(validator) = &self.cleanup_validator {
            validator.start(env).await?;
        }

        Ok(())
    }

    /// Stop the automated cleanup system
    pub async fn stop(&self) -> crate::utils::error::ArbitrageResult<()> {
        // Stop all components
        if let Some(scheduler) = &self.cleanup_scheduler {
            scheduler.stop().await?;
        }

        if let Some(analytics) = &self.storage_analytics {
            analytics.stop().await?;
        }

        // Impact analyzer doesn't need explicit stop - it's stateless

        if let Some(policy_manager) = &self.policy_manager {
            policy_manager.stop().await?;
        }

        if let Some(validator) = &self.cleanup_validator {
            validator.stop().await?;
        }

        Ok(())
    }

    /// Check if the system is healthy
    pub async fn is_healthy(&self) -> crate::utils::error::ArbitrageResult<bool> {
        if !self.is_initialized {
            return Ok(false);
        }

        // Check all components
        if let Some(scheduler) = &self.cleanup_scheduler {
            if !scheduler.is_healthy().await? {
                return Ok(false);
            }
        }

        if let Some(analytics) = &self.storage_analytics {
            if !analytics.is_healthy().await? {
                return Ok(false);
            }
        }

        if let Some(analyzer) = &self.impact_analyzer {
            if analyzer.health_check().await.is_err() {
                return Ok(false);
            }
        }

        if let Some(policy_manager) = &self.policy_manager {
            if !policy_manager.is_healthy().await? {
                return Ok(false);
            }
        }

        if let Some(validator) = &self.cleanup_validator {
            if !validator.is_healthy().await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get cleanup scheduler reference
    pub fn get_cleanup_scheduler(&self) -> Option<&CleanupScheduler> {
        self.cleanup_scheduler.as_ref()
    }

    /// Get storage analytics reference
    pub fn get_storage_analytics(&self) -> Option<&StorageAnalyticsService> {
        self.storage_analytics.as_ref()
    }

    /// Get impact analyzer reference
    pub fn get_impact_analyzer(&self) -> Option<&CleanupImpactAnalysisEngine> {
        self.impact_analyzer.as_ref()
    }

    /// Get policy manager reference
    pub fn get_policy_manager(&self) -> Option<&PolicyManagementInterface> {
        self.policy_manager.as_ref()
    }

    /// Get cleanup validator reference
    pub fn get_cleanup_validator(&self) -> Option<&CleanupValidator> {
        self.cleanup_validator.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_automated_cleanup_system_creation() {
        let config = AutomatedCleanupConfig::default();
        let system = AutomatedCleanupSystem::new(config);

        assert!(!system.is_initialized);
        assert!(system.cleanup_scheduler.is_none());
        assert!(system.storage_analytics.is_none());
        assert!(system.impact_analyzer.is_none());
        assert!(system.policy_manager.is_none());
        assert!(system.cleanup_validator.is_none());
    }
}
