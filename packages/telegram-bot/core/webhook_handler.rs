// src/services/interfaces/telegram/core/webhook_handler.rs

//! Telegram Webhook Handler
//!
//! Processes incoming webhook updates from Telegram including:
//! - Message processing
//! - Callback query handling
//! - Update routing
//! - Error handling

use crate::core::bot_client::{TelegramError, TelegramResult};
use crate::core::command_router::{CommandRouter, CommandContext, UserPermissions};
use crate::handlers::initialize_command_handlers;
use serde_json::Value;
use worker::console_log;
use chrono;

/// Telegram webhook update processor
pub struct WebhookHandler {
    command_router: CommandRouter,
}

impl WebhookHandler {
    pub fn new() -> Self {
        Self {
            command_router: initialize_command_handlers(),
        }
    }

    /// Process incoming webhook update
    pub async fn handle_update(&self, update: Value) -> TelegramResult<String> {
        console_log!("📱 Processing Telegram update: {}", update);

        // Handle different types of updates
        if let Some(message) = update.get("message") {
            return self.handle_message(message).await;
        }

        if let Some(callback_query) = update.get("callback_query") {
            return self.handle_callback_query(callback_query).await;
        }

        if let Some(inline_query) = update.get("inline_query") {
            return self.handle_inline_query(inline_query).await;
        }

        if let Some(chosen_inline_result) = update.get("chosen_inline_result") {
            return self.handle_chosen_inline_result(chosen_inline_result).await;
        }

        console_log!("⚠️ Unknown update type received");
        Ok("Unknown update type processed".to_string())
    }

    /// Handle incoming message
    async fn handle_message(&self, message: &Value) -> TelegramResult<String> {
        let chat_id = message
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| TelegramError::Api("Missing chat ID".to_string()))?;

