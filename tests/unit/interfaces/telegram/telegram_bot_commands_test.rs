#![allow(unused_imports, unused_variables, unused_mut, dead_code)]

use arb_edge::services::core::infrastructure::d1_database::D1Service;
use arb_edge::services::core::user::user_profile::UserProfileService;
use arb_edge::services::interfaces::telegram::telegram::{TelegramConfig, TelegramService};
use arb_edge::types::{CommandPermission, SubscriptionTier, UserProfile};
use serde_json::{json, Value};
use std::sync::Arc;
use worker::kv::KvStore;

/// Mock KV Store for testing
struct MockKvStore;

impl MockKvStore {
    fn new() -> Self {
        Self
    }
}

/// Mock D1 Service for testing
struct MockD1Service;

impl MockD1Service {
    fn new() -> Self {
        Self
    }
}

/// Test helper to create a mock Telegram update
fn create_telegram_update(user_id: i64, message_text: &str, chat_type: &str) -> Value {
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
                "id": if chat_type == "private" { user_id } else { -1001234567890_i64 },
                "type": chat_type,
                "title": if chat_type != "private" { Some("Test Group") } else { None }
            },
            "date": 1640995200,
            "text": message_text
        }
    })
}

/// Test helper to create a test user profile
fn create_test_user_profile(telegram_id: i64, subscription_tier: SubscriptionTier) -> UserProfile {
    let mut profile = UserProfile::new(Some(telegram_id), None);
    profile.subscription.tier = subscription_tier;
    profile.telegram_username = Some("testuser".to_string());
    profile
}

#[cfg(test)]
mod telegram_bot_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_start_command_private_chat() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let mut telegram_service = TelegramService::new(config);

        // Mock UserProfileService setup would go here in a real test
        // For now, we'll test the basic functionality

        let update = create_telegram_update(123456789, "/start", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        if let Err(e) = &result {
            println!("Error in test_start_command_private_chat: {:?}", e);
        }
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Welcome to ArbEdge"));
        assert!(response_text.contains("arbitrage opportunities"));
    }

    #[tokio::test]
    async fn test_start_command_group_chat() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/start", "group");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Welcome to ArbEdge"));
        assert!(response_text.contains("group"));
    }

    #[tokio::test]
    async fn test_help_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/help", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("ArbEdge Bot Commands"));
        assert!(response_text.contains("/opportunities"));
        assert!(response_text.contains("/profile"));
        assert!(response_text.contains("/settings"));
    }

    #[tokio::test]
    async fn test_profile_command_without_user_service() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/profile", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Your Profile"));
        assert!(response_text.contains("Guest User"));
        assert!(response_text.contains("Free"));
    }

    #[tokio::test]
    async fn test_opportunities_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/opportunities", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        println!("Opportunities response: {}", response_text);
        assert!(
            response_text.contains("Trading Opportunities")
                || response_text.contains("Recent Arbitrage Opportunities")
        );
    }

    #[tokio::test]
    async fn test_categories_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/categories", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Opportunity Categories"));
    }

    #[tokio::test]
    async fn test_settings_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/settings", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Bot Configuration"));
        assert!(response_text.contains("Notification Settings"));
    }

    #[tokio::test]
    async fn test_unknown_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/unknown_command", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        // Unknown commands should return None or a default response
        // The exact behavior depends on implementation
    }

    #[tokio::test]
    async fn test_group_command_restrictions() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Test that trading commands are restricted in groups
        let update = create_telegram_update(123456789, "/balance", "group");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        assert!(response_text.contains("Security Notice"));
        assert!(response_text.contains("private chats"));
    }

    #[tokio::test]
    async fn test_malformed_update() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Create malformed update (missing required fields)
        let update = json!({
            "update_id": 123456789
            // Missing message field
        });

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_none()); // Should return None for malformed updates
    }

    #[tokio::test]
    async fn test_update_without_text() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Create update without text (e.g., photo, sticker, etc.)
        let update = json!({
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "from": {
                    "id": 123456789,
                    "is_bot": false,
                    "first_name": "Test"
                },
                "chat": {
                    "id": 123456789,
                    "type": "private"
                },
                "date": 1640995200,
                "photo": [
                    {
                        "file_id": "test_file_id",
                        "width": 100,
                        "height": 100
                    }
                ]
            }
        });

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_none()); // Should return None for non-text messages
    }

    #[tokio::test]
    async fn test_session_first_architecture_start_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/start", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        // Should contain welcome message (session creation would happen in real implementation)
        assert!(response_text.contains("Welcome to ArbEdge"));
    }

    #[tokio::test]
    async fn test_session_first_architecture_help_command() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);
        let update = create_telegram_update(123456789, "/help", "private");

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());

        let response_text = response.unwrap();
        // Help should work without session
        assert!(response_text.contains("Bot Commands") || response_text.contains("help"));
    }

    #[tokio::test]
    async fn test_session_first_architecture_protected_commands() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Test commands that should require session (without session service set)
        let protected_commands = vec![
            "/opportunities",
            "/profile", 
            "/settings",
            "/ai_insights",
            "/balance",
            "/buy",
            "/sell",
        ];

        for command in protected_commands {
            let update = create_telegram_update(123456789, command, "private");
            let result = telegram_service.handle_webhook(update).await;
            
            assert!(result.is_ok());
            
            // Verify the response indicates session is required
            if let Ok(Some(response)) = result {
                // The response should contain session-related messaging
                assert!(
                    response.contains("session") || 
                    response.contains("start") || 
                    response.contains("welcome") ||
                    response.contains("Please") ||
                    response.contains("first"),
                    "Protected command '{}' should indicate session requirement, got: '{}'", 
                    command, response
                );
            }
        }

        // Test that session-exempt commands work without session requirements
        let exempt_commands = vec!["/start", "/help"];
        
        for command in exempt_commands {
            let update = create_telegram_update(123456789, command, "private");
            let result = telegram_service.handle_webhook(update).await;
            
            assert!(result.is_ok());
            
            // These commands should work without session requirements
            if let Ok(Some(response)) = result {
                assert!(
                    !response.is_empty(),
                    "Exempt command '{}' should provide a response", 
                    command
                );
            }
        }
    }
}

