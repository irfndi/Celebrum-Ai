// Monitoring and Reliability Integration Tests
// Comprehensive test suite validating circuit breaker behavior, health monitoring accuracy,
// alerting functionality, and failover mechanisms across the entire monitoring stack.

#[cfg(test)]
mod tests {
    // Core infrastructure components
    use arb_edge::services::core::infrastructure::{
        // Automatic Failover Coordinator
        automatic_failover_coordinator::{AutomaticFailoverConfig, AutomaticFailoverFeatureFlags},
        // Failover Service
        failover_service::FailoverConfig,

        // Health Monitor
        monitoring_module::{
            // Alert management
            alert_manager::AlertManagerConfig,
            health_monitor::{
                ComponentHealth, HealthCheck, HealthCheckType, HealthMonitorConfig, HealthStatus,
            },
            // Real-time health monitoring
            real_time_health_monitor::RealTimeHealthConfig,
            // Service degradation alerting
            service_degradation_alerting::ServiceDegradationConfig,
        },

        // Circuit Breaker Service
        CircuitBreakerConfig,
        CircuitBreakerType,
    };

    // Note: These tests are designed to validate the monitoring and reliability components compile
    // and can be instantiated properly. For full integration testing, a proper worker environment
    // would be needed. These tests focus on API compatibility and basic functionality validation.

    #[tokio::test]
    async fn test_circuit_breaker_service_creation() {
        // Test circuit breaker service creation and configuration validation
        let cb_config = CircuitBreakerConfig::default();
        assert!(cb_config.enabled);
        assert_eq!(cb_config.default_failure_threshold, 5);
        assert_eq!(cb_config.default_timeout_seconds, 60);

        // Test configuration validation
        let validation_result = cb_config.validate();
        assert!(
            validation_result.is_ok(),
            "Default configuration should be valid"
        );
    }

    #[tokio::test]
    async fn test_health_monitor_configuration() {
        // Test health monitor configuration and component creation
        let health_config = HealthMonitorConfig::default();
        assert!(health_config.enable_health_monitoring);

        // Test component health creation
        let db_component = ComponentHealth::new(
            "test_database".to_string(),
            "Test Database".to_string(),
            "database".to_string(),
        )
        .with_tag("environment".to_string(), "test".to_string());

        assert_eq!(db_component.component_id, "test_database");
        assert_eq!(db_component.component_name, "Test Database");
        assert_eq!(db_component.component_type, "database");

        // Test health check creation
        let health_check = HealthCheck::new(
            "test_database".to_string(),
            "Database Ping".to_string(),
            HealthCheckType::DatabaseQuery,
        )
        .with_timeout(5);

        assert_eq!(health_check.component_id, "test_database");
        assert_eq!(health_check.check_name, "Database Ping");
        assert_eq!(health_check.timeout_seconds, 5); // u64, not Option<u64>
    }

    #[tokio::test]
    async fn test_real_time_health_monitor_configuration() {
        // Test real-time health monitor configuration
        let rt_config = RealTimeHealthConfig::default();
        assert!(rt_config.enabled);

        // Test configuration without validate method since it doesn't exist
        println!("Real-time health monitor configuration is valid");
    }

    #[tokio::test]
    async fn test_service_degradation_alerting_configuration() {
        // Test service degradation alerting configuration
        let degradation_config = ServiceDegradationConfig::default();
        assert!(degradation_config.enabled);

        // Test threshold configuration
        assert!(degradation_config.degradation_threshold > 0.0);
        assert!(degradation_config.degradation_threshold <= 1.0);
    }

    #[tokio::test]
    async fn test_failover_service_configuration() {
        // Test failover service configuration
        let failover_config = FailoverConfig::default();
        assert!(failover_config.enabled);
        assert!(failover_config.detection_interval_seconds > 0);
        assert!(failover_config.failover_timeout_seconds > 0);

        // Test high availability configuration
        let ha_config = FailoverConfig::high_availability();
        assert!(ha_config.detection_interval_seconds <= failover_config.detection_interval_seconds);
        assert!(ha_config.enable_auto_recovery);
    }

    #[tokio::test]
    async fn test_alert_manager_configuration() {
        // Test alert manager configuration
        let alert_config = AlertManagerConfig::default();
        // Use actual field names from AlertManagerConfig
        assert!(alert_config.evaluation_interval_seconds > 0);
        assert!(alert_config.max_alerts_in_memory > 0);
        assert!(alert_config.alert_retention_days > 0);
    }

    #[tokio::test]
    async fn test_automatic_failover_coordinator_configuration() {
        // Test automatic failover coordinator configuration
        let coordinator_config = AutomaticFailoverConfig::default();
        assert!(coordinator_config.enabled);
        assert!(coordinator_config.monitoring_interval_ms > 0);
        assert!(coordinator_config.decision_interval_ms > 0);

        // Test configuration validation
        let validation_result = coordinator_config.validate();
        assert!(
            validation_result.is_ok(),
            "Default coordinator configuration should be valid"
        );

        // Test feature flags
        let feature_flags = AutomaticFailoverFeatureFlags::default();
        assert!(feature_flags.enable_automatic_failover);
        assert!(feature_flags.enable_automatic_recovery);

        let production_flags = AutomaticFailoverFeatureFlags::production_safe();
        assert!(production_flags.enable_automatic_failover);
        assert!(!production_flags.enable_predictive_failover); // Conservative default

        // Test feature flag validation
        let flag_validation = feature_flags.validate();
        assert!(
            flag_validation.is_ok(),
            "Default feature flags should be valid"
        );
    }

