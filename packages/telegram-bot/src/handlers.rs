use crate::integrations::{
    get_admin_statistics, get_user_balance, get_user_opportunities, get_user_profile_data,
    get_user_settings, send_telegram_message, verify_admin_access,
};
use crate::types::{TelegramCallbackQuery, TelegramMessage, TelegramUpdate};
use worker::{console_log, Env, Request, Response, Result, RouteContext};

// Re-export from handlers module
#[path = "../handlers/mod.rs"]
mod handlers_module;
pub use handlers_module::initialize_command_handlers;

/// Handle incoming webhook from Telegram
pub async fn handle_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    // Parse the incoming request as TelegramUpdate
    let update: TelegramUpdate = req.json().await?;
    console_log!("📨 Received Telegram update: {:?}", update);

    handle_telegram_update(update, &env).await
}

/// Handle parsed Telegram update
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

/// Handle incoming messages
async fn handle_message(message: TelegramMessage, env: &Env) -> Result<Response> {
    console_log!("💬 Processing message: {:?}", message.text);

    if let Some(text) = &message.text {
        if text.starts_with('/') {
            return handle_command(message, env).await;
        }
    }

    // Handle non-command messages
    send_telegram_message(
        env,
        message.chat.id,
        "I understand text messages, but I work best with commands. Try /help to see what I can do!"
    ).await
}

/// Handle command messages
async fn handle_command(message: TelegramMessage, env: &Env) -> Result<Response> {
    let text = message.text.as_ref().unwrap();
    let parts: Vec<&str> = text.split_whitespace().collect();
    let command = parts[0];
    let args = if parts.len() > 1 { &parts[1..] } else { &[] };

    let user_id = message
        .from
        .as_ref()
        .map(|u| u.id.to_string())
        .unwrap_or_default();

    match command {
        "/start" => handle_start_command(&message, env).await,
        "/help" => handle_help_command(&message, env).await,
        "/profile" => handle_profile_command(&message, env, &user_id).await,
        "/opportunities" => handle_opportunities_command(&message, env, &user_id, args).await,
        "/admin" => handle_admin_command(&message, env, &user_id, args).await,
        "/balance" => handle_balance_command(&message, env, &user_id).await,
        "/settings" => handle_settings_command(&message, env, &user_id).await,
        _ => {
            send_telegram_message(
                env,
                message.chat.id,
                &format!(
                    "Unknown command: {}. Try /help for available commands.",
                    command
                ),
            )
            .await
        }
    }
}

/// Handle /start command
async fn handle_start_command(message: &TelegramMessage, env: &Env) -> Result<Response> {
    let user_id = message
        .from
        .as_ref()
        .map(|u| u.id.to_string())
        .unwrap_or_default();

    // Get user profile to personalize welcome message
    let profile_result = get_user_profile_data(env, &user_id).await;

    let welcome_text = match profile_result {
        Ok(profile) => {
            format!(
                "🎯 Welcome back, {}!\n\n\
                Your AI-powered arbitrage trading assistant is ready.\n\
                📊 Your Stats: {} trades, ${:.2} PnL\n\n\
                📊 Available Commands:\n\
                • /profile - View your trading profile\n\
                • /opportunities - Check current arbitrage opportunities\n\
                • /balance - View your account balance\n\
                • /settings - Manage your trading preferences\n\
                • /help - Get detailed help\n\n\
                💡 Tip: Check /opportunities for the latest arbitrage deals!",
                profile
                    .telegram_username
                    .unwrap_or_else(|| "Trader".to_string()),
                profile.total_trades,
                profile.total_pnl_usdt
            )
        }
        Err(_) => "🚀 Welcome to ArbEdge Bot!\n\n\
            I'm your AI-powered arbitrage trading assistant. Here's what I can help you with:\n\n\
            📊 /opportunities - View current arbitrage opportunities\n\
            👤 /profile - Manage your trading profile\n\
            💰 /balance - Check your account balance\n\
            ⚙️ /settings - Configure your preferences\n\
            ❓ /help - Get detailed help\n\n\
            Let's start making profitable trades together!"
            .to_string(),
    };

    send_telegram_message(env, message.chat.id, &welcome_text).await
}

