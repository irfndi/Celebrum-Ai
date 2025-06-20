//! RBAC Configuration Module
//!
//! Comprehensive configuration for the simplified RBAC system with:
//! - Role-based permissions
//! - API access management
//! - Trading configuration limits
//! - Feature flag integration
//! - Group and channel access control

use crate::types::{SubscriptionTier, UserAccessLevel};
use crate::utils::feature_flags::FeatureFlagManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::console_log;

/// API access configuration separate from user roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAccessConfig {
    pub exchange_api_required: u32,     // Minimum exchange APIs required
    pub exchange_api_recommended: u32,  // Recommended exchange APIs
    pub ai_api_enabled: bool,          // AI API access enabled
    pub ai_api_required: bool,         // AI API required for features
}

/// Trading configuration limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub max_concurrent_trades: u32,
    pub max_leverage: f64,
    pub max_position_size_percent: f64, // % of portfolio
    pub stop_loss_required: bool,
    pub take_profit_recommended: bool,
    pub risk_management_required: bool,
}

/// Group and channel access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChannelAccess {
    pub basic_groups: Vec<String>,
    pub premium_groups: Vec<String>,
    pub ultra_groups: Vec<String>,
    pub admin_groups: Vec<String>,
    pub max_groups_per_tier: u32,
}

/// Comprehensive RBAC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RBACConfig {
    pub role_permissions: HashMap<String, Vec<String>>,
    pub api_access: HashMap<String, ApiAccessConfig>,
    pub trading_limits: HashMap<String, TradingConfig>,
    pub group_access: HashMap<String, GroupChannelAccess>,
    pub feature_flags: HashMap<String, bool>,
}

impl Default for RBACConfig {
    fn default() -> Self {
        let mut config = RBACConfig {
            role_permissions: HashMap::new(),
            api_access: HashMap::new(),
            trading_limits: HashMap::new(),
            group_access: HashMap::new(),
            feature_flags: HashMap::new(),
        };

        // Initialize default configurations
        config.init_role_permissions();
        config.init_api_access();
        config.init_trading_limits();
        config.init_group_access();
        config.init_feature_flags();

        config
    }
}

impl RBACConfig {
    /// Initialize role-based permissions
    fn init_role_permissions(&mut self) {
        // Free tier permissions
        self.role_permissions.insert(
            "free".to_string(),
            vec![
                "view_basic_opportunities".to_string(),
                "access_basic_groups".to_string(),
                "view_market_data".to_string(),
                "basic_portfolio_view".to_string(),
            ],
        );

        // Pro tier permissions
        self.role_permissions.insert(
            "pro".to_string(),
            vec![
                "view_basic_opportunities".to_string(),
                "view_enhanced_opportunities".to_string(),
                "access_basic_groups".to_string(),
                "access_pro_groups".to_string(),
                "manual_trading".to_string(),
                "basic_risk_management".to_string(),
                "enhanced_portfolio_view".to_string(),
                "technical_analysis_basic".to_string(),
            ],
        );

        // Ultra tier permissions
        self.role_permissions.insert(
            "ultra".to_string(),
            vec![
                "view_basic_opportunities".to_string(),
                "view_enhanced_opportunities".to_string(),
                "view_premium_opportunities".to_string(),
                "access_basic_groups".to_string(),
                "access_pro_groups".to_string(),
                "access_ultra_groups".to_string(),
                "manual_trading".to_string(),
                "auto_trading".to_string(),
                "advanced_risk_management".to_string(),
                "ai_analysis".to_string(),
                "ai_opportunities".to_string(),
                "premium_portfolio_view".to_string(),
                "technical_analysis_advanced".to_string(),
                "yaml_strategy_creation".to_string(),
                "strategy_backtesting".to_string(),
            ],
        );

        // Admin permissions
        self.role_permissions.insert(
            "admin".to_string(),
            vec![
                "all_user_permissions".to_string(),
                "user_management".to_string(),
                "group_management".to_string(),
                "system_monitoring".to_string(),
                "configuration_management".to_string(),
            ],
        );

        // SuperAdmin permissions
        self.role_permissions.insert(
            "superadmin".to_string(),
            vec![
                "all_permissions".to_string(),
                "system_administration".to_string(),
                "security_management".to_string(),
                "infrastructure_control".to_string(),
            ],
        );
    }

