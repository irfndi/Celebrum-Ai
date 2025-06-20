//! YAML-Based Technical Strategy System
//!
//! Manages technical trading strategies defined in YAML format with versioning,
//! role-based access control, and strategy execution management.

use crate::services::core::auth::rbac_config::RBACConfigManager;
use crate::types::{SubscriptionTier, UserAccessLevel};
use crate::utils::feature_flags::FeatureFlagManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::console_log;

/// Strategy execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyStatus {
    Draft,
    Active,
    Paused,
    Stopped,
    Archived,
    Error,
}

/// Strategy complexity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrategyComplexity {
    Basic = 1,
    Intermediate = 2,
    Advanced = 3,
    Expert = 4,
}

/// Technical indicator types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndicatorType {
    SMA,
    EMA,
    RSI,
    MACD,
    BollingerBands,
    Stochastic,
    ATR,
    ADX,
    CCI,
    Williams,
    Momentum,
    ROC,
    Custom(String),
}

/// Strategy condition operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
    CrossAbove,
    CrossBelow,
    And,
    Or,
    Not,
}

/// Strategy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCondition {
    pub id: String,
    pub indicator: IndicatorType,
    pub operator: ConditionOperator,
    pub value: f64,
    pub timeframe: String,
    pub lookback_periods: u32,
    pub weight: f64, // 0.0 to 1.0
}

/// Strategy entry/exit rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRules {
    pub entry_conditions: Vec<StrategyCondition>,
    pub exit_conditions: Vec<StrategyCondition>,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
    pub trailing_stop_percentage: Option<f64>,
    pub max_position_size: f64,
    pub min_confidence_score: f64,
}

/// Strategy risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRiskManagement {
    pub max_drawdown_percentage: f64,
    pub max_daily_loss_percentage: f64,
    pub max_concurrent_positions: u32,
    pub position_sizing_method: String, // "fixed", "percentage", "kelly", "volatility"
    pub risk_per_trade_percentage: f64,
    pub correlation_limit: f64,
    pub volatility_adjustment: bool,
}

/// Strategy backtesting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
    pub commission_percentage: f64,
    pub slippage_percentage: f64,
    pub benchmark_symbol: Option<String>,
    pub rebalance_frequency: String, // "daily", "weekly", "monthly"
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub volatility: f64,
    pub beta: Option<f64>,
    pub alpha: Option<f64>,
}

/// Technical strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: StrategyStatus,
    pub complexity: StrategyComplexity,
    pub required_role: UserAccessLevel,
    pub symbols: Vec<String>,
    pub timeframes: Vec<String>,
    pub rules: StrategyRules,
    pub risk_management: StrategyRiskManagement,
    pub backtest_config: Option<BacktestConfig>,
    pub performance: Option<StrategyPerformance>,
    pub yaml_content: String,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub download_count: u64,
    pub rating: f64, // 0.0 to 5.0
    pub rating_count: u64,
}

/// Strategy access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAccessConfig {
    pub max_strategies: u32,
    pub max_active_strategies: u32,
    pub can_create_strategies: bool,
    pub can_modify_strategies: bool,
    pub can_share_strategies: bool,
    pub can_access_marketplace: bool,
    pub allowed_complexity_levels: Vec<StrategyComplexity>,
    pub max_backtest_period_days: u32,
    pub can_use_custom_indicators: bool,
    pub max_concurrent_backtests: u32,
}

/// User strategy access tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStrategyAccess {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub subscription_tier: SubscriptionTier,
    pub created_strategies: u32,
    pub active_strategies: u32,
    pub total_backtests: u64,
    pub concurrent_backtests: u32,
    pub last_strategy_creation: u64,
    pub last_backtest: u64,
    pub strategy_performance_avg: f64,
}

/// Strategy version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersion {
    pub version: String,
    pub created_at: u64,
    pub author: String,
    pub changelog: String,
    pub yaml_content: String,
    pub performance: Option<StrategyPerformance>,
    pub is_stable: bool,
}

/// Technical Strategy Manager
pub struct TechnicalStrategyManager {
    rbac_manager: RBACConfigManager,
    strategies: HashMap<String, TechnicalStrategy>,
    strategy_versions: HashMap<String, Vec<StrategyVersion>>,
    user_access: HashMap<String, UserStrategyAccess>,
    access_configs: HashMap<String, StrategyAccessConfig>,
    feature_flag_manager: Option<FeatureFlagManager>,
    marketplace_strategies: HashMap<String, TechnicalStrategy>,
}

