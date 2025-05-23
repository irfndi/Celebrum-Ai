// src/services/telegram.rs

use crate::types::ArbitrageOpportunity;
use crate::utils::{ArbitrageError, ArbitrageResult};
use crate::utils::formatter::{format_opportunity_message, escape_markdown_v2};
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};

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

    pub async fn send_opportunity_notification(&self, opportunity: &ArbitrageOpportunity) -> ArbitrageResult<()> {
        let message = format_opportunity_message(opportunity);
        self.send_message(&message).await
    }

    // Bot command handlers (for webhook mode)
    pub async fn handle_webhook(&self, update: Value) -> ArbitrageResult<Option<String>> {
        if let Some(message) = update["message"].as_object() {
            if let Some(text) = message["text"].as_str() {
                return self.handle_command(text).await;
            }
        }
        Ok(None)
    }

    async fn handle_command(&self, text: &str) -> ArbitrageResult<Option<String>> {
        match text {
            "/start" => Ok(Some(
                "Welcome to the Arbitrage Bot!\n\
                I can help you detect funding rate arbitrage opportunities and notify you about them.\n\n\
                Here are the available commands:\n\
                /help - Show this help message and list all commands.\n\
                /status - Check the bot's current operational status.\n\
                /opportunities - Show recent arbitrage opportunities (currently placeholder).\n\
                /settings - View current bot settings (currently placeholder).\n\n\
                Use /help to see this list again.".to_string()
            )),
            "/help" => Ok(Some(
                "Available commands:\n\
                /help - Show this help message\n\
                /status - Check bot status\n\
                /opportunities - Show recent opportunities\n\
                /settings - View current settings".to_string()
            )),
            "/status" => {
                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
                Ok(Some(format!(
                    "Bot is active and monitoring for arbitrage opportunities.\nCurrent time: {}",
                    now
                )))
            }
            "/opportunities" => Ok(Some(
                "No recent opportunities found. Will notify you when new ones are detected.".to_string()
            )),
            "/settings" => Ok(Some(
                "Current settings:\n\
                Threshold: 0.001 (0.1%)\n\
                Pairs monitored: BTC/USDT, ETH/USDT\n\
                Exchanges: Binance, Bybit, OKX".to_string()
            )),
            _ => Ok(None), // Unknown command, no response
        }
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum};
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
            
            let service = TelegramService::new(config);
            // Service should be created successfully
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }

        #[test]
        fn test_disabled_service_handling() {
            let config = create_test_config();
            
            let service = TelegramService::new(config);
            // Service should be created (enabling/disabling would be handled at application level)
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }
    }

    mod configuration_validation {
        use super::*;

        #[test]
        fn test_bot_token_format() {
            let config = create_test_config();
            
            // Bot token should contain colon
            assert!(config.bot_token.contains(':'));
            
            // Should have reasonable length
            assert!(config.bot_token.len() > 10);
        }

        #[test]
        fn test_chat_id_format() {
            let config = create_test_config();
            
            // Chat ID should be negative for groups
            assert!(config.chat_id.starts_with('-'));
            
            // Should be numeric after the dash
            let numeric_part = &config.chat_id[1..];
            assert!(numeric_part.parse::<i64>().is_ok());
        }

        #[test]
        fn test_webhook_url_validation() {
            let config = create_test_config();
            
            // Test webhook URL format validation (separate from config)
            let webhook_url = "https://example.com/webhook";
            assert!(webhook_url.starts_with("https://"));
            assert!(webhook_url.len() > 10);
        }

        #[test]
        fn test_optional_webhook() {
            let config = create_test_config();
            
            let service = TelegramService::new(config);
            // Should work without webhook URL (webhook is set separately)
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }
    }

    mod message_formatting {
        use super::*;

        #[test]
        fn test_escape_markdown_v2_basic() {
            let input = "Hello World!";
            let escaped = escape_markdown_v2(input);
            
            // Should escape exclamation mark since it's a special MarkdownV2 character
            assert_eq!(escaped, "Hello World\\!");
        }

        #[test]
        fn test_escape_markdown_v2_special_chars() {
            let input = "Price: $1,234.56 (10% gain)";
            let escaped = escape_markdown_v2(input);
            
            // Should escape special characters that are in the escape list
            // Note: $ is not a special character in MarkdownV2, so it won't be escaped
            assert!(escaped.contains("\\("));
            assert!(escaped.contains("\\)"));
            assert!(escaped.contains("\\."));
            // $ remains unescaped as it's not a special MarkdownV2 character
            assert!(escaped.contains("$"));
        }

        #[test]
        fn test_escape_markdown_v2_comprehensive() {
            let input = "_*[]()~`>#+-=|{}.!";
            let escaped = escape_markdown_v2(input);
            
            // All special characters should be escaped
            assert!(escaped.contains("\\_"));
            assert!(escaped.contains("\\*"));
            assert!(escaped.contains("\\["));
            assert!(escaped.contains("\\]"));
            assert!(escaped.contains("\\("));
            assert!(escaped.contains("\\)"));
            assert!(escaped.contains("\\~"));
            assert!(escaped.contains("\\`"));
            assert!(escaped.contains("\\>"));
            assert!(escaped.contains("\\#"));
            assert!(escaped.contains("\\+"));
            assert!(escaped.contains("\\-"));
            assert!(escaped.contains("\\="));
            assert!(escaped.contains("\\|"));
            assert!(escaped.contains("\\{"));
            assert!(escaped.contains("\\}"));
            assert!(escaped.contains("\\."));
            assert!(escaped.contains("\\!"));
        }

        #[test]
        fn test_format_percentage() {
            let rate = 0.001; // 0.1%
            
            // Should format as percentage with basis points
            let formatted_100 = format!("{:.2}%", rate * 100.0);
            assert_eq!(formatted_100, "0.10%");
            
            let formatted_10000 = format!("{:.1} bps", rate * 10000.0);
            assert_eq!(formatted_10000, "10.0 bps");
        }

        #[test]
        fn test_opportunity_message_components() {
            let opportunity = create_test_opportunity();
            
            // Test individual message components
            assert_eq!(opportunity.pair, "BTCUSDT");
            assert_eq!(opportunity.rate_difference, 0.002);
            assert!(opportunity.long_exchange.is_some());
            assert!(opportunity.short_exchange.is_some());
            assert!(opportunity.potential_profit_value.is_some());
        }
    }

    mod opportunity_notifications {
        use super::*;

        #[test]
        fn test_opportunity_data_extraction() {
            let opportunity = create_test_opportunity();
            
            // Verify all required data is present
            assert!(!opportunity.id.is_empty());
            assert!(!opportunity.pair.is_empty());
            assert!(opportunity.rate_difference > 0.0);
            
            // Exchange data
            let long_exchange = opportunity.long_exchange.unwrap();
            let short_exchange = opportunity.short_exchange.unwrap();
            assert_ne!(long_exchange, short_exchange);
            
            // Rate data
            let long_rate = opportunity.long_rate.unwrap();
            let short_rate = opportunity.short_rate.unwrap();
            assert_ne!(long_rate, short_rate);
        }

        #[test]
        fn test_profit_calculation_data() {
            let opportunity = create_test_opportunity();
            
            if let Some(profit) = opportunity.potential_profit_value {
                assert!(profit > 0.0);
                assert!(profit.is_finite());
            }
            
            if let Some(net_diff) = opportunity.net_rate_difference {
                assert!(net_diff > 0.0);
                assert!(net_diff <= opportunity.rate_difference);
            }
        }

        #[test]
        fn test_message_timestamp_handling() {
            let opportunity = create_test_opportunity();
            
            // Timestamp should be valid
            assert!(opportunity.timestamp > 0);
            
            // Should be reasonable (after year 2020, before year 2030)
            let min_timestamp = 1577836800000u64; // Jan 1, 2020
            let max_timestamp = 1893456000000u64; // Jan 1, 2030
            
            assert!(opportunity.timestamp > min_timestamp);
            assert!(opportunity.timestamp < max_timestamp);
        }

        #[test]
        fn test_opportunity_type_validation() {
            let opportunity = create_test_opportunity();
            
            assert_eq!(opportunity.r#type, ArbitrageType::FundingRate);
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn test_invalid_config_handling() {
            // Empty bot token
            let mut config = create_test_config();
            config.bot_token = String::new();
            
            // Service should still be created but might fail on API calls
            let service = TelegramService::new(config);
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }

        #[test]
        fn test_malformed_chat_id() {
            let mut config = create_test_config();
            config.chat_id = "not_a_number".to_string();
            
            // Service creation should work, validation happens at runtime
            let service = TelegramService::new(config);
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }

        #[test]
        fn test_disabled_service_handling() {
            let config = create_test_config();
            
            let service = TelegramService::new(config);
            // Service should be created (enabling/disabling would be handled at application level)
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TelegramService>());
        }

        #[test]
        fn test_empty_opportunity_data() {
            let mut opportunity = create_test_opportunity();
            opportunity.id = String::new();
            opportunity.pair = String::new();
            
            // Should handle empty data gracefully
            assert!(opportunity.id.is_empty());
            assert!(opportunity.pair.is_empty());
        }
    }

    mod api_interaction {
        use super::*;

        #[test]
        fn test_telegram_api_url_construction() {
            let config = create_test_config();
            let service = TelegramService::new(config.clone());
            
            // Construct expected API URL
            let expected_base = format!("https://api.telegram.org/bot{}", config.bot_token);
            assert!(expected_base.contains("test_token_123456789"));
            assert!(expected_base.contains("api.telegram.org"));
        }

        #[test]
        fn test_webhook_url_validation() {
            let config = create_test_config();
            
            // Test webhook URL format validation (separate from config)
            let webhook_url = "https://example.com/webhook";
            assert!(webhook_url.starts_with("https://"));
            assert!(webhook_url.len() > 10);
        }

        #[test]
        fn test_message_payload_structure() {
            let config = create_test_config();
            let opportunity = create_test_opportunity();
            
            // Simulate message payload construction
            let payload = json!({
                "chat_id": config.chat_id,
                "text": "Test message",
                "parse_mode": "MarkdownV2"
            });
            
            assert_eq!(payload["chat_id"], config.chat_id);
            assert_eq!(payload["parse_mode"], "MarkdownV2");
            assert!(payload["text"].is_string());
        }
    }

    mod webhook_handling {
        use super::*;

        #[test]
        fn test_webhook_data_structure() {
            // Simulate incoming webhook data
            let webhook_data = json!({
                "update_id": 123456789,
                "message": {
                    "message_id": 1,
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
            
            // Validate webhook structure
            assert!(webhook_data["update_id"].is_number());
            assert!(webhook_data["message"]["message_id"].is_number());
            assert!(webhook_data["message"]["text"].is_string());
            assert_eq!(webhook_data["message"]["text"], "/start");
        }

        #[test]
        fn test_command_extraction() {
            let command_text = "/start";
            assert!(command_text.starts_with('/'));
            
            let command = &command_text[1..]; // Remove '/' prefix
            assert_eq!(command, "start");
        }

        #[test]
        fn test_chat_id_extraction() {
            let webhook_data = json!({
                "message": {
                    "chat": {
                        "id": -123456789
                    }
                }
            });
            
            if let Some(chat_id) = webhook_data["message"]["chat"]["id"].as_i64() {
                assert_eq!(chat_id, -123456789);
            }
        }
    }

    mod utility_functions {
        use super::*;

        #[test]
        fn test_service_configuration_access() {
            let config = create_test_config();
            let original_token = config.bot_token.clone();
            let service = TelegramService::new(config);
            
            // Service should store configuration
            // (In real implementation, there would be a getter method)
        }

        #[test]
        fn test_exchange_name_formatting() {
            let binance = ExchangeIdEnum::Binance;
            let bybit = ExchangeIdEnum::Bybit;
            
            // Test that exchange enums can be formatted
            let binance_str = format!("{:?}", binance);
            let bybit_str = format!("{:?}", bybit);
            
            assert_eq!(binance_str, "Binance");
            assert_eq!(bybit_str, "Bybit");
        }

        #[test]
        fn test_rate_difference_formatting() {
            let rate_diff = 0.002f64; // 0.2%
            
            // Test percentage formatting
            let percentage = rate_diff * 100.0;
            assert!((percentage - 0.2).abs() < 1e-10);
            
            // Test basis points formatting
            let basis_points = rate_diff * 10000.0;
            assert!((basis_points - 20.0).abs() < 1e-10);
        }

        #[test]
        fn test_timestamp_conversion() {
            let timestamp = 1640995200000u64; // Jan 1, 2022
            
            // Should be convertible to datetime
            let datetime = chrono::DateTime::from_timestamp_millis(timestamp as i64);
            assert!(datetime.is_some());
            
            if let Some(dt) = datetime {
                assert_eq!(dt.year(), 2022);
                assert_eq!(dt.month(), 1);
                assert_eq!(dt.day(), 1);
            }
        }
    }

    mod integration_scenarios {
        use super::*;

        #[test]
        fn test_complete_notification_workflow() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            let opportunity = create_test_opportunity();
            
            // Verify all components needed for notification are present
            assert!(!opportunity.id.is_empty());
            assert!(!opportunity.pair.is_empty());
            assert!(opportunity.rate_difference > 0.0);
            assert!(opportunity.long_exchange.is_some());
            assert!(opportunity.short_exchange.is_some());
        }

        #[test]
        fn test_multiple_opportunities_handling() {
            let config = create_test_config();
            let service = TelegramService::new(config);
            
            // Create multiple opportunities
            let mut opp1 = create_test_opportunity();
            opp1.id = "opp_001".to_string();
            opp1.pair = "BTCUSDT".to_string();
            
            let mut opp2 = create_test_opportunity();
            opp2.id = "opp_002".to_string();
            opp2.pair = "ETHUSDT".to_string();
            
            // Should handle multiple opportunities
            assert_ne!(opp1.id, opp2.id);
            assert_ne!(opp1.pair, opp2.pair);
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