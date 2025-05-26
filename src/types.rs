// src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
// UUID is used throughout the file as uuid::Uuid::new_v4()
// Keeping the full path for clarity

// use thiserror::Error; // TODO: Re-enable when implementing custom error types

/// Exchange identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::upper_case_acronyms)]
pub enum ExchangeIdEnum {
    Binance,
    Bybit,
    OKX,
    Bitget,
    // Add other exchanges as needed
}

impl ExchangeIdEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeIdEnum::Binance => "binance",
            ExchangeIdEnum::Bybit => "bybit",
            ExchangeIdEnum::OKX => "okx",
            ExchangeIdEnum::Bitget => "bitget",
        }
    }

    /// Get all supported exchanges
    pub fn all_supported() -> Vec<ExchangeIdEnum> {
        vec![
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
            ExchangeIdEnum::Bitget,
        ]
    }
}

impl std::fmt::Display for ExchangeIdEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ExchangeIdEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "binance" => Ok(ExchangeIdEnum::Binance),
            "bybit" => Ok(ExchangeIdEnum::Bybit),
            "okx" => Ok(ExchangeIdEnum::OKX),
            "bitget" => Ok(ExchangeIdEnum::Bitget),
            _ => Err(format!("Unknown exchange: {}", s)),
        }
    }
}

// AI Provider enum for different AI services
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyProvider {
    OpenAI,
    Anthropic,
    Custom,
    Exchange(ExchangeIdEnum), // For exchange API keys
}

impl std::fmt::Display for ApiKeyProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyProvider::OpenAI => write!(f, "openai"),
            ApiKeyProvider::Anthropic => write!(f, "anthropic"),
            ApiKeyProvider::Custom => write!(f, "custom"),
            ApiKeyProvider::Exchange(exchange) => write!(f, "exchange_{}", exchange),
        }
    }
}

// String alias for exchange identifiers (for compatibility with CCXT-like interface)
pub type ExchangeId = String;
pub type TradingPairSymbol = String;

/// Types of arbitrage opportunities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArbitrageType {
    FundingRate,
    SpotFutures,
    CrossExchange,
}

/// Core arbitrage opportunity structure
/// **POSITION STRUCTURE**: Requires exactly 2 exchanges (long + short)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub pair: String,
    pub long_exchange: ExchangeIdEnum, // **REQUIRED**: Long position exchange
    pub short_exchange: ExchangeIdEnum, // **REQUIRED**: Short position exchange
    pub long_rate: Option<f64>,
    pub short_rate: Option<f64>,
    pub rate_difference: f64,
    pub net_rate_difference: Option<f64>,
    pub potential_profit_value: Option<f64>,
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub r#type: ArbitrageType,
    pub details: Option<String>,
    pub min_exchanges_required: u8, // **ALWAYS 2** for arbitrage
}

impl Default for ArbitrageOpportunity {
    fn default() -> Self {
        Self {
            id: String::new(),
            pair: "BTCUSDT".to_string(), // Default fallback pair
            long_exchange: ExchangeIdEnum::Binance,
            short_exchange: ExchangeIdEnum::Bybit,
            long_rate: None,
            short_rate: None,
            rate_difference: 0.0,
            net_rate_difference: None,
            potential_profit_value: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            r#type: ArbitrageType::CrossExchange,
            details: None,
            min_exchanges_required: 2,
        }
    }
}

impl ArbitrageOpportunity {
    pub fn new(
        pair: String,
        long_exchange: ExchangeIdEnum, // **REQUIRED**: No longer optional
        short_exchange: ExchangeIdEnum, // **REQUIRED**: No longer optional
        long_rate: Option<f64>,
        short_rate: Option<f64>,
        rate_difference: f64,
        r#type: ArbitrageType,
    ) -> Result<Self, String> {
        // Validate trading pair is not empty
        if pair.trim().is_empty() {
            return Err("Trading pair cannot be empty".to_string());
        }

        Ok(Self {
            id: String::new(),
            pair,
            long_exchange,
            short_exchange,
            long_rate,
            short_rate,
            rate_difference,
            net_rate_difference: None,
            potential_profit_value: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            r#type,
            details: None,
            min_exchanges_required: 2, // **ALWAYS 2** for arbitrage
        })
    }

    pub fn with_net_difference(mut self, net_rate_difference: f64) -> Self {
        self.net_rate_difference = Some(net_rate_difference);
        self
    }

    pub fn with_potential_profit(mut self, potential_profit_value: f64) -> Self {
        self.potential_profit_value = Some(potential_profit_value);
        self
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Validate that this arbitrage opportunity has exactly 2 exchanges
    /// **POSITION STRUCTURE VALIDATION**
    pub fn validate_position_structure(&self) -> Result<(), String> {
        if self.long_exchange == self.short_exchange {
            return Err("Arbitrage opportunity cannot use the same exchange for both long and short positions".to_string());
        }

        if self.min_exchanges_required != 2 {
            return Err("Arbitrage opportunity must require exactly 2 exchanges".to_string());
        }

        Ok(())
    }

    /// Get required exchanges for this arbitrage opportunity
    pub fn get_required_exchanges(&self) -> Vec<ExchangeIdEnum> {
        vec![self.long_exchange, self.short_exchange]
    }
}

/// Technical analysis opportunity structure
/// **POSITION STRUCTURE**: Requires exactly 1 exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalOpportunity {
    pub id: String,
    pub pair: String,
    pub exchange: ExchangeIdEnum, // **REQUIRED**: Single exchange
    pub signal_type: TechnicalSignalType,
    pub signal_strength: TechnicalSignalStrength,
    pub entry_price: f64,
    pub target_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub confidence_score: f64,             // 0.0 to 1.0
    pub technical_indicators: Vec<String>, // RSI, MACD, SMA, etc.
    pub timeframe: String,                 // 1m, 5m, 1h, 4h, 1d
    pub expected_return_percentage: f64,
    pub risk_level: TechnicalRiskLevel,
    pub timestamp: u64,
    pub expires_at: u64,
    pub details: Option<String>,
    pub min_exchanges_required: u8, // **ALWAYS 1** for technical
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalSignalType {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalSignalStrength {
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalRiskLevel {
    Low,
    Medium,
    High,
}

impl TechnicalOpportunity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pair: String,
        exchange: ExchangeIdEnum, // **REQUIRED**: Single exchange
        signal_type: TechnicalSignalType,
        signal_strength: TechnicalSignalStrength,
        entry_price: f64,
        confidence_score: f64,
        technical_indicators: Vec<String>,
        timeframe: String,
        expected_return_percentage: f64,
        risk_level: TechnicalRiskLevel,
        expires_at: u64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pair,
            exchange,
            signal_type,
            signal_strength,
            entry_price,
            target_price: None,
            stop_loss_price: None,
            confidence_score,
            technical_indicators,
            timeframe,
            expected_return_percentage,
            risk_level,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            expires_at,
            details: None,
            min_exchanges_required: 1, // **ALWAYS 1** for technical
        }
    }

    pub fn with_target_price(mut self, target_price: f64) -> Self {
        self.target_price = Some(target_price);
        self
    }

    pub fn with_stop_loss(mut self, stop_loss_price: f64) -> Self {
        self.stop_loss_price = Some(stop_loss_price);
        self
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Validate that this technical opportunity has exactly 1 exchange
    /// **POSITION STRUCTURE VALIDATION**
    pub fn validate_position_structure(&self) -> Result<(), String> {
        if self.min_exchanges_required != 1 {
            return Err("Technical opportunity must require exactly 1 exchange".to_string());
        }

        Ok(())
    }

    /// Get required exchanges for this technical opportunity
    pub fn get_required_exchanges(&self) -> Vec<ExchangeIdEnum> {
        vec![self.exchange]
    }

    /// Check if opportunity is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now > self.expires_at
    }

    /// Calculate profit potential based on target price
    pub fn calculate_profit_potential(&self) -> Option<f64> {
        self.target_price.map(|target| match self.signal_type {
            TechnicalSignalType::Buy => (target - self.entry_price) / self.entry_price * 100.0,
            TechnicalSignalType::Sell => (self.entry_price - target) / self.entry_price * 100.0,
            TechnicalSignalType::Hold => 0.0,
        })
    }
}

/// Exchange rate data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub exchange: ExchangeIdEnum,
    pub pair: String,
    pub rate: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionStatus {
    Open,
    Closed,
    Pending,
}

