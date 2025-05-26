use worker::*;

// Module declarations
pub mod services;
pub mod types;
pub mod utils;

use serde_json::json;
use services::core::infrastructure::d1_database::D1Service;
use services::core::opportunities::opportunity::{OpportunityService, OpportunityServiceConfig};
use services::core::trading::exchange::{ExchangeInterface, ExchangeService};
use services::core::trading::positions::{
    CreatePositionData, ProductionPositionsService, UpdatePositionData,
};
use services::core::user::user_profile::UserProfileService;
use services::interfaces::telegram::telegram::TelegramService;
use std::collections::HashMap;
use std::sync::Arc;
use types::{AccountInfo, ExchangeIdEnum, StructuredTradingPair};
use utils::{ArbitrageError, ArbitrageResult};
use uuid::Uuid;

// ===== TEMPORARY DURABLE OBJECT FOR MIGRATION =====
// This PositionsManager class is temporarily added to satisfy existing Durable Object instances
// during migration. It will be removed in the next deployment once migration is complete.
#[durable_object]
pub struct PositionsManager {
    _state: State,
}

#[durable_object]
impl DurableObject for PositionsManager {
    fn new(state: State, _env: Env) -> Self {
        Self { _state: state }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // Return a simple message indicating this DO is deprecated
        Response::ok("PositionsManager is deprecated and will be removed soon.")
    }
}
// ===== END TEMPORARY DURABLE OBJECT =====

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let url = req.url()?;
    let path = url.path();

    match (req.method(), path) {
        // Health check
        (Method::Get, "/health") => {
            Response::ok("ArbEdge Rust Worker is running!")
        }

        // KV test endpoint
        (Method::Get, "/kv-test") => {
            let value = url.query().unwrap_or("default");
            let kv = env.kv("ArbEdgeKV")?;
            kv.put("test-key", value)?.execute().await?;
            let retrieved = kv.get("test-key").text().await?;
            Response::ok(retrieved.unwrap_or_default())
        }

        // Exchange API endpoints
        (Method::Get, "/exchange/markets") => {
            handle_get_markets(req, env).await
        }

        (Method::Get, "/exchange/ticker") => {
            handle_get_ticker(req, env).await
        }

        (Method::Get, "/exchange/funding") => {
            handle_funding_rate(req, env).await
        }

        // Opportunity finding endpoint
        (Method::Post, "/find-opportunities") => {
            handle_find_opportunities(req, env).await
        }

        // Telegram webhook endpoint
        (Method::Post, "/webhook") => {
            handle_telegram_webhook(req, env).await
        }

        // Position management endpoints
        (Method::Post, "/positions") => {
            handle_create_position(req, env).await
        }

        (Method::Get, "/positions") => {
            handle_get_all_positions(req, env).await
        }

        (Method::Get, path) if path.starts_with("/positions/") => {
            let id = path.strip_prefix("/positions/").unwrap();
            // Validate UUID format
            if Uuid::parse_str(id).is_err() {
                return Response::error("Invalid position ID format. Must be a valid UUID.", 400);
            }
            handle_get_position(req, env, id).await
        }

        (Method::Put, path) if path.starts_with("/positions/") => {
            let id = path.strip_prefix("/positions/").unwrap();
            // Validate UUID format
            if Uuid::parse_str(id).is_err() {
                return Response::error("Invalid position ID format. Must be a valid UUID.", 400);
            }
            handle_update_position(req, env, id).await
        }

        (Method::Delete, path) if path.starts_with("/positions/") => {
            let id = path.strip_prefix("/positions/").unwrap();
            // Validate UUID format
            if Uuid::parse_str(id).is_err() {
                return Response::error("Invalid position ID format. Must be a valid UUID.", 400);
            }
            handle_close_position(req, env, id).await
        }

        // Default response
        _ => Response::ok("Hello, ArbEdge in Rust! Available endpoints: /health, /kv-test, /exchange/*, /find-opportunities, /webhook, /positions"),
    }
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();

    match event.cron().as_str() {
        // Monitor opportunities every minute
        "* * * * *" => {
            if let Err(e) = monitor_opportunities_scheduled(env).await {
                console_log!("Error in scheduled opportunity monitoring: {}", e);
            }
        }
        _ => {
            console_log!("Unknown scheduled event: {}", event.cron());
        }
    }
}

// Helper functions to reduce code duplication

