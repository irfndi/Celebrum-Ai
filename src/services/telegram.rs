// src/services/telegram.rs

use crate::utils::{ArbitrageError, ArbitrageResult};

pub struct TelegramService {
    // TODO: Implement telegram service
}

impl TelegramService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send_message(&self, _message: &str) -> ArbitrageResult<()> {
        // TODO: Implement telegram message sending
        Err(ArbitrageError::not_implemented("Telegram service not yet implemented"))
    }
} 