/// Position data for tracking open positions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbitragePosition {
    pub id: String,
    pub exchange: ExchangeIdEnum,
    pub pair: String,
    pub side: PositionSide,
    pub size: f64,
    pub entry_price: f64,
    pub current_price: Option<f64>,
    pub pnl: Option<f64>,
    pub status: PositionStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub calculated_size_usd: Option<f64>,
    pub risk_percentage_applied: Option<f64>,

    // Advanced Risk Management Fields (Task 6)
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub trailing_stop_distance: Option<f64>, // Distance in price points for trailing stop
    pub max_loss_usd: Option<f64>,           // Maximum acceptable loss in USD
    pub risk_reward_ratio: Option<f64>,      // Target risk/reward ratio

    // Multi-Exchange Position Tracking (Task 6)
    pub related_positions: Vec<String>, // IDs of related positions on other exchanges
    pub hedge_position_id: Option<String>, // ID of hedge position if this is part of arbitrage
    pub position_group_id: Option<String>, // Group ID for coordinated multi-exchange positions

    // Position Optimization (Task 6)
    pub optimization_score: Option<f64>, // AI-calculated optimization score
    pub recommended_action: Option<PositionAction>, // AI recommendation
    pub last_optimization_check: Option<u64>, // Timestamp of last optimization analysis

    // Advanced Metrics (Task 6)
    pub max_drawdown: Option<f64>, // Maximum drawdown since position opened
    pub unrealized_pnl_percentage: Option<f64>, // PnL as percentage of entry value
    pub holding_period_hours: Option<f64>, // How long position has been held
    pub volatility_score: Option<f64>, // Calculated volatility of the position
}

/// Represents basic account information for sizing calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub total_balance_usd: f64,
    // Add other relevant fields like available_margin, etc. in the future
}

/// Configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub environment: String,
    pub usdt_amount: f64,
    pub bybit_leverage: u32,
    pub binance_leverage: u32,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            usdt_amount: 10.0,
            bybit_leverage: 20,
            binance_leverage: 20,
            log_level: "info".to_string(),
        }
    }
}

