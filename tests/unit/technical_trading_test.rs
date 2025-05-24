// tests/unit/technical_trading_test.rs
// Task 9.3 Tests: Technical Trading Opportunities Generation

use arb_edge::services::technical_trading::{TechnicalTradingService, TechnicalTradingServiceConfig, TechnicalSignal, TradingSignalType, SignalStrength};
use arb_edge::services::market_analysis::{MarketAnalysisService, PricePoint, PriceSeries, OpportunityType, RiskLevel, TimeHorizon};
use arb_edge::services::user_trading_preferences::{UserTradingPreferencesService, TradingFocus, ExperienceLevel, RiskTolerance, AutomationLevel};
use arb_edge::types::ExchangeIdEnum;
use arb_edge::utils::logger::{Logger, LogLevel};
use std::sync::Arc;

fn create_test_logger() -> Logger {
    Logger::new(LogLevel::Info)
}

fn create_test_config() -> TechnicalTradingServiceConfig {
    TechnicalTradingServiceConfig {
        exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
        monitored_pairs: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
        rsi_overbought_threshold: 70.0,
        rsi_oversold_threshold: 30.0,
        rsi_strong_threshold: 80.0,
        ma_short_period: 10,
        ma_long_period: 20,
        bb_period: 20,
        bb_std_dev: 2.0,
        min_confidence_score: 0.6,
        signal_expiry_minutes: 60,
        default_stop_loss_percentage: 0.02, // 2%
        default_take_profit_ratio: 2.0, // 2:1 ratio
    }
}

fn create_test_technical_trading_service() -> TechnicalTradingService {
    let config = create_test_config();
    let logger = create_test_logger();
    
    // Create mock services
    let market_analysis_service = Arc::new(MarketAnalysisService::new());
    let preferences_service = Arc::new(UserTradingPreferencesService::new());

    TechnicalTradingService::new(
        config,
        market_analysis_service,
        preferences_service,
        logger,
    )
}

#[tokio::test]
async fn test_technical_trading_service_creation() {
    let service = create_test_technical_trading_service();
    let config = service.get_config();
    
    assert_eq!(config.exchanges.len(), 2);
    assert_eq!(config.monitored_pairs.len(), 2);
    assert_eq!(config.rsi_overbought_threshold, 70.0);
    assert_eq!(config.rsi_oversold_threshold, 30.0);
    assert_eq!(config.min_confidence_score, 0.6);
    assert_eq!(config.signal_expiry_minutes, 60);
    assert_eq!(config.default_stop_loss_percentage, 0.02);
    assert_eq!(config.default_take_profit_ratio, 2.0);
}

#[tokio::test]
async fn test_default_technical_config() {
    let config = TechnicalTradingServiceConfig::default();
    
    assert_eq!(config.exchanges.len(), 2);
    assert_eq!(config.monitored_pairs.len(), 2);
    assert!(config.monitored_pairs.contains(&"BTC/USDT".to_string()));
    assert!(config.monitored_pairs.contains(&"ETH/USDT".to_string()));
    assert_eq!(config.rsi_overbought_threshold, 70.0);
    assert_eq!(config.rsi_oversold_threshold, 30.0);
    assert_eq!(config.rsi_strong_threshold, 80.0);
    assert_eq!(config.ma_short_period, 10);
    assert_eq!(config.ma_long_period, 20);
    assert_eq!(config.bb_period, 20);
    assert_eq!(config.bb_std_dev, 2.0);
    assert_eq!(config.min_confidence_score, 0.6);
    assert_eq!(config.signal_expiry_minutes, 60);
}

