//! Simple Retry Service - Basic retry logic for Cloudflare Workers
//!
//! This service provides simple retry functionality without complex circuit breaker patterns:
//! - Basic exponential backoff retry
//! - Configurable retry attempts and delays
//! - Simple failure tracking
//! - Lightweight and WASM-compatible

use crate::utils::{logger::LogLevel, ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;
#[cfg(target_arch = "wasm32")]
use web_sys;

/// Simple retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleRetryConfig {
    /// Enable retry functionality
    pub enabled: bool,
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,
    /// Enable jitter to prevent thundering herd
    pub enable_jitter: bool,
    /// Track recent failures for basic health awareness
    pub track_failures: bool,
    /// Maximum failures to track
    pub max_tracked_failures: usize,
    /// Time window for failure tracking (seconds)
    pub failure_window_seconds: u64,
}

impl Default for SimpleRetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            enable_jitter: true,
            track_failures: true,
            max_tracked_failures: 50,
            failure_window_seconds: 300, // 5 minutes
        }
    }
}

impl SimpleRetryConfig {
    /// Fast retry configuration for high-performance scenarios
    pub fn fast() -> Self {
        Self {
            max_attempts: 2,
            initial_delay_ms: 50,
            max_delay_ms: 1000,
            backoff_multiplier: 1.5,
            failure_window_seconds: 60,
            ..Default::default()
        }
    }

    /// Reliable retry configuration for high-reliability scenarios
    pub fn reliable() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 200,
            max_delay_ms: 10000,
            backoff_multiplier: 2.5,
            failure_window_seconds: 600, // 10 minutes
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.max_attempts == 0 {
            return Err(ArbitrageError::config_error(
                "Max attempts must be greater than 0",
            ));
        }

        if self.initial_delay_ms == 0 {
            return Err(ArbitrageError::config_error(
                "Initial delay must be greater than 0",
            ));
        }

        if self.max_delay_ms < self.initial_delay_ms {
            return Err(ArbitrageError::config_error(
                "Max delay must be greater than or equal to initial delay",
            ));
        }

        if self.backoff_multiplier <= 1.0 {
            return Err(ArbitrageError::config_error(
                "Backoff multiplier must be greater than 1.0",
            ));
        }

        Ok(())
    }
}

/// Simple failure tracking for basic health awareness
#[derive(Debug, Clone)]
pub struct FailureTracker {
    failures: Vec<u64>, // Timestamps of failures
    config: SimpleRetryConfig,
}

impl FailureTracker {
    fn new(config: SimpleRetryConfig) -> Self {
        Self {
            failures: Vec::new(),
            config,
        }
    }

    fn record_failure(&mut self) {
        if !self.config.track_failures {
            return;
        }

        let now = Self::current_timestamp();
        self.failures.push(now);

        // Keep only recent failures within the time window
        let cutoff = now - (self.config.failure_window_seconds * 1000);
        self.failures.retain(|&timestamp| timestamp > cutoff);

        // Limit the number of tracked failures
        if self.failures.len() > self.config.max_tracked_failures {
            self.failures.remove(0);
        }
    }

    fn get_recent_failure_count(&self) -> usize {
        if !self.config.track_failures {
            return 0;
        }

        let now = Self::current_timestamp();
        let cutoff = now - (self.config.failure_window_seconds * 1000);
        self.failures
            .iter()
            .filter(|&&timestamp| timestamp > cutoff)
            .count()
    }

    fn is_healthy(&self) -> bool {
        // Consider healthy if recent failures are less than max attempts * 2
        let threshold = (self.config.max_attempts * 2) as usize;
        self.get_recent_failure_count() < threshold
    }

