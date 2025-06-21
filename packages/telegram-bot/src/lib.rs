mod handlers;
mod types;
mod utils;

use crate::handlers::handle_webhook;
use crate::types::*;
use worker::{event, Env, Request, Response, Result, RouteContext, Router};

/// Main Telegram Bot wrapper
#[derive(Clone)]
pub struct TelegramBot {
    config: TelegramConfig,
}

impl TelegramBot {
    pub fn new(env: &Env) -> worker::Result<Self> {
        let config = TelegramConfig::from_env(env)?;
        Ok(Self { config })
    }

    pub async fn handle_webhook(
        &self,
        req: Request,
        ctx: RouteContext<()>,
    ) -> worker::Result<Response> {
        handle_webhook(req, ctx).await
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    utils::init_logger();

    // Initialize telegram bot
    let bot = TelegramBot::new(&env)?;

    // Route the request
    Router::new()
        .post_async("/webhook", move |req, ctx| {
            let bot = bot.clone();
            async move { bot.handle_webhook(req, ctx).await }
        })
        .get("/health", |_, _| Response::ok("Telegram Bot is healthy"))
        .run(req, env)
        .await
}

#[event(start)]
pub fn start() {
    console_log!("🤖 ArbEdge Telegram Bot starting...");
}