#[tokio::test]
async fn test_rsi_confidence_calculation() {
    let service = create_test_technical_trading_service();
    
    // Test buy signals (oversold RSI)
    let very_oversold_confidence = service.calculate_rsi_confidence(15.0, TradingSignalType::Buy);
    let strongly_oversold_confidence = service.calculate_rsi_confidence(25.0, TradingSignalType::Buy);
    let moderately_oversold_confidence = service.calculate_rsi_confidence(30.0, TradingSignalType::Buy);
    let neutral_confidence = service.calculate_rsi_confidence(50.0, TradingSignalType::Buy);
    
    assert!((very_oversold_confidence - 0.9).abs() < 1e-10);
    assert!((strongly_oversold_confidence - 0.8).abs() < 1e-10);
    assert!((moderately_oversold_confidence - 0.7).abs() < 1e-10);
    assert!((neutral_confidence - 0.5).abs() < 1e-10);
    
    // Test sell signals (overbought RSI)
    let very_overbought_confidence = service.calculate_rsi_confidence(85.0, TradingSignalType::Sell);
    let strongly_overbought_confidence = service.calculate_rsi_confidence(75.0, TradingSignalType::Sell);
    let moderately_overbought_confidence = service.calculate_rsi_confidence(70.0, TradingSignalType::Sell);
    
    assert!((very_overbought_confidence - 0.9).abs() < 1e-10);
    assert!((strongly_overbought_confidence - 0.8).abs() < 1e-10);
    assert!((moderately_overbought_confidence - 0.7).abs() < 1e-10);
}

#[tokio::test]
async fn test_crossover_confidence_calculation() {
    let service = create_test_technical_trading_service();
    
    // Test strong crossover (>2% difference)
    let strong_crossover = service.calculate_crossover_confidence(102.0, 100.0, TradingSignalType::Buy);
    assert!((strong_crossover - 0.8).abs() < 1e-10);
    
    // Test moderate crossover (>1% difference)
    let moderate_crossover = service.calculate_crossover_confidence(101.0, 100.0, TradingSignalType::Buy);
    assert!((moderate_crossover - 0.7).abs() < 1e-10);
    
    // Test weak crossover (<1% difference)
    let weak_crossover = service.calculate_crossover_confidence(100.5, 100.0, TradingSignalType::Buy);
    assert!((weak_crossover - 0.6).abs() < 1e-10);
}

#[tokio::test]
async fn test_bollinger_confidence_calculation() {
    let service = create_test_technical_trading_service();
    
    // Test buy signal (price below lower band)
    let strong_buy_signal = service.calculate_bollinger_confidence(90.0, 95.0, 105.0, TradingSignalType::Buy);
    let weak_buy_signal = service.calculate_bollinger_confidence(94.0, 95.0, 105.0, TradingSignalType::Buy);
    
    assert!((strong_buy_signal - 0.8).abs() < 1e-10); // Well below lower band
    assert!((weak_buy_signal - 0.6).abs() < 1e-10);   // Just touching lower band
    
    // Test sell signal (price above upper band)
    let strong_sell_signal = service.calculate_bollinger_confidence(110.0, 95.0, 105.0, TradingSignalType::Sell);
    let weak_sell_signal = service.calculate_bollinger_confidence(106.0, 95.0, 105.0, TradingSignalType::Sell);
    
    assert!((strong_sell_signal - 0.8).abs() < 1e-10); // Well above upper band
    assert!((weak_sell_signal - 0.6).abs() < 1e-10);   // Just touching upper band
}

#[tokio::test]
async fn test_momentum_confidence_calculation() {
    let service = create_test_technical_trading_service();
    
    // Test strong momentum (>5% change)
    let strong_momentum = service.calculate_momentum_confidence(0.06);
    assert!((strong_momentum - 0.8).abs() < 1e-10);
    
    // Test moderate momentum (>3% change)
    let moderate_momentum = service.calculate_momentum_confidence(0.04);
    assert!((moderate_momentum - 0.7).abs() < 1e-10);
    
    // Test weak momentum (2-3% change)
    let weak_momentum = service.calculate_momentum_confidence(0.025);
    assert!((weak_momentum - 0.6).abs() < 1e-10);
}

