use arb_edge::services::interfaces::telegram::telegram::{TelegramConfig, TelegramService};
use serde_json::{json, Value};

/// Helper function to create test TelegramConfig
fn create_test_config() -> TelegramConfig {
    TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    }
}

/// Test helper to create a mock Telegram update
fn create_telegram_update(user_id: i64, message_text: &str, chat_type: &str) -> Value {
    create_telegram_update_with_options(user_id, message_text, chat_type, None, None)
}

fn create_telegram_update_with_options(
    user_id: i64,
    message_text: &str,
    chat_type: &str,
    group_chat_id: Option<i64>,
    timestamp: Option<i64>,
) -> Value {
    let default_group_chat_id = -1001234567890_i64;
    let default_timestamp = 1640995200;

    json!({
        "update_id": 123456789,
        "message": {
            "message_id": 1,
            "from": {
                "id": user_id,
                "is_bot": false,
                "first_name": "Test",
                "username": "testuser",
                "language_code": "en"
            },
            "chat": {
                "id": if chat_type == "private" {
                    user_id
                } else {
                    group_chat_id.unwrap_or(default_group_chat_id)
                },
                "type": chat_type,
                "title": if chat_type != "private" { Some("Test Group") } else { None }
            },
            "date": timestamp.unwrap_or(default_timestamp),
            "text": message_text
        }
    })
}

#[cfg(test)]
mod telegram_trading_commands_tests {
    use super::*;

    #[tokio::test]
    async fn test_balance_command_private_chat() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/balance", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(
            response_text.contains("Account Balance")
                || response_text.contains("Subscription Required")
        );
    }

    #[tokio::test]
    async fn test_buy_command_with_parameters() {
        // Arrange
        let config = create_test_config();

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/buy BTCUSDT 0.001 50000", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(
            response_text.contains("Buy Order") || response_text.contains("Subscription Required")
        );
    }

    #[tokio::test]
    async fn test_sell_command_with_parameters() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/sell ETHUSDT 0.5 3200", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(
            response_text.contains("Sell Order") || response_text.contains("Subscription Required")
        );
    }

    #[tokio::test]
    async fn test_trading_commands_blocked_in_groups() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Test multiple trading commands in group chat
        let commands = vec!["/balance", "/buy BTCUSDT 0.001", "/sell ETHUSDT 0.5"];

        for command in commands {
            let update = create_telegram_update(123456789, command, "group");

            // Act
            let result = telegram_service.handle_webhook(update).await;

            // Assert
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(response.is_some());

            let response_text = response.unwrap();
            assert!(
                response_text.contains("Security Notice")
                    || response_text.contains("private chats")
            );
        }
    }
}

#[cfg(test)]
mod telegram_auto_trading_commands_tests {
    use super::*;

    #[tokio::test]
    async fn test_auto_enable_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/auto_enable", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(
            response_text.contains("Auto Trading")
                || response_text.contains("Subscription Required")
        );
    }

    #[tokio::test]
    async fn test_auto_config_command_with_params() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/auto_config max_position 1000", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(
            response_text.contains("Configuration Updated")
                || response_text.contains("Subscription Required")
        );
    }
}

#[cfg(test)]
mod telegram_admin_commands_tests {
    use super::*;

    #[tokio::test]
    async fn test_admin_stats_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/admin_stats", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(
            response_text.contains("System Administration")
                || response_text.contains("Access Denied")
        );
    }

    #[tokio::test]
    async fn test_admin_broadcast_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/admin_broadcast Test message", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Broadcast") || response_text.contains("Access Denied"));
    }
}

#[cfg(test)]
mod telegram_ai_commands_tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_insights_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/ai_insights", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("AI Analysis") || response_text.contains("AI"));
    }

    #[tokio::test]
    async fn test_risk_assessment_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/risk_assessment", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Risk Assessment") || response_text.contains("Portfolio"));
    }
}

#[cfg(test)]
mod telegram_edge_cases_tests {
    use super::*;

    #[tokio::test]
    async fn test_command_with_extra_whitespace() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "  /help  ", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Bot Commands") || response_text.contains("help"));
    }

    #[tokio::test]
    async fn test_case_sensitive_commands() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/HELP", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        // Commands should be case sensitive, so /HELP should not match /help
        assert!(response.is_none());
    }

    #[tokio::test]
    async fn test_unicode_in_command_parameters() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update =
            create_telegram_update(123456789, "/admin_broadcast Hello 🚀 World! 💎", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Broadcast") || response_text.contains("Access Denied"));
    }
}
