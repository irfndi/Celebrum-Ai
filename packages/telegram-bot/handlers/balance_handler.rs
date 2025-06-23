//! Balance Command Handler
//! 
//! Handles the /balance command to show a user's account balance and P&L.

use crate::core::bot_client::TelegramResult;
use crate::core::command_router::{CommandContext, CommandHandler, UserPermissions};
use crate::integrations::get_user_balance;
use async_trait::async_trait;
use worker::console_log;

pub struct BalanceHandler;

impl BalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler for BalanceHandler {
    async fn handle(
        &self,
        _chat_id: i64,
        user_id: i64,
        _args: &[&str],
        _context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!("💰 Processing /balance command for user {}", user_id);

        let user_id_str = user_id.to_string();

        let response_text = match get_user_balance(&user_id_str).await {
            Ok(balance_data) => format!(
                "💰 *Your Balance*\n\n\
                - *Account Balance:* ${:.2}\n\
                - *Total P&L:* ${:.2}\n\
                - *Total Trades:* {}\n\
                - *Win Rate:* {:.1}%\n\
                - *Risk Level:* {}\n\n\
                💡 Ready to find new opportunities? Try `/opportunities`",
                balance_data.account_balance_usdt,
                balance_data.total_pnl_usdt,
                balance_data.total_trades,
                balance_data.win_rate,
                balance_data.risk_level
            ),
            Err(e) => {
                console_log!("❌ Failed to get balance for user {}: {:?}", user_id, e);
                "❌ Unable to retrieve your balance information. Please try again later.".to_string()
            }
        };

        Ok(response_text)
    }

    fn command_name(&self) -> &'static str {
        "balance"
    }

    fn help_text(&self) -> &'static str {
        "Check your account balance and P&L"
    }

    fn check_permission(&self, _user_permissions: &UserPermissions) -> bool {
        // All registered users can check their balance
        true
    }
}

impl Default for BalanceHandler {
    fn default() -> Self {
        Self::new()
    }
}
