//! Trading Configuration Management System
//!
//! Manages trading limits, risk management parameters, and configuration
//! based on user roles with feature flag integration.

use crate::services::core::auth::rbac_config::RBACConfigManager;
use crate::types::UserAccessLevel;
use crate::utils::feature_flags::FeatureFlagManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::console_log;

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_daily_loss_percent: f64,
    pub max_drawdown_percent: f64,
    pub position_sizing_method: PositionSizingMethod,
    pub stop_loss_required: bool,
    pub take_profit_recommended: bool,
    pub trailing_stop_enabled: bool,
    pub risk_reward_ratio_min: f64,
}

/// Position sizing methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    FixedAmount,
    PercentageOfPortfolio,
    KellyFormula,
    VolatilityBased,
    RiskParity,
}

/// Trading session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSessionConfig {
    pub session_id: String,
    pub user_id: String,
    pub role: UserAccessLevel,
    pub active_trades: u32,
    pub max_concurrent_trades: u32,
    pub current_leverage: f64,
    pub max_leverage: f64,
    pub total_position_size_percent: f64,
    pub max_position_size_percent: f64,
    pub risk_management: RiskManagementConfig,
    pub auto_trading_enabled: bool,
    pub manual_trading_enabled: bool,
    pub session_start_time: u64,
    pub last_activity: u64,
}

/// Trade execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecutionRequest {
    pub user_id: String,
    pub symbol: String,
    pub side: TradeSide,
    pub quantity: f64,
    pub price: Option<f64>, // None for market orders
    pub leverage: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub order_type: OrderType,
    pub is_auto_trade: bool,
}

/// Trade side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Order type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    TrailingStop,
}

/// Trade validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeValidationResult {
    pub is_valid: bool,
    pub can_execute: bool,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub risk_assessment: RiskAssessment,
}

/// Risk assessment for trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_score: f64, // 0.0 to 1.0
    pub position_size_percent: f64,
    pub leverage_utilization: f64,
    pub concurrent_trades_utilization: f64,
    pub estimated_max_loss: f64,
    pub risk_reward_ratio: Option<f64>,
}

/// Trading Configuration Manager
pub struct TradingConfigManager {
    rbac_manager: RBACConfigManager,
    trading_sessions: HashMap<String, TradingSessionConfig>,
    feature_flag_manager: Option<FeatureFlagManager>,
}