    /// Initialize API access configurations
    fn init_api_access(&mut self) {
        // Free tier API access
        self.api_access.insert(
            "free".to_string(),
            ApiAccessConfig {
                exchange_api_required: 0,
                exchange_api_recommended: 1,
                ai_api_enabled: false,
                ai_api_required: false,
            },
        );

        // Pro tier API access
        self.api_access.insert(
            "pro".to_string(),
            ApiAccessConfig {
                exchange_api_required: 1,
                exchange_api_recommended: 2,
                ai_api_enabled: false,
                ai_api_required: false,
            },
        );

        // Ultra tier API access
        self.api_access.insert(
            "ultra".to_string(),
            ApiAccessConfig {
                exchange_api_required: 2,
                exchange_api_recommended: 3,
                ai_api_enabled: true,
                ai_api_required: true,
            },
        );

        // Admin API access
        self.api_access.insert(
            "admin".to_string(),
            ApiAccessConfig {
                exchange_api_required: 0,
                exchange_api_recommended: 5,
                ai_api_enabled: true,
                ai_api_required: false,
            },
        );

        // SuperAdmin API access
        self.api_access.insert(
            "superadmin".to_string(),
            ApiAccessConfig {
                exchange_api_required: 0,
                exchange_api_recommended: 10,
                ai_api_enabled: true,
                ai_api_required: false,
            },
        );
    }

    /// Initialize trading configuration limits
    fn init_trading_limits(&mut self) {
        // Free tier trading limits
        self.trading_limits.insert(
            "free".to_string(),
            TradingConfig {
                max_concurrent_trades: 1,
                max_leverage: 2.0,
                max_position_size_percent: 10.0,
                stop_loss_required: true,
                take_profit_recommended: true,
                risk_management_required: true,
            },
        );

        // Pro tier trading limits
        self.trading_limits.insert(
            "pro".to_string(),
            TradingConfig {
                max_concurrent_trades: 3,
                max_leverage: 5.0,
                max_position_size_percent: 25.0,
                stop_loss_required: true,
                take_profit_recommended: true,
                risk_management_required: true,
            },
        );

        // Ultra tier trading limits
        self.trading_limits.insert(
            "ultra".to_string(),
            TradingConfig {
                max_concurrent_trades: 10,
                max_leverage: 20.0,
                max_position_size_percent: 50.0,
                stop_loss_required: false,
                take_profit_recommended: true,
                risk_management_required: false,
            },
        );

        // Admin trading limits
        self.trading_limits.insert(
            "admin".to_string(),
            TradingConfig {
                max_concurrent_trades: 50,
                max_leverage: 100.0,
                max_position_size_percent: 100.0,
                stop_loss_required: false,
                take_profit_recommended: false,
                risk_management_required: false,
            },
        );

        // SuperAdmin trading limits
        self.trading_limits.insert(
            "superadmin".to_string(),
            TradingConfig {
                max_concurrent_trades: u32::MAX,
                max_leverage: f64::MAX,
                max_position_size_percent: 100.0,
                stop_loss_required: false,
                take_profit_recommended: false,
                risk_management_required: false,
            },
        );
    }

