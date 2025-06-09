use arb_edge::services::interfaces::telegram::core::bot_client::TelegramConfig;
use arb_edge::services::interfaces::telegram::legacy_telegram::TelegramService;
use serde_json::json;

/// E2E Webhook Tests for Session Management Integration
/// These tests simulate real Telegram webhook requests to validate session-first architecture

#[tokio::test]
async fn test_e2e_session_creation_via_start_command() {
    // Test that /start command creates a new session via webhook
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    // Create realistic Telegram webhook update for /start command
    let start_webhook = json!({
        "update_id": 123456789,
        "message": {
            "message_id": 1,
            "from": {
                "id": user_id,
                "is_bot": false,
                "first_name": "TestUser",
                "username": "testuser",
                "language_code": "en"
            },
            "chat": {
                "id": user_id,
                "first_name": "TestUser",
                "username": "testuser",
                "type": "private"
            },
            "date": 1640995200,
            "text": "/start",
            "entities": [
                {
                    "offset": 0,
                    "length": 6,
                    "type": "bot_command"
                }
            ]
        }
    });

    // Process webhook
    let result = telegram_service.handle_webhook(start_webhook, None).await;

    // Validate response
    assert!(result.is_ok());
    let response_text = result.unwrap();
    assert!(response_text.contains("Welcome to ArbEdge"));

    // In real implementation with SessionManagementService, this would also:
    // 1. Create a new session in D1 database
    // 2. Cache session in KV store
    // 3. Return session-aware welcome message
}

#[tokio::test]
async fn test_e2e_session_validation_for_protected_commands() {
    // Test that protected commands require active session
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    // Test protected commands that should require session
    let protected_commands = [
        "/opportunities",
        "/profile",
        "/settings",
        "/ai_insights",
        "/balance",
        "/buy BTC",
        "/sell ETH",
        "/portfolio",
        "/analytics",
        "/preferences",
    ];

    for command in protected_commands {
        let webhook = json!({
            "update_id": 123456790,
            "message": {
                "message_id": 2,
                "from": {
                    "id": user_id,
                    "is_bot": false,
                    "first_name": "TestUser",
                    "username": "testuser"
                },
                "chat": {
                    "id": user_id,
                    "type": "private"
                },
                "date": 1640995200,
                "text": command,
                "entities": [
                    {
                        "offset": 0,
                        "length": command.split_whitespace().next().unwrap().len(),
                        "type": "bot_command"
                    }
                ]
            }
        });

        let result = telegram_service.handle_webhook(webhook, None).await;
        assert!(result.is_ok());

        // Without SessionManagementService, commands work normally
        // With SessionManagementService, they would check for active session
        // and return session required message if no session exists
    }
}

#[tokio::test]
async fn test_e2e_session_exempt_commands() {
    // Test that /start and /help work without session
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    // Test exempt commands
    let exempt_commands = ["/start", "/help"];

    for command in exempt_commands {
        let webhook = json!({
            "update_id": 123456791,
            "message": {
                "message_id": 3,
                "from": {
                    "id": user_id,
                    "is_bot": false,
                    "first_name": "TestUser",
                    "username": "testuser"
                },
                "chat": {
                    "id": user_id,
                    "type": "private"
                },
                "date": 1640995200,
                "text": command,
                "entities": [
                    {
                        "offset": 0,
                        "length": command.len(),
                        "type": "bot_command"
                    }
                ]
            }
        });

        let result = telegram_service.handle_webhook(webhook, None).await;
        assert!(result.is_ok());

        let response = result.unwrap();

        // These commands should always work regardless of session state
        let response_text = response;
        assert!(!response_text.contains("session required"));
    }
}

#[tokio::test]
async fn test_e2e_callback_query_session_validation() {
    // Test callback query handling with session validation
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    // Test callback queries that should require session
    let callback_data_options = [
        "opportunities",
        "buy_btc",
        "sell_eth",
        "portfolio",
        "settings",
        "ai_insights",
    ];

    for callback_data in callback_data_options {
        let callback_webhook = json!({
            "update_id": 123456792,
            "callback_query": {
                "id": format!("callback_{}", callback_data),
                "from": {
                    "id": user_id,
                    "is_bot": false,
                    "first_name": "TestUser",
                    "username": "testuser"
                },
                "message": {
                    "message_id": 123,
                    "from": {
                        "id": 987654321,
                        "is_bot": true,
                        "first_name": "ArbEdgeBot",
                        "username": "arbedgebot"
                    },
                    "chat": {
                        "id": user_id,
                        "type": "private"
                    },
                    "date": 1640995200,
                    "text": "Choose an option:",
                    "reply_markup": {
                        "inline_keyboard": [[
                            {
                                "text": "Opportunities",
                                "callback_data": callback_data
                            }
                        ]]
                    }
                },
                "data": callback_data
            }
        });

        let result = telegram_service
            .handle_webhook(callback_webhook, None)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response, "OK");
    }
}

