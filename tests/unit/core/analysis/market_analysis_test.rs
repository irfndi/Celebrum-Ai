// Task 9.1: Technical Indicators Foundation - Unit Tests
// Tests for market analysis service and mathematical foundation

use arb_edge::services::core::analysis::market_analysis::*;

#[tokio::test]
async fn test_price_point_creation() {
    let point = PricePoint {
        timestamp: 1640995200000, // 2022-01-01 00:00:00 UTC
        price: 46000.0,
        volume: Some(1.5),
        exchange_id: "binance".to_string(),
        trading_pair: "BTC/USDT".to_string(),
    };

    assert_eq!(point.timestamp, 1640995200000);
    assert_eq!(point.price, 46000.0);
    assert_eq!(point.volume, Some(1.5));
    assert_eq!(point.exchange_id, "binance");
    assert_eq!(point.trading_pair, "BTC/USDT");
}

#[tokio::test]
async fn test_price_series_operations() {
    let mut series = PriceSeries::new(
        "BTC/USDT".to_string(),
        "binance".to_string(),
        TimeFrame::OneMinute,
    );

    // Test initial state
    assert_eq!(series.trading_pair, "BTC/USDT");
    assert_eq!(series.exchange_id, "binance");
    assert_eq!(series.timeframe, TimeFrame::OneMinute);
    assert!(series.data_points.is_empty());
    assert!(series.latest_price().is_none());

    // Add price points
    let points = vec![
        PricePoint {
            timestamp: 1640995200000,
            price: 46000.0,
            volume: Some(1.0),
            exchange_id: "binance".to_string(),
            trading_pair: "BTC/USDT".to_string(),
        },
        PricePoint {
            timestamp: 1640995260000, // 1 minute later
            price: 46100.0,
            volume: Some(1.2),
            exchange_id: "binance".to_string(),
            trading_pair: "BTC/USDT".to_string(),
        },
        PricePoint {
            timestamp: 1640995320000, // 2 minutes later
            price: 45950.0,
            volume: Some(0.8),
            exchange_id: "binance".to_string(),
            trading_pair: "BTC/USDT".to_string(),
        },
    ];

    for point in points {
        series.add_price_point(point);
    }

    // Test latest price
    assert_eq!(series.data_points.len(), 3);
    assert_eq!(series.latest_price().unwrap().price, 45950.0);

    // Test price values extraction
    let price_values = series.price_values();
    assert_eq!(price_values, vec![46000.0, 46100.0, 45950.0]);

    // Test price range filtering
    let range_points = series.price_range(1640995200000, 1640995300000);
    assert_eq!(range_points.len(), 2); // First two points
}

#[tokio::test]
async fn test_timeframe_durations() {
    assert_eq!(TimeFrame::OneMinute.duration_ms(), 60 * 1000);
    assert_eq!(TimeFrame::FiveMinutes.duration_ms(), 5 * 60 * 1000);
    assert_eq!(TimeFrame::FifteenMinutes.duration_ms(), 15 * 60 * 1000);
    assert_eq!(TimeFrame::OneHour.duration_ms(), 60 * 60 * 1000);
    assert_eq!(TimeFrame::FourHours.duration_ms(), 4 * 60 * 60 * 1000);
    assert_eq!(TimeFrame::OneDay.duration_ms(), 24 * 60 * 60 * 1000);
}

#[tokio::test]
async fn test_simple_moving_average() {
    // Test with known data
    let prices = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0];
    let sma = MathUtils::simple_moving_average(&prices, 3).unwrap();

    assert_eq!(sma.len(), 4); // 6 prices - 3 period + 1
    assert_eq!(sma[0], 12.0); // (10+12+14)/3
    assert_eq!(sma[1], 14.0); // (12+14+16)/3
    assert_eq!(sma[2], 16.0); // (14+16+18)/3
    assert_eq!(sma[3], 18.0); // (16+18+20)/3

    // Test insufficient data
    let short_prices = vec![10.0, 12.0];
    assert!(MathUtils::simple_moving_average(&short_prices, 3).is_err());
}

#[tokio::test]
async fn test_exponential_moving_average() {
    let prices = vec![10.0, 12.0, 14.0, 16.0, 18.0];
    let ema = MathUtils::exponential_moving_average(&prices, 3).unwrap();

    assert_eq!(ema.len(), 5);
    assert_eq!(ema[0], 10.0); // First value is the first price

    // Test the EMA formula: EMA = α * price + (1-α) * previous_EMA
    // where α = 2/(period+1) = 2/4 = 0.5 for period 3
    let alpha = 2.0 / (3.0 + 1.0);
    let expected_ema_1 = alpha * 12.0 + (1.0 - alpha) * 10.0;
    assert!((ema[1] - expected_ema_1).abs() < 0.001);

    // Test empty data
    assert!(MathUtils::exponential_moving_average(&[], 3).is_err());
}

