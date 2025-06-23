mod handlers;
mod integrations;
mod types;
mod utils;

// Include core module from parent directory
#[path = "../core/mod.rs"]
mod core;

use worker::{console_log, Env};

// Re-export handle_webhook for external use
pub use crate::handlers::handle_webhook;

/// Main Telegram Bot wrapper with modular command routing
#[derive(Clone)]
pub struct TelegramBot {
    // For now, we'll use the existing handlers module
}

impl TelegramBot {
    pub fn new(_env: &Env) -> worker::Result<Self> {
        console_log!("🔧 Initializing TelegramBot with modular command routing");
        Ok(Self {})
    }
}

// Note: Event handlers removed to avoid symbol conflicts
// This package is used as a library by the main worker
