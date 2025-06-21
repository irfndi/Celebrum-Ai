//! Tests for cache_manager module
//! Extracted from src/services/core/opportunities/cache_manager.rs

#[cfg(test)]
mod tests {
    use cerebrum_ai::services::core::opportunities::cache_manager::{OpportunityDataCache, CachePrefixes};
    use cerebrum_ai::types::{ArbitrageOpportunity, TechnicalOpportunity, ArbitrageType, ExchangeIdEnum, TechnicalRiskLevel, TechnicalSignalType};

    fn create_test_arbitrage_opportunity() -> ArbitrageOpportunity {
        let buy_exchange_enum = ExchangeIdEnum::Binance;
        let sell_exchange_enum = ExchangeIdEnum::Bybit;
        ArbitrageOpportunity {
            id: "test_arb_1".to_string(),
            trading_pair: "BTCUSDT".to_string(), // from pair
            exchanges: vec![
                buy_exchange_enum.as_str().to_string(),
                sell_exchange_enum.as_str().to_string(),
            ],
            profit_percentage: 0.01, // Calculated from prices or default
            confidence_score: 0.85,  // from confidence
            risk_level: "low".to_string(), // Default test value
            buy_exchange: buy_exchange_enum.as_str().to_string(), // from long_exchange
            sell_exchange: sell_exchange_enum.as_str().to_string(), // from short_exchange
            buy_price: 50000.0,      // Default test value
            sell_price: 50050.0,     // Default test value, implies 0.1% profit before fees
            volume: 1000.0,
            created_at: 1234567890,
            expires_at: Some(1234567890 + 60000),
            // Aliases and additional fields
            pair: "BTCUSDT".to_string(),
            long_exchange: buy_exchange_enum,
            short_exchange: sell_exchange_enum,
            long_rate: Some(0.0001),
            short_rate: Some(0.0002),
            rate_difference: 0.0001,
            net_rate_difference: Some(0.0001),
            potential_profit_value: Some(10.0),
            timestamp: 1234567890,
            detected_at: 1234567890,
            r#type: ArbitrageType::CrossExchange,
            details: Some("Test arbitrage opportunity".to_string()),
            min_exchanges_required: 2,
        }
    }

    fn create_test_technical_opportunity() -> TechnicalOpportunity {
        TechnicalOpportunity {
            id: "test_tech_1".to_string(),
            trading_pair: "ETHUSDT".to_string(), // Added from symbol
            exchanges: vec![ExchangeIdEnum::Binance.as_str().to_string()], // Added from exchange
            signal_type: TechnicalSignalType::Buy,
            confidence: 0.85,
            risk_level: "medium".to_string(), // Converted from TechnicalRiskLevel::Medium
            entry_price: 3000.0,
            target_price: 3150.0,
            stop_loss: 2950.0,
            created_at: 1234567890,
            expires_at: Some(1234567890 + 60000),
            pair: "ETHUSDT".to_string(),
            expected_return_percentage: 0.05,
            details: Some("Strong buy signal".to_string()),
            timestamp: 1234567890,
            metadata: serde_json::json!({"signal_strength": "strong"}),
            // Unified modular fields (legacy aliases removed)
            timeframe: "1h".to_string(),
            indicators: serde_json::json!({"RSI": 70, "MACD": "bullish"}),
        }
    }

    #[test]
    fn test_cache_prefixes_default() {
        let prefixes = CachePrefixes::default();

        assert_eq!(prefixes.arbitrage_opportunities, "arb_opp");
        assert_eq!(prefixes.technical_opportunities, "tech_opp");
        assert_eq!(prefixes.global_opportunities, "global_opp");
        assert_eq!(prefixes.user_opportunities, "user_opp");
        assert_eq!(prefixes.group_opportunities, "group_opp");
        assert_eq!(prefixes.market_data, "market_data");
        assert_eq!(prefixes.funding_rates, "funding_rates");
        assert_eq!(prefixes.distribution_stats, "dist_stats");
        assert_eq!(prefixes.user_access, "user_access");
    }

    #[test]
    fn test_cache_key_generation() {
        let prefixes = CachePrefixes::default();

        // Test user arbitrage cache key
        let user_arb_key = format!(
            "{}:user:{}:arbitrage",
            prefixes.user_opportunities, "user123"
        );
        assert_eq!(user_arb_key, "user_opp:user:user123:arbitrage");

        // Test market data cache key
        let market_key = format!("{}:{}:{}", prefixes.market_data, "binance", "BTCUSDT");
        assert_eq!(market_key, "market_data:binance:BTCUSDT");

        // Test distribution stats cache key
        let stats_key = format!("{}:{}", prefixes.distribution_stats, "opportunities_today");
        assert_eq!(stats_key, "dist_stats:opportunities_today");
    }

    #[test]
    fn test_opportunity_structures() {
        let arb_opp = create_test_arbitrage_opportunity();
        let tech_opp = create_test_technical_opportunity();

        // Test arbitrage opportunity
        assert_eq!(arb_opp.pair, "BTCUSDT");
        assert_eq!(arb_opp.min_exchanges_required, 2);
        assert!(matches!(arb_opp.r#type, ArbitrageType::CrossExchange));

        // Test technical opportunity
        assert_eq!(tech_opp.trading_pair, "ETHUSDT");
        assert_eq!(tech_opp.confidence, 0.85);
        assert!(matches!(tech_opp.signal_type, TechnicalSignalType::Buy));
        assert_eq!(tech_opp.risk_level, TechnicalRiskLevel::Medium.as_str());
    }
}