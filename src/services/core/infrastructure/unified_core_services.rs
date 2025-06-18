// src/services/core/infrastructure/unified_core_services.rs

//! Unified Core Infrastructure Services
//!
//! This module consolidates all core infrastructure services including:
//! - Circuit Breaker Service
//! - Retry Service  
//! - Health Check Service
//! - Failover Service
//!
//! Designed for high efficiency, zero duplication, and maximum concurrency

use crate::utils::error::{ArbitrageError, ArbitrageResult, ErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use worker::Env;

// ============= UNIFIED CONFIGURATION =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCoreServicesConfig {
    pub circuit_breaker: CircuitBreakerConfig,
    pub retry: RetryConfig,
    pub health_check: HealthCheckConfig,
    pub failover: FailoverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_ms: u64,
    pub half_open_max_calls: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_interval_ms: u64,
    pub timeout_ms: u64,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    pub enable_auto_failover: bool,
    pub failover_timeout_ms: u64,
    pub max_concurrent_failovers: u32,
    pub fallback_strategies: Vec<FailoverStrategy>,
}

// ============= CORE SERVICE STATES =============

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailoverState {
    Active,
    Standby,
    Failed,
    Recovering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailoverStrategy {
    RoundRobin,
    LeastConnections,
    HealthBased,
    Geographic,
}

// ============= METRICS AND TRACKING =============

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub circuit_breaker_metrics: CircuitBreakerMetrics,
    pub retry_metrics: RetryMetrics,
    pub health_metrics: HealthMetrics,
    pub failover_metrics: FailoverMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub state_changes: u64,
    pub current_state: CircuitBreakerState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetryMetrics {
    pub total_attempts: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
    pub max_delay_reached: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub total_checks: u64,
    pub healthy_checks: u64,
    pub unhealthy_checks: u64,
    pub average_response_time_ms: f64,
    pub current_status: HealthStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailoverMetrics {
    pub total_failovers: u64,
    pub successful_failovers: u64,
    pub failed_failovers: u64,
    pub current_active_services: u32,
}

// ============= MAIN UNIFIED SERVICE =============

pub struct UnifiedCoreServices {
    config: UnifiedCoreServicesConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
    health_status: Arc<RwLock<HashMap<String, HealthStatus>>>,
    failover_states: Arc<RwLock<HashMap<String, FailoverState>>>,
    metrics: Arc<RwLock<ServiceMetrics>>,
}

impl UnifiedCoreServices {
    pub fn new(config: UnifiedCoreServicesConfig) -> Self {
        Self {
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            failover_states: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
        }
    }

    // ============= CIRCUIT BREAKER OPERATIONS =============

    pub async fn execute_with_circuit_breaker<F, T>(
        &self,
        service_name: &str,
        operation: F,
    ) -> ArbitrageResult<T>
    where
        F: std::future::Future<Output = ArbitrageResult<T>>,
    {
        let state = self.get_circuit_breaker_state(service_name).await;

        match state {
            CircuitBreakerState::Open => {
                self.update_metrics_rejected(service_name).await;
                return Err(ArbitrageError::new(
                    ErrorKind::ServiceUnavailable,
                    &format!("Circuit breaker open for service: {}", service_name),
                ));
            }
            CircuitBreakerState::HalfOpen => {
                if !self.can_attempt_half_open(service_name).await {
                    self.update_metrics_rejected(service_name).await;
                    return Err(ArbitrageError::new(
                        ErrorKind::ServiceUnavailable,
                        &format!(
                            "Circuit breaker half-open limit reached for: {}",
                            service_name
                        ),
                    ));
                }
            }
            CircuitBreakerState::Closed => {}
        }

        match operation.await {
            Ok(result) => {
                self.record_success(service_name).await;
                Ok(result)
            }
            Err(error) => {
                self.record_failure(service_name).await;
                Err(error)
            }
        }
    }

    // ============= RETRY OPERATIONS =============

    pub async fn execute_with_retry<F, T, Fut>(
        &self,
        service_name: &str,
        mut operation: F,
    ) -> ArbitrageResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ArbitrageResult<T>>,
    {
        let mut attempts = 0;
        let mut delay = self.config.retry.initial_delay_ms;

        loop {
            attempts += 1;

            match operation().await {
                Ok(result) => {
                    self.update_retry_metrics_success(service_name, attempts)
                        .await;
                    return Ok(result);
                }
                Err(error) => {
                    if attempts >= self.config.retry.max_attempts {
                        self.update_retry_metrics_failure(service_name, attempts)
                            .await;
                        return Err(error);
                    }

                    if self.should_retry(&error) {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        delay = (delay as f64 * self.config.retry.backoff_multiplier) as u64;
                        delay = delay.min(self.config.retry.max_delay_ms);
                    } else {
                        self.update_retry_metrics_failure(service_name, attempts)
                            .await;
                        return Err(error);
                    }
                }
            }
        }
    }

    // ============= HEALTH CHECK OPERATIONS =============

    pub async fn perform_health_check(
        &self,
        service_name: &str,
        check_fn: impl std::future::Future<Output = ArbitrageResult<()>>,
    ) -> HealthStatus {
        let start_time = std::time::Instant::now();

        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(self.config.health_check.timeout_ms),
            check_fn,
        )
        .await;

        let duration = start_time.elapsed();
        let status = match result {
            Ok(Ok(_)) => HealthStatus::Healthy,
            Ok(Err(_)) => HealthStatus::Unhealthy,
            Err(_) => HealthStatus::Unhealthy, // Timeout
        };

        self.update_health_status(service_name, status.clone(), duration.as_millis() as f64)
            .await;

        status
    }

    // ============= FAILOVER OPERATIONS =============

    pub async fn execute_with_failover<F, T>(
        &self,
        primary_service: &str,
        fallback_services: &[&str],
        operation: F,
    ) -> ArbitrageResult<T>
    where
        F: Fn(
            &str,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = ArbitrageResult<T>> + Send>>,
    {
        // Try primary service first
        if self.is_service_available(primary_service).await {
            match operation(primary_service).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    self.mark_service_failed(primary_service).await;
                    log::warn!(
                        "Primary service {} failed, attempting failover",
                        primary_service
                    );
                }
            }
        }

        // Try fallback services
        for fallback in fallback_services {
            if self.is_service_available(fallback).await {
                match operation(fallback).await {
                    Ok(result) => {
                        self.update_failover_metrics_success().await;
                        return Ok(result);
                    }
                    Err(_) => {
                        self.mark_service_failed(fallback).await;
                        continue;
                    }
                }
            }
        }

        self.update_failover_metrics_failure().await;
        Err(ArbitrageError::new(
            ErrorKind::ServiceUnavailable,
            "All services failed during failover attempt",
        ))
    }

    // ============= PRIVATE HELPER METHODS =============

    async fn get_circuit_breaker_state(&self, service_name: &str) -> CircuitBreakerState {
        let breakers = self.circuit_breakers.read().await;
        breakers
            .get(service_name)
            .cloned()
            .unwrap_or(CircuitBreakerState::Closed)
    }

    async fn can_attempt_half_open(&self, _service_name: &str) -> bool {
        // Implementation for half-open attempt tracking
        true // Simplified for now
    }

    async fn record_success(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        breakers.insert(service_name.to_string(), CircuitBreakerState::Closed);

        let mut metrics = self.metrics.write().await;
        metrics.circuit_breaker_metrics.successful_requests += 1;
        metrics.circuit_breaker_metrics.total_requests += 1;
    }

    async fn record_failure(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        breakers.insert(service_name.to_string(), CircuitBreakerState::Open);

        let mut metrics = self.metrics.write().await;
        metrics.circuit_breaker_metrics.failed_requests += 1;
        metrics.circuit_breaker_metrics.total_requests += 1;
        metrics.circuit_breaker_metrics.state_changes += 1;
    }

    async fn update_metrics_rejected(&self, _service_name: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.circuit_breaker_metrics.rejected_requests += 1;
        metrics.circuit_breaker_metrics.total_requests += 1;
    }

    async fn update_retry_metrics_success(&self, _service_name: &str, attempts: u32) {
        let mut metrics = self.metrics.write().await;
        metrics.retry_metrics.total_attempts += attempts as u64;
        metrics.retry_metrics.successful_retries += 1;
    }

    async fn update_retry_metrics_failure(&self, _service_name: &str, attempts: u32) {
        let mut metrics = self.metrics.write().await;
        metrics.retry_metrics.total_attempts += attempts as u64;
        metrics.retry_metrics.failed_retries += 1;
    }

    async fn update_health_status(
        &self,
        service_name: &str,
        status: HealthStatus,
        response_time: f64,
    ) {
        let mut health_map = self.health_status.write().await;
        health_map.insert(service_name.to_string(), status.clone());

        let mut metrics = self.metrics.write().await;
        metrics.health_metrics.total_checks += 1;
        match status {
            HealthStatus::Healthy => metrics.health_metrics.healthy_checks += 1,
            _ => metrics.health_metrics.unhealthy_checks += 1,
        }

        // Update rolling average response time
        let total_checks = metrics.health_metrics.total_checks as f64;
        let current_avg = metrics.health_metrics.average_response_time_ms;
        metrics.health_metrics.average_response_time_ms =
            (current_avg * (total_checks - 1.0) + response_time) / total_checks;
        metrics.health_metrics.current_status = status;
    }

    async fn is_service_available(&self, service_name: &str) -> bool {
        let health_map = self.health_status.read().await;
        let failover_map = self.failover_states.read().await;

        let health_status = health_map
            .get(service_name)
            .unwrap_or(&HealthStatus::Unknown);
        let failover_state = failover_map
            .get(service_name)
            .unwrap_or(&FailoverState::Active);

        matches!(
            health_status,
            HealthStatus::Healthy | HealthStatus::Degraded
        ) && matches!(
            failover_state,
            FailoverState::Active | FailoverState::Recovering
        )
    }

    async fn mark_service_failed(&self, service_name: &str) {
        let mut failover_map = self.failover_states.write().await;
        failover_map.insert(service_name.to_string(), FailoverState::Failed);

        let mut health_map = self.health_status.write().await;
        health_map.insert(service_name.to_string(), HealthStatus::Unhealthy);
    }

    async fn update_failover_metrics_success(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.failover_metrics.total_failovers += 1;
        metrics.failover_metrics.successful_failovers += 1;
    }

    async fn update_failover_metrics_failure(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.failover_metrics.total_failovers += 1;
        metrics.failover_metrics.failed_failovers += 1;
    }

    fn should_retry(&self, error: &ArbitrageError) -> bool {
        matches!(
            error.kind(),
            ErrorKind::NetworkError | ErrorKind::TemporaryFailure | ErrorKind::RateLimitExceeded
        )
    }

    // ============= PUBLIC INTERFACE METHODS =============

    pub async fn get_metrics(&self) -> ServiceMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn get_health_status(&self, service_name: &str) -> HealthStatus {
        let health_map = self.health_status.read().await;
        health_map
            .get(service_name)
            .cloned()
            .unwrap_or(HealthStatus::Unknown)
    }

    pub async fn get_all_health_status(&self) -> HashMap<String, HealthStatus> {
        self.health_status.read().await.clone()
    }

    pub async fn reset_circuit_breaker(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        breakers.insert(service_name.to_string(), CircuitBreakerState::Closed);
    }

    pub async fn force_circuit_breaker_open(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        breakers.insert(service_name.to_string(), CircuitBreakerState::Open);
    }
}

