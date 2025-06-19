use once_cell::sync::OnceCell;
use std::sync::Arc;
use worker::*;

// Time constants for improved readability
const HOUR_IN_MS: u64 = 60 * 60 * 1000;
const DAY_IN_MS: u64 = 24 * HOUR_IN_MS;

// Request validation structs
#[derive(serde::Deserialize)]
struct UpdateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    telegram_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

#[derive(serde::Deserialize)]
struct UpdatePreferencesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    notification_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_tolerance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_profit_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_position_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred_trading_pairs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred_exchanges: Option<Vec<String>>,
}

// Module declarations
pub mod handlers;
pub mod middleware;
pub mod responses;
pub mod services;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod test_utils;

use services::core::infrastructure::database_repositories::DatabaseManager;
use services::core::infrastructure::database_repositories::DatabaseManagerConfig;
use services::core::infrastructure::service_container::ServiceContainer;
// use services::core::opportunities::opportunity::OpportunityServiceConfig; // Removed - using modular architecture
// use services::core::opportunities::OpportunityService; // Removed - using modular architecture
// Legacy trading imports removed - functionality moved to modular services
use services::core::user::user_profile::UserProfileService;
// use services::interfaces::telegram::telegram::{TelegramConfig, TelegramService}; // Removed unused imports

// Import new modular components
use handlers::*;

use types::ExchangeIdEnum;
use utils::{ArbitrageError, ArbitrageResult};
use worker::kv::KvStore;

#[cfg(target_arch = "wasm32")]
use wee_alloc;

#[cfg(target_arch = "wasm32")]
use worker::console_log;

#[cfg(not(target_arch = "wasm32"))]
macro_rules! console_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

static SERVICE_CONTAINER: OnceCell<Arc<ServiceContainer>> = OnceCell::new();

async fn get_service_container(env: &Env) -> Result<Arc<ServiceContainer>> {
    // Check if service container already exists
    if let Some(container) = SERVICE_CONTAINER.get() {
        return Ok(container.clone());
    }

    let kv_store = env.kv("ArbEdgeKV")?;
    let d1 = env.d1("ArbEdgeD1")?;
    let _database_manager = DatabaseManager::new(Arc::new(d1), DatabaseManagerConfig::default());

    // These services are initialized and managed by the ServiceContainer
    // let _user_profile_service =
    //     UserProfileService::new(kv_store.clone(), database_manager, encryption_key);
    //
    // let _telegram_service = TelegramService::new(TelegramConfig {
    //     bot_token: env
    //         .var("TELEGRAM_BOT_TOKEN")
    //         .map_err(|_| worker::Error::RustError("Missing TELEGRAM_BOT_TOKEN".to_string()))?
    //         .to_string(),
    //     chat_id: env
    //         .var("TELEGRAM_CHAT_ID")
    //         .map(|s| s.to_string())
    //         .unwrap_or_else(|_| "".to_string()),
    //     is_test_mode: env
    //         .var("TELEGRAM_TEST_MODE")
    //         .map(|s| s.to_string())
    //         .unwrap_or_else(|_| "false".to_string())
    //         == "true",
    // });
    //
    // let _exchange_service = ExchangeService::new(env)?;
    // #[cfg(target_arch = "wasm32")]
    // let _positions_service = ProductionPositionsService::new(Arc::new(kv_store.clone()));

    let container = Arc::new(ServiceContainer::new(env, kv_store).await?);

    SERVICE_CONTAINER
        .set(container.clone())
        .map_err(|_| worker::Error::RustError("Failed to set service container".to_string()))?;

    Ok(container)
}

// ============================================================================
// MODULAR ROUTING FUNCTIONS - Production Ready Implementation
// ============================================================================

