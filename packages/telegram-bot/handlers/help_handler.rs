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

    /// Generate the help message text, potentially customized by user permissions.
    fn generate_help_text(&self, context: &CommandContext) -> String {
        let mut help_text = "🤖 *ArbEdge Bot Commands*\n\n".to_string();

        // Standard commands available to all users
        help_text.push_str("🚀 `/start` - Welcome message and quick start\n");
        help_text.push_str("📊 `/opportunities [filter]` - View arbitrage opportunities\n");
        help_text.push_str("👤 `/profile` - View and manage your profile\n");
        help_text.push_str("💰 `/balance` - Check account balance and P&L\n");
        help_text.push_str("⚙️ `/settings` - Configure trading preferences\n");
        help_text.push_str("❓ `/help` - Show this help message\n");

        // Admin-only commands
        if context.user_permissions.is_admin {
            help_text.push_str("\n*👑 Admin Commands*\n");
            help_text.push_str("`/admin [action]` - Access admin functions\n");
        }

        help_text.push_str(
            "\n💡 *Pro tip:* Use `/opportunities high` to see only high-profit opportunities!",
        );

        help_text
    }
}

#[async_trait]
impl CommandHandler for HelpHandler {
    async fn handle(
        &self,
        _chat_id: i64,
        user_id: i64,
        _args: &[&str],
        context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!("❓ Processing /help command for user {}", user_id);

        let help_text = self.generate_help_text(context);
        Ok(help_text)
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
