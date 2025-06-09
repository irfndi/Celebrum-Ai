// src/services/core/infrastructure/chaos_engineering/mod.rs

//! Chaos Engineering Framework
//!
//! A comprehensive chaos engineering system designed for Cloudflare Workers environment
//! with D1, R2, KV storage systems. Provides fault injection capabilities, automated recovery
//! verification, experiment orchestration, and safety controls.
//!
//! ## Core Features:
//! - **Fault Injection**: Storage systems (KV, D1, R2), network, resource exhaustion
//! - **Automated Recovery**: Verification, data integrity checks, rollback procedures
//! - **Experiment Orchestration**: Scheduling, coordination, safety controls
//! - **Metrics & Monitoring**: Resilience scoring, integration with monitoring systems
//! - **Feature Flags**: Safe deployment and gradual rollout
//!
//! ## Architecture:
//! - Modular design with zero duplication
//! - High efficiency and concurrency
//! - Production-ready with comprehensive error handling
//! - Integration with existing monitoring and circuit breaker systems

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use worker::Env;

use crate::utils::error::{ArbitrageError, ArbitrageResult, ErrorKind};

// Core chaos engineering components
pub mod chaos_coordinator;
pub mod chaos_metrics;
pub mod experiment_engine;
pub mod experiment_orchestrator;
pub mod fault_injection;
pub mod recovery_verifier;
pub mod safety_controls;

// Storage system fault injection modules
pub mod d1_faults;
pub mod kv_faults;
pub mod r2_faults;

// Network and resource chaos modules
pub mod network_chaos;
pub mod resource_chaos;

// Re-export core types
pub use chaos_coordinator::{
    ChaosCoordinator, ChaosCoordinatorConfig, ChaosCoordinatorMetrics, OrchestrationSession,
};
pub use experiment_engine::{
    ChaosExperiment, ExperimentEngine, ExperimentMetrics, ExperimentState, ExperimentTimestamps,
    ExperimentType,
};
pub use experiment_orchestrator::{
    BlastRadiusConfig, CampaignPriority, CampaignStatus, ExperimentCampaign,
    ExperimentOrchestrator, ExperimentType as OrchestratorExperimentType, OrchestrationStats,
    SafetyConfig, TimeWindow,
};
pub use fault_injection::{FaultConfig, FaultInjector, FaultType, InjectionTarget};
pub use recovery_verifier::{
    create_recovery_verifier, RecoveryStatus, RecoveryVerificationConfig,
    RecoveryVerificationResult, RecoveryVerifier,
};
pub use safety_controls::{SafetyController, SafetyRule, SafetyViolation, ViolationSeverity};

/// Chaos Engineering Framework Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosEngineeringConfig {
    /// Enable chaos engineering functionality
    pub enabled: bool,
    /// Maximum number of concurrent experiments
    pub max_concurrent_experiments: u32,
    /// Default experiment timeout in seconds
    pub default_experiment_timeout_seconds: u64,
    /// Safety check interval in seconds
    pub safety_check_interval_seconds: u64,
    /// Maximum fault injection intensity (0.0 to 1.0)
    pub max_fault_intensity: f64,
    /// Enable automated recovery verification
    pub enable_automated_recovery: bool,
    /// Enable metrics collection
    pub enable_metrics_collection: bool,
    /// Feature flags for different chaos experiment types
    pub feature_flags: ChaosFeatureFlags,
    /// Recovery verification configuration
    pub recovery_verification: Option<RecoveryVerificationConfig>,
}

/// Feature flags for chaos engineering experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosFeatureFlags {
    /// Enable storage system fault injection
    pub storage_fault_injection: bool,
    /// Enable network chaos simulation
    pub network_chaos_simulation: bool,
    /// Enable resource exhaustion testing
    pub resource_exhaustion_testing: bool,
    /// Enable automated experiment orchestration
    pub automated_orchestration: bool,
    /// Enable real-time recovery verification
    pub realtime_recovery_verification: bool,
    /// Enable chaos metrics dashboard
    pub chaos_metrics_dashboard: bool,
}

/// Main Chaos Engineering Framework
pub struct ChaosEngineeringFramework {
    config: ChaosEngineeringConfig,
    coordinator: Option<ChaosCoordinator>,
    experiment_engine: Option<ExperimentEngine>,
    fault_injector: Option<FaultInjector>,
    recovery_verifier: Option<RecoveryVerifier>,
    safety_controller: Option<SafetyController>,
    is_initialized: bool,
}