#[tokio::test]
async fn test_price_targets_calculation() {
    let service = create_test_technical_trading_service();
    
    // Test buy signal price targets
    let entry_price = 100.0;
    let (buy_target, buy_stop_loss) = service.calculate_price_targets(entry_price, &TradingSignalType::Buy);
    
    assert!(buy_target.is_some());
    assert!(buy_stop_loss.is_some());
    
    let target = buy_target.unwrap();
    let stop_loss = buy_stop_loss.unwrap();
    
    // Expected: stop_loss = 100 * (1 - 0.02) = 98.0
    // Expected: target = 100 * (1 + 0.02 * 2.0) = 104.0
    assert!((stop_loss - 98.0).abs() < 1e-10);
    assert!((target - 104.0).abs() < 1e-10);
    
    // Test sell signal price targets
    let (sell_target, sell_stop_loss) = service.calculate_price_targets(entry_price, &TradingSignalType::Sell);
    
    assert!(sell_target.is_some());
    assert!(sell_stop_loss.is_some());
    
    let sell_target_price = sell_target.unwrap();
    let sell_stop_loss_price = sell_stop_loss.unwrap();
    
    // Expected: stop_loss = 100 * (1 + 0.02) = 102.0
    // Expected: target = 100 * (1 - 0.02 * 2.0) = 96.0
    assert!((sell_stop_loss_price - 102.0).abs() < 1e-10);
    assert!((sell_target_price - 96.0).abs() < 1e-10);
    
    // Test hold signal (no targets)
    let (hold_target, hold_stop_loss) = service.calculate_price_targets(entry_price, &TradingSignalType::Hold);
    assert!(hold_target.is_none());
    assert!(hold_stop_loss.is_none());
}

#[tokio::test]
async fn test_user_preference_filtering() {
    let service = create_test_technical_trading_service();
    
    // Create test signal
    let test_signal = TechnicalSignal {
        signal_id: "test".to_string(),
        exchange_id: "binance".to_string(),
        trading_pair: "BTC/USDT".to_string(),
        signal_type: TradingSignalType::Buy,
        signal_strength: SignalStrength::Strong,
        indicator_source: "RSI_Oversold".to_string(),
        entry_price: 50000.0,
        target_price: Some(51000.0),
        stop_loss: Some(49000.0),
        confidence_score: 0.8,
        created_at: 1640995200000,
        expires_at: 1640998800000,
        metadata: serde_json::json!({}),
    };
    
    // Test beginner user preferences (high confidence required)
    let beginner_prefs = create_test_user_preferences(
        ExperienceLevel::Beginner,
        RiskTolerance::Conservative,
    );
    
    // High confidence signal should pass
    assert!(service.passes_user_preference_filter(&test_signal, &beginner_prefs));
    
    // Low confidence signal should fail for beginners
    let mut low_confidence_signal = test_signal.clone();
    low_confidence_signal.confidence_score = 0.6;
    assert!(!service.passes_user_preference_filter(&low_confidence_signal, &beginner_prefs));
    
    // Test advanced user preferences (lower confidence acceptable)
    let advanced_prefs = create_test_user_preferences(
        ExperienceLevel::Advanced,
        RiskTolerance::Aggressive,
    );
    
    // Lower confidence should pass for advanced users
    let mut moderate_confidence_signal = test_signal.clone();
    moderate_confidence_signal.confidence_score = 0.5;
    assert!(service.passes_user_preference_filter(&moderate_confidence_signal, &advanced_prefs));
}

