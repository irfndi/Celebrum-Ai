//! Opportunities Command Handler
//!
//! Handles the /opportunities command to show arbitrage opportunities

use crate::core::command_router::{CommandHandler, CommandContext, UserPermissions};
use crate::core::bot_client::{TelegramResult, TelegramError};
use async_trait::async_trait;
use worker::console_log;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub base_currency: String,
    pub quote_currency: String,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub profit_amount: f64,
    pub volume_24h: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Default, Clone)]
pub struct OpportunityFilters {
    pub base_currency: Option<String>,
    pub quote_currency: Option<String>,
    pub min_profit_percentage: Option<f64>,
    pub max_profit_percentage: Option<f64>,
    pub min_volume: Option<f64>,
    pub exchanges: Option<Vec<String>>,
}

pub struct OpportunitiesHandler;

impl OpportunitiesHandler {
    pub fn new() -> Self {
        Self
    }

    /// Format opportunity data for display
    async fn get_opportunities(&self, filters: &OpportunityFilters) -> TelegramResult<Vec<ArbitrageOpportunity>> {
        // TODO: Replace with actual API call to arbitrage service
        // For now, return mock data that respects filters
        let mut opportunities = vec![
            ArbitrageOpportunity {
                base_currency: "BTC".to_string(),
                quote_currency: "USDT".to_string(),
                buy_exchange: "Binance".to_string(),
                sell_exchange: "Coinbase".to_string(),
                buy_price: 45000.0,
                sell_price: 46125.0,
                profit_percentage: 2.5,
                profit_amount: 1125.0,
                volume_24h: 50000.0,
                last_updated: chrono::Utc::now(),
            },
            ArbitrageOpportunity {
                base_currency: "ETH".to_string(),
                quote_currency: "USDT".to_string(),
                buy_exchange: "Kraken".to_string(),
                sell_exchange: "Binance".to_string(),
                buy_price: 2500.0,
                sell_price: 2546.75,
                profit_percentage: 1.87,
                profit_amount: 46.75,
                volume_24h: 25000.0,
                last_updated: chrono::Utc::now(),
            },
            ArbitrageOpportunity {
                base_currency: "BTC".to_string(),
                quote_currency: "USDT".to_string(),
                buy_exchange: "KuCoin".to_string(),
                sell_exchange: "FTX".to_string(),
                buy_price: 44800.0,
                sell_price: 47040.0,
                profit_percentage: 5.0,
                profit_amount: 2240.0,
                volume_24h: 30000.0,
                last_updated: chrono::Utc::now(),
            },
        ];

        // Apply filters
        if let Some(ref base_currency) = filters.base_currency {
            opportunities.retain(|opp| opp.base_currency == *base_currency);
        }

        if let Some(min_profit) = filters.min_profit_percentage {
            opportunities.retain(|opp| opp.profit_percentage >= min_profit);
        }

        Ok(opportunities)
    }

    async fn format_opportunities_response(
        &self,
        opportunities: &[ArbitrageOpportunity],
        invalid_args: &[&str],
    ) -> TelegramResult<String> {
        if opportunities.is_empty() {
            return Err(TelegramError::Api("No opportunities to format".to_string()));
        }

        let mut response = String::from("💰 **Current Arbitrage Opportunities**\n\n");
        
        // Show warning about invalid arguments if any
        if !invalid_args.is_empty() {
            response.push_str(&format!(
                "⚠️ **Note:** Unknown filters ignored: {}\n\n",
                invalid_args.join(", ")
            ));
        }
        
        for (i, opp) in opportunities.iter().enumerate().take(5) {
            response.push_str(&format!(
                "{}. **{}/{}**\n"
                "   📈 Buy: {} @ ${:.4}\n"
                "   📉 Sell: {} @ ${:.4}\n"
                "   💵 Profit: {:.2}% (${:.2})\n"
                "   📊 Volume: ${:.0}\n\n",
                i + 1,
                opp.base_currency,
                opp.quote_currency,
                opp.buy_exchange,
                opp.buy_price,
                opp.sell_exchange,
                opp.sell_price,
                opp.profit_percentage,
                opp.profit_amount,
                opp.volume_24h
            ));
        }

        if opportunities.len() > 5 {
            response.push_str(&format!("\n📋 Showing top 5 of {} opportunities.\n", opportunities.len()));
        }

        response.push_str("\n💡 **Usage Tips:**\n");
        response.push_str("• Use `btc`, `eth` to filter by currency\n");
        response.push_str("• Use `high`, `medium`, `low` for profit filters\n");
        response.push_str("• Use /balance to check your portfolio before trading!\n");
        response.push_str("• Use /help opportunities for more details");

        Ok(response)
    }
}