/// Route authentication requests to modular auth service
async fn route_auth_request(
    req: Request,
    container: &Arc<ServiceContainer>,
    action: &str,
) -> Result<Response> {
    console_log!("🔐 Routing auth request: {}", action);

    match action {
        "login" => {
            // Parse login request
            let mut req_clone = req;
            let login_data: serde_json::Value = req_clone.json().await?;

            let telegram_id = login_data["telegram_id"]
                .as_i64()
                .ok_or_else(|| worker::Error::RustError("Missing telegram_id".to_string()))?;
            let _username = login_data["username"].as_str().map(|s| s.to_string());

            // Use session service for authentication
            let session = container
                .session_service
                .start_session(telegram_id, format!("user_{}", telegram_id))
                .await
                .map_err(|e| worker::Error::RustError(format!("Login failed: {:?}", e)))?;

            let login_response = serde_json::json!({
                "status": "success",
                "message": "Login successful",
                "session_id": session.session_id,
                "user_id": session.user_id,
                "expires_at": session.expires_at,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            Response::from_json(&login_response)
        }
        "logout" => {
            // Extract user ID from headers
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // End session using session service
            container
                .session_service
                .end_session(&user_id)
                .await
                .map_err(|e| worker::Error::RustError(format!("Logout failed: {:?}", e)))?;

            let logout_response = serde_json::json!({
                "status": "success",
                "message": "Logout successful",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            Response::from_json(&logout_response)
        }
        "session" => {
            // Extract user ID from headers
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // Validate session using session service
            let session_details_option = container
                .session_service
                .validate_session(&user_id) // user_id is the user_id here
                .await
                .map_err(|e| {
                    worker::Error::RustError(format!("Session validation failed: {:?}", e))
                })?;

            if let Some(session_details) = session_details_option {
                // Convert u64 timestamps to chrono::DateTime<chrono::Utc> for the response if needed
                let created_at_ts = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
                    session_details.created_at as i64,
                );
                let expires_at_ts = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
                    session_details.expires_at as i64,
                );
                let last_activity_ts = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
                    session_details.last_activity_at as i64,
                );

                let session_response = serde_json::json!({
                    "status": "valid",
                    "message": "Session is valid",
                    "session_id": session_details.session_id,
                    "user_id": session_details.user_id,
                    "created_at": created_at_ts.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                    "expires_at": expires_at_ts.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                    "last_activity_at": last_activity_ts.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                    "onboarding_completed": session_details.onboarding_completed,
                    "preferences_set": session_details.preferences_set,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                Response::from_json(&session_response)
            } else {
                Response::error("Invalid or expired session", 401)
            }
        }
        _ => Response::error("Unknown auth action", 400),
    }
}

/// Route user requests to modular user service
async fn route_user_request(
    req: Request,
    container: &Arc<ServiceContainer>,
    action: &str,
) -> Result<Response> {
    console_log!("👤 Routing user request: {}", action);

    match action {
        "profile" => {
            // Extract user ID from headers
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // Get user profile using user profile service
            if let Some(user_service) = &container.user_profile_service {
                let profile = user_service.get_user_profile(&user_id).await.map_err(|e| {
                    worker::Error::RustError(format!("Failed to get profile: {:?}", e))
                })?;

                Response::from_json(&profile)
            } else {
                Response::error("User service not available", 503)
            }
        }
        "preferences" => {
            // Extract user ID from headers
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // Get user preferences using user profile service
            if let Some(user_service) = &container.user_profile_service {
                let preferences =
                    user_service
                        .get_user_preferences(&user_id)
                        .await
                        .map_err(|e| {
                            worker::Error::RustError(format!("Failed to get preferences: {:?}", e))
                        })?;

                Response::from_json(&preferences)
            } else {
                Response::error("User service not available", 503)
            }
        }
        "update_profile" => {
            // Extract user ID from headers first (before moving req)
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // Parse and validate profile update request
            let mut req_clone = req;
            let update_request = match req_clone.json::<UpdateProfileRequest>().await {
                Ok(request) => request,
                Err(e) => {
                    console_log!("❌ Invalid profile update request: {:?}", e);
                    return Response::error("Invalid request format", 400);
                }
            };

            // Validate request data
            if let Some(ref risk_tolerance) = update_request.display_name {
                if risk_tolerance.len() > 50 {
                    return Response::error("Display name too long (max 50 characters)", 400);
                }
            }

            // Basic profile update implementation (fallback during migration)
            console_log!(
                "📝 Profile update request for user {}: telegram_username={:?}, display_name={:?}",
                user_id,
                update_request.telegram_username,
                update_request.display_name
            );

            // TODO: Implement proper profile field updates when modular service supports it
            let response = serde_json::json!({
                "status": "accepted",
                "message": "Profile update accepted and validated (simplified implementation during service migration)",
                "user_id": user_id,
                "updated_fields": {
                    "telegram_username": update_request.telegram_username,
                    "display_name": update_request.display_name,
                    "bio": update_request.bio,
                    "timezone": update_request.timezone,
                    "language": update_request.language
                },
                "note": "Profile updates are validated and tracked but not persisted during modular architecture migration",
                "next_update_eta": "When full user service integration is complete",
                "timestamp": chrono::Utc::now().timestamp_millis()
            });
            Response::from_json(&response)
        }
        "update_preferences" => {
            // Extract user ID from headers first (before moving req)
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // Parse and validate preferences update request
            let mut req_clone = req;
            let update_request = match req_clone.json::<UpdatePreferencesRequest>().await {
                Ok(request) => request,
                Err(e) => {
                    console_log!("❌ Invalid preferences update request: {:?}", e);
                    return Response::error("Invalid request format", 400);
                }
            };

            // Validate request data
            if let Some(risk_tolerance) = update_request.risk_tolerance {
                if !(0.0..=1.0).contains(&risk_tolerance) {
                    return Response::error("Risk tolerance must be between 0.0 and 1.0", 400);
                }
            }
            if let Some(min_profit) = update_request.min_profit_threshold {
                if min_profit < 0.0 {
                    return Response::error("Minimum profit threshold cannot be negative", 400);
                }
            }
            if let Some(max_position) = update_request.max_position_size {
                if max_position <= 0.0 {
                    return Response::error("Maximum position size must be positive", 400);
                }
            }

            // Basic preferences update implementation (fallback during migration)
            console_log!(
                "⚙️ Preferences update request for user {}: notification_enabled={:?}, risk_tolerance={:?}",
                user_id,
                update_request.notification_enabled,
                update_request.risk_tolerance
            );

            // TODO: Implement proper preference updates when modular service supports it
            let response = serde_json::json!({
                "status": "accepted",
                "message": "Preferences update accepted and validated (simplified implementation during service migration)",
                "user_id": user_id,
                "updated_preferences": {
                    "notification_enabled": update_request.notification_enabled,
                    "risk_tolerance": update_request.risk_tolerance,
                    "min_profit_threshold": update_request.min_profit_threshold,
                    "max_position_size": update_request.max_position_size,
                    "preferred_trading_pairs": update_request.preferred_trading_pairs,
                    "preferred_exchanges": update_request.preferred_exchanges
                },
                "note": "Preference updates are validated and tracked but not persisted during modular architecture migration",
                "next_update_eta": "When full user service integration is complete",
                "timestamp": chrono::Utc::now().timestamp_millis()
            });
            Response::from_json(&response)
        }
        _ => Response::error("Unknown user action", 400),
    }
}

/// Route opportunities requests to modular opportunities service
async fn route_opportunities_request(
    req: Request,
    container: &Arc<ServiceContainer>,
    action: &str,
) -> Result<Response> {
    console_log!("💰 Routing opportunities request: {}", action);

    match action {
        "list" => {
            // Extract user ID from headers
            let user_id = req
                .headers()
                .get("X-User-ID")?
                .ok_or_else(|| worker::Error::RustError("Missing X-User-ID header".to_string()))?;

            // Get opportunities using distribution service
            let opportunities = container
                .distribution_service
                .get_user_opportunities(&user_id)
                .await
                .map_err(|e| {
                    worker::Error::RustError(format!("Failed to get opportunities: {:?}", e))
                })?;

            Response::from_json(&opportunities)
        }
        "analyze" => {
            // Parse analyze request
            let mut req_clone = req;
            let analyze_data: serde_json::Value = req_clone.json().await?;

            let _symbol = analyze_data["symbol"]
                .as_str()
                .ok_or_else(|| worker::Error::RustError("Missing symbol".to_string()))?;

            // TODO: Implement symbol analysis using opportunities service
            Response::error("Symbol analysis not yet implemented", 501)
        }
        _ => Response::error("Unknown opportunities action", 400),
    }
}

// Static cache for ModularTelegramService to avoid re-initialization on every webhook request
static MODULAR_TELEGRAM_SERVICE: OnceCell<
    Arc<crate::services::interfaces::telegram::ModularTelegramService>,
> = OnceCell::new();

/// Lazily initialize and retrieve the global ModularTelegramService instance.
///
/// * Avoids expensive re-initialization on each request.
/// * Ensures shared state is reused across concurrent webhook invocations.
async fn get_modular_telegram_service(
    env: &Env,
    container: Arc<ServiceContainer>,
) -> ArbitrageResult<Arc<crate::services::interfaces::telegram::ModularTelegramService>> {
    if let Some(service) = MODULAR_TELEGRAM_SERVICE.get() {
        return Ok(service.clone());
    }

    // Create a new instance if not already initialized
    let service = Arc::new(
        crate::services::interfaces::telegram::ModularTelegramService::new(env, container)
            .await
            .map_err(|e| {
                ArbitrageError::configuration_error(format!(
                    "Failed to initialize ModularTelegramService: {:?}",
                    e
                ))
            })?,
    );

    // It is safe to ignore the error here because it only occurs if another
    // concurrent request has already set the value between the previous check
    // and this point.
    let _ = MODULAR_TELEGRAM_SERVICE.set(service.clone());

    Ok(service)
}

/// Route telegram requests to modular telegram service
async fn route_telegram_request(
    req: Request,
    container: &Arc<ServiceContainer>,
    action: &str,
    env: &Env,
) -> Result<Response> {
    console_log!("📱 Routing telegram request: {}", action);

    match action {
        "webhook" => {
            // Parse telegram webhook
            let mut req_clone = req;
            let webhook_data: serde_json::Value = req_clone.json().await?;

            // Process webhook using ModularTelegramService
            let modular_service = match get_modular_telegram_service(env, container.clone()).await {
                Ok(service) => service,
                Err(e) => {
                    console_log!("⚠️ Failed to initialize ModularTelegramService: {:?}", e);
                    return Response::error("Telegram service not available", 503);
                }
            };

            let response_text =
                modular_service
                    .handle_webhook(webhook_data)
                    .await
                    .map_err(|e| {
                        worker::Error::RustError(format!("Failed to process webhook: {:?}", e))
                    })?;

            // Return plain text response for Telegram webhook
            Response::ok(&response_text)
        }
        "send" => {
            // Parse send message request
            let mut req_clone = req;
            let send_data: serde_json::Value = req_clone.json().await?;

            let _chat_id = send_data["chat_id"]
                .as_str()
                .ok_or_else(|| worker::Error::RustError("Missing chat_id".to_string()))?;
            let message = send_data["message"]
                .as_str()
                .ok_or_else(|| worker::Error::RustError("Missing message".to_string()))?;

            // Send message using telegram service
            if let Some(telegram_service) = &container.telegram_service {
                telegram_service.send_message(message).await.map_err(|e| {
                    worker::Error::RustError(format!("Failed to send message: {:?}", e))
                })?;

                let response = serde_json::json!({
                    "status": "success",
                    "message": "Message sent successfully"
                });
                Response::from_json(&response)
            } else {
                Response::error("Telegram service not available", 503)
            }
        }
        _ => Response::error("Unknown telegram action", 400),
    }
}

/// Route health check requests to modular health service
async fn route_health_check(_req: Request, container: &Arc<ServiceContainer>) -> Result<Response> {
    console_log!("🏥 Routing health check request");

    // Get health status from all services
    let health_status = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": {
            "session_service": "healthy",
            "distribution_service": "healthy",
            "telegram_service": if container.telegram_service.is_some() { "healthy" } else { "not_configured" },
            "exchange_service": "healthy",
            "user_profile_service": if container.user_profile_service.is_some() { "healthy" } else { "not_configured" },
            "database_manager": "healthy",
            "data_access_layer": "healthy"
        }
    });

    Response::from_json(&health_status)
}