#[tokio::test]
async fn test_standard_deviation() {
    let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let std_dev = MathUtils::standard_deviation(&values).unwrap();

    // Expected standard deviation for this dataset is 2.0
    assert!((std_dev - 2.0).abs() < 0.001);

    // Test empty data
    assert!(MathUtils::standard_deviation(&[]).is_err());

    // Test single value (should be 0)
    let single_value = vec![5.0];
    let std_dev_single = MathUtils::standard_deviation(&single_value).unwrap();
    assert!(std_dev_single < 0.001);
}

#[tokio::test]
async fn test_relative_strength_index() {
    // Test with realistic price data that should produce known RSI values
    let prices = vec![
        44.0, 44.25, 44.5, 43.75, 44.5, 44.0, 44.25, 45.0, 47.0, 46.75, 46.5, 46.25, 47.75, 47.5,
        47.0, 46.5, 46.0, 47.0, 47.25, 48.0,
    ];

    let rsi = MathUtils::relative_strength_index(&prices, 14).unwrap();
    assert!(!rsi.is_empty());

    // RSI should be between 0 and 100
    for value in &rsi {
        assert!(*value >= 0.0 && *value <= 100.0);
    }

    // Test that RSI reflects price momentum
    // After price increases from 44 to 48, RSI should be above 50
    let last_rsi = rsi.last().unwrap();
    assert!(*last_rsi > 50.0);

    // Test insufficient data
    let short_prices = vec![44.0, 44.25, 44.5];
    assert!(MathUtils::relative_strength_index(&short_prices, 14).is_err());
}

#[tokio::test]
async fn test_bollinger_bands() {
    let prices = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0];
    let (upper, middle, lower) = MathUtils::bollinger_bands(&prices, 5, 2.0).unwrap();

    // All bands should have the same length
    assert_eq!(upper.len(), middle.len());
    assert_eq!(middle.len(), lower.len());
    assert_eq!(upper.len(), 4); // 8 prices - 5 period + 1

    // Upper band should be above middle, lower should be below
    for i in 0..upper.len() {
        assert!(
            upper[i] > middle[i],
            "Upper band should be above middle at index {}",
            i
        );
        assert!(
            lower[i] < middle[i],
            "Lower band should be below middle at index {}",
            i
        );
    }

    // Middle band should be SMA
    let sma = MathUtils::simple_moving_average(&prices, 5).unwrap();
    for i in 0..middle.len() {
        assert!((middle[i] - sma[i]).abs() < 0.001);
    }

    // Test insufficient data
    let short_prices = vec![10.0, 12.0];
    assert!(MathUtils::bollinger_bands(&short_prices, 5, 2.0).is_err());
}

#[tokio::test]
async fn test_price_correlation() {
    // Test perfect positive correlation
    let prices1 = vec![10.0, 12.0, 14.0, 16.0, 18.0];
    let prices2 = vec![20.0, 24.0, 28.0, 32.0, 36.0]; // prices1 * 2

    let correlation = MathUtils::price_correlation(&prices1, &prices2).unwrap();
    assert!(
        (correlation - 1.0).abs() < 0.001,
        "Expected perfect positive correlation, got {}",
        correlation
    );

    // Test perfect negative correlation
    let prices3 = vec![20.0, 16.0, 12.0, 8.0, 4.0]; // Inverse of prices1
    let correlation_negative = MathUtils::price_correlation(&prices1, &prices3).unwrap();
    assert!(
        (correlation_negative + 1.0).abs() < 0.001,
        "Expected perfect negative correlation, got {}",
        correlation_negative
    );

    // Test no correlation (random data)
    let prices4 = vec![10.0, 15.0, 8.0, 20.0, 12.0];
    let correlation_random = MathUtils::price_correlation(&prices1, &prices4).unwrap();
    assert!(correlation_random.abs() < 1.0); // Should not be perfect correlation

    // Test error cases
    assert!(MathUtils::price_correlation(&prices1, &[]).is_err()); // Empty array
    assert!(MathUtils::price_correlation(&prices1, &[1.0, 2.0]).is_err()); // Different lengths
}

#[tokio::test]
async fn test_signal_types() {
    // Test signal type serialization
    let signals = vec![
        SignalType::Buy,
        SignalType::Sell,
        SignalType::Hold,
        SignalType::StrongBuy,
        SignalType::StrongSell,
    ];

    for signal in signals {
        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: SignalType = serde_json::from_str(&json).unwrap();
        assert_eq!(signal, deserialized);
    }
}

