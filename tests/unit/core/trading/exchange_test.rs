#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    dead_code,
    clippy::needless_range_loop
)]

// ExchangeService Unit Tests
// Comprehensive testing of market data fetching, authentication, API management, and error handling

use arb_edge::types::{Limits, Market, MinMax, OrderBook, Precision, Ticker};
use arb_edge::utils::{ArbitrageError, ArbitrageResult};
use serde_json::{json, Value};
use std::collections::HashMap;

// Mock KV Store for testing
struct MockKvStore {
    data: HashMap<String, String>,
    error_simulation: Option<String>,
    operation_count: u32,
}

impl MockKvStore {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            error_simulation: None,
            operation_count: 0,
        }
    }

    fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    fn reset_error_simulation(&mut self) {
        self.error_simulation = None;
    }

    async fn mock_put(&mut self, key: &str, value: &str) -> ArbitrageResult<()> {
        self.operation_count += 1;

        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_put_failed" => Err(ArbitrageError::database_error("KV put operation failed")),
                "rate_limit" => Err(ArbitrageError::rate_limit_error("Rate limit exceeded")),
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }

        self.data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn mock_get(&self, key: &str) -> ArbitrageResult<Option<String>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_get_failed" => Err(ArbitrageError::database_error("KV get operation failed")),
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }

        Ok(self.data.get(key).cloned())
    }

    fn get_operation_count(&self) -> u32 {
        self.operation_count
    }

    fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

// Mock HTTP Client for testing
struct MockHttpClient {
    responses: HashMap<String, Value>,
    error_simulation: Option<String>,
    request_count: u32,
    rate_limit_count: u32,
    rate_limit_threshold: u32,
}

impl MockHttpClient {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
            error_simulation: None,
            request_count: 0,
            rate_limit_count: 0,
            rate_limit_threshold: 10, // Default rate limit
        }
    }

    fn add_response(&mut self, endpoint: &str, response: Value) {
        self.responses.insert(endpoint.to_string(), response);
    }

    fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    fn set_rate_limit_threshold(&mut self, threshold: u32) {
        self.rate_limit_threshold = threshold;
    }

    async fn mock_request(&mut self, endpoint: &str) -> ArbitrageResult<Value> {
        self.request_count += 1;
        self.rate_limit_count += 1;

        // Rate limiting simulation
        if self.rate_limit_count > self.rate_limit_threshold {
            return Err(ArbitrageError::rate_limit_error("Rate limit exceeded"));
        }

        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "network_error" => Err(ArbitrageError::network_error("Network connection failed")),
                "timeout" => Err(ArbitrageError::network_error("Request timeout")),
                "invalid_response" => Err(ArbitrageError::parse_error("Invalid JSON response")),
                "auth_failed" => Err(ArbitrageError::authentication_error(
                    "Authentication failed",
                )),
                _ => Err(ArbitrageError::validation_error("Unknown HTTP error")),
            };
        }

        self.responses.get(endpoint).cloned().ok_or_else(|| {
            ArbitrageError::not_found(format!("No mock response for endpoint: {}", endpoint))
        })
    }

    fn get_request_count(&self) -> u32 {
        self.request_count
    }

    fn reset_rate_limit(&mut self) {
        self.rate_limit_count = 0;
    }
}

// Mock ExchangeService for testing
struct MockExchangeService {
    kv_store: MockKvStore,
    http_client: MockHttpClient,
    super_admin_configs: HashMap<String, MockSuperAdminConfig>,
    supported_exchanges: Vec<String>,
    api_key_validation_enabled: bool,
}

#[derive(Debug, Clone)]
struct MockSuperAdminConfig {
    exchange_id: String,
    api_key: String,
    secret: String,
    is_read_only: bool,
}

impl MockExchangeService {
    fn new() -> Self {
        Self {
            kv_store: MockKvStore::new(),
            http_client: MockHttpClient::new(),
            super_admin_configs: HashMap::new(),
            supported_exchanges: vec![
                "binance".to_string(),
                "bybit".to_string(),
                "okx".to_string(),
            ],
            api_key_validation_enabled: true,
        }
    }

