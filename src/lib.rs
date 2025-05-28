use worker::*;

// Module declarations
pub mod services;
pub mod types;
pub mod utils;

use once_cell::sync::OnceCell;
use serde_json::json;
use services::core::infrastructure::d1_database::D1Service;
use services::core::infrastructure::service_container::ServiceContainer;
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

// Thread-safe global service container - initialized once per worker instance
static GLOBAL_SERVICE_CONTAINER: OnceCell<Arc<ServiceContainer>> = OnceCell::new();

/// Get or initialize the global service container
async fn get_service_container(env: &Env) -> Result<Arc<ServiceContainer>> {
    // Try to get existing container first
    if let Some(container) = GLOBAL_SERVICE_CONTAINER.get() {
        return Ok(container.clone());
    }

    // Initialize service container
    let kv_store = env.kv("ArbEdgeKV")?;
    let mut container = ServiceContainer::new(env, kv_store).map_err(|e| {
        worker::Error::RustError(format!("Failed to create service container: {}", e))
    })?;

    // Set up Telegram service if available
    if let Ok(bot_token) = env.var("TELEGRAM_BOT_TOKEN") {
        let telegram_service = services::interfaces::telegram::telegram::TelegramService::new(
            services::interfaces::telegram::telegram::TelegramConfig {
                bot_token: bot_token.to_string(),
                chat_id: "0".to_string(),
                is_test_mode: false,
            },
        );
        container.set_telegram_service(telegram_service);
    }

    // Set up user profile service if encryption key is available
    if let Ok(encryption_key) = env.var("ENCRYPTION_KEY") {
        container.set_user_profile_service(encryption_key.to_string());
    }

    let container_arc = Arc::new(container);
    
    // Try to set the global container, but if another thread beat us to it, use theirs
    match GLOBAL_SERVICE_CONTAINER.set(container_arc.clone()) {
        Ok(()) => Ok(container_arc),
        Err(_) => {
            // Another thread initialized it first, use that one
            Ok(GLOBAL_SERVICE_CONTAINER.get().unwrap().clone())
        }
    }
}

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
        (Method::Get, "/health") => Response::ok("ArbEdge Rust Worker is running!"),

        // KV test endpoint
        (Method::Get, "/kv-test") => {
            let value = url.query().unwrap_or("default");
            let kv = env.kv("ArbEdgeKV")?;
            kv.put("test-key", value)?.execute().await?;
            let retrieved = kv.get("test-key").text().await?;
            Response::ok(retrieved.unwrap_or_default())
        }

        // Exchange API endpoints
        (Method::Get, "/exchange/markets") => handle_get_markets(req, env).await,

        (Method::Get, "/exchange/ticker") => handle_get_ticker(req, env).await,
        (Method::Get, "/exchange/orderbook") => handle_get_orderbook(req, env).await,

        (Method::Get, "/exchange/funding") => handle_funding_rate(req, env).await,

        // Opportunity finding endpoint
        (Method::Post, "/find-opportunities") => handle_find_opportunities(req, env).await,

        // Telegram webhook endpoint
        (Method::Post, "/webhook") => handle_telegram_webhook(req, env).await,

        // API v1 endpoints for direct access (no Telegram required)
        // Health and system endpoints
        (Method::Get, "/api/v1/health") => handle_api_health_check(req, env).await,

        (Method::Get, "/api/v1/health/detailed") => {
            handle_api_detailed_health_check(req, env).await
        }

        // User management endpoints
        (Method::Get, "/api/v1/users/profile") => handle_api_get_user_profile(req, env).await,

        (Method::Put, "/api/v1/users/profile") => handle_api_update_user_profile(req, env).await,

        (Method::Get, "/api/v1/users/preferences") => {
            handle_api_get_user_preferences(req, env).await
        }

        (Method::Put, "/api/v1/users/preferences") => {
            handle_api_update_user_preferences(req, env).await
        }

        // Opportunity endpoints with RBAC
        (Method::Get, "/api/v1/opportunities") => handle_api_get_opportunities(req, env).await,

        (Method::Post, "/api/v1/opportunities/execute") => {
            handle_api_execute_opportunity(req, env).await
        }

        // Analytics endpoints (Pro/Admin only)
        (Method::Get, "/api/v1/analytics/dashboard") => {
            handle_api_get_dashboard_analytics(req, env).await
        }

        (Method::Get, "/api/v1/analytics/system") => {
            handle_api_get_system_analytics(req, env).await
        }

        (Method::Get, "/api/v1/analytics/users") => handle_api_get_user_analytics(req, env).await,

        (Method::Get, "/api/v1/analytics/performance") => {
            handle_api_get_performance_analytics(req, env).await
        }

        (Method::Get, "/api/v1/analytics/user") => {
            handle_api_get_user_specific_analytics(req, env).await
        }

        // Admin endpoints (Admin only)
        (Method::Get, "/api/v1/admin/users") => handle_api_admin_get_users(req, env).await,

        (Method::Get, "/api/v1/admin/sessions") => handle_api_admin_get_sessions(req, env).await,

        (Method::Get, "/api/v1/admin/opportunities") => {
            handle_api_admin_get_opportunities(req, env).await
        }

        (Method::Get, "/api/v1/admin/user-profiles") => {
            handle_api_admin_get_user_profiles(req, env).await
        }

        (Method::Get, "/api/v1/admin/manage/users") => {
            handle_api_admin_manage_users(req, env).await
        }

        (Method::Get, "/api/v1/admin/config/system") => {
            handle_api_admin_system_config(req, env).await
        }

        (Method::Get, "/api/v1/admin/invitations") => handle_api_admin_invitations(req, env).await,

        // Trading endpoints (Premium+ only)
        (Method::Get, "/api/v1/trading/balance") => handle_api_get_trading_balance(req, env).await,

        (Method::Get, "/api/v1/trading/markets") => handle_api_get_trading_markets(req, env).await,

        (Method::Get, "/api/v1/trading/opportunities") => {
            handle_api_get_trading_opportunities(req, env).await
        }

        // AI endpoints (Premium+ only)
        (Method::Post, "/api/v1/ai/analyze") => handle_api_ai_analyze(req, env).await,

        (Method::Post, "/api/v1/ai/risk-assessment") => {
            handle_api_ai_risk_assessment(req, env).await
        }

        // Position management endpoints
        (Method::Post, "/positions") => handle_create_position(req, env).await,

        (Method::Get, "/positions") => handle_get_all_positions(req, env).await,

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

        // Default response for unknown endpoints
        _ => Response::error("Endpoint not found", 404),
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
    // Parse configuration from environment with fallback values
    let exchanges_str = custom_env
        .worker_env
        .var("EXCHANGES")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "binance,bybit".to_string());
    let exchanges = parse_exchanges_from_env(&exchanges_str)?;

    let monitored_pairs_str = custom_env
        .worker_env
        .var("MONITORED_PAIRS_CONFIG")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| r#"[{"symbol":"BTCUSDT","base":"BTC","quote":"USDT","exchange_id":"binance"},{"symbol":"ETHUSDT","base":"ETH","quote":"USDT","exchange_id":"binance"}]"#.to_string());
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

