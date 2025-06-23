use crate::core::bot_client::{TelegramBotClient as BotClient, TelegramConfig};
use crate::core::command_router::{CommandContext, UserPermissions};
use crate::types::{TelegramCallbackQuery, TelegramMessage, TelegramUpdate};
use worker::{console_log, Env, Request, Response, Result, RouteContext};

// Re-export from handlers module
#[path = "../handlers/mod.rs"]
mod handlers_module;
pub use handlers_module::initialize_command_handlers;

fn parse_command(text: &str) -> (&str, Vec<&str>) {
    let mut parts = text.split_whitespace();
    let command = parts.next().unwrap_or("");
    let args = parts.collect();
    (command, args)
}

fn create_command_context(env: &Env, message: &TelegramMessage) -> CommandContext {
    let user = message.from.as_ref().unwrap();
    let is_admin = env
        .var("ADMIN_USER_IDS")
        .map(|ids| {
            ids.to_string()
                .split(',')
                .any(|id| id.trim() == user.id.to_string())
        })
        .unwrap_or(false);

    CommandContext {
        user_permissions: UserPermissions {
            is_admin,
            is_premium: false, // Placeholder
            user_level: 1,     // Placeholder
        },
        message_data: serde_json::to_value(message).unwrap_or_default(),
        bot_token: env.var("TELEGRAM_BOT_TOKEN").unwrap().to_string(),
    }
}

/// Handle incoming webhook from Telegram.
pub async fn handle_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    // Parse the incoming request as a TelegramUpdate
    let update: TelegramUpdate = match req.json().await {
        Ok(upd) => upd,
        Err(e) => {
            console_log!("Error parsing Telegram update: {:?}", e);
            return Response::error("Bad Request: Invalid JSON", 400);
        }
    };
    console_log!("📨 Received Telegram update: {:?}", update);

    handle_telegram_update(update, &env).await
}

/// Route the parsed Telegram update to the appropriate handler.
async fn handle_telegram_update(update: TelegramUpdate, env: &Env) -> Result<Response> {
    match update {
        TelegramUpdate {
            message: Some(message),
            ..
        } => handle_message(message, env).await,
        TelegramUpdate {
            callback_query: Some(callback_query),
            ..
        } => handle_callback_query(callback_query, env).await,
        _ => {
            console_log!("⚠️ Unhandled update type");
            Response::empty()
        }
    }
}

/// Handle incoming messages, routing commands and responding to other text.
async fn handle_message(message: TelegramMessage, env: &Env) -> Result<Response> {
    console_log!("💬 Processing message: {:?}", &message.text);

    if let Some(text) = &message.text {
        if text.starts_with('/') {
            // Initialize the router with all command handlers.
            let router = initialize_command_handlers();
                        let bot_token = env.var("TELEGRAM_BOT_TOKEN").unwrap().to_string();
            let chat_id = env.var("TELEGRAM_CHAT_ID").unwrap().to_string();
            let config = TelegramConfig {
                bot_token,
                chat_id,
                is_test_mode: false,
            };
            let bot_client = BotClient::new(config);

            // Route the command. The router now handles everything from parsing
            // to permission checks and execution.
                        let (command, args) = parse_command(text);
            let context = create_command_context(env, &message);

            return match router.route_command(command, message.chat.id, message.from.as_ref().unwrap().id, &args, &context).await {
                Ok(response_text) => {
                    bot_client.send_message(&message.chat.id.to_string(), &response_text, None, None).await
                }
                Err(e) => {
                    console_log!("Error routing command: {:?}", e);
                    Response::error("Internal Server Error", 500)
                }
            }
        }
    }

    // For non-command messages, send a helpful tip.
        let bot_token = env.var("TELEGRAM_BOT_TOKEN").unwrap().to_string();
    let chat_id = env.var("TELEGRAM_CHAT_ID").unwrap().to_string();
    let config = TelegramConfig {
        bot_token,
        chat_id,
        is_test_mode: false,
    };
    let bot_client = BotClient::new(config);
    let _ = bot_client
        .send_message(
            &message.chat.id.to_string(),
            "I can only understand commands right now. Please try `/help` to see what I can do!",
            None,
            None,
        )
        .await;
    Response::empty()
}

/// Handle callback queries from inline keyboards by routing them as commands.
async fn handle_callback_query(
    callback_query: TelegramCallbackQuery,
    env: &Env,
) -> Result<Response> {
    console_log!("🔘 Processing callback query: {:?}", &callback_query.data);

    if let (Some(data), Some(mut message)) = (callback_query.data, callback_query.message) {
        // Transform the callback data into a command string.
        let command_text = format!("/{}", data);
        message.text = Some(command_text);

        // The user who clicked the button is the 'from' user.
        message.from = Some(callback_query.from);

        // Initialize the router and bot client.
        let router = initialize_command_handlers();

        // Route the constructed message as a command.
                return router.route_command(&message.text.clone().unwrap(), message.chat.id, message.from.as_ref().unwrap().id, &[], &create_command_context(env, &message)).await.map(|_| Response::empty()).unwrap_or_else(|_| Response::error("Failed to handle callback", 500));
    }

    console_log!("⚠️ Callback query is missing data or the original message.");
    Response::empty()
}