        let user_id = message
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| TelegramError::Api("Missing user ID".to_string()))?;

        console_log!("💬 Message from user {} in chat {}", user_id, chat_id);

        // Handle text messages
        if let Some(text) = message.get("text").and_then(|t| t.as_str()) {
            return self.handle_text_message(chat_id, user_id, text).await;
        }

        // Handle other message types
        if message.get("photo").is_some() {
            return self.handle_photo_message(chat_id, user_id).await;
        }

        if message.get("document").is_some() {
            return self.handle_document_message(chat_id, user_id).await;
        }

        if message.get("location").is_some() {
            return self.handle_location_message(chat_id, user_id).await;
        }

        Ok("Message processed".to_string())
    }

    /// Handle text message
    async fn handle_text_message(
        &self,
        chat_id: i64,
        user_id: i64,
        text: &str,
    ) -> TelegramResult<String> {
        console_log!(
            "📝 Text message: '{}' from user {} in chat {}",
            text,
            user_id,
            chat_id
        );

        // Check if it's a command
        if text.starts_with('/') {
            return self.handle_command(chat_id, user_id, text).await;
        }

        // Handle regular text
        self.handle_regular_text(chat_id, user_id, text).await
    }

    /// Handle bot command
    async fn handle_command(
        &self,
        chat_id: i64,
        user_id: i64,
        command: &str,
    ) -> TelegramResult<String> {
        console_log!(
            "🤖 Command: '{}' from user {} in chat {}",
            command,
            user_id,
            chat_id
        );

        // Validate command input
        if command.is_empty() {
            console_log!("❌ Empty command received from user {} in chat {}", user_id, chat_id);
            return Ok("❌ Invalid command. Please use /help to see available commands.".to_string());
        }

        if command.len() > 256 {
            console_log!("❌ Command too long ({} chars) from user {} in chat {}", command.len(), user_id, chat_id);
            return Ok("❌ Command too long. Please use shorter commands.".to_string());
        }

        // Parse command and arguments with error handling
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.first().unwrap_or(&"");
        let args: Vec<&str> = parts[1..].to_vec();

        // Log command parsing details
        console_log!("📝 Parsed command: '{}' with {} args from user {}", cmd, args.len(), user_id);

        // Create command context with error handling
        let user_permissions = match self.get_user_permissions(user_id).await {
            Ok(permissions) => permissions,
            Err(e) => {
                console_log!("⚠️ Failed to get user permissions for user {}: {:?}, using defaults", user_id, e);
                UserPermissions {
                    is_admin: false,
                    is_premium: false,
                    user_level: 1,
                }
            }
        };

        let context = CommandContext {
            user_permissions,
            message_data: serde_json::json!({
                "chat_id": chat_id,
                "user_id": user_id,
                "command": command,
                "timestamp": chrono::Utc::now().timestamp()
            }),
            bot_token: String::new(), // TODO: Get actual bot token from config
        };

        // Route command through the command router with comprehensive error handling
        match self.command_router
            .route_command(cmd, chat_id, user_id, &args, &context)
            .await
        {
            Ok(response) => {
                console_log!("✅ Command '{}' executed successfully for user {}", cmd, user_id);
                Ok(response)
            }
            Err(e) => {
                console_log!("❌ Command '{}' failed for user {} in chat {}: {:?}", cmd, user_id, chat_id, e);
                
                // Log error details for debugging
                self.log_command_error(cmd, user_id, chat_id, &e).await;
                
                // Return user-friendly error message
                match e {
                    TelegramError::Api(msg) => Ok(format!("❌ Command failed: {}", msg)),
                    TelegramError::Http(_) => Ok("❌ Network error occurred. Please try again later.".to_string()),
                    TelegramError::Json(_) => Ok("❌ Data processing error. Please try again.".to_string()),
                    TelegramError::Timeout => Ok("❌ Command timed out. Please try again.".to_string()),
                    TelegramError::RateLimit => Ok("❌ Too many requests. Please wait a moment and try again.".to_string()),
                }
            }
        }
    }

    /// Handle regular text (not a command)
    async fn handle_regular_text(
        &self,
        chat_id: i64,
        user_id: i64,
        text: &str,
    ) -> TelegramResult<String> {
        console_log!(
            "💭 Regular text: '{}' from user {} in chat {}",
            text,
            user_id,
            chat_id
        );

        // Validate text input
        if text.is_empty() {
            console_log!("⚠️ Empty text message from user {} in chat {}", user_id, chat_id);
            return Ok("I received an empty message. How can I help you?".to_string());
        }

        if text.len() > 4096 {
            console_log!("⚠️ Text message too long ({} chars) from user {} in chat {}", text.len(), user_id, chat_id);
            return Ok("Your message is too long. Please send shorter messages.".to_string());
        }

        // Log message processing
        console_log!("📝 Processing regular text ({} chars) from user {}", text.len(), user_id);

        // TODO: Implement natural language processing
        // For now, provide helpful guidance
        let response = if text.to_lowercase().contains("help") {
            "I can help you with cryptocurrency arbitrage opportunities! Use /help to see all available commands.".to_string()
        } else if text.to_lowercase().contains("price") || text.to_lowercase().contains("arbitrage") {
            "To check arbitrage opportunities, use the /opportunities command.".to_string()
        } else if text.to_lowercase().contains("balance") {
            "To check your balance, use the /balance command.".to_string()
        } else {
            format!("I understand you said: \"{}\". Use /help to see what I can do for you!", text)
        };

        console_log!("✅ Regular text processed successfully for user {}", user_id);
        Ok(response)
    }

    /// Handle callback query (inline button press)
    async fn handle_callback_query(&self, callback_query: &Value) -> TelegramResult<String> {
        // Extract and validate callback query data with comprehensive error handling
        let query_id = callback_query
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                console_log!("❌ Missing callback query ID in payload: {:?}", callback_query);
                TelegramError::Api("Missing callback query ID".to_string())
            })?;

        let user_id = callback_query
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                console_log!("❌ Missing or invalid user ID in callback query: {:?}", callback_query);
                TelegramError::Api("Missing user ID".to_string())
            })?;

        let chat_id = callback_query
            .get("message")
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|id| id.as_i64())
            .unwrap_or(user_id); // Fallback to user_id for private chats

        let data = callback_query
            .get("data")
            .and_then(|d| d.as_str())
            .unwrap_or("");

        console_log!("🔘 Callback query: '{}' from user {} in chat {} (query_id: {})", data, user_id, chat_id, query_id);

        // Validate callback data
        if data.is_empty() {
            console_log!("⚠️ Empty callback data from user {}", user_id);
            return Ok("Invalid button data received.".to_string());
        }

        if data.len() > 64 {
            console_log!("⚠️ Callback data too long ({} chars) from user {}", data.len(), user_id);
            return Ok("Button data too long.".to_string());
        }

        // Process callback query with error handling
        match self.process_callback_data(data, user_id, chat_id).await {
            Ok(response) => {
                console_log!("✅ Callback query '{}' processed successfully for user {}", data, user_id);
                Ok(response)
            }
            Err(e) => {
                console_log!("❌ Callback query '{}' failed for user {}: {:?}", data, user_id, e);
                
                // Log error details
                self.log_callback_error(data, user_id, chat_id, &e).await;
                
                Ok("❌ Failed to process button action. Please try again.".to_string())
            }
        }
    }

    /// Get user permissions with error handling
    async fn get_user_permissions(&self, user_id: i64) -> TelegramResult<UserPermissions> {
        // TODO: Implement actual user permission lookup from database
        // For now, return default permissions with some basic logic
        
        console_log!("🔍 Getting permissions for user {}", user_id);
        
        // Placeholder logic - in production, this would query a database
        let permissions = UserPermissions {
            is_admin: false, // TODO: Check admin list
            is_premium: false, // TODO: Check subscription status
            user_level: 1, // TODO: Get actual user level
        };
        
        Ok(permissions)
    }

    /// Process callback data with error handling
    async fn process_callback_data(
        &self,
        data: &str,
        user_id: i64,
        chat_id: i64,
    ) -> TelegramResult<String> {
        console_log!("🔄 Processing callback data: '{}' for user {}", data, user_id);
        
        // Parse callback data (format: "action:param1:param2")
        let parts: Vec<&str> = data.split(':').collect();
        let action = parts.first().copied().unwrap_or("");
        let params = &parts[1..];
        
        match action {
            "refresh" => Ok("🔄 Data refreshed!".to_string()),
            "settings" => Ok("⚙️ Opening settings...".to_string()),
            "help" => Ok("📋 Use /help to see all commands.".to_string()),
            _ => {
                console_log!("❓ Unknown callback action: '{}' from user {}", action, user_id);
                Ok(format!("❓ Unknown action: {}", action))
            }
        }
    }

    /// Log command execution errors for debugging
    async fn log_command_error(
        &self,
        command: &str,
        user_id: i64,
        chat_id: i64,
        error: &TelegramError,
    ) {
        let error_details = match error {
            TelegramError::Api(msg) => format!("API Error: {}", msg),
            TelegramError::Http(e) => format!("HTTP Error: {:?}", e),
            TelegramError::Json(e) => format!("JSON Error: {:?}", e),
            TelegramError::Timeout => "Timeout Error".to_string(),
            TelegramError::RateLimit => "Rate Limit Error".to_string(),
        };
        
        console_log!(
            "🚨 COMMAND_ERROR | Command: {} | User: {} | Chat: {} | Error: {} | Timestamp: {}",
            command,
            user_id,
            chat_id,
            error_details,
            chrono::Utc::now().to_rfc3339()
        );
    }

    /// Log callback query errors for debugging
    async fn log_callback_error(
        &self,
        data: &str,
        user_id: i64,
        chat_id: i64,
        error: &TelegramError,
    ) {
        let error_details = match error {
            TelegramError::Api(msg) => format!("API Error: {}", msg),
            TelegramError::Http(e) => format!("HTTP Error: {:?}", e),
            TelegramError::Json(e) => format!("JSON Error: {:?}", e),
            TelegramError::Timeout => "Timeout Error".to_string(),
            TelegramError::RateLimit => "Rate Limit Error".to_string(),
        };
        
        console_log!(
            "🚨 CALLBACK_ERROR | Data: {} | User: {} | Chat: {} | Error: {} | Timestamp: {}",
            data,
            user_id,
            chat_id,
            error_details,
            chrono::Utc::now().to_rfc3339()
        );
    }

    /// Handle inline query
    async fn handle_inline_query(&self, inline_query: &Value) -> TelegramResult<String> {
        let _query_id = inline_query
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| TelegramError::Api("Missing inline query ID".to_string()))?;

        let user_id = inline_query
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| TelegramError::Api("Missing user ID".to_string()))?;

        let query = inline_query
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("");

        console_log!("🔍 Inline query: '{}' from user {}", query, user_id);

        // TODO: Implement inline query handling
        Ok(format!("Inline query '{}' processed", query))
    }

    /// Handle chosen inline result
    async fn handle_chosen_inline_result(&self, chosen_result: &Value) -> TelegramResult<String> {
        let result_id = chosen_result
            .get("result_id")
            .and_then(|id| id.as_str())
            .unwrap_or("");

        let user_id = chosen_result
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| TelegramError::Api("Missing user ID".to_string()))?;

        console_log!(
            "✅ Chosen inline result: '{}' from user {}",
            result_id,
            user_id
        );

        // TODO: Implement chosen inline result handling
        Ok(format!("Chosen inline result '{}' processed", result_id))
    }

    /// Handle photo message
    async fn handle_photo_message(&self, chat_id: i64, user_id: i64) -> TelegramResult<String> {
        console_log!("📸 Photo message from user {} in chat {}", user_id, chat_id);
        Ok("Photo message processed".to_string())
    }

    /// Handle document message
    async fn handle_document_message(&self, chat_id: i64, user_id: i64) -> TelegramResult<String> {
        console_log!(
            "📄 Document message from user {} in chat {}",
            user_id,
            chat_id
        );
        Ok("Document message processed".to_string())
    }

    /// Handle location message
    async fn handle_location_message(&self, chat_id: i64, user_id: i64) -> TelegramResult<String> {
        console_log!(
            "📍 Location message from user {} in chat {}",
            user_id,
            chat_id
        );
        Ok("Location message processed".to_string())
    }

    // Command handling is now delegated to the CommandRouter and individual handlers
}

impl Default for WebhookHandler {
    fn default() -> Self {
        Self::new()
    }
}
