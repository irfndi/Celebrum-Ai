//! Comprehensive RBAC Service
//!
//! Unified service that integrates all RBAC components including role management,
//! API access control, trading configuration, arbitrage opportunities, and technical strategies.

use crate::services::core::auth::{
    api_access::{ApiAccessManager, ExchangeApiConfig, UserApiAccess},
    arbitrage_opportunities::{ArbitrageOpportunityManager, OpportunityFilter},
    rbac_config::RBACConfigManager,
    technical_strategies::{StrategyComplexity, TechnicalStrategyManager},
    trading_config::{TradeExecutionRequest, TradingConfigManager, TradingSessionConfig},
};
use crate::types::{SubscriptionTier, UserAccessLevel};
use crate::utils::feature_flags::FeatureFlagManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::console_log;

/// Comprehensive user access summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccessSummary {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub subscription_tier: SubscriptionTier,
    pub permissions: Vec<String>,
    pub api_access: UserApiAccess,
    pub trading_config: Option<TradingSessionConfig>,
    pub opportunity_limits: OpportunityLimits,
    pub strategy_limits: StrategyLimits,
    pub feature_flags: HashMap<String, bool>,
    pub last_updated: u64,
}

/// Opportunity access limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityLimits {
    pub daily_limit: u32,
    pub daily_used: u32,
    pub hourly_limit: u32,
    pub hourly_used: u32,
    pub total_accessed: u64,
    pub success_rate: f64,
}

/// Strategy access limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyLimits {
    pub max_strategies: u32,
    pub created_strategies: u32,
    pub max_active_strategies: u32,
    pub active_strategies: u32,
    pub max_concurrent_backtests: u32,
    pub concurrent_backtests: u32,
}

/// RBAC operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RBACOperationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: u64,
}

/// Comprehensive RBAC Service
pub struct RBACService {
    rbac_config_manager: RBACConfigManager,
    api_access_manager: ApiAccessManager,
    trading_config_manager: TradingConfigManager,
    arbitrage_opportunity_manager: ArbitrageOpportunityManager,
    technical_strategy_manager: TechnicalStrategyManager,
    feature_flag_manager: FeatureFlagManager,
    user_sessions: HashMap<String, UserAccessSummary>,
}

impl RBACService {
    /// Create new RBAC service
    pub fn new() -> Self {
        console_log!("🔐 Initializing Comprehensive RBAC Service...");

        Self {
            rbac_config_manager: RBACConfigManager::new(),
            api_access_manager: ApiAccessManager::new(),
            trading_config_manager: TradingConfigManager::new(),
            arbitrage_opportunity_manager: ArbitrageOpportunityManager::new(),
            technical_strategy_manager: TechnicalStrategyManager::new(),
            feature_flag_manager: FeatureFlagManager::default(),
            user_sessions: HashMap::new(),
        }
    }

