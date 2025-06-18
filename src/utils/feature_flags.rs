#![allow(static_mut_refs, unused_must_use, clippy::items_after_test_module)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use worker::console_log;

/// Feature flag configuration for production-ready feature management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub value: Option<serde_json::Value>,
    pub description: String,
    pub rollout_percentage: Option<u8>, // 0-100
}

/// Feature flag manager with caching and environment support
#[derive(Debug, Clone)]
pub struct FeatureFlagManager {
    flags: HashMap<String, FeatureFlag>,
    environment: String,
}

impl Default for FeatureFlagManager {
    fn default() -> Self {
        let mut flags = HashMap::new();

        // Production configuration flags
        flags.insert(
            "opportunity_engine.enhanced_logging".to_string(),
            FeatureFlag {
                name: "opportunity_engine.enhanced_logging".to_string(),
                enabled: true,
                value: None,
                description: "Enable enhanced console logging for opportunity generation debugging"
                    .to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "opportunity_engine.min_rate_threshold".to_string(),
            FeatureFlag {
                name: "opportunity_engine.min_rate_threshold".to_string(),
                enabled: true,
                value: Some(serde_json::json!(0.05)), // Lower threshold: 0.05%
                description: "Minimum rate difference threshold for arbitrage opportunities"
                    .to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "exchange_service.fault_tolerance".to_string(),
            FeatureFlag {
                name: "exchange_service.fault_tolerance".to_string(),
                enabled: true,
                value: None,
                description: "Enable fault-tolerant exchange API calls with graceful degradation"
                    .to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "exchange_service.concurrent_ticker_fetching".to_string(),
            FeatureFlag {
                name: "exchange_service.concurrent_ticker_fetching".to_string(),
                enabled: true,
                value: Some(serde_json::json!({"max_concurrent": 5, "timeout_ms": 3000})),
                description:
                    "Enable concurrent ticker fetching with configurable concurrency limits"
                        .to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "geographic_fallback.enabled".to_string(),
            FeatureFlag {
                name: "geographic_fallback.enabled".to_string(),
                enabled: true,
                value: None,
                description: "Enable geographic fallback for API endpoints".to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "geographic_fallback.binance_us_fallback".to_string(),
            FeatureFlag {
                name: "geographic_fallback.binance_us_fallback".to_string(),
                enabled: true,
                value: None,
                description: "Enable Binance US API fallback for geographic restrictions"
                    .to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "caching.aggressive_ticker_caching".to_string(),
            FeatureFlag {
                name: "caching.aggressive_ticker_caching".to_string(),
                enabled: true,
                value: Some(serde_json::json!({"ttl_seconds": 30, "stale_while_revalidate": 60})),
                description: "Enable aggressive ticker data caching with stale-while-revalidate"
                    .to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "monitoring.detailed_metrics".to_string(),
            FeatureFlag {
                name: "monitoring.detailed_metrics".to_string(),
                enabled: true,
                value: None,
                description: "Enable detailed performance and error metrics collection".to_string(),
                rollout_percentage: Some(100),
            },
        );

        flags.insert(
            "resilience.circuit_breaker".to_string(),
            FeatureFlag {
                name: "resilience.circuit_breaker".to_string(),
                enabled: true,
                value: Some(serde_json::json!({
                    "failure_threshold": 5,
                    "recovery_timeout_ms": 60000,
                    "half_open_max_calls": 2
                })),
                description: "Enable circuit breaker pattern for external API calls".to_string(),
                rollout_percentage: Some(100),
            },
        );

        Self {
            flags,
            environment: "production".to_string(),
        }
    }
}

impl FeatureFlagManager {
    pub fn new(environment: String) -> Self {
        Self {
            environment,
            ..Self::default()
        }
    }

    pub fn is_enabled(&self, flag_name: &str) -> bool {
        if let Some(flag) = self.flags.get(flag_name) {
            flag.enabled && self.check_rollout(flag)
        } else {
            false
        }
    }

    pub fn get_value<T>(&self, flag_name: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        if let Some(flag) = self.flags.get(flag_name) {
            if flag.enabled && self.check_rollout(flag) {
                if let Some(ref value) = flag.value {
                    return serde_json::from_value(value.clone()).ok();
                }
            }
        }
        None
    }

    pub fn get_numeric_value(&self, flag_name: &str) -> Option<f64> {
        self.get_value::<f64>(flag_name)
    }

    pub fn get_string_value(&self, flag_name: &str) -> Option<String> {
        self.get_value::<String>(flag_name)
    }

    fn check_rollout(&self, flag: &FeatureFlag) -> bool {
        if let Some(percentage) = flag.rollout_percentage {
            if percentage >= 100 {
                return true;
            }
            // Simple hash-based rollout (in production, use proper user ID hashing)
            let hash = flag.name.len() % 100;
            hash < percentage as usize
        } else {
            true
        }
    }

    pub fn add_flag(&mut self, flag: FeatureFlag) {
        self.flags.insert(flag.name.clone(), flag);
    }

    pub fn remove_flag(&mut self, flag_name: &str) {
        self.flags.remove(flag_name);
    }

    /// Get the current environment
    pub fn get_environment(&self) -> &str {
        &self.environment
    }

    pub fn list_active_flags(&self) -> Vec<&FeatureFlag> {
        self.flags
            .values()
            .filter(|flag| flag.enabled && self.check_rollout(flag))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_manager() {
        let manager = FeatureFlagManager::default();

        // Test enabled flag
        assert!(manager.is_enabled("opportunity_engine.enhanced_logging"));

        // Test disabled flag (non-existent)
        assert!(!manager.is_enabled("non_existent_flag"));

        // Test value retrieval
        let threshold: Option<f64> = manager.get_value("opportunity_engine.min_rate_threshold");
        assert_eq!(threshold, Some(0.05));
    }

    #[test]
    fn test_global_feature_flag_access() {
        assert!(is_feature_enabled("opportunity_engine.enhanced_logging").unwrap_or(false));

        let threshold = get_numeric_feature_value("opportunity_engine.min_rate_threshold", 0.1);
        assert_eq!(threshold, 0.05);
    }
}

/// Production-ready feature flags for ArbEdge platform
pub struct FeatureFlags {
    flags: HashMap<String, Value>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        let mut flags = HashMap::new();

        // === OPPORTUNITY ENGINE CONFIGURATION ===
        flags.insert(
            "opportunity_engine.enhanced_logging".to_string(),
            Value::Bool(true),
        );
        flags.insert(
            "opportunity_engine.min_rate_threshold".to_string(),
            Value::Number(serde_json::Number::from_f64(0.05).unwrap()),
        ); // 0.05% minimum threshold

        // Legacy alias for backward compatibility
        flags.insert(
            "min_arbitrage_rate_threshold".to_string(),
            Value::Number(serde_json::Number::from_f64(0.05).unwrap()),
        );
        flags.insert(
            "opportunity_engine.min_confidence_threshold".to_string(),
            Value::Number(serde_json::Number::from_f64(60.0).unwrap()),
        ); // 60%
        flags.insert(
            "opportunity_engine.max_opportunities_per_user".to_string(),
            Value::Number(serde_json::Number::from(5)),
        );
        flags.insert(
            "opportunity_engine.quality_filter_enabled".to_string(),
            Value::Bool(true),
        );
        flags.insert(
            "opportunity_engine.funding_rate_arbitrage_enabled".to_string(),
            Value::Bool(true),
        );

        // === FUNDING RATE MANAGEMENT ===
        flags.insert(
            "funding_rate.min_profit_threshold".to_string(),
            Value::Number(serde_json::Number::from_f64(0.008).unwrap()),
        ); // 0.8%
        flags.insert(
            "funding_rate.execution_time_buffer_minutes".to_string(),
            Value::Number(serde_json::Number::from(30)),
        );
        flags.insert(
            "funding_rate.fetch_retry_attempts".to_string(),
            Value::Number(serde_json::Number::from(3)),
        );
        flags.insert(
            "funding_rate.api_timeout_seconds".to_string(),
            Value::Number(serde_json::Number::from(10)),
        );

        // === PERFORMANCE & CACHING ===
        flags.insert(
            "cache.enhanced_multi_tier_enabled".to_string(),
            Value::Bool(true),
        );
        flags.insert(
            "cache.funding_rate_ttl_seconds".to_string(),
            Value::Number(serde_json::Number::from(3600)),
        ); // 1 hour
        flags.insert(
            "cache.market_data_ttl_seconds".to_string(),
            Value::Number(serde_json::Number::from(60)),
        ); // 1 minute
        flags.insert(
            "cache.opportunity_ttl_seconds".to_string(),
            Value::Number(serde_json::Number::from(300)),
        ); // 5 minutes

        // === API RATE LIMITING ===
        flags.insert(
            "api.binance_rate_limit_per_minute".to_string(),
            Value::Number(serde_json::Number::from(1200)),
        );
        flags.insert(
            "api.bybit_rate_limit_per_minute".to_string(),
            Value::Number(serde_json::Number::from(600)),
        );
        flags.insert(
            "api.okx_rate_limit_per_minute".to_string(),
            Value::Number(serde_json::Number::from(600)),
        );
        flags.insert(
            "api.coinbase_rate_limit_per_minute".to_string(),
            Value::Number(serde_json::Number::from(100)),
        );

        // === TELEGRAM INTERFACE ===
        flags.insert(
            "telegram.max_opportunities_per_message".to_string(),
            Value::Number(serde_json::Number::from(5)),
        );
        flags.insert(
            "telegram.funding_rate_display_precision".to_string(),
            Value::Number(serde_json::Number::from(4)),
        ); // 4 decimal places
        flags.insert(
            "telegram.auto_refresh_enabled".to_string(),
            Value::Bool(true),
        );
        flags.insert(
            "telegram.enhanced_formatting".to_string(),
            Value::Bool(true),
        );

        // === PRODUCTION SAFETY ===
        flags.insert(
            "production.fail_fast_on_api_errors".to_string(),
            Value::Bool(true),
        ); // No fallback to mock data
        flags.insert(
            "production.require_super_admin_credentials".to_string(),
            Value::Bool(true),
        );
        flags.insert(
            "production.validate_opportunity_quality".to_string(),
            Value::Bool(true),
        );
        flags.insert(
            "production.enable_monitoring".to_string(),
            Value::Bool(true),
        );

        #[cfg(target_arch = "wasm32")]
        {
            console_log!(
                "🚀 FEATURE_FLAGS - Production configuration loaded with {} flags",
                flags.len()
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!(
                "🚀 FEATURE_FLAGS - Production configuration loaded with {} flags",
                flags.len()
            );
        }

        Self { flags }
    }
}

impl FeatureFlags {
    /// Create new instance with provided flags (for backward compatibility)
    pub fn new(flags: HashMap<String, Value>) -> Self {
        Self { flags }
    }

    /// Check if a feature is enabled (for backward compatibility with service container)
    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        self.get_bool_flag(feature_name, false)
    }

    /// Get a feature flag value with type checking
    pub fn get_flag_value(&self, key: &str) -> Option<&Value> {
        self.flags.get(key)
    }

    /// Get boolean flag with default fallback
    pub fn get_bool_flag(&self, key: &str, default: bool) -> bool {
        self.flags
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Get numeric flag with default fallback
    pub fn get_numeric_flag(&self, key: &str, default: f64) -> f64 {
        self.flags
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or(default)
    }

    /// Get integer flag with default fallback
    pub fn get_int_flag(&self, key: &str, default: i64) -> i64 {
        self.flags
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    /// Update a flag value at runtime
    pub fn set_flag(&mut self, key: String, value: Value) {
        #[cfg(target_arch = "wasm32")]
        console_log!("🔧 FEATURE_FLAGS - Updated flag: {} = {:?}", key, value);
        #[cfg(not(target_arch = "wasm32"))]
        println!("🔧 FEATURE_FLAGS - Updated flag: {} = {:?}", key, value);
        self.flags.insert(key, value);
    }

    /// Get all flags for debugging
    pub fn get_all_flags(&self) -> &HashMap<String, Value> {
        &self.flags
    }
}

/// Backward compatibility function for service container
pub fn load_feature_flags(
    _path: &str,
) -> Result<std::sync::Arc<FeatureFlags>, Box<dyn std::error::Error>> {
    Ok(std::sync::Arc::new(FeatureFlags::default()))
}

// Global feature flags instance
static mut FEATURE_FLAGS: Option<FeatureFlags> = None;

/// Initialize feature flags (call once at startup)
pub fn initialize() {
    unsafe {
        FEATURE_FLAGS = Some(FeatureFlags::default());
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log!("✅ FEATURE_FLAGS - Initialized successfully");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("✅ FEATURE_FLAGS - Initialized successfully");
    }
}

/// Check if a feature is enabled
pub fn is_feature_enabled(feature_name: &str) -> Option<bool> {
    unsafe {
        if FEATURE_FLAGS.is_none() {
            initialize();
        }
        FEATURE_FLAGS
            .as_ref()
            .map(|flags| flags.get_bool_flag(feature_name, false))
    }
}

/// Get numeric feature value
pub fn get_numeric_feature_value(feature_name: &str, default: f64) -> f64 {
    unsafe {
        if FEATURE_FLAGS.is_none() {
            initialize();
        }
        FEATURE_FLAGS
            .as_ref()
            .map(|flags| flags.get_numeric_flag(feature_name, default))
            .unwrap_or(default)
    }
}

/// Get integer feature value
pub fn get_int_feature_value(feature_name: &str, default: i64) -> i64 {
    unsafe {
        if FEATURE_FLAGS.is_none() {
            initialize();
        }
        FEATURE_FLAGS
            .as_ref()
            .map(|flags| flags.get_int_flag(feature_name, default))
            .unwrap_or(default)
    }
}

/// Update feature flag at runtime (for testing/configuration)
pub fn set_feature_flag(feature_name: &str, value: Value) {
    unsafe {
        if let Some(flags) = FEATURE_FLAGS.as_mut() {
            flags.set_flag(feature_name.to_string(), value);
        }
    }
}

// Production-ready feature flags configuration
pub fn get_production_feature_flags() -> FeatureFlags {
    let mut flags = HashMap::new();

    // Opportunity Engine Settings
    flags.insert(
        "min_arbitrage_rate_threshold".to_string(),
        Value::Number(serde_json::Number::from_f64(0.0015).unwrap()),
    ); // 0.15% minimum for production
    flags.insert(
        "min_confidence_score".to_string(),
        Value::Number(serde_json::Number::from_f64(60.0).unwrap()),
    ); // Higher confidence required
    flags.insert("enable_quality_filters".to_string(), Value::Bool(true));
    flags.insert(
        "max_opportunities_per_user".to_string(),
        Value::Number(serde_json::Number::from(5)),
    ); // Focus on quality over quantity
    flags.insert(
        "max_opportunities_per_group".to_string(),
        Value::Number(serde_json::Number::from(10)),
    );
    flags.insert(
        "max_opportunities_global".to_string(),
        Value::Number(serde_json::Number::from(15)),
    );

    // Funding Rate Management
    flags.insert(
        "funding_rate_profit_threshold".to_string(),
        Value::Number(serde_json::Number::from_f64(0.001).unwrap()),
    ); // 0.1% minimum profit
    flags.insert(
        "funding_rate_execution_buffer_minutes".to_string(),
        Value::Number(serde_json::Number::from(30)),
    ); // 30 min buffer before funding
    flags.insert(
        "funding_rate_api_timeout_seconds".to_string(),
        Value::Number(serde_json::Number::from(10)),
    );
    flags.insert("enable_funding_rate_caching".to_string(), Value::Bool(true));
    flags.insert(
        "funding_rate_cache_ttl_seconds".to_string(),
        Value::Number(serde_json::Number::from(60)),
    ); // 1 minute for real-time data

    // Performance & Caching
    flags.insert("enable_multi_tier_caching".to_string(), Value::Bool(true));
    flags.insert(
        "cache_ttl_seconds".to_string(),
        Value::Number(serde_json::Number::from(60)),
    ); // Shorter TTL for production
    flags.insert("enable_background_refresh".to_string(), Value::Bool(true));
    flags.insert(
        "max_concurrent_api_calls".to_string(),
        Value::Number(serde_json::Number::from(10)),
    );

    // API Rate Limiting
    flags.insert(
        "binance_rate_limit_per_minute".to_string(),
        Value::Number(serde_json::Number::from(1200)),
    );
    flags.insert(
        "bybit_rate_limit_per_minute".to_string(),
        Value::Number(serde_json::Number::from(600)),
    );
    flags.insert(
        "okx_rate_limit_per_minute".to_string(),
        Value::Number(serde_json::Number::from(600)),
    );

    // Telegram Interface
    flags.insert(
        "enable_telegram_notifications".to_string(),
        Value::Bool(true),
    );
    flags.insert(
        "telegram_max_message_length".to_string(),
        Value::Number(serde_json::Number::from(4000)),
    );
    flags.insert(
        "telegram_rate_limit_per_minute".to_string(),
        Value::Number(serde_json::Number::from(20)),
    );

    // Production Safety
    flags.insert("enable_fail_fast_on_errors".to_string(), Value::Bool(true));
    flags.insert(
        "require_super_admin_credentials".to_string(),
        Value::Bool(true),
    );
    flags.insert(
        "validate_opportunity_quality".to_string(),
        Value::Bool(true),
    );
    flags.insert(
        "enable_comprehensive_logging".to_string(),
        Value::Bool(true),
    );

    FeatureFlags { flags }
}
