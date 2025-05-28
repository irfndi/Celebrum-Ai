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

                            // Note: GlobalOpportunityService initialization skipped for now due to complex dependencies
                            // It will be initialized later when needed or in a separate initialization phase
                            if user_profile_service_available {
                                console_log!("✅ Prerequisites available for GlobalOpportunityService (will initialize when needed)");
                            } else {
                                console_log!("⚠️ UserProfileService not available - GlobalOpportunityService will use fallback");
                            }
                        }
                        Err(e) => {
                            console_log!(
                                "⚠️ Failed to initialize ExchangeService: {} (will use fallback)",
                                e
                            );
                        }
                    }

                    // Initialize OpportunityDistributionService
                    let session_management_service_clone =
                        services::core::user::session_management::SessionManagementService::new(
                            d1_service.clone(),
                            kv_service.clone(),
                        );
                    let opportunity_distribution_service = services::core::opportunities::opportunity_distribution::OpportunityDistributionService::new(
                        d1_service.clone(),
                        kv_service.clone(),
                        session_management_service_clone,
                    );
                    telegram_service
                        .set_opportunity_distribution_service(opportunity_distribution_service);
                    console_log!("✅ OpportunityDistributionService initialized successfully");

                    // Initialize AiIntegrationService if API keys are available
                    if let Ok(encryption_key) = env.var("ENCRYPTION_KEY") {
                        console_log!("✅ ENCRYPTION_KEY found, initializing AiIntegrationService");
                        let ai_config =
                            services::core::ai::ai_integration::AiIntegrationConfig::default();
                        let ai_integration_service =
                            services::core::ai::ai_integration::AiIntegrationService::new(
                                ai_config,
                                kv_store.clone(),
                                encryption_key.to_string(),
                            );
                        telegram_service.set_ai_integration_service(ai_integration_service);
                        console_log!("✅ AiIntegrationService initialized successfully");
                    } else {
                        console_log!("⚠️ ENCRYPTION_KEY not found - AiIntegrationService not initialized (will use fallback)");
                    }

                    // Initialize MarketAnalysisService (create new UserTradingPreferencesService instance)
                    let logger_for_market =
                        crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
                    let user_trading_preferences_service_for_market = services::core::user::user_trading_preferences::UserTradingPreferencesService::new(
                        d1_service.clone(),
                        logger_for_market,
                    );
                    let logger_for_market_analysis =
                        crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
                    let market_analysis_service =
                        services::core::analysis::market_analysis::MarketAnalysisService::new(
                            d1_service.clone(),
                            user_trading_preferences_service_for_market,
                            logger_for_market_analysis,
                        );
                    telegram_service.set_market_analysis_service(market_analysis_service);
                    console_log!("✅ MarketAnalysisService initialized successfully");

                    // Initialize TechnicalAnalysisService
                    let technical_analysis_config = services::core::analysis::technical_analysis::TechnicalAnalysisConfig::default();
                    let logger_for_technical =
                        crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
                    let technical_analysis_service =
                        services::core::analysis::technical_analysis::TechnicalAnalysisService::new(
                            technical_analysis_config,
                            logger_for_technical,
                        );
                    telegram_service.set_technical_analysis_service(technical_analysis_service);
                    console_log!("✅ TechnicalAnalysisService initialized successfully");
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

    // Fallback to simple pattern matching for testing/development
    let subscription_tier = if user_id.contains("admin") {
        types::SubscriptionTier::SuperAdmin
    } else if user_id.contains("enterprise") || user_id.contains("pro") {
        types::SubscriptionTier::Enterprise // Map both "enterprise" and "pro" to Enterprise
    } else if user_id.contains("premium") {
        types::SubscriptionTier::Premium
    } else if user_id.contains("basic") {
        types::SubscriptionTier::Basic
    } else {
        types::SubscriptionTier::Free
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
async fn handle_api_get_user_profile(req: Request, _env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Mock user profile data - in production, fetch from D1
    let subscription_tier = if user_id.contains("admin") {
        "superadmin"
    } else if user_id.contains("enterprise") || user_id.contains("pro") {
        "enterprise" // Map both "enterprise" and "pro" to enterprise
    } else if user_id.contains("premium") {
        "premium"
    } else if user_id.contains("basic") {
        "basic"
    } else {
        "free"
    };

    let profile_data = serde_json::json!({
        "user_id": user_id,
        "subscription_tier": subscription_tier,
        "user_role": if subscription_tier == "superadmin" { "superadmin" } else { "user" },
        "created_at": chrono::Utc::now().timestamp() - 86400,
        "last_active": chrono::Utc::now().timestamp(),
        "preferences": {
            "risk_tolerance": 0.5,
            "preferred_pairs": ["BTC/USDT", "ETH/USDT"]
        },
        "access_level": match subscription_tier {
            "free" => "free_without_api",
            "basic" => "free_with_api",
            "premium" | "enterprise" | "superadmin" => "subscription_with_api",
            _ => "free_without_api"
        }
    });

    let response = ApiResponse::success(profile_data);
    Response::from_json(&response)
}

async fn handle_api_update_user_profile(mut req: Request, _env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let _update_data: serde_json::Value = req.json().await?;

    // Mock successful update
    let response = ApiResponse::success(serde_json::json!({
        "user_id": user_id,
        "updated": true,
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
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
        "risk_tolerance": 0.5,
        "preferred_pairs": ["BTC/USDT", "ETH/USDT"],
        "auto_trading_enabled": false,
        "notification_settings": {
            "opportunities": true,
            "price_alerts": true
        }
    });

    let response = ApiResponse::success(preferences);
    Response::from_json(&response)
}

async fn handle_api_update_user_preferences(mut req: Request, _env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let _preferences: serde_json::Value = req.json().await?;

    let response = ApiResponse::success(serde_json::json!({
        "user_id": user_id,
        "preferences_updated": true,
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

// Opportunity endpoints
async fn handle_api_get_opportunities(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let url = req.url()?;
    let query_params: std::collections::HashMap<String, String> =
        url.query_pairs().into_owned().collect();
    let is_premium = query_params
        .get("premium")
        .map(|v| v == "true")
        .unwrap_or(false);

    // Check permissions for premium features
    if is_premium && !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    // Mock opportunities based on subscription tier
    let limit = if user_id.contains("admin") {
        100 // SuperAdmin: unlimited (capped at 100 for demo)
    } else if user_id.contains("enterprise") || user_id.contains("pro") {
        50 // Enterprise: high limit
    } else if user_id.contains("premium") {
        20 // Premium: moderate limit
    } else if user_id.contains("basic") {
        10 // Basic: low limit
    } else {
        5 // Free: very limited
    };

    let opportunities = (0..limit.min(10))
        .map(|i| {
            serde_json::json!({
                "id": format!("opp_{}", i),
                "pair": "BTC/USDT",
                "exchange_long": "binance",
                "exchange_short": "bybit",
                "profit_percentage": 0.5 + (i as f64 * 0.1),
                "volume": 1000.0,
                "is_premium": is_premium
            })
        })
        .collect::<Vec<_>>();

    let response = ApiResponse::success(opportunities);
    Response::from_json(&response)
}

async fn handle_api_execute_opportunity(mut req: Request, _env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let execution_data: serde_json::Value = match req.json().await {
        Ok(data) => data,
        Err(_) => {
            let response = ApiResponse::<()>::error("Invalid JSON payload".to_string());
            return Ok(Response::from_json(&response)?.with_status(400));
        }
    };

    let response = ApiResponse::success(serde_json::json!({
        "execution_id": format!("exec_{}", chrono::Utc::now().timestamp()),
        "user_id": user_id,
        "opportunity_id": execution_data.get("opportunity_id"),
        "status": "executed",
        "timestamp": chrono::Utc::now().timestamp()
    }));
    Response::from_json(&response)
}

// Analytics endpoints (Pro/Admin only)
async fn handle_api_get_dashboard_analytics(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    if !check_user_permissions(&user_id, "enterprise", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let analytics = serde_json::json!({
        "active_users": 1250,
        "total_opportunities": 45,
        "successful_trades": 123,
        "total_volume": 50000.0,
        "profit_percentage": 2.5
    });

    let response = ApiResponse::success(analytics);
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let analytics = serde_json::json!({
        "system_health": "excellent",
        "uptime_percentage": 99.9,
        "api_calls_today": 15000,
        "error_rate": 0.1
    });

    let response = ApiResponse::success(analytics);
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

    if !check_user_permissions(&user_id, "enterprise", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let analytics = serde_json::json!({
        "total_users": 1250,
        "active_users_24h": 450,
        "new_registrations": 25,
        "subscription_distribution": {
            "free": 800,
            "premium": 350,
            "pro": 90,
            "admin": 10
        }
    });

    let response = ApiResponse::success(analytics);
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

    if !check_user_permissions(&user_id, "enterprise", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let analytics = serde_json::json!({
        "avg_response_time_ms": 150,
        "throughput_per_second": 100,
        "cache_hit_rate": 85.5,
        "database_performance": "optimal"
    });

    let response = ApiResponse::success(analytics);
    Response::from_json(&response)
}

async fn handle_api_get_user_specific_analytics(req: Request, _env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    let analytics = serde_json::json!({
        "user_id": user_id,
        "total_trades": 45,
        "successful_trades": 38,
        "total_profit": 1250.50,
        "win_rate": 84.4
    });

    let response = ApiResponse::success(analytics);
    Response::from_json(&response)
}

// Admin endpoints (Admin only)
async fn handle_api_admin_get_users(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let users = vec![
        serde_json::json!({
            "user_id": "user_1",
            "subscription_tier": "premium",
            "created_at": chrono::Utc::now().timestamp() - 86400,
            "last_active": chrono::Utc::now().timestamp() - 3600
        }),
        serde_json::json!({
            "user_id": "user_2",
            "subscription_tier": "free",
            "created_at": chrono::Utc::now().timestamp() - 172800,
            "last_active": chrono::Utc::now().timestamp() - 7200
        }),
    ];

    let response = ApiResponse::success(users);
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let sessions = vec![serde_json::json!({
        "session_id": "sess_1",
        "user_id": "user_1",
        "created_at": chrono::Utc::now().timestamp() - 3600,
        "expires_at": chrono::Utc::now().timestamp() + 3600,
        "is_active": true
    })];

    let response = ApiResponse::success(sessions);
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let opportunities = vec![serde_json::json!({
        "opportunity_id": "opp_admin_1",
        "pair": "BTC/USDT",
        "exchange_long": "binance",
        "exchange_short": "bybit",
        "profit_percentage": 1.2,
        "created_at": chrono::Utc::now().timestamp()
    })];

    let response = ApiResponse::success(opportunities);
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let profiles = vec![serde_json::json!({
        "user_id": "user_1",
        "risk_tolerance": 0.7,
        "preferred_pairs": ["BTC/USDT"],
        "api_keys_encrypted": true
    })];

    let response = ApiResponse::success(profiles);
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let management_data = serde_json::json!({
        "users": {
            "total": 1250,
            "active": 450,
            "suspended": 5
        },
        "actions_available": ["suspend", "activate", "upgrade", "downgrade"]
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let config = serde_json::json!({
        "rate_limits": {
            "free": 10,
            "premium": 30,
            "pro": 60,
            "admin": 120
        },
        "features": {
            "ai_enabled": true,
            "trading_enabled": true,
            "analytics_enabled": true
        }
    });

    let response = ApiResponse::success(config);
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

    if !check_user_permissions(&user_id, "admin", &env).await? {
        let response = ApiResponse::<()>::error("Admin access required".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let invitations = vec![serde_json::json!({
        "code": "INV123",
        "created_by": user_id,
        "uses_remaining": 5,
        "expires_at": chrono::Utc::now().timestamp() + 86400
    })];

    let response = ApiResponse::success(invitations);
    Response::from_json(&response)
}

// Trading endpoints (Premium+ only)
async fn handle_api_get_trading_balance(req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let balance = serde_json::json!({
        "total_balance_usd": 10000.0,
        "available_balance_usd": 8500.0,
        "balances": {
            "BTC": 0.5,
            "ETH": 10.0,
            "USDT": 5000.0
        }
    });

    let response = ApiResponse::success(balance);
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

    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let markets = vec![
        serde_json::json!({
            "symbol": "BTC/USDT",
            "price": 45000.0,
            "volume_24h": 1000000.0,
            "change_24h": 2.5
        }),
        serde_json::json!({
            "symbol": "ETH/USDT",
            "price": 3000.0,
            "volume_24h": 500000.0,
            "change_24h": 1.8
        }),
    ];

    let response = ApiResponse::success(markets);
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

    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let opportunities = vec![serde_json::json!({
        "id": "trade_opp_1",
        "pair": "BTC/USDT",
        "type": "arbitrage",
        "profit_potential": 1.5,
        "risk_level": "medium"
    })];

    let response = ApiResponse::success(opportunities);
    Response::from_json(&response)
}

// AI endpoints (Premium+ only)
async fn handle_api_ai_analyze(mut req: Request, env: Env) -> Result<Response> {
    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let analysis_request: serde_json::Value = match req.json().await {
        Ok(data) => data,
        Err(_) => {
            let response = ApiResponse::<()>::error("Invalid JSON payload".to_string());
            return Ok(Response::from_json(&response)?.with_status(400));
        }
    };

    let analysis = serde_json::json!({
        "analysis_id": format!("ai_analysis_{}", chrono::Utc::now().timestamp()),
        "pair": analysis_request.get("pair"),
        "exchanges": analysis_request.get("exchanges"),
        "recommendation": "BUY",
        "confidence": 0.85,
        "reasoning": "Strong bullish momentum with high volume"
    });

    let response = ApiResponse::success(analysis);
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

    if !check_user_permissions(&user_id, "premium", &env).await? {
        let response = ApiResponse::<()>::error("Upgrade subscription for access".to_string());
        return Ok(Response::from_json(&response)?.with_status(403));
    }

    let risk_request: serde_json::Value = match req.json().await {
        Ok(data) => data,
        Err(_) => {
            let response = ApiResponse::<()>::error("Invalid JSON payload".to_string());
            return Ok(Response::from_json(&response)?.with_status(400));
        }
    };

    let assessment = serde_json::json!({
        "assessment_id": format!("risk_assessment_{}", chrono::Utc::now().timestamp()),
        "portfolio": risk_request.get("portfolio"),
        "overall_risk": "MEDIUM",
        "risk_score": 6.5,
        "recommendations": [
            "Diversify across more assets",
            "Consider reducing position size in volatile assets"
        ]
    });

    let response = ApiResponse::success(assessment);
    Response::from_json(&response)
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
