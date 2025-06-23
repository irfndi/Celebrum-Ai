//! Balance Command Handler
//!
//! Handles the /balance command to show user portfolio balance

use crate::core::command_router::{CommandHandler, CommandContext, UserPermissions};
use crate::core::bot_client::TelegramResult;
use async_trait::async_trait;
use worker::console_log;

pub struct BalanceHandler;

impl BalanceHandler {
    pub fn new() -> Self {
        Self
    }

    /// Format balance information for display
    fn format_balance_item(&self, symbol: &str, amount: f64, usd_value: f64, change_24h: f64) -> String {
        let change_emoji = if change_24h >= 0.0 { "📈" } else { "📉" };
        let change_sign = if change_24h >= 0.0 { "+" } else { "" };
        
        format!(
            "💰 **{}**: {:.6}\n\
            💵 ${:.2} {} {}{}%\n",
            symbol, amount, usd_value, change_emoji, change_sign, change_24h
        )
    }
}

#[async_trait]
impl CommandHandler for BalanceHandler {
    async fn handle(
        &self,
        chat_id: i64,
        user_id: i64,
        args: &[&str],
        context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!("💳 Processing /balance command for user {} in chat {}", user_id, chat_id);

        // Parse optional currency filter from arguments
        let filter_currency = args.get(0).map(|s| s.to_uppercase());
        
        let mut response = String::from("💼 **Your Portfolio Balance**\n\n");
        
        // TODO: Replace with actual API call to get user balance
        // For now, return mock data
        let mock_balances = vec![
            ("BTC", 0.025847, 1247.83, 2.45),
            ("ETH", 1.847392, 2891.47, -1.23),
            ("USDT", 5000.0, 5000.0, 0.01),
            ("ADA", 2847.39, 1423.69, 5.67),
            ("SOL", 12.847, 1156.23, -3.21),
        ];
        
        let mut total_usd_value = 0.0;
        let mut displayed_balances = 0;
        
        for (symbol, amount, usd_value, change_24h) in &mock_balances {
            // Apply currency filter if specified
            if let Some(ref filter) = filter_currency {
                if symbol != filter {
                    continue;
                }
            }
            
            // Only show balances with significant amounts (> $1)
            if *usd_value > 1.0 {
                response.push_str(&self.format_balance_item(symbol, *amount, *usd_value, *change_24h));
                response.push('\n');
                total_usd_value += usd_value;
                displayed_balances += 1;
            }
        }
        
        if displayed_balances == 0 {
            if filter_currency.is_some() {
                response.push_str(&format!(
                    "❌ No {} balance found or balance is too small to display.\n\n",
                    filter_currency.unwrap()
                ));
                response.push_str("💡 Try /balance without filters to see all balances.");
            } else {
                response.push_str("📭 No significant balances found.\n\n");
                response.push_str("💡 Start trading to build your portfolio!");
            }
        } else {
            // Add total portfolio value
            response.push_str(&format!(
                "📊 **Total Portfolio Value:** ${:.2}\n\n",
                total_usd_value
            ));
            
            // Add performance summary
            let total_change = mock_balances.iter()
                .map(|(_, _, value, change)| (value * change) / 100.0)
                .sum::<f64>();
            
            let change_emoji = if total_change >= 0.0 { "📈" } else { "📉" };
            let change_sign = if total_change >= 0.0 { "+" } else { "" };
            
            response.push_str(&format!(
                "📈 **24h Change:** {} {}${:.2}\n\n",
                change_emoji, change_sign, total_change
            ));
            
            // Add additional features for premium users
            if context.user_permissions.is_premium {
                response.push_str("⭐ **Premium Features:**\n");
                response.push_str("• Use /analytics for detailed portfolio analysis\n");
                response.push_str("• Use /trades to view trading history\n");
                response.push_str("• Use /alerts to set balance alerts");
            } else {
                response.push_str("💎 Upgrade to Premium for detailed analytics and alerts!");
            }
        }
        
        response.push_str("\n\n💡 **Usage:** `/balance [currency]`\n");
        response.push_str("Example: `/balance BTC` (show only BTC balance)");
        
        Ok(response)
    }

    fn command_name(&self) -> &'static str {
        "balance"
    }

    fn help_text(&self) -> &'static str {
        "Check your portfolio balance and performance"
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