// Exchange trading types (CCXT-like interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TrailingStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    Closed,
    Canceled,
    Expired,
    Rejected,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub active: bool,
    pub precision: Precision,
    pub limits: Limits,
    pub fees: Option<TradingFee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precision {
    pub amount: Option<i32>,
    pub price: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub amount: MinMax,
    pub price: MinMax,
    pub cost: MinMax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMax {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub last: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub volume: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub datetime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<[f64; 2]>, // [price, amount]
    pub asks: Vec<[f64; 2]>, // [price, amount]
    pub timestamp: Option<DateTime<Utc>>,
    pub datetime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub free: f64,
    pub used: f64,
    pub total: f64,
}

pub type Balances = HashMap<String, Balance>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub r#type: OrderType,
    pub side: OrderSide,
    pub amount: f64,
    pub price: Option<f64>,
    pub cost: Option<f64>,
    pub filled: f64,
    pub remaining: f64,
    pub status: OrderStatus,
    pub timestamp: Option<DateTime<Utc>>,
    pub datetime: Option<String>,
    pub fee: Option<Fee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Option<String>,
    pub symbol: String,
    pub side: PositionSide,
    pub size: f64,
    pub notional: f64,
    pub entry_price: f64,
    pub mark_price: Option<f64>,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub leverage: f64,
    pub margin: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub datetime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub currency: String,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingFee {
    pub maker: f64,
    pub taker: f64,
    pub percentage: bool,
}

pub type TradingFeeInterface = TradingFee;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRateInfo {
    pub symbol: String,
    pub funding_rate: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub datetime: Option<String>,
    pub next_funding_time: Option<DateTime<Utc>>,
    pub estimated_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCredentials {
    pub api_key: String,
    pub secret: String,
    pub default_leverage: i32,
    pub exchange_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredTradingPair {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub exchange_id: String,
}

// Environment and configuration types
pub struct LoggerInterface {
    // Implementation will be in logger.rs
}

pub struct Env {
    // Worker environment interface containing the full environment
    pub worker_env: worker::Env,
}

impl Env {
    pub fn new(worker_env: worker::Env) -> Self {
        Self { worker_env }
    }

    pub fn get_kv_store(&self, binding_name: &str) -> Option<worker::kv::KvStore> {
        self.worker_env.kv(binding_name).ok()
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum ExchangeError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Order not found: {0}")]
    OrderNotFound(String),
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Exchange not supported: {0}")]
    ExchangeNotSupported(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type ExchangeResult<T> = Result<T, ExchangeError>;

// User Profile and Subscription System
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Basic,
    Premium,
    Enterprise,
    SuperAdmin, // Super admin with system management access
}

/// User roles for RBAC (Role-Based Access Control)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,       // Regular user (Free, Basic, Premium, Enterprise)
    SuperAdmin, // Super administrator with system access
    BetaUser,   // Beta user (all features during beta period)
}

/// Command permissions for RBAC
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandPermission {
    // Basic commands (available to all)
    BasicCommands,

    // Trading commands (subscription-gated in future)
    ManualTrading,
    AutomatedTrading,

    // Opportunity access levels
    BasicOpportunities,      // Global arbitrage only
    TechnicalAnalysis,       // Global + technical analysis
    AIEnhancedOpportunities, // Premium AI features

    // Admin commands (super admin only)
    SystemAdministration,
    UserManagement,
    GlobalConfiguration,
    GroupAnalytics, // Group/channel analytics access

    // Future subscription-gated features
    AdvancedAnalytics,
    PremiumFeatures,
}

/// Trading modes for users
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradingMode {
    Manual,    // User executes trades manually with API keys
    Automated, // Bot executes trades automatically
    Advisory,  // Bot provides signals, user decides
    Disabled,  // No trading functionality
}

/// Analytics tracking for bot usage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageAnalytics {
    pub message_id: String,
    pub user_id: Option<String>,
    pub chat_id: String,
    pub chat_type: String,       // "private", "group", "supergroup", "channel"
    pub message_type: String,    // "command", "opportunity", "broadcast", "response"
    pub command: Option<String>, // Command name if applicable
    pub content_type: String,    // "global_arbitrage", "technical_analysis", "ai_enhanced", etc.
    pub delivery_status: String, // "sent", "delivered", "failed", "rate_limited"
    pub response_time_ms: Option<u64>, // Time to generate response
    pub timestamp: u64,
    pub metadata: serde_json::Value, // Additional tracking data
}

/// Group/Channel registration and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRegistration {
    pub group_id: String,
    pub group_type: String, // "group", "supergroup", "channel"
    pub group_title: Option<String>,
    pub group_username: Option<String>,
    pub member_count: Option<u32>,
    pub admin_user_ids: Vec<String>,   // Telegram user IDs of admins
    pub bot_permissions: Vec<String>,  // What the bot can do in this group
    pub enabled_features: Vec<String>, // Which features are enabled
    pub global_opportunities_enabled: bool,
    pub technical_analysis_enabled: bool,
    pub rate_limit_config: GroupRateLimitConfig,
    pub registered_at: u64,
    pub last_activity: u64,
    pub total_messages_sent: u32,
    pub last_member_count_update: Option<u64>,
}

/// Rate limiting configuration for groups/channels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRateLimitConfig {
    pub max_opportunities_per_hour: u32,
    pub max_technical_signals_per_hour: u32,
    pub max_broadcasts_per_day: u32,
    pub cooldown_between_messages_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionInfo {
    pub tier: SubscriptionTier,
    pub is_active: bool,
    pub expires_at: Option<u64>, // Unix timestamp in milliseconds
    pub created_at: u64,
    pub features: Vec<String>, // List of enabled features
}

impl Default for SubscriptionInfo {
    fn default() -> Self {
        Self {
            tier: SubscriptionTier::Free,
            is_active: true,
            expires_at: None, // Free tier doesn't expire
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            features: vec!["basic_arbitrage".to_string(), "manual_trading".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConfiguration {
    pub max_leverage: u32,
    pub max_entry_size_usdt: f64,
    pub min_entry_size_usdt: f64,
    pub risk_tolerance_percentage: f64, // 0.0 to 1.0
    pub opportunity_threshold: f64,     // Minimum rate difference to consider
    pub auto_trading_enabled: bool,
    pub notification_preferences: NotificationPreferences,
    pub trading_pairs: Vec<String>,  // Monitored trading pairs
    pub excluded_pairs: Vec<String>, // Excluded trading pairs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    pub push_opportunities: bool,
    pub push_executions: bool,
    pub push_risk_alerts: bool,
    pub push_system_status: bool,
    pub min_profit_threshold_usdt: f64, // Only notify if potential profit > this
    pub max_notifications_per_hour: u32,
}

impl Default for UserConfiguration {
    fn default() -> Self {
        Self {
            max_leverage: 10,
            max_entry_size_usdt: 1000.0,
            min_entry_size_usdt: 50.0,
            risk_tolerance_percentage: 0.02, // 2%
            opportunity_threshold: 0.001,    // 0.1%
            auto_trading_enabled: false,
            notification_preferences: NotificationPreferences::default(),
            trading_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            excluded_pairs: vec![],
        }
    }
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            push_opportunities: true,
            push_executions: true,
            push_risk_alerts: true,
            push_system_status: false,
            min_profit_threshold_usdt: 1.0,
            max_notifications_per_hour: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserApiKey {
    pub id: String,                       // Unique identifier for this API key
    pub user_id: String,                  // User who owns this key
    pub provider: ApiKeyProvider,         // Which service this key is for
    pub encrypted_key: String,            // Encrypted API key
    pub encrypted_secret: Option<String>, // Optional secret (for exchanges)
    pub metadata: serde_json::Value,      // Additional configuration (models, base_urls, etc.)
    pub is_active: bool,
    pub is_read_only: bool, // Whether this API key is read-only
    pub created_at: u64,
    pub last_used: Option<u64>,
    pub permissions: Vec<String>, // e.g., ["read", "trade", "futures"] for exchanges, ["chat", "analysis"] for AI
}

impl UserApiKey {
    pub fn new_exchange_key(
        user_id: String,
        exchange: ExchangeIdEnum,
        encrypted_api_key: String,
        encrypted_secret: String,
        permissions: Vec<String>,
    ) -> Self {
        // Determine if read-only based on permissions
        let is_read_only = !permissions.contains(&"trade".to_string());

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            provider: ApiKeyProvider::Exchange(exchange),
            encrypted_key: encrypted_api_key,
            encrypted_secret: Some(encrypted_secret),
            metadata: serde_json::json!({}),
            is_active: true,
            is_read_only,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            last_used: None,
            permissions,
        }
    }

    pub fn new_ai_key(
        user_id: String,
        provider: ApiKeyProvider,
        encrypted_api_key: String,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            provider,
            encrypted_key: encrypted_api_key,
            encrypted_secret: None,
            metadata,
            is_active: true,
            is_read_only: true, // AI keys are always read-only for trading purposes
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            last_used: None,
            permissions: vec!["analysis".to_string(), "chat".to_string()],
        }
    }

    pub fn is_exchange_key(&self) -> bool {
        matches!(self.provider, ApiKeyProvider::Exchange(_))
    }

    pub fn is_ai_key(&self) -> bool {
        matches!(
            self.provider,
            ApiKeyProvider::OpenAI | ApiKeyProvider::Anthropic | ApiKeyProvider::Custom
        )
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Some(chrono::Utc::now().timestamp_millis() as u64);
    }
}

// Keep the old structure for backward compatibility during migration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyUserApiKey {
    pub exchange: ExchangeIdEnum,
    pub api_key_encrypted: String, // Encrypted with user-specific key
    pub secret_encrypted: String,  // Encrypted with user-specific key
    pub is_active: bool,
    pub created_at: u64,
    pub last_validated: Option<u64>,
    pub permissions: Vec<String>, // e.g., ["read", "trade", "futures"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: String, // Primary identifier
    pub telegram_user_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub subscription: SubscriptionInfo,
    pub configuration: UserConfiguration,
    pub api_keys: Vec<UserApiKey>,
    pub invitation_code: Option<String>, // Code used to join
    pub beta_expires_at: Option<u64>, // Beta access expiration timestamp (180 days from invitation)
    pub created_at: u64,
    pub updated_at: u64,
    pub last_active: u64,
    pub is_active: bool,
    pub total_trades: u32,
    pub total_pnl_usdt: f64,
    pub account_balance_usdt: f64, // Actual account balance for trading
    pub profile_metadata: Option<serde_json::Value>, // Additional profile metadata including role
}

impl UserProfile {
    pub fn new(telegram_user_id: Option<i64>, invitation_code: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let user_id = uuid::Uuid::new_v4().to_string();

        Self {
            user_id,
            telegram_user_id,
            telegram_username: None,
            subscription: SubscriptionInfo::default(),
            configuration: UserConfiguration::default(),
            api_keys: vec![],
            invitation_code,
            beta_expires_at: None, // Will be set when invitation code is used
            created_at: now,
            updated_at: now,
            last_active: now,
            is_active: true,
            total_trades: 0,
            total_pnl_usdt: 0.0,
            account_balance_usdt: 0.0, // Default balance
            profile_metadata: None,
        }
    }

    /// Get user role for RBAC
    pub fn get_user_role(&self) -> UserRole {
        // First check profile_metadata.role if available
        if let Some(metadata) = &self.profile_metadata {
            if let Some(role) = metadata.get("role") {
                if let Some("superadmin") = role.as_str() {
                    return UserRole::SuperAdmin;
                }
            }
        }

        // Fall back to subscription tier-based role determination
        match self.subscription.tier {
            SubscriptionTier::SuperAdmin => UserRole::SuperAdmin,
            _ => UserRole::User, // Regular user for all other tiers
        }
    }

    /// Check if user has permission for a specific command
    pub fn has_permission(&self, permission: CommandPermission) -> bool {
        let user_role = self.get_user_role();

        // Check if beta access has expired
        let has_active_beta = self.has_active_beta_access();

        match permission {
            CommandPermission::BasicCommands | CommandPermission::BasicOpportunities => true, // Everyone has basic access

            CommandPermission::ManualTrading => {
                // Beta users have access if beta hasn't expired
                if has_active_beta {
                    true
                } else {
                    // Non-beta users need Premium+ subscription
                    matches!(
                        self.subscription.tier,
                        SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                            | SubscriptionTier::SuperAdmin
                    )
                }
            }

            CommandPermission::TechnicalAnalysis => {
                // Beta users have access if beta hasn't expired
                if has_active_beta {
                    true
                } else {
                    // Non-beta users need Premium+ subscription
                    matches!(
                        self.subscription.tier,
                        SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                            | SubscriptionTier::SuperAdmin
                    )
                }
            }

            CommandPermission::AIEnhancedOpportunities => {
                // Beta users have access if beta hasn't expired
                if has_active_beta {
                    true
                } else {
                    // Non-beta users need Premium+ subscription
                    matches!(
                        self.subscription.tier,
                        SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                            | SubscriptionTier::SuperAdmin
                    )
                }
            }

            CommandPermission::AutomatedTrading => {
                // Beta users have access if beta hasn't expired
                if has_active_beta {
                    true
                } else {
                    // Non-beta users need Premium+ subscription
                    matches!(
                        self.subscription.tier,
                        SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                            | SubscriptionTier::SuperAdmin
                    )
                }
            }

            CommandPermission::SystemAdministration
            | CommandPermission::UserManagement
            | CommandPermission::GlobalConfiguration
            | CommandPermission::GroupAnalytics => {
                // Only super admins can access admin commands, regardless of beta status
                user_role == UserRole::SuperAdmin
            }

            CommandPermission::AdvancedAnalytics | CommandPermission::PremiumFeatures => {
                // Beta users have access if beta hasn't expired
                if has_active_beta {
                    true
                } else {
                    // Non-beta users need Premium+ subscription
                    matches!(
                        self.subscription.tier,
                        SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                            | SubscriptionTier::SuperAdmin
                    )
                }
            }
        }
    }

    /// Get user's trading mode
    pub fn get_trading_mode(&self) -> TradingMode {
        // TODO: Store this in user configuration
        // For now, default to manual trading
        TradingMode::Manual
    }

    /// Check if user has API keys with trading permissions
    pub fn has_trading_api_keys(&self) -> bool {
        self.api_keys.iter().any(|key| {
            key.is_active && key.is_exchange_key() && key.permissions.contains(&"trade".to_string())
        })
    }

    /// Check if user can use manual trading
    pub fn can_use_manual_trading(&self) -> bool {
        self.has_permission(CommandPermission::ManualTrading) && self.has_trading_api_keys()
    }

    /// Check if user can use automated trading
    pub fn can_use_automated_trading(&self) -> bool {
        self.has_permission(CommandPermission::AutomatedTrading)
            && self.has_trading_api_keys()
            && matches!(
                self.get_trading_mode(),
                TradingMode::Automated | TradingMode::Advisory
            )
    }

    /// Check if user is super admin
    pub fn is_super_admin(&self) -> bool {
        self.subscription.tier == SubscriptionTier::SuperAdmin
    }

    /// Create super admin user profile
    pub fn new_super_admin(telegram_user_id: Option<i64>) -> Self {
        let mut profile = Self::new(telegram_user_id, None);
        profile.subscription.tier = SubscriptionTier::SuperAdmin;
        profile
            .subscription
            .features
            .push("super_admin_access".to_string());
        profile
            .subscription
            .features
            .push("system_administration".to_string());
        profile
            .subscription
            .features
            .push("user_management".to_string());
        profile
            .subscription
            .features
            .push("global_configuration".to_string());
        profile
    }

    pub fn update_last_active(&mut self) {
        self.last_active = chrono::Utc::now().timestamp_millis() as u64;
        self.updated_at = self.last_active;
    }

    pub fn add_api_key(&mut self, api_key: UserApiKey) {
        // Remove existing key for same provider if present
        self.api_keys.retain(|key| key.provider != api_key.provider);
        self.api_keys.push(api_key);
        self.updated_at = chrono::Utc::now().timestamp_millis() as u64;
    }

    pub fn remove_api_key(&mut self, exchange: &ExchangeIdEnum) -> bool {
        let initial_len = self.api_keys.len();
        self.api_keys.retain(|key| {
            if let ApiKeyProvider::Exchange(key_exchange) = &key.provider {
                key_exchange != exchange
            } else {
                true // Keep non-exchange keys
            }
        });
        if self.api_keys.len() < initial_len {
            self.updated_at = chrono::Utc::now().timestamp_millis() as u64;
            true
        } else {
            false
        }
    }

    pub fn get_active_exchanges(&self) -> Vec<ExchangeIdEnum> {
        self.api_keys
            .iter()
            .filter(|key| key.is_active)
            .filter_map(|key| {
                if let ApiKeyProvider::Exchange(exchange) = &key.provider {
                    Some(*exchange)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_minimum_exchanges(&self) -> bool {
        self.get_active_exchanges().len() >= 2
    }

    pub fn can_trade(&self) -> bool {
        self.is_active && self.subscription.is_active && self.has_minimum_exchanges()
    }

    /// Check if user has active beta access (hasn't expired)
    pub fn has_active_beta_access(&self) -> bool {
        if let Some(beta_expires_at) = self.beta_expires_at {
            let now = chrono::Utc::now().timestamp_millis() as u64;
            now < beta_expires_at
        } else {
            false // No beta access if no expiration date set
        }
    }

    /// Set beta expiration date (called when invitation code is used)
    pub fn set_beta_expiration(&mut self, expires_at: u64) {
        self.beta_expires_at = Some(expires_at);
        self.updated_at = chrono::Utc::now().timestamp_millis() as u64;
    }

    /// Check if beta access has expired and user needs downgrade
    pub fn needs_beta_downgrade(&self) -> bool {
        if let Some(beta_expires_at) = self.beta_expires_at {
            let now = chrono::Utc::now().timestamp_millis() as u64;
            now >= beta_expires_at && self.subscription.tier != SubscriptionTier::Free
        } else {
            false
        }
    }

    /// Downgrade user from beta to free tier
    pub fn downgrade_from_beta(&mut self) {
        if self.needs_beta_downgrade() {
            self.subscription.tier = SubscriptionTier::Free;
            self.subscription.features = vec!["basic_opportunities".to_string()];
            self.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        }
    }

    /// Determine user's access level based on subscription and API keys
    pub fn get_access_level(&self) -> UserAccessLevel {
        // Check if user has any active exchange API keys
        let has_exchange_apis = self
            .api_keys
            .iter()
            .any(|key| key.is_active && key.is_exchange_key());

        match (&self.subscription.tier, has_exchange_apis) {
            // Subscription users with APIs get full access
            (
                SubscriptionTier::Basic | SubscriptionTier::Premium | SubscriptionTier::Enterprise,
                true,
            ) => UserAccessLevel::SubscriptionWithAPI,
            // Free users with APIs get limited access
            (SubscriptionTier::Free, true) => UserAccessLevel::FreeWithAPI,
            // Users without APIs only get view access (regardless of subscription)
            (_, false) => UserAccessLevel::FreeWithoutAPI,
            // SuperAdmin gets full access regardless of APIs
            (SubscriptionTier::SuperAdmin, _) => UserAccessLevel::SubscriptionWithAPI,
        }
    }

    /// Check if user can receive opportunities based on their access level
    pub fn can_receive_opportunities(&self) -> bool {
        let access_level = self.get_access_level();
        match access_level {
            UserAccessLevel::FreeWithoutAPI => false,
            UserAccessLevel::FreeWithAPI | UserAccessLevel::SubscriptionWithAPI => true,
        }
    }

    /// Get user's daily opportunity limits with optional group context
    pub fn get_opportunity_limits(&self, is_group_context: bool) -> (u32, u32) {
        let access_level = self.get_access_level();
        let (base_arbitrage, base_technical) = access_level.get_daily_opportunity_limits();

        // Apply 2x multiplier for group/channel contexts
        if is_group_context {
            (
                base_arbitrage.saturating_mul(2),
                base_technical.saturating_mul(2),
            )
        } else {
            (base_arbitrage, base_technical)
        }
    }

    /// Check if user's exchanges are compatible with an opportunity
    pub fn has_compatible_exchanges(&self, required_exchanges: &[ExchangeIdEnum]) -> bool {
        let user_exchanges = self.get_active_exchanges();
        required_exchanges
            .iter()
            .all(|req_exchange| user_exchanges.contains(req_exchange))
    }

    /// Get user's AI access level based on subscription and AI keys
    pub fn get_ai_access_level(&self) -> AIAccessLevel {
        let has_ai_keys = self
            .api_keys
            .iter()
            .any(|key| key.is_ai_key() && key.is_active);

        match (&self.subscription.tier, has_ai_keys) {
            (SubscriptionTier::Free, false) => AIAccessLevel::FreeWithoutAI {
                ai_analysis: false,
                view_global_ai: true,
                daily_ai_limit: 0,
                template_access: TemplateAccess::None,
            },
            (SubscriptionTier::Free, true) => AIAccessLevel::FreeWithAI {
                ai_analysis: true,
                custom_templates: false,
                daily_ai_limit: 5,
                global_ai_enhancement: true,
                personal_ai_generation: false,
                template_access: TemplateAccess::DefaultOnly,
            },
            (SubscriptionTier::Basic, true) => AIAccessLevel::FreeWithAI {
                ai_analysis: true,
                custom_templates: false,
                daily_ai_limit: 10, // Slightly higher for Basic
                global_ai_enhancement: true,
                personal_ai_generation: false,
                template_access: TemplateAccess::DefaultOnly,
            },
            (
                SubscriptionTier::Premium
                | SubscriptionTier::Enterprise
                | SubscriptionTier::SuperAdmin,
                true,
            ) => AIAccessLevel::SubscriptionWithAI {
                ai_analysis: true,
                custom_templates: true,
                daily_ai_limit: if matches!(self.subscription.tier, SubscriptionTier::SuperAdmin) {
                    u32::MAX
                } else {
                    100
                },
                global_ai_enhancement: true,
                personal_ai_generation: true,
                ai_marketplace: true,
                template_access: TemplateAccess::Full,
            },
            // Users without AI keys but with subscription still get view-only access
            (_, false) => AIAccessLevel::FreeWithoutAI {
                ai_analysis: false,
                view_global_ai: true,
                daily_ai_limit: 0,
                template_access: TemplateAccess::None,
            },
        }
    }

    /// Check if user has AI API keys
    pub fn has_ai_api_keys(&self) -> bool {
        self.api_keys
            .iter()
            .any(|key| key.is_ai_key() && key.is_active)
    }
}

// Invitation System
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationCode {
    pub code: String,
    pub created_by: Option<String>, // User ID who created this code
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub max_uses: Option<u32>,
    pub current_uses: u32,
    pub is_active: bool,
    pub purpose: String, // e.g., "beta_testing", "referral", "admin"
}

impl InvitationCode {
    pub fn new(purpose: String, max_uses: Option<u32>, expires_in_days: Option<u32>) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let code = format!(
            "ARB-{}",
            &uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .to_uppercase()[..8]
        );

        let expires_at = expires_in_days.map(|days| {
            now + (days as u64 * 24 * 60 * 60 * 1000) // Convert days to milliseconds
        });

        Self {
            code,
            created_by: None,
            created_at: now,
            expires_at,
            max_uses,
            current_uses: 0,
            is_active: true,
            purpose,
        }
    }

    pub fn can_be_used(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;

        self.is_active
            && self.expires_at.is_none_or(|exp| now < exp)
            && self.max_uses.is_none_or(|max| self.current_uses < max)
    }

    pub fn use_code(&mut self) -> bool {
        if self.can_be_used() {
            self.current_uses += 1;
            true
        } else {
            false
        }
    }
}

// Trading Session and State Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSession {
    pub user_id: String,
    pub telegram_chat_id: i64,
    pub last_command: Option<String>,
    pub current_state: SessionState,
    pub temporary_data: std::collections::HashMap<String, String>,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    AddingApiKey,
    ConfiguringLeverage,
    ConfiguringEntrySize,
    ConfiguringRisk,
    ExecutingTrade,
    ViewingOpportunities,
}

impl UserSession {
    pub fn new(user_id: String, telegram_chat_id: i64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        Self {
            user_id,
            telegram_chat_id,
            last_command: None,
            current_state: SessionState::Idle,
            temporary_data: std::collections::HashMap::new(),
            created_at: now,
            expires_at: now + (24 * 60 * 60 * 1000), // 24 hours
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        now > self.expires_at
    }

    pub fn extend_session(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        self.expires_at = now + (24 * 60 * 60 * 1000); // Extend by 24 hours
    }
}

// ============= ENHANCED SESSION MANAGEMENT TYPES =============

/// Enhanced session management for comprehensive user lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedUserSession {
    pub session_id: String,
    pub user_id: String,
    pub telegram_id: i64,
    pub session_state: EnhancedSessionState,
    pub started_at: u64,
    pub last_activity_at: u64,
    pub expires_at: u64,
    pub onboarding_completed: bool,
    pub preferences_set: bool,
    pub metadata: Option<serde_json::Value>, // JSON for additional session data
    pub created_at: u64,
    pub updated_at: u64,
}

/// Enhanced session states for comprehensive lifecycle management
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnhancedSessionState {
    Active,
    Expired,
    Terminated,
}

impl EnhancedSessionState {
    pub fn to_db_string(&self) -> &'static str {
        match self {
            EnhancedSessionState::Active => "active",
            EnhancedSessionState::Expired => "expired",
            EnhancedSessionState::Terminated => "terminated",
        }
    }
}

impl EnhancedUserSession {
    pub fn new(user_id: String, telegram_id: i64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        // Use UUID for session ID to prevent collisions
        let session_id = format!("sess_{}_{}", telegram_id, uuid::Uuid::new_v4());

        Self {
            session_id,
            user_id,
            telegram_id,
            session_state: EnhancedSessionState::Active,
            started_at: now,
            last_activity_at: now,
            expires_at: now + (7 * 24 * 60 * 60 * 1000), // 7 days default
            onboarding_completed: false,
            preferences_set: false,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        now > self.expires_at || self.session_state == EnhancedSessionState::Expired
    }

    pub fn is_active(&self) -> bool {
        !self.is_expired() && self.session_state == EnhancedSessionState::Active
    }

    pub fn update_activity(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        self.last_activity_at = now;
        self.updated_at = now;

        // Auto-extend session if it's still active
        if self.session_state == EnhancedSessionState::Active {
            self.expires_at = now + (7 * 24 * 60 * 60 * 1000); // Extend by 7 days
        }
    }

    pub fn complete_onboarding(&mut self) {
        self.onboarding_completed = true;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
    }

    pub fn set_preferences_configured(&mut self) {
        self.preferences_set = true;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
    }

    pub fn terminate(&mut self) {
        self.session_state = EnhancedSessionState::Terminated;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
    }

    pub fn expire(&mut self) {
        self.session_state = EnhancedSessionState::Expired;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
    }

    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = Some(metadata);
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
    }

    pub fn get_session_duration_hours(&self) -> f64 {
        let duration_ms = self.last_activity_at - self.started_at;
        duration_ms as f64 / (60.0 * 60.0 * 1000.0)
    }

    pub fn needs_onboarding(&self) -> bool {
        !self.onboarding_completed
    }

    pub fn needs_preferences_setup(&self) -> bool {
        !self.preferences_set
    }
}

/// Session analytics for tracking user engagement and system performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    pub session_id: String,
    pub user_id: String,
    pub telegram_id: i64,
    pub session_duration_minutes: f64,
    pub commands_executed: u32,
    pub opportunities_viewed: u32,
    pub onboarding_completed: bool,
    pub preferences_configured: bool,
    pub last_command: Option<String>,
    pub session_outcome: SessionOutcome,
    pub created_at: u64,
}

/// Possible outcomes when a session ends
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutcome {
    Completed,  // User completed their intended actions
    Abandoned,  // User left without completing actions
    Expired,    // Session expired due to inactivity
    Terminated, // Session was manually terminated
    Error,      // Session ended due to an error
}

impl SessionOutcome {
    /// Get stable string representation for database storage and API responses
    pub fn to_stable_string(&self) -> &'static str {
        match self {
            SessionOutcome::Completed => "completed",
            SessionOutcome::Abandoned => "abandoned",
            SessionOutcome::Expired => "expired",
            SessionOutcome::Terminated => "terminated",
            SessionOutcome::Error => "error",
        }
    }

    /// Parse from stable string representation
    pub fn from_stable_string(s: &str) -> Result<Self, String> {
        match s {
            "completed" => Ok(SessionOutcome::Completed),
            "abandoned" => Ok(SessionOutcome::Abandoned),
            "expired" => Ok(SessionOutcome::Expired),
            "terminated" => Ok(SessionOutcome::Terminated),
            "error" => Ok(SessionOutcome::Error),
            _ => Err(format!("Invalid session outcome: {}", s)),
        }
    }
}

/// Configuration for session management behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub default_session_duration_hours: u32,
    pub max_session_duration_hours: u32,
    pub activity_extension_hours: u32,
    pub cleanup_interval_hours: u32,
    pub require_onboarding: bool,
    pub require_preferences_setup: bool,
    pub analytics_enabled: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_session_duration_hours: 168, // 7 days
            max_session_duration_hours: 720,     // 30 days
            activity_extension_hours: 168,       // 7 days
            cleanup_interval_hours: 24,          // Daily cleanup
            require_onboarding: true,
            require_preferences_setup: false, // Optional during beta
            analytics_enabled: true,
        }
    }
}

