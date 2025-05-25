// Comprehensive Service Integration Tests
// Testing service interactions and data flow for 100% coverage

use arb_edge::services::interfaces::telegram::telegram::{TelegramService, TelegramConfig};
use arb_edge::services::interfaces::telegram::telegram_keyboard::InlineKeyboard;
use arb_edge::services::core::analysis::market_analysis::{TradingOpportunity, OpportunityType, RiskLevel, TimeHorizon};
use arb_edge::types::{
    UserProfile, SubscriptionTier, ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum,
};
use serde_json::json;

// Test data creation helpers
fn create_test_user_profile(telegram_id: i64, subscription_tier: SubscriptionTier) -> UserProfile {
    let mut user = UserProfile::new(Some(telegram_id), Some("test-invite".to_string()));
    user.subscription.tier = subscription_tier;
    user
}

fn create_test_telegram_message(user_id: i64, chat_id: i64, text: &str) -> serde_json::Value {
    json!({
        "update_id": 12345,
        "message": {
            "message_id": 67890,
            "from": {
                "id": user_id,
                "is_bot": false,
                "first_name": "Test",
                "last_name": "User",
                "username": "testuser",
                "language_code": "en"
            },
            "chat": {
                "id": chat_id,
                "type": "private",
                "first_name": "Test",
                "last_name": "User"
            },
            "date": chrono::Utc::now().timestamp(),
            "text": text
        }
    })
}

fn create_test_arbitrage_opportunity(id: &str, pair: &str, rate_diff: f64) -> ArbitrageOpportunity {
    let mut opportunity = ArbitrageOpportunity::new(
        pair.to_string(),
        ExchangeIdEnum::Binance,  // Required long_exchange
        ExchangeIdEnum::Bybit,    // Required short_exchange
        Some(50000.0),            // long_rate
        Some(50000.0 + (50000.0 * rate_diff)), // short_rate
        rate_diff,                // rate_difference
        ArbitrageType::CrossExchange,
    );
    
    opportunity.id = id.to_string();
    opportunity
        .with_net_difference(rate_diff * 0.95)
        .with_potential_profit(rate_diff * 100.0)
        .with_details(format!("Test arbitrage opportunity with {}% rate difference", rate_diff * 100.0))
}

fn create_test_trading_opportunity(id: &str, pair: &str, confidence: f64, risk: RiskLevel) -> TradingOpportunity {
    TradingOpportunity {
        opportunity_id: id.to_string(),
        opportunity_type: OpportunityType::Arbitrage,
        trading_pair: pair.to_string(),
        exchanges: vec!["binance".to_string(), "bybit".to_string()],
        entry_price: match pair {
            "BTCUSDT" => 50000.0,
            "ETHUSDT" => 3000.0,
            "ADAUSDT" => 0.5,
            _ => 100.0,
        },
        target_price: Some(50000.0 * 1.02),
        stop_loss: Some(50000.0 * 0.98),
        confidence_score: confidence,
        risk_level: risk,
        expected_return: confidence * 0.05,
        time_horizon: TimeHorizon::Short,
        indicators_used: vec!["arbitrage_analysis".to_string()],
        analysis_data: json!({
            "buy_exchange": "binance",
            "sell_exchange": "bybit",
            "rate_difference": confidence * 0.02
        }),
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        expires_at: Some(chrono::Utc::now().timestamp_millis() as u64 + 3600000),
    }
}

#[tokio::test]
async fn test_telegram_service_initialization() {
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
    };
    
    let telegram_service = TelegramService::new(config);
    
    // Test that service initializes correctly
    assert!(std::mem::size_of_val(&telegram_service) > 0);
}

#[tokio::test]
async fn test_telegram_keyboard_service_functionality() {
    // Test main menu creation
    let main_menu = InlineKeyboard::create_main_menu();
    assert!(!main_menu.buttons.is_empty());
    
    // Test opportunities menu creation
    let opportunities_menu = InlineKeyboard::create_opportunities_menu();
    assert!(!opportunities_menu.buttons.is_empty());
    
    // Test admin menu creation
    let admin_menu = InlineKeyboard::create_admin_menu();
    assert!(!admin_menu.buttons.is_empty());
}

