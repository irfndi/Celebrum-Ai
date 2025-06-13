use crate::common::test_utils::*;
use arb_edge::services::core::infrastructure::{
    UnifiedCircuitBreaker, UnifiedCircuitBreakerConfig, UnifiedCircuitBreakerManager,
    UnifiedCircuitBreakerType, UnifiedHealthCheckConfig, UnifiedRetryConfig,
    UnifiedRetryExecutor,
};
use arb_edge::utils::ArbitrageError;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test unified retry configuration and execution
#[tokio::test]
async fn test_unified_retry_configuration_and_execution() {
    // Test different retry configurations
    let high_perf_config = UnifiedRetryConfig::high_performance();
    assert_eq!(high_perf_config.max_attempts, 5);
    assert_eq!(high_perf_config.initial_delay, Duration::from_millis(100));
    assert!(high_perf_config.enable_jitter);
    assert!(high_perf_config.validate().is_ok());

    let high_reliability_config = UnifiedRetryConfig::high_reliability();
    assert_eq!(high_reliability_config.max_attempts, 10);
    assert_eq!(high_reliability_config.initial_delay, Duration::from_secs(2));
    assert!(high_reliability_config.validate().is_ok());

    let alert_config = UnifiedRetryConfig::alert_optimized();
    assert_eq!(alert_config.max_attempts, 3);
    assert!(!alert_config.enable_jitter);
    assert!(alert_config.validate().is_ok());

    // Test retry executor
    let executor = UnifiedRetryExecutor::new(high_perf_config);
    let mut attempt_count = 0;

    let result = executor
        .execute_with_retry(|| async {
            attempt_count += 1;
            if attempt_count < 3 {
                Err("Temporary failure")
            } else {
                Ok("Success")
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(attempt_count, 3);

    println!("✅ Unified retry configuration and execution validated");
}

/// Test unified retry configuration specializations
#[tokio::test]
async fn test_unified_retry_configuration_specializations() {
    // Test all specialized configurations
    let alert_config = UnifiedRetryConfig::alert_optimized();
    assert_eq!(alert_config.max_attempts, 3);
    assert!(!alert_config.enable_jitter);
    assert!(alert_config.validate().is_ok());

    let delivery_config = UnifiedRetryConfig::delivery_optimized();
    assert_eq!(delivery_config.max_attempts, 3);
    assert_eq!(delivery_config.initial_delay, Duration::from_millis(1000));
    assert!(delivery_config.validate().is_ok());

    let pipeline_config = UnifiedRetryConfig::pipeline_optimized();
    assert_eq!(pipeline_config.max_attempts, 3);
    assert!(pipeline_config.enable_jitter);
    assert!(pipeline_config.validate().is_ok());

    let validation_config = UnifiedRetryConfig::validation_optimized();
    assert_eq!(validation_config.max_attempts, 2);
    assert!(!validation_config.enable_jitter);
    assert!(validation_config.validate().is_ok());

    let adapter_config = UnifiedRetryConfig::adapter_optimized();
    assert_eq!(adapter_config.max_attempts, 3);
    assert!(adapter_config.enable_jitter);
    assert!(adapter_config.validate().is_ok());

    // Test delay calculation consistency
    for config in [&alert_config, &delivery_config, &pipeline_config, &validation_config, &adapter_config] {
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);
        assert!(delay2 >= delay1); // Delays should increase or stay same
        assert!(delay1 >= config.initial_delay || delay1 == Duration::from_secs(0));
    }

    println!("✅ Unified retry configuration specializations validated");
}

/// Test unified circuit breaker functionality
#[tokio::test]
async fn test_unified_circuit_breaker_functionality() {
    let config = UnifiedCircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 1,
        ..Default::default()
    };

    let mut circuit_breaker =
        UnifiedCircuitBreaker::new("test-breaker".to_string(), config).unwrap();

    // Test initial state
    assert!(circuit_breaker.can_execute());
    assert_eq!(circuit_breaker.get_id(), "test-breaker");

    // Test failure recording and state transitions
    circuit_breaker.record_failure();
    circuit_breaker.record_failure();
    assert!(circuit_breaker.can_execute()); // Still closed

    circuit_breaker.record_failure();
    assert!(!circuit_breaker.can_execute()); // Now open

    // Test timeout and recovery
    sleep(Duration::from_secs(2)).await;
    assert!(circuit_breaker.can_execute()); // Should transition to half-open

    // Test success recording
    circuit_breaker.record_success();
    circuit_breaker.record_success();
    assert!(circuit_breaker.can_execute()); // Should be closed again

    println!("✅ Unified circuit breaker functionality validated");
}

/// Test unified circuit breaker manager
#[tokio::test]
async fn test_unified_circuit_breaker_manager() {
    let config = UnifiedCircuitBreakerConfig::default();
    let manager = UnifiedCircuitBreakerManager::new(config);

    // Test circuit breaker creation and management
    let breaker1 = manager
        .get_or_create_circuit_breaker(
            "service1".to_string(),
            UnifiedCircuitBreakerType::HttpApi,
        )
        .await
        .unwrap();

    let breaker2 = manager
        .get_or_create_circuit_breaker(
            "service2".to_string(),
            UnifiedCircuitBreakerType::Database,
        )
        .await
        .unwrap();

    // Test that same ID returns same breaker
    let breaker1_again = manager
        .get_or_create_circuit_breaker(
            "service1".to_string(),
            UnifiedCircuitBreakerType::HttpApi,
        )
        .await
        .unwrap();

    assert!(Arc::ptr_eq(&breaker1, &breaker1_again));

    // Test execution with circuit breaker
    let result = manager
        .execute_with_circuit_breaker(
            "test-operation".to_string(),
            UnifiedCircuitBreakerType::InternalService,
            || Ok::<i32, String>(42),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Test failure handling
    let result = manager
        .execute_with_circuit_breaker(
            "failing-operation".to_string(),
            UnifiedCircuitBreakerType::ExternalService,
            || Err::<i32, String>("Operation failed".to_string()),
        )
        .await;

    assert!(result.is_err());

    // Test state retrieval
    let states = manager.get_all_states().await;
    assert!(!states.is_empty());

    println!("✅ Unified circuit breaker manager validated");
}

/// Test unified health check configuration
#[tokio::test]
async fn test_unified_health_check_configuration() {
    // Test different health check configurations
    let service_config = UnifiedHealthCheckConfig::service_optimized();
    assert_eq!(service_config.interval_seconds, 30);
    assert_eq!(service_config.degraded_threshold_ms, 1000.0);

    let failover_config = UnifiedHealthCheckConfig::failover_optimized();
    assert_eq!(failover_config.interval_seconds, 15);
    assert_eq!(failover_config.timeout_seconds, 5);
    assert_eq!(failover_config.failure_threshold, 2);

    let external_config = UnifiedHealthCheckConfig::external_system_optimized();
    assert_eq!(external_config.interval_seconds, 60);
    assert_eq!(external_config.timeout_seconds, 15);
    assert_eq!(external_config.failure_threshold, 5);

    // Test configuration consistency across specializations
    assert!(service_config.enable_dependency_checks);
    assert!(service_config.enable_detailed_metrics);
    
    assert!(!failover_config.enable_dependency_checks);
    assert!(!failover_config.enable_detailed_metrics);
    
    assert!(external_config.enable_dependency_checks);
    assert!(external_config.enable_detailed_metrics);

    // Test that all configurations have valid thresholds
    assert!(service_config.degraded_threshold_ms > 0.0);
    assert!(service_config.unhealthy_threshold_ms > service_config.degraded_threshold_ms);
    
    assert!(failover_config.degraded_threshold_ms > 0.0);
    assert!(failover_config.unhealthy_threshold_ms > failover_config.degraded_threshold_ms);
    
    assert!(external_config.degraded_threshold_ms > 0.0);
    assert!(external_config.unhealthy_threshold_ms > external_config.degraded_threshold_ms);

    println!("✅ Unified health check configuration validated");
}

/// Test integration between unified modules
#[tokio::test]
async fn test_unified_modules_integration() {
    // Create configurations for all unified modules
    let retry_config = UnifiedRetryConfig::high_reliability();
    let circuit_breaker_config = UnifiedCircuitBreakerConfig::high_reliability();
    let health_check_config = UnifiedHealthCheckConfig::service_optimized();

    // Test that configurations are compatible
    assert!(retry_config.validate().is_ok());
    assert!(circuit_breaker_config.validate().is_ok());

    // Create components
    let retry_executor = UnifiedRetryExecutor::new(retry_config);
    let circuit_breaker_manager = UnifiedCircuitBreakerManager::new(circuit_breaker_config);

    // Test integrated operation: retry with circuit breaker
    let operation_count = Arc::new(std::sync::Mutex::new(0));
    let operation_count_clone = operation_count.clone();

    let result = retry_executor
        .execute_with_retry(|| async {
            let mut count = operation_count_clone.lock().unwrap();
            *count += 1;

            // Simulate operation that succeeds after 2 attempts
            if *count < 3 {
                Err("Temporary failure")
            } else {
                Ok("Success")
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");

    let final_count = *operation_count.lock().unwrap();
    assert_eq!(final_count, 3);

    // Test circuit breaker with retry integration
    let cb_result = circuit_breaker_manager
        .execute_with_circuit_breaker(
            "integrated-test".to_string(),
            UnifiedCircuitBreakerType::InternalService,
            || Ok::<String, ArbitrageError>("Integrated success".to_string()),
        )
        .await;

    assert!(cb_result.is_ok());
    assert_eq!(cb_result.unwrap(), "Integrated success");

    println!("✅ Unified modules integration validated");
}

/// Test configuration validation and error handling
#[tokio::test]
async fn test_unified_configuration_validation() {
    // Test invalid retry configuration
    let invalid_retry_config = UnifiedRetryConfig {
        max_attempts: 0,
        ..Default::default()
    };
    assert!(invalid_retry_config.validate().is_err());

    let invalid_delay_config = UnifiedRetryConfig {
        initial_delay: Duration::from_secs(10),
        max_delay: Duration::from_secs(5), // Max < initial
        ..Default::default()
    };
    assert!(invalid_delay_config.validate().is_err());

    // Test invalid circuit breaker configuration
    let invalid_cb_config = UnifiedCircuitBreakerConfig {
        failure_threshold: 0,
        ..Default::default()
    };
    assert!(invalid_cb_config.validate().is_err());

    let invalid_timeout_config = UnifiedCircuitBreakerConfig {
        timeout_seconds: 0,
        ..Default::default()
    };
    assert!(invalid_timeout_config.validate().is_err());

    // Test that valid configurations pass validation
    let valid_retry = UnifiedRetryConfig::default();
    let valid_cb = UnifiedCircuitBreakerConfig::default();

    assert!(valid_retry.validate().is_ok());
    assert!(valid_cb.validate().is_ok());

    println!("✅ Configuration validation and error handling validated");
}

/// Test performance characteristics of unified modules
#[tokio::test]
async fn test_unified_modules_performance() {
    let start_time = std::time::Instant::now();

    // Test high-performance configurations
    let hp_retry_config = UnifiedRetryConfig::high_performance();
    let hp_cb_config = UnifiedCircuitBreakerConfig::high_performance();

    let retry_executor = UnifiedRetryExecutor::new(hp_retry_config);
    let cb_manager = UnifiedCircuitBreakerManager::new(hp_cb_config);

    // Perform multiple operations to test performance
    let mut tasks = Vec::new();
    for i in 0..10 {
        let executor = retry_executor.clone();
        let manager = cb_manager.clone();
        
        let task = tokio::spawn(async move {
            let retry_result = executor
                .execute_with_retry(|| async { Ok::<i32, String>(i) })
                .await;

            let cb_result = manager
                .execute_with_circuit_breaker(
                    format!("perf-test-{}", i),
                    UnifiedCircuitBreakerType::InternalService,
                    || Ok::<i32, String>(i * 2),
                )
                .await;

            (retry_result, cb_result)
        });
        
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // Verify all operations succeeded
    for result in results {
        let (retry_result, cb_result) = result.unwrap();
        assert!(retry_result.is_ok());
        assert!(cb_result.is_ok());
    }

    let elapsed = start_time.elapsed();
    println!("✅ Performance test completed in {:?}", elapsed);

    // Performance should be reasonable (less than 1 second for 10 concurrent operations)
    assert!(elapsed < Duration::from_secs(1));

    println!("✅ Unified modules performance validated");
} 