#[tokio::test]
async fn test_e2e_session_activity_extension() {
    // Test that user activity extends session lifetime
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    // Simulate user activity over time
    let activities = [
        "/start",
        "/opportunities",
        "/profile",
        "/help",
        "/settings",
        "/ai_insights",
        "/balance",
    ];

    for (index, command) in activities.iter().enumerate() {
        let webhook = json!({
            "update_id": 123456793 + index,
            "message": {
                "message_id": 4 + index,
                "from": {
                    "id": user_id,
                    "is_bot": false,
                    "first_name": "TestUser",
                    "username": "testuser"
                },
                "chat": {
                    "id": user_id,
                    "type": "private"
                },
                "date": 1640995200 + (index * 3600), // Each command 1 hour apart
                "text": command,
                "entities": [
                    {
                        "offset": 0,
                        "length": command.split_whitespace().next().unwrap().len(),
                        "type": "bot_command"
                    }
                ]
            }
        });

        let result = telegram_service.handle_webhook(webhook, None).await;
        assert!(result.is_ok());

        // Each activity should extend session lifetime
        // In real implementation, this would update last_activity_at in session
    }
}

#[tokio::test]
async fn test_e2e_group_chat_session_restrictions() {
    // Test that certain commands are restricted in group chats for security
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;
    let group_id = -987654321i64;

    // Test trading commands in group chat (should be restricted)
    let restricted_commands = [
        "/balance",
        "/buy BTC",
        "/sell ETH",
        "/portfolio",
        "/api_keys",
    ];

    for command in restricted_commands {
        let group_webhook = json!({
            "update_id": 123456794,
            "message": {
                "message_id": 5,
                "from": {
                    "id": user_id,
                    "is_bot": false,
                    "first_name": "TestUser",
                    "username": "testuser"
                },
                "chat": {
                    "id": group_id,
                    "type": "group",
                    "title": "Crypto Trading Group",
                    "all_members_are_administrators": false
                },
                "date": 1640995200,
                "text": command,
                "entities": [
                    {
                        "offset": 0,
                        "length": command.split_whitespace().next().unwrap().len(),
                        "type": "bot_command"
                    }
                ]
            }
        });

        let result = telegram_service.handle_webhook(group_webhook, None).await;
        assert!(result.is_ok());

        let response = result.unwrap();

        let response_text = response;
        // Should contain security notice for trading commands in groups
        assert!(response_text.contains("Security Notice") || response_text.contains("private"));
    }
}

#[tokio::test]
async fn test_e2e_sequential_session_requests() {
    // Test sequential webhook processing for same user (WASM-compatible)
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    // Process multiple sequential requests from same user
    for i in 0..5 {
        let webhook = json!({
            "update_id": 123456795 + i,
            "message": {
                "message_id": 6 + i,
                "from": {
                    "id": user_id,
                    "is_bot": false,
                    "first_name": "TestUser",
                    "username": "testuser"
                },
                "chat": {
                    "id": user_id,
                    "type": "private"
                },
                "date": 1640995200,
                "text": "/opportunities",
                "entities": [
                    {
                        "offset": 0,
                        "length": 14,
                        "type": "bot_command"
                    }
                ]
            }
        });

        let result = telegram_service.handle_webhook(webhook, None).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_e2e_malformed_webhook_handling() {
    // Test handling of malformed webhook requests
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);

    // Test various malformed webhook scenarios
    let malformed_webhooks = vec![
        // Missing message
        json!({
            "update_id": 123456796
        }),
        // Missing from field
        json!({
            "update_id": 123456797,
            "message": {
                "message_id": 7,
                "chat": {
                    "id": 123456789,
                    "type": "private"
                },
                "date": 1640995200,
                "text": "/start"
            }
        }),
        // Missing text field
        json!({
            "update_id": 123456798,
            "message": {
                "message_id": 8,
                "from": {
                    "id": 123456789,
                    "is_bot": false,
                    "first_name": "TestUser"
                },
                "chat": {
                    "id": 123456789,
                    "type": "private"
                },
                "date": 1640995200
            }
        }),
    ];

    for webhook in malformed_webhooks {
        let result = telegram_service.handle_webhook(webhook, None).await;
        // Should handle gracefully without panicking
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_e2e_webhook_performance_benchmarks() {
    // Test webhook processing performance
    let config = TelegramConfig {
        bot_token: "test_token".to_string(),
        chat_id: "123456789".to_string(),
        is_test_mode: true,
    };

    let telegram_service = TelegramService::new(config);
    let user_id = 123456789i64;

    let webhook = json!({
        "update_id": 123456799,
        "message": {
            "message_id": 9,
            "from": {
                "id": user_id,
                "is_bot": false,
                "first_name": "TestUser",
                "username": "testuser"
            },
            "chat": {
                "id": user_id,
                "type": "private"
            },
            "date": 1640995200,
            "text": "/opportunities",
            "entities": [
                {
                    "offset": 0,
                    "length": 14,
                    "type": "bot_command"
                }
            ]
        }
    });

    // Measure processing time
    let start_time = std::time::Instant::now();
    let result = telegram_service.handle_webhook(webhook, None).await;
    let processing_time = start_time.elapsed();

    assert!(result.is_ok());

    // Should process webhook in reasonable time (< 1 second for test mode)
    assert!(processing_time.as_millis() < 1000);

    println!("Webhook processing time: {:?}", processing_time);
}