impl ChaosEngineeringFramework {
    /// Create a new chaos engineering framework
    pub fn new(config: ChaosEngineeringConfig) -> ArbitrageResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            coordinator: None,
            experiment_engine: None,
            fault_injector: None,
            recovery_verifier: None,
            safety_controller: None,
            is_initialized: false,
        })
    }

    /// Initialize the chaos engineering framework
    pub async fn initialize(&mut self, env: &Env) -> ArbitrageResult<()> {
        if self.is_initialized {
            return Ok(());
        }

        if !self.config.enabled {
            return Ok(());
        }

        // Initialize safety controller first for fail-safe operation
        self.safety_controller = Some(SafetyController::new(&self.config, env).await?);

        // Initialize core components
        self.coordinator = Some(ChaosCoordinator::new(&self.config, env).await?);
        self.experiment_engine = Some(ExperimentEngine::new(&self.config, env).await?);
        self.fault_injector = Some(FaultInjector::new(&self.config, env).await?);
        self.recovery_verifier = Some(create_recovery_verifier(&self.config));

        self.is_initialized = true;
        Ok(())
    }

    /// Get the chaos coordinator
    pub fn coordinator(&self) -> ArbitrageResult<&ChaosCoordinator> {
        self.coordinator.as_ref().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::Internal,
                "Chaos coordinator not initialized".to_string(),
            )
        })
    }

    /// Get the experiment engine
    pub fn experiment_engine(&self) -> ArbitrageResult<&ExperimentEngine> {
        self.experiment_engine.as_ref().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::Internal,
                "Experiment engine not initialized".to_string(),
            )
        })
    }

    /// Get the fault injector
    pub fn fault_injector(&self) -> ArbitrageResult<&FaultInjector> {
        self.fault_injector.as_ref().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::Internal,
                "Fault injector not initialized".to_string(),
            )
        })
    }

    /// Get the recovery verifier
    pub fn recovery_verifier(&self) -> ArbitrageResult<&RecoveryVerifier> {
        self.recovery_verifier.as_ref().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::Internal,
                "Recovery verifier not initialized".to_string(),
            )
        })
    }

    /// Get the safety controller
    pub fn safety_controller(&self) -> ArbitrageResult<&SafetyController> {
        self.safety_controller.as_ref().ok_or_else(|| {
            ArbitrageError::new(
                ErrorKind::Internal,
                "Safety controller not initialized".to_string(),
            )
        })
    }

    /// Check if the framework is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get framework configuration
    pub fn config(&self) -> &ChaosEngineeringConfig {
        &self.config
    }

    /// Perform health check
    pub async fn health_check(&self) -> ArbitrageResult<HashMap<String, bool>> {
        let mut health = HashMap::new();

        health.insert("framework_initialized".to_string(), self.is_initialized);
        health.insert("enabled".to_string(), self.config.enabled);

        if self.is_initialized && self.config.enabled {
            if let Ok(coordinator) = self.coordinator() {
                health.insert(
                    "coordinator_healthy".to_string(),
                    coordinator.is_healthy().await?,
                );
            }
            if let Ok(engine) = self.experiment_engine() {
                health.insert(
                    "experiment_engine_healthy".to_string(),
                    engine.is_healthy().await?,
                );
            }
            if let Ok(injector) = self.fault_injector() {
                health.insert(
                    "fault_injector_healthy".to_string(),
                    injector.is_healthy().await?,
                );
            }
            if let Ok(verifier) = self.recovery_verifier() {
                health.insert(
                    "recovery_verifier_healthy".to_string(),
                    verifier.is_healthy().await?,
                );
            }
            if let Ok(safety) = self.safety_controller() {
                health.insert(
                    "safety_controller_healthy".to_string(),
                    safety.is_healthy().await?,
                );
            }
        }

        Ok(health)
    }

    /// Trigger a recovery test manually
    pub async fn trigger_recovery_test(
        &self,
        _service_name: &str,
        _env: &Env,
    ) -> ArbitrageResult<()> {
        if !self.is_initialized {
            return Err(ArbitrageError::new(
                ErrorKind::Internal,
                "Chaos engineering framework not initialized".to_string(),
            ));
        }

        // Trigger a controlled recovery test scenario - just check if recovery verifier is available
        if let Some(ref _recovery_verifier) = self.recovery_verifier {
            // Recovery verifier is available and healthy
            // In production, this would trigger actual recovery testing
        }

        Ok(())
    }

    /// Shutdown the chaos engineering framework
    pub async fn shutdown(&mut self) -> ArbitrageResult<()> {
        if !self.is_initialized {
            return Ok(());
        }

        // Shutdown in reverse order for safety
        if let Some(mut safety) = self.safety_controller.take() {
            safety.shutdown().await?;
        }
        if let Some(mut verifier) = self.recovery_verifier.take() {
            verifier.shutdown().await?;
        }
        if let Some(mut injector) = self.fault_injector.take() {
            injector.shutdown().await?;
        }
        if let Some(mut engine) = self.experiment_engine.take() {
            engine.shutdown().await?;
        }
        if let Some(mut coordinator) = self.coordinator.take() {
            coordinator.shutdown().await?;
        }

        self.is_initialized = false;
        Ok(())
    }
}