    /// Initialize group and channel access
    fn init_group_access(&mut self) {
        // Free tier group access
        self.group_access.insert(
            "free".to_string(),
            GroupChannelAccess {
                basic_groups: vec!["general".to_string(), "announcements".to_string()],
                premium_groups: vec![],
                ultra_groups: vec![],
                admin_groups: vec![],
                max_groups_per_tier: 2,
            },
        );

        // Pro tier group access
        self.group_access.insert(
            "pro".to_string(),
            GroupChannelAccess {
                basic_groups: vec!["general".to_string(), "announcements".to_string()],
                premium_groups: vec!["pro_signals".to_string(), "pro_analysis".to_string()],
                ultra_groups: vec![],
                admin_groups: vec![],
                max_groups_per_tier: 5,
            },
        );

        // Ultra tier group access
        self.group_access.insert(
            "ultra".to_string(),
            GroupChannelAccess {
                basic_groups: vec!["general".to_string(), "announcements".to_string()],
                premium_groups: vec!["pro_signals".to_string(), "pro_analysis".to_string()],
                ultra_groups: vec![
                    "ultra_signals".to_string(),
                    "ultra_analysis".to_string(),
                    "ai_insights".to_string(),
                ],
                admin_groups: vec![],
                max_groups_per_tier: 10,
            },
        );

        // Admin group access
        self.group_access.insert(
            "admin".to_string(),
            GroupChannelAccess {
                basic_groups: vec!["general".to_string(), "announcements".to_string()],
                premium_groups: vec!["pro_signals".to_string(), "pro_analysis".to_string()],
                ultra_groups: vec![
                    "ultra_signals".to_string(),
                    "ultra_analysis".to_string(),
                    "ai_insights".to_string(),
                ],
                admin_groups: vec!["admin_panel".to_string(), "system_alerts".to_string()],
                max_groups_per_tier: 20,
            },
        );

        // SuperAdmin group access
        self.group_access.insert(
            "superadmin".to_string(),
            GroupChannelAccess {
                basic_groups: vec!["general".to_string(), "announcements".to_string()],
                premium_groups: vec!["pro_signals".to_string(), "pro_analysis".to_string()],
                ultra_groups: vec![
                    "ultra_signals".to_string(),
                    "ultra_analysis".to_string(),
                    "ai_insights".to_string(),
                ],
                admin_groups: vec![
                    "admin_panel".to_string(),
                    "system_alerts".to_string(),
                    "security_logs".to_string(),
                ],
                max_groups_per_tier: u32::MAX,
            },
        );
    }

    /// Initialize feature flags
    fn init_feature_flags(&mut self) {
        self.feature_flags.insert("rbac.simplified_roles".to_string(), true);
        self.feature_flags.insert("rbac.api_access_management".to_string(), true);
        self.feature_flags.insert("rbac.trading_limits".to_string(), true);
        self.feature_flags.insert("rbac.group_access_control".to_string(), true);
        self.feature_flags.insert("rbac.yaml_strategies".to_string(), true);
        self.feature_flags.insert("rbac.enhanced_logging".to_string(), true);
    }

    /// Get permissions for a specific role
    pub fn get_role_permissions(&self, role: &UserAccessLevel) -> Vec<String> {
        let role_key = match role {
            UserAccessLevel::Free => "free",
            UserAccessLevel::Pro => "pro",
            UserAccessLevel::Ultra => "ultra",
            UserAccessLevel::Admin => "admin",
            UserAccessLevel::SuperAdmin => "superadmin",
            // Legacy role mapping
            UserAccessLevel::Paid | UserAccessLevel::Premium => "pro",
            _ => "free",
        };

        self.role_permissions
            .get(role_key)
            .cloned()
            .unwrap_or_default()
    }

    /// Get API access configuration for a role
    pub fn get_api_access(&self, role: &UserAccessLevel) -> ApiAccessConfig {
        let role_key = match role {
            UserAccessLevel::Free => "free",
            UserAccessLevel::Pro => "pro",
            UserAccessLevel::Ultra => "ultra",
            UserAccessLevel::Admin => "admin",
            UserAccessLevel::SuperAdmin => "superadmin",
            // Legacy role mapping
            UserAccessLevel::Paid | UserAccessLevel::Premium => "pro",
            _ => "free",
        };

        self.api_access
            .get(role_key)
            .cloned()
            .unwrap_or_else(|| ApiAccessConfig {
                exchange_api_required: 0,
                exchange_api_recommended: 1,
                ai_api_enabled: false,
                ai_api_required: false,
            })
    }

    /// Get trading configuration for a role
    pub fn get_trading_config(&self, role: &UserAccessLevel) -> TradingConfig {
        let role_key = match role {
            UserAccessLevel::Free => "free",
            UserAccessLevel::Pro => "pro",
            UserAccessLevel::Ultra => "ultra",
            UserAccessLevel::Admin => "admin",
            UserAccessLevel::SuperAdmin => "superadmin",
            // Legacy role mapping
            UserAccessLevel::Paid | UserAccessLevel::Premium => "pro",
            _ => "free",
        };

        self.trading_limits
            .get(role_key)
            .cloned()
            .unwrap_or_else(|| TradingConfig {
                max_concurrent_trades: 1,
                max_leverage: 2.0,
                max_position_size_percent: 10.0,
                stop_loss_required: true,
                take_profit_recommended: true,
                risk_management_required: true,
            })
    }

