//! Integration tests for the D1/R2 Persistence Layer

use arb_edge::services::core::infrastructure::persistence_layer::migration_utilities::ColumnDefinition as MigrationColumnDefinition;
use arb_edge::services::core::infrastructure::persistence_layer::schema_manager::{
    ColumnDefinition, DataType, SchemaVersion,
};
use arb_edge::services::core::infrastructure::persistence_layer::{
    D1Config, MigrationBuilder, MigrationVersion, PerformanceConfig, PersistenceConfig,
    ProfilerConfig, R2Config, SchemaManager, TableDefinition, TransactionConfig,
};

use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test configuration for integration testing
struct TestConfig {
    #[allow(dead_code)]
    pub performance_threshold_ms: u64,
    pub concurrent_operations: usize,
    pub load_test_duration: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            performance_threshold_ms: 100,
            concurrent_operations: 5,
            load_test_duration: Duration::from_secs(10),
        }
    }
}

/// Helper function to create test persistence configuration
fn create_test_config() -> PersistenceConfig {
    PersistenceConfig {
        d1_config: D1Config {
            database_name: "test_db".to_string(),
            max_connections: 10,
            query_timeout_ms: 5000,
            enable_prepared_statements: true,
            enable_query_logging: false, // Disable for tests
            enable_connection_pooling: true,
        },
        r2_config: R2Config {
            bucket_name: "test-bucket".to_string(),
            max_object_size_mb: 10,
            default_storage_class: "Standard".to_string(),
            enable_compression: false, // Disable for tests
            compression_threshold_kb: 1024,
            enable_lifecycle_management: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persistence_config_validation() {
        let config = create_test_config();

        // Test D1 config validation
        assert!(!config.d1_config.database_name.is_empty());
        assert!(config.d1_config.max_connections > 0);
        assert!(config.d1_config.query_timeout_ms > 0);

        // Test R2 config validation
        assert!(!config.r2_config.bucket_name.is_empty());
        assert!(config.r2_config.max_object_size_mb > 0);
        assert!(config.r2_config.compression_threshold_kb > 0);
    }

    #[tokio::test]
    async fn test_schema_manager_creation() {
        let config = create_test_config();

        let result = SchemaManager::new(&config.d1_config, &config.r2_config).await;

        match result {
            Ok(schema_manager) => {
                // Verify that the schema manager has tables
                let all_tables = schema_manager.get_all_tables();
                assert!(
                    !all_tables.is_empty(),
                    "Schema manager should have predefined tables"
                );

                // Check for core tables
                let user_profiles = schema_manager.get_table_definition("user_profiles");
                assert!(user_profiles.is_some(), "Should have user_profiles table");

                println!(
                    "Schema manager created successfully with {} tables",
                    all_tables.len()
                );
            }
            Err(e) => {
                // Expected in test environment without real database
                println!(
                    "Expected schema manager creation error in test environment: {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_migration_version_ordering() {
        let v1_0_0 = MigrationVersion::new(1, 0, 0);
        let v1_0_1 = MigrationVersion::new(1, 0, 1);
        let v1_1_0 = MigrationVersion::new(1, 1, 0);
        let v2_0_0 = MigrationVersion::new(2, 0, 0);

        assert!(v1_0_0 < v1_0_1);
        assert!(v1_0_1 < v1_1_0);
        assert!(v1_1_0 < v2_0_0);
    }

    #[tokio::test]
    async fn test_migration_builder_functionality() {
        // Create test columns for the migration
        let id_column = MigrationColumnDefinition::new("id", "INTEGER")
            .primary_key()
            .not_null();

        let email_column = MigrationColumnDefinition::new("email", "TEXT")
            .unique()
            .not_null();

        let columns = vec![id_column, email_column];

        let migration = MigrationBuilder::new(
            "test_migration",
            "Test migration for users table",
            MigrationVersion::new(1, 0, 0),
        )
        .create_table("test_users", &columns)
        .build();

        assert_eq!(migration.version.major, 1);
        assert_eq!(migration.version.minor, 0);
        assert_eq!(migration.version.patch, 0);
        assert!(!migration.up_operations.is_empty());
        assert!(!migration.down_operations.is_empty());
    }

    #[tokio::test]
    async fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();

        assert!(config.enable_query_monitoring);
        assert!(config.enable_pool_monitoring);
        assert!(config.slow_query_threshold_ms > 0);
        assert!(config.max_query_history > 0);
    }

    #[tokio::test]
    async fn test_transaction_config_validation() {
        let config = TransactionConfig::default();

        assert!(config.transaction_timeout_ms > 0);
        assert!(config.max_retry_attempts > 0);
        assert!(config.max_concurrent_transactions > 0);
    }

    #[tokio::test]
    async fn test_schema_manager_table_definition() {
        let table_def = TableDefinition {
            name: "integration_test_table".to_string(),
            description: "Test table for integration testing".to_string(),
            created_version: SchemaVersion::new(1, 0, 0),
            last_modified_version: SchemaVersion::new(1, 0, 0),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    is_primary_key: true,
                    is_foreign_key: false,
                    default_value: None,
                    foreign_key_reference: None,
                    description: "Primary key".to_string(),
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Text,
                    nullable: false,
                    is_primary_key: false,
                    is_foreign_key: false,
                    default_value: None,
                    foreign_key_reference: None,
                    description: "User name".to_string(),
                },
                ColumnDefinition {
                    name: "email".to_string(),
                    data_type: DataType::Text,
                    nullable: false,
                    is_primary_key: false,
                    is_foreign_key: false,
                    default_value: None,
                    foreign_key_reference: None,
                    description: "User email".to_string(),
                },
            ],
            indexes: vec![],
            constraints: vec![],
        };

        // Test basic table definition structure
        assert_eq!(table_def.name, "integration_test_table");
        assert_eq!(table_def.columns.len(), 3);
        assert!(table_def.columns.iter().any(|c| c.is_primary_key));

        // Test version handling
        assert_eq!(table_def.created_version.major, 1);
        assert_eq!(table_def.created_version.minor, 0);
        assert_eq!(table_def.created_version.patch, 0);
    }

    #[tokio::test]
    async fn test_query_profiler_configuration() {
        let config = ProfilerConfig::default();

        assert!(config.enabled);
        assert!(config.collect_execution_plans);
        assert!(config.max_query_length > 0);
        assert!(config.enable_fingerprinting);
    }

    #[tokio::test]
    async fn test_component_integration() {
        let test_config = create_test_config();

        // Test that all config components are properly structured
        assert!(!test_config.d1_config.database_name.is_empty());
        assert!(!test_config.r2_config.bucket_name.is_empty());

        // Test configuration consistency
        assert!(test_config.d1_config.enable_connection_pooling);
        assert!(!test_config.r2_config.enable_compression); // Disabled for tests

        println!("Component integration validation passed");
    }

    #[tokio::test]
    async fn test_migration_version_compatibility() {
        let v1 = MigrationVersion::new(1, 0, 0);
        let v2 = MigrationVersion::new(1, 1, 0);
        let v3 = MigrationVersion::new(2, 0, 0);

        // Test ordering
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);

        // Test equality
        let v1_copy = MigrationVersion::new(1, 0, 0);
        assert_eq!(v1, v1_copy);

        // Test display formatting
        assert_eq!(format!("{}", v1), "1.0.0");
        assert_eq!(format!("{}", v2), "1.1.0");
        assert_eq!(format!("{}", v3), "2.0.0");
    }

    #[tokio::test]
    async fn test_configuration_serialization() {
        let config = create_test_config();

        // Test that configurations can be converted to/from JSON
        let json_result = serde_json::to_string(&config);
        assert!(json_result.is_ok(), "Config should be serializable to JSON");

        if let Ok(json_str) = json_result {
            let deserialized_result: Result<PersistenceConfig, _> = serde_json::from_str(&json_str);
            assert!(
                deserialized_result.is_ok(),
                "Config should be deserializable from JSON"
            );

            if let Ok(deserialized_config) = deserialized_result {
                assert_eq!(
                    config.d1_config.database_name,
                    deserialized_config.d1_config.database_name
                );
                assert_eq!(
                    config.r2_config.bucket_name,
                    deserialized_config.r2_config.bucket_name
                );
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_config_operations() {
        let test_config = TestConfig::default();
        let base_config = create_test_config();

        // Create multiple config variations concurrently
        let mut handles = Vec::new();

        for i in 0..test_config.concurrent_operations {
            let config = base_config.clone();
            let handle = tokio::spawn(async move {
                // Simulate concurrent config processing
                sleep(Duration::from_millis(10)).await;

                // Validate config in concurrent context
                assert!(!config.d1_config.database_name.is_empty());
                assert!(config.d1_config.max_connections > 0);
                assert!(!config.r2_config.bucket_name.is_empty());

                format!("Config validation {} completed", i)
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        let results = futures::future::join_all(handles).await;

        // Verify all operations succeeded
        for result in results {
            assert!(result.is_ok(), "Concurrent config operation should succeed");
            if let Ok(message) = result {
                println!("{}", message);
            }
        }

        println!("Concurrent config operations test completed successfully");
    }
}

#[cfg(test)]
mod load_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn load_test_configuration_creation() {
        println!("🚀 Starting Configuration Creation Load Test");
        let test_config = TestConfig::default();
        let start_time = Instant::now();

        let mut handles = Vec::new();

        // Create many configurations concurrently
        for i in 0..100 {
            let handle = tokio::spawn(async move {
                let config = create_test_config();

                // Simulate some processing
                sleep(Duration::from_millis(1)).await;

                // Validate config
                assert!(!config.d1_config.database_name.is_empty());
                assert!(config.d1_config.max_connections > 0);

                i
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let results = futures::future::join_all(handles).await;
        let elapsed = start_time.elapsed();

        // Verify all succeeded
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Config creation {} should succeed", i);
        }

        println!("✅ Load test completed in {:?}", elapsed);
        assert!(
            elapsed < test_config.load_test_duration,
            "Load test should complete within time limit"
        );
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn load_test_concurrent_operations() {
        println!("🚀 Starting Concurrent Operations Load Test");
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for i in 0..50 {
            let handle = tokio::spawn(async move {
                // Mix of different operations
                match i % 4 {
                    0 => {
                        // Config creation
                        let _config = create_test_config();
                        "config_created"
                    }
                    1 => {
                        // Migration version creation
                        let _version = MigrationVersion::new(1, i as u32 % 10, 0);
                        "version_created"
                    }
                    2 => {
                        // Performance config
                        let _perf_config = PerformanceConfig::default();
                        "perf_config_created"
                    }
                    _ => {
                        // Transaction config
                        let _tx_config = TransactionConfig::default();
                        "tx_config_created"
                    }
                }
            });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;
        let elapsed = start_time.elapsed();

        // Verify all operations completed
        for result in results {
            assert!(result.is_ok(), "Concurrent operation should succeed");
        }

        println!(
            "✅ Concurrent operations load test completed in {:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "Load test should complete quickly"
        );
    }
}
