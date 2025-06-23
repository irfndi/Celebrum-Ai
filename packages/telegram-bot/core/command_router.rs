//! Command Router for Telegram Bot
//!
//! This module provides a modular command routing system that delegates
//! commands to specific handlers based on command type.

use crate::core::bot_client::{TelegramError, TelegramResult};
use serde_json::Value;
use std::collections::HashMap;
use worker::console_log;

/// Trait for command handlers
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// Handle a command with the given arguments
    async fn handle(
        &self,
        chat_id: i64,
        user_id: i64,
        args: &[&str],
        context: &CommandContext,
    ) -> TelegramResult<String>;

    /// Get the command name this handler responds to
    fn command_name(&self) -> &'static str;

    /// Get help text for this command
    fn help_text(&self) -> &'static str;

    /// Check if user has permission to use this command
    fn check_permission(&self, _user_permissions: &UserPermissions) -> bool {
        // Default: allow all users
        true
    }
}

/// Context passed to command handlers
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub user_permissions: UserPermissions,
    #[allow(dead_code)]
    pub message_data: Value,
    #[allow(dead_code)]
    pub bot_token: String,
}

/// User permissions structure
#[derive(Debug, Clone)]
pub struct UserPermissions {
    pub is_admin: bool,
    pub is_premium: bool,
    pub user_level: u8,
}

/// Command router that manages and dispatches commands
pub struct CommandRouter {
    handlers: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRouter {
    /// Create a new command router
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register_handler(&mut self, handler: Box<dyn CommandHandler>) {
        let command_name = handler.command_name().to_string();
        console_log!("📝 Registering command handler: {}", command_name);
        self.handlers.insert(command_name, handler);
    }

    /// Route a command to the appropriate handler
    pub async fn route_command(
        &self,
        command: &str,
        chat_id: i64,
        user_id: i64,
        args: &[&str],
        context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!(
            "🎯 Routing command: {} from user {} in chat {}",
            command,
            user_id,
            chat_id
        );

        // Validate input parameters
        if command.is_empty() {
            console_log!("❌ Empty command provided to router from user {}", user_id);
            return Err(TelegramError::Api("Empty command".to_string()));
        }

        // Remove leading slash if present
        let clean_command = command.strip_prefix('/').unwrap_or(command);

        // Log command details
        console_log!(
            "📝 Clean command: '{}' with {} args for user {}",
            clean_command,
            args.len(),
            user_id
        );

        match self.handlers.get(clean_command) {
            Some(handler) => {
                console_log!("🔍 Found handler for command: {}", clean_command);

                // Check permissions with detailed logging
                if !handler.check_permission(&context.user_permissions) {
                    console_log!(
                        "❌ Permission denied | Command: {} | User: {} | Admin: {} | Premium: {} | Level: {}",
                        command,
                        user_id,
                        context.user_permissions.is_admin,
                        context.user_permissions.is_premium,
                        context.user_permissions.user_level
                    );
                    return Ok("❌ You don't have permission to use this command.".to_string());
                }

                // Execute the handler with comprehensive error handling
                console_log!(
                    "⚡ Executing handler for command: {} (user: {})",
                    command,
                    user_id
                );

                let start_time = std::time::Instant::now();

                match handler.handle(chat_id, user_id, args, context).await {
                    Ok(response) => {
                        let duration = start_time.elapsed();
                        console_log!(
                            "✅ Command executed successfully | Command: {} | User: {} | Duration: {:?} | Response length: {}",
                            command,
                            user_id,
                            duration,
                            response.len()
                        );
                        Ok(response)
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        console_log!(
                            "❌ Command execution failed | Command: {} | User: {} | Duration: {:?} | Error: {:?}",
                            command,
                            user_id,
                            duration,
                            e
                        );

                        // Log detailed error information
                        self.log_handler_error(command, user_id, chat_id, &e).await;

                        // Re-throw the error to be handled by the webhook handler
                        Err(e)
                    }
                }
            }
            None => {
                console_log!(
                    "❓ Unknown command: '{}' from user {} (available: {:?})",
                    command,
                    user_id,
                    self.get_command_names()
                );

                // Suggest similar commands if available
                let suggestion = self.suggest_similar_command(clean_command);
                let help_text = if let Some(similar) = suggestion {
                    format!(
                        "❓ Unknown command: {}\n\n💡 Did you mean /{} ?\n\nUse /help to see all available commands.",
                        command, similar
                    )
                } else {
                    format!(
                        "❓ Unknown command: {}\n\nUse /help to see available commands.",
                        command
                    )
                };

                Ok(help_text)
            }
        }
    }

    /// Get all registered commands with their help text
    #[allow(dead_code)]
    pub fn get_help_text(&self) -> String {
        let mut help_lines = vec!["📋 Available commands:\n".to_string()];

        let mut commands: Vec<_> = self.handlers.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        for (name, handler) in commands {
            help_lines.push(format!("/{} - {}", name, handler.help_text()));
        }

        help_lines.join("\n")
    }

    /// Get list of registered command names
    pub fn get_command_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Suggest a similar command based on edit distance
    fn suggest_similar_command(&self, command: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for cmd_name in self.handlers.keys() {
            let distance = self.levenshtein_distance(command, cmd_name);
            // Only suggest if the distance is reasonable (less than half the command length)
            if distance < command.len() / 2 + 1 && distance < best_distance {
                best_distance = distance;
                best_match = Some(cmd_name.clone());
            }
        }

        best_match
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        matrix[len1][len2]
    }

    /// Log detailed error information for command handler failures
    async fn log_handler_error(
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
            "🚨 HANDLER_ERROR | Command: {} | User: {} | Chat: {} | Error: {} | Available Commands: {:?}",
            command,
            user_id,
            chat_id,
            error_details,
            self.get_command_names()
        );
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse command and arguments from message text
#[allow(dead_code)]
pub fn parse_command(text: &str) -> Option<(String, Vec<String>)> {
    let text = text.trim();

    if !text.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let command = parts[0].to_string();
    let args = parts[1..].iter().map(|s| s.to_string()).collect();

    Some((command, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        assert_eq!(
            parse_command("/start"),
            Some(("/start".to_string(), vec![]))
        );

        assert_eq!(
            parse_command("/balance BTC ETH"),
            Some((
                "/balance".to_string(),
                vec!["BTC".to_string(), "ETH".to_string()]
            ))
        );

        assert_eq!(parse_command("hello"), None);
        assert_eq!(parse_command(""), None);
    }
}