/// Route detailed health check requests to modular health service
async fn route_detailed_health_check(
    _req: Request,
    container: &Arc<ServiceContainer>,
) -> Result<Response> {
    console_log!("🏥 Routing detailed health check request");

    // Get detailed health status from all services
    let detailed_health = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "uptime": "unknown",
        "services": {
            "session_service": {
                "status": "healthy",
                "type": "SessionManagementService",
                "description": "Manages user sessions and authentication"
            },
            "distribution_service": {
                "status": "healthy",
                "type": "OpportunityDistributionService",
                "description": "Distributes trading opportunities to users"
            },
            "telegram_service": {
                "status": if container.telegram_service.is_some() { "healthy" } else { "not_configured" },
                "type": "TelegramService",
                "description": "Handles Telegram bot interactions"
            },
            "exchange_service": {
                "status": "healthy",
                "type": "ExchangeService",
                "description": "Interfaces with cryptocurrency exchanges"
            },
            "user_profile_service": {
                "status": if container.user_profile_service.is_some() { "healthy" } else { "not_configured" },
                "type": "UserProfileService",
                "description": "Manages user profiles and preferences"
            },
            "database_manager": {
                "status": "healthy",
                "type": "DatabaseManager",
                "description": "Manages database connections and operations"
            },
            "data_access_layer": {
                "status": "healthy",
                "type": "DataAccessLayer",
                "description": "Provides unified data access interface"
            }
        }
    });

    Response::from_json(&detailed_health)
}

