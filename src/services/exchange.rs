// src/services/exchange.rs

use std::collections::HashMap;
use std::sync::Arc;
use reqwest::{Client, Method, Request, Url};
use chrono::Utc;
use serde_json::{json, Value};
use worker::{Env, kv::KvStore};

use crate::types::*;
use crate::utils::{ArbitrageError, ArbitrageResult};

// Exchange authentication helper
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

pub trait ExchangeInterface {
    async fn save_api_key(&self, exchange_id: &str, api_key: &str, api_secret: &str) -> ArbitrageResult<()>;
    async fn get_api_key(&self, exchange_id: &str) -> ArbitrageResult<Option<ExchangeCredentials>>;
    async fn delete_api_key(&self, exchange_id: &str) -> ArbitrageResult<()>;
    async fn load_markets(&self, exchange_id: &str) -> ArbitrageResult<HashMap<String, Market>>;
    async fn get_markets(&self, exchange_id: &str) -> ArbitrageResult<HashMap<String, Market>>;
    async fn get_ticker(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<Ticker>;
    async fn get_orderbook(&self, exchange_id: &str, symbol: &str, limit: Option<u32>) -> ArbitrageResult<OrderBook>;
    async fn get_funding_rate(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<FundingRateInfo>;
    async fn fetch_funding_rates(&self, exchange_id: &str, pairs: &[String]) -> ArbitrageResult<Vec<FundingRateInfo>>;
    async fn get_balance(&self, exchange_id: &str, currency: Option<&str>) -> ArbitrageResult<Balances>;
    async fn create_order(&self, exchange_id: &str, symbol: &str, order_type: OrderType, side: OrderSide, amount: f64, price: Option<f64>) -> ArbitrageResult<Order>;
    async fn cancel_order(&self, exchange_id: &str, order_id: &str, symbol: Option<&str>) -> ArbitrageResult<Order>;
    async fn get_open_orders(&self, exchange_id: &str, symbol: Option<&str>) -> ArbitrageResult<Vec<Order>>;
    async fn get_open_positions(&self, exchange_id: &str, symbol: Option<&str>) -> ArbitrageResult<Vec<Position>>;
    async fn set_leverage(&self, exchange_id: &str, symbol: &str, leverage: i32) -> ArbitrageResult<()>;
    async fn get_trading_fees(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<TradingFee>;
}

pub struct ExchangeService {
    http_client: Client,
    kv_store: Arc<KvStore>,
    markets_cache: HashMap<String, (HashMap<String, Market>, std::time::Instant)>,
    cache_ttl: std::time::Duration,
}

impl ExchangeService {
    pub fn new(env: &Env) -> ArbitrageResult<Self> {
        let kv_store = env.kv("ArbEdgeKV")
            .map_err(|e| ArbitrageError::database_error(format!("Failed to get KV store: {}", e)))?;
        
        Ok(Self {
            http_client: Client::new(),
            kv_store: Arc::new(kv_store),
            markets_cache: HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(300), // 5 minutes
        })
    }

    // Exchange-specific implementations
    async fn binance_request(&self, endpoint: &str, method: Method, params: Option<Value>, auth: Option<&ExchangeCredentials>) -> ArbitrageResult<Value> {
        let base_url = "https://api.binance.com";
        let url = format!("{}{}", base_url, endpoint);
        
        let mut request_url = Url::parse(&url).map_err(|e| ArbitrageError::network_error(format!("Invalid URL: {}", e)))?;
        
        // Add query parameters
        if let Some(params) = params {
            if let Some(obj) = params.as_object() {
                for (key, value) in obj {
                    if let Some(str_val) = value.as_str() {
                        request_url.query_pairs_mut().append_pair(key, str_val);
                    } else {
                        request_url.query_pairs_mut().append_pair(key, &value.to_string());
                    }
                }
            }
        }
        
        let mut request = self.http_client.request(method, request_url);
        
        // Add authentication if provided
        if let Some(creds) = auth {
            let timestamp = Utc::now().timestamp_millis();
            let query_string = format!("timestamp={}", timestamp);
            
            // Create signature
            let mut mac = Hmac::<Sha256>::new_from_slice(creds.secret.as_bytes())
                .map_err(|e| ArbitrageError::authentication_error(format!("Invalid secret key: {}", e)))?;
            mac.update(query_string.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            
            request = request.header("X-MBX-APIKEY", &creds.api_key);
            request = request.query(&[("timestamp", timestamp.to_string()), ("signature", signature)]);
        }
        
        let response = request.send().await
            .map_err(|e| ArbitrageError::network_error(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::api_error(format!("Binance API error: {}", error_text)));
        }
        
        let json: Value = response.json().await
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to parse JSON: {}", e)))?;
        Ok(json)
    }

    async fn bybit_request(&self, endpoint: &str, method: Method, params: Option<Value>, auth: Option<&ExchangeCredentials>) -> ArbitrageResult<Value> {
        let base_url = "https://api.bybit.com";
        let url = format!("{}{}", base_url, endpoint);
        
        let mut request = self.http_client.request(method, url);
        
        // Add authentication if provided
        if let Some(creds) = auth {
            let timestamp = Utc::now().timestamp_millis();
            let recv_window = 5000;
            
            let param_str = if let Some(ref params) = params {
                serde_json::to_string(params).unwrap_or_default()
            } else {
                "{}".to_string()
            };
            
            let sign_str = format!("{}{}{}{}", timestamp, &creds.api_key, recv_window, param_str);
            
            let mut mac = Hmac::<Sha256>::new_from_slice(creds.secret.as_bytes())
                .map_err(|e| ArbitrageError::authentication_error(format!("Invalid secret key: {}", e)))?;
            mac.update(sign_str.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            
            request = request.header("X-BAPI-API-KEY", &creds.api_key);
            request = request.header("X-BAPI-TIMESTAMP", timestamp.to_string());
            request = request.header("X-BAPI-RECV-WINDOW", recv_window.to_string());
            request = request.header("X-BAPI-SIGN", signature);
        }
        
        if let Some(params) = params {
            request = request.json(&params);
        }
        
        let response = request.send().await
            .map_err(|e| ArbitrageError::network_error(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::api_error(format!("Bybit API error: {}", error_text)));
        }
        
        let json: Value = response.json().await
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to parse JSON: {}", e)))?;
        Ok(json)
    }

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

    fn parse_bybit_ticker(&self, data: &Value, symbol: &str) -> ArbitrageResult<Ticker> {
        let result = &data["result"];
        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: result["bid1Price"].as_str().and_then(|s| s.parse().ok()),
            ask: result["ask1Price"].as_str().and_then(|s| s.parse().ok()),
            last: result["lastPrice"].as_str().and_then(|s| s.parse().ok()),
            high: result["highPrice24h"].as_str().and_then(|s| s.parse().ok()),
            low: result["lowPrice24h"].as_str().and_then(|s| s.parse().ok()),
            volume: result["volume24h"].as_str().and_then(|s| s.parse().ok()),
            timestamp: Some(Utc::now()),
            datetime: Some(Utc::now().to_rfc3339()),
        })
    }
}

impl ExchangeInterface for ExchangeService {
    async fn save_api_key(&self, exchange_id: &str, api_key: &str, api_secret: &str) -> ArbitrageResult<()> {
        let credentials = ExchangeCredentials {
            api_key: api_key.to_string(),
            secret: api_secret.to_string(),
            default_leverage: 20,
            exchange_type: exchange_id.to_string(),
        };
        
        let key = format!("exchange_credentials_{}", exchange_id);
        let value = serde_json::to_string(&credentials)
            .map_err(|e| ArbitrageError::serialization_error(format!("Failed to serialize credentials: {}", e)))?;
        
        self.kv_store.put(&key, value)
            .map_err(|e| ArbitrageError::database_error(format!("Failed to save credentials: {}", e)))?
            .execute()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute KV put: {}", e)))?;
        
        Ok(())
    }

    async fn get_api_key(&self, exchange_id: &str) -> ArbitrageResult<Option<ExchangeCredentials>> {
        let key = format!("exchange_credentials_{}", exchange_id);
        
        match self.kv_store.get(&key).text().await {
            Ok(Some(value)) => {
                let credentials = serde_json::from_str(&value)
                    .map_err(|e| ArbitrageError::serialization_error(format!("Failed to deserialize credentials: {}", e)))?;
                Ok(Some(credentials))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ArbitrageError::database_error(format!("Failed to get credentials: {}", e)))
        }
    }

    async fn delete_api_key(&self, exchange_id: &str) -> ArbitrageResult<()> {
        let key = format!("exchange_credentials_{}", exchange_id);
        self.kv_store.delete(&key).await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to delete credentials: {}", e)))?;
        Ok(())
    }

    async fn load_markets(&self, exchange_id: &str) -> ArbitrageResult<HashMap<String, Market>> {
        match exchange_id {
            "binance" => {
                let response = self.binance_request("/api/v3/exchangeInfo", Method::GET, None, None).await?;
                let symbols = response["symbols"].as_array()
                    .ok_or(ArbitrageError::api_error("Invalid exchange info response"))?;
                
                let mut markets = HashMap::new();
                for symbol_data in symbols {
                    if let Some(symbol) = symbol_data["symbol"].as_str() {
                        let market = Market {
                            id: symbol.to_string(),
                            symbol: symbol.to_string(),
                            base: symbol_data["baseAsset"].as_str().unwrap_or_default().to_string(),
                            quote: symbol_data["quoteAsset"].as_str().unwrap_or_default().to_string(),
                            active: symbol_data["status"].as_str() == Some("TRADING"),
                            precision: Precision {
                                amount: symbol_data["baseAssetPrecision"].as_i64().map(|x| x as i32),
                                price: symbol_data["quotePrecision"].as_i64().map(|x| x as i32),
                            },
                            limits: Limits {
                                amount: MinMax { min: None, max: None },
                                price: MinMax { min: None, max: None },
                                cost: MinMax { min: None, max: None },
                            },
                            fees: None,
                        };
                        markets.insert(symbol.to_string(), market);
                    }
                }
                Ok(markets)
            }
            "bybit" => {
                let response = self.bybit_request("/v5/market/instruments-info", Method::GET, Some(json!({"category": "linear"})), None).await?;
                let list = response["result"]["list"].as_array()
                    .ok_or(ArbitrageError::api_error("Invalid instruments info response"))?;
                
                let mut markets = HashMap::new();
                for instrument in list {
                    if let Some(symbol) = instrument["symbol"].as_str() {
                        let market = Market {
                            id: symbol.to_string(),
                            symbol: symbol.to_string(),
                            base: instrument["baseCoin"].as_str().unwrap_or_default().to_string(),
                            quote: instrument["quoteCoin"].as_str().unwrap_or_default().to_string(),
                            active: instrument["status"].as_str() == Some("Trading"),
                            precision: Precision {
                                amount: None,
                                price: None,
                            },
                            limits: Limits {
                                amount: MinMax { min: None, max: None },
                                price: MinMax { min: None, max: None },
                                cost: MinMax { min: None, max: None },
                            },
                            fees: None,
                        };
                        markets.insert(symbol.to_string(), market);
                    }
                }
                Ok(markets)
            }
            _ => Err(ArbitrageError::exchange_error(exchange_id, format!("Exchange {} not supported", exchange_id)))
        }
    }

    async fn get_markets(&self, exchange_id: &str) -> ArbitrageResult<HashMap<String, Market>> {
        // Check cache first
        if let Some((cached_markets, cached_time)) = self.markets_cache.get(exchange_id) {
            if cached_time.elapsed() < self.cache_ttl {
                return Ok(cached_markets.clone());
            }
        }
        
        // Load fresh markets and cache them
        let markets = self.load_markets(exchange_id).await?;
        // Note: In a real implementation, we'd need mutable access to update the cache
        Ok(markets)
    }

    async fn get_ticker(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<Ticker> {
        match exchange_id {
            "binance" => {
                let response = self.binance_request(
                    "/api/v3/ticker/24hr",
                    Method::GET,
                    Some(json!({"symbol": symbol})),
                    None
                ).await?;
                
                Ok(self.parse_binance_ticker(&response, symbol)?)
            }
            "bybit" => {
                let response = self.bybit_request(
                    "/v5/market/tickers",
                    Method::GET,
                    Some(json!({"category": "linear", "symbol": symbol})),
                    None
                ).await?;
                
                Ok(self.parse_bybit_ticker(&response, symbol)?)
            }
            _ => Err(ArbitrageError::exchange_error(exchange_id, format!("Exchange {} not supported", exchange_id)))
        }
    }

    async fn get_orderbook(&self, exchange_id: &str, symbol: &str, limit: Option<u32>) -> ArbitrageResult<OrderBook> {
        let limit = limit.unwrap_or(100);
        
        match exchange_id {
            "binance" => {
                let response = self.binance_request(
                    "/api/v3/depth",
                    Method::GET,
                    Some(json!({"symbol": symbol, "limit": limit})),
                    None
                ).await?;
                
                let mut bids = Vec::new();
                let mut asks = Vec::new();
                
                if let Some(bids_array) = response["bids"].as_array() {
                    for bid in bids_array {
                        if let Some(bid_array) = bid.as_array() {
                            if bid_array.len() >= 2 {
                                let price: f64 = bid_array[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                                let amount: f64 = bid_array[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                                bids.push([price, amount]);
                            }
                        }
                    }
                }
                
                if let Some(asks_array) = response["asks"].as_array() {
                    for ask in asks_array {
                        if let Some(ask_array) = ask.as_array() {
                            if ask_array.len() >= 2 {
                                let price: f64 = ask_array[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                                let amount: f64 = ask_array[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                                asks.push([price, amount]);
                            }
                        }
                    }
                }
                
                Ok(OrderBook {
                    symbol: symbol.to_string(),
                    bids,
                    asks,
                    timestamp: Some(Utc::now()),
                    datetime: Some(Utc::now().to_rfc3339()),
                })
            }
            _ => Err(ArbitrageError::exchange_error(exchange_id, format!("Exchange {} not supported", exchange_id)))
        }
    }

    async fn get_funding_rate(&self, exchange_id: &str, symbol: &str) -> ArbitrageResult<FundingRateInfo> {
        match exchange_id {
            "binance" => {
                let response = self.binance_request(
                    "/fapi/v1/fundingRate",
                    Method::GET,
                    Some(json!({"symbol": symbol})),
                    None
                ).await?;
                
                if let Some(funding_data) = response.as_array().and_then(|arr| arr.last()) {
                    Ok(FundingRateInfo {
                        symbol: symbol.to_string(),
                        funding_rate: funding_data["fundingRate"].as_str()
                            .and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        timestamp: Some(Utc::now()),
                        datetime: Some(Utc::now().to_rfc3339()),
                        next_funding_time: None,
                        estimated_rate: None,
                    })
                } else {
                    Err(ArbitrageError::exchange_error(exchange_id, "No funding rate data found"))
                }
            }
            "bybit" => {
                let response = self.bybit_request(
                    "/v5/market/funding/history",
                    Method::GET,
                    Some(json!({"category": "linear", "symbol": symbol, "limit": 1})),
                    None
                ).await?;
                
                if let Some(list) = response["result"]["list"].as_array() {
                    if let Some(funding_data) = list.first() {
                        Ok(FundingRateInfo {
                            symbol: symbol.to_string(),
                            funding_rate: funding_data["fundingRate"].as_str()
                                .and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            timestamp: Some(Utc::now()),
                            datetime: Some(Utc::now().to_rfc3339()),
                            next_funding_time: None,
                            estimated_rate: None,
                        })
                    } else {
                        Err(ArbitrageError::exchange_error(exchange_id, "No funding rate data found"))
                    }
                } else {
                    Err(ArbitrageError::exchange_error(exchange_id, "Invalid funding rate response"))
                }
            }
            _ => Err(ArbitrageError::exchange_error(exchange_id, format!("Exchange {} not supported", exchange_id)))
        }
    }

    async fn fetch_funding_rates(&self, exchange_id: &str, pairs: &[String]) -> ArbitrageResult<Vec<FundingRateInfo>> {
        let mut funding_rates = Vec::new();
        
        for pair in pairs {
            match self.get_funding_rate(exchange_id, pair).await {
                Ok(rate) => funding_rates.push(rate),
                Err(_) => continue, // Skip failed pairs
            }
        }
        
        Ok(funding_rates)
    }

    // Placeholder implementations for remaining methods
    async fn get_balance(&self, _exchange_id: &str, _currency: Option<&str>) -> ArbitrageResult<Balances> {
        // TODO: Implement balance fetching
        Ok(HashMap::new())
    }

    async fn create_order(&self, _exchange_id: &str, _symbol: &str, _order_type: OrderType, _side: OrderSide, _amount: f64, _price: Option<f64>) -> ArbitrageResult<Order> {
        // TODO: Implement order creation
        Err(ArbitrageError::not_implemented("Order creation not yet implemented".to_string()))
    }

    async fn cancel_order(&self, _exchange_id: &str, _order_id: &str, _symbol: Option<&str>) -> ArbitrageResult<Order> {
        // TODO: Implement order cancellation
        Err(ArbitrageError::not_implemented("Order cancellation not yet implemented".to_string()))
    }

    async fn get_open_orders(&self, _exchange_id: &str, _symbol: Option<&str>) -> ArbitrageResult<Vec<Order>> {
        // TODO: Implement open orders fetching
        Ok(Vec::new())
    }

    async fn get_open_positions(&self, _exchange_id: &str, _symbol: Option<&str>) -> ArbitrageResult<Vec<Position>> {
        // TODO: Implement open positions fetching
        Ok(Vec::new())
    }

    async fn set_leverage(&self, _exchange_id: &str, _symbol: &str, _leverage: i32) -> ArbitrageResult<()> {
        // TODO: Implement leverage setting
        Err(ArbitrageError::not_implemented("Leverage setting not yet implemented".to_string()))
    }

    async fn get_trading_fees(&self, _exchange_id: &str, _symbol: &str) -> ArbitrageResult<TradingFee> {
        // TODO: Implement trading fees fetching
        Ok(TradingFee {
            maker: 0.001,
            taker: 0.001,
            percentage: true,
        })
    }
} 