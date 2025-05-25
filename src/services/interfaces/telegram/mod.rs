// src/services/interfaces/telegram/mod.rs

pub mod telegram;
pub mod telegram_keyboard;

pub use telegram::TelegramService;
pub use telegram_keyboard::{InlineKeyboard, InlineKeyboardButton}; 