    /// Register user with comprehensive access setup
    pub fn register_user(
        &mut self,
        user_id: &str,
        role: UserAccessLevel,
        subscription_tier: SubscriptionTier,
    ) -> RBACOperationResult {
        console_log!(
            "👤 Registering user '{}' with role: {:?}, tier: {:?}",
            user_id,
            role,
            subscription_tier
        );

        // Register user in all subsystems
        let user_api_access = UserApiAccess {
            user_id: user_id.to_string(),
            role: role.clone(),
            exchange_apis: Vec::new(),
            ai_apis: Vec::new(),
            last_updated: Utc::now().timestamp_millis() as u64,
        };
        self.api_access_manager
            .register_user_api_access(user_api_access);

        // Note: Other managers may need similar updates based on their actual method signatures

        // Create comprehensive access summary
        let access_summary = self.create_user_access_summary(user_id, role, subscription_tier);

        // Store user session
        self.user_sessions
            .insert(user_id.to_string(), access_summary.clone());

        RBACOperationResult {
            success: true,
            message: format!("User '{}' registered successfully", user_id),
            data: Some(serde_json::to_value(access_summary).unwrap_or_default()),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }

    /// Create comprehensive user access summary
    fn create_user_access_summary(
        &self,
        user_id: &str,
        role: UserAccessLevel,
        subscription_tier: SubscriptionTier,
    ) -> UserAccessSummary {
        // Get permissions
        let permissions = self
            .rbac_config_manager
            .config()
            .get_role_permissions(&role);

        // Get API access
        let default_api_access = UserApiAccess {
            user_id: user_id.to_string(),
            role: role.clone(),
            exchange_apis: Vec::new(),
            ai_apis: Vec::new(),
            last_updated: Utc::now().timestamp_millis() as u64,
        };
        let api_access = self
            .api_access_manager
            .get_user_api_access(user_id)
            .unwrap_or(&default_api_access);

        // Get trading configuration - convert to session config if needed
        let trading_config = None; // TODO: Convert TradingConfig to TradingSessionConfig or get from sessions

        // Get opportunity limits
        let opportunity_stats = self
            .arbitrage_opportunity_manager
            .get_user_stats(user_id)
            .unwrap_or_else(|_| {
                crate::services::core::auth::arbitrage_opportunities::UserOpportunityStats {
                    user_id: user_id.to_string(),
                    role: role.clone(),
                    subscription_tier: subscription_tier.clone(),
                    daily_limit: 0,
                    daily_used: 0,
                    hourly_limit: 0,
                    hourly_used: 0,
                    total_accessed: 0,
                    successful_executions: 0,
                    failed_executions: 0,
                    success_rate: 0.0,
                    last_access: 0,
                }
            });

        let opportunity_limits = OpportunityLimits {
            daily_limit: opportunity_stats.daily_limit,
            daily_used: opportunity_stats.daily_used,
            hourly_limit: opportunity_stats.hourly_limit,
            hourly_used: opportunity_stats.hourly_used,
            total_accessed: opportunity_stats.total_accessed,
            success_rate: opportunity_stats.success_rate,
        };

        // Get strategy limits
        let strategy_stats = self
            .technical_strategy_manager
            .get_user_stats(user_id)
            .unwrap_or_else(|_| {
                crate::services::core::auth::technical_strategies::UserStrategyStats {
                    user_id: user_id.to_string(),
                    role: role.clone(),
                    subscription_tier: subscription_tier.clone(),
                    max_strategies: 0,
                    created_strategies: 0,
                    max_active_strategies: 0,
                    active_strategies: 0,
                    total_backtests: 0,
                    max_concurrent_backtests: 0,
                    concurrent_backtests: 0,
                    average_performance: 0.0,
                    last_strategy_creation: 0,
                    last_backtest: 0,
                }
            });

        let strategy_limits = StrategyLimits {
            max_strategies: strategy_stats.max_strategies,
            created_strategies: strategy_stats.created_strategies,
            max_active_strategies: strategy_stats.max_active_strategies,
            active_strategies: strategy_stats.active_strategies,
            max_concurrent_backtests: strategy_stats.max_concurrent_backtests,
            concurrent_backtests: strategy_stats.concurrent_backtests,
        };

        // Get relevant feature flags
        let mut feature_flags = HashMap::new();
        feature_flags.insert(
            "rbac.enabled".to_string(),
            self.feature_flag_manager.is_enabled("rbac.enabled"),
        );
        feature_flags.insert(
            "api_access.enabled".to_string(),
            self.feature_flag_manager.is_enabled("api_access.enabled"),
        );
        feature_flags.insert(
            "trading.enabled".to_string(),
            self.feature_flag_manager.is_enabled("trading.enabled"),
        );
        feature_flags.insert(
            "opportunity_engine.enabled".to_string(),
            self.feature_flag_manager
                .is_enabled("opportunity_engine.enabled"),
        );
        feature_flags.insert(
            "technical_strategies.enabled".to_string(),
            self.feature_flag_manager
                .is_enabled("technical_strategies.enabled"),
        );

        UserAccessSummary {
            user_id: user_id.to_string(),
            role,
            subscription_tier,
            permissions,
            api_access: api_access.clone(),
            trading_config,
            opportunity_limits,
            strategy_limits,
            feature_flags,
            last_updated: Utc::now().timestamp_millis() as u64,
        }
    }

    /// Check if user has specific permission
    pub fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(user_session) = self.user_sessions.get(user_id) {
            return user_session.permissions.contains(&permission.to_string());
        }
        false
    }