/// Global Opportunity System Types for Task 2
/// Global opportunity with distribution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOpportunity {
    pub opportunity: ArbitrageOpportunity,
    pub detection_timestamp: u64,
    pub expiry_timestamp: u64,
    pub priority_score: f64,           // Higher means more urgent/profitable
    pub distributed_to: Vec<String>,   // User IDs who received this opportunity
    pub max_participants: Option<u32>, // Maximum number of users who can take this opportunity
    pub current_participants: u32,
    pub distribution_strategy: DistributionStrategy,
    pub source: OpportunitySource,
}

/// How opportunities should be distributed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionStrategy {
    FirstComeFirstServe, // Simple queue-based
    RoundRobin,          // Fair rotation among active users
    PriorityBased,       // Based on user subscription tier and activity
    Broadcast,           // Send to all eligible users
}

/// Source of the opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunitySource {
    SystemGenerated, // Generated by default strategy
    UserAI(String),  // Generated by user's AI with user_id
    External,        // From external sources
}

/// Opportunity queue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityQueue {
    pub id: String,
    pub opportunities: Vec<GlobalOpportunity>,
    pub created_at: u64,
    pub updated_at: u64,
    pub total_distributed: u32,
    pub active_users: Vec<String>, // Currently active user IDs
}

/// Distribution tracking per user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpportunityDistribution {
    pub user_id: String,
    pub last_opportunity_received: Option<u64>, // timestamp
    pub total_opportunities_received: u32,
    pub opportunities_today: u32,
    pub last_daily_reset: u64, // timestamp for daily reset
    pub priority_weight: f64,  // User's priority in distribution
    pub is_eligible: bool,     // Whether user can receive opportunities
}