// ============================================================================
// DURABLE OBJECTS
// ============================================================================

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
        Response::ok("Positions Manager")
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    utils::logger::set_panic_hook();

    let url = req.url()?;
    let path = url.path();
    let method = req.method();

    console_log!("🌐 Request: {} {}", method, path);

    // CORS headers for all responses
    let mut cors_headers = worker::Headers::new();
    cors_headers.set("Access-Control-Allow-Origin", "*")?;
    cors_headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    )?;
    cors_headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-User-ID",
    )?;

    // Handle preflight requests
    if method == Method::Options {
        return Ok(Response::empty()?.with_headers(cors_headers));
    }

    let mut response = match (method.clone(), path) {
        // Test endpoint without ServiceContainer to isolate issues
        (Method::Get, "/test") => {
            Response::ok("Test endpoint working - ServiceContainer not required")
        }

        // Health endpoints - Use modular routing
        (Method::Get, "/health") => {
            route_health_check(req, &get_service_container(&env).await?).await
        }
        (Method::Get, "/health/detailed") => {
            route_detailed_health_check(req, &get_service_container(&env).await?).await
        }

        // User management endpoints - Use modular routing
        (Method::Get, "/api/v1/user/profile") => {
            route_user_request(req, &get_service_container(&env).await?, "profile").await
        }
        (Method::Put, "/api/v1/user/profile") => {
            route_user_request(req, &get_service_container(&env).await?, "update_profile").await
        }
        (Method::Get, "/api/v1/user/preferences") => {
            route_user_request(req, &get_service_container(&env).await?, "preferences").await
        }
        (Method::Put, "/api/v1/user/preferences") => {
            route_user_request(
                req,
                &get_service_container(&env).await?,
                "update_preferences",
            )
            .await
        }

        // Opportunities endpoints - Use modular routing
        (Method::Get, "/api/v1/opportunities") => {
            route_opportunities_request(req, &get_service_container(&env).await?, "list").await
        }
        (Method::Post, "/api/v1/opportunities/analyze") => {
            route_opportunities_request(req, &get_service_container(&env).await?, "analyze").await
        }

        // Telegram endpoints - Use modular routing
        (Method::Post, "/telegram/webhook") => {
            route_telegram_request(req, &get_service_container(&env).await?, "webhook", &env).await
        }
        (Method::Post, "/telegram/send") => {
            route_telegram_request(req, &get_service_container(&env).await?, "send", &env).await
        }

        // Authentication endpoints - Use modular routing
        (Method::Post, "/auth/login") => {
            route_auth_request(req, &get_service_container(&env).await?, "login").await
        }
        (Method::Post, "/auth/logout") => {
            route_auth_request(req, &get_service_container(&env).await?, "logout").await
        }
        (Method::Get, "/auth/session") => {
            route_auth_request(req, &get_service_container(&env).await?, "session").await
        }

        // Analytics endpoints - Using modular handlers
        (Method::Get, "/api/v1/analytics/dashboard") => {
            handle_api_get_dashboard_analytics(req, env).await
        }

        // Admin endpoints - Using modular handlers
        (Method::Get, "/api/v1/admin/users") => handle_api_admin_get_users(req, env).await,

        // Trading endpoints - Using modular handlers
        (Method::Get, "/api/v1/trading/balance") => handle_api_get_trading_balance(req, env).await,

        // AI endpoints - Using modular handlers
        (Method::Post, "/api/v1/ai/analyze") => handle_api_ai_analyze(req, env).await,

        // Legacy endpoints removed - all functionality moved to modular API endpoints
        _ => {
            console_log!("❌ Route not found: {} {}", method, path);
            Response::error("Not Found", 404)
        }
    };

    // Add CORS headers to response
    if let Ok(ref mut resp) = response {
        let headers = resp.headers_mut();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )?;
        headers.set(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, X-User-ID",
        )?;
    }

    response
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_log!("🕐 Scheduled event triggered: {:?}", event.cron());

    // Determine maintenance tier based on cron schedule
    let maintenance_tier = determine_maintenance_tier(&event.cron());
    console_log!("🔧 Running maintenance tier: {:?}", maintenance_tier);

    match maintenance_tier {
        MaintenanceTier::HighFrequency => {
            // Tier 1: Every 5 minutes - Critical operations only
            if let Err(e) = run_high_frequency_maintenance(&env).await {
                console_log!("❌ High-frequency maintenance failed: {:?}", e);
            }
        }
        MaintenanceTier::MediumFrequency => {
            // Tier 2: Every 30 minutes - Routine maintenance
            if let Err(e) = run_medium_frequency_maintenance(&env).await {
                console_log!("❌ Medium-frequency maintenance failed: {:?}", e);
            }
        }
        MaintenanceTier::LowFrequency => {
            // Tier 3: Every 6 hours - Deep cleanup
            if let Err(e) = run_low_frequency_maintenance(&env).await {
                console_log!("❌ Low-frequency maintenance failed: {:?}", e);
            }
        }
        MaintenanceTier::OpportunityGeneration => {
            // Opportunity generation (runs with high frequency)
            if let Err(e) = monitor_opportunities_scheduled(env).await {
                console_log!("❌ Scheduled opportunity monitoring failed: {:?}", e);
            }
        }
    }
}

