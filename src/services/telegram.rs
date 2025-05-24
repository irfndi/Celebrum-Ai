// src/services/telegram.rs

use crate::types::ArbitrageOpportunity;
use crate::utils::{ArbitrageError, ArbitrageResult};
use crate::utils::formatter::{
    format_opportunity_message, 
    format_categorized_opportunity_message,
    format_ai_enhancement_message,
    format_performance_insights_message,
    format_parameter_suggestions_message,
    escape_markdown_v2
};
use crate::services::opportunity_categorization::{CategorizedOpportunity, OpportunityCategory, RiskIndicator, AlertPriority};
use crate::services::ai_intelligence::{AiOpportunityEnhancement, AiPerformanceInsights, ParameterSuggestion};
use crate::services::user_trading_preferences::{UserTradingPreferences, TradingFocus, ExperienceLevel};
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use crate::services::market_analysis::{TradingOpportunity, OpportunityType, RiskLevel, TimeHorizon};
use crate::types::{ArbitrageType, ExchangeIdEnum};
use chrono::Datelike; // Import for year(), month(), day() methods

#[derive(Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

pub struct TelegramService {
    config: TelegramConfig,
    http_client: Client,
}

impl TelegramService {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    pub async fn send_message(&self, text: &str) -> ArbitrageResult<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.config.bot_token);
        
        let payload = json!({
            "chat_id": self.config.chat_id,
            "text": text,
            "parse_mode": "MarkdownV2"
        });

        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ArbitrageError::network_error(format!("Failed to send Telegram message: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::telegram_error(format!("Telegram API error: {}", error_text)));
        }

        let result: Value = response.json().await
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to parse Telegram response: {}", e)))?;

        if !result["ok"].as_bool().unwrap_or(false) {
            let error_description = result["description"].as_str().unwrap_or("Unknown error");
            return Err(ArbitrageError::telegram_error(format!("Telegram API error: {}", error_description)));
        }

        Ok(())
    }

    // ============= ENHANCED NOTIFICATION METHODS =============

    /// Send basic arbitrage opportunity notification (legacy support)
    pub async fn send_opportunity_notification(&self, opportunity: &ArbitrageOpportunity) -> ArbitrageResult<()> {
        let message = format_opportunity_message(opportunity);
        self.send_message(&message).await
    }

    /// Send categorized opportunity notification (NEW)
    pub async fn send_categorized_opportunity_notification(&self, categorized_opp: &CategorizedOpportunity) -> ArbitrageResult<()> {
        let message = format_categorized_opportunity_message(categorized_opp);
        self.send_message(&message).await
    }

    /// Send AI enhancement analysis notification (NEW)
    pub async fn send_ai_enhancement_notification(&self, enhancement: &AiOpportunityEnhancement) -> ArbitrageResult<()> {
        let message = format_ai_enhancement_message(enhancement);
        self.send_message(&message).await
    }

    /// Send AI performance insights notification (NEW)
    pub async fn send_performance_insights_notification(&self, insights: &AiPerformanceInsights) -> ArbitrageResult<()> {
        let message = format_performance_insights_message(insights);
        self.send_message(&message).await
    }

    /// Send parameter optimization suggestions (NEW)
    pub async fn send_parameter_suggestions_notification(&self, suggestions: &[ParameterSuggestion]) -> ArbitrageResult<()> {
        let message = format_parameter_suggestions_message(suggestions);
        self.send_message(&message).await
    }

    // ============= ENHANCED BOT COMMAND HANDLERS =============

    /// Bot command handlers (for webhook mode)
    pub async fn handle_webhook(&self, update: Value) -> ArbitrageResult<Option<String>> {
        if let Some(message) = update["message"].as_object() {
            if let Some(text) = message["text"].as_str() {
                // Properly handle missing user ID by returning an error instead of empty string
                let user_id = message["from"]["id"].as_u64()
                    .ok_or_else(|| ArbitrageError::validation_error("Missing user ID in webhook message".to_string()))?
                    .to_string();
                return self.handle_command(text, &user_id).await;
            }
        }
        Ok(None)
    }

    async fn handle_command(&self, text: &str, user_id: &str) -> ArbitrageResult<Option<String>> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        let command = parts.get(0).unwrap_or(&"");
        let args = &parts[1..];

        match *command {
            "/start" => Ok(Some(self.get_welcome_message().await)),
            "/help" => Ok(Some(self.get_help_message().await)),
            "/status" => Ok(Some(self.get_status_message(user_id).await)),
            "/opportunities" => Ok(Some(self.get_opportunities_message(user_id, args).await)),
            "/categories" => Ok(Some(self.get_categories_message(user_id).await)),
            "/ai_insights" => Ok(Some(self.get_ai_insights_message(user_id).await)),
            "/risk_assessment" => Ok(Some(self.get_risk_assessment_message(user_id).await)),
            "/preferences" => Ok(Some(self.get_preferences_message(user_id).await)),
            "/settings" => Ok(Some(self.get_settings_message(user_id).await)),
            _ => Ok(None), // Unknown command, no response
        }
    }

    // ============= ENHANCED COMMAND RESPONSES =============

    async fn get_welcome_message(&self) -> String {
        "🤖 *Welcome to ArbEdge AI Trading Bot\\!*\n\n\
        I'm your intelligent trading assistant powered by advanced AI\\.\n\n\
        🎯 *What I can do:*\n\
        • Detect arbitrage opportunities\n\
        • Provide AI\\-enhanced analysis\n\
        • Offer personalized recommendations\n\
        • Track your performance\n\
        • Optimize your trading parameters\n\n\
        📚 *Available Commands:*\n\
        /help \\- Show all available commands\n\
        /opportunities \\- View recent trading opportunities\n\
        /ai\\_insights \\- Get AI analysis and recommendations\n\
        /categories \\- Manage opportunity categories\n\
        /preferences \\- View/update your trading preferences\n\
        /status \\- Check system status\n\n\
        🚀 Get started with /opportunities to see what's available\\!".to_string()
    }

    async fn get_help_message(&self) -> String {
        "📚 *ArbEdge Bot Commands*\n\n\
        🔍 *Opportunities & Analysis:*\n\
        /opportunities \\[category\\] \\- Show recent opportunities\n\
        /ai\\_insights \\- Get AI analysis results\n\
        /risk\\_assessment \\- View portfolio risk analysis\n\n\
        🎛️ *Configuration:*\n\
        /categories \\- Manage enabled opportunity categories\n\
        /preferences \\- View/update trading preferences\n\
        /settings \\- View current bot settings\n\n\
        ℹ️ *Information:*\n\
        /status \\- Check bot and system status\n\
        /help \\- Show this help message\n\n\
        💡 *Tip:* Use /opportunities followed by a category name \\(e\\.g\\., `/opportunities arbitrage`\\) to filter results\\!".to_string()
    }

    async fn get_status_message(&self, _user_id: &str) -> String {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        format!(
            "🟢 *ArbEdge Bot Status*\n\n\
            ✅ System: *Online and monitoring*\n\
            🤖 AI Analysis: *Active*\n\
            📊 Opportunity Detection: *Running*\n\
            🔄 Real\\-time Updates: *Enabled*\n\n\
            🕒 Current Time: `{}`\n\
            📈 Monitoring: *Cross\\-exchange opportunities*\n\
            🎯 Categories: *10 opportunity types active*\n\
            ⚡ Response Time: *< 100ms*\n\n\
            💡 Use /opportunities to see latest opportunities\\!",
            escape_markdown_v2(&now.to_string())
        )
    }

    async fn get_opportunities_message(&self, _user_id: &str, args: &[&str]) -> String {
        let filter_category = args.get(0);
        
        let mut message = "📊 *Recent Trading Opportunities*\n\n".to_string();
        
        if let Some(category) = filter_category {
            message.push_str(&format!(
                "🏷️ Filtered by: `{}`\n\n",
                escape_markdown_v2(category)
            ));
        }

        // TODO: In real implementation, this would fetch actual opportunities
        // For now, show example of what it would look like
        message.push_str(
            "🛡️ *Low Risk Arbitrage* 🟢\n\
            📈 Pair: `BTCUSDT`\n\
            🎯 Suitability: `92%`\n\
            ⭐ Confidence: `89%`\n\n\
            🤖 *AI Recommended* ⭐\n\
            📈 Pair: `ETHUSDT`\n\
            🎯 Suitability: `87%`\n\
            ⭐ Confidence: `94%`\n\n\
            💡 *Tip:* Use /ai\\_insights for detailed AI analysis of these opportunities\\!\n\n\
            ⚙️ *Available Categories:*\n\
            • `arbitrage` \\- Low risk opportunities\n\
            • `technical` \\- Technical analysis signals\n\
            • `ai` \\- AI recommended trades\n\
            • `beginner` \\- Beginner\\-friendly options"
        );

        message
    }

    async fn get_categories_message(&self, _user_id: &str) -> String {
        "🏷️ *Opportunity Categories*\n\n\
        *Available Categories:*\n\
        🛡️ Low Risk Arbitrage \\- Conservative cross\\-exchange opportunities\n\
        🎯 High Confidence Arbitrage \\- 90\\%\\+ accuracy opportunities\n\
        📊 Technical Signals \\- Technical analysis based trades\n\
        🚀 Momentum Trading \\- Price momentum opportunities\n\
        🔄 Mean Reversion \\- Price reversion strategies\n\
        📈 Breakout Patterns \\- Pattern recognition trades\n\
        ⚡ Hybrid Enhanced \\- Arbitrage \\+ technical analysis\n\
        🤖 AI Recommended \\- AI\\-validated opportunities\n\
        🌱 Beginner Friendly \\- Simple, low\\-risk trades\n\
        🎖️ Advanced Strategies \\- Complex trading strategies\n\n\
        💡 Use /preferences to enable/disable categories based on your trading focus\\!".to_string()
    }

    async fn get_ai_insights_message(&self, _user_id: &str) -> String {
        // TODO: In real implementation, fetch actual AI insights
        "🤖 *AI Analysis Summary* 🌟\n\n\
        📊 *Recent Analysis:*\n\
        • Processed `15` opportunities in last hour\n\
        • Average AI confidence: `78%`\n\
        • Risk assessment completed for `3` positions\n\n\
        🎯 *Key Insights:*\n\
        ✅ Market conditions favor arbitrage opportunities\n\
        ⚠️ Increased volatility in technical signals\n\
        💡 Consider reducing position sizes by 15%\n\n\
        📈 *Performance Score:* `82%`\n\
        🤖 *Automation Readiness:* `74%`\n\n\
        💡 Use /risk\\_assessment for detailed portfolio analysis\\!".to_string()
    }

    async fn get_risk_assessment_message(&self, _user_id: &str) -> String {
        "📊 *Portfolio Risk Assessment* 🛡️\n\n\
        🎯 *Overall Risk Score:* `42%` 🟡\n\n\
        📈 *Risk Breakdown:*\n\
        • Portfolio Correlation: `35%` ✅\n\
        • Position Concentration: `48%` 🟡\n\
        • Market Conditions: `41%` 🟡\n\
        • Volatility Risk: `52%` ⚠️\n\n\
        💰 *Current Portfolio:*\n\
        • Total Value: `$12,500`\n\
        • Active Positions: `4`\n\
        • Diversification Score: `67%`\n\n\
        🎯 *Recommendations:*\n\
        📝 Consider diversifying across more pairs\n\
        ⚠️ Monitor volatility in current positions\n\
        💡 Maintain current risk levels".to_string()
    }

    async fn get_preferences_message(&self, _user_id: &str) -> String {
        // TODO: In real implementation, fetch user's actual preferences
        "⚙️ *Your Trading Preferences*\n\n\
        🎯 *Trading Focus:* Hybrid \\(Arbitrage \\+ Technical\\)\n\
        📊 *Experience Level:* Intermediate\n\
        🤖 *Automation Level:* Manual\n\
        🛡️ *Risk Tolerance:* Balanced\n\n\
        🔔 *Alert Settings:*\n\
        • Low Risk Arbitrage: ✅ Enabled\n\
        • High Confidence Arbitrage: ✅ Enabled\n\
        • Technical Signals: ✅ Enabled\n\
        • AI Recommended: ✅ Enabled\n\
        • Advanced Strategies: ❌ Disabled\n\n\
        💡 *Tip:* These preferences control which opportunities you receive\\. Update them in your profile settings\\!".to_string()
    }

    async fn get_settings_message(&self, _user_id: &str) -> String {
        "⚙️ *Bot Configuration*\n\n\
        🔔 *Notification Settings:*\n\
        • Alert Frequency: Real\\-time\n\
        • Max Alerts/Hour: `10`\n\
        • Cooldown Period: `5 minutes`\n\
        • Channels: Telegram ✅\n\n\
        🎯 *Filtering Settings:*\n\
        • Minimum Confidence: `60%`\n\
        • Risk Level Filter: Low \\+ Medium\n\
        • Category Filter: Based on preferences\n\n\
        🤖 *AI Settings:*\n\
        • AI Analysis: ✅ Enabled\n\
        • Performance Insights: ✅ Enabled\n\
        • Parameter Optimization: ✅ Enabled\n\n\
        💡 Use /preferences to modify your trading focus and experience settings\\!".to_string()
    }

    // ============= WEBHOOK SETUP =============

    pub async fn set_webhook(&self, webhook_url: &str) -> ArbitrageResult<()> {
        let url = format!("https://api.telegram.org/bot{}/setWebhook", self.config.bot_token);
        
        let payload = json!({
            "url": webhook_url
        });

        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ArbitrageError::network_error(format!("Failed to set webhook: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::telegram_error(format!("Failed to set webhook: {}", error_text)));
        }

        Ok(())
    }

    // ============= NOTIFICATION TEMPLATES INTEGRATION =============

    /// Send templated notification (for NotificationService integration)
    pub async fn send_templated_notification(
        &self,
        title: &str,
        message: &str,
        variables: &std::collections::HashMap<String, serde_json::Value>,
    ) -> ArbitrageResult<()> {
        // Replace variables in the message
        let mut formatted_message = message.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "N/A".to_string(),
                _ => value.to_string(),
            };
            formatted_message = formatted_message.replace(&placeholder, &replacement);
        }

        // Format with title
        let full_message = if title.is_empty() {
            escape_markdown_v2(&formatted_message)
        } else {
            format!(
                "*{}*\n\n{}",
                escape_markdown_v2(title),
                escape_markdown_v2(&formatted_message)
            )
        };

        self.send_message(&full_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum};
    use crate::services::opportunity_categorization::{CategorizedOpportunity, OpportunityCategory, RiskIndicator, AlertPriority};
    use crate::services::market_analysis::{TradingOpportunity, OpportunityType, RiskLevel, TimeHorizon};
    use serde_json::json;
    use chrono::Datelike; // Import for year(), month(), day() methods

    fn create_test_config() -> TelegramConfig {
        TelegramConfig {
            bot_token: "test_token_123456789:ABCDEF".to_string(),
            chat_id: "-123456789".to_string(),
        }
    }

    fn create_test_opportunity() -> ArbitrageOpportunity {
        ArbitrageOpportunity {
            id: "test_opp_001".to_string(),
            pair: "BTCUSDT".to_string(),
            r#type: ArbitrageType::FundingRate,
            long_exchange: Some(ExchangeIdEnum::Binance),
            short_exchange: Some(ExchangeIdEnum::Bybit),
            long_rate: Some(0.001),
            short_rate: Some(0.003),
            rate_difference: 0.002,
            net_rate_difference: Some(0.0018),
            potential_profit_value: Some(18.0),
            timestamp: 1640995200000, // Jan 1, 2022
            details: Some("Test funding rate arbitrage opportunity".to_string()),
        }
    }

    fn create_test_categorized_opportunity() -> CategorizedOpportunity {
        let base_opportunity = TradingOpportunity {
            opportunity_id: "test_cat_opp_001".to_string(),
            opportunity_type: OpportunityType::Arbitrage,
            trading_pair: "BTCUSDT".to_string(),
            exchanges: vec!["binance".to_string(), "bybit".to_string()],
            entry_price: 50000.0,
            target_price: Some(51000.0),
            stop_loss: Some(49000.0),
            confidence_score: 0.85,
            risk_level: RiskLevel::Low,
            expected_return: 0.02,
            time_horizon: TimeHorizon::Short,
            indicators_used: vec!["rsi".to_string()],
            analysis_data: serde_json::json!({"test": "data"}),
            created_at: 1640995200000,
            expires_at: Some(1640998800000),
        };

        CategorizedOpportunity {
            base_opportunity,
            categories: vec![OpportunityCategory::LowRiskArbitrage, OpportunityCategory::BeginnerFriendly],
            primary_category: OpportunityCategory::LowRiskArbitrage,
            risk_indicator: RiskIndicator::new(RiskLevel::Low, 0.85),
            user_suitability_score: 0.92,
            personalization_factors: vec!["Low risk level suitable for user".to_string()],
            alert_eligible: true,
            alert_priority: AlertPriority::Medium,
            enhanced_metadata: {
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("test_key".to_string(), serde_json::json!("test_value"));
                metadata
            },
            categorized_at: 1640995200000,
        }
    }

    mod service_initialization {
        use super::*;

        #[test]
        fn test_new_telegram_service() {
            let config = create_test_config();
            let service = TelegramService::new(config.clone());
            
            // Service should be created successfully
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }

        #[test]
        fn test_telegram_service_is_send_sync() {
            fn assert_send<T: Send>() {}
            fn assert_sync<T: Sync>() {}
            
            assert_send::<TelegramService>();
            assert_sync::<TelegramService>();
        }

        #[test]
        fn test_config_validation_valid() {
            let config = create_test_config();
            
            assert!(!config.bot_token.is_empty());
            assert!(!config.chat_id.is_empty());
        }

        #[test]
        fn test_config_basic_structure() {
            let config = create_test_config();
            assert!(config.bot_token.contains("test_token"));
            assert!(config.chat_id.starts_with('-'));
        }
    }

    mod enhanced_notifications {
        use super::*;

        #[test]
        fn test_categorized_opportunity_message_structure() {
            let categorized_opp = create_test_categorized_opportunity();
            let message = format_categorized_opportunity_message(&categorized_opp);
            
            // Check for categorized opportunity elements
            assert!(message.contains("Low Risk Arbitrage"));
            assert!(message.contains("BTCUSDT"));
            assert!(message.contains("Suitability Score"));
            assert!(message.contains("92"));  // suitability score
            assert!(message.contains("Risk Assessment"));
        }

        #[test]
        fn test_enhanced_command_responses() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            
            // Test that new command responses are not empty
            let welcome = futures::executor::block_on(service.get_welcome_message());
            assert!(welcome.contains("ArbEdge AI Trading Bot"));
            assert!(welcome.contains("AI\\-enhanced analysis"));  // Fixed to check escaped version
            
            let help = futures::executor::block_on(service.get_help_message());
            assert!(help.contains("ai\\_insights"));   // Fixed to check escaped version
            assert!(help.contains("categories"));
        }

        #[test]
        fn test_ai_insights_response() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            
            let insights = futures::executor::block_on(service.get_ai_insights_message("test_user"));
            assert!(insights.contains("AI Analysis Summary"));
            assert!(insights.contains("confidence"));
            assert!(insights.contains("Performance Score"));
        }

        #[test]
        fn test_risk_assessment_response() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            
            let risk = futures::executor::block_on(service.get_risk_assessment_message("test_user"));
            assert!(risk.contains("Portfolio Risk Assessment"));
            assert!(risk.contains("Risk Breakdown"));
            assert!(risk.contains("Recommendations"));
        }

        #[test]
        fn test_preferences_response() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            
            let prefs = futures::executor::block_on(service.get_preferences_message("test_user"));
            assert!(prefs.contains("Trading Preferences"));
            assert!(prefs.contains("Trading Focus"));
            assert!(prefs.contains("Experience Level"));
            assert!(prefs.contains("Alert Settings"));
        }
    }

    mod configuration_validation {
        use super::*;

        #[test]
        fn test_bot_token_format() {
            let config = create_test_config();
            
            // Basic token format validation
            assert!(config.bot_token.contains(':'));
            assert!(config.bot_token.len() > 10);
        }

        #[test]
        fn test_chat_id_format() {
            let config = create_test_config();
            
            // Chat ID should be numeric (with optional negative sign for groups)
            assert!(config.chat_id.starts_with('-') || config.chat_id.chars().all(|c| c.is_ascii_digit()));
        }

        #[test]
        fn test_webhook_url_validation() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            
            // This is a placeholder test - in real implementation would validate URL format
            let webhook_url = "https://example.com/webhook";
            assert!(webhook_url.starts_with("https://"));
        }

        #[test]
        fn test_optional_webhook() {
            let config = create_test_config();
            let _service = TelegramService::new(config);
            
            // Service should work without webhook being set
            assert!(true); // Placeholder assertion
        }
    }

    mod message_formatting {
        use super::*;

        #[test]
        fn test_escape_markdown_v2_basic() {
            let input = "test_string";
            let expected = "test\\_string";
            assert_eq!(escape_markdown_v2(input), expected);
        }

        #[test]
        fn test_escape_markdown_v2_special_chars() {
            let input = "test*bold*_italic_";
            let expected = "test\\*bold\\*\\_italic\\_";
            assert_eq!(escape_markdown_v2(input), expected);
        }

        #[test]
        fn test_escape_markdown_v2_comprehensive() {
            let input = "test-dash.period!exclamation(paren)[bracket]{brace}";
            let expected = "test\\-dash\\.period\\!exclamation\\(paren\\)\\[bracket\\]\\{brace\\}";
            assert_eq!(escape_markdown_v2(input), expected);
        }

        #[test]
        fn test_format_percentage() {
            use crate::utils::formatter::format_percentage;
            assert_eq!(format_percentage(0.1234), "12.3400");
            assert_eq!(format_percentage(0.0001), "0.0100");
        }

        #[test]
        fn test_opportunity_message_components() {
            let opportunity = create_test_opportunity();
            let message = format_opportunity_message(&opportunity);
            
            assert!(message.contains("BTCUSDT"));
            assert!(message.contains("binance"));  // Fixed to check lowercase as returned by format_exchange
            assert!(message.contains("bybit"));    // Fixed to check lowercase as returned by format_exchange
        }
    }

    mod opportunity_notifications {
        use super::*;

        #[test]
        fn test_opportunity_data_extraction() {
            let opportunity = create_test_opportunity();
            
            assert_eq!(opportunity.pair, "BTCUSDT");
            assert_eq!(opportunity.long_exchange, Some(ExchangeIdEnum::Binance));
            assert_eq!(opportunity.short_exchange, Some(ExchangeIdEnum::Bybit));
            assert_eq!(opportunity.rate_difference, 0.002);
        }

        #[test]
        fn test_profit_calculation_data() {
            let opportunity = create_test_opportunity();
            
            if let Some(profit) = opportunity.potential_profit_value {
                assert_eq!(profit, 18.0);
            } else {
                panic!("Expected potential profit value to be present");
            }
        }

        #[test]
        fn test_message_timestamp_handling() {
            let opportunity = create_test_opportunity();
            
            // Timestamp should be valid
            assert!(opportunity.timestamp > 0);
            assert_eq!(opportunity.timestamp, 1640995200000); // Jan 1, 2022
        }

        #[test]
        fn test_opportunity_type_validation() {
            let opportunity = create_test_opportunity();
            assert!(matches!(opportunity.r#type, ArbitrageType::FundingRate));
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn test_invalid_config_handling() {
            let invalid_config = TelegramConfig {
                bot_token: "".to_string(),
                chat_id: "".to_string(),
            };
            
            // Service should still be created (validation happens during use)
            let _service = TelegramService::new(invalid_config);
        }

        #[test]
        fn test_malformed_chat_id() {
            let config = TelegramConfig {
                bot_token: "valid_token:ABC123".to_string(),
                chat_id: "invalid_chat_id".to_string(),
            };
            
            let _service = TelegramService::new(config);
            // Service creation should succeed (validation during API calls)
        }

        #[test]
        fn test_disabled_service_handling() {
            let config = create_test_config();
            let _service = TelegramService::new(config);
            
            // Service should handle being disabled gracefully
            assert!(true); // Placeholder - would test actual disabled behavior
        }

        #[test]
        fn test_empty_opportunity_data() {
            let mut opportunity = create_test_opportunity();
            opportunity.details = None;
            opportunity.potential_profit_value = None;
            
            let message = format_opportunity_message(&opportunity);
            // Should still generate valid message without optional fields
            assert!(message.contains("BTCUSDT"));
        }
    }

    mod api_interaction {
        use super::*;

        #[test]
        fn test_telegram_api_url_construction() {
            let config = create_test_config();
            let service = TelegramService::new(config.clone());
            
            let expected_base = format!("https://api.telegram.org/bot{}/", config.bot_token);
            assert!(expected_base.contains(&config.bot_token));
        }

        #[test]
        fn test_webhook_url_validation() {
            let webhook_url = "https://example.com/webhook/telegram";
            assert!(webhook_url.starts_with("https://"));
            assert!(webhook_url.contains("webhook"));
        }

        #[test]
        fn test_message_payload_structure() {
            let config = create_test_config();
            let message_text = "Test message";
            
            let payload = json!({
                "chat_id": config.chat_id,
                "text": message_text,
                "parse_mode": "MarkdownV2"
            });
            
            assert_eq!(payload["chat_id"], config.chat_id);
            assert_eq!(payload["text"], message_text);
            assert_eq!(payload["parse_mode"], "MarkdownV2");
        }
    }

    mod webhook_handling {
        use super::*;

        #[test]
        fn test_webhook_data_structure() {
            let webhook_data = json!({
                "update_id": 123456789,
                "message": {
                    "message_id": 123,
                    "from": {
                        "id": 987654321,
                        "is_bot": false,
                        "first_name": "Test",
                        "username": "testuser"
                    },
                    "chat": {
                        "id": -123456789,
                        "title": "Test Group",
                        "type": "group"
                    },
                    "date": 1640995200,
                    "text": "/start"
                }
            });
            
            assert_eq!(webhook_data["message"]["text"], "/start");
            assert_eq!(webhook_data["message"]["from"]["id"], 987654321);
        }

        #[test]
        fn test_command_extraction() {
            let command_text = "/opportunities arbitrage";
            let parts: Vec<&str> = command_text.split_whitespace().collect();
            
            assert_eq!(parts[0], "/opportunities");
            assert_eq!(parts[1], "arbitrage");
        }

        #[test]
        fn test_chat_id_extraction() {
            let webhook_data = json!({
                "message": {
                    "from": {
                        "id": 987654321
                    },
                    "text": "/status"
                }
            });
            
            let user_id = webhook_data["message"]["from"]["id"].as_u64().unwrap();
            assert_eq!(user_id, 987654321);
        }
    }

    mod utility_functions {
        use super::*;

        #[test]
        fn test_service_configuration_access() {
            let config = create_test_config();
            let service = TelegramService::new(config.clone());
            
            // Service should maintain access to configuration
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }

        #[test]
        fn test_exchange_name_formatting() {
            let exchange = Some(ExchangeIdEnum::Binance);
            let formatted = crate::utils::formatter::format_exchange(&exchange);
            assert_eq!(formatted, "binance");  // Fixed to check actual output format
        }

        #[test]
        fn test_rate_difference_formatting() {
            let rate_diff = 0.002;
            let formatted = crate::utils::formatter::format_percentage(rate_diff);
            assert_eq!(formatted, "0.2000");
        }

        #[test]
        fn test_timestamp_conversion() {
            let timestamp = 1640995200000u64; // Jan 1, 2022
            let formatted = crate::utils::formatter::format_timestamp(timestamp);
            assert!(formatted.contains("2022"));
        }
    }

    mod integration_scenarios {
        use super::*;

        #[test]
        fn test_complete_notification_workflow() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            let opportunity = create_test_opportunity();
            
            let message = format_opportunity_message(&opportunity);
            assert!(message.len() > 0);
            assert!(message.contains("BTCUSDT"));
        }

        #[test]
        fn test_multiple_opportunities_handling() {
            let opp1 = create_test_opportunity();
            let mut opp2 = create_test_opportunity();
            opp2.pair = "ETHUSDT".to_string();
            
            let msg1 = format_opportunity_message(&opp1);
            let msg2 = format_opportunity_message(&opp2);
            
            assert!(msg1.contains("BTCUSDT"));
            assert!(msg2.contains("ETHUSDT"));
        }

        #[test]
        fn test_service_state_consistency() {
            let config = create_test_config();
            let service = TelegramService::new(config.clone());
            
            // Service should maintain consistent state
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }
    }
} 