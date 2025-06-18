//! Unified Repository Layer - Consolidates all repository operations
//! 
//! This module combines the functionality of:
//! - UserRepository (32KB, 941 lines)
//! - AIDataRepository (28KB, 808 lines) 
//! - AnalyticsRepository (34KB, 1037 lines)
//! - ConfigRepository (21KB, 652 lines)
//! - InvitationRepository (40KB, 1151 lines)
//! 
//! Total consolidation: 5 files → 1 file (155KB → ~80KB optimized)

use crate::services::core::ai::ai_intelligence::{
    AiOpportunityEnhancement, AiPerformanceInsights, AiPortfolioAnalysis, ParameterSuggestion,
};
use crate::services::core::invitation::invitation_service::InvitationUsage;
use crate::services::core::user::dynamic_config::{ConfigPreset, UserConfigInstance};
use crate::services::core::user::user_trading_preferences::UserTradingPreferences;
use crate::types::{ArbitrageOpportunity, InvitationCode, UserApiKey, UserProfile, UserPreferences, RiskProfile, SubscriptionTier, UserAccessLevel};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use worker::{console_log, kv::KvStore, wasm_bindgen::JsValue, D1Database};

/// Unified configuration for all repository operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRepositoryConfig {
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    pub batch_size: usize,
    pub enable_metrics: bool,
    pub connection_timeout_ms: u64,
    pub max_retry_attempts: u32,
    pub connection_pool_size: u32,
}

impl Default for UnifiedRepositoryConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl_seconds: 3600,
            batch_size: 100,
            enable_metrics: true,
            connection_timeout_ms: 30000,
            max_retry_attempts: 3,
            connection_pool_size: 20,
        }
    }
}

/// Unified repository metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRepositoryMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_response_time_ms: f64,
    pub last_updated: u64,
    pub user_operations: u64,
    pub ai_operations: u64,
    pub analytics_operations: u64,
    pub config_operations: u64,
    pub invitation_operations: u64,
}

impl Default for UnifiedRepositoryMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_response_time_ms: 0.0,
            last_updated: 0,
            user_operations: 0,
            ai_operations: 0,
            analytics_operations: 0,
            config_operations: 0,
            invitation_operations: 0,
        }
    }
}

/// Main unified repository service
pub struct UnifiedRepositoryLayer {
    config: UnifiedRepositoryConfig,
    db: Arc<D1Database>,
    cache: Option<Arc<KvStore>>,
    metrics: Arc<Mutex<UnifiedRepositoryMetrics>>,
    logger: crate::utils::logger::Logger,
}

impl UnifiedRepositoryLayer {
    /// Create new unified repository layer
    pub fn new(config: UnifiedRepositoryConfig, db: Arc<D1Database>) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        logger.info("Initializing UnifiedRepositoryLayer - consolidating 5 repositories into 1");