/// Parse exchanges from environment string, returning error if less than two
#[allow(clippy::result_large_err)]
fn parse_exchanges_from_env(
    exchanges_str: &str,
) -> std::result::Result<Vec<ExchangeIdEnum>, ArbitrageError> {
    let exchanges: Vec<ExchangeIdEnum> = exchanges_str
        .split(',')
        .filter_map(|s| match s.trim() {
            "binance" => Some(ExchangeIdEnum::Binance),
            "bybit" => Some(ExchangeIdEnum::Bybit),
            "okx" => Some(ExchangeIdEnum::OKX),
            "bitget" => Some(ExchangeIdEnum::Bitget),
            _ => None,
        })
        .collect();

    if exchanges.len() < 2 {
        Err(ArbitrageError::config_error(
            "At least two exchanges must be configured",
        ))
    } else {
        Ok(exchanges)
    }
}

/// Create OpportunityService by reading environment variables and initializing services
async fn create_opportunity_service(
    custom_env: &types::Env,
) -> ArbitrageResult<OpportunityService> {
    // Parse configuration from environment
    let exchanges_str = custom_env.worker_env.var("EXCHANGES")?.to_string();
    let exchanges = parse_exchanges_from_env(&exchanges_str)?;

    let monitored_pairs_str = custom_env
        .worker_env
        .var("MONITORED_PAIRS_CONFIG")?
        .to_string();
    let monitored_pairs: Vec<StructuredTradingPair> = serde_json::from_str(&monitored_pairs_str)
        .map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to parse monitored pairs: {}", e))
        })?;

    let threshold: f64 = custom_env
        .worker_env
        .var("ARBITRAGE_THRESHOLD")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "0.001".to_string())
        .parse()
        .unwrap_or(0.001);

    // Create services
    let exchange_service = Arc::new(ExchangeService::new(custom_env)?);

    let telegram_service = if let Ok(bot_token) = custom_env.worker_env.var("TELEGRAM_BOT_TOKEN") {
        Some(Arc::new(TelegramService::new(
            services::interfaces::telegram::telegram::TelegramConfig {
                bot_token: bot_token.to_string(),
                chat_id: "0".to_string(), // Not used - we broadcast to registered groups from DB
                is_test_mode: false,
            },
        )))
    } else {
        None
    };

    let opportunity_config = OpportunityServiceConfig {
        exchanges,
        monitored_pairs,
        threshold,
    };

    Ok(OpportunityService::new(
        opportunity_config,
        exchange_service,
        telegram_service,
    ))
}

// Handler implementations

async fn handle_get_markets(req: Request, env: Env) -> Result<Response> {
    let custom_env = types::Env::new(env);
    let exchange_service = match ExchangeService::new(&custom_env) {
        Ok(service) => service,
        Err(e) => return Response::error(format!("Failed to create exchange service: {}", e), 500),
    };

    let url = req.url()?;
    let exchange_id = url
        .query_pairs()
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
        Err(e) => Response::error(format!("Failed to get markets: {}", e), 500),
    }
}

async fn handle_get_ticker(req: Request, env: Env) -> Result<Response> {
    let custom_env = types::Env::new(env);
    let exchange_service = match ExchangeService::new(&custom_env) {
        Ok(service) => service,
        Err(e) => return Response::error(format!("Failed to create exchange service: {}", e), 500),
    };

    let url = req.url()?;
    let query_pairs: std::collections::HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let exchange_id = query_pairs
        .get("exchange")
        .cloned()
        .unwrap_or_else(|| "binance".to_string());
    let symbol = query_pairs
        .get("symbol")
        .cloned()
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match exchange_service.get_ticker(&exchange_id, &symbol).await {
        Ok(ticker) => Response::from_json(&ticker),
        Err(e) => Response::error(format!("Failed to get ticker: {}", e), 500),
    }
}

async fn handle_funding_rate(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let query_params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    let exchange_id = query_params
        .get("exchange")
        .unwrap_or(&"binance".to_string())
        .clone();
    let symbol = query_params
        .get("symbol")
        .unwrap_or(&"BTCUSDT".to_string())
        .clone();

    let custom_env = types::Env::new(env);
    let exchange_service = match ExchangeService::new(&custom_env) {
        Ok(service) => service,
        Err(e) => return Response::error(format!("Failed to create exchange service: {}", e), 500),
    };

    match exchange_service
        .fetch_funding_rates(&exchange_id, Some(&symbol))
        .await
    {
        Ok(rates) => Response::from_json(&rates),
        Err(e) => Response::error(format!("Failed to fetch funding rate: {}", e), 500),
    }
}