#[tokio::test]
async fn test_signal_to_opportunity_conversion() {
    let service = create_test_technical_trading_service();
    
    // Create test signal
    let test_signal = TechnicalSignal {
        signal_id: "test-signal-123".to_string(),
        exchange_id: "binance".to_string(),
        trading_pair: "BTC/USDT".to_string(),
        signal_type: TradingSignalType::Buy,
        signal_strength: SignalStrength::Strong,
        indicator_source: "RSI_Oversold".to_string(),
        entry_price: 50000.0,
        target_price: Some(52000.0),
        stop_loss: Some(49000.0),
        confidence_score: 0.8,
        created_at: 1640995200000,
        expires_at: 1640998800000,
        metadata: serde_json::json!({"rsi_value": 25.0}),
    };
    
    let trading_opportunity = service.convert_signal_to_opportunity(test_signal.clone()).await.unwrap();
    
    assert_eq!(trading_opportunity.opportunity_id, test_signal.signal_id);
    assert_eq!(trading_opportunity.opportunity_type, OpportunityType::Technical);
    assert_eq!(trading_opportunity.trading_pair, test_signal.trading_pair);
    assert_eq!(trading_opportunity.exchanges, vec![test_signal.exchange_id]);
    assert!((trading_opportunity.entry_price - test_signal.entry_price).abs() < 1e-10);
    assert_eq!(trading_opportunity.target_price, test_signal.target_price);
    assert_eq!(trading_opportunity.stop_loss, test_signal.stop_loss);
    assert!((trading_opportunity.confidence_score - test_signal.confidence_score).abs() < 1e-10);
    assert_eq!(trading_opportunity.risk_level, RiskLevel::Medium); // Strong signal -> Medium risk
    assert_eq!(trading_opportunity.time_horizon, TimeHorizon::Short); // RSI -> Short term
    assert_eq!(trading_opportunity.indicators_used, vec![test_signal.indicator_source]);
    assert!(trading_opportunity.expires_at.is_some());
    
    // Test expected return calculation for buy signal
    let expected_return = ((52000.0 - 50000.0) / 50000.0) * 100.0; // 4%
    assert!((trading_opportunity.expected_return - expected_return).abs() < 0.001);
}

#[tokio::test]
async fn test_risk_level_assignment() {
    let service = create_test_technical_trading_service();
    
    let base_signal = TechnicalSignal {
        signal_id: "test".to_string(),
        exchange_id: "binance".to_string(),
        trading_pair: "BTC/USDT".to_string(),
        signal_type: TradingSignalType::Buy,
        signal_strength: SignalStrength::Moderate,
        indicator_source: "test".to_string(),
        entry_price: 50000.0,
        target_price: Some(51000.0),
        stop_loss: Some(49000.0),
        confidence_score: 0.7,
        created_at: 1640995200000,
        expires_at: 1640998800000,
        metadata: serde_json::json!({}),
    };
    
            // Test Extreme -> Low risk
    let mut very_strong_signal = base_signal.clone();
            very_strong_signal.signal_strength = SignalStrength::Extreme;
    let very_strong_opp = service.convert_signal_to_opportunity(very_strong_signal).await.unwrap();
    assert_eq!(very_strong_opp.risk_level, RiskLevel::Low);
    
    // Test Strong -> Medium risk
    let mut strong_signal = base_signal.clone();
    strong_signal.signal_strength = SignalStrength::Strong;
    let strong_opp = service.convert_signal_to_opportunity(strong_signal).await.unwrap();
    assert_eq!(strong_opp.risk_level, RiskLevel::Medium);
    
    // Test Moderate -> Medium risk
    let moderate_opp = service.convert_signal_to_opportunity(base_signal.clone()).await.unwrap();
    assert_eq!(moderate_opp.risk_level, RiskLevel::Medium);
    
    // Test Weak -> High risk
    let mut weak_signal = base_signal.clone();
    weak_signal.signal_strength = SignalStrength::Weak;
    let weak_opp = service.convert_signal_to_opportunity(weak_signal).await.unwrap();
    assert_eq!(weak_opp.risk_level, RiskLevel::High);
}

