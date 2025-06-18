//! Cloudflare Native Health Service - Simple health monitoring for Workers
//!
//! This service provides lightweight health monitoring tailored for Cloudflare Workers:
//! - Simple boolean health status checks
//! - Basic service availability verification
//! - Lightweight error counting
//! - No complex state management or persistent metrics
//! - WASM-compatible and Workers-optimized

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use js_sys;

/// Simple health status for a service component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

/// Basic health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub service_name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check_timestamp: u64,
    pub error_count: u32,
}

impl HealthCheckResult {
    pub fn healthy(service_name: String) -> Self {
        Self {
            service_name,
            status: HealthStatus::Healthy,
            message: None,
            last_check_timestamp: Self::current_timestamp(),
            error_count: 0,
        }
    }

    fn current_timestamp() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
    }
}

/// Simple health service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareHealthConfig {
    pub enabled: bool,
    pub max_errors: u32,
    pub timeout_ms: u64,
}

impl Default for CloudflareHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_errors: 5,
            timeout_ms: 1000,
        }
    }
}

/// Cloudflare native health service
pub struct CloudflareHealthService {
    config: CloudflareHealthConfig,
    error_counters: Arc<Mutex<HashMap<String, u32>>>,
    logger: crate::utils::logger::Logger,
}

impl CloudflareHealthService {
    pub fn new(config: CloudflareHealthConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
        
        logger.info("CloudflareHealthService initialized with simple monitoring");
        
        Ok(Self {
            config,
            error_counters: Arc::new(Mutex::new(HashMap::new())),
            logger,
        })
    }

    pub async fn record_success(&self, service_name: &str) {
        if !self.config.enabled {
            return;
        }

        if let Ok(mut counters) = self.error_counters.lock() {
            counters.insert(service_name.to_string(), 0);
        }
    }

    pub async fn record_error(&self, service_name: &str, _error: &ArbitrageError) {
        if !self.config.enabled {
            return;
        }

        if let Ok(mut counters) = self.error_counters.lock() {
            let count = counters.entry(service_name.to_string()).or_insert(0);
            *count += 1;
        }
    }

    pub async fn is_healthy(&self) -> bool {
        if !self.config.enabled {
            return true;
        }

        if let Ok(counters) = self.error_counters.lock() {
            return counters.values().all(|&count| count < self.config.max_errors);
        }

        true
    }

    pub async fn health_endpoint(&self) -> ArbitrageResult<serde_json::Value> {
        Ok(serde_json::json!({
            "status": if self.is_healthy().await { "healthy" } else { "unhealthy" },
            "timestamp": HealthCheckResult::current_timestamp()
        }))
    }
}

/// Simple trait for services to implement basic health checks
pub trait SimpleHealthCheck {
    async fn health_check(&self) -> ArbitrageResult<bool>;
} 