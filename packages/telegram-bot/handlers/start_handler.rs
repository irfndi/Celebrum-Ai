//! Start Command Handler
//!
//! Handles the /start command for user onboarding

use crate::core::command_router::{CommandHandler, CommandContext, UserPermissions};
use crate::core::bot_client::TelegramResult;
use async_trait::async_trait;
use worker::console_log;

pub struct StartHandler;

impl StartHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler for StartHandler {
    async fn handle(
        &self,
        chat_id: i64,
        user_id: i64,
        _args: &[&str],
        _context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!("🚀 Processing /start command for user {} in chat {}", user_id, chat_id);

        let welcome_message = format!(
            "🎯 **Welcome to ArbEdge!**\n\n\
            🔍 Your gateway to cryptocurrency arbitrage opportunities\n\n\
            **Quick Start:**\n\
            • Use /opportunities to view current arbitrage opportunities\n\
            • Use /balance to check your portfolio\n\
            • Use /settings to configure your preferences\n\
            • Use /help to see all available commands\n\n\
            💡 **Tip:** Start by checking out /opportunities to see what's available!\n\n\
            🔐 Your user ID: `{}`",
            user_id
        );

        Ok(welcome_message)
    }

    fn command_name(&self) -> &'static str {
        "start"
    }

    fn help_text(&self) -> &'static str {
        "Start the bot and get welcome information"
    }

    fn check_permission(&self, _user_permissions: &UserPermissions) -> bool {
        // Everyone can use /start
        true
    }
}

impl Default for StartHandler {
    fn default() -> Self {
        Self::new()
    }
}