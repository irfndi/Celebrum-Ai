use crate::services::core::market_data::coinmarketcap::*;

#[test]
fn test_cmc_config_creation() {
    let config = CoinMarketCapConfig::default();
    assert_eq!(config.monthly_credit_limit, 10000);
    assert_eq!(config.daily_credit_target, 333);
    assert!(config.priority_symbols.contains(&"BTC".to_string()));
}

#[test]
fn test_quota_usage_structure() {
    let usage = QuotaUsage {
        daily_credits_used: 100,
        monthly_credits_used: 1500,
        last_reset_date: "2025-01-28".to_string(),
        last_monthly_reset: "2025-01".to_string(),
    };

    assert_eq!(usage.daily_credits_used, 100);
    assert_eq!(usage.monthly_credits_used, 1500);
}

#[test]
fn test_cmc_quote_data_structure() {
    let quote = CmcQuoteData {
        symbol: "BTC".to_string(),
        price: 45000.0,
        volume_24h: 1000000000.0,
        percent_change_1h: 0.5,
        percent_change_24h: 2.1,
        percent_change_7d: -1.2,
        market_cap: 850000000000.0,
        last_updated: "2025-01-28T10:00:00Z".to_string(),
    };

    assert_eq!(quote.symbol, "BTC");
    assert_eq!(quote.price, 45000.0);
}