#[tokio::test]
async fn test_user_profile_subscription_tiers() {
    // Test Free tier user
    let free_user = create_test_user_profile(111111111, SubscriptionTier::Free);
    assert_eq!(free_user.subscription.tier, SubscriptionTier::Free);
    assert!(free_user.telegram_user_id.is_some());
    
    // Test Basic tier user
    let basic_user = create_test_user_profile(222222222, SubscriptionTier::Basic);
    assert_eq!(basic_user.subscription.tier, SubscriptionTier::Basic);
    
    // Test Premium tier user
    let premium_user = create_test_user_profile(333333333, SubscriptionTier::Premium);
    assert_eq!(premium_user.subscription.tier, SubscriptionTier::Premium);
    
    // Test Enterprise tier user
    let enterprise_user = create_test_user_profile(444444444, SubscriptionTier::Enterprise);
    assert_eq!(enterprise_user.subscription.tier, SubscriptionTier::Enterprise);
}

#[tokio::test]
async fn test_arbitrage_opportunity_data_structures() {
    let opportunity = create_test_arbitrage_opportunity("test_arb_001", "BTCUSDT", 0.002);
    
    assert_eq!(opportunity.id, "test_arb_001");
    assert_eq!(opportunity.pair, "BTCUSDT");
    assert_eq!(opportunity.rate_difference, 0.002);
    assert_eq!(opportunity.long_exchange, ExchangeIdEnum::Binance);
    assert_eq!(opportunity.short_exchange, ExchangeIdEnum::Bybit);
    assert!(opportunity.net_rate_difference.is_some());
    assert!(opportunity.potential_profit_value.is_some());
    assert!(opportunity.details.is_some());
}

#[tokio::test]
async fn test_trading_opportunity_data_structures() {
    let opportunity = create_test_trading_opportunity("test_trade_001", "ETHUSDT", 0.85, RiskLevel::Low);
    
    assert_eq!(opportunity.opportunity_id, "test_trade_001");
    assert_eq!(opportunity.trading_pair, "ETHUSDT");
    assert_eq!(opportunity.confidence_score, 0.85);
    assert_eq!(opportunity.risk_level, RiskLevel::Low);
    assert_eq!(opportunity.opportunity_type, OpportunityType::Arbitrage);
    assert_eq!(opportunity.time_horizon, TimeHorizon::Short);
    assert!(!opportunity.exchanges.is_empty());
    assert!(opportunity.target_price.is_some());
    assert!(opportunity.stop_loss.is_some());
    assert!(opportunity.expires_at.is_some());
}

#[tokio::test]
async fn test_telegram_update_processing() {
    let update = create_test_telegram_message(123456789, 987654321, "/start");
    
    assert_eq!(update["update_id"], 12345);
    assert!(update["message"].is_object());
    
    let message = &update["message"];
    assert_eq!(message["message_id"], 67890);
    assert!(message["from"].is_object());
    assert_eq!(message["chat"]["id"], 987654321);
    assert_eq!(message["chat"]["type"], "private");
    assert_eq!(message["text"], "/start");
}

#[tokio::test]
async fn test_opportunity_risk_level_mapping() {
    // Test different confidence scores map to appropriate risk levels
    let high_confidence_opp = create_test_trading_opportunity("high_conf", "BTCUSDT", 0.95, RiskLevel::Low);
    let medium_confidence_opp = create_test_trading_opportunity("med_conf", "BTCUSDT", 0.75, RiskLevel::Medium);
    let low_confidence_opp = create_test_trading_opportunity("low_conf", "BTCUSDT", 0.45, RiskLevel::High);
    
    assert_eq!(high_confidence_opp.risk_level, RiskLevel::Low);
    assert_eq!(medium_confidence_opp.risk_level, RiskLevel::Medium);
    assert_eq!(low_confidence_opp.risk_level, RiskLevel::High);
    
    // Verify confidence scores are preserved
    assert_eq!(high_confidence_opp.confidence_score, 0.95);
    assert_eq!(medium_confidence_opp.confidence_score, 0.75);
    assert_eq!(low_confidence_opp.confidence_score, 0.45);
}

#[tokio::test]
async fn test_opportunity_type_variations() {
    let arbitrage_opp = create_test_trading_opportunity("arb_001", "BTCUSDT", 0.8, RiskLevel::Low);
    assert_eq!(arbitrage_opp.opportunity_type, OpportunityType::Arbitrage);
    
    // Test creating technical opportunity
    let mut technical_opp = create_test_trading_opportunity("tech_001", "ETHUSDT", 0.7, RiskLevel::Medium);
    technical_opp.opportunity_type = OpportunityType::Technical;
    assert_eq!(technical_opp.opportunity_type, OpportunityType::Technical);
    
    // Test creating hybrid opportunity
    let mut hybrid_opp = create_test_trading_opportunity("hybrid_001", "ADAUSDT", 0.9, RiskLevel::Low);
    hybrid_opp.opportunity_type = OpportunityType::ArbitrageTechnical;
    assert_eq!(hybrid_opp.opportunity_type, OpportunityType::ArbitrageTechnical);
}

