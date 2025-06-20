//! Arbitrage Opportunity System
//!
//! Manages arbitrage opportunities with role-based access control,
//! opportunity limits, and feature flag integration.

use crate::services::core::auth::rbac_config::RBACConfigManager;
use crate::types::UserAccessLevel;
use crate::utils::feature_flags::FeatureFlagManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::console_log;

/// Arbitrage opportunity types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArbitrageType {
    SpotArbitrage,
    FuturesArbitrage,
    CrossExchangeArbitrage,
    TriangularArbitrage,
    StatisticalArbitrage,
    LatencyArbitrage,
}

/// Opportunity priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpportunityPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Arbitrage opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub opportunity_type: ArbitrageType,
    pub priority: OpportunityPriority,
    pub symbol: String,
    pub exchange_a: String,
    pub exchange_b: String,
    pub price_a: f64,
    pub price_b: f64,
    pub profit_percentage: f64,
    pub profit_usd: f64,
    pub volume_available: f64,
    pub min_investment: f64,
    pub max_investment: f64,
    pub execution_time_ms: u64,
    pub confidence_score: f64, // 0.0 to 1.0
    pub risk_score: f64,       // 0.0 to 1.0
    pub created_at: u64,
    pub expires_at: u64,
    pub required_role: UserAccessLevel,
    pub required_apis: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Opportunity access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityAccessConfig {
    pub daily_limit: u32,
    pub hourly_limit: u32,
    pub min_profit_threshold: f64,
    pub max_risk_threshold: f64,
    pub allowed_types: Vec<ArbitrageType>,
    pub priority_access: Vec<OpportunityPriority>,
    pub requires_ai_analysis: bool,
}

/// User opportunity access tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpportunityAccess {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub subscription_tier: SubscriptionTier,
    pub daily_accessed: u32,
    pub hourly_accessed: u32,
    pub last_access: u64,
    pub last_daily_reset: u64,
    pub last_hourly_reset: u64,
    pub total_opportunities_accessed: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
}

/// Opportunity filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityFilter {
    pub min_profit_percentage: Option<f64>,
    pub max_risk_score: Option<f64>,
    pub min_confidence_score: Option<f64>,
    pub opportunity_types: Option<Vec<ArbitrageType>>,
    pub exchanges: Option<Vec<String>>,
    pub symbols: Option<Vec<String>>,
    pub min_volume: Option<f64>,
    pub max_execution_time: Option<u64>,
    pub priority_levels: Option<Vec<OpportunityPriority>>,
}

/// Arbitrage Opportunity Manager
pub struct ArbitrageOpportunityManager {
    rbac_manager: RBACConfigManager,
    opportunities: HashMap<String, ArbitrageOpportunity>,
    user_access: HashMap<String, UserOpportunityAccess>,
    access_configs: HashMap<String, OpportunityAccessConfig>,
    feature_flag_manager: Option<FeatureFlagManager>,
}