impl TechnicalStrategyManager {
    /// Create new technical strategy manager
    pub fn new() -> Self {
        console_log!("📊 Initializing Technical Strategy Manager...");
        
        let mut manager = Self {
            rbac_manager: RBACConfigManager::new(),
            strategies: HashMap::new(),
            strategy_versions: HashMap::new(),
            user_access: HashMap::new(),
            access_configs: HashMap::new(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
            marketplace_strategies: HashMap::new(),
        };
        
        manager.init_access_configs();
        manager.init_marketplace_strategies();
        manager
    }

    /// Initialize strategy access configurations for different roles
    fn init_access_configs(&mut self) {
        // Free tier configuration
        self.access_configs.insert(
            "free".to_string(),
            StrategyAccessConfig {
                max_strategies: 3,
                max_active_strategies: 1,
                can_create_strategies: true,
                can_modify_strategies: true,
                can_share_strategies: false,
                can_access_marketplace: true,
                allowed_complexity_levels: vec![StrategyComplexity::Basic],
                max_backtest_period_days: 30,
                can_use_custom_indicators: false,
                max_concurrent_backtests: 1,
            },
        );

        // Pro tier configuration
        self.access_configs.insert(
            "pro".to_string(),
            StrategyAccessConfig {
                max_strategies: 15,
                max_active_strategies: 5,
                can_create_strategies: true,
                can_modify_strategies: true,
                can_share_strategies: true,
                can_access_marketplace: true,
                allowed_complexity_levels: vec![
                    StrategyComplexity::Basic,
                    StrategyComplexity::Intermediate,
                ],
                max_backtest_period_days: 365,
                can_use_custom_indicators: true,
                max_concurrent_backtests: 3,
            },
        );

        // Ultra tier configuration
        self.access_configs.insert(
            "ultra".to_string(),
            StrategyAccessConfig {
                max_strategies: 50,
                max_active_strategies: 20,
                can_create_strategies: true,
                can_modify_strategies: true,
                can_share_strategies: true,
                can_access_marketplace: true,
                allowed_complexity_levels: vec![
                    StrategyComplexity::Basic,
                    StrategyComplexity::Intermediate,
                    StrategyComplexity::Advanced,
                    StrategyComplexity::Expert,
                ],
                max_backtest_period_days: 1825, // 5 years
                can_use_custom_indicators: true,
                max_concurrent_backtests: 10,
            },
        );

        // Admin configuration
        self.access_configs.insert(
            "admin".to_string(),
            StrategyAccessConfig {
                max_strategies: 200,
                max_active_strategies: 50,
                can_create_strategies: true,
                can_modify_strategies: true,
                can_share_strategies: true,
                can_access_marketplace: true,
                allowed_complexity_levels: vec![
                    StrategyComplexity::Basic,
                    StrategyComplexity::Intermediate,
                    StrategyComplexity::Advanced,
                    StrategyComplexity::Expert,
                ],
                max_backtest_period_days: u32::MAX,
                can_use_custom_indicators: true,
                max_concurrent_backtests: 25,
            },
        );

        // SuperAdmin configuration
        self.access_configs.insert(
            "superadmin".to_string(),
            StrategyAccessConfig {
                max_strategies: u32::MAX,
                max_active_strategies: u32::MAX,
                can_create_strategies: true,
                can_modify_strategies: true,
                can_share_strategies: true,
                can_access_marketplace: true,
                allowed_complexity_levels: vec![
                    StrategyComplexity::Basic,
                    StrategyComplexity::Intermediate,
                    StrategyComplexity::Advanced,
                    StrategyComplexity::Expert,
                ],
                max_backtest_period_days: u32::MAX,
                can_use_custom_indicators: true,
                max_concurrent_backtests: u32::MAX,
            },
        );
    }

    /// Initialize marketplace with sample strategies
    fn init_marketplace_strategies(&mut self) {
        // Sample basic strategy for marketplace
        let basic_strategy = TechnicalStrategy {
            id: "marketplace_basic_sma_crossover".to_string(),
            name: "Simple Moving Average Crossover".to_string(),
            description: "Basic SMA crossover strategy for beginners".to_string(),
            version: "1.0.0".to_string(),
            author: "ArbEdge Team".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            updated_at: Utc::now().timestamp_millis() as u64,
            status: StrategyStatus::Active,
            complexity: StrategyComplexity::Basic,
            required_role: UserAccessLevel::Free,
            symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            timeframes: vec!["1h".to_string(), "4h".to_string()],
            rules: StrategyRules {
                entry_conditions: vec![StrategyCondition {
                    id: "sma_cross_up".to_string(),
                    indicator: IndicatorType::SMA,
                    operator: ConditionOperator::CrossAbove,
                    value: 50.0,
                    timeframe: "1h".to_string(),
                    lookback_periods: 20,
                    weight: 1.0,
                }],
                exit_conditions: vec![StrategyCondition {
                    id: "sma_cross_down".to_string(),
                    indicator: IndicatorType::SMA,
                    operator: ConditionOperator::CrossBelow,
                    value: 50.0,
                    timeframe: "1h".to_string(),
                    lookback_periods: 20,
                    weight: 1.0,
                }],
                stop_loss_percentage: Some(2.0),
                take_profit_percentage: Some(4.0),
                trailing_stop_percentage: None,
                max_position_size: 0.1,
                min_confidence_score: 0.6,
            },
            risk_management: StrategyRiskManagement {
                max_drawdown_percentage: 10.0,
                max_daily_loss_percentage: 2.0,
                max_concurrent_positions: 1,
                position_sizing_method: "percentage".to_string(),
                risk_per_trade_percentage: 1.0,
                correlation_limit: 0.7,
                volatility_adjustment: false,
            },
            backtest_config: None,
            performance: None,
            yaml_content: self.generate_sample_yaml(),
            metadata: HashMap::new(),
            tags: vec!["beginner".to_string(), "sma".to_string(), "crossover".to_string()],
            is_public: true,
            download_count: 0,
            rating: 4.2,
            rating_count: 15,
        };
        
        self.marketplace_strategies.insert(basic_strategy.id.clone(), basic_strategy);
    }

    /// Generate sample YAML content
    fn generate_sample_yaml(&self) -> String {
        r#"# Simple Moving Average Crossover Strategy
name: "SMA Crossover"
version: "1.0.0"
author: "ArbEdge Team"
description: "Basic SMA crossover strategy for beginners"

# Strategy Configuration
strategy:
  complexity: "basic"
  symbols: ["BTCUSDT", "ETHUSDT"]
  timeframes: ["1h", "4h"]
  
# Technical Indicators
indicators:
  sma_fast:
    type: "SMA"
    period: 20
    source: "close"
  sma_slow:
    type: "SMA"
    period: 50
    source: "close"

# Entry Conditions
entry:
  conditions:
    - indicator: "sma_fast"
      operator: "cross_above"
      value: "sma_slow"
      weight: 1.0
  min_confidence: 0.6

# Exit Conditions
exit:
  conditions:
    - indicator: "sma_fast"
      operator: "cross_below"
      value: "sma_slow"
      weight: 1.0
  stop_loss: 2.0  # percentage
  take_profit: 4.0  # percentage

# Risk Management
risk_management:
  max_drawdown: 10.0  # percentage
  max_daily_loss: 2.0  # percentage
  max_positions: 1
  position_sizing: "percentage"
  risk_per_trade: 1.0  # percentage
  correlation_limit: 0.7
"#.to_string()
    }

    /// Register user for strategy access
    pub fn register_user(
        &mut self,
        user_id: &str,
        role: UserAccessLevel,
        subscription_tier: SubscriptionTier,
    ) {
        let user_access = UserStrategyAccess {
            user_id: user_id.to_string(),
            role,
            subscription_tier,
            created_strategies: 0,
            active_strategies: 0,
            total_backtests: 0,
            concurrent_backtests: 0,
            last_strategy_creation: 0,
            last_backtest: 0,
            strategy_performance_avg: 0.0,
        };
        
        self.user_access.insert(user_id.to_string(), user_access);
        
        console_log!(
            "📝 Registered user '{}' for strategy access with role: {:?}",
            user_id,
            role
        );
    }

    /// Create new strategy from YAML
    pub fn create_strategy_from_yaml(
        &mut self,
        user_id: &str,
        yaml_content: &str,
        strategy_name: &str,
        description: &str,
    ) -> Result<String, String> {
        // Check if strategy system is enabled
        if let Some(ffm) = &self.feature_flag_manager {
            if !ffm.is_enabled("technical_strategies.enabled") {
                return Err("Technical strategy system is disabled".to_string());
            }
        }

        // Get user access info
        let user_access = self.user_access
            .get_mut(user_id)
            .ok_or_else(|| "User not registered for strategy access".to_string())?;
        
        // Get access configuration
        let access_config = self.get_access_config(&user_access.role)?;
        
        // Check if user can create strategies
        if !access_config.can_create_strategies {
            return Err("User does not have permission to create strategies".to_string());
        }
        
        // Check strategy limit
        if user_access.created_strategies >= access_config.max_strategies {
            return Err("Maximum strategy limit reached".to_string());
        }
        
        // Parse and validate YAML
        let strategy_data = self.parse_yaml_strategy(yaml_content)?;
        
        // Check complexity level access
        if !access_config.allowed_complexity_levels.contains(&strategy_data.complexity) {
            return Err(format!(
                "User does not have access to {:?} complexity strategies",
                strategy_data.complexity
            ));
        }
        
        // Generate strategy ID
        let strategy_id = format!("user_{}_{}", user_id, Utc::now().timestamp_millis() as u64);
        
        // Create strategy
        let strategy = TechnicalStrategy {
            id: strategy_id.clone(),
            name: strategy_name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            author: user_id.to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            updated_at: Utc::now().timestamp_millis() as u64,
            status: StrategyStatus::Draft,
            complexity: strategy_data.complexity,
            required_role: user_access.role.clone(),
            symbols: strategy_data.symbols,
            timeframes: strategy_data.timeframes,
            rules: strategy_data.rules,
            risk_management: strategy_data.risk_management,
            backtest_config: strategy_data.backtest_config,
            performance: None,
            yaml_content: yaml_content.to_string(),
            metadata: HashMap::new(),
            tags: Vec::new(),
            is_public: false,
            download_count: 0,
            rating: 0.0,
            rating_count: 0,
        };
        
        // Store strategy
        self.strategies.insert(strategy_id.clone(), strategy);
        
        // Initialize version history
        let initial_version = StrategyVersion {
            version: "1.0.0".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            author: user_id.to_string(),
            changelog: "Initial version".to_string(),
            yaml_content: yaml_content.to_string(),
            performance: None,
            is_stable: false,
        };
        
        self.strategy_versions.insert(strategy_id.clone(), vec![initial_version]);
        
        // Update user access
        user_access.created_strategies += 1;
        user_access.last_strategy_creation = Utc::now().timestamp_millis() as u64;
        
        console_log!(
            "✅ Created strategy '{}' for user: {} (total: {})",
            strategy_name,
            user_id,
            user_access.created_strategies
        );
        
        Ok(strategy_id)
    }

    /// Parse YAML strategy content
    fn parse_yaml_strategy(&self, _yaml_content: &str) -> Result<StrategyData, String> {
        // In a real implementation, this would parse the YAML content
        // For now, return a basic strategy structure
        Ok(StrategyData {
            complexity: StrategyComplexity::Basic,
            symbols: vec!["BTCUSDT".to_string()],
            timeframes: vec!["1h".to_string()],
            rules: StrategyRules {
                entry_conditions: Vec::new(),
                exit_conditions: Vec::new(),
                stop_loss_percentage: Some(2.0),
                take_profit_percentage: Some(4.0),
                trailing_stop_percentage: None,
                max_position_size: 0.1,
                min_confidence_score: 0.6,
            },
            risk_management: StrategyRiskManagement {
                max_drawdown_percentage: 10.0,
                max_daily_loss_percentage: 2.0,
                max_concurrent_positions: 1,
                position_sizing_method: "percentage".to_string(),
                risk_per_trade_percentage: 1.0,
                correlation_limit: 0.7,
                volatility_adjustment: false,
            },
            backtest_config: None,
        })
    }

    /// Get user strategies
    pub fn get_user_strategies(&self, user_id: &str) -> Result<Vec<&TechnicalStrategy>, String> {
        let user_access = self.user_access
            .get(user_id)
            .ok_or_else(|| "User not registered".to_string())?;
        
        let strategies: Vec<&TechnicalStrategy> = self.strategies
            .values()
            .filter(|strategy| strategy.author == user_id)
            .collect();
        
        console_log!(
            "📊 Retrieved {} strategies for user: {}",
            strategies.len(),
            user_id
        );
        
        Ok(strategies)
    }

    /// Get marketplace strategies
    pub fn get_marketplace_strategies(
        &self,
        user_id: &str,
        complexity_filter: Option<StrategyComplexity>,
    ) -> Result<Vec<&TechnicalStrategy>, String> {
        let user_access = self.user_access
            .get(user_id)
            .ok_or_else(|| "User not registered".to_string())?;
        
        let access_config = self.get_access_config(&user_access.role)?;
        
        if !access_config.can_access_marketplace {
            return Err("User does not have marketplace access".to_string());
        }
        
        let mut strategies: Vec<&TechnicalStrategy> = self.marketplace_strategies
            .values()
            .filter(|strategy| {
                // Check if user has access to this complexity level
                access_config.allowed_complexity_levels.contains(&strategy.complexity)
            })
            .collect();
        
        // Apply complexity filter if provided
        if let Some(complexity) = complexity_filter {
            strategies.retain(|strategy| strategy.complexity == complexity);
        }
        
        // Sort by rating and download count
        strategies.sort_by(|a, b| {
            b.rating.partial_cmp(&a.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.download_count.cmp(&a.download_count))
        });
        
        console_log!(
            "🏪 Retrieved {} marketplace strategies for user: {}",
            strategies.len(),
            user_id
        );
        
        Ok(strategies)
    }

    /// Update strategy version
    pub fn update_strategy_version(
        &mut self,
        user_id: &str,
        strategy_id: &str,
        new_yaml_content: &str,
        changelog: &str,
    ) -> Result<String, String> {
        // Get strategy
        let strategy = self.strategies
            .get_mut(strategy_id)
            .ok_or_else(|| "Strategy not found".to_string())?;
        
        // Check ownership
        if strategy.author != user_id {
            return Err("User does not own this strategy".to_string());
        }
        
        // Parse new version
        let strategy_data = self.parse_yaml_strategy(new_yaml_content)?;
        
        // Generate new version number
        let versions = self.strategy_versions
            .get(strategy_id)
            .ok_or_else(|| "Strategy versions not found".to_string())?;
        
        let new_version = self.increment_version(&strategy.version);
        
        // Create new version
        let version = StrategyVersion {
            version: new_version.clone(),
            created_at: Utc::now().timestamp_millis() as u64,
            author: user_id.to_string(),
            changelog: changelog.to_string(),
            yaml_content: new_yaml_content.to_string(),
            performance: None,
            is_stable: false,
        };
        
        // Update strategy
        strategy.version = new_version.clone();
        strategy.updated_at = Utc::now().timestamp_millis() as u64;
        strategy.yaml_content = new_yaml_content.to_string();
        strategy.rules = strategy_data.rules;
        strategy.risk_management = strategy_data.risk_management;
        
        // Add version to history
        self.strategy_versions
            .get_mut(strategy_id)
            .unwrap()
            .push(version);
        
        console_log!(
            "🔄 Updated strategy '{}' to version: {}",
            strategy_id,
            new_version
        );
        
        Ok(new_version)
    }

    /// Increment version number
    fn increment_version(&self, current_version: &str) -> String {
        let parts: Vec<&str> = current_version.split('.').collect();
        if parts.len() == 3 {
            if let (Ok(major), Ok(minor), Ok(patch)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return format!("{}.{}.{}", major, minor, patch + 1);
            }
        }
        "1.0.1".to_string()
    }

    /// Get access configuration for role
    fn get_access_config(&self, role: &UserAccessLevel) -> Result<&StrategyAccessConfig, String> {
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

    /// Get user strategy statistics
    pub fn get_user_stats(&self, user_id: &str) -> Result<UserStrategyStats, String> {
        let user_access = self.user_access
            .get(user_id)
            .ok_or_else(|| "User not found".to_string())?;
        
        let access_config = self.get_access_config(&user_access.role)?;
        
        Ok(UserStrategyStats {
            user_id: user_id.to_string(),
            role: user_access.role.clone(),
            subscription_tier: user_access.subscription_tier.clone(),
            max_strategies: access_config.max_strategies,
            created_strategies: user_access.created_strategies,
            max_active_strategies: access_config.max_active_strategies,
            active_strategies: user_access.active_strategies,
            total_backtests: user_access.total_backtests,
            max_concurrent_backtests: access_config.max_concurrent_backtests,
            concurrent_backtests: user_access.concurrent_backtests,
            average_performance: user_access.strategy_performance_avg,
            last_strategy_creation: user_access.last_strategy_creation,
            last_backtest: user_access.last_backtest,
        })
    }
}

/// Parsed strategy data from YAML
#[derive(Debug, Clone)]
struct StrategyData {
    complexity: StrategyComplexity,
    symbols: Vec<String>,
    timeframes: Vec<String>,
    rules: StrategyRules,
    risk_management: StrategyRiskManagement,
    backtest_config: Option<BacktestConfig>,
}

/// User strategy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStrategyStats {
    pub user_id: String,
    pub role: UserAccessLevel,
    pub subscription_tier: SubscriptionTier,
    pub max_strategies: u32,
    pub created_strategies: u32,
    pub max_active_strategies: u32,
    pub active_strategies: u32,
    pub total_backtests: u64,
    pub max_concurrent_backtests: u32,
    pub concurrent_backtests: u32,
    pub average_performance: f64,
    pub last_strategy_creation: u64,
    pub last_backtest: u64,
}

impl Default for TechnicalStrategyManager {
    fn default() -> Self {
        Self::new()
    }
}