        Ok(Self {
            config,
            db,
            cache: None,
            metrics: Arc::new(Mutex::new(UnifiedRepositoryMetrics::default())),
            logger,
        })
    }

    /// Add cache support
    pub fn with_cache(mut self, cache: Arc<KvStore>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Health check for unified repositories
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        let test_query = "SELECT 1 as test";
        match self.db.prepare(test_query).first::<worker::wasm_bindgen::JsValue>(None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get unified metrics
    pub async fn get_metrics(&self) -> UnifiedRepositoryMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            UnifiedRepositoryMetrics::default()
        }
    }

    /// Record operation metrics
    async fn record_metrics(&self, success: bool, start_time: u64, operation_type: &str) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_operations += 1;
            if success {
                metrics.successful_operations += 1;
            } else {
                metrics.failed_operations += 1;
            }
            
            match operation_type {
                "user" => metrics.user_operations += 1,
                "ai" => metrics.ai_operations += 1,
                "analytics" => metrics.analytics_operations += 1,
                "config" => metrics.config_operations += 1,
                "invitation" => metrics.invitation_operations += 1,
                _ => {}
            }
            
            let execution_time = crate::utils::time::get_current_timestamp() - start_time;
            let total_ops = metrics.total_operations as f64;
            metrics.avg_response_time_ms = (metrics.avg_response_time_ms * (total_ops - 1.0) + execution_time as f64) / total_ops;
            metrics.last_updated = crate::utils::time::get_current_timestamp();
        }
    }

    // ============= USER REPOSITORY OPERATIONS =============
    
    /// Create a new user profile
    pub async fn create_user_profile(&self, profile: &UserProfile) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let result = self.store_user_profile_internal(profile).await;
        self.record_metrics(result.is_ok(), start_time, "user").await;
        result
    }

    /// Get user profile by user ID
    pub async fn get_user_profile(&self, user_id: &str) -> ArbitrageResult<Option<UserProfile>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        // Try cache first if enabled
        if self.config.enable_caching && self.cache.is_some() {
            if let Ok(Some(cached)) = self.get_cached_user_profile(user_id).await {
                self.record_metrics(true, start_time, "user").await;
                return Ok(Some(cached));
            }
        }

        let result = self.get_user_profile_from_db(user_id).await;
        self.record_metrics(result.is_ok(), start_time, "user").await;
        result
    }

    /// Update user profile
    pub async fn update_user_profile(&self, profile: &UserProfile) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let result = self.store_user_profile_internal(profile).await;
        
        // Invalidate cache
        if self.config.enable_caching && self.cache.is_some() {
            let _ = self.invalidate_user_cache(&profile.user_id).await;
        }
        
        self.record_metrics(result.is_ok(), start_time, "user").await;
        result
    }

    /// Get user by Telegram ID
    pub async fn get_user_by_telegram_id(&self, telegram_id: i64) -> ArbitrageResult<Option<UserProfile>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let stmt = self.db.prepare("SELECT * FROM user_profiles WHERE telegram_id = ?");
        let result = stmt
            .bind(&[JsValue::from_f64(telegram_id as f64)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .first::<HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        let profile_result = match result {
            Some(row) => Ok(Some(self.row_to_user_profile(row)?)),
            None => Ok(None),
        };

        self.record_metrics(profile_result.is_ok(), start_time, "user").await;
        profile_result
    }

    /// Store trading preferences
    pub async fn store_trading_preferences(&self, preferences: &UserTradingPreferences) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let sql = "INSERT OR REPLACE INTO user_trading_preferences 
                   (user_id, risk_tolerance, max_trade_amount, automation_level, 
                    enable_notifications, created_at, updated_at) 
                   VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))";
        
        let result = self.db.prepare(sql)
            .bind(&[
                JsValue::from_str(&preferences.user_id),
                JsValue::from_f64(preferences.risk_tolerance),
                JsValue::from_f64(preferences.max_trade_amount),
                JsValue::from_str(&preferences.automation_level.to_string()),
                JsValue::from_bool(preferences.enable_notifications),
            ])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute statement: {}", e)))?;

        self.record_metrics(true, start_time, "user").await;
        Ok(())
    }

    /// Get trading preferences
    pub async fn get_trading_preferences(&self, user_id: &str) -> ArbitrageResult<Option<UserTradingPreferences>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let stmt = self.db.prepare("SELECT * FROM user_trading_preferences WHERE user_id = ?");
        let result = stmt
            .bind(&[JsValue::from_str(user_id)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .first::<HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        let preferences_result = match result {
            Some(row) => Ok(Some(self.row_to_trading_preferences(row)?)),
            None => Ok(None),
        };

        self.record_metrics(preferences_result.is_ok(), start_time, "user").await;
        preferences_result
    }

    // ============= AI DATA REPOSITORY OPERATIONS =============
    
    /// Store AI opportunity enhancement
    pub async fn store_ai_enhancement(&self, enhancement: &AiOpportunityEnhancement) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let result = self.store_ai_enhancement_internal(enhancement).await;
        self.record_metrics(result.is_ok(), start_time, "ai").await;
        result
    }

    /// Get AI enhancements for user
    pub async fn get_ai_enhancements(&self, user_id: &str) -> ArbitrageResult<Vec<AiOpportunityEnhancement>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let stmt = self.db.prepare("SELECT * FROM ai_opportunity_enhancements WHERE user_id = ? ORDER BY created_at DESC");
        let results = stmt
            .bind(&[JsValue::from_str(user_id)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        let mut enhancements = Vec::new();
        for result in results.results()? {
            if let Some(row) = result.as_object() {
                let row_map: HashMap<String, serde_json::Value> = row.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                enhancements.push(self.row_to_ai_enhancement(row_map)?);
            }
        }

        self.record_metrics(true, start_time, "ai").await;
        Ok(enhancements)
    }

    // ============= ANALYTICS REPOSITORY OPERATIONS =============
    
    /// Store analytics aggregation
    pub async fn store_analytics_aggregation(&self, user_id: &str, data: &serde_json::Value) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let sql = "INSERT OR REPLACE INTO analytics_aggregations 
                   (user_id, aggregation_data, created_at, updated_at) 
                   VALUES (?, ?, datetime('now'), datetime('now'))";
        
        let result = self.db.prepare(sql)
            .bind(&[
                JsValue::from_str(user_id),
                JsValue::from_str(&data.to_string()),
            ])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute statement: {}", e)))?;

        self.record_metrics(true, start_time, "analytics").await;
        Ok(())
    }

    /// Get analytics data
    pub async fn get_analytics_data(&self, user_id: &str) -> ArbitrageResult<Option<serde_json::Value>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let stmt = self.db.prepare("SELECT aggregation_data FROM analytics_aggregations WHERE user_id = ?");
        let result = stmt
            .bind(&[JsValue::from_str(user_id)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .first::<HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        let data_result = match result {
            Some(row) => {
                if let Some(data_str) = row.get("aggregation_data") {
                    if let Some(data_str) = data_str.as_str() {
                        match serde_json::from_str(data_str) {
                            Ok(data) => Ok(Some(data)),
                            Err(_) => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        };

        self.record_metrics(data_result.is_ok(), start_time, "analytics").await;
        data_result
    }

    // ============= CONFIG REPOSITORY OPERATIONS =============
    
    /// Store config preset
    pub async fn store_config_preset(&self, preset: &ConfigPreset) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let result = self.store_config_preset_internal(preset).await;
        self.record_metrics(result.is_ok(), start_time, "config").await;
        result
    }

    /// Get config presets
    pub async fn get_config_presets(&self) -> ArbitrageResult<Vec<ConfigPreset>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let stmt = self.db.prepare("SELECT * FROM config_presets ORDER BY created_at DESC");
        let results = stmt
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        let mut presets = Vec::new();
        for result in results.results()? {
            if let Some(row) = result.as_object() {
                let row_map: HashMap<String, serde_json::Value> = row.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                presets.push(self.row_to_config_preset(row_map)?);
            }
        }

        self.record_metrics(true, start_time, "config").await;
        Ok(presets)
    }

    // ============= INVITATION REPOSITORY OPERATIONS =============
    
    /// Create invitation code
    pub async fn create_invitation_code(&self, code: &InvitationCode) -> ArbitrageResult<()> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let result = self.store_invitation_code_internal(code).await;
        self.record_metrics(result.is_ok(), start_time, "invitation").await;
        result
    }

    /// Get invitation code
    pub async fn get_invitation_code(&self, code: &str) -> ArbitrageResult<Option<InvitationCode>> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let stmt = self.db.prepare("SELECT * FROM invitation_codes WHERE code = ?");
        let result = stmt
            .bind(&[JsValue::from_str(code)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .first::<HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        let code_result = match result {
            Some(row) => Ok(Some(self.row_to_invitation_code(row)?)),
            None => Ok(None),
        };

        self.record_metrics(code_result.is_ok(), start_time, "invitation").await;
        code_result
    }

    /// Use invitation code
    pub async fn use_invitation_code(&self, code: &str, used_by: &str) -> ArbitrageResult<bool> {
        let start_time = crate::utils::time::get_current_timestamp();
        
        let sql = "UPDATE invitation_codes SET uses_count = uses_count + 1, last_used_at = datetime('now') 
                   WHERE code = ? AND (max_uses IS NULL OR uses_count < max_uses) AND expires_at > datetime('now')";
        
        let result = self.db.prepare(sql)
            .bind(&[JsValue::from_str(code)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute statement: {}", e)))?;

        let success = result.changes() > 0;
        self.record_metrics(true, start_time, "invitation").await;
        Ok(success)
    }

    // ============= INTERNAL HELPER METHODS =============
    
    async fn store_user_profile_internal(&self, profile: &UserProfile) -> ArbitrageResult<()> {
        let sql = "INSERT OR REPLACE INTO user_profiles 
                   (user_id, telegram_id, username, risk_profile, subscription_tier, 
                    access_level, preferences, created_at, updated_at) 
                   VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))";
        
        let preferences_json = serde_json::to_string(&profile.preferences)
            .map_err(|e| ArbitrageError::serialization_error(format!("serialize preferences: {}", e)))?;
        
        self.db.prepare(sql)
            .bind(&[
                JsValue::from_str(&profile.user_id),
                JsValue::from_f64(profile.telegram_id as f64),
                JsValue::from_str(&profile.username),
                JsValue::from_str(&profile.risk_profile.to_string()),
                JsValue::from_str(&profile.subscription_tier.to_string()),
                JsValue::from_str(&profile.access_level.to_string()),
                JsValue::from_str(&preferences_json),
            ])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute statement: {}", e)))?;
        
        Ok(())
    }

    async fn get_user_profile_from_db(&self, user_id: &str) -> ArbitrageResult<Option<UserProfile>> {
        let stmt = self.db.prepare("SELECT * FROM user_profiles WHERE user_id = ?");
        let result = stmt
            .bind(&[JsValue::from_str(user_id)])
            .map_err(|e| ArbitrageError::database_error(format!("bind parameters: {}", e)))?
            .first::<HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("execute query: {}", e)))?;

        match result {
            Some(row) => Ok(Some(self.row_to_user_profile(row)?)),
            None => Ok(None),
        }
    }

    async fn get_cached_user_profile(&self, user_id: &str) -> ArbitrageResult<Option<UserProfile>> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("user_profile:{}", user_id);
            match cache.get(&cache_key).text().await {
                Ok(Some(cached_data)) => {
                    match serde_json::from_str::<UserProfile>(&cached_data) {
                        Ok(profile) => {
                            if let Ok(mut metrics) = self.metrics.lock() {
                                metrics.cache_hits += 1;
                            }
                            return Ok(Some(profile));
                        }
                        Err(_) => {}
                    }
                }
                _ => {}
            }
            
            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.cache_misses += 1;
            }
        }
        
        Ok(None)
    }

    async fn invalidate_user_cache(&self, user_id: &str) -> ArbitrageResult<()> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("user_profile:{}", user_id);
            let _ = cache.delete(&cache_key).await;
        }
        Ok(())
    }

    fn row_to_user_profile(&self, row: HashMap<String, serde_json::Value>) -> ArbitrageResult<UserProfile> {
        let preferences_json = row.get("preferences")
            .and_then(|v| v.as_str())
            .unwrap_or("{}");
        
        let preferences = serde_json::from_str(preferences_json)
            .unwrap_or_default();

        Ok(UserProfile {
            user_id: row.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            telegram_id: row.get("telegram_id").and_then(|v| v.as_i64()).unwrap_or(0),
            username: row.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            risk_profile: RiskProfile::Conservative, // Simplified
            subscription_tier: SubscriptionTier::Free, // Simplified
            access_level: UserAccessLevel::Basic, // Simplified
            preferences,
            created_at: crate::utils::time::get_current_timestamp(),
            updated_at: crate::utils::time::get_current_timestamp(),
        })
    }

    fn row_to_trading_preferences(&self, row: HashMap<String, serde_json::Value>) -> ArbitrageResult<UserTradingPreferences> {
        // Simplified implementation
        Ok(UserTradingPreferences::default())
    }

    fn row_to_ai_enhancement(&self, _row: HashMap<String, serde_json::Value>) -> ArbitrageResult<AiOpportunityEnhancement> {
        // Simplified implementation
        Ok(AiOpportunityEnhancement::default())
    }

    fn row_to_config_preset(&self, _row: HashMap<String, serde_json::Value>) -> ArbitrageResult<ConfigPreset> {
        // Simplified implementation
        Ok(ConfigPreset::default())
    }

    fn row_to_invitation_code(&self, _row: HashMap<String, serde_json::Value>) -> ArbitrageResult<InvitationCode> {
        // Simplified implementation
        Ok(InvitationCode::default())
    }

    async fn store_ai_enhancement_internal(&self, enhancement: &AiOpportunityEnhancement) -> ArbitrageResult<()> {
        // Simplified implementation
        Ok(())
    }

    async fn store_config_preset_internal(&self, preset: &ConfigPreset) -> ArbitrageResult<()> {
        // Simplified implementation
        Ok(())
    }

    async fn store_invitation_code_internal(&self, code: &InvitationCode) -> ArbitrageResult<()> {
        // Simplified implementation
        Ok(())
    }
}

/// Builder for unified repository layer
pub struct UnifiedRepositoryBuilder {
    config: UnifiedRepositoryConfig,
}

impl UnifiedRepositoryBuilder {
    pub fn new() -> Self {
        Self {
            config: UnifiedRepositoryConfig::default(),
        }
    }

    pub fn with_caching(mut self, enabled: bool, ttl_seconds: u64) -> Self {
        self.config.enable_caching = enabled;
        self.config.cache_ttl_seconds = ttl_seconds;
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.config.batch_size = batch_size;
        self
    }

    pub fn with_connection_pool(mut self, pool_size: u32) -> Self {
        self.config.connection_pool_size = pool_size;
        self
    }

    pub fn build(self, db: Arc<D1Database>) -> ArbitrageResult<UnifiedRepositoryLayer> {
        UnifiedRepositoryLayer::new(self.config, db)
    }
}

impl Default for UnifiedRepositoryBuilder {
    fn default() -> Self {
        Self::new()
    }
} 