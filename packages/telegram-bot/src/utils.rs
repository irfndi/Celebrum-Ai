//! Utility functions for Telegram Bot

use console_error_panic_hook;
use log::Level;

/// Set up panic hook for better error reporting
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Initialize logger
pub fn init_logger() {
    console_log::init_with_level(Level::Info).expect("Failed to initialize logger");
}

/// Log macro for console output
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log::info!($($t)*));
}

/// Utility function to validate Telegram bot token
pub fn validate_bot_token(token: &str) -> bool {
    // Basic validation: should be in format "bot123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
    token.len() > 10 && token.contains(':')
}

/// Extract chat ID from Telegram update
pub fn extract_chat_id(update: &crate::types::TelegramUpdate) -> Option<i64> {
    if let Some(ref message) = update.message {
        Some(message.chat.id)
    } else if let Some(ref callback_query) = update.callback_query {
        callback_query.message.as_ref().map(|msg| msg.chat.id)
    } else {
        None
    }
}

/// Extract user ID from Telegram update
pub fn extract_user_id(update: &crate::types::TelegramUpdate) -> Option<i64> {
    if let Some(ref message) = update.message {
        message.from.as_ref().map(|user| user.id)
    } else if let Some(ref callback_query) = update.callback_query {
        Some(callback_query.from.id)
    } else {
        None
    }
}
