use crate::services::core::opportunities::opportunity_builders::*;
use crate::services::core::opportunities::types::*;
use crate::services::core::exchanges::types::ExchangeIdEnum;
use chrono::Utc;

fn create_test_config() -> OpportunityConfig {
    OpportunityConfig::default()
}

#[test]
fn test_funding_rate_arbitrage_builder() {
    let config = create_test_config();
    let builder = OpportunityBuilder::new(config);

    let result = builder.build_funding_rate_arbitrage(
        "BTCUSDT".to_string(),
        ExchangeIdEnum::Binance,
        ExchangeIdEnum::Bybit,
        0.0005,
        0.0015,
        &OpportunityContext::Personal {
            user_id: "test_user".to_string(),
        },
    );

    assert!(result.is_ok());
    let opportunity = result.unwrap();
    assert_eq!(opportunity.pair, "BTCUSDT");
    assert_eq!(opportunity.long_exchange, ExchangeIdEnum::Binance);
    assert_eq!(opportunity.short_exchange, ExchangeIdEnum::Bybit);
    assert_eq!(opportunity.rate_difference, 0.001);
    assert!(matches!(opportunity.r#type, ArbitrageType::FundingRate));
}

#[test]
fn test_technical_opportunity_builder() {
    let config = create_test_config();
    let builder = OpportunityBuilder::new(config);
    let context = OpportunityContext::Personal {
        user_id: "test_user".to_string(),
    };

    let result = builder.build_technical_opportunity(
        "ETHUSDT".to_string(),
        ExchangeIdEnum::Binance,
        TechnicalSignalType::Buy,
        TechnicalSignalStrength::Strong,
        0.85,
        3000.0,                                      // entry_price
        Some(3150.0),                                // target_price
        Some(2950.0),                                // stop_loss_price
        vec!["RSI".to_string(), "MACD".to_string()], // technical_indicators
        "1h".to_string(),                            // timeframe
        0.05,                                        // expected_return_percentage
        "Bullish momentum".to_string(),
        &context,
    );

    assert!(result.is_ok());
    let opportunity = result.unwrap();
    assert_eq!(opportunity.trading_pair, "ETHUSDT");
    assert_eq!(opportunity.exchanges, vec!["binance".to_string()]);
    assert_eq!(opportunity.signal_type, TechnicalSignalType::Buy);
    assert_eq!(opportunity.confidence, 0.85);
    assert_eq!(opportunity.entry_price, 3000.0);
    assert_eq!(opportunity.target_price, 3150.0);
    assert_eq!(opportunity.stop_loss, 2950.0);
    assert_eq!(opportunity.timeframe, "1h");
    assert!(opportunity.expected_return_percentage > 0.0);
}

#[test]
fn test_momentum_opportunity_builder() {
    let config = create_test_config();
    let builder = OpportunityBuilder::new(config);
    let context = OpportunityContext::Personal {
        user_id: "test_user".to_string(),
    };

    let result = builder.build_momentum_opportunity(
        "ADAUSDT".to_string(),
        ExchangeIdEnum::Bybit,
        0.75, // momentum_score
        0.05, // price_change_24h (5%)
        0.20, // volume_change_24h (20%)
        0.5,  // current_price
        &context,
    );

    assert!(result.is_ok());
    let opportunity = result.unwrap();
    assert_eq!(opportunity.trading_pair, "ADAUSDT");
    assert_eq!(opportunity.exchanges, vec!["bybit".to_string()]);
    assert_eq!(opportunity.signal_type, TechnicalSignalType::Buy);
    assert_eq!(opportunity.confidence, 0.605); // Calculated confidence: (0.75*0.5)+(0.5*0.3)+(0.4*0.2) = 0.605
    assert_eq!(opportunity.entry_price, 0.5);
    assert_eq!(opportunity.target_price, 0.575); // 0.5 * (1.0 + 0.15) = 0.575
    assert_eq!(opportunity.stop_loss, 0.4625); // 0.5 * (1.0 - 0.075) = 0.4625
    assert_eq!(opportunity.timeframe, "24h");
    assert!(opportunity.confidence > 0.0);
}

#[test]
fn test_global_opportunity_from_arbitrage() {
    let config = create_test_config();
    let builder = OpportunityBuilder::new(config);

    let arbitrage_opp = ArbitrageOpportunity {
        id: "test_arb_001".to_string(),
        trading_pair: "BTCUSDT".to_string(),
        exchanges: vec!["binance".to_string(), "bybit".to_string()],
        profit_percentage: 0.015,
        confidence_score: 0.85,
        risk_level: "low".to_string(),
        buy_exchange: "binance".to_string(),
        sell_exchange: "bybit".to_string(),
        buy_price: 50000.0,
        sell_price: 50750.0,
        volume: 1000.0,
        created_at: Utc::now().timestamp_millis() as u64,
        expires_at: Some(Utc::now().timestamp_millis() as u64 + (15 * 60 * 1000)),
        pair: "BTCUSDT".to_string(),
        long_exchange: ExchangeIdEnum::Binance,
        short_exchange: ExchangeIdEnum::Bybit,
        long_rate: Some(0.01),
        short_rate: Some(-0.005),
        rate_difference: 0.015,
        net_rate_difference: Some(0.015),
        potential_profit_value: Some(150.0),
        timestamp: Utc::now().timestamp_millis() as u64,
        detected_at: Utc::now().timestamp_millis() as u64,
        r#type: ArbitrageType::FundingRate,
        details: Some("Test arbitrage".to_string()),
        min_exchanges_required: 2,
    };

    let result = builder.build_global_opportunity_from_arbitrage(
        arbitrage_opp,
        OpportunitySource::SystemGenerated,
        Utc::now().timestamp_millis() as u64 + (10 * 60 * 1000),
        Some(100),
        DistributionStrategy::FirstComeFirstServe,
    );

    assert!(result.is_ok());
    let global_opp = result.unwrap();
    assert!(global_opp.id.starts_with("global_arb_"));
    assert_eq!(global_opp.source, OpportunitySource::SystemGenerated);
    assert_eq!(global_opp.max_participants, Some(100));
    assert_eq!(
        global_opp.distribution_strategy,
        DistributionStrategy::FirstComeFirstServe
    );
}

#[test]
fn test_risk_level_determination() {
    let config = create_test_config();
    let builder = OpportunityBuilder::new(config);

    // High risk: high expected return, low confidence
    let high_risk = builder.determine_risk_level(0.2, 0.1, 0.3);
    assert!(matches!(high_risk, TechnicalRiskLevel::High));

    // Low risk: moderate return, high confidence
    let low_risk = builder.determine_risk_level(0.05, 0.02, 0.9);
    assert!(matches!(low_risk, TechnicalRiskLevel::Low));

    // Low risk: moderate values with high confidence (0.045 risk score)
    let low_risk_2 = builder.determine_risk_level(0.1, 0.05, 0.7);
    assert!(matches!(low_risk_2, TechnicalRiskLevel::Low));
}

#[test]
fn test_validation_errors() {
    let config = create_test_config();
    let builder = OpportunityBuilder::new(config);

    // Test rate difference below threshold
    let result = builder.build_funding_rate_arbitrage(
        "BTCUSDT".to_string(),
        ExchangeIdEnum::Binance,
        ExchangeIdEnum::Bybit,
        0.0001,
        0.0002,
        &OpportunityContext::Personal {
            user_id: "test_user".to_string(),
        },
    );
    assert!(result.is_err());

    // Test invalid confidence score
    let result = builder.build_technical_opportunity(
        "ETHUSDT".to_string(),
        ExchangeIdEnum::Binance,
        TechnicalSignalType::Buy,
        TechnicalSignalStrength::Strong,
        1.5, // Invalid: > 1.0
        3500.0,
        Some(3200.0),
        Some(3300.0),
        vec!["Test".to_string()],
        "1h".to_string(),
        0.05,
        "Test market conditions".to_string(),
        &OpportunityContext::Personal {
            user_id: "test_user".to_string(),
        },
    );
    assert!(result.is_err());
}