/// Fairness algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessConfig {
    pub rotation_interval_minutes: u32, // How often to rotate in round-robin
    pub max_opportunities_per_user_per_hour: u32,
    pub max_opportunities_per_user_per_day: u32,
    pub tier_multipliers: std::collections::HashMap<String, f64>, // Subscription tier multipliers
    pub activity_boost_factor: f64,                               // Boost for active users
    pub cooldown_period_minutes: u32, // Minimum time between opportunities for same user
}

impl Default for FairnessConfig {
    fn default() -> Self {
        let mut tier_multipliers = std::collections::HashMap::new();
        tier_multipliers.insert("Free".to_string(), 1.0);
        tier_multipliers.insert("Basic".to_string(), 1.5);
        tier_multipliers.insert("Premium".to_string(), 2.0);
        tier_multipliers.insert("Enterprise".to_string(), 3.0);

        Self {
            rotation_interval_minutes: 15,
            max_opportunities_per_user_per_hour: 2, // Updated: max 2 opportunities per cycle
            max_opportunities_per_user_per_day: 10, // Updated: max 10 daily
            tier_multipliers,
            activity_boost_factor: 1.2,
            cooldown_period_minutes: 240, // Updated: 4-hour cooldown (240 minutes)
        }
    }
}

/// Global opportunity detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOpportunityConfig {
    pub detection_interval_seconds: u32,
    pub min_threshold: f64,
    pub max_threshold: f64,
    pub max_queue_size: u32,
    pub opportunity_ttl_minutes: u32, // Time to live for opportunities
    pub distribution_strategy: DistributionStrategy,
    pub fairness_config: FairnessConfig,
    pub monitored_exchanges: Vec<ExchangeIdEnum>,
    pub monitored_pairs: Vec<String>,
}

