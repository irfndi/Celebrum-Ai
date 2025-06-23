//! Help Command Handler
//!
//! Handles the /help command to show available commands

use crate::core::bot_client::TelegramResult;
use crate::core::command_router::{CommandContext, CommandHandler, UserPermissions};
use async_trait::async_trait;
use worker::console_log;

pub struct HelpHandler;

impl HelpHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler for HelpHandler {
    async fn handle(
        &self,
        chat_id: i64,
        user_id: i64,
        _args: &[&str],
        context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!(
            "❓ Processing /help command for user {} in chat {}",
            user_id,
            chat_id
        );

        let mut help_message = String::from("📋 **ArbEdge Bot Commands**\n\n");

        // Basic commands
        help_message.push_str("**🔰 Basic Commands:**\n");
        help_message.push_str("/start - Start the bot and get welcome information\n");
        help_message.push_str("/help - Show this help message\n\n");

        // Trading commands
        help_message.push_str("**💰 Trading Commands:**\n");
        help_message.push_str("/opportunities - View current arbitrage opportunities\n");
        help_message.push_str("/balance - Check your portfolio balance\n");
        help_message.push_str("/trades - View your recent trades\n\n");

        // Settings commands
        help_message.push_str("**⚙️ Settings Commands:**\n");
        help_message.push_str("/settings - Configure your preferences\n");
        help_message.push_str("/notifications - Manage notification settings\n\n");

        // Admin commands (only show if user is admin)
        if context.user_permissions.is_admin {
            help_message.push_str("**👑 Admin Commands:**\n");
            help_message.push_str("/admin - Access admin panel\n");
            help_message.push_str("/stats - View bot statistics\n");
            help_message.push_str("/broadcast - Send message to all users\n\n");
        }

        // Premium commands (only show if user is premium)
        if context.user_permissions.is_premium {
            help_message.push_str("**⭐ Premium Commands:**\n");
            help_message.push_str("/alerts - Set up custom price alerts\n");
            help_message.push_str("/analytics - View detailed analytics\n\n");
        }

        help_message.push_str("💡 **Need more help?** Contact support or visit our documentation.");

        Ok(help_message)
    }

    fn command_name(&self) -> &'static str {
        "help"
    }

    fn help_text(&self) -> &'static str {
        "Show available commands and help information"
    }

    fn check_permission(&self, _user_permissions: &UserPermissions) -> bool {
        // Everyone can use /help
        true
    }
}

impl Default for HelpHandler {
    fn default() -> Self {
        Self::new()
    }
}
