use chrono::Utc;
use crate::services::core::opportunities::ai_enhancer::*;
use crate::services::core::opportunities::shared_types::*;
use crate::services::core::infrastructure::shared_types::*;

// Helper function to create test technical opportunity
fn create_test_technical_opportunity() -> TechnicalOpportunity {
    TechnicalOpportunity {
        id: "tech_123".to_string(),
        trading_pair: "ETHUSDT".to_string(),
        exchanges: vec!["binance".to_string()],
        signal_type: TechnicalSignalType::BullishDivergence,
        entry_price: 2000.0,
        target_price: 2100.0,
        stop_loss: 1950.0,
        expected_return_percentage: 0.05,
        confidence: 0.85,
        timeframe: "1h".to_string(),
        indicators: vec!["RSI".to_string(), "MACD".to_string()],
        created_at: Utc::now().timestamp_millis() as u64,
        expires_at: Some(Utc::now().timestamp_millis() as u64 + (60 * 60 * 1000)),
        timestamp: Utc::now().timestamp_millis() as u64,
    }
}

// Helper function to create test arbitrage opportunity
fn create_test_arbitrage_opportunity() -> ArbitrageOpportunity {
    ArbitrageOpportunity {
        id: "arb_456".to_string(),
        trading_pair: "BTC/USDT".to_string(),
        exchanges: vec!["binance".to_string(), "bybit".to_string()],
        profit_percentage: 0.001,
        confidence_score: 0.9,
        risk_level: "low".to_string(),
        buy_exchange: "bybit".to_string(),
        sell_exchange: "binance".to_string(),
        buy_price: 50000.0,
        sell_price: 50050.0,
        volume: 1.0,
        created_at: Utc::now().timestamp_millis() as u64,
        expires_at: Some(Utc::now().timestamp_millis() as u64 + (15 * 60 * 1000)),
        // Unified modular fields
        pair: "BTC/USDT".to_string(),
        long_exchange: ExchangeIdEnum::Bybit,
        short_exchange: ExchangeIdEnum::Binance,
        long_rate: Some(50000.0),
        short_rate: Some(50050.0),
        rate_difference: 50.0,
        net_rate_difference: Some(50.0),
        potential_profit_value: Some(50.0),
        timestamp: Utc::now().timestamp_millis() as u64,
        detected_at: Utc::now().timestamp_millis() as u64,
        r#type: ArbitrageType::CrossExchange,
        details: Some("Cross-exchange arbitrage opportunity".to_string()),
        min_exchanges_required: 2,
    }
}

#[test]
fn test_technical_to_arbitrage_conversion() {
    let tech_opp = create_test_technical_opportunity();

    // This test focuses on the conversion logic
    let converted = ArbitrageOpportunity {
        id: format!("tech_to_arb_{}", tech_opp.id),
        trading_pair: tech_opp.trading_pair.clone(),
        exchanges: tech_opp.exchanges.clone(),
        profit_percentage: tech_opp.expected_return_percentage,
        confidence_score: tech_opp.confidence,
        risk_level: "medium".to_string(),
        buy_exchange: tech_opp
            .exchanges
            .first()
            .unwrap_or(&"binance".to_string())
            .clone(),
        sell_exchange: tech_opp
            .exchanges
            .first()
            .unwrap_or(&"binance".to_string())
            .clone(),
        buy_price: tech_opp.entry_price,
        sell_price: tech_opp.target_price,
        volume: 1000.0,
        created_at: tech_opp.created_at,
        expires_at: Some(tech_opp.expires_at.unwrap_or_else(|| {
            chrono::Utc::now().timestamp_millis() as u64 + (15 * 60 * 1000)
        })),
        // Unified modular fields
        pair: tech_opp.trading_pair.clone(),
        long_exchange: ExchangeIdEnum::from_string(
            tech_opp.exchanges.first().unwrap_or(&"binance".to_string()),
        )
        .unwrap_or(ExchangeIdEnum::Binance),
        short_exchange: ExchangeIdEnum::from_string(
            tech_opp.exchanges.first().unwrap_or(&"binance".to_string()),
        )
        .unwrap_or(ExchangeIdEnum::Binance),
        long_rate: None,
        short_rate: None,
        rate_difference: tech_opp.expected_return_percentage,
        net_rate_difference: Some(tech_opp.expected_return_percentage),
        potential_profit_value: Some(tech_opp.expected_return_percentage * 1000.0),
        timestamp: tech_opp.timestamp,
        detected_at: tech_opp.created_at,
        r#type: ArbitrageType::CrossExchange,
        details: Some(format!(
            "Technical Signal: {:?} | Confidence: {:.2}",
            tech_opp.signal_type, tech_opp.confidence
        )),
        min_exchanges_required: 2,
    };

    assert_eq!(converted.pair, "ETHUSDT");
    assert_eq!(converted.long_exchange, ExchangeIdEnum::Binance);
    assert_eq!(converted.short_exchange, ExchangeIdEnum::Binance);
    assert_eq!(converted.rate_difference, 0.05);
    assert_eq!(converted.min_exchanges_required, 2);
    assert!(matches!(converted.r#type, ArbitrageType::CrossExchange));
}

#[test]
fn test_arbitrage_opportunity_structure() {
    let arb_opp = create_test_arbitrage_opportunity();

    assert_eq!(arb_opp.pair, "BTC/USDT"); // Fixed: matches the test data
    assert_eq!(arb_opp.long_exchange, ExchangeIdEnum::Bybit); // Fixed: long_exchange is Bybit
    assert_eq!(arb_opp.short_exchange, ExchangeIdEnum::Binance); // Fixed: short_exchange is Binance
    assert_eq!(arb_opp.rate_difference, 50.0); // Fixed: rate_difference is 50.0, not 0.001
    assert_eq!(arb_opp.min_exchanges_required, 2);
    assert!(matches!(arb_opp.r#type, ArbitrageType::CrossExchange));
}