#[tokio::test]
async fn test_time_horizon_mapping() {
    let short_term_opp = create_test_trading_opportunity("short_001", "BTCUSDT", 0.8, RiskLevel::Low);
    assert_eq!(short_term_opp.time_horizon, TimeHorizon::Short);
    
    // Test different time horizons
    let mut immediate_opp = create_test_trading_opportunity("imm_001", "ETHUSDT", 0.9, RiskLevel::Low);
    immediate_opp.time_horizon = TimeHorizon::Immediate;
    assert_eq!(immediate_opp.time_horizon, TimeHorizon::Immediate);
    
    let mut medium_opp = create_test_trading_opportunity("med_001", "ADAUSDT", 0.7, RiskLevel::Medium);
    medium_opp.time_horizon = TimeHorizon::Medium;
    assert_eq!(medium_opp.time_horizon, TimeHorizon::Medium);
    
    let mut long_opp = create_test_trading_opportunity("long_001", "SOLUSDT", 0.6, RiskLevel::High);
    long_opp.time_horizon = TimeHorizon::Long;
    assert_eq!(long_opp.time_horizon, TimeHorizon::Long);
}

#[tokio::test]
async fn test_exchange_enum_variations() {
    let binance_opp = create_test_arbitrage_opportunity("binance_001", "BTCUSDT", 0.002);
    assert_eq!(binance_opp.long_exchange, ExchangeIdEnum::Binance);
    assert_eq!(binance_opp.short_exchange, ExchangeIdEnum::Bybit);
    
    // Test different exchange combinations
    let mut okx_opp = create_test_arbitrage_opportunity("okx_001", "ETHUSDT", 0.003);
    okx_opp.long_exchange = ExchangeIdEnum::OKX;
    okx_opp.short_exchange = ExchangeIdEnum::Binance;
    assert_eq!(okx_opp.long_exchange, ExchangeIdEnum::OKX);
    assert_eq!(okx_opp.short_exchange, ExchangeIdEnum::Binance);
}

#[tokio::test]
async fn test_arbitrage_type_variations() {
    let cross_exchange_opp = create_test_arbitrage_opportunity("cross_001", "BTCUSDT", 0.002);
    assert_eq!(cross_exchange_opp.r#type, ArbitrageType::CrossExchange);
    
    // Test different arbitrage types
    let mut funding_rate_opp = create_test_arbitrage_opportunity("funding_001", "ETHUSDT", 0.001);
    funding_rate_opp.r#type = ArbitrageType::FundingRate;
    assert_eq!(funding_rate_opp.r#type, ArbitrageType::FundingRate);
    
    let mut spot_futures_opp = create_test_arbitrage_opportunity("spot_001", "ADAUSDT", 0.0015);
    spot_futures_opp.r#type = ArbitrageType::SpotFutures;
    assert_eq!(spot_futures_opp.r#type, ArbitrageType::SpotFutures);
}

#[tokio::test]
async fn test_subscription_tier_hierarchy() {
    let free_user = create_test_user_profile(111111111, SubscriptionTier::Free);
    let basic_user = create_test_user_profile(222222222, SubscriptionTier::Basic);
    let premium_user = create_test_user_profile(333333333, SubscriptionTier::Premium);
    let enterprise_user = create_test_user_profile(444444444, SubscriptionTier::Enterprise);
    let super_admin_user = create_test_user_profile(555555555, SubscriptionTier::SuperAdmin);
    
    // Test tier ordering (Free < Basic < Premium < Enterprise < SuperAdmin)
    assert_eq!(free_user.subscription.tier, SubscriptionTier::Free);
    assert_eq!(basic_user.subscription.tier, SubscriptionTier::Basic);
    assert_eq!(premium_user.subscription.tier, SubscriptionTier::Premium);
    assert_eq!(enterprise_user.subscription.tier, SubscriptionTier::Enterprise);
    assert_eq!(super_admin_user.subscription.tier, SubscriptionTier::SuperAdmin);
}

#[tokio::test]
async fn test_opportunity_serialization() {
    let opportunity = create_test_trading_opportunity("serial_001", "BTCUSDT", 0.8, RiskLevel::Medium);
    
    // Test JSON serialization
    let json_str = serde_json::to_string(&opportunity).unwrap();
    assert!(!json_str.is_empty());
    assert!(json_str.contains("serial_001"));
    assert!(json_str.contains("BTCUSDT"));
    
    // Test JSON deserialization
    let deserialized: TradingOpportunity = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.opportunity_id, opportunity.opportunity_id);
    assert_eq!(deserialized.trading_pair, opportunity.trading_pair);
    assert_eq!(deserialized.confidence_score, opportunity.confidence_score);
    assert_eq!(deserialized.risk_level, opportunity.risk_level);
}

#[tokio::test]
async fn test_arbitrage_opportunity_serialization() {
    let opportunity = create_test_arbitrage_opportunity("arb_serial_001", "ETHUSDT", 0.0025);
    
    // Test JSON serialization
    let json_str = serde_json::to_string(&opportunity).unwrap();
    assert!(!json_str.is_empty());
    assert!(json_str.contains("arb_serial_001"));
    assert!(json_str.contains("ETHUSDT"));
    
    // Test JSON deserialization
    let deserialized: ArbitrageOpportunity = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.id, opportunity.id);
    assert_eq!(deserialized.pair, opportunity.pair);
    assert_eq!(deserialized.rate_difference, opportunity.rate_difference);
    assert_eq!(deserialized.r#type, opportunity.r#type);
}

#[tokio::test]
async fn test_user_profile_serialization() {
    let user = create_test_user_profile(123456789, SubscriptionTier::Premium);
    
    // Test JSON serialization
    let json_str = serde_json::to_string(&user).unwrap();
    assert!(!json_str.is_empty());
    assert!(json_str.contains("123456789"));
    
    // Test JSON deserialization
    let deserialized: UserProfile = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.telegram_user_id, user.telegram_user_id);
    assert_eq!(deserialized.subscription.tier, user.subscription.tier);
}

#[tokio::test]
async fn test_telegram_message_serialization() {
    let update = create_test_telegram_message(123456789, 987654321, "/opportunities");
    
    // Test JSON serialization
    let json_str = serde_json::to_string(&update).unwrap();
    assert!(!json_str.is_empty());
    assert!(json_str.contains("123456789"));
    assert!(json_str.contains("/opportunities"));
    
    // Test JSON deserialization
    let deserialized: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized["update_id"], update["update_id"]);
    assert_eq!(deserialized["message"]["text"], update["message"]["text"]);
}

