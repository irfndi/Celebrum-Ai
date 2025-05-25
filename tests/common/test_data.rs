// Common Test Data
// Predefined test data constants and fixtures

use arb_edge::services::core::analysis::market_analysis::{
    OpportunityType, RiskLevel, TimeHorizon,
};
use arb_edge::services::core::user::user_trading_preferences::{
    ExperienceLevel, RiskTolerance, TradingFocus,
};
use arb_edge::types::SubscriptionTier;

// Test User Constants
pub const TEST_TELEGRAM_ID_1: i64 = 111111111;
pub const TEST_TELEGRAM_ID_2: i64 = 222222222;
pub const TEST_TELEGRAM_ID_3: i64 = 333333333;
pub const TEST_CHAT_ID_1: i64 = 987654321;
pub const TEST_CHAT_ID_2: i64 = 876543210;

// Test Trading Pairs
pub const TRADING_PAIRS: &[&str] = &["BTCUSDT", "ETHUSDT", "ADAUSDT", "SOLUSDT", "BNBUSDT"];

// Test Exchanges
pub const EXCHANGES: &[&str] = &["binance", "bybit", "okx", "coinbase", "kraken"];

// Test Price Data
pub const BTC_BASE_PRICE: f64 = 50000.0;
pub const ETH_BASE_PRICE: f64 = 3000.0;
pub const ADA_BASE_PRICE: f64 = 0.5;
pub const SOL_BASE_PRICE: f64 = 100.0;
pub const BNB_BASE_PRICE: f64 = 300.0;

// Test User Profiles
pub struct TestUserProfiles;

impl TestUserProfiles {
    pub fn free_user_beginner() -> (
        i64,
        SubscriptionTier,
        TradingFocus,
        ExperienceLevel,
        RiskTolerance,
    ) {
        (
            TEST_TELEGRAM_ID_1,
            SubscriptionTier::Free,
            TradingFocus::Arbitrage,
            ExperienceLevel::Beginner,
            RiskTolerance::Conservative,
        )
    }

    pub fn basic_user_intermediate() -> (
        i64,
        SubscriptionTier,
        TradingFocus,
        ExperienceLevel,
        RiskTolerance,
    ) {
        (
            TEST_TELEGRAM_ID_2,
            SubscriptionTier::Basic,
            TradingFocus::TechnicalAnalysis,
            ExperienceLevel::Intermediate,
            RiskTolerance::Balanced,
        )
    }

    pub fn premium_user_advanced() -> (
        i64,
        SubscriptionTier,
        TradingFocus,
        ExperienceLevel,
        RiskTolerance,
    ) {
        (
            TEST_TELEGRAM_ID_3,
            SubscriptionTier::Premium,
            TradingFocus::Hybrid,
            ExperienceLevel::Advanced,
            RiskTolerance::Aggressive,
        )
    }
}

// Test Opportunity Templates
pub struct TestOpportunityTemplates;

impl TestOpportunityTemplates {
    pub fn high_confidence_low_risk() -> (OpportunityType, f64, RiskLevel, TimeHorizon) {
        (
            OpportunityType::Arbitrage,
            0.95,
            RiskLevel::Low,
            TimeHorizon::Short,
        )
    }

    pub fn medium_confidence_medium_risk() -> (OpportunityType, f64, RiskLevel, TimeHorizon) {
        (
            OpportunityType::Technical,
            0.75,
            RiskLevel::Medium,
            TimeHorizon::Medium,
        )
    }

    pub fn low_confidence_high_risk() -> (OpportunityType, f64, RiskLevel, TimeHorizon) {
        (
            OpportunityType::Arbitrage,
            0.65,
            RiskLevel::High,
            TimeHorizon::Short,
        )
    }

    pub fn ai_enhanced_opportunity() -> (OpportunityType, f64, RiskLevel, TimeHorizon) {
        (
            OpportunityType::ArbitrageTechnical,
            0.88,
            RiskLevel::Low,
            TimeHorizon::Medium,
        )
    }
}

// Test Market Scenarios
pub struct TestMarketScenarios;

impl TestMarketScenarios {
    /// Bull market scenario with generally rising prices
    pub fn bull_market_prices() -> Vec<(&'static str, &'static str, f64)> {
        vec![
            ("binance", "BTCUSDT", BTC_BASE_PRICE * 1.05),
            ("bybit", "BTCUSDT", BTC_BASE_PRICE * 1.04),
            ("okx", "BTCUSDT", BTC_BASE_PRICE * 1.06),
            ("binance", "ETHUSDT", ETH_BASE_PRICE * 1.08),
            ("bybit", "ETHUSDT", ETH_BASE_PRICE * 1.07),
            ("okx", "ETHUSDT", ETH_BASE_PRICE * 1.09),
        ]
    }