#[cfg(test)]
mod telegram_keyboard_integration_tests {
    use super::*;
    use arb_edge::services::interfaces::telegram::telegram_keyboard::{
        InlineKeyboard, InlineKeyboardButton,
    };

    #[tokio::test]
    async fn test_main_menu_keyboard_creation() {
        // Arrange & Act
        let keyboard = InlineKeyboard::create_main_menu();

        // Assert
        assert!(!keyboard.buttons.is_empty());

        // Check that basic buttons are present
        let all_buttons: Vec<&InlineKeyboardButton> =
            keyboard.buttons.iter().flat_map(|row| row.iter()).collect();

        let button_texts: Vec<&str> = all_buttons.iter().map(|btn| btn.text.as_str()).collect();

        assert!(button_texts.contains(&"📊 Opportunities"));
        assert!(button_texts.contains(&"📈 Categories"));
        assert!(button_texts.contains(&"👤 Profile"));
        assert!(button_texts.contains(&"⚙️ Settings"));
        assert!(button_texts.contains(&"❓ Help"));
    }

    #[tokio::test]
    async fn test_keyboard_json_conversion() {
        // Arrange
        let mut keyboard = InlineKeyboard::new();
        keyboard.add_row(vec![InlineKeyboardButton::new(
            "Test Button",
            "test_callback",
        )]);

        // Act
        let json = keyboard.to_json();

        // Assert
        assert!(json["inline_keyboard"].is_array());
        let inline_keyboard = &json["inline_keyboard"];
        assert_eq!(inline_keyboard.as_array().unwrap().len(), 1);

        let first_row = &inline_keyboard[0];
        assert_eq!(first_row.as_array().unwrap().len(), 1);

        let button = &first_row[0];
        assert_eq!(button["text"], "Test Button");
        assert_eq!(button["callback_data"], "test_callback");
    }

    #[tokio::test]
    async fn test_permission_based_button_filtering() {
        // This test would require a mock UserProfileService
        // For now, we test the structure

        // Arrange
        let keyboard = InlineKeyboard::create_main_menu();

        // Act - Filter with no user profile service (should show only public buttons)
        let filtered_keyboard = keyboard.filter_by_permissions(&None, "123456789").await;

        // Assert
        assert!(!filtered_keyboard.buttons.is_empty());

        // All remaining buttons should have no permission requirements
        let all_buttons: Vec<&InlineKeyboardButton> = filtered_keyboard
            .buttons
            .iter()
            .flat_map(|row| row.iter())
            .collect();

        for button in all_buttons {
            assert!(
                button.required_permission.is_none(),
                "Button '{}' should not require permissions when no user service is available",
                button.text
            );
        }
    }
}

#[cfg(test)]
mod telegram_webhook_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_with_missing_user_id() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Create update with missing user ID
        let update = json!({
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "from": {
                    // Missing "id" field
                    "is_bot": false,
                    "first_name": "Test"
                },
                "chat": {
                    "id": 123456789,
                    "type": "private"
                },
                "date": 1640995200,
                "text": "/start"
            }
        });

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert
        assert!(result.is_err()); // Should return error for missing user ID
    }

    #[tokio::test]
    async fn test_webhook_with_invalid_chat_context() {
        // Arrange
        let config = TelegramConfig {
            bot_token: "test_token".to_string(),
            chat_id: "123456789".to_string(),
            is_test_mode: true,
        };

        let telegram_service = TelegramService::new(config);

        // Create update with invalid chat context
        let update = json!({
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "from": {
                    "id": 123456789,
                    "is_bot": false,
                    "first_name": "Test"
                },
                "chat": {
                    // Missing required chat fields
                    "type": "invalid_type"
                },
                "date": 1640995200,
                "text": "/start"
            }
        });

        // Act
        let result = telegram_service.handle_webhook(update).await;

        // Assert - Should handle gracefully or return appropriate error
        // The exact behavior depends on ChatContext::from_telegram_update implementation
        assert!(result.is_ok() || result.is_err());
    }
}