impl ArbitrageOpportunityManager {
    /// Create new arbitrage opportunity manager
    pub fn new() -> Self {
        console_log!("🎯 Initializing Arbitrage Opportunity Manager...");
        
        let mut manager = Self {
            rbac_manager: RBACConfigManager::new(),
            opportunities: HashMap::new(),
            user_access: HashMap::new(),
            access_configs: HashMap::new(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
        };
        
        manager.init_access_configs();
        manager
    }

    /// Initialize opportunity access configurations for different roles
    fn init_access_configs(&mut self) {
        // Free tier configuration
        self.access_configs.insert(
            "free".to_string(),
            OpportunityAccessConfig {
                daily_limit: 5,
                hourly_limit: 2,
                min_profit_threshold: 2.0,
                max_risk_threshold: 0.3,
                allowed_types: vec![ArbitrageType::SpotArbitrage],
                priority_access: vec![OpportunityPriority::Low],
                requires_ai_analysis: false,
            },
        );

        // Pro tier configuration
        self.access_configs.insert(
            "pro".to_string(),
            OpportunityAccessConfig {
                daily_limit: 25,
                hourly_limit: 10,
                min_profit_threshold: 1.0,
                max_risk_threshold: 0.5,
                allowed_types: vec![
                    ArbitrageType::SpotArbitrage,
                    ArbitrageType::CrossExchangeArbitrage,
                    ArbitrageType::TriangularArbitrage,
                ],
                priority_access: vec![OpportunityPriority::Low, OpportunityPriority::Medium],
                requires_ai_analysis: false,
            },
        );

        // Ultra tier configuration
        self.access_configs.insert(
            "ultra".to_string(),
            OpportunityAccessConfig {
                daily_limit: 100,
                hourly_limit: 50,
                min_profit_threshold: 0.5,
                max_risk_threshold: 0.8,
                allowed_types: vec![
                    ArbitrageType::SpotArbitrage,
                    ArbitrageType::FuturesArbitrage,
                    ArbitrageType::CrossExchangeArbitrage,
                    ArbitrageType::TriangularArbitrage,
                    ArbitrageType::StatisticalArbitrage,
                    ArbitrageType::LatencyArbitrage,
                ],
                priority_access: vec![
                    OpportunityPriority::Low,
                    OpportunityPriority::Medium,
                    OpportunityPriority::High,
                    OpportunityPriority::Critical,
                ],
                requires_ai_analysis: true,
            },
        );

        // Admin configuration
        self.access_configs.insert(
            "admin".to_string(),
            OpportunityAccessConfig {
                daily_limit: 500,
                hourly_limit: 200,
                min_profit_threshold: 0.1,
                max_risk_threshold: 1.0,
                allowed_types: vec![
                    ArbitrageType::SpotArbitrage,
                    ArbitrageType::FuturesArbitrage,
                    ArbitrageType::CrossExchangeArbitrage,
                    ArbitrageType::TriangularArbitrage,
                    ArbitrageType::StatisticalArbitrage,
                    ArbitrageType::LatencyArbitrage,
                ],
                priority_access: vec![
                    OpportunityPriority::Low,
                    OpportunityPriority::Medium,
                    OpportunityPriority::High,
                    OpportunityPriority::Critical,
                ],
                requires_ai_analysis: false,
            },
        );

        // SuperAdmin configuration
        self.access_configs.insert(
            "superadmin".to_string(),
            OpportunityAccessConfig {
                daily_limit: u32::MAX,
                hourly_limit: u32::MAX,
                min_profit_threshold: 0.0,
                max_risk_threshold: 1.0,
                allowed_types: vec![
                    ArbitrageType::SpotArbitrage,
                    ArbitrageType::FuturesArbitrage,
                    ArbitrageType::CrossExchangeArbitrage,
                    ArbitrageType::TriangularArbitrage,
                    ArbitrageType::StatisticalArbitrage,
                    ArbitrageType::LatencyArbitrage,
                ],
                priority_access: vec![
                    OpportunityPriority::Low,
                    OpportunityPriority::Medium,
                    OpportunityPriority::High,
                    OpportunityPriority::Critical,
                ],
                requires_ai_analysis: false,
            },
        );
    }

    /// Register user for opportunity access
    pub fn register_user(
        &mut self,
        user_id: &str,
        role: UserAccessLevel,
        subscription_tier: SubscriptionTier,
    ) {
        let now = Utc::now().timestamp_millis() as u64;
        
        let user_access = UserOpportunityAccess {
            user_id: user_id.to_string(),
            role,
            subscription_tier,
            daily_accessed: 0,
            hourly_accessed: 0,
            last_access: 0,
            last_daily_reset: now,
            last_hourly_reset: now,
            total_opportunities_accessed: 0,
            successful_executions: 0,
            failed_executions: 0,
        };
        
        self.user_access.insert(user_id.to_string(), user_access);
        
        console_log!(
            "📝 Registered user '{}' for opportunity access with role: {:?}",
            user_id,
            role
        );
    }

    /// Add new arbitrage opportunity
    pub fn add_opportunity(&mut self, opportunity: ArbitrageOpportunity) {
        console_log!(
            "➕ Adding arbitrage opportunity: {} ({}% profit)",
            opportunity.id,
            opportunity.profit_percentage
        );
        
        self.opportunities.insert(opportunity.id.clone(), opportunity);
    }

    /// Remove expired opportunities
    pub fn cleanup_expired_opportunities(&mut self) {
        let now = Utc::now().timestamp_millis() as u64;
        let initial_count = self.opportunities.len();
        
        self.opportunities.retain(|_, opportunity| opportunity.expires_at > now);
        
        let removed_count = initial_count - self.opportunities.len();
        if removed_count > 0 {
            console_log!("🧹 Cleaned up {} expired opportunities", removed_count);
        }
    }

    /// Get opportunities for user with role-based filtering
    pub fn get_opportunities_for_user(
        &mut self,
        user_id: &str,
        filter: Option<OpportunityFilter>,
    ) -> Result<Vec<ArbitrageOpportunity>, String> {
        // Check if opportunity system is enabled
        if let Some(ffm) = &self.feature_flag_manager {
            if !ffm.is_enabled("opportunity_engine.enabled") {
                return Err("Opportunity engine is disabled".to_string());
            }
        }

        // Clean up expired opportunities first
        self.cleanup_expired_opportunities();
        
        // Get user access info
        let user_access = self.user_access
            .get_mut(user_id)
            .ok_or_else(|| "User not registered for opportunity access".to_string())?;
        
        // Reset counters if needed
        self.reset_user_counters(user_access);
        
        // Get access configuration for user role
        let access_config = self.get_access_config(&user_access.role)?;
        
        // Check daily and hourly limits
        if user_access.daily_accessed >= access_config.daily_limit {
            return Err("Daily opportunity limit reached".to_string());
        }
        
        if user_access.hourly_accessed >= access_config.hourly_limit {
            return Err("Hourly opportunity limit reached".to_string());
        }
        
        // Filter opportunities based on role and access config
        let mut filtered_opportunities: Vec<ArbitrageOpportunity> = self.opportunities
            .values()
            .filter(|opp| self.can_access_opportunity(user_access, access_config, opp))
            .cloned()
            .collect();
        
        // Apply additional user filters
        if let Some(filter) = filter {
            filtered_opportunities = self.apply_user_filter(filtered_opportunities, &filter);
        }
        
        // Sort by priority and profit percentage
        filtered_opportunities.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.profit_percentage.partial_cmp(&a.profit_percentage).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        // Update access counters
        user_access.daily_accessed += 1;
        user_access.hourly_accessed += 1;
        user_access.last_access = Utc::now().timestamp_millis() as u64;
        user_access.total_opportunities_accessed += 1;
        
        console_log!(
            "📊 Retrieved {} opportunities for user: {} (daily: {}/{}, hourly: {}/{})",
            filtered_opportunities.len(),
            user_id,
            user_access.daily_accessed,
            access_config.daily_limit,
            user_access.hourly_accessed,
            access_config.hourly_limit
        );
        
        Ok(filtered_opportunities)
    }

    /// Check if user can access specific opportunity
    fn can_access_opportunity(
        &self,
        user_access: &UserOpportunityAccess,
        access_config: &OpportunityAccessConfig,
        opportunity: &ArbitrageOpportunity,
    ) -> bool {
        // Check role requirement
        if !self.rbac_manager.check_permission(&user_access.role, "view_basic_opportunities") {
            return false;
        }
        
        // Check if opportunity type is allowed
        if !access_config.allowed_types.contains(&opportunity.opportunity_type) {
            return false;
        }
        
        // Check priority access
        if !access_config.priority_access.contains(&opportunity.priority) {
            return false;
        }
        
        // Check profit threshold
        if opportunity.profit_percentage < access_config.min_profit_threshold {
            return false;
        }
        
        // Check risk threshold
        if opportunity.risk_score > access_config.max_risk_threshold {
            return false;
        }
        
        // Check if AI analysis is required and user has access
        if access_config.requires_ai_analysis {
            if !self.rbac_manager.check_permission(&user_access.role, "ai_analysis") {
                return false;
            }
        }
        
        true
    }

    /// Apply user-defined filters
    fn apply_user_filter(
        &self,
        opportunities: Vec<ArbitrageOpportunity>,
        filter: &OpportunityFilter,
    ) -> Vec<ArbitrageOpportunity> {
        opportunities
            .into_iter()
            .filter(|opp| {
                if let Some(min_profit) = filter.min_profit_percentage {
                    if opp.profit_percentage < min_profit {
                        return false;
                    }
                }
                
                if let Some(max_risk) = filter.max_risk_score {
                    if opp.risk_score > max_risk {
                        return false;
                    }
                }
                
                if let Some(min_confidence) = filter.min_confidence_score {
                    if opp.confidence_score < min_confidence {
                        return false;
                    }
                }
                
                if let Some(ref types) = filter.opportunity_types {
                    if !types.contains(&opp.opportunity_type) {
                        return false;
                    }
                }
                
                if let Some(ref exchanges) = filter.exchanges {
                    if !exchanges.contains(&opp.exchange_a) && !exchanges.contains(&opp.exchange_b) {
                        return false;
                    }
                }
                
                if let Some(ref symbols) = filter.symbols {
                    if !symbols.contains(&opp.symbol) {
                        return false;
                    }
                }
                
                if let Some(min_volume) = filter.min_volume {
                    if opp.volume_available < min_volume {
                        return false;
                    }
                }
                
                if let Some(max_execution_time) = filter.max_execution_time {
                    if opp.execution_time_ms > max_execution_time {
                        return false;
                    }
                }
                
                if let Some(ref priorities) = filter.priority_levels {
                    if !priorities.contains(&opp.priority) {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }

    /// Reset user counters if time periods have elapsed
    fn reset_user_counters(&self, user_access: &mut UserOpportunityAccess) {
        let now = Utc::now().timestamp_millis() as u64;
        
        // Reset daily counter (24 hours)
        if now - user_access.last_daily_reset > 24 * 60 * 60 * 1000 {
            user_access.daily_accessed = 0;
            user_access.last_daily_reset = now;
        }
        
        // Reset hourly counter (1 hour)
        if now - user_access.last_hourly_reset > 60 * 60 * 1000 {
            user_access.hourly_accessed = 0;
            user_access.last_hourly_reset = now;
        }
    }

    /// Get access configuration for role
    fn get_access_config(&self, role: &UserAccessLevel) -> Result<&OpportunityAccessConfig, String> {
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
        
        self.access_configs
            .get(role_key)
            .ok_or_else(|| format!("Access configuration not found for role: {:?}", role))
    }

    /// Record opportunity execution result
    pub fn record_execution_result(
        &mut self,
        user_id: &str,
        opportunity_id: &str,
        success: bool,
    ) -> Result<(), String> {
        let user_access = self.user_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;
        
        if success {
            user_access.successful_executions += 1;
            console_log!("✅ Recorded successful execution for user: {}", user_id);
        } else {
            user_access.failed_executions += 1;
            console_log!("❌ Recorded failed execution for user: {}", user_id);
        }
        
        Ok(())
    }

    /// Get user opportunity statistics
    pub fn get_user_stats(&self, user_id: &str) -> Result<UserOpportunityStats, String> {
        let user_access = self.user_access
            .get(user_id)
            .ok_or_else(|| "User not found".to_string())?;
        
        let access_config = self.get_access_config(&user_access.role)?;
        
        let success_rate = if user_access.total_opportunities_accessed > 0 {
            user_access.successful_executions as f64 / user_access.total_opportunities_accessed as f64 * 100.0
        } else {
            0.0
        };
        
        Ok(UserOpportunityStats {
            user_id: user_id.to_string(),
            role: user_access.role.clone(),
            subscription_tier: user_access.subscription_tier.clone(),
            daily_limit: access_config.daily_limit,
            daily_used: user_access.daily_accessed,
            hourly_limit: access_config.hourly_limit,
            hourly_used: user_access.hourly_accessed,
            total_accessed: user_access.total_opportunities_accessed,
            successful_executions: user_access.successful_executions,
            failed_executions: user_access.failed_executions,
            success_rate,
            last_access: user_access.last_access,
        })
    }

    /// Get opportunity by ID
    pub fn get_opportunity(&self, opportunity_id: &str) -> Option<&ArbitrageOpportunity> {
        self.opportunities.get(opportunity_id)
    }

    /// Update user role
    pub fn update_user_role(
        &mut self,
        user_id: &str,
        new_role: UserAccessLevel,
        new_subscription_tier: SubscriptionTier,
    ) -> Result<(), String> {
        let user_access = self.user_access
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;
        
        user_access.role = new_role.clone();
        user_access.subscription_tier = new_subscription_tier.clone();
        
        console_log!(
            "🔄 Updated user role to {:?} and subscription to {:?} for user: {}",
            new_role,
            new_subscription_tier,
            user_id
        );
        
        Ok(())
    }
}

/// User opportunity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpportunityStats {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub subscription_tier: SubscriptionTier,
    pub daily_limit: u32,
    pub daily_used: u32,
    pub hourly_limit: u32,
    pub hourly_used: u32,
    pub total_accessed: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f64,
    pub last_access: u64,
}

impl Default for ArbitrageOpportunityManager {
    fn default() -> Self {
        Self::new()
    }
}