    /// Get group access configuration for a role
    pub fn get_group_access(&self, role: &UserAccessLevel) -> GroupChannelAccess {
        let role_key = match role {
            UserAccessLevel::Free => "free",
            UserAccessLevel::Pro => "pro",
            UserAccessLevel::Ultra => "ultra",
            UserAccessLevel::Admin => "admin",
            UserAccessLevel::SuperAdmin => "superadmin",
            // Legacy role mapping
            UserAccessLevel::Paid | UserAccessLevel::Premium => "pro",
            _ => "free",
        };

        self.group_access
            .get(role_key)
            .cloned()
            .unwrap_or_else(|| GroupChannelAccess {
                basic_groups: vec!["general".to_string()],
                premium_groups: vec![],
                ultra_groups: vec![],
                admin_groups: vec![],
                max_groups_per_tier: 1,
            })
    }

    /// Check if feature flag is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.get(feature).copied().unwrap_or(false)
    }

    /// Update feature flag
    pub fn set_feature_flag(&mut self, feature: &str, enabled: bool) {
        self.feature_flags.insert(feature.to_string(), enabled);
        console_log!("🚩 Feature flag '{}' set to: {}", feature, enabled);
    }

    /// Validate user access for specific operation
    pub fn validate_access(
        &self,
        role: &UserAccessLevel,
        permission: &str,
        feature_flag_manager: Option<&FeatureFlagManager>,
    ) -> bool {
        // Check if RBAC is enabled via feature flags
        if let Some(ffm) = feature_flag_manager {
            if !ffm.is_enabled("rbac.simplified_roles") {
                console_log!("⚠️ RBAC system disabled via feature flag");
                return true; // Allow all access if RBAC is disabled
            }
        }

        // SuperAdmin has all permissions
        if matches!(role, UserAccessLevel::SuperAdmin) {
            return true;
        }

        // Check role-specific permissions
        let permissions = self.get_role_permissions(role);
        permissions.contains(&permission.to_string())
            || permissions.contains(&"all_permissions".to_string())
            || permissions.contains(&"all_user_permissions".to_string())
    }
}

/// RBAC Configuration Manager
pub struct RBACConfigManager {
    config: RBACConfig,
    feature_flag_manager: Option<FeatureFlagManager>,
}

impl RBACConfigManager {
    /// Create new RBAC configuration manager
    pub fn new() -> Self {
        console_log!("🔧 Initializing RBAC Configuration Manager...");
        
        Self {
            config: RBACConfig::default(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: RBACConfig) -> Self {
        console_log!("🔧 Initializing RBAC Configuration Manager with custom config...");
        
        Self {
            config,
            feature_flag_manager: Some(FeatureFlagManager::default()),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &RBACConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut RBACConfig {
        &mut self.config
    }

    /// Check if user has permission
    pub fn check_permission(&self, role: &UserAccessLevel, permission: &str) -> bool {
        self.config.validate_access(role, permission, self.feature_flag_manager.as_ref())
    }

    /// Get comprehensive user access summary
    pub fn get_user_access_summary(&self, role: &UserAccessLevel) -> UserAccessSummary {
        UserAccessSummary {
            role: role.clone(),
            permissions: self.config.get_role_permissions(role),
            api_access: self.config.get_api_access(role),
            trading_config: self.config.get_trading_config(role),
            group_access: self.config.get_group_access(role),
        }
    }
}

/// Comprehensive user access summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccessSummary {
    pub role: UserAccessLevel,
    pub permissions: Vec<String>,
    pub api_access: ApiAccessConfig,
    pub trading_config: TradingConfig,
    pub group_access: GroupChannelAccess,
}

impl Default for RBACConfigManager {
    fn default() -> Self {
        Self::new()
    }
}