impl TradingConfigManager {
    /// Create new trading configuration manager
    pub fn new() -> Self {
        console_log!("📊 Initializing Trading Configuration Manager...");

        Self {
            rbac_manager: RBACConfigManager::new(),
            trading_sessions: HashMap::new(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
        }
    }

    /// Create with custom RBAC manager
    pub fn with_rbac_manager(rbac_manager: RBACConfigManager) -> Self {
        console_log!("📊 Initializing Trading Configuration Manager with custom RBAC...");

        Self {
            rbac_manager,
            trading_sessions: HashMap::new(),
            feature_flag_manager: Some(FeatureFlagManager::default()),
        }
    }

    /// Create trading session for user
    pub fn create_trading_session(
        &mut self,
        user_id: &str,
        role: UserAccessLevel,
    ) -> Result<String, String> {
        // Check if trading limits are enabled
        if let Some(ffm) = &self.feature_flag_manager {
            if !ffm.is_enabled("rbac.trading_limits") {
                console_log!("⚠️ Trading limits disabled via feature flag");
            }
        }

        let trading_config = self.rbac_manager.config().get_trading_config(&role);
        let session_id = format!("{}_{}", user_id, Utc::now().timestamp_millis() as u64);

        let risk_management = self.get_default_risk_management(&role);

        let session = TradingSessionConfig {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            role: role.clone(),
            active_trades: 0,
            max_concurrent_trades: trading_config.max_concurrent_trades,
            current_leverage: 1.0,
            max_leverage: trading_config.max_leverage,
            total_position_size_percent: 0.0,
            max_position_size_percent: trading_config.max_position_size_percent,
            risk_management,
            auto_trading_enabled: self.rbac_manager.check_permission(&role, "auto_trading"),
            manual_trading_enabled: self.rbac_manager.check_permission(&role, "manual_trading"),
            session_start_time: Utc::now().timestamp_millis() as u64,
            last_activity: Utc::now().timestamp_millis() as u64,
        };

        self.trading_sessions.insert(session_id.clone(), session);

        console_log!(
            "✅ Created trading session '{}' for user: {} with role: {:?}",
            session_id,
            user_id,
            role
        );

        Ok(session_id)
    }

    /// Get default risk management configuration for role
    fn get_default_risk_management(&self, role: &UserAccessLevel) -> RiskManagementConfig {
        match role {
            UserAccessLevel::Free => RiskManagementConfig {
                max_daily_loss_percent: 5.0,
                max_drawdown_percent: 10.0,
                position_sizing_method: PositionSizingMethod::PercentageOfPortfolio,
                stop_loss_required: true,
                take_profit_recommended: true,
                trailing_stop_enabled: false,
                risk_reward_ratio_min: 1.5,
            },
            UserAccessLevel::Pro => RiskManagementConfig {
                max_daily_loss_percent: 10.0,
                max_drawdown_percent: 20.0,
                position_sizing_method: PositionSizingMethod::PercentageOfPortfolio,
                stop_loss_required: true,
                take_profit_recommended: true,
                trailing_stop_enabled: true,
                risk_reward_ratio_min: 1.2,
            },
            UserAccessLevel::Ultra => RiskManagementConfig {
                max_daily_loss_percent: 20.0,
                max_drawdown_percent: 30.0,
                position_sizing_method: PositionSizingMethod::VolatilityBased,
                stop_loss_required: false,
                take_profit_recommended: true,
                trailing_stop_enabled: true,
                risk_reward_ratio_min: 1.0,
            },
            UserAccessLevel::Admin | UserAccessLevel::SuperAdmin => RiskManagementConfig {
                max_daily_loss_percent: 100.0,
                max_drawdown_percent: 100.0,
                position_sizing_method: PositionSizingMethod::RiskParity,
                stop_loss_required: false,
                take_profit_recommended: false,
                trailing_stop_enabled: true,
                risk_reward_ratio_min: 0.5,
            },
            // Legacy role mapping
            UserAccessLevel::Paid | UserAccessLevel::Premium => RiskManagementConfig {
                max_daily_loss_percent: 10.0,
                max_drawdown_percent: 20.0,
                position_sizing_method: PositionSizingMethod::PercentageOfPortfolio,
                stop_loss_required: true,
                take_profit_recommended: true,
                trailing_stop_enabled: true,
                risk_reward_ratio_min: 1.2,
            },
            _ => RiskManagementConfig {
                max_daily_loss_percent: 5.0,
                max_drawdown_percent: 10.0,
                position_sizing_method: PositionSizingMethod::FixedAmount,
                stop_loss_required: true,
                take_profit_recommended: true,
                trailing_stop_enabled: false,
                risk_reward_ratio_min: 2.0,
            },
        }
    }

    /// Validate trade execution request
    pub fn validate_trade_execution(
        &self,
        session_id: &str,
        trade_request: &TradeExecutionRequest,
    ) -> Result<TradeValidationResult, String> {
        let session = self
            .trading_sessions
            .get(session_id)
            .ok_or_else(|| "Trading session not found".to_string())?;

        if session.user_id != trade_request.user_id {
            return Err("User ID mismatch with trading session".to_string());
        }

        let mut validation_errors = Vec::new();
        let mut warnings = Vec::new();

        // Check trading permissions
        if trade_request.is_auto_trade && !session.auto_trading_enabled {
            validation_errors.push("Auto trading not enabled for this role".to_string());
        }

        if !trade_request.is_auto_trade && !session.manual_trading_enabled {
            validation_errors.push("Manual trading not enabled for this role".to_string());
        }

        // Check concurrent trades limit
        if session.active_trades >= session.max_concurrent_trades {
            validation_errors.push(format!(
                "Maximum concurrent trades ({}) reached",
                session.max_concurrent_trades
            ));
        }

        // Check leverage limits
        if trade_request.leverage > session.max_leverage {
            validation_errors.push(format!(
                "Leverage ({}) exceeds maximum allowed ({})",
                trade_request.leverage, session.max_leverage
            ));
        }

        // Calculate position size percentage (simplified calculation)
        let position_size_percent = (trade_request.quantity * trade_request.leverage) / 10000.0; // Assuming portfolio size

        // Check position size limits
        if session.total_position_size_percent + position_size_percent
            > session.max_position_size_percent
        {
            validation_errors.push(format!(
                "Position size would exceed maximum allowed ({}%)",
                session.max_position_size_percent
            ));
        }

        // Check risk management requirements
        if session.risk_management.stop_loss_required && trade_request.stop_loss.is_none() {
            validation_errors.push("Stop loss is required for this role".to_string());
        }

        if session.risk_management.take_profit_recommended && trade_request.take_profit.is_none() {
            warnings.push("Take profit is recommended for better risk management".to_string());
        }

        // Calculate risk assessment
        let risk_assessment =
            self.calculate_risk_assessment(session, trade_request, position_size_percent);

        // Check risk score
        if risk_assessment.risk_score > 0.8 {
            warnings.push("High risk trade detected".to_string());
        }

        if risk_assessment.risk_score > 0.95 {
            validation_errors.push("Risk score too high for execution".to_string());
        }

        let is_valid = validation_errors.is_empty();
        let can_execute = is_valid;

        Ok(TradeValidationResult {
            is_valid,
            can_execute,
            validation_errors,
            warnings,
            risk_assessment,
        })
    }

    /// Calculate risk assessment for trade
    fn calculate_risk_assessment(
        &self,
        session: &TradingSessionConfig,
        trade_request: &TradeExecutionRequest,
        position_size_percent: f64,
    ) -> RiskAssessment {
        let leverage_utilization = trade_request.leverage / session.max_leverage;
        let concurrent_trades_utilization =
            (session.active_trades + 1) as f64 / session.max_concurrent_trades as f64;
        let position_utilization = (session.total_position_size_percent + position_size_percent)
            / session.max_position_size_percent;

        // Calculate estimated max loss (simplified)
        let estimated_max_loss = if let Some(stop_loss) = trade_request.stop_loss {
            let price = trade_request.price.unwrap_or(100.0); // Default price for calculation
            let loss_percent = ((price - stop_loss).abs() / price) * 100.0;
            loss_percent * position_size_percent
        } else {
            position_size_percent * 0.1 // Assume 10% potential loss without stop loss
        };

        // Calculate risk-reward ratio
        let risk_reward_ratio = if let (Some(stop_loss), Some(take_profit)) =
            (trade_request.stop_loss, trade_request.take_profit)
        {
            let price = trade_request.price.unwrap_or(100.0);
            let risk = (price - stop_loss).abs();
            let reward = (take_profit - price).abs();
            if risk > 0.0 {
                Some(reward / risk)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate overall risk score (0.0 to 1.0)
        let risk_score = (leverage_utilization * 0.3
            + concurrent_trades_utilization * 0.2
            + position_utilization * 0.3
            + (estimated_max_loss / 100.0) * 0.2)
            .min(1.0);

        RiskAssessment {
            risk_score,
            position_size_percent,
            leverage_utilization,
            concurrent_trades_utilization,
            estimated_max_loss,
            risk_reward_ratio,
        }
    }

    /// Execute trade (update session state)
    pub fn execute_trade(
        &mut self,
        session_id: &str,
        trade_request: &TradeExecutionRequest,
    ) -> Result<String, String> {
        // Validate trade first
        let validation = self.validate_trade_execution(session_id, trade_request)?;

        if !validation.can_execute {
            return Err(format!(
                "Trade validation failed: {:?}",
                validation.validation_errors
            ));
        }

        let session = self
            .trading_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Trading session not found".to_string())?;

        // Update session state
        session.active_trades += 1;
        session.current_leverage = session.current_leverage.max(trade_request.leverage);
        session.total_position_size_percent += validation.risk_assessment.position_size_percent;
        session.last_activity = Utc::now().timestamp_millis() as u64;

        let trade_id = format!("trade_{}_{}", session_id, session.active_trades);

        console_log!(
            "✅ Executed trade '{}' for user: {} in session: {}",
            trade_id,
            trade_request.user_id,
            session_id
        );

        if !validation.warnings.is_empty() {
            console_log!("⚠️ Trade warnings: {:?}", validation.warnings);
        }

        Ok(trade_id)
    }

    /// Close trade (update session state)
    pub fn close_trade(
        &mut self,
        session_id: &str,
        trade_id: &str,
        position_size_percent: f64,
    ) -> Result<(), String> {
        let session = self
            .trading_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Trading session not found".to_string())?;

        if session.active_trades == 0 {
            return Err("No active trades to close".to_string());
        }

        session.active_trades -= 1;
        session.total_position_size_percent =
            (session.total_position_size_percent - position_size_percent).max(0.0);
        session.last_activity = Utc::now().timestamp_millis() as u64;

        console_log!("✅ Closed trade '{}' in session: {}", trade_id, session_id);

        Ok(())
    }

    /// Get trading session
    pub fn get_trading_session(&self, session_id: &str) -> Option<&TradingSessionConfig> {
        self.trading_sessions.get(session_id)
    }

    /// Update trading session configuration
    pub fn update_session_config(
        &mut self,
        session_id: &str,
        risk_management: Option<RiskManagementConfig>,
    ) -> Result<(), String> {
        let session = self
            .trading_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Trading session not found".to_string())?;

        if let Some(risk_config) = risk_management {
            session.risk_management = risk_config;
        }

        session.last_activity = Utc::now().timestamp_millis() as u64;

        console_log!("🔧 Updated trading session configuration: {}", session_id);

        Ok(())
    }

    /// Get user trading sessions
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<&TradingSessionConfig> {
        self.trading_sessions
            .values()
            .filter(|session| session.user_id == user_id)
            .collect()
    }

    /// Close trading session
    pub fn close_session(&mut self, session_id: &str) -> Result<(), String> {
        let session = self
            .trading_sessions
            .remove(session_id)
            .ok_or_else(|| "Trading session not found".to_string())?;

        console_log!(
            "🔒 Closed trading session '{}' for user: {}",
            session_id,
            session.user_id
        );

        Ok(())
    }

    /// Get trading statistics for user
    pub fn get_user_trading_stats(&self, user_id: &str) -> UserTradingStats {
        let sessions: Vec<&TradingSessionConfig> = self.get_user_sessions(user_id);

        let total_sessions = sessions.len() as u32;
        let active_sessions = sessions.iter().filter(|s| s.active_trades > 0).count() as u32;
        let total_active_trades = sessions.iter().map(|s| s.active_trades).sum();
        let max_concurrent_trades = sessions
            .iter()
            .map(|s| s.max_concurrent_trades)
            .max()
            .unwrap_or(0);
        let current_leverage = sessions
            .iter()
            .map(|s| s.current_leverage)
            .fold(0.0, f64::max);
        let max_leverage = sessions.iter().map(|s| s.max_leverage).fold(0.0, f64::max);

        UserTradingStats {
            user_id: user_id.to_string(),
            total_sessions,
            active_sessions,
            total_active_trades,
            max_concurrent_trades,
            current_leverage,
            max_leverage,
            last_activity: sessions.iter().map(|s| s.last_activity).max().unwrap_or(0),
        }
    }
}

/// User trading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradingStats {
    pub user_id: String,
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub total_active_trades: u32,
    pub max_concurrent_trades: u32,
    pub current_leverage: f64,
    pub max_leverage: f64,
    pub last_activity: u64,
}

impl Default for TradingConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