impl Default for GlobalOpportunityConfig {
    fn default() -> Self {
        Self {
            detection_interval_seconds: 30,
            min_threshold: 0.0005, // 0.05% minimum rate difference
            max_threshold: 0.02,   // 2% maximum rate difference (avoid unrealistic opportunities)
            max_queue_size: 100,
            opportunity_ttl_minutes: 10,
            distribution_strategy: DistributionStrategy::RoundRobin,
            fairness_config: FairnessConfig::default(),
            monitored_exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            monitored_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        }
    }
}

/// Risk tolerance levels for users
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RiskTolerance {
    Low,
    #[default]
    Medium,
    High,
    Custom,
}

impl std::fmt::Display for RiskTolerance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskTolerance::Low => write!(f, "low"),
            RiskTolerance::Medium => write!(f, "medium"),
            RiskTolerance::High => write!(f, "high"),
            RiskTolerance::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for RiskTolerance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(RiskTolerance::Low),
            "medium" => Ok(RiskTolerance::Medium),
            "high" => Ok(RiskTolerance::High),
            "custom" => Ok(RiskTolerance::Custom),
            _ => Err(format!("Invalid risk tolerance: {}", s)),
        }
    }
}

/// Account status for users
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum AccountStatus {
    #[default]
    Active,
    Suspended,
    Pending,
    Disabled,
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "active"),
            AccountStatus::Suspended => write!(f, "suspended"),
            AccountStatus::Pending => write!(f, "pending"),
            AccountStatus::Disabled => write!(f, "disabled"),
        }
    }
}

impl std::str::FromStr for AccountStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(AccountStatus::Active),
            "suspended" => Ok(AccountStatus::Suspended),
            "pending" => Ok(AccountStatus::Pending),
            "disabled" => Ok(AccountStatus::Disabled),
            _ => Err(format!("Invalid account status: {}", s)),
        }
    }
}

/// Email verification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EmailVerificationStatus {
    #[default]
    Pending,
    Verified,
    Failed,
    Expired,
}

impl std::fmt::Display for EmailVerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailVerificationStatus::Pending => write!(f, "pending"),
            EmailVerificationStatus::Verified => write!(f, "verified"),
            EmailVerificationStatus::Failed => write!(f, "failed"),
            EmailVerificationStatus::Expired => write!(f, "expired"),
        }
    }
}

impl std::str::FromStr for EmailVerificationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(EmailVerificationStatus::Pending),
            "verified" => Ok(EmailVerificationStatus::Verified),
            "failed" => Ok(EmailVerificationStatus::Failed),
            "expired" => Ok(EmailVerificationStatus::Expired),
            _ => Err(format!("Invalid email verification status: {}", s)),
        }
    }
}

/// User invitation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum InvitationType {
    Email,
    Telegram,
    Referral,
    #[default]
    Direct,
}

impl std::fmt::Display for InvitationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationType::Email => write!(f, "email"),
            InvitationType::Telegram => write!(f, "telegram"),
            InvitationType::Referral => write!(f, "referral"),
            InvitationType::Direct => write!(f, "direct"),
        }
    }
}

impl std::str::FromStr for InvitationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "email" => Ok(InvitationType::Email),
            "telegram" => Ok(InvitationType::Telegram),
            "referral" => Ok(InvitationType::Referral),
            "direct" => Ok(InvitationType::Direct),
            _ => Err(format!("Invalid invitation type: {}", s)),
        }
    }
}

/// Invitation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum InvitationStatus {
    #[default]
    Pending,
    Accepted,
    Expired,
    Cancelled,
    Failed,
}

impl std::fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "pending"),
            InvitationStatus::Accepted => write!(f, "accepted"),
            InvitationStatus::Expired => write!(f, "expired"),
            InvitationStatus::Cancelled => write!(f, "cancelled"),
            InvitationStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for InvitationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(InvitationStatus::Pending),
            "accepted" => Ok(InvitationStatus::Accepted),
            "expired" => Ok(InvitationStatus::Expired),
            "cancelled" => Ok(InvitationStatus::Cancelled),
            "failed" => Ok(InvitationStatus::Failed),
            _ => Err(format!("Invalid invitation status: {}", s)),
        }
    }
}

/// User invitation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInvitation {
    pub invitation_id: String,
    pub inviter_user_id: String,
    pub invitee_identifier: String, // email, telegram username, or phone
    pub invitation_type: InvitationType,
    pub status: InvitationStatus,
    pub message: Option<String>,
    pub invitation_data: serde_json::Value, // Additional invitation-specific data
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
}

impl UserInvitation {
    pub fn new(
        inviter_user_id: String,
        invitee_identifier: String,
        invitation_type: InvitationType,
        message: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let expires_at = Some(now + chrono::Duration::days(7)); // Default 7 days expiry

        Self {
            invitation_id: uuid::Uuid::new_v4().to_string(),
            inviter_user_id,
            invitee_identifier,
            invitation_type,
            status: InvitationStatus::default(),
            message,
            invitation_data: serde_json::Value::Null,
            created_at: now,
            expires_at,
            accepted_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn accept(&mut self) {
        self.status = InvitationStatus::Accepted;
        self.accepted_at = Some(Utc::now());
    }
}

/// Trading analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAnalytics {
    pub analytics_id: String,
    pub user_id: String,
    pub metric_type: String, // e.g., "opportunity_found", "trade_executed", "profit_loss"
    pub metric_value: f64,
    pub metric_data: serde_json::Value, // Detailed metric data
    pub exchange_id: Option<String>,
    pub trading_pair: Option<String>,
    pub opportunity_type: Option<String>, // e.g., "arbitrage", "momentum", "pattern"
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub analytics_metadata: serde_json::Value, // Additional analytics metadata
}

impl TradingAnalytics {
    pub fn new(
        user_id: String,
        metric_type: String,
        metric_value: f64,
        metric_data: serde_json::Value,
    ) -> Self {
        Self {
            analytics_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            metric_type,
            metric_value,
            metric_data,
            exchange_id: None,
            trading_pair: None,
            opportunity_type: None,
            timestamp: Utc::now(),
            session_id: None,
            analytics_metadata: serde_json::Value::Null,
        }
    }

    pub fn with_exchange(mut self, exchange_id: String) -> Self {
        self.exchange_id = Some(exchange_id);
        self
    }

    pub fn with_trading_pair(mut self, trading_pair: String) -> Self {
        self.trading_pair = Some(trading_pair);
        self
    }

