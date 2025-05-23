use worker::*;

// Module declarations
mod types;
mod utils;
mod services;

use utils::error::ArbitrageError;
use services::exchange::{ExchangeService, ExchangeInterface};
use serde_json::json;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let url = req.url()?;
    match url.path().as_ref() {
        "/kv-test" => {
            // Basic KV get/put demonstration
            let value = url.query().unwrap_or("default");
            let kv = env.kv("ArbEdgeKV")?;
            // Store value under a test key
            kv.put("test-key", value)?.execute().await?;
            // Retrieve stored value
            let retrieved = kv.get("test-key").text().await?;
            Response::ok(retrieved.unwrap_or_default())
        }
        "/health" => {
            Response::ok("ArbEdge Rust Worker is running!")
        }
        "/exchange/markets" => {
            // Test exchange markets endpoint
            let exchange_service = match ExchangeService::new(&env) {
                Ok(service) => service,
                Err(e) => return Response::error(format!("Failed to create exchange service: {}", e), 500)
            };
            
            let exchange_id = url.query_pairs()
                .find(|(key, _)| key == "exchange")
                .map(|(_, value)| value.to_string())
                .unwrap_or_else(|| "binance".to_string());
            
            match exchange_service.get_markets(&exchange_id).await {
                Ok(markets) => {
                    let market_count = markets.len();
                    let sample_markets: Vec<_> = markets.into_iter().take(5).collect();
                    let response = json!({
                        "exchange": exchange_id,
                        "total_markets": market_count,
                        "sample_markets": sample_markets
                    });
                    Response::from_json(&response)
                }
                Err(e) => Response::error(format!("Failed to get markets: {}", e), 500)
            }
        }
        "/exchange/ticker" => {
            // Test exchange ticker endpoint
            let exchange_service = match ExchangeService::new(&env) {
                Ok(service) => service,
                Err(e) => return Response::error(format!("Failed to create exchange service: {}", e), 500)
            };
            
            let query_pairs: std::collections::HashMap<String, String> = url.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            
            let exchange_id = query_pairs.get("exchange").cloned().unwrap_or_else(|| "binance".to_string());
            let symbol = query_pairs.get("symbol").cloned().unwrap_or_else(|| "BTCUSDT".to_string());
            
            match exchange_service.get_ticker(&exchange_id, &symbol).await {
                Ok(ticker) => Response::from_json(&ticker),
                Err(e) => Response::error(format!("Failed to get ticker: {}", e), 500)
            }
        }
        "/exchange/funding" => {
            // Test exchange funding rate endpoint
            let exchange_service = match ExchangeService::new(&env) {
                Ok(service) => service,
                Err(e) => return Response::error(format!("Failed to create exchange service: {}", e), 500)
            };
            
            let query_pairs: std::collections::HashMap<String, String> = url.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            
            let exchange_id = query_pairs.get("exchange").cloned().unwrap_or_else(|| "binance".to_string());
            let symbol = query_pairs.get("symbol").cloned().unwrap_or_else(|| "BTCUSDT".to_string());
            
            match exchange_service.get_funding_rate(&exchange_id, &symbol).await {
                Ok(funding_rate) => Response::from_json(&funding_rate),
                Err(e) => Response::error(format!("Failed to get funding rate: {}", e), 500)
            }
        }
        _ => Response::ok("Hello, ArbEdge in Rust! Available endpoints: /health, /kv-test, /exchange/markets, /exchange/ticker, /exchange/funding"),
    }
} 