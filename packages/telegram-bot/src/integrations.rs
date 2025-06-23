//! Integration functions to connect Telegram bot with sophisticated ArbEdge services

use worker::{console_log, Env, Response, Result};

// Data structures for integration
#[derive(Debug, Clone)]
pub struct UserProfileData {
    pub user_id: String,
    pub telegram_username: Option<String>,
    pub subscription_tier: String,
    pub total_trades: u32,
    pub total_pnl_usdt: f64,
    pub account_balance_usdt: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct OpportunityData {
    pub profit_percentage: f64,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub volume_usdt: f64,
}

#[derive(Debug, Clone)]
pub struct AdminStatistics {
    pub total_users: u32,
    pub active_users: u32,
    pub admin_users: u32,
    pub total_volume_usdt: f64,
    pub total_trades: u32,
}

#[derive(Debug, Clone)]
pub struct BalanceData {
    pub account_balance_usdt: f64,
    pub total_pnl_usdt: f64,
    pub total_trades: u32,
    pub win_rate: f64,
    pub risk_level: String,
}

#[derive(Debug, Clone)]
pub struct UserSettings {
    pub risk_tolerance_percentage: f64,
    pub auto_trading_enabled: bool,
    pub max_leverage: u32,
    pub max_entry_size_usdt: f64,
    pub notifications_enabled: bool,
}

/// Get user profile data from sophisticated user management service
pub async fn get_user_profile_data(env: &Env, user_id: &str) -> Result<UserProfileData> {
    console_log!("🔍 Fetching user profile for: {}", user_id);

    // Initialize services similar to main handlers
    let _kv_store = env.kv("ArbEdgeKV")?;
    let _encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;

    // For now, return mock data that matches the sophisticated service structure
    // TODO: Integrate with actual UserProfileService from src/services/core/user/user_profile
    Ok(UserProfileData {
        user_id: user_id.to_string(),
        telegram_username: Some(format!("user_{}", user_id)),
        subscription_tier: "Premium".to_string(),
        total_trades: 42,
        total_pnl_usdt: 1250.75,
        account_balance_usdt: 5000.00,
        is_active: true,
    })
}

/// Get user opportunities from sophisticated opportunity engine
pub async fn get_user_opportunities(
    env: &Env,
    user_id: &str,
    filters: &[&str],
) -> Result<Vec<OpportunityData>> {
    console_log!(
        "📊 Fetching opportunities for user: {} with filters: {:?}",
        user_id,
        filters
    );

    // Initialize opportunity services
    let _kv_store = env.kv("ArbEdgeKV")?;

    // For now, return mock data that matches the sophisticated opportunity structure
    // TODO: Integrate with actual OpportunityEngine from src/services/core/opportunities
    let mut opportunities = vec![
        OpportunityData {
            profit_percentage: 2.45,
            buy_exchange: "Binance".to_string(),
            sell_exchange: "Coinbase".to_string(),
            volume_usdt: 10000.0,
        },
        OpportunityData {
            profit_percentage: 1.87,
            buy_exchange: "Kraken".to_string(),
            sell_exchange: "Binance".to_string(),
            volume_usdt: 7500.0,
        },
        OpportunityData {
            profit_percentage: 3.12,
            buy_exchange: "Coinbase".to_string(),
            sell_exchange: "Kraken".to_string(),
            volume_usdt: 5000.0,
        },
    ];

    // Apply filters
    if filters.contains(&"high") {
        opportunities.retain(|opp| opp.profit_percentage > 2.0);
    }

    Ok(opportunities)
}

/// Verify admin access using sophisticated admin service
pub async fn verify_admin_access(env: &Env, user_id: &str) -> Result<bool> {
    console_log!("🔐 Verifying admin access for user: {}", user_id);

    // Initialize services
    let _kv_store = env.kv("ArbEdgeKV")?;
    let _encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;

    // For now, return mock admin verification
    // TODO: Integrate with actual admin verification from src/handlers/admin.rs
    Ok(user_id == "admin" || user_id == "123456789") // Mock admin user IDs
}

/// Get admin statistics from sophisticated admin service
pub fn get_admin_statistics() -> Result<AdminStatistics> {
    console_log!("📊 Fetching admin statistics");

    // For now, return mock statistics
    // TODO: Integrate with actual SimpleAdminService from src/services/core/admin
    Ok(AdminStatistics {
        total_users: 1247,
        active_users: 892,
        admin_users: 5,
        total_volume_usdt: 2_450_000.0,
        total_trades: 15_678,
    })
}

/// Get user balance data
pub async fn get_user_balance(user_id: &str) -> Result<BalanceData> {
    console_log!("💰 Fetching balance for user: {}", user_id);

    // For now, return mock balance data
    // TODO: Integrate with actual trading service from src/handlers/trading.rs
    Ok(BalanceData {
        account_balance_usdt: 5000.00,
        total_pnl_usdt: 1250.75,
        total_trades: 42,
        win_rate: 73.8,
        risk_level: "Medium".to_string(),
    })
}

/// Get user settings
pub async fn get_user_settings(_env: &Env, user_id: &str) -> Result<UserSettings> {
    console_log!("⚙️ Fetching settings for user: {}", user_id);

    // For now, return mock settings data
    // TODO: Integrate with actual user management service
    Ok(UserSettings {
        risk_tolerance_percentage: 15.0,
        auto_trading_enabled: true,
        max_leverage: 3,
        max_entry_size_usdt: 1000.0,
        notifications_enabled: true,
    })
}

/// Send a message to Telegram
pub async fn send_telegram_message(env: &Env, chat_id: i64, text: &str) -> Result<Response> {
    console_log!("📤 Sending message to chat {}: {}", chat_id, text);

    let bot_token = env
        .var("TELEGRAM_BOT_TOKEN")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError(
                "TELEGRAM_BOT_TOKEN environment variable is required".to_string(),
            )
        })?;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML"
    });

    let headers = worker::Headers::new();
    headers.set("Content-Type", "application/json")?;

    let request = worker::Request::new_with_init(
        &url,
        worker::RequestInit::new()
            .with_method(worker::Method::Post)
            .with_headers(headers)
            .with_body(Some(worker::wasm_bindgen::JsValue::from_str(
                &payload.to_string(),
            ))),
    )?;

    match worker::Fetch::Request(request).send().await {
        Ok(_response) => {
            console_log!("✅ Message sent successfully");
            Ok(Response::ok("Message sent")?)
        }
        Err(e) => {
            console_log!("❌ Failed to send message: {:?}", e);
            Err(worker::Error::RustError(format!(
                "Failed to send Telegram message: {:?}",
                e
            )))
        }
    }
}