#[async_trait]
impl CommandHandler for OpportunitiesHandler {
    async fn handle(
        &self,
        chat_id: i64,
        user_id: i64,
        args: &[&str],
        _context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!("💰 Processing /opportunities command for user {} in chat {} with {} args", user_id, chat_id, args.len());

        // Validate arguments length
        if args.len() > 10 {
            console_log!("⚠️ Too many arguments ({}) provided by user {}", args.len(), user_id);
            return Ok("❌ Too many arguments. Use /help opportunities for usage information.".to_string());
        }

        // Parse optional filters from arguments with error handling
        let mut filters = OpportunityFilters::default();
        let mut invalid_args = Vec::new();
        
        for arg in args {
            match arg.to_lowercase().as_str() {
                "btc" | "bitcoin" => {
                    filters.base_currency = Some("BTC".to_string());
                    console_log!("🔍 Filter applied: base_currency=BTC by user {}", user_id);
                }
                "eth" | "ethereum" => {
                    filters.base_currency = Some("ETH".to_string());
                    console_log!("🔍 Filter applied: base_currency=ETH by user {}", user_id);
                }
                "high" => {
                    filters.min_profit_percentage = Some(5.0);
                    console_log!("🔍 Filter applied: min_profit=5.0% by user {}", user_id);
                }
                "medium" => {
                    filters.min_profit_percentage = Some(2.0);
                    console_log!("🔍 Filter applied: min_profit=2.0% by user {}", user_id);
                }
                "low" => {
                    filters.min_profit_percentage = Some(0.5);
                    console_log!("🔍 Filter applied: min_profit=0.5% by user {}", user_id);
                }
                _ => {
                    console_log!("⚠️ Unknown argument '{}' from user {}", arg, user_id);
                    invalid_args.push(*arg);
                }
            }
        }

        // Warn about invalid arguments but continue processing
        if !invalid_args.is_empty() {
            console_log!("⚠️ Invalid arguments ignored: {:?} from user {}", invalid_args, user_id);
        }

        console_log!("🔍 Final filters: base_currency={:?}, min_profit={:?} for user {}", 
                    filters.base_currency, filters.min_profit_percentage, user_id);

        // Fetch arbitrage opportunities with error handling
        let opportunities = match self.get_opportunities(&filters).await {
            Ok(opps) => {
                console_log!("✅ Successfully fetched {} opportunities for user {}", opps.len(), user_id);
                opps
            }
            Err(e) => {
                console_log!("❌ Failed to fetch opportunities for user {}: {:?}", user_id, e);
                return Err(TelegramError::Api("Failed to fetch arbitrage opportunities. Please try again later.".to_string()));
            }
        };

        // Handle empty results
        if opportunities.is_empty() {
            let empty_message = if !invalid_args.is_empty() {
                format!(
                    "📊 No arbitrage opportunities found matching your criteria.\n\n⚠️ Note: Unknown filters ignored: {}\n\nTry adjusting your filters or check back later!",
                    invalid_args.join(", ")
                )
            } else {
                "📊 No arbitrage opportunities found matching your criteria.\n\nTry adjusting your filters or check back later!".to_string()
            };
            return Ok(empty_message);
        }

        // Build response with error handling
        match self.format_opportunities_response(&opportunities, &invalid_args).await {
            Ok(response) => {
                console_log!("✅ Opportunities response formatted successfully for user {} ({} chars)", user_id, response.len());
                Ok(response)
            }
            Err(e) => {
                console_log!("❌ Failed to format opportunities response for user {}: {:?}", user_id, e);
                Err(TelegramError::Api("Failed to format opportunities data. Please try again.".to_string()));
            }
        }
    }

    fn command_name(&self) -> &'static str {
        "opportunities"
    }

    fn help_text(&self) -> &'static str {
        "View current arbitrage opportunities with optional filters"
    }

    fn check_permission(&self, _user_permissions: &UserPermissions) -> bool {
        // All registered users can view opportunities
        true
    }
}

impl Default for OpportunitiesHandler {
    fn default() -> Self {
        Self::new()
    }
}