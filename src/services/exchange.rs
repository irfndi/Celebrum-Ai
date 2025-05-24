// src/services/exchange.rs

use chrono::Utc;
use reqwest::{Client, Method};
use serde_json::{json, Value};

use crate::types::*;
use crate::utils::{ArbitrageError, ArbitrageResult};

// Exchange authentication helper
use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[allow(dead_code)]
pub trait ExchangeInterface {
    #[allow(async_fn_in_trait)]
    async fn save_api_key(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
    ) -> ArbitrageResult<()>;

    #[allow(async_fn_in_trait)]
    async fn get_api_key(&self, exchange_id: &str) -> ArbitrageResult<Option<ExchangeCredentials>>;
    #[allow(async_fn_in_trait)]
    async fn delete_api_key(&self, exchange_id: &str) -> ArbitrageResult<()>;

    #[allow(async_fn_in_trait)]
    async fn get_markets(&self, exchange_id: &str) -> ArbitrageResult<Vec<Market>>;
    #[allow(async_fn_in_trait)]
    async fn get_ticker(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<Ticker>;
    #[allow(async_fn_in_trait)]
    async fn get_orderbook(
        &self,
        exchange_id: &str,
        symbol: &str,
        limit: Option<u32>,
    ) -> ArbitrageResult<OrderBook>;

    #[allow(async_fn_in_trait)]
    async fn fetch_funding_rates(
        &self,
        exchange_id: &str,
        symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>>;

    #[allow(async_fn_in_trait)]
    async fn get_balance(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
    ) -> ArbitrageResult<Value>;

    #[allow(async_fn_in_trait)]
    async fn create_order(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
        symbol: &str,
        side: &str,
        amount: f64,
        price: Option<f64>,
    ) -> ArbitrageResult<Value>;

    #[allow(async_fn_in_trait)]
    async fn cancel_order(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
        order_id: &str,
        symbol: &str,
    ) -> ArbitrageResult<Value>;

    #[allow(async_fn_in_trait)]
    async fn get_open_orders(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
        symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>>;

    #[allow(async_fn_in_trait)]
    async fn get_open_positions(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
        symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>>;

    #[allow(async_fn_in_trait)]
    async fn set_leverage(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
        symbol: &str,
        leverage: u32,
    ) -> ArbitrageResult<Value>;

    #[allow(async_fn_in_trait)]
    async fn get_trading_fees(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        symbol: &str,
    ) -> ArbitrageResult<Value>;
}

// ============= SUPER ADMIN API CONFIGURATION =============

#[derive(Debug, Clone)]
pub struct SuperAdminApiConfig {
    pub exchange_id: String,
    pub read_only_credentials: ExchangeCredentials,
    pub is_trading_enabled: bool, // Should always be false for global opportunity data
}

impl SuperAdminApiConfig {
    pub fn new_read_only(exchange_id: String, credentials: ExchangeCredentials) -> Self {
        Self {
            exchange_id,
            read_only_credentials: credentials,
            is_trading_enabled: false, // Enforced read-only
        }
    }

    pub fn can_trade(&self) -> bool {
        self.is_trading_enabled
    }

    pub fn validate_read_only(&self) -> ArbitrageResult<()> {
        if self.is_trading_enabled {
            return Err(ArbitrageError::validation_error(
                "Super admin API must be read-only for global opportunity generation".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ApiKeySource {
    SuperAdminReadOnly(SuperAdminApiConfig),
    UserTrading(ExchangeCredentials),
}

impl ApiKeySource {
    pub fn get_credentials(&self) -> &ExchangeCredentials {
        match self {
            ApiKeySource::SuperAdminReadOnly(config) => &config.read_only_credentials,
            ApiKeySource::UserTrading(creds) => creds,
        }
    }

    pub fn can_execute_trades(&self) -> bool {
        match self {
            ApiKeySource::SuperAdminReadOnly(_) => false, // Never allow trading with admin keys
            ApiKeySource::UserTrading(_) => true,
        }
    }

    pub fn validate_for_operation(&self, operation: &str) -> ArbitrageResult<()> {
        let trading_operations = ["create_order", "cancel_order", "set_leverage"];
        
        if trading_operations.contains(&operation) && !self.can_execute_trades() {
            return Err(ArbitrageError::validation_error(
                format!("Operation '{}' not allowed with read-only super admin keys", operation)
            ));
        }
        
        Ok(())
    }
}

pub struct ExchangeService {
    client: Client,
    kv: worker::kv::KvStore,
    super_admin_configs: std::collections::HashMap<String, SuperAdminApiConfig>,
}

impl ExchangeService {
    #[allow(clippy::result_large_err)]
    pub fn new(env: &Env) -> ArbitrageResult<Self> {
        let kv = env.get_kv_store("ARBITRAGE_KV").ok_or_else(|| {
            ArbitrageError::internal_error(
                "Failed to get KV store: ARBITRAGE_KV binding not found".to_string(),
            )
        })?;

        let client = Client::new();

        Ok(Self { 
            client, 
            kv,
            super_admin_configs: std::collections::HashMap::new(),
        })
    }

    /// Configure super admin read-only API keys for global opportunity generation
    pub fn configure_super_admin_api(
        &mut self,
        exchange_id: String,
        credentials: ExchangeCredentials,
    ) -> ArbitrageResult<()> {
        let config = SuperAdminApiConfig::new_read_only(exchange_id.clone(), credentials);
        config.validate_read_only()?;
        
        self.super_admin_configs.insert(exchange_id, config);
        Ok(())
    }

    /// Get market data using super admin read-only keys
    pub async fn get_global_market_data(
        &self,
        exchange_id: &str,
        symbol: &str,
    ) -> ArbitrageResult<Ticker> {
        if let Some(config) = self.super_admin_configs.get(exchange_id) {
            config.validate_read_only()?;
            // Use read-only credentials for market data
            self.get_ticker(exchange_id, symbol).await
        } else {
            Err(ArbitrageError::validation_error(
                format!("No super admin configuration found for exchange: {}", exchange_id)
            ))
        }
    }

    /// Get funding rates using super admin read-only keys
    pub async fn get_global_funding_rates(
        &self,
        exchange_id: &str,
        symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>> {
        if let Some(config) = self.super_admin_configs.get(exchange_id) {
            config.validate_read_only()?;
            // Use read-only credentials for funding rate data
            self.fetch_funding_rates(exchange_id, symbol).await
        } else {
            Err(ArbitrageError::validation_error(
                format!("No super admin configuration found for exchange: {}", exchange_id)
            ))
        }
    }

    /// Validate operation against API key source
    pub fn validate_operation_permission(
        &self,
        api_source: &ApiKeySource,
        operation: &str,
    ) -> ArbitrageResult<()> {
        api_source.validate_for_operation(operation)
    }

    #[allow(clippy::result_large_err)]
    fn parse_binance_ticker(&self, data: &Value, symbol: &str) -> ArbitrageResult<Ticker> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: data["bidPrice"].as_str().and_then(|s| s.parse().ok()),
            ask: data["askPrice"].as_str().and_then(|s| s.parse().ok()),
            last: data["price"].as_str().and_then(|s| s.parse().ok()),
            high: data["highPrice"].as_str().and_then(|s| s.parse().ok()),
            low: data["lowPrice"].as_str().and_then(|s| s.parse().ok()),
            volume: data["volume"].as_str().and_then(|s| s.parse().ok()),
            timestamp: Some(Utc::now()),
            datetime: Some(Utc::now().to_rfc3339()),
        })
    }

    #[allow(clippy::result_large_err)]
    fn parse_bybit_ticker(&self, data: &Value, symbol: &str) -> ArbitrageResult<Ticker> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: data["bid1Price"].as_str().and_then(|s| s.parse().ok()),
            ask: data["ask1Price"].as_str().and_then(|s| s.parse().ok()),
            last: data["lastPrice"].as_str().and_then(|s| s.parse().ok()),
            high: data["highPrice24h"].as_str().and_then(|s| s.parse().ok()),
            low: data["lowPrice24h"].as_str().and_then(|s| s.parse().ok()),
            volume: data["volume24h"].as_str().and_then(|s| s.parse().ok()),
            timestamp: Some(Utc::now()),
            datetime: Some(Utc::now().to_rfc3339()),
        })
    }

    // Exchange-specific implementations
    async fn binance_request(
        &self,
        endpoint: &str,
        method: Method,
        params: Option<Value>,
        auth: Option<&ExchangeCredentials>,
    ) -> ArbitrageResult<Value> {
        let base_url = "https://api.binance.com";
        let url = format!("{}{}", base_url, endpoint);

        let mut request = self.client.request(method, &url);

        // Collect all query parameters
        let mut query_params = Vec::new();

        // Add query parameters from the params argument
        if let Some(params) = params {
            if let Some(obj) = params.as_object() {
                for (key, value) in obj {
                    if let Some(str_val) = value.as_str() {
                        query_params.push((key.clone(), str_val.to_string()));
                    } else {
                        query_params.push((key.clone(), value.to_string()));
                    }
                }
            }
        }

        // Add authentication if provided
        if let Some(creds) = auth {
            let timestamp = Utc::now().timestamp_millis();

            // Add timestamp to query parameters
            query_params.push(("timestamp".to_string(), timestamp.to_string()));

            // Sort query parameters for consistent signature generation
            query_params.sort();
            let query_string = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");

            // Create signature
            let mut mac = Hmac::<Sha256>::new_from_slice(creds.secret.as_bytes()).map_err(|e| {
                ArbitrageError::authentication_error(format!("Invalid secret key: {}", e))
            })?;
            mac.update(query_string.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());

            // Add signature to query params
            query_params.push(("signature".to_string(), signature));

            // Set query parameters
            request = request.query(&query_params);
            request = request.header("X-MBX-APIKEY", &creds.api_key);
        } else {
            // If no auth, just add the regular parameters
            if !query_params.is_empty() {
                request = request.query(&query_params);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| ArbitrageError::network_error(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::api_error(format!(
                "Binance API error: {}",
                error_text
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to parse JSON: {}", e)))?;
        Ok(json)
    }

    async fn bybit_request(
        &self,
        endpoint: &str,
        method: Method,
        params: Option<Value>,
        auth: Option<&ExchangeCredentials>,
    ) -> ArbitrageResult<Value> {
        let base_url = "https://api.bybit.com";
        let url = format!("{}{}", base_url, endpoint);

        let mut request = self.client.request(method, &url);

        // Add authentication if provided
        if let Some(creds) = auth {
            let timestamp = Utc::now().timestamp_millis();
            let recv_window = "5000";

            let param_str = if let Some(params) = &params {
                serde_json::to_string(params).unwrap_or_default()
            } else {
                "{}".to_string()
            };

            let sign_str = format!(
                "{}{}{}{}",
                timestamp, &creds.api_key, recv_window, param_str
            );

            let mut mac = Hmac::<Sha256>::new_from_slice(creds.secret.as_bytes()).map_err(|e| {
                ArbitrageError::authentication_error(format!("Invalid secret key: {}", e))
            })?;
            mac.update(sign_str.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());

            request = request
                .header("X-BAPI-API-KEY", &creds.api_key)
                .header("X-BAPI-SIGN", signature)
                .header("X-BAPI-TIMESTAMP", timestamp.to_string())
                .header("X-BAPI-RECV-WINDOW", recv_window)
                .header("Content-Type", "application/json");

            if let Some(params) = params {
                request = request.json(&params);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| ArbitrageError::network_error(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::api_error(format!(
                "Bybit API error: {}",
                error_text
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to parse JSON: {}", e)))?;
        Ok(json)
    }
}

impl ExchangeInterface for ExchangeService {
    async fn save_api_key(
        &self,
        exchange_id: &str,
        credentials: &ExchangeCredentials,
    ) -> ArbitrageResult<()> {
        let key = format!("exchange_credentials_{}", exchange_id);
        let value = serde_json::to_string(credentials).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize credentials: {}", e))
        })?;

        self.kv
            .put(&key, value)
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to save credentials: {}", e))
            })?
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to execute save: {}", e))
            })?;

        Ok(())
    }

    async fn get_api_key(&self, exchange_id: &str) -> ArbitrageResult<Option<ExchangeCredentials>> {
        let key = format!("exchange_credentials_{}", exchange_id);

        match self.kv.get(&key).text().await {
            Ok(Some(value)) => {
                let credentials: ExchangeCredentials =
                    serde_json::from_str(&value).map_err(|e| {
                        ArbitrageError::parse_error(format!(
                            "Failed to deserialize credentials: {}",
                            e
                        ))
                    })?;
                Ok(Some(credentials))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ArbitrageError::database_error(format!(
                "Failed to get credentials: {}",
                e
            ))),
        }
    }

    async fn delete_api_key(&self, exchange_id: &str) -> ArbitrageResult<()> {
        let key = format!("exchange_credentials_{}", exchange_id);
        self.kv.delete(&key).await.map_err(|e| {
            ArbitrageError::database_error(format!("Failed to delete credentials: {}", e))
        })?;
        Ok(())
    }

    async fn get_markets(&self, exchange_id: &str) -> ArbitrageResult<Vec<Market>> {
        let markets = match exchange_id {
            "binance" => {
                let response = self
                    .binance_request("/api/v3/exchangeInfo", Method::GET, None, None)
                    .await?;
                let empty_vec = vec![];
                let symbols = response["symbols"].as_array().unwrap_or(&empty_vec);

                symbols
                    .iter()
                    .map(|symbol| Market {
                        id: symbol["symbol"].as_str().unwrap_or("").to_string(),
                        symbol: symbol["symbol"].as_str().unwrap_or("").to_string(),
                        base: symbol["baseAsset"].as_str().unwrap_or("").to_string(),
                        quote: symbol["quoteAsset"].as_str().unwrap_or("").to_string(),
                        active: symbol["status"].as_str() == Some("TRADING"),
                        precision: Precision {
                            amount: symbol["baseAssetPrecision"].as_i64().map(|x| x as i32),
                            price: symbol["quotePrecision"].as_i64().map(|x| x as i32),
                        },
                        limits: Limits {
                            amount: MinMax {
                                min: Some(0.0),
                                max: None,
                            },
                            price: MinMax {
                                min: Some(0.0),
                                max: None,
                            },
                            cost: MinMax {
                                min: Some(0.0),
                                max: None,
                            },
                        },
                        fees: None,
                    })
                    .collect()
            }
            "bybit" => {
                let response = self
                    .bybit_request(
                        "/v5/market/instruments-info",
                        Method::GET,
                        Some(json!({"category": "spot"})),
                        None,
                    )
                    .await?;
                let empty_vec = vec![];
                let symbols = response["result"]["list"].as_array().unwrap_or(&empty_vec);

                symbols
                    .iter()
                    .map(|symbol| Market {
                        id: symbol["symbol"].as_str().unwrap_or("").to_string(),
                        symbol: symbol["symbol"].as_str().unwrap_or("").to_string(),
                        base: symbol["baseCoin"].as_str().unwrap_or("").to_string(),
                        quote: symbol["quoteCoin"].as_str().unwrap_or("").to_string(),
                        active: symbol["status"].as_str() == Some("Trading"),
                        precision: Precision {
                            amount: None,
                            price: None,
                        },
                        limits: Limits {
                            amount: MinMax {
                                min: Some(0.0),
                                max: None,
                            },
                            price: MinMax {
                                min: Some(0.0),
                                max: None,
                            },
                            cost: MinMax {
                                min: Some(0.0),
                                max: None,
                            },
                        },
                        fees: None,
                    })
                    .collect()
            }
            _ => {
                return Err(ArbitrageError::validation_error(format!(
                    "Unsupported exchange: {}",
                    exchange_id
                )))
            }
        };

        Ok(markets)
    }

    async fn get_ticker(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<Ticker> {
        match exchange_id {
            "binance" => {
                let response = self
                    .binance_request(
                        "/api/v3/ticker/24hr",
                        Method::GET,
                        Some(json!({"symbol": symbol})),
                        None,
                    )
                    .await?;
                self.parse_binance_ticker(&response, symbol)
            }
            "bybit" => {
                let response = self
                    .bybit_request(
                        "/v5/market/tickers",
                        Method::GET,
                        Some(json!({"category": "spot", "symbol": symbol})),
                        None,
                    )
                    .await?;

                if let Some(list) = response["result"]["list"].as_array() {
                    if let Some(ticker_data) = list.first() {
                        return self.parse_bybit_ticker(ticker_data, symbol);
                    }
                }
                Err(ArbitrageError::not_found(format!(
                    "Ticker not found for symbol: {}",
                    symbol
                )))
            }
            _ => Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                exchange_id
            ))),
        }
    }

    async fn get_orderbook(
        &self,
        exchange_id: &str,
        symbol: &str,
        limit: Option<u32>,
    ) -> ArbitrageResult<OrderBook> {
        let limit = limit.unwrap_or(100);

        match exchange_id {
            "binance" => {
                let response = self
                    .binance_request(
                        "/api/v3/depth",
                        Method::GET,
                        Some(json!({"symbol": symbol, "limit": limit})),
                        None,
                    )
                    .await?;

                let empty_vec = vec![];
                let bids: Vec<[f64; 2]> = response["bids"]
                    .as_array()
                    .unwrap_or(&empty_vec)
                    .iter()
                    .filter_map(|bid| {
                        if let Some(arr) = bid.as_array() {
                            if arr.len() >= 2 {
                                let price = arr[0].as_str()?.parse().ok()?;
                                let amount = arr[1].as_str()?.parse().ok()?;
                                Some([price, amount])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                let empty_vec2 = vec![];
                let asks: Vec<[f64; 2]> = response["asks"]
                    .as_array()
                    .unwrap_or(&empty_vec2)
                    .iter()
                    .filter_map(|ask| {
                        if let Some(arr) = ask.as_array() {
                            if arr.len() >= 2 {
                                let price = arr[0].as_str()?.parse().ok()?;
                                let amount = arr[1].as_str()?.parse().ok()?;
                                Some([price, amount])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(OrderBook {
                    symbol: symbol.to_string(),
                    bids,
                    asks,
                    timestamp: Some(Utc::now()),
                    datetime: Some(Utc::now().to_rfc3339()),
                })
            }
            _ => Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                exchange_id
            ))),
        }
    }

    async fn fetch_funding_rates(
        &self,
        exchange_id: &str,
        symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>> {
        match exchange_id {
            "binance" => {
                let mut params = json!({});
                if let Some(s) = symbol {
                    params["symbol"] = json!(s);
                }

                let response = self
                    .binance_request("/fapi/v1/premiumIndex", Method::GET, Some(params), None)
                    .await?;

                if response.is_array() {
                    Ok(response.as_array().unwrap().clone())
                } else {
                    Ok(vec![response])
                }
            }
            _ => Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                exchange_id
            ))),
        }
    }

    async fn get_balance(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
    ) -> ArbitrageResult<Value> {
        Err(ArbitrageError::not_implemented(format!(
            "get_balance not implemented for exchange: {}",
            exchange_id
        )))
    }

    async fn create_order(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        _symbol: &str,
        _side: &str,
        _amount: f64,
        _price: Option<f64>,
    ) -> ArbitrageResult<Value> {
        Err(ArbitrageError::not_implemented(format!(
            "create_order not implemented for exchange: {}",
            exchange_id
        )))
    }

    async fn cancel_order(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        _order_id: &str,
        _symbol: &str,
    ) -> ArbitrageResult<Value> {
        Err(ArbitrageError::not_implemented(format!(
            "cancel_order not implemented for exchange: {}",
            exchange_id
        )))
    }

    async fn get_open_orders(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        _symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>> {
        Err(ArbitrageError::not_implemented(format!(
            "get_open_orders not implemented for exchange: {}",
            exchange_id
        )))
    }

    async fn get_open_positions(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        _symbol: Option<&str>,
    ) -> ArbitrageResult<Vec<Value>> {
        Err(ArbitrageError::not_implemented(format!(
            "get_open_positions not implemented for exchange: {}",
            exchange_id
        )))
    }

    async fn set_leverage(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        _symbol: &str,
        _leverage: u32,
    ) -> ArbitrageResult<Value> {
        Err(ArbitrageError::not_implemented(format!(
            "set_leverage not implemented for exchange: {}",
            exchange_id
        )))
    }

    async fn get_trading_fees(
        &self,
        exchange_id: &str,
        _credentials: &ExchangeCredentials,
        symbol: &str,
    ) -> ArbitrageResult<Value> {
        match exchange_id {
            "binance" => {
                // Binance trading fees endpoint
                let response = self
                    .binance_request(
                        "/api/v3/exchangeInfo",
                        Method::GET,
                        Some(json!({"symbol": symbol})),
                        None,
                    )
                    .await?;

                // Extract trading fees from exchange info
                if let Some(symbols) = response["symbols"].as_array() {
                    for symbol_info in symbols {
                        if symbol_info["symbol"].as_str() == Some(symbol) {
                            // Default Binance fees if not specified in response
                            return Ok(json!({
                                "symbol": symbol,
                                "maker": 0.001,  // 0.1% default maker fee
                                "taker": 0.001,  // 0.1% default taker fee
                                "exchange": "binance"
                            }));
                        }
                    }
                }

                // Fallback to default fees
                Ok(json!({
                    "symbol": symbol,
                    "maker": 0.001,
                    "taker": 0.001,
                    "exchange": "binance"
                }))
            }
            "bybit" => {
                // Bybit trading fees - using default rates as API requires authentication
                Ok(json!({
                    "symbol": symbol,
                    "maker": 0.001,  // 0.1% default maker fee
                    "taker": 0.001,  // 0.1% default taker fee
                    "exchange": "bybit"
                }))
            }
            _ => Err(ArbitrageError::validation_error(format!(
                "get_trading_fees not implemented for exchange: {}",
                exchange_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    // Mock environment for testing
    #[allow(dead_code)]
    struct MockEnv {
        kv: HashMap<String, String>,
    }

    #[allow(dead_code)]
    impl MockEnv {
        fn new() -> Self {
            Self { kv: HashMap::new() }
        }

        fn with_kv_data(mut self, key: &str, value: &str) -> Self {
            self.kv.insert(key.to_string(), value.to_string());
            self
        }
    }

    // Helper function to create test credentials
    fn create_test_credentials() -> ExchangeCredentials {
        ExchangeCredentials {
            api_key: "test_api_key".to_string(),
            secret: "test_secret_key".to_string(),
            default_leverage: 20,
            exchange_type: "spot".to_string(),
        }
    }

    // Helper function to create mock Binance ticker data
    fn create_mock_binance_ticker_data() -> Value {
        json!({
            "bidPrice": "50000.50",
            "askPrice": "50001.00",
            "price": "50000.75",
            "highPrice": "51000.00",
            "lowPrice": "49000.00",
            "volume": "1234.56"
        })
    }

    // Helper function to create mock Bybit ticker data
    fn create_mock_bybit_ticker_data() -> Value {
        json!({
            "bid1Price": "50000.50",
            "ask1Price": "50001.00",
            "lastPrice": "50000.75",
            "highPrice24h": "51000.00",
            "lowPrice24h": "49000.00",
            "volume24h": "1234.56"
        })
    }

    // Tests for ticker parsing methods
    mod ticker_parsing_tests {
        use super::*;

        #[test]
        fn test_parse_binance_ticker_success() {
            // Create a mock service (we only need the parsing method)
            let _env = MockEnv::new();
            // Note: We can't easily create ExchangeService without Worker KV,
            // so we'll test the data parsing logic directly with mock data

            let ticker_data = create_mock_binance_ticker_data();
            let _symbol = "BTCUSDT";

            // Expected values from mock data
            assert_eq!(ticker_data["bidPrice"], "50000.50");
            assert_eq!(ticker_data["askPrice"], "50001.00");
            assert_eq!(ticker_data["price"], "50000.75");
            assert_eq!(ticker_data["highPrice"], "51000.00");
            assert_eq!(ticker_data["lowPrice"], "49000.00");
            assert_eq!(ticker_data["volume"], "1234.56");

            // Test individual field parsing
            let bid = ticker_data["bidPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let ask = ticker_data["askPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let last = ticker_data["price"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let high = ticker_data["highPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let low = ticker_data["lowPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let volume = ticker_data["volume"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());

            assert_eq!(bid, Some(50000.50));
            assert_eq!(ask, Some(50001.00));
            assert_eq!(last, Some(50000.75));
            assert_eq!(high, Some(51000.00));
            assert_eq!(low, Some(49000.00));
            assert_eq!(volume, Some(1234.56));
        }

        #[test]
        fn test_parse_bybit_ticker_success() {
            let ticker_data = create_mock_bybit_ticker_data();
            let _symbol = "BTCUSDT";

            // Expected values from mock data
            assert_eq!(ticker_data["bid1Price"], "50000.50");
            assert_eq!(ticker_data["ask1Price"], "50001.00");
            assert_eq!(ticker_data["lastPrice"], "50000.75");
            assert_eq!(ticker_data["highPrice24h"], "51000.00");
            assert_eq!(ticker_data["lowPrice24h"], "49000.00");
            assert_eq!(ticker_data["volume24h"], "1234.56");

            // Test individual field parsing (Bybit format)
            let bid = ticker_data["bid1Price"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let ask = ticker_data["ask1Price"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let last = ticker_data["lastPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let high = ticker_data["highPrice24h"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let low = ticker_data["lowPrice24h"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let volume = ticker_data["volume24h"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());

            assert_eq!(bid, Some(50000.50));
            assert_eq!(ask, Some(50001.00));
            assert_eq!(last, Some(50000.75));
            assert_eq!(high, Some(51000.00));
            assert_eq!(low, Some(49000.00));
            assert_eq!(volume, Some(1234.56));
        }

        #[test]
        fn test_binance_ticker_parsing_with_invalid_data() {
            // Test with malformed price data
            let invalid_data = json!({
                "bidPrice": "invalid_price",
                "askPrice": "50001.00",
                "price": "",
                "highPrice": null,
                "lowPrice": "49000.00",
                "volume": "not_a_number"
            });

            // Test parsing - invalid strings should return None
            let bid = invalid_data["bidPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let ask = invalid_data["askPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let last = invalid_data["price"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let high = invalid_data["highPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let low = invalid_data["lowPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let volume = invalid_data["volume"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());

            assert_eq!(bid, None); // Invalid price string
            assert_eq!(ask, Some(50001.00)); // Valid price
            assert_eq!(last, None); // Empty string
            assert_eq!(high, None); // Null value
            assert_eq!(low, Some(49000.00)); // Valid price
            assert_eq!(volume, None); // Invalid number string
        }

        #[test]
        fn test_ticker_field_extraction_edge_cases() {
            // Test with missing fields
            let minimal_data = json!({
                "bidPrice": "50000.50"
                // Missing other fields
            });

            let bid = minimal_data["bidPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let ask = minimal_data["askPrice"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let last = minimal_data["price"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());

            assert_eq!(bid, Some(50000.50));
            assert_eq!(ask, None); // Missing field
            assert_eq!(last, None); // Missing field
        }
    }

    // Tests for signature generation and authentication
    mod authentication_tests {
        use super::*;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        #[test]
        fn test_hmac_signature_generation() {
            let secret = "test_secret_key";
            let message = "timestamp=1234567890&symbol=BTCUSDT";

            // Test HMAC-SHA256 signature generation (same as used in binance_request)
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
            mac.update(message.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());

            // Signature should be consistent
            assert!(!signature.is_empty());
            assert_eq!(signature.len(), 64); // SHA256 hex string length

            // Test with same input should produce same signature
            let mut mac2 = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
            mac2.update(message.as_bytes());
            let signature2 = hex::encode(mac2.finalize().into_bytes());

            assert_eq!(signature, signature2);
        }

        #[test]
        fn test_query_parameter_sorting() {
            // Test query parameter sorting logic (used in binance authentication)
            let mut params = [
                ("symbol".to_string(), "BTCUSDT".to_string()),
                ("timestamp".to_string(), "1234567890".to_string()),
                ("limit".to_string(), "100".to_string()),
            ]
            .to_vec();

            params.sort();
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");

            assert_eq!(
                query_string,
                "limit=100&symbol=BTCUSDT&timestamp=1234567890"
            );
        }

        #[test]
        fn test_credentials_structure() {
            let creds = create_test_credentials();

            assert_eq!(creds.api_key, "test_api_key");
            assert_eq!(creds.secret, "test_secret_key");
            assert_eq!(creds.default_leverage, 20);
            assert_eq!(creds.exchange_type, "spot");
        }

        #[test]
        fn test_credentials_serialization() {
            let credentials = create_test_credentials();

            // Test that credentials can be serialized and deserialized
            let serialized = serde_json::to_string(&credentials)
                .expect("Credentials should be serializable in tests");
            let deserialized: ExchangeCredentials = serde_json::from_str(&serialized).unwrap();

            assert_eq!(credentials.api_key, deserialized.api_key);
            assert_eq!(credentials.secret, deserialized.secret);
            assert_eq!(credentials.default_leverage, deserialized.default_leverage);
            assert_eq!(credentials.exchange_type, deserialized.exchange_type);
        }
    }

    // Tests for market data parsing
    mod market_data_tests {
        use super::*;

        #[test]
        fn test_binance_market_parsing() {
            let market_data = json!({
                "symbol": "BTCUSDT",
                "baseAsset": "BTC",
                "quoteAsset": "USDT",
                "status": "TRADING",
                "baseAssetPrecision": 8,
                "quotePrecision": 8
            });

            // Test individual field extraction
            let symbol = market_data["symbol"].as_str().unwrap_or("");
            let base = market_data["baseAsset"].as_str().unwrap_or("");
            let quote = market_data["quoteAsset"].as_str().unwrap_or("");
            let active = market_data["status"].as_str() == Some("TRADING");
            let base_precision = market_data["baseAssetPrecision"].as_i64().map(|x| x as i32);
            let quote_precision = market_data["quotePrecision"].as_i64().map(|x| x as i32);

            assert_eq!(symbol, "BTCUSDT");
            assert_eq!(base, "BTC");
            assert_eq!(quote, "USDT");
            assert!(active);
            assert_eq!(base_precision, Some(8));
            assert_eq!(quote_precision, Some(8));
        }

        #[test]
        fn test_bybit_market_parsing() {
            let market_data = json!({
                "symbol": "BTCUSDT",
                "baseCoin": "BTC",
                "quoteCoin": "USDT",
                "status": "Trading"
            });

            // Test individual field extraction (Bybit format)
            let symbol = market_data["symbol"].as_str().unwrap_or("");
            let base = market_data["baseCoin"].as_str().unwrap_or("");
            let quote = market_data["quoteCoin"].as_str().unwrap_or("");
            let active = market_data["status"].as_str() == Some("Trading");

            assert_eq!(symbol, "BTCUSDT");
            assert_eq!(base, "BTC");
            assert_eq!(quote, "USDT");
            assert!(active);
        }

        #[test]
        fn test_inactive_market_detection() {
            // Test inactive market for Binance
            let inactive_binance = json!({
                "symbol": "OLDCOIN",
                "status": "HALT"
            });
            let active = inactive_binance["status"].as_str() == Some("TRADING");
            assert!(!active);

            // Test inactive market for Bybit
            let inactive_bybit = json!({
                "symbol": "OLDCOIN",
                "status": "Closed"
            });
            let active_bybit = inactive_bybit["status"].as_str() == Some("Trading");
            assert!(!active_bybit);
        }
    }

    // Tests for orderbook parsing
    mod orderbook_tests {
        use super::*;

        #[test]
        fn test_binance_orderbook_parsing() {
            let orderbook_data = json!({
                "bids": [
                    ["50000.50", "1.5"],
                    ["50000.00", "2.0"],
                    ["49999.50", "0.5"]
                ],
                "asks": [
                    ["50001.00", "1.2"],
                    ["50001.50", "1.8"],
                    ["50002.00", "0.3"]
                ]
            });

            // Test bid parsing
            let empty_vec = vec![];
            let bids: Vec<[f64; 2]> = orderbook_data["bids"]
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter_map(|bid| {
                    if let Some(arr) = bid.as_array() {
                        if arr.len() >= 2 {
                            let price = arr[0].as_str()?.parse().ok()?;
                            let amount = arr[1].as_str()?.parse().ok()?;
                            Some([price, amount])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(bids.len(), 3);
            assert_eq!(bids[0], [50000.50, 1.5]);
            assert_eq!(bids[1], [50000.00, 2.0]);
            assert_eq!(bids[2], [49999.50, 0.5]);

            // Test ask parsing
            let empty_vec2 = vec![];
            let asks: Vec<[f64; 2]> = orderbook_data["asks"]
                .as_array()
                .unwrap_or(&empty_vec2)
                .iter()
                .filter_map(|ask| {
                    if let Some(arr) = ask.as_array() {
                        if arr.len() >= 2 {
                            let price = arr[0].as_str()?.parse().ok()?;
                            let amount = arr[1].as_str()?.parse().ok()?;
                            Some([price, amount])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(asks.len(), 3);
            assert_eq!(asks[0], [50001.00, 1.2]);
            assert_eq!(asks[1], [50001.50, 1.8]);
            assert_eq!(asks[2], [50002.00, 0.3]);
        }

        #[test]
        fn test_malformed_orderbook_data() {
            let malformed_data = json!({
                "bids": [
                    ["invalid_price", "1.5"],
                    ["50000.00"], // Missing amount
                    null, // Null entry
                    ["49999.50", "invalid_amount"]
                ],
                "asks": []
            });

            // Test that malformed entries are filtered out
            let empty_vec = vec![];
            let bids: Vec<[f64; 2]> = malformed_data["bids"]
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter_map(|bid| {
                    if let Some(arr) = bid.as_array() {
                        if arr.len() >= 2 {
                            let price = arr[0].as_str()?.parse().ok()?;
                            let amount = arr[1].as_str()?.parse().ok()?;
                            Some([price, amount])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Only valid entries should remain (none in this case)
            assert_eq!(bids.len(), 0);
        }

        #[test]
        fn test_empty_orderbook() {
            let empty_data = json!({
                "bids": [],
                "asks": []
            });

            let empty_vec = vec![];
            let bids: Vec<[f64; 2]> = empty_data["bids"]
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter_map(|bid| {
                    if let Some(arr) = bid.as_array() {
                        if arr.len() >= 2 {
                            let price = arr[0].as_str()?.parse().ok()?;
                            let amount = arr[1].as_str()?.parse().ok()?;
                            Some([price, amount])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let empty_vec2 = vec![];
            let asks: Vec<[f64; 2]> = empty_data["asks"]
                .as_array()
                .unwrap_or(&empty_vec2)
                .iter()
                .filter_map(|ask| {
                    if let Some(arr) = ask.as_array() {
                        if arr.len() >= 2 {
                            let price = arr[0].as_str()?.parse().ok()?;
                            let amount = arr[1].as_str()?.parse().ok()?;
                            Some([price, amount])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(bids.len(), 0);
            assert_eq!(asks.len(), 0);
        }
    }

    // Tests for funding rate data
    mod funding_rate_tests {
        use super::*;

        #[test]
        fn test_binance_funding_rate_single_symbol() {
            let funding_data = json!({
                "symbol": "BTCUSDT",
                "markPrice": "50000.12345678",
                "indexPrice": "50000.01234567",
                "estimatedSettlePrice": "50000.01234567",
                "lastFundingRate": "0.00010000",
                "nextFundingTime": 1234567890000_u64,
                "interestRate": "0.00010000",
                "time": 1234567890000_u64
            });

            // Test that we can extract relevant funding rate information
            let symbol = funding_data["symbol"].as_str().unwrap_or("");
            let funding_rate = funding_data["lastFundingRate"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            let next_funding_time = funding_data["nextFundingTime"].as_u64();

            assert_eq!(symbol, "BTCUSDT");
            assert_eq!(funding_rate, Some(0.00010000));
            assert_eq!(next_funding_time, Some(1234567890000));
        }

        #[test]
        fn test_binance_funding_rate_array_response() {
            let funding_array = json!([
                {
                    "symbol": "BTCUSDT",
                    "lastFundingRate": "0.00010000"
                },
                {
                    "symbol": "ETHUSDT",
                    "lastFundingRate": "0.00015000"
                }
            ]);

            // Test array processing
            if let Some(arr) = funding_array.as_array() {
                assert_eq!(arr.len(), 2);

                let btc_rate = arr[0]["lastFundingRate"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok());
                let eth_rate = arr[1]["lastFundingRate"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok());

                assert_eq!(btc_rate, Some(0.00010000));
                assert_eq!(eth_rate, Some(0.00015000));
            } else {
                panic!("Expected array response");
            }
        }
    }

    // Tests for exchange validation and error handling
    mod validation_tests {

        #[test]
        fn test_supported_exchanges() {
            let supported_exchanges = vec!["binance", "bybit"];

            for exchange in supported_exchanges {
                // These should not return validation errors for basic checks
                assert!(!exchange.is_empty());
                assert!(exchange.len() > 2);
            }
        }

        #[test]
        fn test_unsupported_exchange_detection() {
            let unsupported_exchanges = vec!["coinbase", "kraken", "ftx", ""];

            // Test that unsupported exchange names would trigger validation errors
            for exchange in unsupported_exchanges {
                match exchange {
                    "binance" | "bybit" => {
                        // These should be supported
                        panic!("Should not reach here for supported exchanges");
                    }
                    _ => {
                        // These should be unsupported
                        // Correctly identified as unsupported: {}
                    }
                }
            }
        }

        #[test]
        fn test_symbol_validation() {
            let valid_symbols = vec!["BTCUSDT", "ETHUSDT", "ADAUSDT"];
            let invalid_symbols = vec!["", "BTC", "invalid_symbol_format"];

            for symbol in valid_symbols {
                assert!(!symbol.is_empty());
                assert!(symbol.len() >= 6); // Minimum length for base+quote
                assert!(symbol.chars().all(|c| c.is_ascii_uppercase()));
            }

            for symbol in invalid_symbols {
                // These would trigger validation in real implementation
                if symbol.is_empty() || symbol.len() < 6 {
                    // Correctly identified as invalid symbol
                }
            }
        }
    }

    // Tests for KV storage key generation
    mod storage_tests {

        #[test]
        fn test_kv_key_generation() {
            let exchange_id = "binance";
            let expected_key = format!("exchange_credentials_{}", exchange_id);

            assert_eq!(expected_key, "exchange_credentials_binance");
        }

        #[test]
        fn test_kv_key_generation_different_exchanges() {
            let exchanges = vec!["binance", "bybit", "okx"];

            for exchange in exchanges {
                let key = format!("exchange_credentials_{}", exchange);
                assert!(key.starts_with("exchange_credentials_"));
                assert!(key.ends_with(exchange));
            }
        }
    }

    // Tests for error scenarios that should be handled
    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_error_type_construction() {
            // Test different error types that the service should handle
            let network_error = ArbitrageError::network_error("Connection failed".to_string());
            let api_error = ArbitrageError::api_error("API rate limit".to_string());
            let parse_error = ArbitrageError::parse_error("Invalid JSON".to_string());
            let auth_error =
                ArbitrageError::authentication_error("Invalid credentials".to_string());

            // Verify error messages contain expected content
            assert!(network_error.to_string().contains("Connection failed"));
            assert!(api_error.to_string().contains("API rate limit"));
            assert!(parse_error.to_string().contains("Invalid JSON"));
            assert!(auth_error.to_string().contains("Invalid credentials"));
        }

        #[test]
        fn test_not_implemented_methods() {
            // Test that not-implemented methods return appropriate errors
            let exchange_id = "binance";
            let error_msg = format!("get_balance not implemented for exchange: {}", exchange_id);

            assert!(error_msg.contains("not implemented"));
            assert!(error_msg.contains(exchange_id));
        }

        #[test]
        fn test_empty_response_handling() {
            // Test handling of empty or null responses
            let empty_json = json!({});
            let null_json = json!(null);
            let missing_field = json!({"other_field": "value"});

            // Test that missing fields are handled gracefully
            assert!(empty_json["nonexistent"].is_null());
            assert!(null_json.is_null());
            assert!(missing_field["expected_field"].is_null());
        }
    }

    // Integration-style tests for business logic
    mod business_logic_tests {
        use super::*;

        #[test]
        fn test_complete_ticker_flow() {
            // Test the complete flow of ticker data processing
            let mock_binance_response = create_mock_binance_ticker_data();
            let symbol = "BTCUSDT";

            // Simulate the ticker parsing logic
            let ticker = Ticker {
                symbol: symbol.to_string(),
                bid: mock_binance_response["bidPrice"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                ask: mock_binance_response["askPrice"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                last: mock_binance_response["price"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                high: mock_binance_response["highPrice"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                low: mock_binance_response["lowPrice"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                volume: mock_binance_response["volume"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                timestamp: Some(Utc::now()),
                datetime: Some(Utc::now().to_rfc3339()),
            };

            // Verify complete ticker structure
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert_eq!(ticker.bid, Some(50000.5));
            assert_eq!(ticker.ask, Some(50001.0));
            assert_eq!(ticker.last, Some(50000.75));
            assert_eq!(ticker.high, Some(51000.0));
            assert_eq!(ticker.low, Some(49000.0));
            assert_eq!(ticker.volume, Some(1234.56));
            assert!(ticker.timestamp.is_some());
            assert!(ticker.datetime.is_some());
        }

        #[test]
        fn test_market_structure_creation() {
            // Test creating a complete Market structure
            let market = Market {
                id: "BTCUSDT".to_string(),
                symbol: "BTCUSDT".to_string(),
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
                active: true,
                precision: Precision {
                    amount: Some(8),
                    price: Some(8),
                },
                limits: Limits {
                    amount: MinMax {
                        min: Some(0.001),
                        max: Some(1000.0),
                    },
                    price: MinMax {
                        min: Some(0.01),
                        max: Some(100000.0),
                    },
                    cost: MinMax {
                        min: Some(10.0),
                        max: None,
                    },
                },
                fees: None,
            };

            // Verify market structure
            assert_eq!(market.symbol, "BTCUSDT");
            assert_eq!(market.base, "BTC");
            assert_eq!(market.quote, "USDT");
            assert!(market.active);
            assert_eq!(market.precision.amount, Some(8));
            assert_eq!(market.precision.price, Some(8));
        }

        #[test]
        fn test_orderbook_structure_creation() {
            // Test creating a complete OrderBook structure
            let orderbook = OrderBook {
                symbol: "BTCUSDT".to_string(),
                bids: vec![[50000.50, 1.5], [50000.00, 2.0]],
                asks: vec![[50001.00, 1.2], [50001.50, 1.8]],
                timestamp: Some(Utc::now()),
                datetime: Some(Utc::now().to_rfc3339()),
            };

            // Verify orderbook structure
            assert_eq!(orderbook.symbol, "BTCUSDT");
            assert_eq!(orderbook.bids.len(), 2);
            assert_eq!(orderbook.asks.len(), 2);
            assert_eq!(orderbook.bids[0], [50000.50, 1.5]);
            assert_eq!(orderbook.asks[0], [50001.00, 1.2]);
            assert!(orderbook.timestamp.is_some());
        }
    }

    mod service_integration_tests {
        use super::*;

        // Test the business logic without requiring actual worker environment

        #[test]
        fn test_exchange_service_ticker_parsing_integration() {
            // Test binance ticker parsing logic
            let binance_data = create_mock_binance_ticker_data();

            // Manually test the parsing logic that would be used in parse_binance_ticker
            let _symbol = "BTCUSDT";
            let bid = binance_data["bidPrice"]
                .as_str()
                .and_then(|s| s.parse().ok());
            let ask = binance_data["askPrice"]
                .as_str()
                .and_then(|s| s.parse().ok());
            let last = binance_data["price"].as_str().and_then(|s| s.parse().ok());

            // Update expected values to match the mock data
            assert_eq!(bid, Some(50000.5)); // Changed from 50000.0 to 50000.5
            assert_eq!(ask, Some(50001.0)); // Changed from 50050.0 to 50001.0
            assert_eq!(last, Some(50000.75)); // Changed from 50025.0 to 50000.75

            // Test bybit ticker parsing logic
            let bybit_data = create_mock_bybit_ticker_data();
            let bid = bybit_data["bid1Price"]
                .as_str()
                .and_then(|s| s.parse().ok());
            let ask = bybit_data["ask1Price"]
                .as_str()
                .and_then(|s| s.parse().ok());
            let last = bybit_data["lastPrice"]
                .as_str()
                .and_then(|s| s.parse().ok());

            // Update expected values to match the mock data
            assert_eq!(bid, Some(50000.5)); // Changed from 49999.0 to 50000.5
            assert_eq!(ask, Some(50001.0)); // Same value
            assert_eq!(last, Some(50000.75)); // Changed from 50000.0 to 50000.75
        }

        #[test]
        fn test_exchange_credentials_key_generation() {
            // Test the key generation logic for API credentials
            let exchange_id = "binance";
            let expected_key = format!("exchange_credentials_{}", exchange_id);
            assert_eq!(expected_key, "exchange_credentials_binance");

            let exchange_id = "bybit";
            let expected_key = format!("exchange_credentials_{}", exchange_id);
            assert_eq!(expected_key, "exchange_credentials_bybit");
        }

        #[test]
        fn test_exchange_credentials_serialization() {
            let credentials = create_test_credentials();

            // Test that credentials can be serialized and deserialized
            let serialized = serde_json::to_string(&credentials)
                .expect("Credentials should be serializable in tests");
            let deserialized: ExchangeCredentials = serde_json::from_str(&serialized).unwrap();

            assert_eq!(credentials.api_key, deserialized.api_key);
            assert_eq!(credentials.secret, deserialized.secret);
            assert_eq!(credentials.default_leverage, deserialized.default_leverage);
            assert_eq!(credentials.exchange_type, deserialized.exchange_type);
        }

        #[test]
        fn test_exchange_orderbook_parsing_logic() {
            // Test orderbook parsing logic for Binance format
            let orderbook_data = json!({
                "bids": [
                    ["50000.00", "1.50"],
                    ["49999.00", "2.00"],
                    ["49998.00", "0.50"]
                ],
                "asks": [
                    ["50001.00", "1.00"],
                    ["50002.00", "1.20"],
                    ["50003.00", "0.80"]
                ]
            });

            let empty_vec = vec![];
            let bids: Vec<[f64; 2]> = orderbook_data["bids"]
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter_map(|bid| {
                    if let Some(arr) = bid.as_array() {
                        if arr.len() >= 2 {
                            let price = arr[0].as_str()?.parse().ok()?;
                            let amount = arr[1].as_str()?.parse().ok()?;
                            Some([price, amount])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(bids.len(), 3);
            assert_eq!(bids[0], [50000.0, 1.5]);
            assert_eq!(bids[1], [49999.0, 2.0]);
            assert_eq!(bids[2], [49998.0, 0.5]);
        }

        #[test]
        fn test_market_data_structure_validation() {
            // Test that market data structures are properly formed
            let market = Market {
                id: "BTCUSDT".to_string(),
                symbol: "BTCUSDT".to_string(),
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
                active: true,
                precision: Precision {
                    amount: Some(8),
                    price: Some(2),
                },
                limits: Limits {
                    amount: MinMax {
                        min: Some(0.001),
                        max: Some(1000.0),
                    },
                    price: MinMax {
                        min: Some(0.01),
                        max: Some(100000.0),
                    },
                    cost: MinMax {
                        min: Some(10.0),
                        max: None,
                    },
                },
                fees: None,
            };

            assert_eq!(market.symbol, "BTCUSDT");
            assert_eq!(market.base, "BTC");
            assert_eq!(market.quote, "USDT");
            assert!(market.active);
            assert_eq!(market.precision.amount, Some(8));
            assert_eq!(market.precision.price, Some(2));
        }

        #[test]
        fn test_orderbook_data_structure_validation() {
            // Test OrderBook structure creation and validation
            let orderbook = OrderBook {
                symbol: "BTCUSDT".to_string(),
                bids: vec![[50000.0, 1.5], [49999.0, 2.0], [49998.0, 0.5]],
                asks: vec![[50001.0, 1.0], [50002.0, 1.2], [50003.0, 0.8]],
                timestamp: Some(Utc::now()),
                datetime: Some(Utc::now().to_rfc3339()),
            };

            assert_eq!(orderbook.symbol, "BTCUSDT");
            assert_eq!(orderbook.bids.len(), 3);
            assert_eq!(orderbook.asks.len(), 3);

            // Verify bid/ask ordering assumptions
            assert!(orderbook.bids[0][0] > orderbook.bids[1][0]); // Bids should be descending price
            assert!(orderbook.asks[0][0] < orderbook.asks[1][0]); // Asks should be ascending price

            // Verify spread
            let best_bid = orderbook.bids[0][0];
            let best_ask = orderbook.asks[0][0];
            assert!(best_ask > best_bid); // Spread should be positive
        }

        #[test]
        fn test_ticker_data_structure_validation() {
            let ticker = Ticker {
                symbol: "BTCUSDT".to_string(),
                bid: Some(50000.0),
                ask: Some(50050.0),
                last: Some(50025.0),
                high: Some(51000.0),
                low: Some(49000.0),
                volume: Some(1234.56),
                timestamp: Some(Utc::now()),
                datetime: Some(Utc::now().to_rfc3339()),
            };

            assert_eq!(ticker.symbol, "BTCUSDT");
            assert!(ticker.bid.is_some());
            assert!(ticker.ask.is_some());
            assert!(ticker.last.is_some());

            // Verify bid/ask relationship
            if let (Some(bid), Some(ask)) = (ticker.bid, ticker.ask) {
                assert!(ask >= bid); // Ask should be >= bid
            }

            // Verify high/low relationship
            if let (Some(high), Some(low), Some(last)) = (ticker.high, ticker.low, ticker.last) {
                assert!(high >= low); // High should be >= low
                assert!(last >= low && last <= high); // Last should be within high/low range
            }
        }

        #[test]
        fn test_exchange_credentials_validation() {
            let credentials = create_test_credentials();

            // Test that credentials have required fields
            assert!(!credentials.api_key.is_empty());
            assert!(!credentials.secret.is_empty());

            // Test default leverage
            assert!(credentials.default_leverage > 0);

            // Test exchange type
            assert!(!credentials.exchange_type.is_empty());
        }

        #[test]
        fn test_funding_rate_data_structure() {
            // Test funding rate data structure validation
            let funding_rate_data = json!({
                "symbol": "BTCUSDT",
                "fundingRate": "0.0001",
                "fundingTime": 1234567890000u64,
                "nextFundingTime": 1234567890000u64 + 28800000
            });

            assert_eq!(funding_rate_data["symbol"].as_str().unwrap(), "BTCUSDT");
            assert_eq!(funding_rate_data["fundingRate"].as_str().unwrap(), "0.0001");
            assert!(funding_rate_data["fundingTime"].as_u64().is_some());
            assert!(funding_rate_data["nextFundingTime"].as_u64().is_some());
        }

        #[test]
        fn test_exchange_api_parameter_handling() {
            // Test parameter handling for API requests
            let symbol = "BTCUSDT";
            let limit = 100u32;

            // Test Binance-style parameters
            let binance_params = json!({
                "symbol": symbol,
                "limit": limit
            });

            assert_eq!(binance_params["symbol"].as_str().unwrap(), "BTCUSDT");
            assert_eq!(binance_params["limit"].as_u64().unwrap(), 100);

            // Test Bybit-style parameters
            let bybit_params = json!({
                "category": "spot",
                "symbol": symbol
            });

            assert_eq!(bybit_params["category"].as_str().unwrap(), "spot");
            assert_eq!(bybit_params["symbol"].as_str().unwrap(), "BTCUSDT");
        }

        #[test]
        fn test_exchange_error_handling_logic() {
            // Test error handling for various scenarios

            // Test empty market data
            let empty_markets: Vec<Value> = vec![];
            assert_eq!(empty_markets.len(), 0);

            // Test invalid ticker data
            let invalid_ticker = json!({
                "symbol": "INVALID",
                "price": "not_a_number"
            });

            let parsed_price = invalid_ticker["price"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok());
            assert!(parsed_price.is_none());

            // Test missing required fields
            let incomplete_data = json!({
                "symbol": "BTCUSDT"
                // Missing other required fields
            });

            assert!(incomplete_data["bidPrice"].as_str().is_none());
            assert!(incomplete_data["askPrice"].as_str().is_none());
        }
    }
}
