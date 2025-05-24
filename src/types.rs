// src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
// use thiserror::Error; // TODO: Re-enable when implementing custom error types

/// Exchange identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}

impl std::fmt::Display for ExchangeIdEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub pair: String,
    pub long_exchange: Option<ExchangeIdEnum>,
    pub short_exchange: Option<ExchangeIdEnum>,
    pub long_rate: Option<f64>,
    pub short_rate: Option<f64>,
    pub rate_difference: f64,
    pub net_rate_difference: Option<f64>,
    pub potential_profit_value: Option<f64>,
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub r#type: ArbitrageType,
    pub details: Option<String>,
}

impl ArbitrageOpportunity {
    pub fn new(
        pair: String,
        long_exchange: Option<ExchangeIdEnum>,
        short_exchange: Option<ExchangeIdEnum>,
        long_rate: Option<f64>,
        short_rate: Option<f64>,
        rate_difference: f64,
        r#type: ArbitrageType,
    ) -> Self {
        Self {
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
        }
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
    User,      // Regular user (Free, Basic, Premium, Enterprise)
    SuperAdmin, // Super administrator with system access
    BetaUser,   // Beta user (all features during beta period)
}

/// Command permissions for RBAC
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandPermission {
    // Basic commands (available to all)
    BasicCommands,
    
    // Trading commands (subscription-gated in future)
    ManualTrading,
    AutomatedTrading,
    
    // Opportunity access levels
    BasicOpportunities,     // Global arbitrage only
    TechnicalAnalysis,      // Global + technical analysis
    AIEnhancedOpportunities, // Premium AI features
    
    // Admin commands (super admin only)
    SystemAdministration,
    UserManagement,
    GlobalConfiguration,
    GroupAnalytics,         // Group/channel analytics access
    
    // Future subscription-gated features
    AdvancedAnalytics,
    PremiumFeatures,
}

/// Trading modes for users
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradingMode {
    Manual,      // User executes trades manually with API keys
    Automated,   // Bot executes trades automatically
    Advisory,    // Bot provides signals, user decides
    Disabled,    // No trading functionality
}

/// Analytics tracking for bot usage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageAnalytics {
    pub message_id: String,
    pub user_id: Option<String>,
    pub chat_id: String,
    pub chat_type: String,           // "private", "group", "supergroup", "channel"
    pub message_type: String,        // "command", "opportunity", "broadcast", "response"
    pub command: Option<String>,     // Command name if applicable
    pub content_type: String,        // "global_arbitrage", "technical_analysis", "ai_enhanced", etc.
    pub delivery_status: String,     // "sent", "delivered", "failed", "rate_limited"
    pub response_time_ms: Option<u64>, // Time to generate response
    pub timestamp: u64,
    pub metadata: serde_json::Value, // Additional tracking data
}

/// Group/Channel registration and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRegistration {
    pub group_id: String,
    pub group_type: String,          // "group", "supergroup", "channel"
    pub group_title: Option<String>,
    pub group_username: Option<String>,
    pub member_count: Option<u32>,
    pub admin_user_ids: Vec<String>, // Telegram user IDs of admins
    pub bot_permissions: Vec<String>, // What the bot can do in this group
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
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            provider: ApiKeyProvider::Exchange(exchange),
            encrypted_key: encrypted_api_key,
            encrypted_secret: Some(encrypted_secret),
            metadata: serde_json::json!({}),
            is_active: true,
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
    pub created_at: u64,
    pub updated_at: u64,
    pub last_active: u64,
    pub is_active: bool,
    pub total_trades: u32,
    pub total_pnl_usdt: f64,
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
            created_at: now,
            updated_at: now,
            last_active: now,
            is_active: true,
            total_trades: 0,
            total_pnl_usdt: 0.0,
        }
    }

    /// Get user role for RBAC
    pub fn get_user_role(&self) -> UserRole {
        match self.subscription.tier {
            SubscriptionTier::SuperAdmin => UserRole::SuperAdmin,
            _ => UserRole::User, // Regular user for all other tiers
        }
    }

    /// Check if user has permission for a specific command
    pub fn has_permission(&self, permission: CommandPermission) -> bool {
        let user_role = self.get_user_role();
        
        match permission {
            CommandPermission::BasicCommands |
            CommandPermission::BasicOpportunities => true, // Everyone has basic access
            
            CommandPermission::ManualTrading => {
                // During beta: all users have access
                // Future: require Basic+ subscription + API keys with trade permissions
                true // Beta override - remove this when implementing subscription gates
            }
            
            CommandPermission::TechnicalAnalysis => {
                // During beta: all users have access
                // Future: require Basic+ subscription
                true // Beta override
            }
            
            CommandPermission::AIEnhancedOpportunities => {
                // During beta: all users have access
                // Future: require Premium+ subscription
                true // Beta override
            }
            
            CommandPermission::AutomatedTrading => {
                // During beta: all users have access
                // Future: require Premium+ subscription + risk management setup
                true // Beta override
            }
            
            CommandPermission::SystemAdministration |
            CommandPermission::UserManagement |
            CommandPermission::GlobalConfiguration |
            CommandPermission::GroupAnalytics => {
                user_role == UserRole::SuperAdmin
            }
            
            CommandPermission::AdvancedAnalytics |
            CommandPermission::PremiumFeatures => {
                // During beta: all users have access
                // Future: require Premium+ subscription
                true // Beta override
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
            key.is_active && 
            key.is_exchange_key() && 
            key.permissions.contains(&"trade".to_string())
        })
    }

    /// Check if user can use manual trading
    pub fn can_use_manual_trading(&self) -> bool {
        self.has_permission(CommandPermission::ManualTrading) && self.has_trading_api_keys()
    }

    /// Check if user can use automated trading
    pub fn can_use_automated_trading(&self) -> bool {
        self.has_permission(CommandPermission::AutomatedTrading) && 
        self.has_trading_api_keys() &&
        matches!(self.get_trading_mode(), TradingMode::Automated | TradingMode::Advisory)
    }

    /// Check if user is super admin
    pub fn is_super_admin(&self) -> bool {
        self.subscription.tier == SubscriptionTier::SuperAdmin
    }

    /// Create super admin user profile
    pub fn new_super_admin(telegram_user_id: Option<i64>) -> Self {
        let mut profile = Self::new(telegram_user_id, None);
        profile.subscription.tier = SubscriptionTier::SuperAdmin;
        profile.subscription.features.push("super_admin_access".to_string());
        profile.subscription.features.push("system_administration".to_string());
        profile.subscription.features.push("user_management".to_string());
        profile.subscription.features.push("global_configuration".to_string());
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
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let expires_at = now + (24 * 60 * 60 * 1000); // 24 hours

        Self {
            user_id,
            telegram_chat_id,
            last_command: None,
            current_state: SessionState::Idle,
            temporary_data: std::collections::HashMap::new(),
            created_at: now,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now > self.expires_at
    }

    pub fn extend_session(&mut self) {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        self.expires_at = now + (24 * 60 * 60 * 1000); // Extend by 24 hours
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
            max_opportunities_per_user_per_hour: 2,  // Updated: max 2 opportunities per cycle
            max_opportunities_per_user_per_day: 10,  // Updated: max 10 daily
            tier_multipliers,
            activity_boost_factor: 1.2,
            cooldown_period_minutes: 240,  // Updated: 4-hour cooldown (240 minutes)
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