#[tokio::test]
async fn test_opportunity_types() {
    // Test opportunity type serialization
    let types = vec![
        OpportunityType::Arbitrage,
        OpportunityType::Technical,
        OpportunityType::ArbitrageTechnical,
    ];

    for opp_type in types {
        let json = serde_json::to_string(&opp_type).unwrap();
        let deserialized: OpportunityType = serde_json::from_str(&json).unwrap();
        assert_eq!(opp_type, deserialized);
    }
}

#[tokio::test]
async fn test_risk_levels() {
    let levels = vec![RiskLevel::Low, RiskLevel::Medium, RiskLevel::High];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}

#[tokio::test]
async fn test_time_horizons() {
    let horizons = vec![
        TimeHorizon::Immediate,
        TimeHorizon::Short,
        TimeHorizon::Medium,
        TimeHorizon::Long,
    ];

    for horizon in horizons {
        let json = serde_json::to_string(&horizon).unwrap();
        let deserialized: TimeHorizon = serde_json::from_str(&json).unwrap();
        assert_eq!(horizon, deserialized);
    }
}

#[tokio::test]
async fn test_indicator_result_creation() {
    let indicator_values = vec![
        IndicatorValue {
            timestamp: 1640995200000,
            value: 65.5,
            signal: Some(SignalType::Sell),
        },
        IndicatorValue {
            timestamp: 1640995260000,
            value: 45.2,
            signal: Some(SignalType::Buy),
        },
    ];

    let indicator_result = IndicatorResult {
        indicator_name: "RSI_14".to_string(),
        values: indicator_values,
        metadata: std::collections::HashMap::new(),
        calculated_at: 1640995320000,
    };

    assert_eq!(indicator_result.indicator_name, "RSI_14");
    assert_eq!(indicator_result.values.len(), 2);
    assert_eq!(indicator_result.values[0].value, 65.5);
    assert_eq!(indicator_result.values[0].signal, Some(SignalType::Sell));
    assert_eq!(indicator_result.values[1].signal, Some(SignalType::Buy));
}

#[tokio::test]
async fn test_trading_opportunity_serialization() {
    let opportunity = TradingOpportunity {
        opportunity_id: "opp_123".to_string(),
        opportunity_type: OpportunityType::Arbitrage,
        trading_pair: "BTC/USDT".to_string(),
        exchanges: vec!["binance".to_string(), "coinbase".to_string()],
        entry_price: 46000.0,
        target_price: Some(46500.0),
        stop_loss: Some(45500.0),
        confidence_score: 0.85,
        risk_level: RiskLevel::Low,
        expected_return: 1.08, // 1.08%
        time_horizon: TimeHorizon::Short,
        indicators_used: vec!["SMA_20".to_string(), "RSI_14".to_string()],
        analysis_data: serde_json::json!({"volume_trend": "increasing"}),
        created_at: 1640995200000,
        expires_at: Some(1640995800000), // 10 minutes later
    };

    // Test serialization
    let json = serde_json::to_string(&opportunity).unwrap();
    let deserialized: TradingOpportunity = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.opportunity_id, opportunity.opportunity_id);
    assert_eq!(deserialized.opportunity_type, opportunity.opportunity_type);
    assert_eq!(deserialized.confidence_score, opportunity.confidence_score);
    assert_eq!(deserialized.risk_level, opportunity.risk_level);
}

#[tokio::test]
async fn test_mathematical_edge_cases() {
    // Test SMA with exact period
    let prices = vec![10.0, 12.0, 14.0];
    let sma = MathUtils::simple_moving_average(&prices, 3).unwrap();
    assert_eq!(sma.len(), 1);
    assert_eq!(sma[0], 12.0);

    // Test RSI with minimal data
    let minimal_prices = vec![
        10.0, 11.0, 12.0, 11.0, 10.0, 11.0, 12.0, 13.0, 12.0, 11.0, 10.0, 11.0, 12.0, 13.0, 14.0,
        13.0, // 16 prices for RSI 14
    ];
    let rsi = MathUtils::relative_strength_index(&minimal_prices, 14).unwrap();
    assert_eq!(rsi.len(), 2); // With 16 prices and period 14, should produce 2 RSI values

    // Test correlation with constant values
    let constant1 = vec![5.0, 5.0, 5.0, 5.0, 5.0];
    let constant2 = vec![10.0, 10.0, 10.0, 10.0, 10.0];
    let correlation = MathUtils::price_correlation(&constant1, &constant2).unwrap();
    assert_eq!(correlation, 0.0); // No correlation when no variance
}
