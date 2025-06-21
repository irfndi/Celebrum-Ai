use crate::services::core::infrastructure::persistence_layer::database_manager::{
    DatabaseManagerConfig, RepositoryRegistry, RepositoryRegistration, DatabaseHealthSummary
};
use crate::services::core::infrastructure::shared_types::current_timestamp_ms;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_manager_config_validation() {
        let mut config = DatabaseManagerConfig::default();
        assert!(config.validate().is_ok());

        config.health_check_interval_seconds = 0;
        assert!(config.validate().is_err());

        config.health_check_interval_seconds = 30;
        config.max_retry_attempts = 0;
        assert!(config.validate().is_err());

        config.max_retry_attempts = 3;
        config.connection_timeout_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_repository_registry() {
        let registry = RepositoryRegistry::new();
        assert_eq!(registry.get_repository_names().len(), 0);

        // Note: We can't easily test repository registration without implementing
        // a mock repository that implements the Repository trait
    }

    #[test]
    fn test_repository_registration() {
        let registration = RepositoryRegistration {
            name: "test_repo".to_string(),
            repository_type: "TestRepository".to_string(),
            version: "1.0.0".to_string(),
            description: "Test repository".to_string(),
            is_critical: true,
            auto_initialize: true,
            dependencies: vec!["dependency1".to_string()],
            configuration: HashMap::new(),
        };

        assert_eq!(registration.name, "test_repo");
        assert_eq!(registration.repository_type, "TestRepository");
        assert!(registration.is_critical);
        assert!(registration.auto_initialize);
        assert_eq!(registration.dependencies.len(), 1);
    }

    #[test]
    fn test_database_health_summary() {
        let health_summary = DatabaseHealthSummary {
            overall_healthy: true,
            total_repositories: 2,
            healthy_repositories: 2,
            unhealthy_repositories: 0,
            health_percentage: 100.0,
            last_updated: current_timestamp_ms(),
            repository_health: HashMap::new(),
        };

        assert!(health_summary.overall_healthy);
        assert_eq!(health_summary.total_repositories, 2);
        assert_eq!(health_summary.healthy_repositories, 2);
        assert_eq!(health_summary.unhealthy_repositories, 0);
        assert_eq!(health_summary.health_percentage, 100.0);
    }
}