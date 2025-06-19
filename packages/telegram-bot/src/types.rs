//! Types for Telegram Bot

use serde::{Deserialize, Serialize};
use worker::Env;

/// Configuration for Telegram Bot
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub webhook_url: Option<String>,
}

impl TelegramConfig {
    pub fn from_env(env: &Env) -> worker::Result<Self> {
        let bot_token = env.var("TELEGRAM_BOT_TOKEN")?
            .to_string();
        
        let webhook_url = env.var("TELEGRAM_WEBHOOK_URL")
            .ok()
            .map(|v| v.to_string());
            
        Ok(Self {
            bot_token,
            webhook_url,
        })
    }
}

/// Telegram Update structure
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub callback_query: Option<TelegramCallbackQuery>,
}

/// Telegram Message structure
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub date: i64,
    pub text: Option<String>,
}

/// Telegram User structure
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

/// Telegram Chat structure
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
}

/// Telegram Callback Query structure
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramCallbackQuery {
    pub id: String,
    pub from: TelegramUser,
    pub message: Option<TelegramMessage>,
    pub data: Option<String>,
}