use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use worker::{kv::KvStore, Env};

/// System configuration service for super admin operations
#[derive(Clone)]
pub struct SystemConfigService {
    kv_store: KvStore,
}

impl SystemConfigService {
    pub fn new(_env: Env, kv_store: KvStore) -> Self {
        Self { kv_store }
    }

    /// Generic helper to get configuration from KV store
    async fn get_config<T: DeserializeOwned + Default>(&self, key: &str) -> ArbitrageResult<T> {
        if let Some(data) = self.kv_store.get(key).text().await? {
            serde_json::from_str::<T>(&data).map_err(|e| {
                ArbitrageError::database_error(format!("Failed to parse {}: {}", key, e))
            })
        } else {
            Ok(T::default())
        }
    }

    /// Get system configuration
    pub async fn get_system_config(&self) -> ArbitrageResult<SystemConfig> {
        self.get_config("system_config").await
    }

    /// Update system configuration (super admin only)
    pub async fn update_system_config(&self, config: SystemConfig) -> ArbitrageResult<()> {
        let config_key = "system_config";
        let config_data = serde_json::to_string(&config).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize system config: {}", e))
        })?;

        self.kv_store
            .put(config_key, &config_data)?
            .execute()
            .await?;

        Ok(())
    }

    /// Get feature flags configuration
    pub async fn get_feature_flags(&self) -> ArbitrageResult<FeatureFlagsConfig> {
        self.get_config("feature_flags").await
    }

    /// Update feature flags (super admin only)
    pub async fn update_feature_flags(&self, flags: FeatureFlagsConfig) -> ArbitrageResult<()> {
        let flags_key = "feature_flags";
        let flags_data = serde_json::to_string(&flags).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize feature flags: {}", e))
        })?;

        self.kv_store.put(flags_key, &flags_data)?.execute().await?;

        Ok(())
    }

    /// Get current system configuration (alias for get_system_config)
    pub async fn get_current_config(&self) -> ArbitrageResult<SystemConfig> {
        self.get_system_config().await
    }

    /// Update specific configuration value
    pub async fn update_config(
        &self,
        config_key: &str,
        config_value: serde_json::Value,
    ) -> ArbitrageResult<()> {
        let mut system_config = self.get_system_config().await?;

        // Update the specific field based on config_key
        match config_key {
            "max_concurrent_users" => {
                if let Some(value) = config_value.as_u64() {
                    system_config.max_concurrent_users = value as u32;
                }
            }
            "session_timeout_minutes" => {
                if let Some(value) = config_value.as_u64() {
                    system_config.session_timeout_minutes = value as u32;
                }
            }
            "log_level" => {
                if let Some(value) = config_value.as_str() {
                    system_config.log_level = value.to_string();
                }
            }
            _ => {
                return Err(ArbitrageError::validation_error(format!(
                    "Unknown config key: {}",
                    config_key
                )));
            }
        }

        system_config.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.update_system_config(system_config).await
    }

    /// Enable maintenance mode
    pub async fn enable_maintenance_mode(&self) -> ArbitrageResult<()> {
        let mut maintenance = self.get_maintenance_mode().await?;
        maintenance.enabled = true;
        maintenance.started_at = Some(chrono::Utc::now().timestamp_millis() as u64);
        maintenance.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.update_maintenance_mode(maintenance).await
    }

    /// Disable maintenance mode
    pub async fn disable_maintenance_mode(&self) -> ArbitrageResult<()> {
        let mut maintenance = self.get_maintenance_mode().await?;
        maintenance.enabled = false;
        maintenance.started_at = None;
        maintenance.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.update_maintenance_mode(maintenance).await
    }

    /// Get maintenance mode configuration
    pub async fn get_maintenance_mode(&self) -> ArbitrageResult<MaintenanceMode> {
        self.get_config("maintenance_mode").await
    }

    /// Update maintenance mode configuration
    pub async fn update_maintenance_mode(
        &self,
        maintenance: MaintenanceMode,
    ) -> ArbitrageResult<()> {
        let maintenance_key = "maintenance_mode";
        let maintenance_data = serde_json::to_string(&maintenance).map_err(|e| {
            ArbitrageError::serialization_error(format!(
                "Failed to serialize maintenance mode: {}",
                e
            ))
        })?;

        self.kv_store
            .put(maintenance_key, &maintenance_data)?
            .execute()
            .await?;

        Ok(())
    }

    /// Health check for system config service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        // Try to read system config to verify KV store connectivity
        match self.get_system_config().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get rate limiting configuration
    pub async fn get_rate_limits(&self) -> ArbitrageResult<RateLimitConfig> {
        self.get_config("rate_limits").await
    }

    /// Update rate limiting configuration (super admin only)
    pub async fn update_rate_limits(&self, limits: RateLimitConfig) -> ArbitrageResult<()> {
        let limits_key = "rate_limits";
        let limits_data = serde_json::to_string(&limits).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize rate limits: {}", e))
        })?;

        self.kv_store
            .put(limits_key, &limits_data)?
            .execute()
            .await?;

        Ok(())
    }

    /// Get API configuration
    pub async fn get_api_config(&self) -> ArbitrageResult<ApiConfig> {
        self.get_config("api_config").await
    }

    /// Update API configuration (super admin only)
    pub async fn update_api_config(&self, config: ApiConfig) -> ArbitrageResult<()> {
        let api_key = "api_config";
        let api_data = serde_json::to_string(&config).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize API config: {}", e))
        })?;

        self.kv_store.put(api_key, &api_data)?.execute().await?;

        Ok(())
    }

    /// Get all configuration as a summary
    pub async fn get_config_summary(&self) -> ArbitrageResult<ConfigSummary> {
        let system_config = self.get_system_config().await?;
        let feature_flags = self.get_feature_flags().await?;
        let rate_limits = self.get_rate_limits().await?;
        let api_config = self.get_api_config().await?;
        let maintenance_mode = self.get_maintenance_mode().await?;

        Ok(ConfigSummary {
            system_config,
            feature_flags,
            rate_limits,
            api_config,
            maintenance_mode,
        })
    }
}

