//! API Access Management System
//!
//! Manages separate Exchange API and AI API access controls
//! with user role-based permissions and feature flag integration.

use crate::services::core::auth::rbac_config::RBACConfigManager;
use crate::types::UserAccessLevel;
use crate::utils::feature_flags::FeatureFlagManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::console_log;

/// Exchange API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeApiConfig {
    pub api_key: String,
    pub secret_key: String,
    pub exchange_name: String,
    pub is_testnet: bool,
    pub rate_limit_per_minute: u32,
    pub enabled: bool,
}

/// AI API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiApiConfig {
    pub provider: String, // "openai", "anthropic", "perplexity", etc.
    pub api_key: String,
    pub model: String,
    pub rate_limit_per_hour: u32,
    pub enabled: bool,
}

/// User API access status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserApiAccess {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub exchange_apis: Vec<ExchangeApiConfig>,
    pub ai_apis: Vec<AiApiConfig>,
    pub last_updated: u64, // timestamp
}

/// API access validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAccessValidation {
    pub is_valid: bool,
    pub exchange_api_count: u32,
    pub exchange_api_required: u32,
    pub ai_api_enabled: bool,
    pub ai_api_required: bool,
    pub missing_requirements: Vec<String>,
    pub recommendations: Vec<String>,
}

/// API Access Manager
pub struct ApiAccessManager {
    rbac_manager: RBACConfigManager,
    user_api_access: HashMap<String, UserApiAccess>,
    feature_flag_manager: Option<FeatureFlagManager>,
}