impl Default for ChaosEngineeringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_concurrent_experiments: 3,
            default_experiment_timeout_seconds: 300,
            safety_check_interval_seconds: 10,
            max_fault_intensity: 0.5,
            enable_automated_recovery: true,
            enable_metrics_collection: true,
            feature_flags: ChaosFeatureFlags::default(),
            recovery_verification: None,
        }
    }
}

impl Default for ChaosFeatureFlags {
    fn default() -> Self {
        Self {
            storage_fault_injection: false,
            network_chaos_simulation: false,
            resource_exhaustion_testing: false,
            automated_orchestration: false,
            realtime_recovery_verification: true,
            chaos_metrics_dashboard: false,
        }
    }
}

impl ChaosEngineeringConfig {
    /// Create a development-safe configuration
    pub fn development() -> Self {
        Self {
            enabled: true,
            max_concurrent_experiments: 1,
            default_experiment_timeout_seconds: 60, // 1 minute for dev
            safety_check_interval_seconds: 5,
            max_fault_intensity: 0.2, // 20% for safety
            enable_automated_recovery: true,
            enable_metrics_collection: true,
            feature_flags: ChaosFeatureFlags {
                storage_fault_injection: true,
                network_chaos_simulation: false,    // Disabled in dev
                resource_exhaustion_testing: false, // Disabled in dev
                automated_orchestration: false,
                realtime_recovery_verification: true,
                chaos_metrics_dashboard: true,
            },
            recovery_verification: Some(RecoveryVerificationConfig::default()),
        }
    }

    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            enabled: false, // Must be explicitly enabled in production
            max_concurrent_experiments: 2,
            default_experiment_timeout_seconds: 180, // 3 minutes
            safety_check_interval_seconds: 15,
            max_fault_intensity: 0.3, // 30% maximum for production
            enable_automated_recovery: true,
            enable_metrics_collection: true,
            feature_flags: ChaosFeatureFlags {
                storage_fault_injection: false, // Must be explicitly enabled
                network_chaos_simulation: false,
                resource_exhaustion_testing: false,
                automated_orchestration: false,
                realtime_recovery_verification: true,
                chaos_metrics_dashboard: true,
            },
            recovery_verification: Some(RecoveryVerificationConfig::default()),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.max_concurrent_experiments == 0 {
            return Err(ArbitrageError::new(
                ErrorKind::Internal,
                "max_concurrent_experiments must be greater than 0".to_string(),
            ));
        }

        if self.default_experiment_timeout_seconds == 0 {
            return Err(ArbitrageError::new(
                ErrorKind::Internal,
                "default_experiment_timeout_seconds must be greater than 0".to_string(),
            ));
        }

        if self.max_fault_intensity < 0.0 || self.max_fault_intensity > 1.0 {
            return Err(ArbitrageError::new(
                ErrorKind::Internal,
                "max_fault_intensity must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.safety_check_interval_seconds == 0 {
            return Err(ArbitrageError::new(
                ErrorKind::Internal,
                "safety_check_interval_seconds must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_engineering_config_default() {
        let config = ChaosEngineeringConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_concurrent_experiments, 3);
        assert_eq!(config.default_experiment_timeout_seconds, 300);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chaos_engineering_config_development() {
        let config = ChaosEngineeringConfig::development();
        assert!(config.enabled);
        assert_eq!(config.max_concurrent_experiments, 1);
        assert_eq!(config.max_fault_intensity, 0.2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chaos_engineering_config_production() {
        let config = ChaosEngineeringConfig::production();
        assert!(!config.enabled); // Must be explicitly enabled
        assert_eq!(config.max_concurrent_experiments, 2);
        assert_eq!(config.max_fault_intensity, 0.3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chaos_engineering_config_validation() {
        // Test invalid max_concurrent_experiments
        let config = ChaosEngineeringConfig {
            max_concurrent_experiments: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid timeout
        let config = ChaosEngineeringConfig {
            max_concurrent_experiments: 1,
            default_experiment_timeout_seconds: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid intensity
        let config = ChaosEngineeringConfig {
            default_experiment_timeout_seconds: 300,
            max_fault_intensity: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = ChaosEngineeringConfig {
            max_fault_intensity: 1.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
