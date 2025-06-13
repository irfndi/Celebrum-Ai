// Unified Retry System - Consolidates retry functionality to eliminate duplication
// Combines features from cloudflare_pipelines.rs, alert_manager.rs, and other modules

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::utils::{ArbitrageError, ArbitrageResult};

/// Unified retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Specific error types to retry on
    pub retry_on_errors: Vec<String>,
    /// Enable jitter to avoid thundering herd
    pub enable_jitter: bool,
    /// Maximum total retry duration
    pub max_total_duration: Option<Duration>,
}

impl Default for UnifiedRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "ServiceUnavailable".to_string(),
                "TemporaryFailure".to_string(),
            ],
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

impl UnifiedRetryConfig {
    /// High performance configuration with aggressive retries
    pub fn high_performance() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(60)),
            ..Default::default()
        }
    }

    /// High reliability configuration with conservative retries
    pub fn high_reliability() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(300),
            backoff_multiplier: 2.5,
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(1800)), // 30 minutes
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "ServiceUnavailable".to_string(),
                "TemporaryFailure".to_string(),
                "RateLimitExceeded".to_string(),
                "InternalServerError".to_string(),
            ],
        }
    }

    /// Configuration optimized for alert delivery
    pub fn alert_optimized() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            enable_jitter: false,
            max_total_duration: Some(Duration::from_secs(180)),
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "ServiceUnavailable".to_string(),
                "TimeoutError".to_string(),
            ],
        }
    }

    /// Configuration optimized for pipeline operations
    pub fn pipeline_optimized() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(300)),
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "ServiceUnavailable".to_string(),
            ],
        }
    }

    /// Configuration optimized for delivery operations
    pub fn delivery_optimized() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(10000),
            backoff_multiplier: 2.0,
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(60)),
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "ServiceUnavailable".to_string(),
            ],
        }
    }

    /// Configuration optimized for validation operations
    pub fn validation_optimized() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            enable_jitter: false,
            max_total_duration: Some(Duration::from_secs(30)),
            retry_on_errors: vec![
                "ValidationError".to_string(),
                "TemporaryFailure".to_string(),
            ],
        }
    }

    /// Configuration optimized for adapter operations
    pub fn adapter_optimized() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(60)),
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "ServiceUnavailable".to_string(),
                "LegacySystemError".to_string(),
            ],
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.max_attempts == 0 {
            return Err(ArbitrageError::config_error(
                "Max attempts must be greater than 0",
            ));
        }

        if self.initial_delay.is_zero() {
            return Err(ArbitrageError::config_error(
                "Initial delay must be greater than 0",
            ));
        }

        if self.max_delay < self.initial_delay {
            return Err(ArbitrageError::config_error(
                "Max delay must be greater than or equal to initial delay",
            ));
        }

        if self.backoff_multiplier <= 0.0 {
            return Err(ArbitrageError::config_error(
                "Backoff multiplier must be greater than 0",
            ));
        }

        Ok(())
    }

    /// Calculate delay for a specific attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let base_delay = self.initial_delay.as_millis() as f64;
        let multiplier = self.backoff_multiplier.powi((attempt - 1) as i32);
        let calculated_delay = Duration::from_millis((base_delay * multiplier) as u64);

        let delay = std::cmp::min(calculated_delay, self.max_delay);

        if self.enable_jitter {
            self.add_jitter(delay)
        } else {
            delay
        }
    }

    /// Add jitter to delay to avoid thundering herd
    fn add_jitter(&self, delay: Duration) -> Duration {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        let seed = hasher.finish();

        // Simple linear congruential generator for jitter
        let jitter_factor =
            ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % 100) as f64 / 100.0;
        let jitter_range = delay.as_millis() as f64 * 0.1; // 10% jitter
        let jitter = Duration::from_millis((jitter_range * jitter_factor) as u64);

        delay + jitter
    }

    /// Check if error should be retried
    pub fn should_retry_error(&self, error: &str) -> bool {
        self.retry_on_errors.iter().any(|retry_error| {
            error.contains(retry_error)
                || error.to_lowercase().contains(&retry_error.to_lowercase())
        })
    }
}

/// Unified retry executor for handling retry logic
#[derive(Debug)]
pub struct UnifiedRetryExecutor {
    config: UnifiedRetryConfig,
}

impl UnifiedRetryExecutor {
    pub fn new(config: UnifiedRetryConfig) -> ArbitrageResult<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    pub fn get_config(&self) -> &UnifiedRetryConfig {
        &self.config
    }
}

impl UnifiedRetryConfig {}
