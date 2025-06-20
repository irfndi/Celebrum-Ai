use std::sync::Arc;
use worker::console_log;
use worker::*;

// Time constants for improved readability
const HOUR_IN_MS: u64 = 60 * 60 * 1000;

// Request validation structs will be added when needed

// Core modules - business logic and infrastructure
pub mod handlers;
pub mod middleware;
pub mod queue_handlers;
pub mod responses;
pub mod services;
pub mod types;
pub mod utils;

// Re-export commonly used types and functions
pub use handlers::*;
pub use middleware::{cors, rbac};
pub use responses::api_response::*;
pub use services::core::{infrastructure::ServiceContainer, opportunities::OpportunityEngine};
pub use types::*;
pub use utils::*;

// Main entry point for the Unified ArbEdge Worker
#[event(fetch)]
pub async fn main(req: Request, env: worker::Env, _ctx: worker::Context) -> Result<Response> {
    // Initialize logging and panic hook
    utils::logger::set_panic_hook();
    utils::logger::init_logger(utils::logger::LogLevel::Info);

    // Apply CORS and other middleware
    let req = middleware::cors::handle_cors_preflight(&req)?;
    let _url = req.url()?;

    // Route based on path
    Router::new()
        // === TELEGRAM BOT WEBHOOK ===
        .post_async("/telegram/webhook", |_req, _ctx| async move {
            // TODO: Implement proper telegram webhook integration
            // For now, return a simple response
            console_log!("Telegram webhook received");
            Ok(Response::ok("Telegram webhook received").unwrap())
        })
        // === CORE API ROUTES ===
        // Health check endpoint
        .get("/health", |_req, _ctx| {
            Response::ok("ArbEdge API is healthy")
        })
        // Admin routes (specific endpoints would be added here)
        // .get_async("/api/v1/admin/users", |req, ctx| async move { handlers::admin::handle_api_admin_get_users(req, ctx.env).await })
        // Trading routes (specific endpoints would be added here)
        // .get_async("/api/v1/trading/status", |req, ctx| async move { handlers::trading::handle_trading_status(req, ctx.env).await })
        // Analytics routes
        .get_async("/api/v1/analytics/*", |req, ctx| async move {
            handlers::analytics::handle_analytics_request(req, ctx.env).await
        })
        // AI routes
        .post_async("/api/v1/ai/*", |req, ctx| async move {
            handlers::ai::handle_ai_request(req, ctx.env).await
        })
        // === WEB FRONTEND FALLBACK (Astro) ===
        .get_async("/*", |req, _ctx| async move {
            // Serve static web content from R2 bucket or return index.html for SPA routing
            let path = req.path();
            if path.starts_with("/api/") || path.starts_with("/telegram/") {
                return Response::error("Not Found", 404);
            }

            // For now, return a simple response - will be enhanced when web deployment is added
            Response::ok("ArbEdge Trading Platform - Web interface coming soon")
        })
        // Catch all
        .or_else_any_method("/*", |_req, _ctx| Response::error("Not Found", 404))
        .run(req, env)
        .await
}

// Note: Scheduled event handler is currently broken in workers-rs
// See: https://github.com/cloudflare/workers-rs/issues/53
// The #[event(scheduled)] macro has a broken implementation
// For now, scheduled tasks should be handled via external cron services
// or implemented when the workers-rs library fixes the scheduled event support

// Queue consumer event handler (when available on paid plan)
#[event(queue)]
pub async fn queue(
    message_batch: MessageBatch<serde_json::Value>,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<()> {
    // Initialize services
    let kv_store = env.kv("ArbEdgeKV")?;
    let service_container = Arc::new(ServiceContainer::new(&env, kv_store).await.unwrap());

    // Process queue messages
    for message in message_batch.messages()? {
        let body = message.body();
        queue_handlers::process_queue_message(&env, body, service_container.clone()).await?;
    }

    Ok(())
}
