//! Core module for Telegram bot functionality

pub mod bot_client;
pub mod command_router;
pub mod message_handler;
pub mod webhook_handler;

// Re-export commonly used types
pub use bot_client::{TelegramError, TelegramResult};
pub use command_router::{CommandContext, CommandRouter, UserPermissions};
pub use message_handler::MessageHandler;
pub use webhook_handler::WebhookHandler;
