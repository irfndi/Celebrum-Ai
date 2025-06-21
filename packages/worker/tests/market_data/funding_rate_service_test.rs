use cerebrum_ai::services::core::market_data::funding_rate_service::FundingRateService;
use cerebrum_ai::types::ExchangeIdEnum;

#[cfg(test)]
mod tests {
    use super::*;
    // ExchangeService + worker Env are intentionally not imported in unit tests
    // to keep the test lightweight and avoid unused-import warnings.

    #[tokio::test]
    async fn test_cache_key() {
        let key = FundingRateService::cache_key(&ExchangeIdEnum::Binance, "BTCUSDT");
        assert_eq!(key, "funding_rate:binance:BTCUSDT");
    }

    #[tokio::test]
    async fn test_cache_key_case_normalization() {
        let key = FundingRateService::cache_key(&ExchangeIdEnum::Binance, "btcusdt");
        assert_eq!(key, "funding_rate:binance:BTCUSDT");
        
        let key = FundingRateService::cache_key(&ExchangeIdEnum::Bybit, "ETH-USDT");
        assert_eq!(key, "funding_rate:bybit:ETH-USDT");
    }

    #[tokio::test]
    async fn test_cache_key_different_exchanges() {
        let binance_key = FundingRateService::cache_key(&ExchangeIdEnum::Binance, "BTCUSDT");
        let bybit_key = FundingRateService::cache_key(&ExchangeIdEnum::Bybit, "BTCUSDT");
        
        assert_ne!(binance_key, bybit_key);
        assert_eq!(binance_key, "funding_rate:binance:BTCUSDT");
        assert_eq!(bybit_key, "funding_rate:bybit:BTCUSDT");
    }

    // Integration tests for live exchanges are excluded from CI to avoid rate limits.
    // Provide a mocked ExchangeService if necessary.  The service intentionally avoids
    // any mock implementation in production code paths.
}