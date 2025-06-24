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
            Ok(balance_data) => {
                let total_balance = balance_data["total_balance"].as_str().unwrap_or("0.00");
                let available_balance = balance_data["available_balance"].as_str().unwrap_or("0.00");
                let currency = balance_data["currency"].as_str().unwrap_or("USD");
                let last_updated = balance_data["last_updated"].as_str().unwrap_or("Unknown");
                
                format!(
                    "💰 **Account Balance**\n\n\
                    - *Total Balance:* {} {}\n\
                    - *Available:* {} {}\n\
                    - *Currency:* {}\n\
                    - *Last Updated:* {}\n\n\
                    💡 Use /opportunities to find new trades!",
                    total_balance, currency,
                    available_balance, currency,
                    currency,
                    last_updated
                )
            },
            Err(e) => {
                console_log!("❌ Failed to get balance for user {}: {:?}", user_id, e);
                "❌ Unable to retrieve your balance information. Please try again later."
                    .to_string()
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
