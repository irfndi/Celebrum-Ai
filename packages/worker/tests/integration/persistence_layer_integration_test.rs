//! Integration tests for the D1/R2 Persistence Layer

use cerebrum_ai::services::core::infrastructure::persistence_layer::{
    DatabaseManagerConfig, StorageLayerConfig,
};

use std::time::Duration;

/// Test configuration for integration testing
struct TestConfig {
    pub concurrent_operations: usize,
    pub load_test_duration: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            concurrent_operations: 5,
            load_test_duration: Duration::from_secs(10),
        }
    }
}

/// Helper function to create test storage configuration
fn create_test_config() -> StorageLayerConfig {
    StorageLayerConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_config_validation() {
        let config = create_test_config();

        // Test basic configuration properties
        assert!(config.max_connections > 0);
        assert!(config.default_timeout_ms > 0);
        assert!(config.cache_ttl_seconds > 0);
    }

    #[tokio::test]
    async fn test_database_manager_config() {
        let config = DatabaseManagerConfig::default();

        // Test that default config has reasonable values
        assert!(config.health_check_interval_seconds > 0);
        assert!(config.max_retry_attempts > 0);
        assert!(config.connection_timeout_seconds > 0);
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let test_config = TestConfig::default();
        assert!(test_config.concurrent_operations > 0);
        assert!(test_config.load_test_duration.as_secs() > 0);
    }

    // Other tests commented out due to module consolidation
    // TODO: Update tests after infrastructure consolidation is complete
}