impl ApiAccessManager {
    /// Create new API access manager
    pub fn new() -> Self {
        console_log!("🔑 Initializing API Access Manager...");
        
        Self {
            rbac_manager: RBACConfigManager::new(),
            user_api_access: HashMap::new(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
        }
    }

    /// Create with custom RBAC manager
    pub fn with_rbac_manager(rbac_manager: RBACConfigManager) -> Self {
        console_log!("🔑 Initializing API Access Manager with custom RBAC...");
        
        Self {
            rbac_manager,
            user_api_access: HashMap::new(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
        }
    }

    /// Register user API access
    pub fn register_user_api_access(&mut self, user_api_access: UserApiAccess) {
        console_log!(
            "📝 Registering API access for user: {} with role: {:?}",
            user_api_access.user_id,
            user_api_access.role
        );
        
        self.user_api_access.insert(
            user_api_access.user_id.clone(),
            user_api_access,
        );
    }

    /// Add exchange API for user
    pub fn add_exchange_api(
        &mut self,
        user_id: &str,
        exchange_config: ExchangeApiConfig,
    ) -> Result<(), String> {
        // Check if API access management is enabled
        if let Some(ffm) = &self.feature_flag_manager {
            if !ffm.is_enabled("rbac.api_access_management") {
                return Err("API access management is disabled".to_string());
            }
        }

        let user_access = self.user_api_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        // Get role-based API access config
        let api_config = self.rbac_manager.config().get_api_access(&user_access.role);
        
        // Check if user can add more exchange APIs
        if user_access.exchange_apis.len() >= api_config.exchange_api_recommended as usize {
            return Err(format!(
                "Maximum exchange APIs ({}) reached for role: {:?}",
                api_config.exchange_api_recommended,
                user_access.role
            ));
        }

        // Validate exchange API doesn't already exist
        if user_access.exchange_apis.iter().any(|api| {
            api.exchange_name == exchange_config.exchange_name
                && api.api_key == exchange_config.api_key
        }) {
            return Err("Exchange API already exists".to_string());
        }

        user_access.exchange_apis.push(exchange_config.clone());
        user_access.last_updated = Utc::now().timestamp_millis() as u64;
        
        console_log!(
            "✅ Added exchange API '{}' for user: {}",
            exchange_config.exchange_name,
            user_id
        );
        
        Ok(())
    }

    /// Add AI API for user
    pub fn add_ai_api(
        &mut self,
        user_id: &str,
        ai_config: AiApiConfig,
    ) -> Result<(), String> {
        // Check if API access management is enabled
        if let Some(ffm) = &self.feature_flag_manager {
            if !ffm.is_enabled("rbac.api_access_management") {
                return Err("API access management is disabled".to_string());
            }
        }

        let user_access = self.user_api_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        // Get role-based API access config
        let api_config = self.rbac_manager.config().get_api_access(&user_access.role);
        
        // Check if AI API is enabled for this role
        if !api_config.ai_api_enabled {
            return Err(format!(
                "AI API access not enabled for role: {:?}",
                user_access.role
            ));
        }

        // Validate AI API doesn't already exist
        if user_access.ai_apis.iter().any(|api| {
            api.provider == ai_config.provider && api.api_key == ai_config.api_key
        }) {
            return Err("AI API already exists".to_string());
        }

        user_access.ai_apis.push(ai_config.clone());
        user_access.last_updated = Utc::now().timestamp_millis() as u64;
        
        console_log!(
            "✅ Added AI API '{}' for user: {}",
            ai_config.provider,
            user_id
        );
        
        Ok(())
    }

    /// Remove exchange API for user
    pub fn remove_exchange_api(
        &mut self,
        user_id: &str,
        exchange_name: &str,
        api_key: &str,
    ) -> Result<(), String> {
        let user_access = self.user_api_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let initial_count = user_access.exchange_apis.len();
        user_access.exchange_apis.retain(|api| {
            !(api.exchange_name == exchange_name && api.api_key == api_key)
        });

        if user_access.exchange_apis.len() == initial_count {
            return Err("Exchange API not found".to_string());
        }

        user_access.last_updated = Utc::now().timestamp_millis() as u64;
        
        console_log!(
            "🗑️ Removed exchange API '{}' for user: {}",
            exchange_name,
            user_id
        );
        
        Ok(())
    }

    /// Remove AI API for user
    pub fn remove_ai_api(
        &mut self,
        user_id: &str,
        provider: &str,
        api_key: &str,
    ) -> Result<(), String> {
        let user_access = self.user_api_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let initial_count = user_access.ai_apis.len();
        user_access.ai_apis.retain(|api| {
            !(api.provider == provider && api.api_key == api_key)
        });

        if user_access.ai_apis.len() == initial_count {
            return Err("AI API not found".to_string());
        }

        user_access.last_updated = Utc::now().timestamp_millis() as u64;
        
        console_log!(
            "🗑️ Removed AI API '{}' for user: {}",
            provider,
            user_id
        );
        
        Ok(())
    }

    /// Validate user API access against role requirements
    pub fn validate_user_api_access(&self, user_id: &str) -> Result<ApiAccessValidation, String> {
        let user_access = self.user_api_access
            .get(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let api_config = self.rbac_manager.config().get_api_access(&user_access.role);
        
        let exchange_api_count = user_access.exchange_apis.len() as u32;
        let has_ai_api = !user_access.ai_apis.is_empty();
        
        let mut missing_requirements = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check exchange API requirements
        if exchange_api_count < api_config.exchange_api_required {
            missing_requirements.push(format!(
                "Need {} more exchange API(s) (current: {}, required: {})",
                api_config.exchange_api_required - exchange_api_count,
                exchange_api_count,
                api_config.exchange_api_required
            ));
        }
        
        if exchange_api_count < api_config.exchange_api_recommended {
            recommendations.push(format!(
                "Consider adding {} more exchange API(s) for optimal performance (current: {}, recommended: {})",
                api_config.exchange_api_recommended - exchange_api_count,
                exchange_api_count,
                api_config.exchange_api_recommended
            ));
        }
        
        // Check AI API requirements
        if api_config.ai_api_required && !has_ai_api {
            missing_requirements.push("AI API is required for this role".to_string());
        }
        
        if api_config.ai_api_enabled && !has_ai_api {
            recommendations.push("Consider adding AI API for enhanced features".to_string());
        }
        
        let is_valid = missing_requirements.is_empty();
        
        Ok(ApiAccessValidation {
            is_valid,
            exchange_api_count,
            exchange_api_required: api_config.exchange_api_required,
            ai_api_enabled: api_config.ai_api_enabled,
            ai_api_required: api_config.ai_api_required,
            missing_requirements,
            recommendations,
        })
    }

    /// Get user API access
    pub fn get_user_api_access(&self, user_id: &str) -> Option<&UserApiAccess> {
        self.user_api_access.get(user_id)
    }

    /// Check if user can use exchange APIs
    pub fn can_use_exchange_apis(&self, user_id: &str) -> bool {
        if let Ok(validation) = self.validate_user_api_access(user_id) {
            validation.exchange_api_count > 0
        } else {
            false
        }
    }

    /// Check if user can use AI APIs
    pub fn can_use_ai_apis(&self, user_id: &str) -> bool {
        if let Some(user_access) = self.user_api_access.get(user_id) {
            let api_config = self.rbac_manager.config().get_api_access(&user_access.role);
            api_config.ai_api_enabled && !user_access.ai_apis.is_empty()
        } else {
            false
        }
    }

    /// Get active exchange APIs for user
    pub fn get_active_exchange_apis(&self, user_id: &str) -> Vec<&ExchangeApiConfig> {
        if let Some(user_access) = self.user_api_access.get(user_id) {
            user_access.exchange_apis
                .iter()
                .filter(|api| api.enabled)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get active AI APIs for user
    pub fn get_active_ai_apis(&self, user_id: &str) -> Vec<&AiApiConfig> {
        if let Some(user_access) = self.user_api_access.get(user_id) {
            user_access.ai_apis
                .iter()
                .filter(|api| api.enabled)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update user role and revalidate API access
    pub fn update_user_role(&mut self, user_id: &str, new_role: UserAccessLevel) -> Result<(), String> {
        let user_access = self.user_api_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let old_role = user_access.role.clone();
        user_access.role = new_role.clone();
        user_access.last_updated = Utc::now().timestamp_millis() as u64;
        
        console_log!(
            "🔄 Updated user role from {:?} to {:?} for user: {}",
            old_role,
            new_role,
            user_id
        );
        
        // Validate new role requirements
        match self.validate_user_api_access(user_id) {
            Ok(validation) => {
                if !validation.is_valid {
                    console_log!(
                        "⚠️ User {} API access validation failed after role update: {:?}",
                        user_id,
                        validation.missing_requirements
                    );
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to validate API access after role update: {}", e))
        }
    }

    /// Get comprehensive API access report for user
    pub fn get_user_api_report(&self, user_id: &str) -> Result<UserApiReport, String> {
        let user_access = self.user_api_access
            .get(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let validation = self.validate_user_api_access(user_id)?;
        let api_config = self.rbac_manager.config().get_api_access(&user_access.role);
        
        Ok(UserApiReport {
            user_id: user_id.to_string(),
            role: user_access.role.clone(),
            validation,
            api_config,
            exchange_apis_count: user_access.exchange_apis.len() as u32,
            ai_apis_count: user_access.ai_apis.len() as u32,
            last_updated: user_access.last_updated,
        })
    }
}

/// Comprehensive user API report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserApiReport {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub validation: ApiAccessValidation,
    pub api_config: ApiAccessConfig,
    pub exchange_apis_count: u32,
    pub ai_apis_count: u32,
    pub last_updated: u64,
}

impl Default for ApiAccessManager {
    fn default() -> Self {
        Self::new()
    }
}