#[tokio::test]
async fn test_time_horizon_assignment() {
    let service = create_test_technical_trading_service();
    
    let base_signal = TechnicalSignal {
        signal_id: "test".to_string(),
        exchange_id: "binance".to_string(),
        trading_pair: "BTC/USDT".to_string(),
        signal_type: TradingSignalType::Buy,
        signal_strength: SignalStrength::Strong,
        indicator_source: "test".to_string(),
        entry_price: 50000.0,
        target_price: Some(51000.0),
        stop_loss: Some(49000.0),
        confidence_score: 0.8,
        created_at: 1640995200000,
        expires_at: 1640998800000,
        metadata: serde_json::json!({}),
    };
    
    // Test RSI -> Short term
    let mut rsi_signal = base_signal.clone();
    rsi_signal.indicator_source = "RSI_Oversold".to_string();
    let rsi_opp = service.convert_signal_to_opportunity(rsi_signal).await.unwrap();
    assert_eq!(rsi_opp.time_horizon, TimeHorizon::Short);
    
    // Test MA -> Medium term
    let mut ma_signal = base_signal.clone();
    ma_signal.indicator_source = "MA_Golden_Cross".to_string();
    let ma_opp = service.convert_signal_to_opportunity(ma_signal).await.unwrap();
    assert_eq!(ma_opp.time_horizon, TimeHorizon::Medium);
    
    // Test BB -> Short term
    let mut bb_signal = base_signal.clone();
    bb_signal.indicator_source = "BB_Lower_Touch".to_string();
    let bb_opp = service.convert_signal_to_opportunity(bb_signal).await.unwrap();
    assert_eq!(bb_opp.time_horizon, TimeHorizon::Short);
    
    // Test Momentum -> Short term
    let mut momentum_signal = base_signal.clone();
    momentum_signal.indicator_source = "Momentum_Signal".to_string();
    let momentum_opp = service.convert_signal_to_opportunity(momentum_signal).await.unwrap();
    assert_eq!(momentum_opp.time_horizon, TimeHorizon::Short);
}

#[tokio::test]
async fn test_trading_signal_type_and_strength_enums() {
    // Test TradingSignalType enum
    assert_eq!(TradingSignalType::Buy, TradingSignalType::Buy);
    assert_ne!(TradingSignalType::Buy, TradingSignalType::Sell);
    assert_ne!(TradingSignalType::Sell, TradingSignalType::Hold);
    
    // Test SignalStrength enum
    assert_eq!(SignalStrength::Weak, SignalStrength::Weak);
    assert_ne!(SignalStrength::Weak, SignalStrength::Moderate);
            assert_ne!(SignalStrength::Strong, SignalStrength::Extreme);
}

#[tokio::test]
async fn test_find_technical_opportunities_empty_result() {
    let service = create_test_technical_trading_service();
    
    let exchange_ids = vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit];
    let pairs = vec!["BTC/USDT".to_string()];
    
    // Test with no user ID (should work without user preferences)
    let opportunities = service.find_technical_opportunities(
        &exchange_ids,
        &pairs,
        None,
    ).await.unwrap();
    
    // Should return empty vector when no price data is available
    // (since we're using mock services that don't return actual data)
    assert_eq!(opportunities.len(), 0);
}

#[tokio::test]
async fn test_find_technical_opportunities_arbitrage_only_user() {
    let service = create_test_technical_trading_service();
    
    let exchange_ids = vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit];
    let pairs = vec!["BTC/USDT".to_string()];
    
    // Test with arbitrage-only user (should skip technical trading)
    let opportunities = service.find_technical_opportunities(
        &exchange_ids,
        &pairs,
        Some("arbitrage_only_user"),
    ).await.unwrap();
    
    // Should return empty vector for arbitrage-only users
    assert_eq!(opportunities.len(), 0);
}

// Helper functions for creating test data

fn create_test_user_preferences(
    experience_level: ExperienceLevel,
    risk_tolerance: RiskTolerance,
) -> arb_edge::services::user_trading_preferences::UserTradingPreferences {
    arb_edge::services::user_trading_preferences::UserTradingPreferences {
        user_id: "test_user".to_string(),
        trading_focus: TradingFocus::Technical, // Technical trading focus
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