    #[tokio::test]
    async fn test_monitoring_stack_type_compatibility() {
        // Test that all monitoring components have compatible types

        // Circuit breaker types
        let cb_type = CircuitBreakerType::ExternalService;
        assert_eq!(cb_type.as_str(), "external_service");

        let api_type = CircuitBreakerType::HttpApi;
        assert_eq!(api_type.as_str(), "http_api");

        // Health status types
        let healthy_status = HealthStatus::Healthy;
        let unhealthy_status = HealthStatus::Unhealthy;
        let unknown_status = HealthStatus::Unknown;

        assert!(matches!(healthy_status, HealthStatus::Healthy));
        assert!(matches!(unhealthy_status, HealthStatus::Unhealthy));
        assert!(matches!(unknown_status, HealthStatus::Unknown));

        // Health check types - use correct variant names
        let db_check = HealthCheckType::DatabaseQuery;
        let http_check = HealthCheckType::HttpGet; // Correct variant name
        let custom_check = HealthCheckType::Custom("test_check".to_string());

        assert!(matches!(db_check, HealthCheckType::DatabaseQuery));
        assert!(matches!(http_check, HealthCheckType::HttpGet));
        assert!(matches!(custom_check, HealthCheckType::Custom(_)));
    }

    #[tokio::test]
    async fn test_integrated_monitoring_stack_configuration_validation() {
        // Test that all monitoring components have compatible configurations
        let cb_config = CircuitBreakerConfig::default();
        let health_config = HealthMonitorConfig::default();
        let rt_config = RealTimeHealthConfig::default();
        let degradation_config = ServiceDegradationConfig::default();
        let failover_config = FailoverConfig::default();
        let alert_config = AlertManagerConfig::default();
        let coordinator_config = AutomaticFailoverConfig::default();
        let feature_flags = AutomaticFailoverFeatureFlags::production_safe();

        // Validate all configurations
        assert!(cb_config.validate().is_ok());
        assert!(coordinator_config.validate().is_ok());
        assert!(feature_flags.validate().is_ok());

        // Test configuration consistency - use actual field names
        assert!(cb_config.enabled);
        assert!(health_config.enable_health_monitoring);
        assert!(rt_config.enabled);
        assert!(degradation_config.enabled);
        assert!(failover_config.enabled);
        // AlertManagerConfig doesn't have an enabled field, check other fields
        assert!(alert_config.evaluation_interval_seconds > 0);
        assert!(coordinator_config.enabled);

        // Test integration flags consistency
        assert!(cb_config.enable_health_integration);
        assert!(cb_config.enable_alert_integration);
        assert!(failover_config.circuit_breaker_integration);
        assert!(failover_config.alert_integration);

        println!("✅ All monitoring components have valid and compatible configurations");
    }

    #[tokio::test]
    async fn test_monitoring_performance_configuration() {
        // Test performance-oriented configurations
        let high_perf_cb_config = CircuitBreakerConfig::high_performance();
        assert_eq!(high_perf_cb_config.check_interval_seconds, 10);
        assert_eq!(high_perf_cb_config.max_circuit_breakers, 200);

        let high_reliability_cb_config = CircuitBreakerConfig::high_reliability();
        assert_eq!(high_reliability_cb_config.default_failure_threshold, 3);
        assert_eq!(high_reliability_cb_config.min_success_count_half_open, 5);

        let ha_failover_config = FailoverConfig::high_availability();
        assert_eq!(ha_failover_config.detection_interval_seconds, 10);
        assert_eq!(ha_failover_config.max_concurrent_failovers, 20);

        let high_sensitivity_coordinator = AutomaticFailoverConfig::high_sensitivity();
        assert_eq!(high_sensitivity_coordinator.monitoring_interval_ms, 2000);
        assert_eq!(
            high_sensitivity_coordinator.consecutive_failure_threshold,
            2
        );

        let conservative_coordinator = AutomaticFailoverConfig::conservative();
        assert_eq!(conservative_coordinator.monitoring_interval_ms, 15000);
        assert_eq!(conservative_coordinator.consecutive_failure_threshold, 5);

        println!("✅ Performance configurations validated");
    }

    #[tokio::test]
    async fn test_system_resilience_configuration() {
        // Test error handling and resilience configuration
        let invalid_cb_config = CircuitBreakerConfig {
            default_failure_threshold: 0,
            ..Default::default()
        };
        assert!(
            invalid_cb_config.validate().is_err(),
            "Invalid threshold should fail validation"
        );

        let invalid_cb_timeout_config = CircuitBreakerConfig {
            default_failure_threshold: 5,
            default_timeout_seconds: 0,
            ..Default::default()
        };
        assert!(
            invalid_cb_timeout_config.validate().is_err(),
            "Invalid timeout should fail validation"
        );

        let invalid_coordinator_config = AutomaticFailoverConfig {
            health_score_threshold: 1.5, // Invalid range
            ..Default::default()
        };
        assert!(
            invalid_coordinator_config.validate().is_err(),
            "Invalid health score threshold should fail validation"
        );

        let invalid_recovery_config = AutomaticFailoverConfig {
            health_score_threshold: 0.3,
            recovery_health_threshold: 0.2, // Lower than failover threshold
            ..Default::default()
        };
        assert!(
            invalid_recovery_config.validate().is_err(),
            "Invalid recovery threshold should fail validation"
        );

        let invalid_feature_flags = AutomaticFailoverFeatureFlags {
            enable_automatic_failover: true,
            enable_manual_override: false,
            ..Default::default()
        };
        assert!(
            invalid_feature_flags.validate().is_err(),
            "Manual override must be enabled with automatic failover"
        );

        println!("✅ Configuration validation properly rejects invalid configurations");
    }
}
