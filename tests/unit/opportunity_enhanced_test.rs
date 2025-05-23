// tests/unit/opportunity_enhanced_test.rs
// Task 9.2 Tests: Enhanced arbitrage detection with technical analysis

use arb_edge::services::opportunity_enhanced::{EnhancedOpportunityService, EnhancedOpportunityServiceConfig};
use arb_edge::services::exchange::ExchangeService;
use arb_edge::services::telegram::TelegramService;
use arb_edge::services::market_analysis::{MarketAnalysisService, PricePoint, PriceSeries, OpportunityType, RiskLevel, TimeHorizon};
use arb_edge::services::user_trading_preferences::{UserTradingPreferencesService, TradingFocus, ExperienceLevel, RiskTolerance, AutomationLevel};
use arb_edge::types::{ExchangeIdEnum, StructuredTradingPair};
use arb_edge::utils::logger::{Logger, LogLevel};
use std::sync::Arc;
use std::collections::HashMap;

fn create_test_logger() -> Logger {
    Logger::new(LogLevel::Info)
}

fn create_test_config() -> EnhancedOpportunityServiceConfig {
    EnhancedOpportunityServiceConfig {
        exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
        monitored_pairs: vec![
            StructuredTradingPair {
                symbol: "BTC/USDT".to_string(),
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
            }
        ],
        threshold: 0.001, // 0.1%
        enable_technical_analysis: true,
        technical_confirmation_weight: 0.3,
        min_technical_confidence: 0.6,
        volatility_threshold: 0.05,
        correlation_threshold: 0.7,
        rsi_overbought: 70.0,
        rsi_oversold: 30.0,
    }
}

fn create_test_enhanced_opportunity_service() -> EnhancedOpportunityService {
    let config = create_test_config();
    let logger = create_test_logger();
    
    // Create mock services
    let exchange_service = Arc::new(ExchangeService::new());
    let telegram_service = Some(Arc::new(TelegramService::new("test_token".to_string())));
    let market_analysis_service = Arc::new(MarketAnalysisService::new());
    let preferences_service = Arc::new(UserTradingPreferencesService::new());

    EnhancedOpportunityService::new(
        config,
        exchange_service,
        telegram_service,
        market_analysis_service,
        preferences_service,
        logger,
    )
}

#[tokio::test]
async fn test_enhanced_opportunity_service_creation() {
    let service = create_test_enhanced_opportunity_service();
    let config = service.get_config();
    
    assert_eq!(config.threshold, 0.001);
    assert!(config.enable_technical_analysis);
    assert_eq!(config.technical_confirmation_weight, 0.3);
    assert_eq!(config.min_technical_confidence, 0.6);
    assert_eq!(config.volatility_threshold, 0.05);
    assert_eq!(config.correlation_threshold, 0.7);
    assert_eq!(config.rsi_overbought, 70.0);
    assert_eq!(config.rsi_oversold, 30.0);
}

#[tokio::test]
async fn test_default_enhanced_config() {
    let config = EnhancedOpportunityServiceConfig::default();
    
    assert_eq!(config.exchanges.len(), 2);
    assert_eq!(config.threshold, 0.0001); // 0.01%
    assert!(config.enable_technical_analysis);
    assert_eq!(config.technical_confirmation_weight, 0.3);
    assert_eq!(config.min_technical_confidence, 0.6);
    assert_eq!(config.volatility_threshold, 0.05);
    assert_eq!(config.correlation_threshold, 0.7);
    assert_eq!(config.rsi_overbought, 70.0);
    assert_eq!(config.rsi_oversold, 30.0);
}

#[tokio::test]
async fn test_rsi_score_calculation() {
    let service = create_test_enhanced_opportunity_service();
    
    // Test extreme RSI values (overbought/oversold)
    let high_rsi_score = service.calculate_rsi_score(75.0); // Overbought
    let low_rsi_score = service.calculate_rsi_score(25.0);  // Oversold
    assert_eq!(high_rsi_score, 0.3);
    assert_eq!(low_rsi_score, 0.3);
    
    // Test neutral RSI range (good for arbitrage)
    let neutral_rsi_score = service.calculate_rsi_score(50.0);
    assert_eq!(neutral_rsi_score, 0.8);
    
    // Test moderate RSI values
    let moderate_rsi_score = service.calculate_rsi_score(65.0);
    assert_eq!(moderate_rsi_score, 0.6);
}

