// src/services/interfaces/mod.rs

pub mod api;
pub mod discord;
pub mod telegram;

// Re-export specific items from telegram module
pub use telegram::telegram_keyboard::{InlineKeyboard, InlineKeyboardButton};
pub use telegram::{ModularTelegramService, TelegramService, UserInfo, UserPermissions};
