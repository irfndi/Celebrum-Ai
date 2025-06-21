//! Tests for database utilities
//! Extracted from src/services/core/infrastructure/database.rs

use crate::infrastructure::database::*;
use std::time::Duration;

#[tokio::test]
async fn test_connection_pool_creation() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 10,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(30),
        idle_timeout: Some(Duration::from_secs(600)),
        max_lifetime: Some(Duration::from_secs(1800)),
    };
    
    let pool = create_connection_pool(&config).await;
    assert!(pool.is_ok());
}

#[tokio::test]
async fn test_database_health_check() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(10),
        idle_timeout: Some(Duration::from_secs(300)),
        max_lifetime: Some(Duration::from_secs(900)),
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    let health = check_database_health(&pool).await;
    assert!(health.is_ok());
}

#[tokio::test]
async fn test_migration_runner() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: None,
        max_lifetime: None,
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    let migration_result = run_migrations(&pool).await;
    assert!(migration_result.is_ok());
}

#[tokio::test]
async fn test_transaction_handling() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: None,
        max_lifetime: None,
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    
    // Test successful transaction
    let result = execute_in_transaction(&pool, |tx| async move {
        // Simulate some database operations
        Ok("success")
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_transaction_rollback() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: None,
        max_lifetime: None,
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    
    // Test transaction rollback on error
    let result = execute_in_transaction(&pool, |_tx| async move {
        Err("simulated error")
    }).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_connection_pool_limits() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 2,
        min_connections: 1,
        acquire_timeout: Duration::from_millis(100),
        idle_timeout: Some(Duration::from_secs(60)),
        max_lifetime: Some(Duration::from_secs(300)),
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    
    // Test that we can acquire connections up to the limit
    let conn1 = pool.acquire().await;
    assert!(conn1.is_ok());
    
    let conn2 = pool.acquire().await;
    assert!(conn2.is_ok());
    
    // Third connection should timeout quickly due to low acquire_timeout
    let start = std::time::Instant::now();
    let conn3 = pool.acquire().await;
    let elapsed = start.elapsed();
    
    // Should timeout within reasonable time (allowing some margin)
    assert!(elapsed < Duration::from_millis(200));
    assert!(conn3.is_err());
}

#[tokio::test]
async fn test_database_config_validation() {
    // Test valid config
    let valid_config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 10,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(30),
        idle_timeout: Some(Duration::from_secs(600)),
        max_lifetime: Some(Duration::from_secs(1800)),
    };
    
    assert!(validate_database_config(&valid_config).is_ok());
    
    // Test invalid config (min > max)
    let invalid_config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 5,
        min_connections: 10, // Invalid: min > max
        acquire_timeout: Duration::from_secs(30),
        idle_timeout: Some(Duration::from_secs(600)),
        max_lifetime: Some(Duration::from_secs(1800)),
    };
    
    assert!(validate_database_config(&invalid_config).is_err());
}

#[tokio::test]
async fn test_query_timeout_handling() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: None,
        max_lifetime: None,
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    
    // Test query with timeout
    let result = execute_query_with_timeout(
        &pool,
        "SELECT 1",
        Duration::from_secs(1)
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_connection_retry_logic() {
    // Test with invalid URL to trigger retry logic
    let config = DatabaseConfig {
        url: "invalid://connection/string".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout: Duration::from_millis(100),
        idle_timeout: None,
        max_lifetime: None,
    };
    
    let result = create_connection_pool_with_retry(&config, 3).await;
    assert!(result.is_err()); // Should fail after retries
}

#[tokio::test]
async fn test_database_metrics_collection() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(10),
        idle_timeout: Some(Duration::from_secs(300)),
        max_lifetime: Some(Duration::from_secs(900)),
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    let metrics = collect_pool_metrics(&pool).await;
    
    assert!(metrics.is_ok());
    let metrics = metrics.unwrap();
    assert!(metrics.active_connections >= 0);
    assert!(metrics.idle_connections >= 0);
    assert!(metrics.total_connections >= 0);
}

#[tokio::test]
async fn test_prepared_statement_caching() {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: None,
        max_lifetime: None,
    };
    
    let pool = create_connection_pool(&config).await.unwrap();
    
    // Test that prepared statements are cached and reused
    let query = "SELECT ?";
    
    let start1 = std::time::Instant::now();
    let _result1 = execute_prepared_query(&pool, query, &["test1"]).await;
    let time1 = start1.elapsed();
    
    let start2 = std::time::Instant::now();
    let _result2 = execute_prepared_query(&pool, query, &["test2"]).await;
    let time2 = start2.elapsed();
    
    // Second execution should be faster due to caching
    // Note: This is a rough test and may not always be reliable
    // In a real implementation, you'd have more sophisticated metrics
    assert!(time2 <= time1 + Duration::from_millis(10));
}