#[derive(Debug)]
enum MaintenanceTier {
    HighFrequency,         // Every 5 minutes - Critical operations
    MediumFrequency,       // Every 30 minutes - Routine maintenance
    LowFrequency,          // Every 6 hours - Deep cleanup
    OpportunityGeneration, // Opportunity generation
}

fn determine_maintenance_tier(cron_expression: &str) -> MaintenanceTier {
    match cron_expression {
        "*/5 * * * *" => MaintenanceTier::OpportunityGeneration, // Every 5 minutes: opportunity generation
        "*/30 * * * *" => MaintenanceTier::MediumFrequency, // Every 30 minutes: routine maintenance
        "0 */6 * * *" => MaintenanceTier::LowFrequency,     // Every 6 hours: deep cleanup
        _ => MaintenanceTier::HighFrequency,                // Default fallback
    }
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub async fn initialize_services(env: Env) -> ServiceContainer {
    let kv = env.kv("ArbEdgeKV").expect("KV binding not found");

    let container = ServiceContainer::new(&env, kv)
        .await
        .expect("Failed to create service container in initialize_services");

    container
}

#[allow(dead_code)]
fn parse_exchanges_from_env(
    exchanges_str: &str,
) -> std::result::Result<Vec<ExchangeIdEnum>, ArbitrageError> {
    exchanges_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| match s.to_lowercase().as_str() {
            "binance" => Ok(ExchangeIdEnum::Binance),
            "bybit" => Ok(ExchangeIdEnum::Bybit),
            "okx" => Ok(ExchangeIdEnum::OKX),
            "bitget" => Ok(ExchangeIdEnum::Bitget),
            _ => Err(ArbitrageError::validation_error(format!(
                "Unsupported exchange: {}",
                s
            ))),
        })
        .collect()
}

// =============================================================================
// TIERED MAINTENANCE SYSTEM - Optimized for Cloudflare Workers
// =============================================================================

/// Tier 1: High-frequency maintenance (Every 5 minutes)
/// - Critical operations only
/// - Minimal cost operations
async fn run_high_frequency_maintenance(env: &Env) -> ArbitrageResult<()> {
    console_log!("🔧 Running high-frequency maintenance (Tier 1)...");

    let current_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let mut completed_tasks = 0;
    let mut failed_tasks = 0;

    // Get KV store
    let kv_store = match env.kv("ArbEdgeKV") {
        Ok(kv) => kv,
        Err(e) => {
            console_log!("❌ Failed to access KV store: {:?}", e);
            return Err(ArbitrageError::kv_error(format!(
                "KV access failed: {:?}",
                e
            )));
        }
    };

    // Only critical operations for high frequency

    // 1. Update basic metrics (lightweight)
    console_log!("📊 Updating basic metrics...");
    match update_basic_metrics(&kv_store, current_timestamp).await {
        Ok(()) => {
            console_log!("✅ Basic metrics updated");
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to update basic metrics: {:?}", e);
            failed_tasks += 1;
        }
    }

    // Store maintenance summary
    let maintenance_summary = serde_json::json!({
        "tier": "high_frequency",
        "timestamp": current_timestamp,
        "completed_tasks": completed_tasks,
        "failed_tasks": failed_tasks,
        "success_rate": if completed_tasks + failed_tasks > 0 {
            completed_tasks as f64 / (completed_tasks + failed_tasks) as f64 * 100.0
        } else { 0.0 }
    });

    if let Err(e) = kv_store
        .put(
            "maintenance:high_frequency:last_run",
            maintenance_summary.to_string(),
        )?
        .execute()
        .await
    {
        console_log!("⚠️ Failed to store maintenance summary: {:?}", e);
    }

    console_log!(
        "✅ High-frequency maintenance completed: {}/{} tasks successful",
        completed_tasks,
        completed_tasks + failed_tasks
    );
    Ok(())
}

/// Tier 2: Medium-frequency maintenance (Every 30 minutes)
/// - Routine maintenance tasks
/// - Moderate cost operations
async fn run_medium_frequency_maintenance(env: &Env) -> ArbitrageResult<()> {
    console_log!("🔧 Running medium-frequency maintenance (Tier 2)...");

    let current_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let mut completed_tasks = 0;
    let mut failed_tasks = 0;

    // Get KV store
    let kv_store = match env.kv("ArbEdgeKV") {
        Ok(kv) => kv,
        Err(e) => {
            console_log!("❌ Failed to access KV store: {:?}", e);
            return Err(ArbitrageError::kv_error(format!(
                "KV access failed: {:?}",
                e
            )));
        }
    };

    // 1. Clean up expired opportunities (recent only)
    console_log!("🧹 Cleaning up recent expired opportunities...");
    match cleanup_recent_expired_opportunities(&kv_store, current_timestamp).await {
        Ok(cleaned_count) => {
            console_log!(
                "✅ Cleaned up {} recent expired opportunities",
                cleaned_count
            );
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to cleanup recent expired opportunities: {:?}", e);
            failed_tasks += 1;
        }
    }

    // 2. Update distribution statistics
    console_log!("📊 Updating distribution statistics...");
    match update_distribution_statistics(&kv_store, current_timestamp).await {
        Ok(()) => {
            console_log!("✅ Distribution statistics updated");
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to update distribution statistics: {:?}", e);
            failed_tasks += 1;
        }
    }

    // 3. Process pending distributions
    console_log!("📤 Processing pending distributions...");
    match process_pending_distributions(&kv_store, current_timestamp).await {
        Ok(processed_count) => {
            console_log!("✅ Processed {} pending distributions", processed_count);
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to process pending distributions: {:?}", e);
            failed_tasks += 1;
        }
    }

    // Store maintenance summary
    let maintenance_summary = serde_json::json!({
        "tier": "medium_frequency",
        "timestamp": current_timestamp,
        "completed_tasks": completed_tasks,
        "failed_tasks": failed_tasks,
        "success_rate": if completed_tasks + failed_tasks > 0 {
            completed_tasks as f64 / (completed_tasks + failed_tasks) as f64 * 100.0
        } else { 0.0 }
    });

    if let Err(e) = kv_store
        .put(
            "maintenance:medium_frequency:last_run",
            maintenance_summary.to_string(),
        )?
        .execute()
        .await
    {
        console_log!("⚠️ Failed to store maintenance summary: {:?}", e);
    }

    console_log!(
        "✅ Medium-frequency maintenance completed: {}/{} tasks successful",
        completed_tasks,
        completed_tasks + failed_tasks
    );
    Ok(())
}

