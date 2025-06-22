// Tests for shared_types module
// Moved from src/services/core/infrastructure/shared_types.rs

use arbedge::services::core::infrastructure::shared_types::*;

#[test]
fn test_component_health_default() {
    let health = ComponentHealth::default();
    assert!(!health.is_healthy);
    assert_eq!(health.error_count, 0);
    assert_eq!(health.component_name, "unknown");
}

#[test]
fn test_circuit_breaker_default() {
    let mut breaker = CircuitBreaker::default();
    assert_eq!(breaker.state, CircuitBreakerState::Closed);
    assert_eq!(breaker.threshold, 5);
    assert!(breaker.can_execute());
}

#[test]
fn test_circuit_breaker_failure_handling() {
    let mut breaker = CircuitBreaker::new(2, 60);

    // Should be closed initially
    assert!(breaker.can_execute());

    // Record failures
    breaker.record_failure();
    assert!(breaker.can_execute()); // Still closed

    breaker.record_failure();
    assert!(!breaker.can_execute()); // Now open
    assert_eq!(breaker.state, CircuitBreakerState::Open);
}

#[test]
fn test_validation_cache_entry() {
    let entry = ValidationCacheEntry::new("test_key".to_string());
    assert_eq!(entry.key, "test_key");
    assert!(!entry.is_expired());
    assert_eq!(entry.access_count, 0);
}

#[test]
fn test_rate_limiter() {
    let mut limiter = RateLimiter::new(2);

    assert!(limiter.can_proceed());
    assert!(limiter.can_proceed());
    assert!(!limiter.can_proceed()); // Rate limit exceeded

    assert_eq!(limiter.total_requests, 3);
    assert_eq!(limiter.rejected_requests, 1);
}

#[test]
fn test_health_check_result() {
    let healthy = HealthCheckResult::healthy("test_component".to_string(), 100);
    assert!(healthy.is_healthy);
    assert_eq!(healthy.response_time_ms, 100);
    assert!(healthy.error_message.is_none());

    let unhealthy = HealthCheckResult::unhealthy(
        "test_component".to_string(),
        500,
        "Connection failed".to_string(),
    );
    assert!(!unhealthy.is_healthy);
    assert_eq!(unhealthy.response_time_ms, 500);
    assert!(unhealthy.error_message.is_some());
}