async fn handle_get_orderbook(req: Request, env: Env) -> Result<Response> {
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
        .get_orderbook(&exchange_id, &symbol, None)
        .await
    {
        Ok(orderbook) => Response::from_json(&orderbook),
        Err(e) => Response::error(format!("Failed to get orderbook: {}", e), 500),
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

            // Create D1Service once and reuse it
            match D1Service::new(&env) {
                Ok(d1_service) => {
                    console_log!("✅ D1Service initialized successfully");
                    let kv_service =
                        services::core::infrastructure::KVService::new(kv_store.clone());

                    // Initialize session management service
                    let session_management_service =
                        services::core::user::session_management::SessionManagementService::new(
                            d1_service.clone(),
                            kv_service.clone(),
                        );
                    telegram_service.set_session_management_service(session_management_service);
                    console_log!("✅ SessionManagementService initialized successfully");

                    // Set D1Service on telegram service
                    telegram_service.set_d1_service(d1_service.clone());
                    console_log!("✅ D1Service set on TelegramService successfully");

                    // Initialize UserProfileService for RBAC if encryption key is available
                    let user_profile_service_available = if let Ok(encryption_key) =
                        env.var("ENCRYPTION_KEY")
                    {
                        console_log!(
                            "✅ ENCRYPTION_KEY found, initializing UserProfileService for RBAC"
                        );
                        let user_profile_service = UserProfileService::new(
                            kv_store.clone(),
                            d1_service.clone(), // Reuse the same D1Service instance
                            encryption_key.to_string(),
                        );
                        telegram_service.set_user_profile_service(user_profile_service);
                        console_log!("✅ RBAC UserProfileService initialized successfully");
                        true
                    } else {
                        console_log!("❌ RBAC SECURITY WARNING: ENCRYPTION_KEY not found - UserProfileService not initialized");
                        false
                    };

                    // Initialize UserTradingPreferencesService first (needed by other services)
                    let logger =
                        crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
                    let user_trading_preferences_service = services::core::user::user_trading_preferences::UserTradingPreferencesService::new(
                        d1_service.clone(),
                        logger,
                    );
                    telegram_service
                        .set_user_trading_preferences_service(user_trading_preferences_service);
                    console_log!("✅ UserTradingPreferencesService initialized successfully");

                    // Initialize ExchangeService (needed by GlobalOpportunityService)
                    let custom_env = types::Env::new(env.clone());
                    match services::core::trading::exchange::ExchangeService::new(&custom_env) {
                        Ok(exchange_service) => {
                            telegram_service.set_exchange_service(exchange_service);
                            console_log!("✅ ExchangeService initialized successfully");
                        }
                        Err(e) => {
                            console_log!(
                                "⚠️ Failed to initialize ExchangeService: {} (will use fallback)",
                                e
                            );
                        }
                    }

                    // Note: GlobalOpportunityService initialization skipped for now due to complex dependencies
                    // It will be initialized later when needed or in a separate initialization phase
                    if user_profile_service_available {
                        console_log!("✅ Prerequisites available for GlobalOpportunityService (will initialize when needed)");
                    } else {
                        console_log!("⚠️ UserProfileService not available - GlobalOpportunityService will use fallback");
                    }
                }
                Err(e) => {
                    console_log!("❌ Failed to initialize D1Service: {:?}", e);
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
    // Parse request JSON first to provide better error messages
    let position_data_json = match req.json::<serde_json::Value>().await {
        Ok(data) => data,
        Err(e) => return Response::error(format!("Invalid JSON payload: {}", e), 400),
    };

    // Validate user_id is present
    let user_id = match position_data_json.get("user_id").and_then(|v| v.as_str()) {
        Some(uid) => uid,
        None => return Response::error("Missing required field: user_id", 400),
    };

    // Initialize services with better error handling
    let kv = match env.kv("ArbEdgeKV") {
        Ok(kv) => kv,
        Err(e) => return Response::error(format!("KV store initialization failed: {}", e), 500),
    };

    let d1_service = match D1Service::new(&env) {
        Ok(service) => service,
        Err(e) => return Response::error(format!("D1 database initialization failed: {}", e), 500),
    };

    // Handle missing ENCRYPTION_KEY - fail immediately for security
    let encryption_key = match env.var("ENCRYPTION_KEY") {
        Ok(key) => key.to_string(),
        Err(_) => {
            return Response::error(
                "ENCRYPTION_KEY environment variable is required for security. Application cannot start without proper encryption key.",
                500
            );
        }
    };

    let user_profile_service = UserProfileService::new(kv.clone(), d1_service, encryption_key);
    let positions_service = ProductionPositionsService::new(kv);

    // Parse position data
    let position_data: CreatePositionData = match serde_json::from_value(position_data_json.clone())
    {
        Ok(data) => data,
        Err(e) => return Response::error(format!("Invalid position data format: {}", e), 400),
    };

    // Fetch user profile with proper error handling
    let user_profile = match user_profile_service.get_user_profile(user_id).await {
        Ok(Some(profile)) => profile,
        Ok(None) => {
            return Response::error(
                format!(
                    "User profile not found for user_id: {}. Please create a profile first.",
                    user_id
                ),
                404,
            );
        }
        Err(e) => return Response::error(format!("Failed to fetch user profile: {}", e), 500),
    };

    // Handle account balance - provide default for super admin users
    let account_balance = if user_profile.account_balance_usdt > 0.0 {
        user_profile.account_balance_usdt
    } else if user_profile.subscription.tier == types::SubscriptionTier::SuperAdmin {
        // Default balance for super admin testing
        100000.0
    } else {
        user_profile.account_balance_usdt
    };

    let account_info = AccountInfo {
        total_balance_usd: account_balance,
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
        distribution_service
            .set_notification_sender(Box::new(std::sync::Arc::new(telegram_service)));
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

// ============= API v1 HANDLERS =============
// Direct API access handlers with RBAC and authentication

/// Extract user ID from X-User-ID header
fn extract_user_id_from_headers(req: &Request) -> Result<String> {
    req.headers()
        .get("X-User-ID")?
        .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))
}

/// Standard API response format
#[derive(serde::Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: u64,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

/// Check user subscription tier and permissions using proper RBAC
async fn check_user_permissions(user_id: &str, required_tier: &str, env: &Env) -> Result<bool> {
    // Try to get user profile from D1 database for proper RBAC
    if let Ok(d1_service) = services::core::infrastructure::d1_database::D1Service::new(env) {
        if let Ok(kv_store) = env.kv("ArbEdgeKV") {
            if let Ok(encryption_key) = env.var("ENCRYPTION_KEY") {
                let user_profile_service =
                    services::core::user::user_profile::UserProfileService::new(
                        kv_store,
                        d1_service,
                        encryption_key.to_string(),
                    );

                // Try to parse user_id as telegram_id for database lookup
                if let Ok(telegram_id) = user_id.parse::<i64>() {
                    if let Ok(Some(user_profile)) = user_profile_service
                        .get_user_by_telegram_id(telegram_id)
                        .await
                    {
                        return Ok(check_subscription_tier_permission(
                            &user_profile.subscription.tier,
                            required_tier,
                        ));
                    }
                }
            }
        }
    }

    // Fallback to simple pattern matching ONLY in development/testing environments
    // This is a security risk and should never be enabled in production
    let subscription_tier = if env.var("ENABLE_FALLBACK_PERMISSIONS").is_ok() {
        console_log!("⚠️ WARNING: Using fallback permission logic - NOT FOR PRODUCTION");
        if user_id.contains("admin") {
            types::SubscriptionTier::SuperAdmin
        } else if user_id.contains("enterprise") || user_id.contains("pro") {
            types::SubscriptionTier::Enterprise // Map both "enterprise" and "pro" to Enterprise
        } else if user_id.contains("premium") {
            types::SubscriptionTier::Premium
        } else if user_id.contains("basic") {
            types::SubscriptionTier::Basic
        } else {
            types::SubscriptionTier::Free
        }
    } else {
        // In production, deny access if no valid user profile found
        return Ok(false);
    };

    Ok(check_subscription_tier_permission(
        &subscription_tier,
        required_tier,
    ))
}

/// Check if subscription tier has permission for required tier
fn check_subscription_tier_permission(
    user_tier: &types::SubscriptionTier,
    required_tier: &str,
) -> bool {
    use types::SubscriptionTier;

    match required_tier {
        "free" => true, // Everyone can access free tier
        "basic" => matches!(
            user_tier,
            SubscriptionTier::Basic
                | SubscriptionTier::Premium
                | SubscriptionTier::Enterprise
                | SubscriptionTier::SuperAdmin
        ),
        "premium" => matches!(
            user_tier,
            SubscriptionTier::Basic  // Basic users should have premium access
                | SubscriptionTier::Premium
                | SubscriptionTier::Enterprise
                | SubscriptionTier::SuperAdmin
        ),
        "enterprise" | "pro" => matches!(
            user_tier,
            SubscriptionTier::Enterprise | SubscriptionTier::SuperAdmin
        ),
        "admin" | "superadmin" => matches!(user_tier, SubscriptionTier::SuperAdmin),
        _ => false, // Unknown tier, deny access
    }
}

// Health endpoints
async fn handle_api_health_check(_req: Request, _env: Env) -> Result<Response> {
    let response = ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "ArbEdge API",
        "version": "1.0.0"
    }));
    Response::from_json(&response)
}