    /// Bear market scenario with generally falling prices
    pub fn bear_market_prices() -> Vec<(&'static str, &'static str, f64)> {
        vec![
            ("binance", "BTCUSDT", BTC_BASE_PRICE * 0.92),
            ("bybit", "BTCUSDT", BTC_BASE_PRICE * 0.91),
            ("okx", "BTCUSDT", BTC_BASE_PRICE * 0.93),
            ("binance", "ETHUSDT", ETH_BASE_PRICE * 0.88),
            ("bybit", "ETHUSDT", ETH_BASE_PRICE * 0.87),
            ("okx", "ETHUSDT", ETH_BASE_PRICE * 0.89),
        ]
    }

    /// High volatility scenario with significant price differences
    pub fn high_volatility_prices() -> Vec<(&'static str, &'static str, f64)> {
        vec![
            ("binance", "BTCUSDT", BTC_BASE_PRICE),
            ("bybit", "BTCUSDT", BTC_BASE_PRICE * 1.025), // 2.5% difference
            ("okx", "BTCUSDT", BTC_BASE_PRICE * 0.98),    // 2% difference
            ("binance", "ETHUSDT", ETH_BASE_PRICE),
            ("bybit", "ETHUSDT", ETH_BASE_PRICE * 1.03), // 3% difference
            ("okx", "ETHUSDT", ETH_BASE_PRICE * 0.975),  // 2.5% difference
        ]
    }

    /// Low volatility scenario with minimal price differences
    pub fn low_volatility_prices() -> Vec<(&'static str, &'static str, f64)> {
        vec![
            ("binance", "BTCUSDT", BTC_BASE_PRICE),
            ("bybit", "BTCUSDT", BTC_BASE_PRICE * 1.001), // 0.1% difference
            ("okx", "BTCUSDT", BTC_BASE_PRICE * 0.999),   // 0.1% difference
            ("binance", "ETHUSDT", ETH_BASE_PRICE),
            ("bybit", "ETHUSDT", ETH_BASE_PRICE * 1.0015), // 0.15% difference
            ("okx", "ETHUSDT", ETH_BASE_PRICE * 0.9985),   // 0.15% difference
        ]
    }
}

// Test Command Scenarios
pub struct TestCommandScenarios;

impl TestCommandScenarios {
    pub fn basic_commands() -> Vec<&'static str> {
        vec![
            "/start",
            "/help",
            "/opportunities",
            "/categories",
            "/settings",
        ]
    }

    pub fn trading_commands() -> Vec<&'static str> {
        vec!["/buy", "/sell", "/balance", "/orders", "/positions"]
    }

    pub fn admin_commands() -> Vec<&'static str> {
        vec![
            "/admin_stats",
            "/admin_users",
            "/admin_config",
            "/admin_broadcast",
        ]
    }

    pub fn ai_commands() -> Vec<&'static str> {
        vec!["/ai_insights", "/risk_assessment", "/ai_analysis"]
    }

    pub fn auto_trading_commands() -> Vec<&'static str> {
        vec![
            "/auto_enable",
            "/auto_disable",
            "/auto_config",
            "/auto_status",
        ]
    }
}

// Test Error Scenarios
pub struct TestErrorScenarios;

impl TestErrorScenarios {
    pub fn invalid_telegram_ids() -> Vec<i64> {
        vec![-1, 0, i64::MAX]
    }

    pub fn invalid_confidence_scores() -> Vec<f64> {
        vec![-0.1, 1.1, f64::NAN, f64::INFINITY]
    }

    pub fn invalid_trading_pairs() -> Vec<&'static str> {
        vec!["", "INVALID", "BTC", "USDT", "BTCUSD"]
    }

    pub fn malformed_commands() -> Vec<&'static str> {
        vec!["", "/", "//start", "/start extra params", "/nonexistent"]
    }
}

// Test Performance Scenarios
pub struct TestPerformanceScenarios;

impl TestPerformanceScenarios {
    pub fn large_dataset_sizes() -> Vec<usize> {
        vec![100, 500, 1000, 5000, 10000]
    }

    pub fn concurrent_user_counts() -> Vec<usize> {
        vec![10, 50, 100, 500, 1000]
    }

    pub fn notification_burst_sizes() -> Vec<usize> {
        vec![5, 10, 25, 50, 100]
    }
}