    fn add_super_admin_config(&mut self, exchange_id: &str, api_key: &str, secret: &str) {
        let config = MockSuperAdminConfig {
            exchange_id: exchange_id.to_string(),
            api_key: api_key.to_string(),
            secret: secret.to_string(),
            is_read_only: true,
        };
        self.super_admin_configs
            .insert(exchange_id.to_string(), config);
    }

    async fn mock_get_ticker(
        &mut self,
        exchange_id: &str,
        symbol: &str,
    ) -> ArbitrageResult<Ticker> {
        // Validate exchange support
        if !self.supported_exchanges.contains(&exchange_id.to_string()) {
            return Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                exchange_id
            )));
        }

        // Check cache first
        let cache_key = format!("ticker:{}:{}", exchange_id, symbol);
        if let Ok(Some(cached_data)) = self.kv_store.mock_get(&cache_key).await {
            if let Ok(ticker) = serde_json::from_str::<Ticker>(&cached_data) {
                return Ok(ticker);
            }
        }

        // Fetch from API
        let endpoint = format!("/api/v3/ticker/24hr?symbol={}", symbol);
        let response = self.http_client.mock_request(&endpoint).await?;

        let ticker = self.parse_ticker_response(exchange_id, &response, symbol)?;

        // Cache the result
        let ticker_json = serde_json::to_string(&ticker).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize ticker: {}", e))
        })?;
        self.kv_store.mock_put(&cache_key, &ticker_json).await?;

        Ok(ticker)
    }

    async fn mock_get_orderbook(
        &mut self,
        exchange_id: &str,
        symbol: &str,
        limit: Option<u32>,
    ) -> ArbitrageResult<OrderBook> {
        if !self.supported_exchanges.contains(&exchange_id.to_string()) {
            return Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                exchange_id
            )));
        }

        let limit_param = limit.unwrap_or(100);
        let endpoint = format!("/api/v3/depth?symbol={}&limit={}", symbol, limit_param);
        let response = self.http_client.mock_request(&endpoint).await?;

        self.parse_orderbook_response(exchange_id, &response, symbol)
    }

    async fn mock_get_markets(&mut self, exchange_id: &str) -> ArbitrageResult<Vec<Market>> {
        if !self.supported_exchanges.contains(&exchange_id.to_string()) {
            return Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                exchange_id
            )));
        }

        let endpoint = "/api/v3/exchangeInfo";
        let response = self.http_client.mock_request(endpoint).await?;

        self.parse_markets_response(exchange_id, &response)
    }

    async fn mock_test_api_connection(
        &mut self,
        exchange_id: &str,
        api_key: &str,
        secret: &str,
    ) -> ArbitrageResult<Value> {
        if !self.api_key_validation_enabled {
            return Ok(json!({"status": "ok", "message": "API validation disabled"}));
        }

        if api_key.is_empty() || secret.is_empty() {
            return Err(ArbitrageError::authentication_error(
                "API key and secret are required",
            ));
        }

        // Simulate API key validation
        let endpoint = "/api/v3/account";
        let response = self.http_client.mock_request(endpoint).await?;

        Ok(json!({
            "status": "ok",
            "exchange": exchange_id,
            "permissions": ["SPOT", "FUTURES"],
            "response": response
        }))
    }

    async fn mock_validate_user_api_compatibility(
        &self,
        user_exchanges: &[String],
        required_exchanges: &[String],
    ) -> ArbitrageResult<bool> {
        for required in required_exchanges {
            if !user_exchanges.contains(required) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn parse_ticker_response(
        &self,
        exchange_id: &str,
        response: &Value,
        symbol: &str,
    ) -> ArbitrageResult<Ticker> {
        match exchange_id {
            "binance" => self.parse_binance_ticker(response, symbol),
            "bybit" => self.parse_bybit_ticker(response, symbol),
            _ => Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange for ticker parsing: {}",
                exchange_id
            ))),
        }
    }

    fn parse_binance_ticker(&self, data: &Value, symbol: &str) -> ArbitrageResult<Ticker> {
        let price = data["lastPrice"]
            .as_str()
            .ok_or_else(|| ArbitrageError::parse_error("Missing lastPrice in Binance ticker"))?
            .parse::<f64>()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid price format: {}", e)))?;

        let volume = data["volume"]
            .as_str()
            .ok_or_else(|| ArbitrageError::parse_error("Missing volume in Binance ticker"))?
            .parse::<f64>()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid volume format: {}", e)))?;

        let change_24h = data["priceChangePercent"]
            .as_str()
            .ok_or_else(|| {
                ArbitrageError::parse_error("Missing priceChangePercent in Binance ticker")
            })?
            .parse::<f64>()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid change format: {}", e)))?;

        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: None,
            ask: None,
            last: Some(price),
            high: None,
            low: None,
            volume: Some(volume),
            timestamp: Some(chrono::Utc::now()),
            datetime: None,
        })
    }

    fn parse_bybit_ticker(&self, data: &Value, symbol: &str) -> ArbitrageResult<Ticker> {
        let result = data["result"]
            .as_object()
            .ok_or_else(|| ArbitrageError::parse_error("Missing result object in Bybit ticker"))?;

        let price = result["lastPrice"]
            .as_str()
            .ok_or_else(|| ArbitrageError::parse_error("Missing lastPrice in Bybit ticker"))?
            .parse::<f64>()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid price format: {}", e)))?;

        let volume = result["volume24h"]
            .as_str()
            .ok_or_else(|| ArbitrageError::parse_error("Missing volume24h in Bybit ticker"))?
            .parse::<f64>()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid volume format: {}", e)))?;

        let change_24h = result["price24hPcnt"]
            .as_str()
            .ok_or_else(|| ArbitrageError::parse_error("Missing price24hPcnt in Bybit ticker"))?
            .parse::<f64>()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid change format: {}", e)))?;

        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: None,
            ask: None,
            last: Some(price),
            high: None,
            low: None,
            volume: Some(volume),
            timestamp: Some(chrono::Utc::now()),
            datetime: None,
        })
    }

    fn parse_orderbook_response(
        &self,
        exchange_id: &str,
        response: &Value,
        symbol: &str,
    ) -> ArbitrageResult<OrderBook> {
        let bids_array = response["bids"]
            .as_array()
            .ok_or_else(|| ArbitrageError::parse_error("Missing bids array in orderbook"))?;

        let asks_array = response["asks"]
            .as_array()
            .ok_or_else(|| ArbitrageError::parse_error("Missing asks array in orderbook"))?;

        let mut bids = Vec::new();
        for bid in bids_array.iter().take(20) {
            // Limit to top 20
            let bid_array = bid
                .as_array()
                .ok_or_else(|| ArbitrageError::parse_error("Invalid bid format"))?;

            if bid_array.len() >= 2 {
                let price = bid_array[0]
                    .as_str()
                    .ok_or_else(|| ArbitrageError::parse_error("Invalid bid price"))?
                    .parse::<f64>()
                    .map_err(|e| {
                        ArbitrageError::parse_error(format!("Invalid bid price format: {}", e))
                    })?;

                let quantity = bid_array[1]
                    .as_str()
                    .ok_or_else(|| ArbitrageError::parse_error("Invalid bid quantity"))?
                    .parse::<f64>()
                    .map_err(|e| {
                        ArbitrageError::parse_error(format!("Invalid bid quantity format: {}", e))
                    })?;

                bids.push([price, quantity]);
            }
        }

        let mut asks = Vec::new();
        for ask in asks_array.iter().take(20) {
            // Limit to top 20
            let ask_array = ask
                .as_array()
                .ok_or_else(|| ArbitrageError::parse_error("Invalid ask format"))?;

            if ask_array.len() >= 2 {
                let price = ask_array[0]
                    .as_str()
                    .ok_or_else(|| ArbitrageError::parse_error("Invalid ask price"))?
                    .parse::<f64>()
                    .map_err(|e| {
                        ArbitrageError::parse_error(format!("Invalid ask price format: {}", e))
                    })?;

                let quantity = ask_array[1]
                    .as_str()
                    .ok_or_else(|| ArbitrageError::parse_error("Invalid ask quantity"))?
                    .parse::<f64>()
                    .map_err(|e| {
                        ArbitrageError::parse_error(format!("Invalid ask quantity format: {}", e))
                    })?;

                asks.push([price, quantity]);
            }
        }

        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids,
            asks,
            timestamp: Some(chrono::Utc::now()),
            datetime: None,
        })
    }

    fn parse_markets_response(
        &self,
        _exchange_id: &str,
        response: &Value,
    ) -> ArbitrageResult<Vec<Market>> {
        let symbols = response["symbols"]
            .as_array()
            .ok_or_else(|| ArbitrageError::parse_error("Missing symbols array in exchange info"))?;

        let mut markets = Vec::new();
        for symbol_data in symbols.iter().take(50) {
            // Limit for testing
            let symbol = symbol_data["symbol"]
                .as_str()
                .ok_or_else(|| ArbitrageError::parse_error("Missing symbol in market data"))?;

            let status = symbol_data["status"]
                .as_str()
                .ok_or_else(|| ArbitrageError::parse_error("Missing status in market data"))?;

            let base_asset = symbol_data["baseAsset"]
                .as_str()
                .ok_or_else(|| ArbitrageError::parse_error("Missing baseAsset in market data"))?;

            let quote_asset = symbol_data["quoteAsset"]
                .as_str()
                .ok_or_else(|| ArbitrageError::parse_error("Missing quoteAsset in market data"))?;

            if status == "TRADING" {
                markets.push(Market {
                    id: symbol.to_string(),
                    symbol: symbol.to_string(),
                    base: base_asset.to_string(),
                    quote: quote_asset.to_string(),
                    active: true,
                    precision: Precision {
                        amount: Some(3),
                        price: Some(2),
                    },
                    limits: Limits {
                        amount: MinMax {
                            min: Some(0.001),
                            max: Some(1000000.0),
                        },
                        price: MinMax {
                            min: Some(0.01),
                            max: Some(100000.0),
                        },
                        cost: MinMax {
                            min: Some(1.0),
                            max: Some(1000000.0),
                        },
                    },
                    fees: None,
                });
            }
        }

        Ok(markets)
    }

    fn validate_exchange_support(&self, exchange_id: &str) -> ArbitrageResult<()> {
        if !self.supported_exchanges.contains(&exchange_id.to_string()) {
            return Err(ArbitrageError::validation_error(format!(
                "Exchange '{}' is not supported",
                exchange_id
            )));
        }
        Ok(())
    }

    fn validate_symbol_format(&self, symbol: &str) -> ArbitrageResult<()> {
        if symbol.is_empty() {
            return Err(ArbitrageError::validation_error("Symbol cannot be empty"));
        }

        if symbol.len() < 3 || symbol.len() > 20 {
            return Err(ArbitrageError::validation_error(
                "Symbol length must be between 3 and 20 characters",
            ));
        }

        if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ArbitrageError::validation_error(
                "Symbol must contain only alphanumeric characters",
            ));
        }

        Ok(())
    }

    async fn mock_rate_limit_check(&mut self, operation: &str) -> ArbitrageResult<()> {
        let current_count = self.http_client.get_request_count();
        let threshold = self.http_client.rate_limit_threshold;

        if current_count >= threshold {
            return Err(ArbitrageError::rate_limit_error(format!(
                "Rate limit exceeded for operation '{}': {}/{}",
                operation, current_count, threshold
            )));
        }

        Ok(())
    }

    fn get_performance_metrics(&self) -> MockPerformanceMetrics {
        MockPerformanceMetrics {
            total_requests: self.http_client.get_request_count(),
            cache_operations: self.kv_store.get_operation_count(),
            supported_exchanges: self.supported_exchanges.len() as u32,
            super_admin_configs: self.super_admin_configs.len() as u32,
        }
    }
}