/// System configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub app_name: String,
    pub app_version: String,
    pub environment: String, // "development", "staging", "production"
    pub max_concurrent_users: u32,
    pub session_timeout_minutes: u32,
    pub default_opportunity_ttl_seconds: u32,
    pub max_opportunities_per_user_per_hour: u32,
    pub enable_analytics: bool,
    pub enable_logging: bool,
    pub log_level: String, // "debug", "info", "warn", "error"
    pub updated_at: u64,
    pub updated_by: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            app_name: "ArbEdge".to_string(),
            app_version: "0.1.0".to_string(),
            environment: "development".to_string(),
            max_concurrent_users: 10000,
            session_timeout_minutes: 30,
            default_opportunity_ttl_seconds: 300, // 5 minutes
            max_opportunities_per_user_per_hour: 100,
            enable_analytics: true,
            enable_logging: true,
            log_level: "info".to_string(),
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_by: "system".to_string(),
        }
    }
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlagsConfig {
    pub modular_auth: bool,
    pub modular_telegram: bool,
    pub modular_trading: bool,
    pub modular_ai: bool,
    pub modular_analytics: bool,
    pub super_admin_priority: bool,
    pub beta_features_enabled: bool,
    pub ai_enhancement_enabled: bool,
    pub group_management_enabled: bool,
    pub advanced_analytics_enabled: bool,
    pub updated_at: u64,
    pub updated_by: String,
}

impl Default for FeatureFlagsConfig {
    fn default() -> Self {
        Self {
            modular_auth: false,
            modular_telegram: false,
            modular_trading: false,
            modular_ai: false,
            modular_analytics: false,
            super_admin_priority: true,
            beta_features_enabled: true,
            ai_enhancement_enabled: true,
            group_management_enabled: true,
            advanced_analytics_enabled: false,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_by: "system".to_string(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    pub api_requests_per_minute: u32,
    pub telegram_messages_per_minute: u32,
    pub opportunities_per_hour: u32,
    pub login_attempts_per_hour: u32,
    pub enable_rate_limiting: bool,
    pub rate_limit_window_minutes: u32,
    pub updated_at: u64,
    pub updated_by: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            api_requests_per_minute: 60,
            telegram_messages_per_minute: 20,
            opportunities_per_hour: 100,
            login_attempts_per_hour: 10,
            enable_rate_limiting: true,
            rate_limit_window_minutes: 60,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_by: "system".to_string(),
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
    pub api_timeout_seconds: u32,
    pub max_request_size_mb: u32,
    pub enable_compression: bool,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u32,
    pub updated_at: u64,
    pub updated_by: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            allowed_origins: vec!["*".to_string()],
            api_timeout_seconds: 30,
            max_request_size_mb: 10,
            enable_compression: true,
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_by: "system".to_string(),
        }
    }
}

/// Maintenance mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceMode {
    pub enabled: bool,
    pub message: String,
    pub allowed_user_ids: Vec<String>, // Users who can still access during maintenance
    pub estimated_duration_minutes: Option<u32>,
    pub started_at: Option<u64>,
    pub updated_at: u64,
    pub updated_by: String,
}

impl Default for MaintenanceMode {
    fn default() -> Self {
        Self {
            enabled: false,
            message: "System is under maintenance. Please try again later.".to_string(),
            allowed_user_ids: Vec::new(),
            estimated_duration_minutes: None,
            started_at: None,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_by: "system".to_string(),
        }
    }
}

/// Configuration summary for admin dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSummary {
    pub system_config: SystemConfig,
    pub feature_flags: FeatureFlagsConfig,
    pub rate_limits: RateLimitConfig,
    pub api_config: ApiConfig,
    pub maintenance_mode: MaintenanceMode,
}