    /// Get user access summary
    pub fn get_user_access_summary(&mut self, user_id: &str) -> Result<UserAccessSummary, String> {
        if let Some(mut summary) = self.user_sessions.get(user_id).cloned() {
            // Refresh the summary with latest data
            summary = self.create_user_access_summary(
                user_id,
                summary.role.clone(),
                summary.subscription_tier.clone(),
            );
            self.user_sessions
                .insert(user_id.to_string(), summary.clone());
            Ok(summary)
        } else {
            Err("User session not found".to_string())
        }
    }

    /// Update user role and subscription
    pub fn update_user_access(
        &mut self,
        user_id: &str,
        new_role: UserAccessLevel,
        new_subscription_tier: SubscriptionTier,
    ) -> RBACOperationResult {
        console_log!(
            "🔄 Updating user '{}' access: role={:?}, tier={:?}",
            user_id,
            new_role,
            new_subscription_tier
        );

        // Update in all subsystems
        if let Err(e) = self.arbitrage_opportunity_manager.update_user_role(
            user_id,
            new_role.clone(),
            new_subscription_tier.clone(),
        ) {
            return RBACOperationResult {
                success: false,
                message: format!("Failed to update opportunity access: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            };
        }

        // Update user session
        let updated_summary =
            self.create_user_access_summary(user_id, new_role, new_subscription_tier);
        self.user_sessions
            .insert(user_id.to_string(), updated_summary.clone());

        RBACOperationResult {
            success: true,
            message: format!("User '{}' access updated successfully", user_id),
            data: Some(serde_json::to_value(updated_summary).unwrap_or_default()),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }

    /// Add API access for user
    pub fn add_user_api_access(
        &mut self,
        user_id: &str,
        _api_type: &str,
        api_name: &str,
        api_key: &str,
        api_secret: Option<&str>,
    ) -> RBACOperationResult {
        let exchange_config = ExchangeApiConfig {
            api_key: api_key.to_string(),
            secret_key: api_secret.unwrap_or("").to_string(),
            exchange_name: api_name.to_string(),
            is_testnet: false,
            rate_limit_per_minute: 60, // Default rate limit
            enabled: true,
        };

        match self
            .api_access_manager
            .add_exchange_api(user_id, exchange_config)
        {
            Ok(_) => {
                // Update user session
                if let Ok(summary) = self.get_user_access_summary(user_id) {
                    self.user_sessions.insert(user_id.to_string(), summary);
                }

                RBACOperationResult {
                    success: true,
                    message: format!("API access added for user: {}", user_id),
                    data: None,
                    timestamp: Utc::now().timestamp_millis() as u64,
                }
            }
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to add API access: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Create trading session for user
    pub fn create_trading_session(
        &mut self,
        user_id: &str,
        _session_name: &str,
    ) -> RBACOperationResult {
        // Get user role first
        let user_role = self
            .get_user_access_summary(user_id)
            .map(|summary| summary.role)
            .unwrap_or(UserAccessLevel::Free);

        match self
            .trading_config_manager
            .create_trading_session(user_id, user_role)
        {
            Ok(session_id) => RBACOperationResult {
                success: true,
                message: format!("Trading session created: {}", session_id),
                data: Some(serde_json::json!({ "session_id": session_id })),
                timestamp: Utc::now().timestamp_millis() as u64,
            },
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to create trading session: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Get arbitrage opportunities for user
    pub fn get_arbitrage_opportunities(
        &mut self,
        user_id: &str,
        filter: Option<OpportunityFilter>,
    ) -> RBACOperationResult {
        match self
            .arbitrage_opportunity_manager
            .get_opportunities_for_user(user_id, filter)
        {
            Ok(opportunities) => RBACOperationResult {
                success: true,
                message: format!("Retrieved {} opportunities", opportunities.len()),
                data: Some(serde_json::to_value(opportunities).unwrap_or_default()),
                timestamp: Utc::now().timestamp_millis() as u64,
            },
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to get opportunities: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Create technical strategy from YAML
    pub fn create_technical_strategy(
        &mut self,
        user_id: &str,
        yaml_content: &str,
        strategy_name: &str,
        description: &str,
    ) -> RBACOperationResult {
        match self.technical_strategy_manager.create_strategy_from_yaml(
            user_id,
            yaml_content,
            strategy_name,
            description,
        ) {
            Ok(strategy_id) => RBACOperationResult {
                success: true,
                message: format!("Strategy created: {}", strategy_id),
                data: Some(serde_json::json!({ "strategy_id": strategy_id })),
                timestamp: Utc::now().timestamp_millis() as u64,
            },
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to create strategy: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Get user technical strategies
    pub fn get_user_strategies(&self, user_id: &str) -> RBACOperationResult {
        match self.technical_strategy_manager.get_user_strategies(user_id) {
            Ok(strategies) => RBACOperationResult {
                success: true,
                message: format!("Retrieved {} strategies", strategies.len()),
                data: Some(serde_json::to_value(strategies).unwrap_or_default()),
                timestamp: Utc::now().timestamp_millis() as u64,
            },
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to get strategies: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Get marketplace strategies
    pub fn get_marketplace_strategies(
        &self,
        user_id: &str,
        complexity_filter: Option<StrategyComplexity>,
    ) -> RBACOperationResult {
        match self
            .technical_strategy_manager
            .get_marketplace_strategies(user_id, complexity_filter)
        {
            Ok(strategies) => RBACOperationResult {
                success: true,
                message: format!("Retrieved {} marketplace strategies", strategies.len()),
                data: Some(serde_json::to_value(strategies).unwrap_or_default()),
                timestamp: Utc::now().timestamp_millis() as u64,
            },
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to get marketplace strategies: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Execute trade with RBAC validation
    pub fn execute_trade(
        &mut self,
        user_id: &str,
        trade_request: TradeExecutionRequest,
    ) -> RBACOperationResult {
        match self
            .trading_config_manager
            .execute_trade(user_id, &trade_request)
        {
            Ok(trade_id) => RBACOperationResult {
                success: true,
                message: format!("Trade executed: {}", trade_id),
                data: Some(serde_json::json!({ "trade_id": trade_id })),
                timestamp: Utc::now().timestamp_millis() as u64,
            },
            Err(e) => RBACOperationResult {
                success: false,
                message: format!("Failed to execute trade: {}", e),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            },
        }
    }

    /// Get comprehensive system health
    pub fn get_system_health(&self) -> RBACOperationResult {
        let mut health_data = serde_json::Map::new();

        // Check feature flags
        health_data.insert(
            "rbac_enabled".to_string(),
            serde_json::Value::Bool(self.feature_flag_manager.is_enabled("rbac.enabled")),
        );
        health_data.insert(
            "api_access_enabled".to_string(),
            serde_json::Value::Bool(self.feature_flag_manager.is_enabled("api_access.enabled")),
        );
        health_data.insert(
            "trading_enabled".to_string(),
            serde_json::Value::Bool(self.feature_flag_manager.is_enabled("trading.enabled")),
        );
        health_data.insert(
            "opportunities_enabled".to_string(),
            serde_json::Value::Bool(
                self.feature_flag_manager
                    .is_enabled("opportunity_engine.enabled"),
            ),
        );
        health_data.insert(
            "strategies_enabled".to_string(),
            serde_json::Value::Bool(
                self.feature_flag_manager
                    .is_enabled("technical_strategies.enabled"),
            ),
        );

        // System statistics
        health_data.insert(
            "active_users".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.user_sessions.len())),
        );
        health_data.insert(
            "timestamp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                Utc::now().timestamp_millis() as u64
            )),
        );

        RBACOperationResult {
            success: true,
            message: "System health retrieved successfully".to_string(),
            data: Some(serde_json::Value::Object(health_data)),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }

    /// Remove user session
    pub fn remove_user_session(&mut self, user_id: &str) -> RBACOperationResult {
        if self.user_sessions.remove(user_id).is_some() {
            console_log!("🗑️ Removed user session: {}", user_id);
            RBACOperationResult {
                success: true,
                message: format!("User session removed: {}", user_id),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            }
        } else {
            RBACOperationResult {
                success: false,
                message: format!("User session not found: {}", user_id),
                data: None,
                timestamp: Utc::now().timestamp_millis() as u64,
            }
        }
    }

    /// Get active user count
    pub fn get_active_user_count(&self) -> usize {
        self.user_sessions.len()
    }

    /// Cleanup expired sessions (placeholder for future implementation)
    pub fn cleanup_expired_sessions(&mut self) {
        // In a real implementation, this would check session expiration times
        // and remove expired sessions
        console_log!("🧹 Session cleanup completed");
    }
}

impl Default for RBACService {
    fn default() -> Self {
        Self::new()
    }
}