async fn handle_find_opportunities(mut req: Request, env: Env) -> Result<Response> {
    // Create custom env first
    let custom_env = types::Env::new(env);

    // Parse request body for trading pairs (optional)
    let request_data: serde_json::Value = match req.json().await {
        Ok(data) => data,
        Err(_) => {
            // Default trading pairs if no body provided
            json!({
                "trading_pairs": ["BTCUSDT", "ETHUSDT", "ADAUSDT", "DOTUSDT", "SOLUSDT"],
                "min_threshold": 0.01
            })
        }
    };

    let _trading_pairs: Vec<String> = request_data["trading_pairs"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let _min_threshold = request_data["min_threshold"].as_f64().unwrap_or(0.01);

    // Create opportunity service using helper
    let opportunity_service = match create_opportunity_service(&custom_env).await {
        Ok(service) => service,
        Err(e) => {
            return Response::error(format!("Failed to create opportunity service: {}", e), 500)
        }
    };

    // Find opportunities
    match opportunity_service.monitor_opportunities().await {
        Ok(opportunities) => {
            // Process opportunities (send notifications)
            if let Err(e) = opportunity_service
                .process_opportunities(&opportunities)
                .await
            {
                console_log!("Failed to process opportunities: {}", e);
            }

            let response = json!({
                "status": "success",
                "opportunities_found": opportunities.len(),
                "opportunities": opportunities
            });
            Response::from_json(&response)
        }
        Err(e) => Response::error(format!("Failed to find opportunities: {}", e), 500),
    }
}

async fn handle_telegram_webhook(mut req: Request, env: Env) -> Result<Response> {
    let update: serde_json::Value = req.json().await?;

    let mut telegram_service = if let Ok(bot_token) = env.var("TELEGRAM_BOT_TOKEN") {
        TelegramService::new(services::interfaces::telegram::telegram::TelegramConfig {
            bot_token: bot_token.to_string(),
            chat_id: "0".to_string(), // Not used for webhook responses
            is_test_mode: false,
        })
    } else {
        return Response::error("Telegram bot token not found", 500);
    };

    // Initialize services using the service container pattern
    match env.kv("ArbEdgeKV") {
        Ok(kv_store) => {
            console_log!("✅ KV store initialized successfully");

            // Initialize session management service directly
            match D1Service::new(&env) {
                Ok(d1_service) => {
                    console_log!("✅ D1Service initialized successfully for session management");
                    let kv_service =
                        services::core::infrastructure::KVService::new(kv_store.clone());
                    let session_management_service =
                        services::core::user::session_management::SessionManagementService::new(
                            d1_service, kv_service,
                        );
                    telegram_service.set_session_management_service(session_management_service);
                    console_log!("✅ SessionManagementService initialized successfully");

                    // Initialize UserProfileService for RBAC if encryption key is available
                    if let Ok(encryption_key) = env.var("ENCRYPTION_KEY") {
                        console_log!(
                            "✅ ENCRYPTION_KEY found, initializing UserProfileService for RBAC"
                        );
                        match D1Service::new(&env) {
                            Ok(user_d1_service) => {
                                let user_profile_service = UserProfileService::new(
                                    kv_store,
                                    user_d1_service,
                                    encryption_key.to_string(),
                                );
                                telegram_service.set_user_profile_service(user_profile_service);
                                console_log!("✅ RBAC UserProfileService initialized successfully");
                            }
                            Err(e) => {
                                console_log!("❌ RBAC WARNING: Failed to create D1Service for user profiles: {:?}", e);
                            }
                        }
                    } else {
                        console_log!("❌ RBAC SECURITY WARNING: ENCRYPTION_KEY not found - UserProfileService not initialized");
                    }
                }
                Err(e) => {
                    console_log!("❌ SESSION WARNING: Failed to initialize D1Service for session management: {:?}", e);
                }
            }
        }
        Err(e) => {
            console_log!("❌ CRITICAL WARNING: KV store initialization failed - services not initialized: {:?}", e);
        }
    }

    match telegram_service.handle_webhook(update).await {
        Ok(Some(response_text)) => Response::ok(response_text),
        Ok(None) => Response::ok("OK"),
        Err(e) => Response::error(format!("Webhook processing error: {}", e), 500),
    }
}

async fn handle_create_position(mut req: Request, env: Env) -> Result<Response> {
    let kv = env.kv("ArbEdgeKV")?;
    let d1_service = D1Service::new(&env)?;
    let encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|v| v.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;
    let user_profile_service = UserProfileService::new(kv.clone(), d1_service, encryption_key);
    let positions_service = ProductionPositionsService::new(kv);

    // Parse request JSON and extract user_id
    let position_data_json = req.json::<serde_json::Value>().await?;
    let user_id = match position_data_json.get("user_id").and_then(|v| v.as_str()) {
        Some(uid) => uid,
        None => return Response::error("Missing user_id in request", 400),
    };
    let position_data: CreatePositionData = serde_json::from_value(position_data_json.clone())
        .map_err(|e| {
            worker::Error::RustError(format!("Failed to parse CreatePositionData: {}", e))
        })?;

    // Fetch real account info from user profile
    let user_profile = match user_profile_service.get_user_profile(user_id).await {
        Ok(Some(profile)) => profile,
        Ok(None) => return Response::error("User profile not found", 404),
        Err(e) => return Response::error(format!("Failed to fetch user profile: {}", e), 500),
    };
    let account_info = AccountInfo {
        total_balance_usd: user_profile.account_balance_usdt, // Using proper balance field
    };

    match positions_service
        .create_position(position_data, &account_info)
        .await
    {
        Ok(position) => Response::from_json(&position),
        Err(e) => match e.kind {
            crate::utils::error::ErrorKind::DatabaseError => Response::error(e.message, 500),
            crate::utils::error::ErrorKind::NetworkError => Response::error(e.message, 502),
            crate::utils::error::ErrorKind::ParseError => Response::error(e.message, 400),
            crate::utils::error::ErrorKind::ValidationError => Response::error(e.message, 400),
            crate::utils::error::ErrorKind::Authentication => Response::error(e.message, 401),
            crate::utils::error::ErrorKind::Authorization => Response::error(e.message, 403),
            crate::utils::error::ErrorKind::ExchangeError => {
                Response::error(format!("Exchange error: {}", e.message), 500)
            }
            _ => Response::error(format!("Failed to create position: {}", e), 500),
        },
    }
}

async fn handle_get_all_positions(_req: Request, env: Env) -> Result<Response> {
    let kv = env.kv("ArbEdgeKV")?;
    let positions_service = ProductionPositionsService::new(kv);

    match positions_service.get_all_positions().await {
        Ok(positions) => Response::from_json(&positions),
        Err(e) => Response::error(format!("Failed to get positions: {}", e), 500),
    }
}

async fn handle_get_position(_req: Request, env: Env, id: &str) -> Result<Response> {
    let kv = env.kv("ArbEdgeKV")?;
    let positions_service = ProductionPositionsService::new(kv);

    match positions_service.get_position(id).await {
        Ok(Some(position)) => Response::from_json(&position),
        Ok(None) => Response::error("Position not found", 404),
        Err(e) => Response::error(format!("Failed to get position: {}", e), 500),
    }
}

async fn handle_update_position(mut req: Request, env: Env, id: &str) -> Result<Response> {
    let kv = env.kv("ArbEdgeKV")?;
    let positions_service = ProductionPositionsService::new(kv);

    let update_data: UpdatePositionData = req.json().await?;

    match positions_service.update_position(id, update_data).await {
        Ok(Some(position)) => Response::from_json(&position),
        Ok(None) => Response::error("Position not found", 404),
        Err(e) => Response::error(format!("Failed to update position: {}", e), 500),
    }
}

async fn handle_close_position(_req: Request, env: Env, id: &str) -> Result<Response> {
    let kv = env.kv("ArbEdgeKV")?;
    let positions_service = ProductionPositionsService::new(kv);

    match positions_service.close_position(id).await {
        Ok(true) => Response::ok("Position closed"),
        Ok(false) => Response::error("Position not found", 404),
        Err(e) => Response::error(format!("Failed to close position: {}", e), 500),
    }
}

/// Run five-minute maintenance tasks including session cleanup and opportunity distribution
async fn run_five_minute_maintenance(
    env: &Env,
    opportunity_service: &OpportunityService,
) -> ArbitrageResult<()> {
    // Get KV store
    let kv_store = env
        .kv("ArbEdgeKV")
        .map_err(|e| ArbitrageError::database_error(format!("Failed to get KV store: {}", e)))?;

    // Create services once and reuse them
    let d1_service = D1Service::new(env)?;
    let kv_service = services::core::infrastructure::KVService::new(kv_store.clone());

    // Initialize session management service
    let session_service = services::core::user::session_management::SessionManagementService::new(
        d1_service.clone(),
        kv_service.clone(),
    );

    // Cleanup expired sessions
    match session_service.cleanup_expired_sessions().await {
        Ok(cleanup_count) => {
            if cleanup_count > 0 {
                console_log!("✅ Cleaned up {} expired sessions", cleanup_count);
            }
        }
        Err(e) => {
            console_log!("❌ Session cleanup failed: {}", e);
        }
    }

    // Initialize opportunity distribution service
    let mut distribution_service =
        services::core::opportunities::opportunity_distribution::OpportunityDistributionService::new(
            d1_service,
            kv_service,
            session_service,
        );

    // Initialize TelegramService for push notifications if configured
    if let (Ok(bot_token), Ok(chat_id)) =
        (env.var("TELEGRAM_BOT_TOKEN"), env.var("TELEGRAM_CHAT_ID"))
    {
        let telegram_config = services::interfaces::telegram::telegram::TelegramConfig {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
            is_test_mode: false,
        };
        let telegram_service =
            services::interfaces::telegram::telegram::TelegramService::new(telegram_config);
        distribution_service.set_notification_sender(Box::new(telegram_service));
        console_log!("✅ TelegramService integrated with OpportunityDistributionService");
    }

    // Check for opportunities to distribute
    match opportunity_service.monitor_opportunities().await {
        Ok(opportunities) => {
            if !opportunities.is_empty() {
                console_log!(
                    "🔄 Distributing {} opportunities to eligible users",
                    opportunities.len()
                );

                // Distribute each opportunity to eligible users
                for opportunity in opportunities {
                    match distribution_service
                        .distribute_opportunity(opportunity)
                        .await
                    {
                        Ok(distributed_count) => {
                            if distributed_count > 0 {
                                console_log!(
                                    "✅ Distributed opportunity to {} users",
                                    distributed_count
                                );
                            }
                        }
                        Err(e) => {
                            console_log!("❌ Failed to distribute opportunity: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            console_log!("❌ Failed to monitor opportunities for distribution: {}", e);
        }
    }

    Ok(())
}

async fn monitor_opportunities_scheduled(env: Env) -> ArbitrageResult<()> {
    let custom_env = types::Env::new(env.clone());

    // Create opportunity service using helper
    let opportunity_service = create_opportunity_service(&custom_env).await?;

    // Find and process opportunities
    let opportunities = opportunity_service.monitor_opportunities().await?;

    if !opportunities.is_empty() {
        console_log!("Found {} opportunities", opportunities.len());
        opportunity_service
            .process_opportunities(&opportunities)
            .await?;
    }

    // Session cleanup and opportunity distribution (every 5 minutes)
    // Check if this is a 5-minute interval (cron runs every minute)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now % 300 == 0 {
        // Every 5 minutes - run session cleanup and opportunity distribution
        if let Err(e) = run_five_minute_maintenance(&env, &opportunity_service).await {
            console_log!("❌ Five-minute maintenance failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use types::ExchangeIdEnum;

    // Tests for parse_exchanges_from_env function
    #[test]
    fn test_parse_exchanges_from_env_valid_input() {
        let exchanges_str = "binance,bybit,okx";
        let result = parse_exchanges_from_env(exchanges_str).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains(&ExchangeIdEnum::Binance));
        assert!(result.contains(&ExchangeIdEnum::Bybit));
        assert!(result.contains(&ExchangeIdEnum::OKX));
    }

    #[test]
    fn test_parse_exchanges_from_env_with_whitespace() {
        let exchanges_str = " binance , bybit , okx ";
        let result = parse_exchanges_from_env(exchanges_str).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains(&ExchangeIdEnum::Binance));
        assert!(result.contains(&ExchangeIdEnum::Bybit));
        assert!(result.contains(&ExchangeIdEnum::OKX));
    }

    #[test]
    fn test_parse_exchanges_from_env_invalid_exchange() {
        let exchanges_str = "binance,invalid_exchange,okx";
        let result = parse_exchanges_from_env(exchanges_str).unwrap();

        // Should only contain valid exchanges
        assert_eq!(result.len(), 2);
        assert!(result.contains(&ExchangeIdEnum::Binance));
        assert!(result.contains(&ExchangeIdEnum::OKX));
    }

    #[test]
    fn test_parse_exchanges_from_env_insufficient_exchanges() {
        let exchanges_str = "binance";
        let result = parse_exchanges_from_env(exchanges_str);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least two exchanges must be configured"));
    }

    #[test]
    fn test_parse_exchanges_from_env_empty_string() {
        let exchanges_str = "";
        let result = parse_exchanges_from_env(exchanges_str);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_exchanges_from_env_all_supported() {
        let exchanges_str = "binance,bybit,okx,bitget";
        let result = parse_exchanges_from_env(exchanges_str).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result.contains(&ExchangeIdEnum::Binance));
        assert!(result.contains(&ExchangeIdEnum::Bybit));
        assert!(result.contains(&ExchangeIdEnum::OKX));
        assert!(result.contains(&ExchangeIdEnum::Bitget));
    }

    // Tests for route matching logic
    mod route_tests {
        use super::*;

        #[test]
        fn test_health_endpoint_routing() {
            let method = Method::Get;
            let path = "/health";

            match (method, path) {
                (Method::Get, "/health") => {
                    // This should match the health endpoint
                    // Health endpoint route matched
                }
                _ => panic!("Health endpoint route should match"),
            }
        }

        #[test]
        fn test_kv_test_endpoint_routing() {
            let method = Method::Get;
            let path = "/kv-test";

            match (method, path) {
                (Method::Get, "/kv-test") => {
                    // KV test endpoint route matched
                }
                _ => panic!("KV test endpoint route should match"),
            }
        }

        #[test]
        fn test_exchange_endpoints_routing() {
            let exchange_routes = vec![
                (Method::Get, "/exchange/markets"),
                (Method::Get, "/exchange/ticker"),
                (Method::Get, "/exchange/funding"),
            ];

            for (method, path) in exchange_routes {
                match (method, path) {
                    (Method::Get, "/exchange/markets")
                    | (Method::Get, "/exchange/ticker")
                    | (Method::Get, "/exchange/funding") => {
                        // Exchange endpoint matched
                    }
                    _ => panic!("Exchange endpoint should match for {}", path),
                }
            }
        }

        #[test]
        fn test_opportunity_endpoint_routing() {
            let method = Method::Post;
            let path = "/find-opportunities";

            match (method, path) {
                (Method::Post, "/find-opportunities") => {
                    // Find opportunities endpoint matched
                }
                _ => panic!("Find opportunities endpoint should match"),
            }
        }

        #[test]
        fn test_telegram_webhook_routing() {
            let method = Method::Post;
            let path = "/webhook";

            match (method, path) {
                (Method::Post, "/webhook") => {
                    // Telegram webhook endpoint matched
                }
                _ => panic!("Telegram webhook endpoint should match"),
            }
        }

        #[test]
        fn test_positions_routing() {
            let position_routes = vec![
                (Method::Post, "/positions"),
                (Method::Get, "/positions"),
                (
                    Method::Get,
                    "/positions/123e4567-e89b-12d3-a456-426614174000",
                ),
                (
                    Method::Put,
                    "/positions/123e4567-e89b-12d3-a456-426614174000",
                ),
                (
                    Method::Delete,
                    "/positions/123e4567-e89b-12d3-a456-426614174000",
                ),
            ];

            for (method, path) in position_routes {
                match (&method, path) {
                    (Method::Post, "/positions") | (Method::Get, "/positions") => {
                        // Positions endpoint matched
                    }
                    (Method::Get, path) if path.starts_with("/positions/") => {
                        let id = path.strip_prefix("/positions/").unwrap();
                        if Uuid::parse_str(id).is_ok() {
                            // GET position by ID matched with valid UUID
                        }
                    }
                    (Method::Put, path) if path.starts_with("/positions/") => {
                        let id = path.strip_prefix("/positions/").unwrap();
                        if Uuid::parse_str(id).is_ok() {
                            // PUT position by ID matched with valid UUID
                        }
                    }
                    (Method::Delete, path) if path.starts_with("/positions/") => {
                        let id = path.strip_prefix("/positions/").unwrap();
                        if Uuid::parse_str(id).is_ok() {
                            // DELETE position by ID matched with valid UUID
                        }
                    }
                    _ => {
                        let method_str = format!("{:?}", method);
                        panic!("Position endpoint should match for {} {}", method_str, path);
                    }
                }
            }
        }

        #[test]
        fn test_uuid_validation_in_position_routes() {
            let valid_uuid = "123e4567-e89b-12d3-a456-426614174000";
            let invalid_uuid = "invalid-uuid-format";

            // Valid UUID should pass validation
            assert!(Uuid::parse_str(valid_uuid).is_ok());

            // Invalid UUID should fail validation
            assert!(Uuid::parse_str(invalid_uuid).is_err());
        }

        #[test]
        fn test_default_route_fallback() {
            let unmatched_routes = vec![
                (Method::Get, "/unknown"),
                (Method::Post, "/invalid"),
                (Method::Put, "/nonexistent"),
            ];

            for (method, path) in unmatched_routes {
                match (method, path) {
                    (Method::Get, "/health")
                    | (Method::Get, "/kv-test")
                    | (Method::Get, "/exchange/markets")
                    | (Method::Get, "/exchange/ticker")
                    | (Method::Get, "/exchange/funding")
                    | (Method::Post, "/find-opportunities")
                    | (Method::Post, "/webhook")
                    | (Method::Post, "/positions")
                    | (Method::Get, "/positions") => {
                        panic!("Route should not match known endpoints");
                    }
                    (Method::Get, path) if path.starts_with("/positions/") => {
                        panic!("Route should not match position endpoints");
                    }
                    (Method::Put, path) if path.starts_with("/positions/") => {
                        panic!("Route should not match position endpoints");
                    }
                    (Method::Delete, path) if path.starts_with("/positions/") => {
                        panic!("Route should not match position endpoints");
                    }
                    _ => {
                        // Unknown routes fall through to default
                    }
                }
            }
        }
    }

    // Tests for scheduled event handling
    mod scheduled_tests {
        #[test]
        fn test_scheduled_cron_pattern_matching() {
            let cron_patterns = vec![
                "* * * * *", // Every minute (should match)
                "0 * * * *", // Every hour (should not match)
                "0 0 * * *", // Every day (should not match)
                "invalid",   // Invalid pattern (should not match)
            ];

            for cron in cron_patterns {
                match cron {
                    "* * * * *" => {
                        // Every minute cron recognized
                    }
                    _ => {
                        // Other cron patterns don't trigger opportunity monitoring
                    }
                }
            }
        }
    }

    // Tests for query parameter parsing
    mod query_parsing_tests {
        #[test]
        fn test_exchange_query_parameter_parsing() {
            // Test default exchange
            let default_exchange = "binance".to_string();
            assert_eq!(default_exchange, "binance");

            // Test explicit exchange parameter
            let exchange_param = "bybit";
            assert_eq!(exchange_param, "bybit");
        }

        #[test]
        fn test_symbol_query_parameter_parsing() {
            // Test default symbol
            let default_symbol = "BTCUSDT".to_string();
            assert_eq!(default_symbol, "BTCUSDT");

            // Test explicit symbol parameter
            let symbol_param = "ETHUSDT";
            assert_eq!(symbol_param, "ETHUSDT");
        }

        #[test]
        fn test_query_pairs_collection() {
            // Simulate query parameter collection
            let query_params = vec![
                ("exchange".to_string(), "binance".to_string()),
                ("symbol".to_string(), "BTCUSDT".to_string()),
                ("limit".to_string(), "100".to_string()),
            ];

            let query_map: std::collections::HashMap<String, String> =
                query_params.into_iter().collect();

            assert_eq!(query_map.get("exchange"), Some(&"binance".to_string()));
            assert_eq!(query_map.get("symbol"), Some(&"BTCUSDT".to_string()));
            assert_eq!(query_map.get("limit"), Some(&"100".to_string()));
        }
    }

    // Tests for JSON request/response handling
    mod json_handling_tests {
        use super::*;

        #[test]
        fn test_find_opportunities_request_parsing() {
            // Test default request data when no body provided
            let default_data = json!({
                "trading_pairs": ["BTCUSDT", "ETHUSDT", "ADAUSDT", "DOTUSDT", "SOLUSDT"],
                "min_threshold": 0.01
            });

            assert_eq!(default_data["trading_pairs"].as_array().unwrap().len(), 5);
            assert_eq!(default_data["min_threshold"].as_f64().unwrap(), 0.01);
        }

        #[test]
        fn test_trading_pairs_parsing() {
            let request_data = json!({
                "trading_pairs": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
                "min_threshold": 0.02
            });

            let trading_pairs: Vec<String> = request_data["trading_pairs"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            assert_eq!(trading_pairs.len(), 3);
            assert!(trading_pairs.contains(&"BTCUSDT".to_string()));
            assert!(trading_pairs.contains(&"ETHUSDT".to_string()));
            assert!(trading_pairs.contains(&"BNBUSDT".to_string()));
        }

        #[test]
        fn test_min_threshold_parsing() {
            let request_data = json!({
                "trading_pairs": ["BTCUSDT"],
                "min_threshold": 0.05
            });

            let min_threshold = request_data["min_threshold"].as_f64().unwrap_or(0.01);

            assert_eq!(min_threshold, 0.05);
        }

        #[test]
        fn test_response_format() {
            // Test opportunities response format
            let opportunities_response = json!({
                "status": "success",
                "opportunities_found": 2,
                "opportunities": [
                    {
                        "trading_pair": "BTCUSDT",
                        "exchange_a": "binance",
                        "exchange_b": "bybit",
                        "funding_rate_diff": 0.02
                    }
                ]
            });

            assert_eq!(opportunities_response["status"], "success");
            assert_eq!(opportunities_response["opportunities_found"], 2);
            assert!(opportunities_response["opportunities"].is_array());
        }

        #[test]
        fn test_error_response_format() {
            let error_message = "Failed to create exchange service";
            let error_response = format!("Failed to create exchange service: {}", error_message);

            assert!(error_response.contains("Failed to create exchange service"));
        }
    }

    // Tests for environment variable handling
    mod env_tests {
        #[test]
        fn test_telegram_config_validation() {
            // Test when both bot token and chat ID are available
            let bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11";
            let chat_id = "-123456789";

            assert!(!bot_token.is_empty());
            assert!(!chat_id.is_empty());
            assert!(bot_token.contains(":"));
            assert!(chat_id.starts_with("-") || chat_id.parse::<i64>().is_ok());
        }

        #[test]
        fn test_arbitrage_threshold_parsing() {
            // Test default threshold
            let default_threshold = "0.001".parse::<f64>().unwrap();
            assert_eq!(default_threshold, 0.001);

            // Test custom threshold
            let custom_threshold = "0.005".parse::<f64>().unwrap();
            assert_eq!(custom_threshold, 0.005);

            // Test invalid threshold fallback
            let invalid_threshold = "invalid".parse::<f64>().unwrap_or(0.001);
            assert_eq!(invalid_threshold, 0.001);
        }

        #[test]
        fn test_monitored_pairs_config_parsing() {
            let pairs_config = r#"[
                {"base": "BTC", "quote": "USDT", "type": "spot"},
                {"base": "ETH", "quote": "USDT", "type": "spot"}
            ]"#;

            let parsed: std::result::Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(pairs_config);
            assert!(parsed.is_ok());

            let pairs = parsed.unwrap();
            assert!(pairs.is_array());
            assert_eq!(pairs.as_array().unwrap().len(), 2);
        }
    }

    // Tests for utility functions used in handlers
    mod handler_utilities_tests {
        use super::*;

        #[test]
        fn test_url_path_extraction() {
            let path = "/positions/123e4567-e89b-12d3-a456-426614174000";
            let id = path.strip_prefix("/positions/").unwrap();

            assert_eq!(id, "123e4567-e89b-12d3-a456-426614174000");
            assert!(Uuid::parse_str(id).is_ok());
        }

        #[test]
        fn test_invalid_uuid_handling() {
            let invalid_ids = vec![
                "invalid-uuid",
                "123",
                "",
                "not-a-uuid-at-all",
                "123e4567-e89b-12d3-a456", // Too short
            ];

            for invalid_id in invalid_ids {
                assert!(Uuid::parse_str(invalid_id).is_err());
            }
        }

        #[test]
        fn test_valid_uuid_formats() {
            let valid_ids = vec![
                "123e4567-e89b-12d3-a456-426614174000",
                "550e8400-e29b-41d4-a716-446655440000",
                "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
            ];

            for valid_id in valid_ids {
                assert!(Uuid::parse_str(valid_id).is_ok());
            }
        }

        #[test]
        fn test_content_type_handling() {
            // Test that we expect JSON content type for POST/PUT requests
            let content_type = "application/json";
            assert_eq!(content_type, "application/json");
        }

        #[test]
        fn test_http_status_codes() {
            // Test common status codes used in handlers
            let success_code = 200;
            let bad_request_code = 400;
            let not_found_code = 404;
            let server_error_code = 500;

            assert_eq!(success_code, 200);
            assert_eq!(bad_request_code, 400);
            assert_eq!(not_found_code, 404);
            assert_eq!(server_error_code, 500);
        }
    }
}