    #[cfg(target_arch = "wasm32")]
    fn current_timestamp() -> u64 {
        js_sys::Date::now() as u64
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Retry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStats {
    pub service_id: String,
    pub total_attempts: u64,
    pub successful_attempts: u64,
    pub failed_attempts: u64,
    pub retry_attempts: u64,
    pub average_attempts_per_operation: f64,
    pub recent_failure_count: usize,
    pub is_healthy: bool,
    pub last_updated: u64,
}

/// Simple retry service for Cloudflare Workers
pub struct SimpleRetryService {
    config: SimpleRetryConfig,
    logger: crate::utils::logger::Logger,

    // Failure tracking by service
    failure_trackers: Arc<Mutex<HashMap<String, FailureTracker>>>,

    // Basic statistics
    stats: Arc<Mutex<HashMap<String, RetryStats>>>,
}

impl SimpleRetryService {
    /// Create a new simple retry service
    pub fn new(config: SimpleRetryConfig) -> ArbitrageResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            logger: crate::utils::logger::Logger::new(LogLevel::Info),
            failure_trackers: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, Fut, T, E>(
        &self,
        service_id: &str,
        operation: F,
    ) -> Result<T, ArbitrageError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        if !self.config.enabled {
            return operation()
                .await
                .map_err(|e| ArbitrageError::api_error(format!("Operation failed: {}", e)));
        }

        let mut attempt = 1;
        let mut delay = self.config.initial_delay_ms;

        loop {
            // Execute the operation
            match operation().await {
                Ok(result) => {
                    self.record_success(service_id, attempt).await;
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        self.record_failure(service_id, attempt).await;
                        return Err(ArbitrageError::api_error(format!(
                            "Operation failed after {} attempts: {}",
                            attempt, e
                        )));
                    }

                    // Log retry attempt
                    self.logger.warn(&format!(
                        "Attempt {} failed for service '{}': {}. Retrying in {}ms...",
                        attempt, service_id, e, delay
                    ));

                    // Wait before retry
                    self.sleep(delay).await;

                    // Calculate next delay with exponential backoff
                    delay = ((delay as f64) * self.config.backoff_multiplier) as u64;
                    if delay > self.config.max_delay_ms {
                        delay = self.config.max_delay_ms;
                    }

                    // Add jitter if enabled
                    if self.config.enable_jitter {
                        // Simple jitter using current timestamp
                        let timestamp = FailureTracker::current_timestamp();
                        let jitter =
                            (delay as f64 * 0.1 * ((timestamp % 100) as f64 / 100.0)) as u64;
                        delay = delay.saturating_add(jitter);
                    }

                    attempt += 1;
                }
            }
        }
    }