// ============= DEFAULT IMPLEMENTATIONS =============

impl Default for UnifiedCoreServicesConfig {
    fn default() -> Self {
        Self {
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout_ms: 60000,
                half_open_max_calls: 3,
            },
            retry: RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
                backoff_multiplier: 2.0,
            },
            health_check: HealthCheckConfig {
                check_interval_ms: 30000,
                timeout_ms: 5000,
                healthy_threshold: 3,
                unhealthy_threshold: 3,
            },
            failover: FailoverConfig {
                enable_auto_failover: true,
                failover_timeout_ms: 10000,
                max_concurrent_failovers: 5,
                fallback_strategies: vec![
                    FailoverStrategy::HealthBased,
                    FailoverStrategy::RoundRobin,
                ],
            },
        }
    }
}

impl UnifiedCoreServicesConfig {
    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.circuit_breaker.failure_threshold == 0 {
            return Err(ArbitrageError::new(
                ErrorKind::InvalidInput,
                "circuit_breaker failure_threshold must be greater than 0",
            ));
        }
        if self.retry.max_attempts == 0 {
            return Err(ArbitrageError::new(
                ErrorKind::InvalidInput,
                "retry max_attempts must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        CircuitBreakerState::Closed
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Unknown
    }
}

impl Default for FailoverState {
    fn default() -> Self {
        FailoverState::Active
    }
}

// ============= TESTS =============

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_operations() {
        let config = UnifiedCoreServicesConfig::default();
        let service = UnifiedCoreServices::new(config);

        // Test successful operation
        let result = service
            .execute_with_circuit_breaker("test_service", async { Ok("success") })
            .await;
        assert!(result.is_ok());

        // Test circuit breaker state
        let state = service.get_circuit_breaker_state("test_service").await;
        assert_eq!(state, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_retry_operations() {
        let config = UnifiedCoreServicesConfig::default();
        let service = UnifiedCoreServices::new(config);

        let mut call_count = 0;
        let result = service
            .execute_with_retry("test_service", || {
                call_count += 1;
                async move {
                    if call_count < 2 {
                        Err(ArbitrageError::new(
                            ErrorKind::NetworkError,
                            "Network error",
                        ))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(call_count, 2);
    }

    #[tokio::test]
    async fn test_health_check_operations() {
        let config = UnifiedCoreServicesConfig::default();
        let service = UnifiedCoreServices::new(config);

        let status = service
            .perform_health_check("test_service", async { Ok(()) })
            .await;
        assert_eq!(status, HealthStatus::Healthy);

        let status = service
            .perform_health_check("test_service", async {
                Err(ArbitrageError::new(
                    ErrorKind::ServiceUnavailable,
                    "Test error",
                ))
            })
            .await;
        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_failover_operations() {
        let config = UnifiedCoreServicesConfig::default();
        let service = UnifiedCoreServices::new(config);

        let fallback_services = vec!["fallback1", "fallback2"];
        let result = service
            .execute_with_failover("primary", &fallback_services, |service_name| {
                Box::pin(async move {
                    if service_name == "primary" {
                        Err(ArbitrageError::new(
                            ErrorKind::ServiceUnavailable,
                            "Primary failed",
                        ))
                    } else if service_name == "fallback1" {
                        Ok("fallback1_success")
                    } else {
                        Err(ArbitrageError::new(
                            ErrorKind::ServiceUnavailable,
                            "Fallback failed",
                        ))
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback1_success");
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = UnifiedCoreServicesConfig::default();
        let service = UnifiedCoreServices::new(config);

        // Execute some operations to generate metrics
        let _ = service
            .execute_with_circuit_breaker("test", async { Ok("success") })
            .await;

        let metrics = service.get_metrics().await;
        assert!(metrics.circuit_breaker_metrics.total_requests > 0);
        assert!(metrics.circuit_breaker_metrics.successful_requests > 0);
    }
}