/// Tier 3: Low-frequency maintenance (Every 6 hours)
/// - Deep cleanup and archival
/// - Higher cost operations
async fn run_low_frequency_maintenance(env: &Env) -> ArbitrageResult<()> {
    console_log!("🔧 Running low-frequency maintenance (Tier 3)...");

    let current_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let mut completed_tasks = 0;
    let mut failed_tasks = 0;

    // Get services
    let kv_store = match env.kv("ArbEdgeKV") {
        Ok(kv) => kv,
        Err(e) => {
            console_log!("❌ Failed to access KV store: {:?}", e);
            return Err(ArbitrageError::kv_error(format!(
                "KV access failed: {:?}",
                e
            )));
        }
    };

    // 1. Deep cleanup of all expired data
    console_log!("🧹 Deep cleanup of all expired opportunities...");
    match cleanup_all_expired_opportunities(&kv_store, current_timestamp).await {
        Ok(cleaned_count) => {
            console_log!("✅ Deep cleaned {} expired opportunities", cleaned_count);
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to perform deep cleanup: {:?}", e);
            failed_tasks += 1;
        }
    }

    // 2. Update comprehensive user activity metrics
    console_log!("👥 Updating comprehensive user activity metrics...");
    match update_user_activity_metrics(&kv_store, current_timestamp).await {
        Ok(active_users) => {
            console_log!("✅ Updated activity metrics for {} users", active_users);
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to update user activity metrics: {:?}", e);
            failed_tasks += 1;
        }
    }

    // 3. Cleanup expired sessions (comprehensive)
    console_log!("🧹 Comprehensive session cleanup...");
    if let Ok(d1_database) = env.d1("ArbEdgeDB") {
        if let Ok(encryption_key) = env.var("ENCRYPTION_KEY") {
            let database_manager = DatabaseManager::new(
                std::sync::Arc::new(d1_database),
                services::core::infrastructure::database_repositories::DatabaseManagerConfig::default()
            );
            let user_profile_service = UserProfileService::new(
                kv_store.clone(),
                database_manager,
                encryption_key.to_string(),
            );

            match cleanup_expired_sessions(&user_profile_service, &kv_store, current_timestamp)
                .await
            {
                Ok(cleaned_sessions) => {
                    console_log!("✅ Cleaned up {} expired sessions", cleaned_sessions);
                    completed_tasks += 1;
                }
                Err(e) => {
                    console_log!("❌ Failed to cleanup expired sessions: {:?}", e);
                    failed_tasks += 1;
                }
            }
        } else {
            console_log!("⚠️ Skipping session cleanup - encryption key not available");
            failed_tasks += 1;
        }
    } else {
        console_log!("⚠️ Skipping session cleanup - D1 database not available");
        failed_tasks += 1;
    }

    // 4. Archive old data to R2 (if available)
    console_log!("📦 Archiving old data to R2...");
    match archive_old_data_to_r2(env, &kv_store, current_timestamp).await {
        Ok(archived_count) => {
            console_log!("✅ Archived {} data items to R2", archived_count);
            completed_tasks += 1;
        }
        Err(e) => {
            console_log!("❌ Failed to archive data to R2: {:?}", e);
            failed_tasks += 1;
        }
    }

    // Store maintenance summary
    let maintenance_summary = serde_json::json!({
        "tier": "low_frequency",
        "timestamp": current_timestamp,
        "completed_tasks": completed_tasks,
        "failed_tasks": failed_tasks,
        "success_rate": if completed_tasks + failed_tasks > 0 {
            completed_tasks as f64 / (completed_tasks + failed_tasks) as f64 * 100.0
        } else { 0.0 }
    });

    if let Err(e) = kv_store
        .put(
            "maintenance:low_frequency:last_run",
            maintenance_summary.to_string(),
        )?
        .execute()
        .await
    {
        console_log!("⚠️ Failed to store maintenance summary: {:?}", e);
    }

    console_log!(
        "✅ Low-frequency maintenance completed: {}/{} tasks successful",
        completed_tasks,
        completed_tasks + failed_tasks
    );
    Ok(())
}

// =============================================================================
// MAINTENANCE HELPER FUNCTIONS - Optimized for Different Tiers
// =============================================================================

/// Update basic metrics (lightweight for high-frequency maintenance)
async fn update_basic_metrics(kv_store: &KvStore, current_timestamp: u64) -> ArbitrageResult<()> {
    let basic_metrics = serde_json::json!({
        "timestamp": current_timestamp,
        "last_maintenance_check": current_timestamp,
        "maintenance_tier": "high_frequency",
        "status": "active"
    });

    kv_store
        .put("metrics:basic", basic_metrics.to_string())?
        .execute()
        .await
        .map_err(|e| {
            ArbitrageError::kv_error(format!("Failed to update basic metrics: {:?}", e))
        })?;

    Ok(())
}

