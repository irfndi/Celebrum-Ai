// src/services/interfaces/telegram/mod.rs

//! Telegram Interface Module
//! 
//! This module contains the Telegram bot interface components for user interaction,
//! including message handling, keyboard interfaces, and command processing.
//! 
//! ## Components
//! - `TelegramService`: Main Telegram bot service for message handling
//! - `InlineKeyboard`: Interactive keyboard components for user interface
//! - `InlineKeyboardButton`: Individual button components for keyboards

pub mod telegram;
pub mod telegram_keyboard;

pub use telegram::TelegramService;
pub use telegram_keyboard::{InlineKeyboard, InlineKeyboardButton}; 