#[tokio::test]
async fn test_volatility_score_calculation() {
    let service = create_test_enhanced_opportunity_service();
    
    // Create test price series with different volatility levels
    
    // Low volatility series (stable prices)
    let stable_prices = vec![100.0, 100.1, 99.9, 100.0, 100.2, 99.8, 100.1, 99.9, 100.0, 100.1];
    let stable_series = create_test_price_series(stable_prices);
    let stable_score = service.calculate_volatility_score(&stable_series).unwrap();
    assert_eq!(stable_score, 0.9); // Low volatility should get high score
    
    // High volatility series (volatile prices)
    let volatile_prices = vec![100.0, 110.0, 90.0, 105.0, 85.0, 115.0, 95.0, 120.0, 80.0, 125.0];
    let volatile_series = create_test_price_series(volatile_prices);
    let volatile_score = service.calculate_volatility_score(&volatile_series).unwrap();
    assert_eq!(volatile_score, 0.2); // High volatility should get low score
    
    // Insufficient data
    let short_prices = vec![100.0, 101.0, 99.0];
    let short_series = create_test_price_series(short_prices);
    let short_score = service.calculate_volatility_score(&short_series).unwrap();
    assert_eq!(short_score, 0.5); // Insufficient data should get neutral score
}

#[tokio::test]
async fn test_arbitrage_confidence_calculation() {
    let service = create_test_enhanced_opportunity_service();
    
    // Create test arbitrage opportunities with different rate differences
    let high_rate_diff_opp = create_test_arbitrage_opportunity(0.005); // 5x threshold
    let high_confidence = service.calculate_arbitrage_confidence(&high_rate_diff_opp);
    assert_eq!(high_confidence, 0.9);
    
    let medium_rate_diff_opp = create_test_arbitrage_opportunity(0.003); // 3x threshold
    let medium_confidence = service.calculate_arbitrage_confidence(&medium_rate_diff_opp);
    assert_eq!(medium_confidence, 0.8);
    
    let low_rate_diff_opp = create_test_arbitrage_opportunity(0.002); // 2x threshold
    let low_confidence = service.calculate_arbitrage_confidence(&low_rate_diff_opp);
    assert_eq!(low_confidence, 0.7);
    
    let minimal_rate_diff_opp = create_test_arbitrage_opportunity(0.001); // 1x threshold
    let minimal_confidence = service.calculate_arbitrage_confidence(&minimal_rate_diff_opp);
    assert_eq!(minimal_confidence, 0.6);
}

#[tokio::test]
async fn test_confidence_scores_combination() {
    let service = create_test_enhanced_opportunity_service();
    
    let arbitrage_score = 0.8;
    let technical_score = 0.6;
    
    // With 30% technical weight: 0.8 * 0.7 + 0.6 * 0.3 = 0.56 + 0.18 = 0.74
    let combined_score = service.combine_confidence_scores(arbitrage_score, technical_score);
    assert!((combined_score - 0.74).abs() < 0.001);
}

#[tokio::test]
async fn test_user_preference_filtering() {
    let service = create_test_enhanced_opportunity_service();
    
    // Test beginner user preferences (high confidence required)
    let beginner_prefs = create_test_user_preferences(
        ExperienceLevel::Beginner,
        RiskTolerance::Conservative,
    );
    
    // High confidence should pass
    assert!(service.passes_user_preference_filter(0.85, 0.8, &beginner_prefs));
    
    // Low confidence should fail
    assert!(!service.passes_user_preference_filter(0.7, 0.6, &beginner_prefs));
    
    // Test advanced user preferences (lower confidence acceptable)
    let advanced_prefs = create_test_user_preferences(
        ExperienceLevel::Advanced,
        RiskTolerance::Aggressive,
    );
    
    // Lower confidence should pass for advanced users
    assert!(service.passes_user_preference_filter(0.5, 0.4, &advanced_prefs));
}

#[tokio::test]
async fn test_trading_opportunity_conversion() {
    let service = create_test_enhanced_opportunity_service();
    
    let arbitrage_opportunity = create_test_arbitrage_opportunity(0.002);
    let confidence_score = 0.75;
    let analysis_type = "arbitrage_technical_score_0.650";
    
    let trading_opportunity = service.convert_arbitrage_to_trading_opportunity(
        arbitrage_opportunity.clone(),
        confidence_score,
        analysis_type,
    ).await.unwrap();
    
    assert_eq!(trading_opportunity.opportunity_id, arbitrage_opportunity.id);
    assert_eq!(trading_opportunity.opportunity_type, OpportunityType::Arbitrage);
    assert_eq!(trading_opportunity.trading_pair, arbitrage_opportunity.pair);
    assert_eq!(trading_opportunity.confidence_score, confidence_score);
    assert_eq!(trading_opportunity.risk_level, RiskLevel::Medium); // 0.75 confidence
    assert_eq!(trading_opportunity.time_horizon, TimeHorizon::Short);
    assert_eq!(trading_opportunity.expected_return, 0.2); // 0.002 * 100
    assert_eq!(trading_opportunity.exchanges.len(), 2);
    assert!(trading_opportunity.indicators_used.contains(&"funding_rate".to_string()));
    assert!(trading_opportunity.indicators_used.contains(&analysis_type.to_string()));
    assert!(trading_opportunity.expires_at.is_some());
}