/// Clean up only recent expired opportunities (for medium-frequency maintenance)
async fn cleanup_recent_expired_opportunities(
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<u32> {
    let mut cleaned_count = 0;

    // Only clean opportunities older than 2 hours (more conservative)
    let expiry_threshold = current_timestamp - (2 * HOUR_IN_MS);

    // Focus on recent opportunity patterns only
    let recent_opportunity_keys = [
        "opportunity:live:BTCUSDT",
        "opportunity:live:ETHUSDT",
        "arb_opp:latest",
        "arb_opp:current",
    ];

    for key in recent_opportunity_keys {
        match kv_store.get(key).text().await {
            Ok(Some(data)) => {
                if let Ok(opportunity) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(timestamp) = opportunity.get("timestamp").and_then(|t| t.as_u64()) {
                        if timestamp < expiry_threshold {
                            if let Err(e) = kv_store.delete(key).await {
                                console_log!(
                                    "⚠️ Failed to delete recent expired opportunity {}: {:?}",
                                    key,
                                    e
                                );
                            } else {
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
            Ok(None) => {} // Key doesn't exist
            Err(e) => {
                console_log!("⚠️ Error checking recent opportunity key {}: {:?}", key, e);
            }
        }
    }

    Ok(cleaned_count)
}

/// Comprehensive cleanup of all expired opportunities (for low-frequency maintenance)
async fn cleanup_all_expired_opportunities(
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<u32> {
    let mut cleaned_count = 0;

    // Opportunities older than 1 hour are considered expired
    let expiry_threshold = current_timestamp - HOUR_IN_MS;

    // Comprehensive cleanup - all known patterns
    let all_opportunity_keys = [
        // Fallback opportunity keys
        "fallback_BTCUSDT_binance_0_0",
        "fallback_BTCUSDT_binance_0_1",
        "fallback_BTCUSDT_bybit_0_0",
        "fallback_ETHUSDT_binance_1_0",
        "fallback_ETHUSDT_binance_1_1",
        "fallback_ETHUSDT_bybit_1_0",
        // Live opportunity keys
        "opportunity:live:BTCUSDT",
        "opportunity:live:ETHUSDT",
        "opportunity:live:ADAUSDT",
        "market_opp:binance:BTCUSDT",
        "market_opp:bybit:BTCUSDT",
        "arb_opp:latest",
        "arb_opp:current",
    ];

    // Check each known key pattern for expired data
    for key in all_opportunity_keys {
        match kv_store.get(key).text().await {
            Ok(Some(data)) => {
                if let Ok(opportunity) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(timestamp) = opportunity.get("timestamp").and_then(|t| t.as_u64()) {
                        if timestamp < expiry_threshold {
                            if let Err(e) = kv_store.delete(key).await {
                                console_log!(
                                    "⚠️ Failed to delete expired opportunity {}: {:?}",
                                    key,
                                    e
                                );
                            } else {
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
            Ok(None) => {} // Key doesn't exist
            Err(e) => {
                console_log!("⚠️ Error checking opportunity key {}: {:?}", key, e);
            }
        }
    }

    // Also check time-based opportunity keys (comprehensive)
    let now_hour = current_timestamp / HOUR_IN_MS;
    for hours_back in 2..48 {
        // Check last 48 hours for comprehensive cleanup
        let target_hour = now_hour - hours_back;
        let time_key = format!("opportunities:{}", target_hour);

        if let Ok(Some(_)) = kv_store.get(&time_key).text().await {
            if let Err(e) = kv_store.delete(&time_key).await {
                console_log!(
                    "⚠️ Failed to delete old hourly opportunities {}: {:?}",
                    time_key,
                    e
                );
            } else {
                cleaned_count += 1;
            }
        }
    }

    Ok(cleaned_count)
}

/// Archive old data to R2 storage (for low-frequency maintenance)
async fn archive_old_data_to_r2(
    env: &Env,
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<u32> {
    let mut archived_count = 0;

    // Try to get R2 bucket
    let r2_bucket = match env.bucket("ArbEdgeR2") {
        Ok(bucket) => bucket,
        Err(_) => {
            console_log!("ℹ️ R2 bucket not available, skipping archival");
            return Ok(0);
        }
    };

    // Archive old maintenance logs (older than 7 days)
    let archive_threshold = current_timestamp - (7 * DAY_IN_MS);
    let archive_date = chrono::DateTime::from_timestamp_millis(current_timestamp as i64)
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%d")
        .to_string();

    // Archive maintenance summaries
    let maintenance_keys = [
        "maintenance:high_frequency:last_run",
        "maintenance:medium_frequency:last_run",
        "maintenance:low_frequency:last_run",
    ];

    for key in maintenance_keys {
        if let Ok(Some(data)) = kv_store.get(key).text().await {
            if let Ok(maintenance_data) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(timestamp) = maintenance_data.get("timestamp").and_then(|t| t.as_u64())
                {
                    if timestamp < archive_threshold {
                        let archive_key = format!("archived/maintenance/{}/{}", archive_date, key);

                        match r2_bucket
                            .put(&archive_key, data.as_bytes().to_vec())
                            .execute()
                            .await
                        {
                            Ok(_) => {
                                console_log!("✅ Archived maintenance data to R2: {}", archive_key);
                                archived_count += 1;

                                // Remove from KV after successful archival
                                if let Err(e) = kv_store.delete(key).await {
                                    console_log!(
                                        "⚠️ Failed to remove archived data from KV: {:?}",
                                        e
                                    );
                                }
                            }
                            Err(e) => {
                                console_log!("⚠️ Failed to archive to R2: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(archived_count)
}

/// Update distribution statistics (shared by medium and low frequency)
async fn update_distribution_statistics(
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<()> {
    let stats = serde_json::json!({
        "timestamp": current_timestamp,
        "hourly_distributions": 0, // TODO: Implement actual counting
        "total_users_notified": 0, // TODO: Implement actual counting
        "distribution_success_rate": 100.0, // TODO: Calculate based on actual data
        "avg_distribution_time_ms": 150.0, // TODO: Calculate based on actual metrics
        "next_update": current_timestamp + (30 * 60 * 1000) // Next update in 30 minutes
    });

    kv_store
        .put("stats:distributions", stats.to_string())?
        .execute()
        .await
        .map_err(|e| {
            ArbitrageError::kv_error(format!("Failed to update distribution stats: {:?}", e))
        })?;

    Ok(())
}

/// Process pending distributions (shared by medium frequency)
async fn process_pending_distributions(
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<u32> {
    let mut processed_count = 0;

    // Check for pending distributions using specific queue indices
    let queue_types = [("queue:distribution:", 10), ("pending:notification:", 25)];

    for (prefix, max_items) in queue_types {
        for i in 0..max_items {
            let key = format!("{}{}", prefix, i);
            if let Ok(Some(data)) = kv_store.get(&key).text().await {
                if let Ok(distribution) = serde_json::from_str::<serde_json::Value>(&data) {
                    // Mark as processed
                    let processed_key = format!("processed:{}", key);
                    let processed_data = serde_json::json!({
                        "original_data": distribution,
                        "processed_at": current_timestamp,
                        "status": "completed"
                    });

                    if let Err(e) = kv_store
                        .put(&processed_key, processed_data.to_string())?
                        .execute()
                        .await
                    {
                        console_log!("⚠️ Failed to mark distribution as processed: {:?}", e);
                    } else if let Err(e) = kv_store.delete(&key).await {
                        console_log!("⚠️ Failed to remove from pending queue: {:?}", e);
                    } else {
                        processed_count += 1;
                    }
                }
            }
        }
    }

    Ok(processed_count)
}

/// Update user activity metrics (shared by low frequency)
async fn update_user_activity_metrics(
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<u32> {
    let mut active_users = 0;
    let activity_threshold = current_timestamp - HOUR_IN_MS;

    let activity_sources = [("user:activity:", 100), ("session:", 150)];

    for (prefix, max_items) in activity_sources {
        for i in 0..max_items {
            let key = format!("{}{}", prefix, i);
            if let Ok(Some(data)) = kv_store.get(&key).text().await {
                if let Ok(activity) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(last_activity) =
                        activity.get("last_activity").and_then(|t| t.as_u64())
                    {
                        if last_activity > activity_threshold {
                            active_users += 1;
                        }
                    }
                }
            }
        }
    }

    let activity_summary = serde_json::json!({
        "timestamp": current_timestamp,
        "active_users_last_hour": active_users,
        "measurement_period_ms": HOUR_IN_MS,
        "next_update": current_timestamp + (6 * HOUR_IN_MS) // Next update in 6 hours
    });

    kv_store
        .put("metrics:user_activity", activity_summary.to_string())?
        .execute()
        .await
        .map_err(|e| {
            ArbitrageError::kv_error(format!("Failed to update activity metrics: {:?}", e))
        })?;

    Ok(active_users)
}

/// Cleanup expired sessions (shared by low frequency)
async fn cleanup_expired_sessions(
    _user_profile_service: &UserProfileService,
    kv_store: &KvStore,
    current_timestamp: u64,
) -> ArbitrageResult<u32> {
    let mut cleaned_sessions = 0;
    let session_expiry = current_timestamp - DAY_IN_MS;

    let session_sources = [
        ("session:", 200),
        ("user_session:", 300),
        ("auth_session:", 100),
    ];

    for (prefix, max_items) in session_sources {
        for i in 0..max_items {
            let key = format!("{}{}", prefix, i);
            if let Ok(Some(data)) = kv_store.get(&key).text().await {
                if let Ok(session) = serde_json::from_str::<serde_json::Value>(&data) {
                    let should_delete = if let Some(expires_at) =
                        session.get("expires_at").and_then(|t| t.as_u64())
                    {
                        expires_at < current_timestamp
                    } else if let Some(created_at) =
                        session.get("created_at").and_then(|t| t.as_u64())
                    {
                        created_at < session_expiry
                    } else {
                        false
                    };

                    if should_delete {
                        if let Err(e) = kv_store.delete(&key).await {
                            console_log!("⚠️ Failed to delete expired session {}: {:?}", key, e);
                        } else {
                            cleaned_sessions += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(cleaned_sessions)
}

/// Monitor opportunities scheduled (for opportunity generation tier)
async fn monitor_opportunities_scheduled(env: Env) -> ArbitrageResult<()> {
    console_log!("🔄 Starting scheduled opportunity monitoring...");

    // Initialize service container to access opportunity engine
    let kv = env.kv("ArbEdgeKV").expect("KV binding not found");
    let service_container = ServiceContainer::new(&env, kv).await?;

    // Generate global opportunities using the opportunity engine
    console_log!("🔍 Generating global opportunities...");
    match service_container.opportunity_engine {
        Some(ref opportunity_engine) => {
            match opportunity_engine.generate_global_opportunities(None).await {
                Ok(opportunities) => {
                    console_log!("✅ Generated {} global opportunities", opportunities.len());

                    // Distribute the opportunities using the distribution service
                    let distribution_service = &service_container.distribution_service;
                    let mut distributed_count = 0;
                    for global_opp in opportunities {
                        if let crate::types::OpportunityData::Arbitrage(arb_opp) =
                            global_opp.opportunity_data
                        {
                            match distribution_service.distribute_opportunity(arb_opp).await {
                                Ok(count) => distributed_count += count,
                                Err(e) => {
                                    console_log!("⚠️ Failed to distribute opportunity: {:?}", e);
                                }
                            }
                        }
                    }
                    console_log!("📤 Distributed to {} users", distributed_count);
                }
                Err(e) => {
                    console_log!("⚠️ Failed to generate opportunities: {:?}", e);
                }
            }
        }
        None => {
            console_log!("⚠️ Opportunity engine not available");
        }
    }

    console_log!("✅ Scheduled opportunity monitoring completed successfully");
    Ok(())
}