    pub fn with_opportunity_type(mut self, opportunity_type: String) -> Self {
        self.opportunity_type = Some(opportunity_type);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

// Advanced Position Management Types (Task 6)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionAction {
    Hold,               // Keep position as is
    IncreaseSize,       // Add to position
    DecreaseSize,       // Reduce position size
    Close,              // Close position immediately
    SetStopLoss,        // Update stop loss
    SetTakeProfit,      // Update take profit
    EnableTrailingStop, // Enable trailing stop
    Hedge,              // Create hedge position
    Rebalance,          // Rebalance across exchanges
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_position_size_usd: f64,
    pub max_total_exposure_usd: f64,
    pub default_stop_loss_percentage: f64, // e.g., 0.02 for 2%
    pub default_take_profit_percentage: f64, // e.g., 0.04 for 4%
    pub max_positions_per_exchange: u32,
    pub max_positions_per_pair: u32,
    pub enable_trailing_stops: bool,
    pub min_risk_reward_ratio: f64, // e.g., 1.5 for 1:1.5 risk/reward
}

impl Default for RiskManagementConfig {
    fn default() -> Self {
        Self {
            max_position_size_usd: 1000.0,
            max_total_exposure_usd: 5000.0,
            default_stop_loss_percentage: 0.02,
            default_take_profit_percentage: 0.04,
            max_positions_per_exchange: 10,
            max_positions_per_pair: 3,
            enable_trailing_stops: true,
            min_risk_reward_ratio: 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionOptimizationResult {
    pub position_id: String,
    pub current_score: f64,
    pub recommended_action: PositionAction,
    pub confidence_level: f64, // 0.0 to 1.0
    pub reasoning: String,
    pub suggested_stop_loss: Option<f64>,
    pub suggested_take_profit: Option<f64>,
    pub risk_assessment: RiskAssessment,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: RiskLevel,
    pub volatility_score: f64,
    pub correlation_risk: f64,   // Risk from correlated positions
    pub liquidity_risk: f64,     // Risk from low liquidity
    pub concentration_risk: f64, // Risk from position concentration
    pub overall_risk_score: f64, // Combined risk score 0-100
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// User access levels for opportunity distribution and trading features
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserAccessLevel {
    /// Free user without any API keys - view-only access
    FreeWithoutAPI,
    /// Free user with exchange API keys - limited opportunities + trading
    FreeWithAPI,
    /// Subscription user with exchange API keys - unlimited opportunities + advanced features
    SubscriptionWithAPI,
}

impl UserAccessLevel {
    /// Get daily opportunity limits for this access level
    pub fn get_daily_opportunity_limits(&self) -> (u32, u32) {
        match self {
            UserAccessLevel::FreeWithoutAPI => (0, 0), // No opportunities for users without APIs
            UserAccessLevel::FreeWithAPI => (10, 10),  // 10 arbitrage + 10 technical daily
            UserAccessLevel::SubscriptionWithAPI => (u32::MAX, u32::MAX), // Unlimited
        }
    }

    /// Check if user can access trading features
    pub fn can_trade(&self) -> bool {
        match self {
            UserAccessLevel::FreeWithoutAPI => false,
            UserAccessLevel::FreeWithAPI => true,
            UserAccessLevel::SubscriptionWithAPI => true,
        }
    }

    /// Check if user can receive real-time opportunities
    pub fn gets_realtime_opportunities(&self) -> bool {
        match self {
            UserAccessLevel::FreeWithoutAPI => false, // No opportunities
            UserAccessLevel::FreeWithAPI => false,    // 5-minute delay
            UserAccessLevel::SubscriptionWithAPI => true, // Real-time
        }
    }

    /// Get opportunity delivery delay in seconds
    pub fn get_opportunity_delay_seconds(&self) -> u64 {
        match self {
            UserAccessLevel::FreeWithoutAPI => 0,      // No opportunities
            UserAccessLevel::FreeWithAPI => 300,       // 5-minute delay
            UserAccessLevel::SubscriptionWithAPI => 0, // Real-time
        }
    }

    /// Check if user can access personal opportunity generation
    pub fn can_generate_personal_opportunities(&self) -> bool {
        match self {
            UserAccessLevel::FreeWithoutAPI => false,
            UserAccessLevel::FreeWithAPI => false,
            UserAccessLevel::SubscriptionWithAPI => true,
        }
    }
}

impl std::fmt::Display for UserAccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserAccessLevel::FreeWithoutAPI => write!(f, "free_without_api"),
            UserAccessLevel::FreeWithAPI => write!(f, "free_with_api"),
            UserAccessLevel::SubscriptionWithAPI => write!(f, "subscription_with_api"),
        }
    }
}

impl std::str::FromStr for UserAccessLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free_without_api" => Ok(UserAccessLevel::FreeWithoutAPI),
            "free_with_api" => Ok(UserAccessLevel::FreeWithAPI),
            "subscription_with_api" => Ok(UserAccessLevel::SubscriptionWithAPI),
            _ => Err(format!("Invalid user access level: {}", s)),
        }
    }
}

/// Daily opportunity tracking for users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpportunityLimits {
    pub user_id: String,
    pub access_level: UserAccessLevel,
    pub date: String, // YYYY-MM-DD format
    pub arbitrage_opportunities_received: u32,
    pub technical_opportunities_received: u32,
    pub arbitrage_limit: u32,
    pub technical_limit: u32,
    pub last_reset: u64,                // Timestamp of last daily reset
    pub is_group_context: bool,         // Whether user is in group/channel context
    pub group_multiplier_applied: bool, // Whether 2x multiplier has been applied
}

impl UserOpportunityLimits {
    pub fn new(user_id: String, access_level: UserAccessLevel, is_group_context: bool) -> Self {
        let (arbitrage_limit, technical_limit) = access_level.get_daily_opportunity_limits();

        // Apply 2x multiplier for group/channel contexts
        let (final_arbitrage_limit, final_technical_limit) = if is_group_context {
            (
                arbitrage_limit.saturating_mul(2),
                technical_limit.saturating_mul(2),
            )
        } else {
            (arbitrage_limit, technical_limit)
        };

        let now = chrono::Utc::now();
        Self {
            user_id,
            access_level,
            date: now.format("%Y-%m-%d").to_string(),
            arbitrage_opportunities_received: 0,
            technical_opportunities_received: 0,
            arbitrage_limit: final_arbitrage_limit,
            technical_limit: final_technical_limit,
            last_reset: now.timestamp() as u64,
            is_group_context,
            group_multiplier_applied: is_group_context,
        }
    }

    /// Check if user can receive more arbitrage opportunities
    pub fn can_receive_arbitrage(&self) -> bool {
        self.arbitrage_opportunities_received < self.arbitrage_limit
    }

    /// Check if user can receive more technical opportunities
    pub fn can_receive_technical(&self) -> bool {
        self.technical_opportunities_received < self.technical_limit
    }

    /// Record that user received an arbitrage opportunity
    pub fn record_arbitrage_received(&mut self) -> bool {
        if self.can_receive_arbitrage() {
            self.arbitrage_opportunities_received += 1;
            true
        } else {
            false
        }
    }

    /// Record that user received a technical opportunity
    pub fn record_technical_received(&mut self) -> bool {
        if self.can_receive_technical() {
            self.technical_opportunities_received += 1;
            true
        } else {
            false
        }
    }

    /// Check if daily reset is needed
    pub fn needs_daily_reset(&self) -> bool {
        let now = chrono::Utc::now();
        let current_date = now.format("%Y-%m-%d").to_string();
        self.date != current_date
    }

    /// Reset daily counters
    pub fn reset_daily_counters(&mut self) {
        let now = chrono::Utc::now();
        self.date = now.format("%Y-%m-%d").to_string();
        self.arbitrage_opportunities_received = 0;
        self.technical_opportunities_received = 0;
        self.last_reset = now.timestamp() as u64;
    }

    /// Get remaining opportunities for both types
    pub fn get_remaining_opportunities(&self) -> (u32, u32) {
        let remaining_arbitrage = self
            .arbitrage_limit
            .saturating_sub(self.arbitrage_opportunities_received);
        let remaining_technical = self
            .technical_limit
            .saturating_sub(self.technical_opportunities_received);
        (remaining_arbitrage, remaining_technical)
    }
}

/// Group/Channel context information for opportunity distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatContext {
    Private,
    Group(String),   // Group ID
    Channel(String), // Channel ID
}

impl ChatContext {
    /// Check if this is a group or channel context (gets 2x multiplier)
    pub fn is_group_context(&self) -> bool {
        matches!(self, ChatContext::Group(_) | ChatContext::Channel(_))
    }

    /// Get context ID for tracking
    pub fn get_context_id(&self) -> String {
        match self {
            ChatContext::Private => "private".to_string(),
            ChatContext::Group(id) => format!("group_{}", id),
            ChatContext::Channel(id) => format!("channel_{}", id),
        }
    }
}

/// AI access levels based on subscription tier and AI key availability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIAccessLevel {
    /// Free user without AI keys - can view AI-enhanced global opportunities (read-only)
    FreeWithoutAI {
        ai_analysis: bool,               // false - No AI features
        view_global_ai: bool,            // true - Can view AI-enhanced global opportunities
        daily_ai_limit: u32,             // 0 - No AI calls allowed
        template_access: TemplateAccess, // None - No template access
    },
    /// Free user with AI keys - basic AI analysis with default templates
    FreeWithAI {
        ai_analysis: bool,               // true - Basic AI analysis
        custom_templates: bool,          // false - Only default templates
        daily_ai_limit: u32,             // 5 - Limited AI calls per day
        global_ai_enhancement: bool,     // true - AI enhances global opportunities
        personal_ai_generation: bool,    // false - No personal opportunity generation
        template_access: TemplateAccess, // DefaultOnly - Only system templates
    },
    /// Subscription user with AI keys - full AI access with custom templates
    SubscriptionWithAI {
        ai_analysis: bool,               // true - Full AI analysis
        custom_templates: bool,          // true - Custom AI templates
        daily_ai_limit: u32,             // 100+ - High daily limits
        global_ai_enhancement: bool,     // true - AI enhances global opportunities
        personal_ai_generation: bool,    // true - AI generates personal opportunities
        ai_marketplace: bool,            // true - Access to AI marketplace
        template_access: TemplateAccess, // Full - Full template customization
    },
}