#[tokio::test]
async fn test_risk_level_assignment() {
    let service = create_test_enhanced_opportunity_service();
    let base_opportunity = create_test_arbitrage_opportunity(0.002);
    
    // High confidence -> Low risk
    let high_conf_opp = service.convert_arbitrage_to_trading_opportunity(
        base_opportunity.clone(),
        0.85,
        "test",
    ).await.unwrap();
    assert_eq!(high_conf_opp.risk_level, RiskLevel::Low);
    
    // Medium confidence -> Medium risk
    let med_conf_opp = service.convert_arbitrage_to_trading_opportunity(
        base_opportunity.clone(),
        0.65,
        "test",
    ).await.unwrap();
    assert_eq!(med_conf_opp.risk_level, RiskLevel::Medium);
    
    // Low confidence -> High risk
    let low_conf_opp = service.convert_arbitrage_to_trading_opportunity(
        base_opportunity.clone(),
        0.45,
        "test",
    ).await.unwrap();
    assert_eq!(low_conf_opp.risk_level, RiskLevel::High);
}

#[tokio::test]
async fn test_enhanced_opportunity_detection_empty_arbitrage() {
    let service = create_test_enhanced_opportunity_service();
    
    let exchange_ids = vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit];
    let pairs = vec!["BTC/USDT".to_string()];
    let threshold = 0.001;
    
    // Test with no user ID (should work without user preferences)
    let opportunities = service.find_enhanced_opportunities(
        &exchange_ids,
        &pairs,
        threshold,
        None,
    ).await.unwrap();
    
    // Should return empty vector when no arbitrage opportunities are found
    // (since we're using mock services that don't return actual data)
    assert_eq!(opportunities.len(), 0);
}

// Helper functions for creating test data

fn create_test_price_series(prices: Vec<f64>) -> PriceSeries {
    let mut data_points = Vec::new();
    let base_timestamp = 1640995200000u64; // 2022-01-01 00:00:00 UTC
    
    for (i, price) in prices.iter().enumerate() {
        data_points.push(PricePoint {
            price: *price,
            timestamp: base_timestamp + (i as u64 * 60000), // 1-minute intervals
            volume: Some(1000.0),
        });
    }
    
    PriceSeries {
        exchange_id: "test_exchange".to_string(),
        trading_pair: "BTC/USDT".to_string(),
        data_points,
        last_updated: base_timestamp + (prices.len() as u64 * 60000),
    }
}

fn create_test_arbitrage_opportunity(rate_difference: f64) -> arb_edge::types::ArbitrageOpportunity {
    arb_edge::types::ArbitrageOpportunity {
        id: uuid::Uuid::new_v4().to_string(),
        pair: "BTC/USDT".to_string(),
        r#type: arb_edge::types::ArbitrageType::FundingRate,
        long_exchange: Some(ExchangeIdEnum::Binance),
        short_exchange: Some(ExchangeIdEnum::Bybit),
        long_rate: Some(0.0001),
        short_rate: Some(0.0001 + rate_difference),
        rate_difference,
        net_rate_difference: Some(rate_difference),
        potential_profit_value: None,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        details: Some("Test arbitrage opportunity".to_string()),
    }
}

fn create_test_user_preferences(
    experience_level: ExperienceLevel,
    risk_tolerance: RiskTolerance,
) -> arb_edge::services::user_trading_preferences::UserTradingPreferences {
    arb_edge::services::user_trading_preferences::UserTradingPreferences {
        user_id: "test_user".to_string(),
        trading_focus: TradingFocus::Arbitrage,
        automation_level: AutomationLevel::Manual,
        automation_scope: vec![],
        experience_level,
        risk_tolerance,
        max_position_size: Some(1000.0),
        max_daily_trades: Some(10),
        preferred_exchanges: vec![],
        excluded_pairs: vec![],
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        updated_at: chrono::Utc::now().timestamp_millis() as u64,
    }
} 