#[derive(Debug)]
struct MockPerformanceMetrics {
    total_requests: u32,
    cache_operations: u32,
    supported_exchanges: u32,
    super_admin_configs: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_fetching_and_parsing() {
        let mut service = MockExchangeService::new();

        // Setup mock response for Binance ticker
        let binance_ticker_response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.50",
            "volume": "1234.56",
            "priceChangePercent": "2.5"
        });
        service.http_client.add_response(
            "/api/v3/ticker/24hr?symbol=BTCUSDT",
            binance_ticker_response,
        );

        // Test successful ticker fetch
        let ticker_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(ticker_result.is_ok());

        let ticker = ticker_result.unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.last, Some(45000.50));
        assert_eq!(ticker.volume, Some(1234.56));
        assert!(ticker.timestamp.is_some());

        // Test unsupported exchange
        let unsupported_result = service.mock_get_ticker("unsupported", "BTCUSDT").await;
        assert!(unsupported_result.is_err());
        assert!(unsupported_result
            .unwrap_err()
            .to_string()
            .contains("Unsupported exchange"));

        // Test caching behavior (second call should use cache)
        let cached_ticker = service.mock_get_ticker("binance", "BTCUSDT").await.unwrap();
        assert_eq!(cached_ticker.symbol, ticker.symbol);
        assert_eq!(cached_ticker.last, ticker.last);
    }

    #[tokio::test]
    async fn test_orderbook_processing() {
        let mut service = MockExchangeService::new();

        // Setup mock orderbook response
        let orderbook_response = json!({
            "bids": [
                ["44999.50", "0.5"],
                ["44999.00", "1.0"],
                ["44998.50", "2.0"]
            ],
            "asks": [
                ["45000.50", "0.3"],
                ["45001.00", "0.8"],
                ["45001.50", "1.5"]
            ]
        });
        service.http_client.add_response(
            "/api/v3/depth?symbol=BTCUSDT&limit=100",
            orderbook_response.clone(),
        );

        // Test successful orderbook fetch
        let orderbook_result = service.mock_get_orderbook("binance", "BTCUSDT", None).await;
        assert!(orderbook_result.is_ok());

        let orderbook = orderbook_result.unwrap();
        assert_eq!(orderbook.symbol, "BTCUSDT");
        assert_eq!(orderbook.bids.len(), 3);
        assert_eq!(orderbook.asks.len(), 3);

        // Verify bid/ask ordering and values
        assert_eq!(orderbook.bids[0][0], 44999.50); // Highest bid
        assert_eq!(orderbook.bids[0][1], 0.5);
        assert_eq!(orderbook.asks[0][0], 45000.50); // Lowest ask
        assert_eq!(orderbook.asks[0][1], 0.3);

        // Test with custom limit
        service
            .http_client
            .add_response("/api/v3/depth?symbol=ETHUSDT&limit=50", orderbook_response);
        let limited_orderbook = service
            .mock_get_orderbook("binance", "ETHUSDT", Some(50))
            .await;
        assert!(limited_orderbook.is_ok());
    }

    #[tokio::test]
    async fn test_authentication_and_api_key_management() {
        let mut service = MockExchangeService::new();

        // Setup mock authentication response
        let auth_response = json!({
            "accountType": "SPOT",
            "balances": [],
            "permissions": ["SPOT", "FUTURES"]
        });
        service
            .http_client
            .add_response("/api/v3/account", auth_response);

        // Test successful API key validation
        let api_key = "test_api_key_123";
        let secret = "test_secret_456";
        let connection_result = service
            .mock_test_api_connection("binance", api_key, secret)
            .await;
        assert!(connection_result.is_ok());

        let response = connection_result.unwrap();
        assert_eq!(response["status"], "ok");
        assert_eq!(response["exchange"], "binance");

        // Test empty API key validation
        let empty_key_result = service
            .mock_test_api_connection("binance", "", secret)
            .await;
        assert!(empty_key_result.is_err());
        assert!(empty_key_result
            .unwrap_err()
            .to_string()
            .contains("API key and secret are required"));

        // Test empty secret validation
        let empty_secret_result = service
            .mock_test_api_connection("binance", api_key, "")
            .await;
        assert!(empty_secret_result.is_err());
        assert!(empty_secret_result
            .unwrap_err()
            .to_string()
            .contains("API key and secret are required"));

        // Test super admin config
        service.add_super_admin_config("binance", "admin_key", "admin_secret");
        assert!(service.super_admin_configs.contains_key("binance"));
        assert!(service.super_admin_configs["binance"].is_read_only);
    }

    #[tokio::test]
    async fn test_rate_limiting_and_throttling() {
        let mut service = MockExchangeService::new();

        // Set low rate limit for testing
        service.http_client.set_rate_limit_threshold(3);

        // Setup mock responses for different symbols to avoid caching
        let ticker_response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.00",
            "volume": "1000.00",
            "priceChangePercent": "1.0"
        });

        let symbols = ["BTCUSDT", "ETHUSDT", "ADAUSDT", "DOTUSDT"];
        for symbol in &symbols {
            let endpoint = format!("/api/v3/ticker/24hr?symbol={}", symbol);
            service
                .http_client
                .add_response(&endpoint, ticker_response.clone());
        }

        // Test successful requests within limit
        for i in 0..3 {
            let result = service.mock_get_ticker("binance", symbols[i]).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }

        // Test rate limit exceeded (4th request should fail)
        let rate_limited_result = service.mock_get_ticker("binance", symbols[3]).await;
        assert!(rate_limited_result.is_err());
        assert!(rate_limited_result
            .unwrap_err()
            .to_string()
            .contains("Rate limit exceeded"));

        // Test rate limit reset
        service.http_client.reset_rate_limit();
        let reset_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(reset_result.is_ok());

        // Test rate limit check function
        service.http_client.set_rate_limit_threshold(1);
        service.http_client.request_count = 2; // Simulate exceeded limit
        let rate_check_result = service.mock_rate_limit_check("get_ticker").await;
        assert!(rate_check_result.is_err());
        assert!(rate_check_result
            .unwrap_err()
            .to_string()
            .contains("Rate limit exceeded for operation 'get_ticker'"));
    }

    #[tokio::test]
    async fn test_error_handling_and_retry_logic() {
        let mut service = MockExchangeService::new();

        // Test network error handling
        service.http_client.simulate_error("network_error");
        let network_error_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(network_error_result.is_err());
        assert!(network_error_result
            .unwrap_err()
            .to_string()
            .contains("Network connection failed"));

        // Test timeout error handling
        service.http_client.simulate_error("timeout");
        let timeout_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(timeout_result.is_err());
        assert!(timeout_result
            .unwrap_err()
            .to_string()
            .contains("Request timeout"));

        // Test authentication error handling
        service.http_client.simulate_error("auth_failed");
        let auth_error_result = service
            .mock_test_api_connection("binance", "invalid_key", "invalid_secret")
            .await;
        assert!(auth_error_result.is_err());
        assert!(auth_error_result
            .unwrap_err()
            .to_string()
            .contains("Authentication failed"));

        // Test invalid response handling
        service.http_client.simulate_error("invalid_response");
        let parse_error_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(parse_error_result.is_err());
        assert!(parse_error_result
            .unwrap_err()
            .to_string()
            .contains("Invalid JSON response"));

        // Test KV store error handling
        service.kv_store.simulate_error("kv_put_failed");
        service.http_client.error_simulation = None; // Reset HTTP errors

        let kv_response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.00",
            "volume": "1000.00",
            "priceChangePercent": "1.0"
        });
        service
            .http_client
            .add_response("/api/v3/ticker/24hr?symbol=BTCUSDT", kv_response);

        let kv_error_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(kv_error_result.is_err());
        assert!(kv_error_result
            .unwrap_err()
            .to_string()
            .contains("KV put operation failed"));
    }

    #[tokio::test]
    async fn test_exchange_specific_api_handling() {
        let mut service = MockExchangeService::new();

        // Test Binance-specific ticker parsing
        let binance_response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.50",
            "volume": "1234.56",
            "priceChangePercent": "2.5"
        });

        let binance_ticker = service
            .parse_binance_ticker(&binance_response, "BTCUSDT")
            .unwrap();
        assert_eq!(binance_ticker.symbol, "BTCUSDT");
        assert_eq!(binance_ticker.last, Some(45000.50));

        // Test Bybit-specific ticker parsing
        let bybit_response = json!({
            "result": {
                "symbol": "BTCUSDT",
                "lastPrice": "45000.50",
                "volume24h": "1234.56",
                "price24hPcnt": "0.025"
            }
        });

        let bybit_ticker = service
            .parse_bybit_ticker(&bybit_response, "BTCUSDT")
            .unwrap();
        assert_eq!(bybit_ticker.symbol, "BTCUSDT");
        assert_eq!(bybit_ticker.last, Some(45000.50));

        // Test invalid exchange for parsing
        let invalid_exchange_result =
            service.parse_ticker_response("invalid", &binance_response, "BTCUSDT");
        assert!(invalid_exchange_result.is_err());
        assert!(invalid_exchange_result
            .unwrap_err()
            .to_string()
            .contains("Unsupported exchange for ticker parsing"));
    }

    #[tokio::test]
    async fn test_data_validation_and_sanitization() {
        let service = MockExchangeService::new();

        // Test valid symbol validation
        assert!(service.validate_symbol_format("BTCUSDT").is_ok());
        assert!(service.validate_symbol_format("ETH").is_ok());
        assert!(service.validate_symbol_format("DOGEUSDT").is_ok());

        // Test invalid symbol validation
        assert!(service.validate_symbol_format("").is_err());
        assert!(service.validate_symbol_format("BT").is_err()); // Too short
        assert!(service
            .validate_symbol_format("VERYLONGSYMBOLNAME123")
            .is_err()); // Too long
        assert!(service.validate_symbol_format("BTC-USDT").is_err()); // Invalid characters
        assert!(service.validate_symbol_format("BTC USDT").is_err()); // Space not allowed

        // Test exchange validation
        assert!(service.validate_exchange_support("binance").is_ok());
        assert!(service.validate_exchange_support("bybit").is_ok());
        assert!(service.validate_exchange_support("okx").is_ok());
        assert!(service.validate_exchange_support("unsupported").is_err());

        // Test ticker data validation with missing fields
        let invalid_binance_data = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.50"
            // Missing volume and priceChangePercent
        });

        let invalid_ticker_result = service.parse_binance_ticker(&invalid_binance_data, "BTCUSDT");
        assert!(invalid_ticker_result.is_err());
        assert!(invalid_ticker_result
            .unwrap_err()
            .to_string()
            .contains("Missing volume"));
    }

    #[tokio::test]
    async fn test_connection_management() {
        let mut service = MockExchangeService::new();

        // Test multiple concurrent connections simulation
        let ticker_response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.00",
            "volume": "1000.00",
            "priceChangePercent": "1.0"
        });

        // Setup responses for multiple symbols
        let symbols = ["BTCUSDT", "ETHUSDT", "ADAUSDT", "DOTUSDT", "LINKUSDT"];
        for symbol in &symbols {
            let endpoint = format!("/api/v3/ticker/24hr?symbol={}", symbol);
            service
                .http_client
                .add_response(&endpoint, ticker_response.clone());
        }

        // Test concurrent requests (simulated)
        let mut results = Vec::new();
        for symbol in &symbols {
            let result = service.mock_get_ticker("binance", symbol).await;
            results.push(result);
        }

        // Verify all requests succeeded
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Request for {} should succeed", symbols[i]);
            let ticker = result.as_ref().unwrap();
            assert_eq!(ticker.symbol, symbols[i]);
            assert_eq!(ticker.last, Some(45000.00));
        }

        // Verify request count tracking
        let metrics = service.get_performance_metrics();
        assert_eq!(metrics.total_requests, symbols.len() as u32);
        assert!(metrics.cache_operations >= symbols.len() as u32); // At least one cache operation per request
    }

    #[tokio::test]
    async fn test_performance_optimization() {
        let mut service = MockExchangeService::new();

        // Test caching performance
        let ticker_response = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "45000.00",
            "volume": "1000.00",
            "priceChangePercent": "1.0"
        });
        service
            .http_client
            .add_response("/api/v3/ticker/24hr?symbol=BTCUSDT", ticker_response);

        // First request should hit the API
        let first_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(first_result.is_ok());
        let initial_requests = service.http_client.get_request_count();

        // Second request should use cache (no additional API call)
        let second_result = service.mock_get_ticker("binance", "BTCUSDT").await;
        assert!(second_result.is_ok());
        let cached_requests = service.http_client.get_request_count();

        // Verify cache was used (no additional API requests)
        assert_eq!(initial_requests, cached_requests);

        // Test cache key generation consistency
        assert!(service.kv_store.contains_key("ticker:binance:BTCUSDT"));

        // Test performance metrics
        let metrics = service.get_performance_metrics();
        assert!(metrics.total_requests > 0);
        assert!(metrics.cache_operations > 0);
        assert_eq!(metrics.supported_exchanges, 3); // binance, bybit, okx
    }

    #[tokio::test]
    async fn test_markets_data_fetching() {
        let mut service = MockExchangeService::new();

        // Setup mock markets response
        let markets_response = json!({
            "symbols": [
                {
                    "symbol": "BTCUSDT",
                    "status": "TRADING",
                    "baseAsset": "BTC",
                    "quoteAsset": "USDT"
                },
                {
                    "symbol": "ETHUSDT",
                    "status": "TRADING",
                    "baseAsset": "ETH",
                    "quoteAsset": "USDT"
                },
                {
                    "symbol": "ADAUSDT",
                    "status": "BREAK",
                    "baseAsset": "ADA",
                    "quoteAsset": "USDT"
                }
            ]
        });
        service
            .http_client
            .add_response("/api/v3/exchangeInfo", markets_response);

        // Test successful markets fetch
        let markets_result = service.mock_get_markets("binance").await;
        assert!(markets_result.is_ok());

        let markets = markets_result.unwrap();
        assert_eq!(markets.len(), 2); // Only TRADING status markets

        // Verify market data structure
        let btc_market = &markets[0];
        assert_eq!(btc_market.symbol, "BTCUSDT");
        assert_eq!(btc_market.base, "BTC");
        assert_eq!(btc_market.quote, "USDT");
        assert!(btc_market.active);

        let eth_market = &markets[1];
        assert_eq!(eth_market.symbol, "ETHUSDT");
        assert_eq!(eth_market.base, "ETH");
        assert_eq!(eth_market.quote, "USDT");
        assert!(eth_market.active);

        // Test unsupported exchange
        let unsupported_markets = service.mock_get_markets("unsupported").await;
        assert!(unsupported_markets.is_err());
        assert!(unsupported_markets
            .unwrap_err()
            .to_string()
            .contains("Unsupported exchange"));
    }

    #[tokio::test]
    async fn test_user_api_compatibility_validation() {
        let service = MockExchangeService::new();

        // Test compatible exchanges
        let user_exchanges = vec!["binance".to_string(), "bybit".to_string()];
        let required_exchanges = vec!["binance".to_string(), "bybit".to_string()];

        let compatible_result = service
            .mock_validate_user_api_compatibility(&user_exchanges, &required_exchanges)
            .await;
        assert!(compatible_result.is_ok());
        assert!(compatible_result.unwrap());

        // Test partial compatibility
        let partial_user_exchanges = vec!["binance".to_string()];
        let partial_required = vec!["binance".to_string(), "okx".to_string()];

        let partial_result = service
            .mock_validate_user_api_compatibility(&partial_user_exchanges, &partial_required)
            .await;
        assert!(partial_result.is_ok());
        assert!(!partial_result.unwrap());

        // Test no compatibility
        let incompatible_user = vec!["binance".to_string()];
        let incompatible_required = vec!["okx".to_string(), "bybit".to_string()];

        let incompatible_result = service
            .mock_validate_user_api_compatibility(&incompatible_user, &incompatible_required)
            .await;
        assert!(incompatible_result.is_ok());
        assert!(!incompatible_result.unwrap());

        // Test empty requirements (should always be compatible)
        let empty_required: Vec<String> = vec![];
        let empty_result = service
            .mock_validate_user_api_compatibility(&user_exchanges, &empty_required)
            .await;
        assert!(empty_result.is_ok());
        assert!(empty_result.unwrap());
    }

    #[test]
    fn test_service_configuration_validation() {
        let service = MockExchangeService::new();

        // Test supported exchanges configuration
        assert_eq!(service.supported_exchanges.len(), 3);
        assert!(service.supported_exchanges.contains(&"binance".to_string()));
        assert!(service.supported_exchanges.contains(&"bybit".to_string()));
        assert!(service.supported_exchanges.contains(&"okx".to_string()));

        // Test API key validation enabled by default
        assert!(service.api_key_validation_enabled);

        // Test initial state
        assert_eq!(service.super_admin_configs.len(), 0);
        assert_eq!(service.kv_store.get_operation_count(), 0);
        assert_eq!(service.http_client.get_request_count(), 0);

        // Test performance metrics initial state
        let metrics = service.get_performance_metrics();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.cache_operations, 0);
        assert_eq!(metrics.supported_exchanges, 3);
        assert_eq!(metrics.super_admin_configs, 0);
    }
}
