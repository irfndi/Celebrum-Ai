use crate::services::core::market_data::market_data_ingestion::*;
use crate::services::core::infrastructure::shared_types::*;

#[test]
fn test_market_data_ingestion_config_creation() {
    let config = MarketDataIngestionConfig::default();
    assert_eq!(config.ingestion_interval_seconds, 30);
    assert_eq!(config.max_ingestion_rate_mb_per_sec, 100);
    assert!(config.enable_funding_rates);
    assert!(config.enable_price_data);
    assert!(config.enable_volume_data);
    assert!(!config.enable_orderbook_snapshots);
}

#[test]
fn test_market_data_snapshot_structure() {
    let snapshot = MarketDataSnapshot {
        exchange: ExchangeIdEnum::Binance,
        symbol: "BTC-USDT".to_string(),
        timestamp: 1640995200000,
        price_data: Some(PriceData {
            price: 50000.0,
            bid: Some(49999.0),
            ask: Some(50001.0),
            high_24h: Some(51000.0),
            low_24h: Some(49000.0),
            change_24h: Some(1000.0),
            change_percentage_24h: Some(2.0),
        }),
        funding_rate_data: None,
        volume_data: None,
        orderbook_data: None,
        source: DataSource::RealAPI,
    };

    assert_eq!(snapshot.exchange, ExchangeIdEnum::Binance);
    assert_eq!(snapshot.symbol, "BTC-USDT");
    assert!(snapshot.price_data.is_some());
    assert!(matches!(snapshot.source, DataSource::RealAPI));
}

#[test]
fn test_price_data_structure() {
    let price_data = PriceData {
        price: 50000.0,
        bid: Some(49999.0),
        ask: Some(50001.0),
        high_24h: Some(51000.0),
        low_24h: Some(49000.0),
        change_24h: Some(1000.0),
        change_percentage_24h: Some(2.0),
    };

    assert_eq!(price_data.price, 50000.0);
    assert_eq!(price_data.bid, Some(49999.0));
    assert_eq!(price_data.ask, Some(50001.0));
    assert_eq!(price_data.change_percentage_24h, Some(2.0));
}

#[test]
fn test_volume_data_structure() {
    let volume_data = VolumeData {
        volume_24h: 1000.0,
        volume_24h_usd: Some(50000000.0),
        trades_count_24h: Some(10000),
    };

    assert_eq!(volume_data.volume_24h, 1000.0);
    assert_eq!(volume_data.volume_24h_usd, Some(50000000.0));
    assert_eq!(volume_data.trades_count_24h, Some(10000));
}

#[test]
fn test_ingestion_metrics_structure() {
    let metrics = IngestionMetrics {
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        cache_hits: 30,
        pipeline_hits: 20,
        api_calls: 45,
        data_volume_mb: 10.5,
        average_latency_ms: 250.0,
        last_ingestion_timestamp: 1640995200000,
    };

    assert_eq!(metrics.total_requests, 100);
    assert_eq!(metrics.successful_requests, 95);
    assert_eq!(metrics.failed_requests, 5);
    assert_eq!(metrics.cache_hits, 30);
    assert_eq!(metrics.pipeline_hits, 20);
    assert_eq!(metrics.api_calls, 45);
}

#[test]
fn test_data_source_enum() {
    assert!(matches!(DataSource::RealAPI, DataSource::RealAPI));
    assert!(matches!(DataSource::Pipeline, DataSource::Pipeline));
    assert!(matches!(DataSource::Cache, DataSource::Cache));
    assert!(matches!(
        DataSource::CoinMarketCap,
        DataSource::CoinMarketCap
    ));
}