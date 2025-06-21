//! Tests for core_architecture module
//! Extracted from src/utils/core_architecture.rs

#[cfg(test)]
mod tests {
    use cerebrum_ai::utils::core_architecture::*;

    #[tokio::test]
    async fn test_service_architecture_creation() {
        let mut architecture = CoreServiceArchitecture::new();
        assert_eq!(architecture.system_status, ServiceStatus::Stopped);

        architecture.initialize().await.unwrap();
        assert!(!architecture.startup_order.is_empty());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let mut architecture = CoreServiceArchitecture::new();

        let config = ServiceConfig {
            service_type: ServiceType::TelegramService,
            enabled: true,
            dependencies: vec![],
            health_check_interval_seconds: 30,
            restart_on_failure: true,
            max_restart_attempts: 3,
            configuration: serde_json::json!({}),
        };

        architecture.register_service(config).await.unwrap();

        let info = architecture
            .get_service_info(&ServiceType::TelegramService)
            .await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().status, ServiceStatus::Stopped);
    }

    #[tokio::test]
    async fn test_service_startup_order() {
        let mut architecture = CoreServiceArchitecture::new();
        architecture.initialize().await.unwrap();

        // Verify we have services in the startup order
        assert!(!architecture.startup_order.is_empty());

        // Find positions of key services
        let telegram_pos = architecture
            .startup_order
            .iter()
            .position(|s| s == &ServiceType::TelegramService);

        let notification_pos = architecture
            .startup_order
            .iter()
            .position(|s| s == &ServiceType::NotificationService);

        // Both services should exist in startup order
        assert!(
            telegram_pos.is_some(),
            "TelegramService should be in startup order"
        );
        assert!(
            notification_pos.is_some(),
            "NotificationService should be in startup order"
        );

        // Notification service should start before Telegram (since Telegram depends on it)
        assert!(notification_pos.unwrap() < telegram_pos.unwrap());
    }

    #[tokio::test]
    async fn test_health_check_result() {
        let healthy = HealthCheckResult::healthy(ServiceType::TelegramService, 150);
        assert_eq!(healthy.status, ServiceStatus::Healthy);
        assert_eq!(healthy.response_time_ms, Some(150));

        let unhealthy = HealthCheckResult::unhealthy(
            ServiceType::D1DatabaseService,
            "Connection failed".to_string(),
        );
        assert_eq!(unhealthy.status, ServiceStatus::Unhealthy);
        assert!(unhealthy.error_message.is_some());
    }

    #[tokio::test]
    async fn test_system_health_overview() {
        let mut architecture = CoreServiceArchitecture::new();
        architecture.initialize().await.unwrap();

        let health = architecture.get_system_health().await;
        assert!(health.total_services > 0);
        assert_eq!(health.overall_status, ServiceStatus::Stopped); // Not started yet
    }

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.service_type, ServiceType::TelegramService);
        assert!(config.enabled);
        assert_eq!(config.health_check_interval_seconds, 30);
        assert!(config.restart_on_failure);
        assert_eq!(config.max_restart_attempts, 3);
    }

    #[test]
    fn test_service_dependency() {
        let dep = ServiceDependency {
            service_type: ServiceType::D1DatabaseService,
            required: true,
            start_order: 1,
        };

        assert_eq!(dep.service_type, ServiceType::D1DatabaseService);
        assert!(dep.required);
        assert_eq!(dep.start_order, 1);
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let mut architecture = CoreServiceArchitecture::new();

        // Create services with circular dependencies: A -> B -> C -> A
        let config_a = ServiceConfig {
            service_type: ServiceType::TelegramService,
            enabled: true,
            dependencies: vec![ServiceDependency {
                service_type: ServiceType::NotificationService,
                required: true,
                start_order: 1,
            }],
            health_check_interval_seconds: 30,
            restart_on_failure: true,
            max_restart_attempts: 3,
            configuration: serde_json::json!({}),
        };

        let config_b = ServiceConfig {
            service_type: ServiceType::NotificationService,
            enabled: true,
            dependencies: vec![ServiceDependency {
                service_type: ServiceType::ExchangeService,
                required: true,
                start_order: 1,
            }],
            health_check_interval_seconds: 30,
            restart_on_failure: true,
            max_restart_attempts: 3,
            configuration: serde_json::json!({}),
        };

        let config_c = ServiceConfig {
            service_type: ServiceType::ExchangeService,
            enabled: true,
            dependencies: vec![ServiceDependency {
                service_type: ServiceType::TelegramService, // Circular dependency!
                required: true,
                start_order: 1,
            }],
            health_check_interval_seconds: 30,
            restart_on_failure: true,
            max_restart_attempts: 3,
            configuration: serde_json::json!({}),
        };

        architecture.register_service(config_a).await.unwrap();
        architecture.register_service(config_b).await.unwrap();
        architecture.register_service(config_c).await.unwrap();

        // This should detect the circular dependency
        let result = architecture.calculate_startup_order().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency detected"));
    }

    #[tokio::test]
    async fn test_service_validation() {
        // Test self-dependency validation
        let configs_with_self_dep = vec![ServiceConfig {
            service_type: ServiceType::TelegramService,
            enabled: true,
            dependencies: vec![ServiceDependency {
                service_type: ServiceType::TelegramService, // Self-dependency
                required: true,
                start_order: 1,
            }],
            health_check_interval_seconds: 30,
            restart_on_failure: true,
            max_restart_attempts: 3,
            configuration: serde_json::json!({}),
        }];

        let result = CoreServiceArchitecture::validate_service_configs(&configs_with_self_dep);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot depend on itself"));

        // Test missing dependency validation
        let configs_with_missing_dep = vec![ServiceConfig {
            service_type: ServiceType::TelegramService,
            enabled: true,
            dependencies: vec![ServiceDependency {
                service_type: ServiceType::NotificationService, // Missing service
                required: true,
                start_order: 1,
            }],
            health_check_interval_seconds: 30,
            restart_on_failure: true,
            max_restart_attempts: 3,
            configuration: serde_json::json!({}),
        }];

        let result = CoreServiceArchitecture::validate_service_configs(&configs_with_missing_dep);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not defined in the configuration"));

        // Test valid configuration
        let valid_configs = vec![
            ServiceConfig {
                service_type: ServiceType::D1DatabaseService,
                enabled: true,
                dependencies: vec![],
                health_check_interval_seconds: 30,
                restart_on_failure: true,
                max_restart_attempts: 3,
                configuration: serde_json::json!({}),
            },
            ServiceConfig {
                service_type: ServiceType::TelegramService,
                enabled: true,
                dependencies: vec![ServiceDependency {
                    service_type: ServiceType::D1DatabaseService,
                    required: true,
                    start_order: 1,
                }],
                health_check_interval_seconds: 30,
                restart_on_failure: true,
                max_restart_attempts: 3,
                configuration: serde_json::json!({}),
            },
        ];

        let result = CoreServiceArchitecture::validate_service_configs(&valid_configs);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_service_operations() {
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let architecture = Arc::new(Mutex::new(CoreServiceArchitecture::new()));
        let mut handles = vec![];

        // Test concurrent service registration
        for i in 0..5 {
            let arch_clone = Arc::clone(&architecture);
            let handle = tokio::spawn(async move {
                let mut arch = arch_clone.lock().await;
                let config = ServiceConfig {
                    service_type: match i {
                        0 => ServiceType::D1DatabaseService,
                        1 => ServiceType::ExchangeService,
                        2 => ServiceType::TelegramService,
                        3 => ServiceType::NotificationService,
                        _ => ServiceType::UserProfileService,
                    },
                    enabled: true,
                    dependencies: vec![],
                    health_check_interval_seconds: 30,
                    restart_on_failure: true,
                    max_restart_attempts: 3,
                    configuration: serde_json::json!({}),
                };
                arch.register_service(config).await
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Verify all services were registered
        let arch = architecture.lock().await;
        let health = arch.get_system_health().await;
        assert_eq!(health.total_services, 5);
    }
}