/// Handle /help command
async fn handle_help_command(message: &TelegramMessage, env: &Env) -> Result<Response> {
    let help_text = "🤖 ArbEdge Bot Commands:\n\n\
        🚀 /start - Welcome message and quick start\n\
        📊 /opportunities [filter] - View arbitrage opportunities\n\
        👤 /profile - View and manage your profile\n\
        💰 /balance - Check account balance and P&L\n\
        ⚙️ /settings - Configure trading preferences\n\
        👑 /admin [action] - Admin functions (admin only)\n\
        ❓ /help - Show this help message\n\n\
        💡 Pro tip: Use /opportunities high to see only high-profit opportunities!";

    send_telegram_message(env, message.chat.id, help_text).await
}

/// Handle /profile command - integrates with sophisticated user management
async fn handle_profile_command(
    message: &TelegramMessage,
    env: &Env,
    user_id: &str,
) -> Result<Response> {
    match get_user_profile_data(env, user_id).await {
        Ok(profile_data) => {
            let profile_text = format!(
                "👤 Your Profile:\n\n\
                🆔 User ID: {}\n\
                📱 Telegram: @{}\n\
                🎯 Subscription: {}\n\
                📈 Total Trades: {}\n\
                💰 Total P&L: ${:.2}\n\
                💳 Balance: ${:.2}\n\
                ⚡ Status: {}\n\n\
                Use /settings to modify your preferences.",
                profile_data.user_id,
                profile_data
                    .telegram_username
                    .unwrap_or("Not set".to_string()),
                profile_data.subscription_tier,
                profile_data.total_trades,
                profile_data.total_pnl_usdt,
                profile_data.account_balance_usdt,
                if profile_data.is_active {
                    "Active"
                } else {
                    "Inactive"
                }
            );
            send_telegram_message(env, message.chat.id, &profile_text).await
        }
        Err(e) => {
            console_log!("❌ Failed to get user profile: {:?}", e);
            send_telegram_message(
                env,
                message.chat.id,
                "❌ Unable to retrieve your profile. Please try again later.",
            )
            .await
        }
    }
}

/// Handle /opportunities command - integrates with sophisticated opportunity engine
async fn handle_opportunities_command(
    message: &TelegramMessage,
    env: &Env,
    user_id: &str,
    args: &[&str],
) -> Result<Response> {
    match get_user_opportunities(env, user_id, args).await {
        Ok(opportunities) => {
            if opportunities.is_empty() {
                send_telegram_message(
                    env,
                    message.chat.id,
                    "📊 No arbitrage opportunities found at the moment. Check back soon!",
                )
                .await
            } else {
                let mut opp_text = format!(
                    "📊 Found {} Arbitrage Opportunities:\n\n",
                    opportunities.len()
                );

                for (i, opp) in opportunities.iter().take(5).enumerate() {
                    opp_text.push_str(&format!(
                        "{}. 💰 {:.2}% profit\n\
                        📈 {} → {}\n\
                        💵 ${:.2} volume\n\n",
                        i + 1,
                        opp.profit_percentage,
                        opp.buy_exchange,
                        opp.sell_exchange,
                        opp.volume_usdt
                    ));
                }

                if opportunities.len() > 5 {
                    opp_text.push_str(&format!(
                        "... and {} more opportunities available.",
                        opportunities.len() - 5
                    ));
                }

                send_telegram_message(env, message.chat.id, &opp_text).await
            }
        }
        Err(e) => {
            console_log!("❌ Failed to get opportunities: {:?}", e);
            send_telegram_message(
                env,
                message.chat.id,
                "❌ Unable to retrieve opportunities. Please try again later.",
            )
            .await
        }
    }
}

