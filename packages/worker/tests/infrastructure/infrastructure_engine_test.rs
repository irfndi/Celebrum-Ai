//! Tests for infrastructure_engine module
//! Extracted from src/services/core/infrastructure/infrastructure_engine.rs

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use cerebrum_ai::services::core::infrastructure::infrastructure_engine::*;
    use std::collections::HashMap;

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