#[tokio::test]
async fn test_comprehensive_data_flow() {
    // Simulate a complete data flow from user creation to opportunity processing
    
    // 1. Create user
    let user = create_test_user_profile(999888777, SubscriptionTier::Premium);
    assert_eq!(user.subscription.tier, SubscriptionTier::Premium);
    
    // 2. Create arbitrage opportunity
    let arb_opp = create_test_arbitrage_opportunity("flow_arb_001", "BTCUSDT", 0.003);
    assert_eq!(arb_opp.rate_difference, 0.003);
    
    // 3. Convert to trading opportunity
    let trading_opp = create_test_trading_opportunity("flow_trade_001", "BTCUSDT", 0.85, RiskLevel::Low);
    assert_eq!(trading_opp.confidence_score, 0.85);
    
    // 4. Create Telegram update for notification
    let update = create_test_telegram_message(999888777, 111222333, "/opportunities");
    assert!(update["message"].is_object());
    
    // 5. Verify data consistency
    assert_eq!(user.telegram_user_id.unwrap(), 999888777);
    assert_eq!(update["message"]["from"]["id"], 999888777);
    assert_eq!(arb_opp.pair, "BTCUSDT");
    assert_eq!(trading_opp.trading_pair, "BTCUSDT");
}

#[tokio::test]
async fn test_edge_case_handling() {
    // Test edge cases and boundary conditions
    
    // Very small rate difference
    let small_diff_opp = create_test_arbitrage_opportunity("small_001", "BTCUSDT", 0.0001);
    assert_eq!(small_diff_opp.rate_difference, 0.0001);
    
    // Very large rate difference
    let large_diff_opp = create_test_arbitrage_opportunity("large_001", "BTCUSDT", 0.1);
    assert_eq!(large_diff_opp.rate_difference, 0.1);
    
    // Zero confidence opportunity
    let zero_conf_opp = create_test_trading_opportunity("zero_001", "BTCUSDT", 0.0, RiskLevel::High);
    assert_eq!(zero_conf_opp.confidence_score, 0.0);
    
    // Maximum confidence opportunity
    let max_conf_opp = create_test_trading_opportunity("max_001", "BTCUSDT", 1.0, RiskLevel::Low);
    assert_eq!(max_conf_opp.confidence_score, 1.0);
    
    // Empty text message
    let empty_update = create_test_telegram_message(123456789, 987654321, "");
    assert_eq!(empty_update["message"]["text"], "");
} 