impl AIAccessLevel {
    /// Get daily AI usage limits based on access level
    pub fn get_daily_ai_limits(&self) -> u32 {
        match self {
            AIAccessLevel::FreeWithoutAI { daily_ai_limit, .. } => *daily_ai_limit,
            AIAccessLevel::FreeWithAI { daily_ai_limit, .. } => *daily_ai_limit,
            AIAccessLevel::SubscriptionWithAI { daily_ai_limit, .. } => *daily_ai_limit,
        }
    }

    /// Check if user can use AI analysis features
    pub fn can_use_ai_analysis(&self) -> bool {
        match self {
            AIAccessLevel::FreeWithoutAI { ai_analysis, .. } => *ai_analysis,
            AIAccessLevel::FreeWithAI { ai_analysis, .. } => *ai_analysis,
            AIAccessLevel::SubscriptionWithAI { ai_analysis, .. } => *ai_analysis,
        }
    }

    /// Check if user can create custom AI templates
    pub fn can_create_custom_templates(&self) -> bool {
        match self {
            AIAccessLevel::FreeWithoutAI { .. } => false,
            AIAccessLevel::FreeWithAI {
                custom_templates, ..
            } => *custom_templates,
            AIAccessLevel::SubscriptionWithAI {
                custom_templates, ..
            } => *custom_templates,
        }
    }

    /// Check if user can generate personal AI opportunities
    pub fn can_generate_personal_ai_opportunities(&self) -> bool {
        match self {
            AIAccessLevel::FreeWithoutAI { .. } => false,
            AIAccessLevel::FreeWithAI {
                personal_ai_generation,
                ..
            } => *personal_ai_generation,
            AIAccessLevel::SubscriptionWithAI {
                personal_ai_generation,
                ..
            } => *personal_ai_generation,
        }
    }

    /// Check if user can view AI-enhanced global opportunities
    pub fn can_view_global_ai_opportunities(&self) -> bool {
        match self {
            AIAccessLevel::FreeWithoutAI { view_global_ai, .. } => *view_global_ai,
            AIAccessLevel::FreeWithAI {
                global_ai_enhancement,
                ..
            } => *global_ai_enhancement,
            AIAccessLevel::SubscriptionWithAI {
                global_ai_enhancement,
                ..
            } => *global_ai_enhancement,
        }
    }

    /// Get template access level
    pub fn get_template_access(&self) -> &TemplateAccess {
        match self {
            AIAccessLevel::FreeWithoutAI {
                template_access, ..
            } => template_access,
            AIAccessLevel::FreeWithAI {
                template_access, ..
            } => template_access,
            AIAccessLevel::SubscriptionWithAI {
                template_access, ..
            } => template_access,
        }
    }
}

impl std::fmt::Display for AIAccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIAccessLevel::FreeWithoutAI { .. } => write!(f, "free_without_ai"),
            AIAccessLevel::FreeWithAI { .. } => write!(f, "free_with_ai"),
            AIAccessLevel::SubscriptionWithAI { .. } => write!(f, "subscription_with_ai"),
        }
    }
}

impl std::str::FromStr for AIAccessLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "free_without_ai" => Ok(AIAccessLevel::FreeWithoutAI {
                ai_analysis: false,
                view_global_ai: true,
                daily_ai_limit: 0,
                template_access: TemplateAccess::None,
            }),
            "free_with_ai" => Ok(AIAccessLevel::FreeWithAI {
                ai_analysis: true,
                custom_templates: false,
                daily_ai_limit: 5,
                global_ai_enhancement: true,
                personal_ai_generation: false,
                template_access: TemplateAccess::DefaultOnly,
            }),
            "subscription_with_ai" => Ok(AIAccessLevel::SubscriptionWithAI {
                ai_analysis: true,
                custom_templates: true,
                daily_ai_limit: 100,
                global_ai_enhancement: true,
                personal_ai_generation: true,
                ai_marketplace: true,
                template_access: TemplateAccess::Full,
            }),
            _ => Err(format!("Invalid AI access level: {}", s)),
        }
    }
}

/// AI template access levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateAccess {
    /// No template access
    None,
    /// Only system default templates
    DefaultOnly,
    /// Full template customization + marketplace
    Full,
}

impl std::fmt::Display for TemplateAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateAccess::None => write!(f, "none"),
            TemplateAccess::DefaultOnly => write!(f, "default_only"),
            TemplateAccess::Full => write!(f, "full"),
        }
    }
}

/// AI template structure for customizable AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITemplate {
    pub template_id: String,
    pub template_name: String,
    pub template_type: AITemplateType,
    pub access_level: TemplateAccess,
    pub prompt_template: String,
    pub parameters: AITemplateParameters,
    pub created_by: Option<String>, // None for system templates
    pub is_system_default: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

impl AITemplate {
    pub fn new_system_template(
        template_name: String,
        template_type: AITemplateType,
        prompt_template: String,
        parameters: AITemplateParameters,
    ) -> Self {
        Self {
            template_id: uuid::Uuid::new_v4().to_string(),
            template_name,
            template_type,
            access_level: TemplateAccess::DefaultOnly,
            prompt_template,
            parameters,
            created_by: None,
            is_system_default: true,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    pub fn new_user_template(
        template_name: String,
        template_type: AITemplateType,
        prompt_template: String,
        parameters: AITemplateParameters,
        created_by: String,
    ) -> Self {
        Self {
            template_id: uuid::Uuid::new_v4().to_string(),
            template_name,
            template_type,
            access_level: TemplateAccess::Full,
            prompt_template,
            parameters,
            created_by: Some(created_by),
            is_system_default: false,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Types of AI templates for different use cases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AITemplateType {
    /// Analyze global opportunities
    GlobalOpportunityAnalysis,
    /// Generate personal opportunities
    PersonalOpportunityGeneration,
    /// Support trading decisions
    TradingDecisionSupport,
    /// Risk analysis
    RiskAssessment,
    /// Position size recommendations
    PositionSizing,
}

impl std::fmt::Display for AITemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AITemplateType::GlobalOpportunityAnalysis => write!(f, "global_opportunity_analysis"),
            AITemplateType::PersonalOpportunityGeneration => {
                write!(f, "personal_opportunity_generation")
            }
            AITemplateType::TradingDecisionSupport => write!(f, "trading_decision_support"),
            AITemplateType::RiskAssessment => write!(f, "risk_assessment"),
            AITemplateType::PositionSizing => write!(f, "position_sizing"),
        }
    }
}

/// AI template parameters for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITemplateParameters {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

impl Default for AITemplateParameters {
    fn default() -> Self {
        Self {
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            custom_parameters: HashMap::new(),
        }
    }
}

/// AI usage tracking for daily limits and cost monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUsageTracker {
    pub user_id: String,
    pub date: String, // YYYY-MM-DD format
    pub ai_calls_used: u32,
    pub ai_calls_limit: u32,
    pub last_reset: u64, // Timestamp of last daily reset
    pub access_level: AIAccessLevel,
    pub total_cost_usd: f64,
    pub cost_breakdown_by_provider: HashMap<String, f64>,
    pub cost_breakdown_by_feature: HashMap<String, f64>,
}

impl AIUsageTracker {
    pub fn new(user_id: String, access_level: AIAccessLevel) -> Self {
        let daily_limit = access_level.get_daily_ai_limits();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        Self {
            user_id,
            date: today,
            ai_calls_used: 0,
            ai_calls_limit: daily_limit,
            last_reset: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            access_level,
            total_cost_usd: 0.0,
            cost_breakdown_by_provider: HashMap::new(),
            cost_breakdown_by_feature: HashMap::new(),
        }
    }

    pub fn can_make_ai_call(&self) -> bool {
        self.ai_calls_used < self.ai_calls_limit
    }

    pub fn record_ai_call(&mut self, cost_usd: f64, provider: String, feature: String) -> bool {
        if !self.can_make_ai_call() {
            return false;
        }

        self.ai_calls_used += 1;
        self.total_cost_usd += cost_usd;

        *self
            .cost_breakdown_by_provider
            .entry(provider)
            .or_insert(0.0) += cost_usd;
        *self.cost_breakdown_by_feature.entry(feature).or_insert(0.0) += cost_usd;

        true
    }

    pub fn needs_daily_reset(&self) -> bool {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        self.date != today
    }

    pub fn reset_daily_counters(&mut self) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        self.date = today;
        self.ai_calls_used = 0;
        self.last_reset = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        // Note: Cost tracking is cumulative, not reset daily
    }

    pub fn get_remaining_calls(&self) -> u32 {
        self.ai_calls_limit.saturating_sub(self.ai_calls_used)
    }
}