/// Handle /admin command - integrates with sophisticated admin services
async fn handle_admin_command(
    message: &TelegramMessage,
    env: &Env,
    user_id: &str,
    args: &[&str],
) -> Result<Response> {
    match verify_admin_access(env, user_id).await {
        Ok(true) => {
            let action = args.first().unwrap_or(&"dashboard");

            match *action {
                "stats" | "statistics" => match get_admin_statistics(env).await {
                    Ok(stats) => {
                        let stats_text = format!(
                            "👑 Admin Statistics:\n\n\
                                👥 Total Users: {}\n\
                                🔥 Active Users: {}\n\
                                👑 Admin Users: {}\n\
                                💰 Total Volume: ${:.2}\n\
                                📈 Total Trades: {}\n\n\
                                System Status: ✅ Operational",
                            stats.total_users,
                            stats.active_users,
                            stats.admin_users,
                            stats.total_volume_usdt,
                            stats.total_trades
                        );
                        send_telegram_message(env, message.chat.id, &stats_text).await
                    }
                    Err(e) => {
                        console_log!("❌ Failed to get admin stats: {:?}", e);
                        send_telegram_message(
                            env,
                            message.chat.id,
                            "❌ Unable to retrieve admin statistics.",
                        )
                        .await
                    }
                },
                "dashboard" => {
                    let dashboard_text = "👑 Admin Dashboard\n\n\
                        Available commands:\n\
                        📊 /admin stats - View system statistics\n\
                        👥 /admin users - User management\n\
                        ⚙️ /admin config - System configuration\n\n\
                        System Status: ✅ All services operational";
                    send_telegram_message(env, message.chat.id, dashboard_text).await
                }
                _ => {
                    let dashboard_text = "👑 Admin Dashboard\n\n\
                        Available commands:\n\
                        📊 /admin stats - View system statistics\n\
                        👥 /admin users - User management\n\
                        ⚙️ /admin config - System configuration\n\n\
                        System Status: ✅ All services operational";
                    send_telegram_message(env, message.chat.id, dashboard_text).await
                }
            }
        }
        Ok(false) => {
            send_telegram_message(
                env,
                message.chat.id,
                "❌ Access denied. Admin privileges required.",
            )
            .await
        }
        Err(e) => {
            console_log!("❌ Failed to verify admin access: {:?}", e);
            send_telegram_message(
                env,
                message.chat.id,
                "❌ Unable to verify permissions. Please try again later.",
            )
            .await
        }
    }
}

/// Handle /balance command
async fn handle_balance_command(
    message: &TelegramMessage,
    env: &Env,
    user_id: &str,
) -> Result<Response> {
    match get_user_balance(env, user_id).await {
        Ok(balance_data) => {
            let balance_text = format!(
                "💰 Your Balance:\n\n\
                💳 Account Balance: ${:.2}\n\
                📈 Total P&L: ${:.2}\n\
                📊 Total Trades: {}\n\
                🎯 Win Rate: {:.1}%\n\
                ⚡ Risk Level: {}\n\n\
                💡 Ready to find new opportunities? Try /opportunities",
                balance_data.account_balance_usdt,
                balance_data.total_pnl_usdt,
                balance_data.total_trades,
                balance_data.win_rate,
                balance_data.risk_level
            );
            send_telegram_message(env, message.chat.id, &balance_text).await
        }
        Err(e) => {
            console_log!("❌ Failed to get balance: {:?}", e);
            send_telegram_message(
                env,
                message.chat.id,
                "❌ Unable to retrieve balance information. Please try again later.",
            )
            .await
        }
    }
}

/// Handle /settings command
async fn handle_settings_command(
    message: &TelegramMessage,
    env: &Env,
    user_id: &str,
) -> Result<Response> {
    match get_user_settings(env, user_id).await {
        Ok(settings) => {
            let settings_text = format!(
                "⚙️ Your Settings:\n\n\
                🎯 Risk Tolerance: {:.1}%\n\
                🤖 Auto Trading: {}\n\
                📊 Max Leverage: {}x\n\
                💰 Max Position: ${:.2}\n\
                🔔 Notifications: {}\n\n\
                To modify settings, visit the web dashboard or contact support.",
                settings.risk_tolerance_percentage,
                if settings.auto_trading_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                settings.max_leverage,
                settings.max_entry_size_usdt,
                if settings.notifications_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
            send_telegram_message(env, message.chat.id, &settings_text).await
        }
        Err(e) => {
            console_log!("❌ Failed to get settings: {:?}", e);
            send_telegram_message(
                env,
                message.chat.id,
                "❌ Unable to retrieve settings. Please try again later.",
            )
            .await
        }
    }
}

/// Handle callback queries from inline keyboards
async fn handle_callback_query(
    callback_query: TelegramCallbackQuery,
    env: &Env,
) -> Result<Response> {
    console_log!("🔘 Processing callback query: {:?}", callback_query.data);

    if let Some(data) = &callback_query.data {
        match data.as_str() {
            "refresh_opportunities" => {
                // Handle opportunity refresh
                if let Some(message) = &callback_query.message {
                    let user_id = callback_query.from.id.to_string();
                    return handle_opportunities_command(message, env, &user_id, &[]).await;
                }
            }
            "view_profile" => {
                // Handle profile view
                if let Some(message) = &callback_query.message {
                    let user_id = callback_query.from.id.to_string();
                    return handle_profile_command(message, env, &user_id).await;
                }
            }
            _ => {
                console_log!("⚠️ Unhandled callback data: {}", data);
            }
        }
    }

    Response::empty()
}
