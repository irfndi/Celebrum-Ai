use serde::{Deserialize, Serialize};

/// Unified health check configuration that consolidates all health check needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedHealthCheckConfig {
    /// Health check interval in seconds
    pub interval_seconds: u64,
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Consecutive successes for recovery
    pub success_threshold: u32,
    /// Critical services that must be healthy
    pub critical_services: Vec<String>,
    /// Response time threshold for degraded status (ms)
    pub degraded_threshold_ms: f64,
    /// Response time threshold for unhealthy status (ms)
    pub unhealthy_threshold_ms: f64,
    /// Enable dependency checks
    pub enable_dependency_checks: bool,
    /// Enable detailed metrics collection
    pub enable_detailed_metrics: bool,
    /// Health check method
    pub check_method: HealthCheckMethod,
    /// Expected HTTP status codes (for HTTP checks)
    pub expected_status_codes: Vec<u16>,
    /// Custom validation expression
    pub custom_validation: Option<String>,
    /// Health endpoints to check
    pub endpoints: Vec<String>,
}

impl Default for UnifiedHealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            timeout_seconds: 10,
            max_retries: 2,
            failure_threshold: 3,
            success_threshold: 2,
            critical_services: vec![
                "database".to_string(),
                "cache".to_string(),
                "notifications".to_string(),
            ],
            degraded_threshold_ms: 1000.0,
            unhealthy_threshold_ms: 5000.0,
            enable_dependency_checks: true,
            enable_detailed_metrics: true,
            check_method: HealthCheckMethod::Ping,
            expected_status_codes: vec![200],
            custom_validation: None,
            endpoints: Vec::new(),
        }
    }
}

impl UnifiedHealthCheckConfig {
    /// Configuration optimized for service health monitoring
    pub fn service_optimized() -> Self {
        Self {
            interval_seconds: 30,
            timeout_seconds: 10,
            max_retries: 2,
            failure_threshold: 3,
            success_threshold: 2,
            degraded_threshold_ms: 1000.0,
            unhealthy_threshold_ms: 5000.0,
            enable_dependency_checks: true,
            enable_detailed_metrics: true,
            check_method: HealthCheckMethod::HttpGet,
            expected_status_codes: vec![200, 201, 202],
            ..Default::default()
        }
    }

    /// Configuration optimized for failover scenarios
    pub fn failover_optimized() -> Self {
        Self {
            interval_seconds: 15,
            timeout_seconds: 5,
            max_retries: 1,
            failure_threshold: 2,
            success_threshold: 3,
            degraded_threshold_ms: 500.0,
            unhealthy_threshold_ms: 2000.0,
            enable_dependency_checks: false,
            enable_detailed_metrics: false,
            check_method: HealthCheckMethod::Ping,
            expected_status_codes: vec![200],
            ..Default::default()
        }
    }

    /// Configuration optimized for external system integration
    pub fn external_system_optimized() -> Self {
        Self {
            interval_seconds: 60,
            timeout_seconds: 15,
            max_retries: 3,
            failure_threshold: 5,
            success_threshold: 2,
            degraded_threshold_ms: 2000.0,
            unhealthy_threshold_ms: 10000.0,
            enable_dependency_checks: true,
            enable_detailed_metrics: true,
            check_method: HealthCheckMethod::Custom("external_check".to_string()),
            expected_status_codes: vec![200, 202, 204],
            ..Default::default()
        }
    }
}

/// Health check method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckMethod {
    /// Simple ping/connectivity check
    Ping,
    /// HTTP GET request
    HttpGet,
    /// HTTP POST request
    HttpPost,
    /// Database query
    DatabaseQuery,
    /// KV store operation
    KvOperation,
    /// Custom check method
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_health_check_config_default() {
        let config = UnifiedHealthCheckConfig::default();
        assert_eq!(config.interval_seconds, 30);
        assert_eq!(config.timeout_seconds, 10);
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.success_threshold, 2);
        assert!(config.enable_dependency_checks);
        assert!(config.enable_detailed_metrics);
    }

    #[test]
    fn test_service_optimized_config() {
        let config = UnifiedHealthCheckConfig::service_optimized();
        assert_eq!(config.interval_seconds, 30);
        assert_eq!(config.degraded_threshold_ms, 1000.0);
        assert!(matches!(config.check_method, HealthCheckMethod::HttpGet));
        assert_eq!(config.expected_status_codes, vec![200, 201, 202]);
    }

    #[test]
    fn test_failover_optimized_config() {
        let config = UnifiedHealthCheckConfig::failover_optimized();
        assert_eq!(config.interval_seconds, 15);
        assert_eq!(config.timeout_seconds, 5);
        assert_eq!(config.failure_threshold, 2);
        assert!(!config.enable_dependency_checks);
        assert!(matches!(config.check_method, HealthCheckMethod::Ping));
    }

    #[test]
    fn test_external_system_optimized_config() {
        let config = UnifiedHealthCheckConfig::external_system_optimized();
        assert_eq!(config.interval_seconds, 60);
        assert_eq!(config.timeout_seconds, 15);
        assert_eq!(config.failure_threshold, 5);
        assert!(matches!(config.check_method, HealthCheckMethod::Custom(_)));
    }
}