    /// Execute an operation with simple retry (non-async version)
    pub fn execute_sync<F, T, E>(
        &self,
        service_id: &str,
        mut operation: F,
    ) -> Result<T, ArbitrageError>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display,
    {
        if !self.config.enabled {
            return operation()
                .map_err(|e| ArbitrageError::api_error(format!("Operation failed: {}", e)));
        }

        let mut attempt = 1;

        loop {
            match operation() {
                Ok(result) => {
                    // Record success (simplified for sync version)
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        return Err(ArbitrageError::api_error(format!(
                            "Operation failed after {} attempts: {}",
                            attempt, e
                        )));
                    }

                    self.logger.warn(&format!(
                        "Attempt {} failed for service '{}': {}. Retrying...",
                        attempt, service_id, e
                    ));

                    attempt += 1;
                }
            }
        }
    }

    /// Check if a service is healthy based on recent failures
    pub async fn is_service_healthy(&self, service_id: &str) -> bool {
        if let Ok(trackers) = self.failure_trackers.lock() {
            if let Some(tracker) = trackers.get(service_id) {
                return tracker.is_healthy();
            }
        }
        true // Default to healthy if no tracking data
    }

    /// Get retry statistics for a service
    pub async fn get_stats(&self, service_id: &str) -> Option<RetryStats> {
        if let Ok(stats) = self.stats.lock() {
            stats.get(service_id).cloned()
        } else {
            None
        }
    }

    /// Get all retry statistics
    pub async fn get_all_stats(&self) -> HashMap<String, RetryStats> {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get service health summary
    pub async fn get_health_summary(&self) -> HashMap<String, bool> {
        let mut health = HashMap::new();

        if let Ok(trackers) = self.failure_trackers.lock() {
            for (service_id, tracker) in trackers.iter() {
                health.insert(service_id.clone(), tracker.is_healthy());
            }
        }

        health
    }

    // Private helper methods

    async fn record_success(&self, service_id: &str, attempts: u32) {
        // Update failure tracker
        if let Ok(mut trackers) = self.failure_trackers.lock() {
            // Success doesn't add failures, but we ensure tracker exists
            if !trackers.contains_key(service_id) {
                trackers.insert(
                    service_id.to_string(),
                    FailureTracker::new(self.config.clone()),
                );
            }
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            let stat = stats
                .entry(service_id.to_string())
                .or_insert_with(|| RetryStats {
                    service_id: service_id.to_string(),
                    total_attempts: 0,
                    successful_attempts: 0,
                    failed_attempts: 0,
                    retry_attempts: 0,
                    average_attempts_per_operation: 0.0,
                    recent_failure_count: 0,
                    is_healthy: true,
                    last_updated: FailureTracker::current_timestamp(),
                });

            stat.total_attempts += attempts as u64;
            stat.successful_attempts += 1;
            if attempts > 1 {
                stat.retry_attempts += (attempts - 1) as u64;
            }
            stat.average_attempts_per_operation =
                stat.total_attempts as f64 / stat.successful_attempts as f64;
            stat.last_updated = FailureTracker::current_timestamp();
        }
    }

    async fn record_failure(&self, service_id: &str, attempts: u32) {
        // Update failure tracker
        if let Ok(mut trackers) = self.failure_trackers.lock() {
            let tracker = trackers
                .entry(service_id.to_string())
                .or_insert_with(|| FailureTracker::new(self.config.clone()));
            tracker.record_failure();
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            let stat = stats
                .entry(service_id.to_string())
                .or_insert_with(|| RetryStats {
                    service_id: service_id.to_string(),
                    total_attempts: 0,
                    successful_attempts: 0,
                    failed_attempts: 0,
                    retry_attempts: 0,
                    average_attempts_per_operation: 0.0,
                    recent_failure_count: 0,
                    is_healthy: true,
                    last_updated: FailureTracker::current_timestamp(),
                });

            stat.total_attempts += attempts as u64;
            stat.failed_attempts += 1;
            if attempts > 1 {
                stat.retry_attempts += (attempts - 1) as u64;
            }

            // Update recent failure count and health status
            if let Ok(trackers) = self.failure_trackers.lock() {
                if let Some(tracker) = trackers.get(service_id) {
                    stat.recent_failure_count = tracker.get_recent_failure_count();
                    stat.is_healthy = tracker.is_healthy();
                }
            }

            stat.last_updated = FailureTracker::current_timestamp();
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn sleep(&self, _ms: u64) {
        // Simple no-op sleep for WASM - Cloudflare Workers handle this differently
        // In production, delays would be handled by the runtime
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL))
            .await
            .unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn sleep(&self, ms: u64) {
        tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_retry_config_default() {
        let config = SimpleRetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_simple_retry_config_validation() {
        // Test invalid max_attempts
        let config = SimpleRetryConfig {
            max_attempts: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid initial_delay_ms
        let config = SimpleRetryConfig {
            max_attempts: 3,
            initial_delay_ms: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid max_delay_ms (less than initial)
        let config = SimpleRetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 500,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid backoff_multiplier
        let config = SimpleRetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 2000,
            backoff_multiplier: 0.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_failure_tracker() {
        let config = SimpleRetryConfig::default();
        let mut tracker = FailureTracker::new(config);

        assert!(tracker.is_healthy());
        assert_eq!(tracker.get_recent_failure_count(), 0);

        // Record some failures
        for _ in 0..3 {
            tracker.record_failure();
        }

        assert_eq!(tracker.get_recent_failure_count(), 3);
        assert!(tracker.is_healthy()); // Should still be healthy with 3 failures
    }

    #[tokio::test]
    async fn test_simple_retry_service_creation() {
        let config = SimpleRetryConfig::default();
        let service = SimpleRetryService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_sync_retry_execution() {
        let config = SimpleRetryConfig::fast();
        let service = SimpleRetryService::new(config).unwrap();

        let mut attempt_count = 0;
        let result = service.execute_sync("test_service", || {
            attempt_count += 1;
            if attempt_count < 2 {
                Err("Simulated failure")
            } else {
                Ok("Success")
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt_count, 2);
    }
}
