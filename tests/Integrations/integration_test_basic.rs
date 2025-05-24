// Basic integration test to verify our test framework works
use arb_edge::{types::*, utils::logger::Logger};

#[tokio::test]
async fn test_basic_integration_framework() {
    // Test basic types and imports work
    println!("✅ Basic integration test framework initialized");
    
    // Test ExchangeIdEnum enum
    let binance = ExchangeIdEnum::Binance;
    let bybit = ExchangeIdEnum::Bybit;
    assert_eq!(binance.as_str(), "binance");
    assert_eq!(bybit.as_str(), "bybit");
    println!("✅ ExchangeIdEnum working correctly");
    
    // Test SubscriptionTier enum
    let free_tier = SubscriptionTier::Free;
    let premium_tier = SubscriptionTier::Premium;
    println!("✅ SubscriptionTier enums working: Free and Premium");
    
    // Test Logger creation
    let logger = Logger::new(arb_edge::utils::logger::LogLevel::Debug);
    println!("✅ Logger created successfully");
    
    // Test UserProfile creation with correct fields
    let user_profile = UserProfile::new(123456789, Some("test-invite".to_string()));
    assert_eq!(user_profile.telegram_user_id, 123456789);
    assert_eq!(user_profile.invitation_code, Some("test-invite".to_string()));
    assert!(user_profile.is_active);
    assert_eq!(user_profile.total_trades, 0);
    println!("✅ UserProfile created with correct fields");
    
    println!("🎉 Basic integration test framework is working!");
}

#[tokio::test] 
async fn test_market_data_structures() {
    // Test Ticker structure
    let ticker = Ticker {
        symbol: "BTCUSDT".to_string(),
        bid: Some(45000.0),
        ask: Some(45001.0),
        last: Some(45000.5),
        high: Some(45200.0),
        low: Some(44800.0),
        volume: Some(1234.567),
        timestamp: Some(chrono::Utc::now()),
        datetime: Some("2024-01-01T00:00:00Z".to_string()),
    };
    
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert!(ticker.bid.unwrap() > 0.0);
    assert!(ticker.ask.unwrap() > ticker.bid.unwrap());
    println!("✅ Ticker structure working correctly");
    
    // Test Market structure
    let market = Market {
        id: "btcusdt".to_string(),
        symbol: "BTC/USDT".to_string(),
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        active: true,
        precision: Precision {
            amount: Some(6),
            price: Some(2),
        },
        limits: Limits {
            amount: MinMax { min: Some(0.001), max: Some(1000.0) },
            price: MinMax { min: Some(0.01), max: Some(100000.0) },
            cost: MinMax { min: Some(1.0), max: Some(100000.0) },
        },
        fees: Some(TradingFee {
            maker: 0.001,
            taker: 0.001,
            percentage: true,
        }),
    };
    
    assert!(market.active);
    assert_eq!(market.base, "BTC");
    assert_eq!(market.quote, "USDT");
    println!("✅ Market structure working correctly");
    
    println!("🎉 Market data structures are working!");
}

#[tokio::test]
async fn test_opportunity_data_structures() {
    // Test ArbitrageOpportunity structure 
    let opportunity = ArbitrageOpportunity::new(
        "BTC/USDT".to_string(),
        Some(ExchangeIdEnum::Binance),
        Some(ExchangeIdEnum::Bybit),
        Some(45000.0),  // long_rate
        Some(45075.0),  // short_rate
        75.0,           // rate_difference
        ArbitrageType::CrossExchange,
    );
    
    assert_eq!(opportunity.pair, "BTC/USDT");
    assert_eq!(opportunity.long_exchange, Some(ExchangeIdEnum::Binance));
    assert_eq!(opportunity.short_exchange, Some(ExchangeIdEnum::Bybit));
    assert_eq!(opportunity.rate_difference, 75.0);
    println!("✅ ArbitrageOpportunity structure working correctly");
    
    // Test with profit calculation
    let opportunity_with_profit = opportunity.with_potential_profit(75.0);
    assert_eq!(opportunity_with_profit.potential_profit_value, Some(75.0));
    println!("✅ ArbitrageOpportunity profit calculation working");
    
    println!("🎉 Opportunity data structures are working!");
} 