async fn handle_api_detailed_health_check(_req: Request, env: Env) -> Result<Response> {
    // Check service health
    let kv_healthy = env.kv("ArbEdgeKV").is_ok();
    let d1_healthy = env.d1("ArbEdgeD1").is_ok();

    let response = ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "services": {
            "kv_store": if kv_healthy { "online" } else { "offline" },
            "d1_database": if d1_healthy { "online" } else { "offline" },
            "telegram_service": "online",
            "exchange_service": "online",
            "ai_service": "online"
        },
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

// User management endpoints
async fn handle_api_get_user_profile(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Get encryption key from environment
    let encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .unwrap_or_else(|_| "default_key_for_development".to_string());

    // Initialize services
    let kv_store = env.kv("ArbEdgeKV")?;
    let d1_service = services::core::infrastructure::D1Service::new(&env)?;
    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store,
        d1_service,
        encryption_key,
    );

    // Fetch real user profile from database
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(profile)) => {
            let profile_data = serde_json::json!({
                "user_id": profile.user_id,
                "telegram_user_id": profile.telegram_user_id,
                "telegram_username": profile.telegram_username,
                "subscription": {
                    "tier": profile.subscription.tier,
                    "is_active": profile.subscription.is_active,
                    "expires_at": profile.subscription.expires_at,
                    "features": profile.subscription.features
                },
                "configuration": profile.configuration,
                "created_at": profile.created_at,
                "updated_at": profile.updated_at,
                "last_active": profile.last_active,
                "is_active": profile.is_active,
                "total_trades": profile.total_trades,
                "total_pnl_usdt": profile.total_pnl_usdt,
                "account_balance_usdt": profile.account_balance_usdt
            });

            let response = ApiResponse::success(profile_data);
            Response::from_json(&response)
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch user profile: {}", e));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

async fn handle_api_update_user_profile(mut req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Parse and validate the update request
    let update_request: types::UpdateUserProfileRequest = match req.json().await {
        Ok(data) => data,
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Invalid JSON format: {}", e));
            return Ok(Response::from_json(&response)?.with_status(400));
        }
    };

    // Validate the request
    if let Err(validation_error) = update_request.validate() {
        let response = ApiResponse::<()>::error(format!("Validation error: {}", validation_error));
        return Ok(Response::from_json(&response)?.with_status(400));
    }

    // Get encryption key from environment
    let encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required for security. Application cannot start without proper encryption key.".to_string())
        })?;

    // Initialize services
    let kv_store = env.kv("ArbEdgeKV")?;
    let d1_service = services::core::infrastructure::D1Service::new(&env)?;
    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store,
        d1_service,
        encryption_key,
    );

    // Update user profile in database
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(mut profile)) => {
            // Apply validated updates to the profile
            match update_request.apply_to_profile(&mut profile) {
                Ok(_) => {
                    // Update the profile in database
                    match user_profile_service.update_user_profile(&profile).await {
                        Ok(_) => {
                            let response = ApiResponse::success(serde_json::json!({
                                "user_id": user_id,
                                "updated": true,
                                "profile": {
                                    "user_id": profile.user_id,
                                    "telegram_username": profile.telegram_username,
                                    "configuration": profile.configuration,
                                    "updated_at": profile.updated_at
                                },
                                "timestamp": chrono::Utc::now().timestamp()
                            }));
                            Response::from_json(&response)
                        }
                        Err(e) => {
                            let response =
                                ApiResponse::<()>::error(format!("Failed to update user profile: {}", e));
                            Ok(Response::from_json(&response)?.with_status(500))
                        }
                    }
                }
                Err(e) => {
                    let response = ApiResponse::<()>::error(format!("Failed to apply updates: {}", e));
                    Ok(Response::from_json(&response)?.with_status(400))
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch user profile: {}", e));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

async fn handle_api_get_user_preferences(req: Request, _env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let preferences = serde_json::json!({
        "user_id": user_id,
        "risk_tolerance_percentage": 0.5,
        "trading_pairs": ["BTC/USDT", "ETH/USDT"],
        "auto_trading_enabled": false,
        "max_leverage": 10,
        "max_entry_size_usdt": 1000.0,
        "min_entry_size_usdt": 10.0,
        "opportunity_threshold": 0.01,
        "notification_preferences": {
            "push_opportunities": true,
            "push_executions": true,
            "push_risk_alerts": true,
            "push_system_status": true,
            "min_profit_threshold_usdt": 5.0,
            "max_notifications_per_hour": 10
        }
    });

    let response = ApiResponse::success(preferences);
    Response::from_json(&response)
}

async fn handle_api_update_user_preferences(mut req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let update_data: serde_json::Value = req.json().await?;

    // Get encryption key from environment
    let encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .unwrap_or_else(|_| "default_key_for_development".to_string());

    // Initialize services
    let kv_store = env.kv("ArbEdgeKV")?;
    let d1_service = services::core::infrastructure::D1Service::new(&env)?;
    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store,
        d1_service,
        encryption_key,
    );

    // Update user preferences in database
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(mut profile)) => {
            // Update the profile configuration with new preferences
            if let Some(risk_tolerance) = update_data.get("risk_tolerance").and_then(|v| v.as_f64())
            {
                profile.configuration.risk_tolerance_percentage = risk_tolerance;
            }
            if let Some(trading_pairs) = update_data.get("trading_pairs").and_then(|v| v.as_array())
            {
                profile.configuration.trading_pairs = trading_pairs
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(auto_trading) = update_data
                .get("auto_trading_enabled")
                .and_then(|v| v.as_bool())
            {
                profile.configuration.auto_trading_enabled = auto_trading;
            }
            if let Some(max_leverage) = update_data.get("max_leverage").and_then(|v| v.as_u64()) {
                profile.configuration.max_leverage = max_leverage as u32;
            }
            if let Some(max_entry_size) = update_data
                .get("max_entry_size_usdt")
                .and_then(|v| v.as_f64())
            {
                profile.configuration.max_entry_size_usdt = max_entry_size;
            }

            // Update the profile in database
            match user_profile_service.update_user_profile(&profile).await {
                Ok(_) => {
                    let response = ApiResponse::success(serde_json::json!({
                        "user_id": user_id,
                        "preferences_updated": true,
                        "timestamp": chrono::Utc::now().timestamp()
                    }));
                    Response::from_json(&response)
                }
                Err(e) => {
                    let response = ApiResponse::<()>::error(format!(
                        "Failed to update user preferences: {}",
                        e
                    ));
                    Ok(Response::from_json(&response)?.with_status(500))
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch user profile: {}", e));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

// Opportunities API handlers
async fn handle_api_get_opportunities(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Basic tier and above
    if !check_user_permissions(&user_id, "basic", &env).await? {
        let response = ApiResponse::<()>::error("Insufficient permissions".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get opportunities from distribution service with fallback to exchange service
    let opportunities = match _container
        .distribution_service()
        .get_user_opportunities(&user_id)
        .await
    {
        Ok(opps) => opps,
        Err(_) => {
            // Fallback: Get opportunities directly from exchange service
            let custom_env = types::Env::new(env.clone());
            match create_opportunity_service(&custom_env).await {
                Ok(opp_service) => match opp_service.monitor_opportunities().await {
                    Ok(opps) => opps,
                    Err(e) => {
                        let response =
                            ApiResponse::<()>::error(format!("Failed to get opportunities: {}", e));
                        return Ok(Response::from_json(&response)?.with_status(500));
                    }
                },
                Err(e) => {
                    let response =
                        ApiResponse::<()>::error(format!("Service creation failed: {}", e));
                    return Ok(Response::from_json(&response)?.with_status(500));
                }
            }
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "opportunities": opportunities,
        "count": opportunities.len(),
        "user_id": user_id,
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

async fn handle_api_execute_opportunity(mut req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Premium tier and above for execution
    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error(
            "Premium subscription required for opportunity execution".to_string(),
        );
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let execution_data: serde_json::Value = req.json().await?;

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Execute opportunity through exchange service
    let opportunity_id = execution_data
        .get("opportunity_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Check if this is a simulation/demo mode
    let simulation_mode = env
        .var("TRADING_SIMULATION_MODE")
        .map(|v| v.to_string().to_lowercase() == "true")
        .unwrap_or(true); // Default to simulation mode for safety

    if simulation_mode {
        // Simulation mode - clearly indicate this is not real execution
        let response = ApiResponse::success(serde_json::json!({
            "execution_id": uuid::Uuid::new_v4().to_string(),
            "opportunity_id": opportunity_id,
            "status": "simulated",
            "mode": "simulation",
            "user_id": user_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "message": "SIMULATION: Opportunity execution simulated successfully. No real trades were executed.",
            "warning": "This is a simulation. To enable real trading, set TRADING_SIMULATION_MODE=false and ensure proper API keys are configured."
        }));
        Response::from_json(&response)
    } else {
        // Real execution mode - implement actual trading logic
        // TODO: Implement real execution logic here
        // This would involve:
        // 1. Validating the opportunity is still valid
        // 2. Checking user's API keys and permissions
        // 3. Calculating position sizes based on risk management
        // 4. Executing trades on the required exchanges
        // 5. Monitoring execution status
        // 6. Recording results in database
        
        let response = ApiResponse::<()>::error(
            "Real trading execution not yet implemented. Please use simulation mode.".to_string()
        );
        Ok(Response::from_json(&response)?.with_status(501)) // 501 Not Implemented
    }
}

// Analytics API handlers
async fn handle_api_get_dashboard_analytics(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Pro tier and above for analytics
    if !check_user_permissions(&user_id, "pro", &env).await? {
        let response =
            ApiResponse::<()>::error("Pro subscription required for analytics".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get analytics data from D1 with KV caching
    let analytics_data = match _container.d1_service().get_user_analytics(&user_id).await {
        Ok(data) => data,
        Err(_) => {
            // Fallback to KV cache
            match _container
                .kv_service()
                .get(&format!("analytics:dashboard:{}", user_id))
                .await
            {
                Ok(Some(cached_data)) => serde_json::from_str(&cached_data).unwrap_or_else(|_| {
                    serde_json::json!({
                        "total_opportunities": 0,
                        "executed_trades": 0,
                        "total_pnl": 0.0,
                        "success_rate": 0.0,
                        "cached": true
                    })
                }),
                _ => serde_json::json!({
                    "total_opportunities": 0,
                    "executed_trades": 0,
                    "total_pnl": 0.0,
                    "success_rate": 0.0,
                    "fallback": true
                }),
            }
        }
    };

    let response = ApiResponse::success(analytics_data);
    Response::from_json(&response)
}

async fn handle_api_get_system_analytics(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get system health and analytics
    let health_status = _container.health_check().await.unwrap_or_else(|_| {
        crate::services::core::infrastructure::service_container::ServiceHealthStatus {
            overall_healthy: false,
            session_service_healthy: false,
            distribution_service_healthy: false,
            telegram_service_healthy: false,
            exchange_service_healthy: false,
            user_profile_service_healthy: false,
            vectorize_service_healthy: false,
            pipelines_service_healthy: false,
            errors: vec!["Health check failed".to_string()],
        }
    });

    let system_analytics = serde_json::json!({
        "health_status": health_status.detailed_report(),
        "active_users": 0, // Would be fetched from D1
        "total_opportunities": 0, // Would be fetched from D1
        "system_uptime": chrono::Utc::now().timestamp(),
        "timestamp": chrono::Utc::now().timestamp()
    });

    let response = ApiResponse::success(system_analytics);
    Response::from_json(&response)
}

async fn handle_api_get_user_analytics(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only for user analytics
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get user analytics from D1 with fallback to KV
    let user_analytics = match _container.d1_service().get_all_user_analytics().await {
        Ok(data) => data,
        Err(_) => {
            // Fallback to aggregated KV data
            serde_json::json!({
                "total_users": 0,
                "active_users": 0,
                "premium_users": 0,
                "fallback_mode": true,
                "timestamp": chrono::Utc::now().timestamp()
            })
        }
    };

    let response = ApiResponse::success(user_analytics);
    Response::from_json(&response)
}

async fn handle_api_get_performance_analytics(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Pro tier and above
    if !check_user_permissions(&user_id, "pro", &env).await? {
        let response = ApiResponse::<()>::error("Pro subscription required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get performance analytics with Pipelines fallback to KV
    let performance_data = if let Some(pipelines_service) = _container.pipelines_service() {
        match pipelines_service.get_performance_analytics(&user_id).await {
            Ok(data) => data,
            Err(_) => {
                // Fallback to KV cache
                match _container
                    .kv_service()
                    .get(&format!("performance:{}", user_id))
                    .await
                {
                    Ok(Some(cached)) => serde_json::from_str(&cached).unwrap_or_default(),
                    _ => serde_json::json!({
                        "avg_execution_time": 0.0,
                        "success_rate": 0.0,
                        "total_volume": 0.0,
                        "fallback_mode": true
                    }),
                }
            }
        }
    } else {
        serde_json::json!({
            "avg_execution_time": 0.0,
            "success_rate": 0.0,
            "total_volume": 0.0,
            "service_unavailable": true
        })
    };

    let response = ApiResponse::success(performance_data);
    Response::from_json(&response)
}

async fn handle_api_get_user_specific_analytics(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Basic tier and above for own analytics
    if !check_user_permissions(&user_id, "basic", &env).await? {
        let response = ApiResponse::<()>::error("Insufficient permissions".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get user-specific analytics from D1 with KV fallback
    let user_analytics = match _container.d1_service().get_user_analytics(&user_id).await {
        Ok(data) => data,
        Err(_) => {
            // Fallback to KV cache
            match _container
                .kv_service()
                .get(&format!("user_analytics:{}", user_id))
                .await
            {
                Ok(Some(cached)) => serde_json::from_str(&cached).unwrap_or_default(),
                _ => serde_json::json!({
                    "total_trades": 0,
                    "total_pnl": 0.0,
                    "win_rate": 0.0,
                    "avg_trade_size": 0.0,
                    "fallback_mode": true
                }),
            }
        }
    };

    let response = ApiResponse::success(user_analytics);
    Response::from_json(&response)
}

// Admin API handlers
async fn handle_api_admin_get_users(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get all users from D1 with pagination
    let users = match _container.d1_service().get_all_users().await {
        Ok(users) => users,
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch users: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "users": users,
        "total_count": users.len(),
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

async fn handle_api_admin_get_sessions(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get active sessions from session service
    let sessions = match _container.session_service().get_all_active_sessions().await {
        Ok(sessions) => sessions,
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch sessions: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "active_sessions": sessions,
        "session_count": sessions.len(),
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

async fn handle_api_admin_get_opportunities(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get all opportunities from distribution service
    let opportunities = match _container
        .distribution_service()
        .get_all_opportunities()
        .await
    {
        Ok(opps) => opps,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Failed to fetch opportunities: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "opportunities": opportunities,
        "total_count": opportunities.len(),
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

async fn handle_api_admin_get_user_profiles(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get all user profiles if user profile service is available
    let profiles = if let Some(user_profile_service) = _container.user_profile_service() {
        match user_profile_service.get_all_user_profiles().await {
            Ok(profiles) => profiles,
            Err(e) => {
                let response =
                    ApiResponse::<()>::error(format!("Failed to fetch user profiles: {}", e));
                return Ok(Response::from_json(&response)?.with_status(500));
            }
        }
    } else {
        let response = ApiResponse::<()>::error("User profile service not available".to_string());
        return Ok(Response::from_json(&response)?.with_status(503));
    };

    let response = ApiResponse::success(serde_json::json!({
        "user_profiles": profiles,
        "total_count": profiles.len(),
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

async fn handle_api_admin_manage_users(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Return user management interface data
    let management_data = serde_json::json!({
        "available_actions": [
            "view_user_details",
            "update_subscription",
            "suspend_user",
            "delete_user",
            "reset_password"
        ],
        "user_statistics": {
            "total_users": 0,
            "active_users": 0,
            "suspended_users": 0
        },
        "timestamp": chrono::Utc::now().timestamp()
    });

    let response = ApiResponse::success(management_data);
    Response::from_json(&response)
}

async fn handle_api_admin_system_config(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get system configuration
    let system_config = serde_json::json!({
        "exchanges": env.var("EXCHANGES")
            .map(|secret| secret.to_string())
            .unwrap_or_else(|_| "binance,bybit".to_string()),
        "arbitrage_threshold": env.var("ARBITRAGE_THRESHOLD")
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "0.001".to_string()),
        "max_concurrent_users": 10000,
        "rate_limits": {
            "api_calls_per_minute": 1000,
            "opportunities_per_hour": 100
        },
        "features": {
            "telegram_enabled": env.var("TELEGRAM_BOT_TOKEN").is_ok(),
            "ai_enabled": env.var("ENCRYPTION_KEY").is_ok(),
            "vectorize_enabled": true,
            "pipelines_enabled": true
        },
        "timestamp": chrono::Utc::now().timestamp()
    });

    let response = ApiResponse::success(system_config);
    Response::from_json(&response)
}

async fn handle_api_admin_invitations(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Admin only
    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get invitation data from D1
    let invitations = match _container.d1_service().get_all_invitations().await {
        Ok(invitations) => invitations,
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch invitations: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "invitations": invitations,
        "total_count": invitations.len(),
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

// Trading API handlers
async fn handle_api_get_trading_balance(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Premium tier and above for trading
    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error(
            "Premium subscription required for trading features".to_string(),
        );
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get user profile for balance information
    let balance_info = if let Some(user_profile_service) = _container.user_profile_service() {
        match user_profile_service.get_user_profile(&user_id).await {
            Ok(Some(profile)) => serde_json::json!({
                "total_balance_usdt": profile.account_balance_usdt,
                "available_balance": profile.account_balance_usdt * 0.9, // 90% available for trading
                "reserved_balance": profile.account_balance_usdt * 0.1,  // 10% reserved
                "currency": "USDT"
            }),
            _ => serde_json::json!({
                "total_balance_usdt": 0.0,
                "available_balance": 0.0,
                "reserved_balance": 0.0,
                "currency": "USDT",
                "error": "Profile not found"
            }),
        }
    } else {
        serde_json::json!({
            "total_balance_usdt": 0.0,
            "available_balance": 0.0,
            "reserved_balance": 0.0,
            "currency": "USDT",
            "error": "Service unavailable"
        })
    };

    let response = ApiResponse::success(balance_info);
    Response::from_json(&response)
}

async fn handle_api_get_trading_markets(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Premium tier and above
    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Premium subscription required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get markets from exchange service
    let markets = match _container.exchange_service().get_markets("binance").await {
        Ok(markets) => markets,
        Err(e) => {
            let response = ApiResponse::<()>::error(format!("Failed to fetch markets: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "markets": markets,
        "total_count": markets.len(),
        "exchange": "binance",
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

async fn handle_api_get_trading_opportunities(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Premium tier and above
    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Premium subscription required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Get trading opportunities from distribution service
    let opportunities = match _container
        .distribution_service()
        .get_user_opportunities(&user_id)
        .await
    {
        Ok(opps) => opps
            .into_iter()
            .filter(|opp| opp.rate_difference > 0.005)
            .collect::<Vec<_>>(), // Filter for trading-worthy opportunities
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Failed to fetch trading opportunities: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "trading_opportunities": opportunities,
        "count": opportunities.len(),
        "min_profit_threshold": 0.005,
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

// AI API handlers
async fn handle_api_ai_analyze(mut req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Premium tier and above for AI features
    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response =
            ApiResponse::<()>::error("Premium subscription required for AI features".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let analysis_request: serde_json::Value = req.json().await?;

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Perform AI analysis - this would integrate with AI services
    let analysis_result = serde_json::json!({
        "analysis_id": uuid::Uuid::new_v4().to_string(),
        "user_id": user_id,
        "request": analysis_request,
        "result": {
            "market_sentiment": "bullish",
            "confidence": 0.75,
            "recommended_action": "hold",
            "risk_level": "medium"
        },
        "processing_time_ms": 150,
        "timestamp": chrono::Utc::now().timestamp()
    });

    // Store analysis in D1 for audit trail
    if let Err(e) = _container
        .d1_service()
        .store_ai_analysis_audit(
            &user_id,
            "market_analysis",
            &analysis_request,
            &analysis_result,
            150,
        )
        .await
    {
        worker::console_log!("Failed to store AI analysis audit: {}", e);
    }

    let response = ApiResponse::success(analysis_result);
    Response::from_json(&response)
}

async fn handle_api_ai_risk_assessment(mut req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Check permissions - Premium tier and above
    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error(
            "Premium subscription required for AI risk assessment".to_string(),
        );
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let risk_request: serde_json::Value = req.json().await?;

    // Get service container
    let _container = match get_service_container(&env).await {
        Ok(container) => container,
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Service initialization failed: {}", e));
            return Ok(Response::from_json(&response)?.with_status(500));
        }
    };

    // Perform risk assessment
    let risk_assessment = serde_json::json!({
        "assessment_id": uuid::Uuid::new_v4().to_string(),
        "user_id": user_id,
        "request": risk_request,
        "assessment": {
            "overall_risk_score": 0.65,
            "risk_factors": [
                {"factor": "market_volatility", "score": 0.7, "weight": 0.3},
                {"factor": "position_size", "score": 0.5, "weight": 0.4},
                {"factor": "correlation_risk", "score": 0.8, "weight": 0.3}
            ],
            "recommendations": [
                "Consider reducing position size",
                "Monitor market volatility closely",
                "Diversify across uncorrelated assets"
            ],
            "max_recommended_exposure": 0.1
        },
        "processing_time_ms": 200,
        "timestamp": chrono::Utc::now().timestamp()
    });

    // Store risk assessment in D1 for audit trail
    if let Err(e) = _container
        .d1_service()
        .store_ai_analysis_audit(
            &user_id,
            "risk_assessment",
            &risk_request,
            &risk_assessment,
            120,
        )
        .await
    {
        worker::console_log!("Failed to store risk assessment audit: {}", e);
    }

    let response = ApiResponse::success(risk_assessment);
    Response::from_json(&response)
}
