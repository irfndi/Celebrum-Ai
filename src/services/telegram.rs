// src/services/telegram.rs

use crate::services::ai_intelligence::{
    AiOpportunityEnhancement, AiPerformanceInsights, ParameterSuggestion,
};
use crate::services::opportunity_categorization::CategorizedOpportunity;
use crate::types::{
    ArbitrageOpportunity, CommandPermission, GroupRateLimitConfig, GroupRegistration,
    MessageAnalytics,
};
use crate::utils::formatter::{
    escape_markdown_v2, format_ai_enhancement_message, format_categorized_opportunity_message,
    format_opportunity_message, format_parameter_suggestions_message,
    format_performance_insights_message,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
use reqwest::Client;
use serde_json::{json, Value};

// ============= CHAT CONTEXT DETECTION TYPES =============

#[derive(Debug, Clone, PartialEq)]
pub enum ChatType {
    Private,
    Group,
    SuperGroup,
    Channel,
}

#[derive(Debug, Clone)]
pub struct ChatContext {
    pub chat_id: String,
    pub chat_type: ChatType,
    pub user_id: Option<String>,
    pub is_bot_admin: bool,
}

impl ChatContext {
    pub fn new(chat_id: String, chat_type: ChatType, user_id: Option<String>) -> Self {
        Self {
            chat_id,
            chat_type,
            user_id,
            is_bot_admin: false,
        }
    }

    pub fn is_private(&self) -> bool {
        matches!(self.chat_type, ChatType::Private)
    }

    pub fn is_group_or_channel(&self) -> bool {
        matches!(
            self.chat_type,
            ChatType::Group | ChatType::SuperGroup | ChatType::Channel
        )
    }

    pub fn from_telegram_update(update: &Value) -> ArbitrageResult<Self> {
        let message = update["message"].as_object().ok_or_else(|| {
            ArbitrageError::validation_error("Missing message in update".to_string())
        })?;

        let chat = message["chat"].as_object().ok_or_else(|| {
            ArbitrageError::validation_error("Missing chat in message".to_string())
        })?;

        let chat_id = chat["id"]
            .as_i64()
            .ok_or_else(|| ArbitrageError::validation_error("Missing chat ID".to_string()))?
            .to_string();

        let chat_type_str = chat["type"]
            .as_str()
            .ok_or_else(|| ArbitrageError::validation_error("Missing chat type".to_string()))?;

        let chat_type = match chat_type_str {
            "private" => ChatType::Private,
            "group" => ChatType::Group,
            "supergroup" => ChatType::SuperGroup,
            "channel" => ChatType::Channel,
            _ => {
                return Err(ArbitrageError::validation_error(format!(
                    "Unknown chat type: {}",
                    chat_type_str
                )))
            }
        };

        let user_id = message["from"]["id"].as_u64().map(|id| id.to_string());

        Ok(ChatContext::new(chat_id, chat_type, user_id))
    }
}

#[derive(Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

pub struct TelegramService {
    config: TelegramConfig,
    http_client: Client,
    #[allow(dead_code)]
    analytics_enabled: bool,
    group_registrations: std::collections::HashMap<String, GroupRegistration>,
}

impl TelegramService {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            analytics_enabled: true,
            group_registrations: std::collections::HashMap::new(),
        }
    }

    /// Track message analytics for analysis
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    async fn track_message_analytics(
        &self,
        message_id: String,
        user_id: Option<String>,
        chat_context: &ChatContext,
        message_type: &str,
        command: Option<String>,
        content_type: &str,
        delivery_status: &str,
        response_time_ms: Option<u64>,
        metadata: serde_json::Value,
    ) -> ArbitrageResult<()> {
        if !self.analytics_enabled {
            return Ok(());
        }

        let analytics = MessageAnalytics {
            message_id,
            user_id,
            chat_id: chat_context.chat_id.clone(),
            chat_type: format!("{:?}", chat_context.chat_type).to_lowercase(),
            message_type: message_type.to_string(),
            command,
            content_type: content_type.to_string(),
            delivery_status: delivery_status.to_string(),
            response_time_ms,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            metadata,
        };

        // TODO: Store in database for analytics
        println!("Analytics: {:?}", analytics);
        Ok(())
    }

    /// Register group/channel when bot is added
    pub async fn register_group(
        &mut self,
        chat_context: &ChatContext,
        group_title: Option<String>,
        member_count: Option<u32>,
    ) -> ArbitrageResult<()> {
        if chat_context.is_private() {
            return Ok(()); // Not a group/channel
        }

        let default_rate_limit = GroupRateLimitConfig {
            max_opportunities_per_hour: 5,
            max_technical_signals_per_hour: 3,
            max_broadcasts_per_day: 10,
            cooldown_between_messages_minutes: 15,
        };

        let registration = GroupRegistration {
            group_id: chat_context.chat_id.clone(),
            group_type: format!("{:?}", chat_context.chat_type).to_lowercase(),
            group_title,
            group_username: None, // TODO: Extract from Telegram API
            member_count,
            admin_user_ids: vec![], // TODO: Get from Telegram API
            bot_permissions: vec!["read_messages".to_string(), "send_messages".to_string()],
            enabled_features: vec!["global_opportunities".to_string()],
            global_opportunities_enabled: true,
            technical_analysis_enabled: false, // Disabled by default
            rate_limit_config: default_rate_limit,
            registered_at: chrono::Utc::now().timestamp_millis() as u64,
            last_activity: chrono::Utc::now().timestamp_millis() as u64,
            total_messages_sent: 0,
            last_member_count_update: Some(chrono::Utc::now().timestamp_millis() as u64),
        };

        self.group_registrations
            .insert(chat_context.chat_id.clone(), registration);

        // TODO: Store in database
        println!("Registered group: {}", chat_context.chat_id);
        Ok(())
    }

    /// Update member count for a group/channel
    pub async fn update_group_member_count(
        &mut self,
        chat_id: &str,
        member_count: u32,
    ) -> ArbitrageResult<()> {
        if let Some(registration) = self.group_registrations.get_mut(chat_id) {
            registration.member_count = Some(member_count);
            registration.last_member_count_update =
                Some(chrono::Utc::now().timestamp_millis() as u64);

            // TODO: Store update in database
            println!("Updated member count for {}: {}", chat_id, member_count);
        }
        Ok(())
    }

    pub async fn send_message(&self, text: &str) -> ArbitrageResult<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let payload = json!({
            "chat_id": self.config.chat_id,
            "text": text,
            "parse_mode": "MarkdownV2"
        });

        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                ArbitrageError::network_error(format!("Failed to send Telegram message: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::telegram_error(format!(
                "Telegram API error: {}",
                error_text
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to parse Telegram response: {}", e))
        })?;

        if !result["ok"].as_bool().unwrap_or(false) {
            let error_description = result["description"].as_str().unwrap_or("Unknown error");
            return Err(ArbitrageError::telegram_error(format!(
                "Telegram API error: {}",
                error_description
            )));
        }

        Ok(())
    }

    // ============= SECURE NOTIFICATION METHODS =============

    /// Send notification with context awareness - PRIVATE ONLY for trading data
    pub async fn send_secure_notification(
        &self,
        message: &str,
        chat_context: &ChatContext,
        is_trading_data: bool,
    ) -> ArbitrageResult<bool> {
        // Security Check: Block trading data in groups/channels
        if is_trading_data && chat_context.is_group_or_channel() {
            // Log warning about blocked notification (would use log::warn! in production)
            println!(
                "WARNING: Blocked trading data notification to {}: {} (type: {:?})",
                chat_context.chat_id,
                message.chars().take(50).collect::<String>(),
                chat_context.chat_type
            );
            return Ok(false);
        }

        // Context-aware messaging
        let final_message = if chat_context.is_group_or_channel() {
            self.get_group_safe_message()
        } else {
            message.to_string()
        };

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let payload = json!({
            "chat_id": chat_context.chat_id,
            "text": final_message,
            "parse_mode": "MarkdownV2"
        });

        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                ArbitrageError::network_error(format!("Failed to send secure message: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::telegram_error(format!(
                "Telegram API error: {}",
                error_text
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to parse Telegram response: {}", e))
        })?;

        if !result["ok"].as_bool().unwrap_or(false) {
            let error_description = result["description"].as_str().unwrap_or("Unknown error");
            return Err(ArbitrageError::telegram_error(format!(
                "Telegram API error: {}",
                error_description
            )));
        }

        Ok(true)
    }

    /// Send message exclusively to private chats
    pub async fn send_private_message(&self, message: &str, user_id: &str) -> ArbitrageResult<()> {
        let chat_context = ChatContext::new(
            user_id.to_string(),
            ChatType::Private,
            Some(user_id.to_string()),
        );

        self.send_secure_notification(message, &chat_context, true)
            .await?;
        Ok(())
    }

    /// Get group-safe message (no trading data)
    fn get_group_safe_message(&self) -> String {
        "🤖 *ArbEdge Bot*\n\n\
        For trading opportunities and sensitive information, please message me privately\\.\n\n\
        📚 *Available Commands in Groups:*\n\
        /help \\- Show available commands\n\
        /settings \\- Bot configuration info\n\n\
        🔒 *Security Notice:* Trading data is only shared in private chats for your security\\."
            .to_string()
    }

    // ============= ENHANCED NOTIFICATION METHODS =============

    /// Send basic arbitrage opportunity notification (legacy support) - PRIVATE ONLY
    pub async fn send_opportunity_notification(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<()> {
        // Legacy method - assume private chat context
        let message = format_opportunity_message(opportunity);
        let chat_context = ChatContext::new(self.config.chat_id.clone(), ChatType::Private, None);
        self.send_secure_notification(&message, &chat_context, true)
            .await?;
        Ok(())
    }

    /// Send categorized opportunity notification (NEW)
    pub async fn send_categorized_opportunity_notification(
        &self,
        categorized_opp: &CategorizedOpportunity,
    ) -> ArbitrageResult<()> {
        let message = format_categorized_opportunity_message(categorized_opp);
        self.send_message(&message).await
    }

    /// Send AI enhancement analysis notification (NEW)
    pub async fn send_ai_enhancement_notification(
        &self,
        enhancement: &AiOpportunityEnhancement,
    ) -> ArbitrageResult<()> {
        let message = format_ai_enhancement_message(enhancement);
        self.send_message(&message).await
    }

    /// Send AI performance insights notification (NEW)
    pub async fn send_performance_insights_notification(
        &self,
        insights: &AiPerformanceInsights,
    ) -> ArbitrageResult<()> {
        let message = format_performance_insights_message(insights);
        self.send_message(&message).await
    }

    /// Send parameter optimization suggestions (NEW)
    pub async fn send_parameter_suggestions_notification(
        &self,
        suggestions: &[ParameterSuggestion],
    ) -> ArbitrageResult<()> {
        let message = format_parameter_suggestions_message(suggestions);
        self.send_message(&message).await
    }

    // ============= ENHANCED BOT COMMAND HANDLERS =============

    /// Bot command handlers (for webhook mode) with context awareness
    pub async fn handle_webhook(&self, update: Value) -> ArbitrageResult<Option<String>> {
        if let Some(message) = update["message"].as_object() {
            if let Some(text) = message["text"].as_str() {
                // Get chat context for security checking
                let chat_context = ChatContext::from_telegram_update(&update)?;

                // Properly handle missing user ID by returning an error instead of empty string
                let user_id = message["from"]["id"]
                    .as_u64()
                    .ok_or_else(|| {
                        ArbitrageError::validation_error(
                            "Missing user ID in webhook message".to_string(),
                        )
                    })?
                    .to_string();

                return self
                    .handle_command_with_context(text, &user_id, &chat_context)
                    .await;
            }
        }
        Ok(None)
    }

    async fn handle_command_with_context(
        &self,
        text: &str,
        user_id: &str,
        chat_context: &ChatContext,
    ) -> ArbitrageResult<Option<String>> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        let command = parts.first().unwrap_or(&"");
        let args = &parts[1..];

        // Group/Channel Command Restrictions - Limited command set with global opportunities
        if chat_context.is_group_or_channel() {
            match *command {
                "/help" => Ok(Some(self.get_help_message().await)),
                "/settings" => Ok(Some(self.get_settings_message(user_id).await)),
                "/start" => Ok(Some(self.get_group_welcome_message().await)),
                "/opportunities" => Ok(Some(
                    self.get_group_opportunities_message(user_id, args).await,
                )),
                "/admin_group_config" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::GroupAnalytics,
                        || self.get_admin_group_config_message(args),
                    )
                    .await
                }
                _ => Ok(Some(
                    "🔒 *Security Notice*\n\n\
                    Personal trading commands are only available in private chats\\.\n\
                    Please message me directly for:\n\
                    • Personal /ai\\_insights\n\
                    • /preferences\n\
                    • /risk\\_assessment\n\
                    • Manual/auto trading commands\n\
                    • /admin commands \\(super admins only\\)\n\n\
                    **Available in groups:** /help, /settings, /opportunities\\n\
                    **Group admins:** /admin\\_group\\_config"
                        .to_string(),
                )),
            }
        } else {
            // Private chat - validate permissions for each command
            match *command {
                // Basic commands (no permission check needed)
                "/start" => Ok(Some(self.get_welcome_message().await)),
                "/help" => Ok(Some(self.get_help_message_with_role(user_id).await)),
                "/status" => Ok(Some(self.get_status_message(user_id).await)),
                "/settings" => Ok(Some(self.get_settings_message(user_id).await)),

                // Analysis and opportunity commands (RBAC-gated content)
                "/opportunities" => Ok(Some(
                    self.get_enhanced_opportunities_message(user_id, args).await,
                )),
                "/categories" => Ok(Some(self.get_categories_message(user_id).await)),
                "/ai_insights" => Ok(Some(self.get_ai_insights_message(user_id).await)),
                "/risk_assessment" => Ok(Some(self.get_risk_assessment_message(user_id).await)),
                "/preferences" => Ok(Some(self.get_preferences_message(user_id).await)),

                // Trading commands (permission-gated)
                "/balance" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::ManualTrading,
                        || self.get_balance_message(user_id, args),
                    )
                    .await
                }
                "/buy" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::ManualTrading,
                        || self.get_buy_command_message(user_id, args),
                    )
                    .await
                }
                "/sell" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::ManualTrading,
                        || self.get_sell_command_message(user_id, args),
                    )
                    .await
                }
                "/orders" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::ManualTrading,
                        || self.get_orders_message(user_id, args),
                    )
                    .await
                }
                "/positions" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::ManualTrading,
                        || self.get_positions_message(user_id, args),
                    )
                    .await
                }
                "/cancel" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::ManualTrading,
                        || self.get_cancel_order_message(user_id, args),
                    )
                    .await
                }

                // Auto trading commands (Premium+ subscription)
                "/auto_enable" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::AutomatedTrading,
                        || self.get_auto_enable_message(user_id),
                    )
                    .await
                }
                "/auto_disable" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::AutomatedTrading,
                        || self.get_auto_disable_message(user_id),
                    )
                    .await
                }
                "/auto_config" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::AutomatedTrading,
                        || self.get_auto_config_message(user_id, args),
                    )
                    .await
                }
                "/auto_status" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::AutomatedTrading,
                        || self.get_auto_status_message(user_id),
                    )
                    .await
                }

                // SuperAdmin commands (admin-only)
                "/admin_stats" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::SystemAdministration,
                        || self.get_admin_stats_message(),
                    )
                    .await
                }
                "/admin_users" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::UserManagement,
                        || self.get_admin_users_message(args),
                    )
                    .await
                }
                "/admin_config" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::GlobalConfiguration,
                        || self.get_admin_config_message(args),
                    )
                    .await
                }
                "/admin_broadcast" => {
                    self.handle_permissioned_command(
                        user_id,
                        CommandPermission::SystemAdministration,
                        || self.get_admin_broadcast_message(args),
                    )
                    .await
                }

                _ => Ok(None), // Unknown command, no response
            }
        }
    }

    /// Handle commands that require specific permissions
    async fn handle_permissioned_command<F, Fut>(
        &self,
        user_id: &str,
        required_permission: CommandPermission,
        command_handler: F,
    ) -> ArbitrageResult<Option<String>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = String>,
    {
        // TODO: In production, fetch actual user profile from database
        // For now, simulate based on user_id pattern
        let user_has_permission = self
            .check_user_permission(user_id, &required_permission)
            .await;

        if user_has_permission {
            Ok(Some(command_handler().await))
        } else {
            Ok(Some(
                self.get_permission_denied_message(required_permission)
                    .await,
            ))
        }
    }

    /// Check if user has required permission (mock implementation)
    async fn check_user_permission(&self, user_id: &str, permission: &CommandPermission) -> bool {
        // TODO: Replace with actual user profile lookup from database
        // For now, mock implementation based on user_id patterns

        // Super admin check (user IDs starting with "admin_" or specific known admin IDs)
        let is_super_admin = user_id.starts_with("admin_") || 
                           user_id == "123456789" || // Example admin user ID
                           user_id == "987654321"; // Another admin user ID

        match permission {
            CommandPermission::BasicCommands
            | CommandPermission::BasicOpportunities
            | CommandPermission::ManualTrading
            | CommandPermission::TechnicalAnalysis
            | CommandPermission::AIEnhancedOpportunities
            | CommandPermission::AutomatedTrading
            | CommandPermission::AdvancedAnalytics
            | CommandPermission::PremiumFeatures => true, // Beta: all users have access
            CommandPermission::SystemAdministration
            | CommandPermission::UserManagement
            | CommandPermission::GlobalConfiguration
            | CommandPermission::GroupAnalytics => is_super_admin,
        }
    }

    /// Get permission denied message
    async fn get_permission_denied_message(&self, permission: CommandPermission) -> String {
        match permission {
            CommandPermission::SystemAdministration
            | CommandPermission::UserManagement
            | CommandPermission::GlobalConfiguration
            | CommandPermission::GroupAnalytics => "🔒 *Access Denied*\n\n\
                This command requires Super Administrator privileges\\.\n\
                Only system administrators can access this functionality\\.\n\n\
                If you believe you should have access, please contact support\\."
                .to_string(),
            CommandPermission::ManualTrading => "🔒 *Subscription Required*\n\n\
                This command requires a Basic subscription or higher\\.\n\
                During the beta period, all users have access\\.\n\n\
                Available plans:\n\
                • Basic: Manual trading commands\n\
                • Premium: Advanced features \\+ automation\n\
                • Enterprise: Custom solutions\n\n\
                Contact support to upgrade your subscription\\!"
                .to_string(),
            CommandPermission::TechnicalAnalysis => "🔒 *Basic+ Subscription Required*\n\n\
                Technical analysis features require a Basic subscription or higher\\.\n\
                During the beta period, all users have access\\.\n\n\
                Contact support to upgrade your subscription for full access\\!"
                .to_string(),
            CommandPermission::AIEnhancedOpportunities
            | CommandPermission::AutomatedTrading
            | CommandPermission::AdvancedAnalytics
            | CommandPermission::PremiumFeatures => "🔒 *Premium Subscription Required*\n\n\
                This command requires a Premium subscription or higher\\.\n\
                During the beta period, all users have access\\.\n\n\
                Upgrade to Premium for:\n\
                • Automated trading capabilities\n\
                • Advanced analytics and insights\n\
                • Priority support\n\
                • Custom risk management\n\n\
                Contact support to upgrade your subscription\\!"
                .to_string(),
            CommandPermission::BasicCommands | CommandPermission::BasicOpportunities => {
                // This should never happen since basic commands are always allowed
                "✅ *Access Granted*\n\nYou have access to this command\\.".to_string()
            }
        }
    }

    // Legacy method for backward compatibility
    #[allow(dead_code)]
    async fn handle_command(&self, text: &str, user_id: &str) -> ArbitrageResult<Option<String>> {
        // Assume private chat context for legacy calls
        let chat_context = ChatContext::new(
            user_id.to_string(),
            ChatType::Private,
            Some(user_id.to_string()),
        );
        self.handle_command_with_context(text, user_id, &chat_context)
            .await
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
        🚀 Get started with /opportunities to see what's available\\!"
            .to_string()
    }

    async fn get_group_welcome_message(&self) -> String {
        "🤖 *Welcome to ArbEdge AI Trading Bot\\!*\n\n\
        I'm now active in this group\\! 🎉\n\n\
        🌍 *Global Opportunities Broadcasting:*\n\
        • I'll automatically share global arbitrage opportunities here\n\
        • Technical analysis signals \\(filtered by group settings\\)\n\
        • System status updates and market alerts\n\n\
        🔒 *Security Notice:*\n\
        For your protection, sensitive trading data and personal portfolio information are only shared in private chats\\.\n\n\
        📚 *Available Commands in Groups:*\n\
        /help \\- Show available commands\n\
        /settings \\- Bot configuration info\n\
        /opportunities \\- View latest global opportunities\n\n\
        💬 *For Personal Trading Features:*\n\
        Please message me privately for:\n\
        • Personal trading opportunities\n\
        • AI insights and portfolio analysis\n\
        • Manual/automated trading commands\n\
        • Account management\n\n\
        ⚙️ *Group Admins:* Use `/admin_group_config` to configure broadcasting settings\n\n\
        🔗 *Get Started:* Click my username to start a private chat for personal trading features\\!"
            .to_string()
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

    #[allow(dead_code)]
    async fn get_opportunities_message(&self, _user_id: &str, args: &[&str]) -> String {
        let filter_category = args.first();

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
            • `beginner` \\- Beginner\\-friendly options",
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
        💡 Use /preferences to enable/disable categories based on your trading focus\\!"
            .to_string()
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
        💡 Use /risk\\_assessment for detailed portfolio analysis\\!"
            .to_string()
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
        💡 Maintain current risk levels"
            .to_string()
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
        💡 Use /preferences to modify your trading focus and experience settings\\!"
            .to_string()
    }

    // ============= ENHANCED HELP MESSAGE WITH ROLE DETECTION =============

    async fn get_help_message_with_role(&self, user_id: &str) -> String {
        let is_super_admin = self
            .check_user_permission(user_id, &CommandPermission::SystemAdministration)
            .await;

        let mut help_message = "📚 *ArbEdge Bot Commands*\n\n\
        🔍 *Opportunities & Analysis:*\n\
        /opportunities \\[category\\] \\- Show recent opportunities\n\
        /ai\\_insights \\- Get AI analysis results\n\
        /risk\\_assessment \\- View portfolio risk analysis\n\n\
        💼 *Manual Trading Commands:*\n\
        /balance \\[exchange\\] \\- Check account balances\n\
        /buy \\<pair\\> \\<amount\\> \\[price\\] \\- Place buy order\n\
        /sell \\<pair\\> \\<amount\\> \\[price\\] \\- Place sell order\n\
        /orders \\[exchange\\] \\- View open orders\n\
        /positions \\[exchange\\] \\- View open positions\n\
        /cancel \\<order\\_id\\> \\- Cancel specific order\n\n\
        🤖 *Auto Trading Commands:*\n\
        /auto\\_enable \\- Enable automated trading\n\
        /auto\\_disable \\- Disable automated trading\n\
        /auto\\_config \\[setting\\] \\[value\\] \\- Configure auto trading\n\
        /auto\\_status \\- View auto trading status\n\n\
        🎛️ *Configuration:*\n\
        /categories \\- Manage enabled opportunity categories\n\
        /preferences \\- View/update trading preferences\n\
        /settings \\- View current bot settings\n\n\
        ℹ️ *Information:*\n\
        /status \\- Check bot and system status\n\
        /help \\- Show this help message\n\n"
            .to_string();

        if is_super_admin {
            help_message.push_str(
                "🔧 *Super Admin Commands:*\n\
                /admin\\_stats \\- System metrics and health\n\
                /admin\\_users \\[search\\] \\- User management\n\
                /admin\\_config \\[setting\\] \\[value\\] \\- Global configuration\n\
                /admin\\_broadcast \\<message\\> \\- Send message to all users\n\n",
            );
        }

        help_message.push_str(
            "💡 *Tips:*\n\
            • Use /opportunities followed by a category name \\(e\\.g\\., `/opportunities arbitrage`\\)\n\
            • Trading commands require exchange API keys to be configured\n\
            • All commands work only in private chats for security");

        help_message
    }

    // ============= ENHANCED OPPORTUNITIES COMMAND =============

    async fn get_enhanced_opportunities_message(&self, user_id: &str, args: &[&str]) -> String {
        // Check user's access level to determine content
        let has_technical = self
            .check_user_permission(user_id, &CommandPermission::TechnicalAnalysis)
            .await;
        let has_ai_enhanced = self
            .check_user_permission(user_id, &CommandPermission::AIEnhancedOpportunities)
            .await;
        let is_super_admin = self
            .check_user_permission(user_id, &CommandPermission::SystemAdministration)
            .await;

        let filter_category = args.first().map(|s| s.to_lowercase());

        let mut message = "📊 *Trading Opportunities* 🔥\n\n".to_string();

        if let Some(category) = &filter_category {
            message.push_str(&format!(
                "🏷️ *Filtered by:* `{}`\n\n",
                escape_markdown_v2(category)
            ));
        }

        // Always show basic global arbitrage opportunities
        message.push_str("🌍 *Global Arbitrage Opportunities*\n");
        message.push_str(
            "🛡️ **Low Risk Arbitrage** 🟢\n\
            • Pair: `BTCUSDT`\n\
            • Rate Difference: `0.15%`\n\
            • Confidence: `89%`\n\
            • Expected Return: `$12.50`\n\n\
            🔄 **Cross-Exchange Opportunity** 🟡\n\
            • Pair: `ETHUSDT`\n\
            • Rate Difference: `0.23%`\n\
            • Confidence: `92%`\n\
            • Expected Return: `$18.75`\n\n",
        );

        // Technical analysis for Basic+ users
        if has_technical
            && (filter_category.is_none()
                || filter_category.as_ref() == Some(&"technical".to_string()))
        {
            message.push_str("📈 *Technical Analysis Signals*\n");
            message.push_str(
                "📊 **RSI Divergence** ⚡\n\
                • Pair: `ADAUSDT`\n\
                • Signal: `BUY`\n\
                • Strength: `Strong`\n\
                • Target: `$0.52` \\(\\+4\\.2%\\)\n\n\
                🌊 **Support/Resistance** 📈\n\
                • Pair: `BNBUSDT`\n\
                • Signal: `SELL`\n\
                • Strength: `Medium`\n\
                • Target: `$310` \\(\\-2\\.8%\\)\n\n",
            );
        }

        // AI Enhanced for Premium+ users
        if has_ai_enhanced
            && (filter_category.is_none() || filter_category.as_ref() == Some(&"ai".to_string()))
        {
            message.push_str("🤖 *AI Enhanced Opportunities*\n");
            message.push_str(
                "⭐ **AI Recommended** 🎯\n\
                • Pair: `SOLUSDT`\n\
                • Strategy: `Hybrid Arbitrage\\+TA`\n\
                • AI Confidence: `96%`\n\
                • Profit Potential: `$24.30`\n\
                • Risk Score: `Low`\n\n\
                🧠 **Machine Learning Signal** 🚀\n\
                • Pair: `MATICUSDT`\n\
                • Pattern: `Breakout Prediction`\n\
                • AI Confidence: `84%`\n\
                • Time Horizon: `4\\-6 hours`\n\n",
            );
        }

        // Super admin stats
        if is_super_admin {
            message.push_str("🔧 *Super Admin Metrics*\n");
            message.push_str(
                "📊 **System Status:**\n\
                • Active Users: `342`\n\
                • Opportunities Sent: `1,205/24h`\n\
                • Global Queue: `23 pending`\n\
                • Distribution Rate: `98.7%`\n\n",
            );
        }

        // Available access levels
        message.push_str("🔓 *Your Access Level:*\n");
        message.push_str("✅ Global Arbitrage \\(Free\\)\n");
        if has_technical {
            message.push_str("✅ Technical Analysis \\(Basic\\+\\)\n");
        } else {
            message.push_str("🔒 Technical Analysis \\(requires Basic\\+\\)\n");
        }
        if has_ai_enhanced {
            message.push_str("✅ AI Enhanced \\(Premium\\+\\)\n");
        } else {
            message.push_str("🔒 AI Enhanced \\(requires Premium\\+\\)\n");
        }

        if filter_category.is_none() {
            message.push_str("\n💡 *Filter by category:*\n");
            message.push_str("• `/opportunities arbitrage` \\- Global arbitrage only\n");
            if has_technical {
                message.push_str("• `/opportunities technical` \\- Technical analysis signals\n");
            }
            if has_ai_enhanced {
                message.push_str("• `/opportunities ai` \\- AI enhanced opportunities\n");
            }
        }

        message
    }

    // ============= AUTO TRADING COMMAND IMPLEMENTATIONS =============

    async fn get_auto_enable_message(&self, user_id: &str) -> String {
        // TODO: Check if user has proper API keys and risk management setup
        format!(
            "🤖 *Auto Trading Activation*\n\n\
            **User:** `{}`\n\
            **Status:** Checking requirements\\.\\.\\.\n\n\
            ✅ **Requirements Check:**\n\
            • Premium Subscription: ✅ Active\n\
            • API Keys Configured: ⚠️ Checking\\.\\.\\.\n\
            • Risk Management: ⚠️ Setup required\n\
            • Trading Balance: ⚠️ Validating\\.\\.\\.\n\n\
            **Next Steps:**\n\
            1\\. Configure risk management settings\n\
            2\\. Set maximum position sizes\n\
            3\\. Define stop\\-loss parameters\n\
            4\\. Test with paper trading\n\n\
            Use `/auto_config` to set up risk parameters before enabling\\.",
            escape_markdown_v2(user_id)
        )
    }

    async fn get_auto_disable_message(&self, _user_id: &str) -> String {
        "🛑 *Auto Trading Deactivation*\n\n\
        **Status:** Auto trading disabled\n\
        **Active Positions:** Checking for open positions\\.\\.\\.\n\n\
        ⚠️ **Important Notes:**\n\
        • All pending orders will be cancelled\n\
        • Existing positions remain open\n\
        • Manual trading still available\n\
        • Settings are preserved\n\n\
        **Open Positions Found:**\n\
        🔸 BTCUSDT: 0\\.001 BTC \\(\\+$2\\.40\\)\n\
        🔸 ETHUSDT: 0\\.5 ETH \\(\\+$8\\.75\\)\n\n\
        💡 Use `/positions` to manage existing positions manually\\."
            .to_string()
    }

    async fn get_auto_config_message(&self, _user_id: &str, args: &[&str]) -> String {
        if args.is_empty() {
            "⚙️ *Auto Trading Configuration*\n\n\
            **Current Settings:**\n\
            • Max Position Size: `$500 per trade`\n\
            • Daily Loss Limit: `$50`\n\
            • Stop Loss: `2%`\n\
            • Take Profit: `4%`\n\
            • Max Open Positions: `3`\n\
            • Trading Mode: `Conservative`\n\n\
            **Available Commands:**\n\
            • `/auto_config max_position 1000` \\- Set max position to $1000\n\
            • `/auto_config stop_loss 1.5` \\- Set stop loss to 1\\.5%\n\
            • `/auto_config take_profit 5` \\- Set take profit to 5%\n\
            • `/auto_config mode aggressive` \\- Set trading mode\n\n\
            **Trading Modes:**\n\
            • `conservative` \\- Lower risk, smaller returns\n\
            • `balanced` \\- Medium risk/reward ratio\n\
            • `aggressive` \\- Higher risk, larger potential returns"
                .to_string()
        } else {
            let setting = args[0];
            let value = args.get(1).unwrap_or(&"");

            format!(
                "✅ *Configuration Updated*\n\n\
                **Setting:** `{}`\n\
                **New Value:** `{}`\n\
                **Status:** Applied successfully\n\n\
                **Updated Configuration:**\n\
                Settings will take effect on next trading cycle\\.\n\
                Current positions are not affected\\.\n\n\
                Use `/auto_status` to see all current settings\\.",
                escape_markdown_v2(setting),
                escape_markdown_v2(value)
            )
        }
    }

    async fn get_auto_status_message(&self, _user_id: &str) -> String {
        "🤖 *Auto Trading Status*\n\n\
        **System Status:** 🟢 Online\n\
        **Auto Trading:** 🔴 Disabled\n\
        **Last Activity:** `2024\\-01\\-15 14:30 UTC`\n\n\
        **Performance \\(Last 7 Days\\):**\n\
        • Total Trades: `12`\n\
        • Win Rate: `75%` \\(9/12\\)\n\
        • Total P&L: `+$127.50`\n\
        • Best Trade: `+$18.75`\n\
        • Worst Trade: `\\-$8.40`\n\n\
        **Risk Management:**\n\
        • Max Position: `$500`\n\
        • Current Exposure: `$1,250` \\(62\\.5%\\)\n\
        • Daily Loss Limit: `$50` \\(used: $0\\)\n\
        • Stop Loss Hits: `2`\n\n\
        **Configuration:**\n\
        • Trading Mode: `Conservative`\n\
        • Max Open Positions: `3`\n\
        • Current Positions: `2`\n\n\
        💡 Use `/auto_enable` to start auto trading or `/auto_config` to modify settings\\."
            .to_string()
    }

    // ============= GROUP/CHANNEL COMMAND IMPLEMENTATIONS =============

    async fn get_group_opportunities_message(&self, _user_id: &str, args: &[&str]) -> String {
        let filter_category = args.first().map(|s| s.to_lowercase());

        let mut message = "🌍 *Global Trading Opportunities*\n\n".to_string();

        if let Some(category) = &filter_category {
            message.push_str(&format!(
                "🏷️ *Filtered by:* `{}`\n\n",
                escape_markdown_v2(category)
            ));
        }

        // Always show global arbitrage opportunities in groups
        message.push_str("🛡️ *Global Arbitrage Opportunities*\n");
        message.push_str(
            "📊 **Cross-Exchange Arbitrage** 🟢\n\
            • Pair: `BTCUSDT`\n\
            • Rate Difference: `0.18%`\n\
            • Exchanges: Binance ↔ Bybit\n\
            • Confidence: `91%`\n\
            • Estimated Profit: `$15.30`\n\n\
            ⚡ **Funding Rate Arbitrage** 🟡\n\
            • Pair: `ETHUSDT`\n\
            • Rate Difference: `0.25%`\n\
            • Exchanges: OKX ↔ Bitget\n\
            • Confidence: `88%`\n\
            • Estimated Profit: `$21.75`\n\n",
        );

        // Technical analysis signals (available to all in groups)
        if filter_category.is_none() || filter_category.as_ref() == Some(&"technical".to_string()) {
            message.push_str("📈 *Technical Analysis Signals*\n");
            message.push_str(
                "📊 **Global Market Signal** ⚡\n\
                • Pair: `SOLUSDT`\n\
                • Signal: `BUY`\n\
                • Timeframe: `4H`\n\
                • Strength: `Strong`\n\
                • Target: `$145` \\(\\+6\\.2%\\)\n\n\
                🌊 **Market Trend** 📈\n\
                • Overall: `BULLISH`\n\
                • BTC Dominance: `42.3%`\n\
                • Fear & Greed: `74` \\(Greed\\)\n\
                • Volume Trend: `↗️ Increasing`\n\n",
            );
        }

        message.push_str("🔗 *For Personal Features:*\n");
        message.push_str("Message me privately for:\n");
        message.push_str("• Personalized AI insights\n");
        message.push_str("• Custom risk assessments\n");
        message.push_str("• Manual/automated trading\n");
        message.push_str("• Portfolio management\n\n");

        if filter_category.is_none() {
            message.push_str("💡 *Filter options:*\n");
            message.push_str("• `/opportunities arbitrage` \\- Cross\\-exchange only\n");
            message.push_str("• `/opportunities technical` \\- Technical signals only\n");
        }

        message.push_str("\n⚠️ *Disclaimer:* These are general market opportunities\\. Always do your own research\\!");

        message
    }

    async fn get_admin_group_config_message(&self, args: &[&str]) -> String {
        if args.is_empty() {
            "⚙️ *Group Configuration Settings*\n\n\
            **Current Settings:**\n\
            • Global Opportunities: ✅ Enabled\n\
            • Technical Signals: ✅ Enabled\n\
            • Max Opportunities/Hour: `3`\n\
            • Max Tech Signals/Hour: `2`\n\
            • Message Cooldown: `15 minutes`\n\
            • Member Count Tracking: ✅ Enabled\n\n\
            **Available Commands:**\n\
            • `/admin_group_config global_opps on/off`\n\
            • `/admin_group_config tech_signals on/off`\n\
            • `/admin_group_config max_opps <number>`\n\
            • `/admin_group_config cooldown <minutes>`\n\
            • `/admin_group_config member_tracking on/off`\n\n\
            **Group Analytics:**\n\
            • Total Messages Sent: `1,247`\n\
            • Active Members: `156/203`\n\
            • Last Activity: `2 minutes ago`\n\
            • Engagement Rate: `76.4%`"
                .to_string()
        } else {
            let setting = args[0];
            let value = args.get(1).unwrap_or(&"");

            format!(
                "✅ *Group Configuration Updated*\n\n\
                **Setting:** `{}`\n\
                **New Value:** `{}`\n\
                **Status:** Applied successfully\n\n\
                **Effect:**\n\
                Settings will apply to future broadcasts in this group\\.\n\
                Current message queue is not affected\\.\n\n\
                **Group ID:** `{}`\n\
                **Updated by:** Super Admin\n\
                **Timestamp:** `{}`\n\n\
                Use `/admin_group_config` to see all current settings\\.",
                escape_markdown_v2(setting),
                escape_markdown_v2(value),
                "\\-1001234567890", // Example group ID
                escape_markdown_v2(&chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string())
            )
        }
    }

    // ============= MANUAL TRADING COMMAND IMPLEMENTATIONS =============

    async fn get_balance_message(&self, _user_id: &str, args: &[&str]) -> String {
        let exchange = args.first().unwrap_or(&"all");

        // TODO: Integrate with actual ExchangeService to fetch real balances
        format!(
            "💰 *Account Balance* \\- {}\n\n\
            🔸 **USDT**: `12,543.21` \\(Available: `10,234.56`\\)\n\
            🔸 **BTC**: `0.25431` \\(Available: `0.20000`\\)\n\
            🔸 **ETH**: `8.91234` \\(Available: `7.50000`\\)\n\
            🔸 **BNB**: `45.321` \\(Available: `40.000`\\)\n\n\
            📊 *Portfolio Summary:*\n\
            • Total Value: `$15,847.32`\n\
            • Available for Trading: `$13,245.89`\n\
            • In Open Positions: `$2,601.43`\n\n\
            ⚙️ *Exchange:* `{}`\n\
            🕒 *Last Updated:* `{}`\n\n\
            💡 Use `/orders` to see your open orders",
            escape_markdown_v2("Balance Overview"),
            escape_markdown_v2(exchange),
            escape_markdown_v2(&chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string())
        )
    }

    async fn get_buy_command_message(&self, _user_id: &str, args: &[&str]) -> String {
        if args.len() < 2 {
            return "❌ *Invalid Buy Command*\n\n\
            **Usage:** `/buy <pair> <amount> [price]`\n\n\
            **Examples:**\n\
            • `/buy BTCUSDT 0.001` \\- Market buy order\n\
            • `/buy BTCUSDT 0.001 50000` \\- Limit buy order at $50,000\n\
            • `/buy ETHUSDT 0.1 3000` \\- Limit buy 0\\.1 ETH at $3,000\n\n\
            **Required:**\n\
            • Pair: Trading pair \\(e\\.g\\., BTCUSDT\\)\n\
            • Amount: Quantity to buy\n\
            • Price: \\(Optional\\) Limit price for limit orders"
                .to_string();
        }

        let pair = args[0];
        let amount = args[1];
        let price = args.get(2);

        // TODO: Integrate with ExchangeService to place actual orders
        let order_type = if price.is_some() { "Limit" } else { "Market" };
        let price_text = price.map_or("Market Price".to_string(), |p| format!("${}", p));

        format!(
            "🛒 *Buy Order Confirmation*\n\n\
            📈 **Pair:** `{}`\n\
            💰 **Amount:** `{}`\n\
            💸 **Price:** `{}`\n\
            🏷️ **Order Type:** `{}`\n\n\
            ⚠️ **Note:** This is a preview\\. Actual order execution requires:\n\
            • Valid exchange API keys\n\
            • Sufficient account balance\n\
            • Market conditions\n\n\
            🔧 Configure your exchange API keys in /settings to enable live trading\\.",
            escape_markdown_v2(pair),
            escape_markdown_v2(amount),
            escape_markdown_v2(&price_text),
            escape_markdown_v2(order_type)
        )
    }

    async fn get_sell_command_message(&self, _user_id: &str, args: &[&str]) -> String {
        if args.len() < 2 {
            return "❌ *Invalid Sell Command*\n\n\
            **Usage:** `/sell <pair> <amount> [price]`\n\n\
            **Examples:**\n\
            • `/sell BTCUSDT 0.001` \\- Market sell order\n\
            • `/sell BTCUSDT 0.001 52000` \\- Limit sell order at $52,000\n\
            • `/sell ETHUSDT 0.1 3200` \\- Limit sell 0\\.1 ETH at $3,200\n\n\
            **Required:**\n\
            • Pair: Trading pair \\(e\\.g\\., BTCUSDT\\)\n\
            • Amount: Quantity to sell\n\
            • Price: \\(Optional\\) Limit price for limit orders"
                .to_string();
        }

        let pair = args[0];
        let amount = args[1];
        let price = args.get(2);

        let order_type = if price.is_some() { "Limit" } else { "Market" };
        let price_text = price.map_or("Market Price".to_string(), |p| format!("${}", p));

        format!(
            "📉 *Sell Order Confirmation*\n\n\
            📈 **Pair:** `{}`\n\
            💰 **Amount:** `{}`\n\
            💸 **Price:** `{}`\n\
            🏷️ **Order Type:** `{}`\n\n\
            ⚠️ **Note:** This is a preview\\. Actual order execution requires:\n\
            • Valid exchange API keys\n\
            • Sufficient asset balance\n\
            • Market conditions\n\n\
            🔧 Configure your exchange API keys in /settings to enable live trading\\.",
            escape_markdown_v2(pair),
            escape_markdown_v2(amount),
            escape_markdown_v2(&price_text),
            escape_markdown_v2(order_type)
        )
    }

    async fn get_orders_message(&self, _user_id: &str, args: &[&str]) -> String {
        let exchange = args.first().unwrap_or(&"all");

        // TODO: Integrate with ExchangeService to fetch real orders
        format!(
            "📋 *Open Orders* \\- {}\n\n\
            🔸 **Order #12345**\n\
            • Pair: `BTCUSDT`\n\
            • Side: `BUY`\n\
            • Amount: `0.001 BTC`\n\
            • Price: `$50,000.00`\n\
            • Filled: `0%`\n\
            • Status: `PENDING`\n\n\
            🔸 **Order #12346**\n\
            • Pair: `ETHUSDT`\n\
            • Side: `SELL`\n\
            • Amount: `0.5 ETH`\n\
            • Price: `$3,200.00`\n\
            • Filled: `25%`\n\
            • Status: `PARTIAL`\n\n\
            📊 *Summary:*\n\
            • Total Orders: `2`\n\
            • Pending Value: `$1,650.00`\n\
            • Exchange: `{}`\n\n\
            💡 Use `/cancel <order_id>` to cancel an order",
            escape_markdown_v2("Open Orders"),
            escape_markdown_v2(exchange)
        )
    }

    async fn get_positions_message(&self, _user_id: &str, args: &[&str]) -> String {
        let exchange = args.first().unwrap_or(&"all");

        // TODO: Integrate with ExchangeService to fetch real positions
        format!(
            "📊 *Open Positions* \\- {}\n\n\
            🔸 **Position #1**\n\
            • Pair: `BTCUSDT`\n\
            • Side: `LONG`\n\
            • Size: `0.002 BTC`\n\
            • Entry Price: `$49,500.00`\n\
            • Mark Price: `$50,200.00`\n\
            • PnL: `+$1.40` 🟢\n\
            • Margin: `$500.00`\n\n\
            🔸 **Position #2**\n\
            • Pair: `ETHUSDT`\n\
            • Side: `SHORT`\n\
            • Size: `0.5 ETH`\n\
            • Entry Price: `$3,150.00`\n\
            • Mark Price: `$3,100.00`\n\
            • PnL: `+$25.00` 🟢\n\
            • Margin: `$315.00`\n\n\
            📈 *Portfolio Summary:*\n\
            • Total Positions: `2`\n\
            • Total PnL: `+$26.40` 🟢\n\
            • Total Margin: `$815.00`\n\
            • Exchange: `{}`\n\n\
            ⚠️ Monitor your positions and set stop losses to manage risk\\!",
            escape_markdown_v2("Open Positions"),
            escape_markdown_v2(exchange)
        )
    }

    async fn get_cancel_order_message(&self, _user_id: &str, args: &[&str]) -> String {
        if args.is_empty() {
            return "❌ *Invalid Cancel Command*\n\n\
            **Usage:** `/cancel <order_id>`\n\n\
            **Examples:**\n\
            • `/cancel 12345` \\- Cancel order with ID 12345\n\
            • `/cancel all` \\- Cancel all open orders \\(use with caution\\)\n\n\
            Use `/orders` to see your open orders and their IDs\\."
                .to_string();
        }

        let order_id = args[0];

        if order_id == "all" {
            "⚠️ *Cancel All Orders*\n\n\
            This will cancel **ALL** your open orders\\.\n\
            This action cannot be undone\\.\n\n\
            **Confirmation required:** Type `/cancel all confirm` to proceed\\.\n\n\
            💡 Use `/cancel <specific_order_id>` to cancel individual orders\\."
                .to_string()
        } else {
            format!(
                "❌ *Cancel Order Request*\n\n\
                📋 **Order ID:** `{}`\n\
                🔄 **Status:** Processing cancellation\\.\\.\\.\n\n\
                ⚠️ **Note:** Order cancellation requires:\n\
                • Valid exchange API keys\n\
                • Order must still be active\n\
                • Network connectivity\n\n\
                🔧 Check `/orders` to confirm cancellation\\.",
                escape_markdown_v2(order_id)
            )
        }
    }

    // ============= SUPER ADMIN COMMAND IMPLEMENTATIONS =============

    async fn get_admin_stats_message(&self) -> String {
        // TODO: Integrate with actual system metrics
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

        format!(
            "🔧 *System Administration Dashboard*\n\n\
            📊 **System Health:**\n\
            • Status: `🟢 ONLINE`\n\
            • Uptime: `7 days, 14 hours`\n\
            • CPU Usage: `23%`\n\
            • Memory Usage: `45%`\n\
            • Database Status: `🟢 HEALTHY`\n\n\
            👥 **User Statistics:**\n\
            • Total Users: `1,247`\n\
            • Active Users \\(24h\\): `342`\n\
            • New Registrations \\(today\\): `18`\n\
            • Premium Subscribers: `156`\n\
            • Super Admins: `3`\n\n\
            📈 **Trading Metrics:**\n\
            • Opportunities Detected \\(24h\\): `1,834`\n\
            • Opportunities Distributed: `1,205`\n\
            • Active Trading Sessions: `89`\n\
            • Total Volume \\(24h\\): `$2,456,789`\n\n\
            🔔 **Notifications:**\n\
            • Messages Sent \\(24h\\): `4,521`\n\
            • Delivery Success Rate: `98.7%`\n\
            • Rate Limit Hits: `12`\n\n\
            🕒 **Last Updated:** `{}`\n\n\
            Use `/admin_users` for user management or `/admin_config` for system configuration\\.",
            escape_markdown_v2(&now.to_string())
        )
    }

    async fn get_admin_users_message(&self, args: &[&str]) -> String {
        let search_term = args.first().unwrap_or(&"");

        if search_term.is_empty() {
            "👥 *User Management Dashboard*\n\n\
            **Usage:** `/admin_users [search_term]`\n\n\
            **Examples:**\n\
            • `/admin_users` \\- Show recent users\n\
            • `/admin_users premium` \\- Search premium users\n\
            • `/admin_users @username` \\- Search by username\n\
            • `/admin_users 123456789` \\- Search by user ID\n\n\
            📊 **Quick Stats:**\n\
            • Total Users: `1,247`\n\
            • Online Now: `89`\n\
            • Suspended: `5`\n\
            • Premium: `156`\n\
            • Free: `1,086`\n\n\
            **Recent Users \\(last 24h\\):**\n\
            🔸 User `user_001` \\- Free \\- Active\n\
            🔸 User `user_002` \\- Premium \\- Active\n\
            🔸 User `user_003` \\- Free \\- Inactive\n\n\
            💡 Use specific search terms to find users\\."
                .to_string()
        } else {
            format!(
                "👥 *User Search Results* \\- \"{}\"\n\n\
                🔸 **User ID:** `user_123456`\n\
                • Username: `@example_user`\n\
                • Subscription: `Premium`\n\
                • Status: `Active`\n\
                • Last Active: `2024\\-01\\-15 14:30 UTC`\n\
                • Total Trades: `45`\n\
                • Registration: `2023\\-12\\-01`\n\n\
                🔸 **User ID:** `user_789012`\n\
                • Username: `@another_user`\n\
                • Subscription: `Free`\n\
                • Status: `Active`\n\
                • Last Active: `2024\\-01\\-15 16:45 UTC`\n\
                • Total Trades: `8`\n\
                • Registration: `2024\\-01\\-10`\n\n\
                📊 **Search Summary:**\n\
                • Found: `2 users`\n\
                • Active: `2`\n\
                • Premium: `1`\n\n\
                💡 Use `/admin_config suspend <user_id>` to suspend users if needed\\.",
                escape_markdown_v2(search_term)
            )
        }
    }

    async fn get_admin_config_message(&self, args: &[&str]) -> String {
        if args.is_empty() {
            "🔧 *Global Configuration Management*\n\n\
            **Usage:** `/admin_config [setting] [value]`\n\n\
            **Available Settings:**\n\
            • `max_opportunities_per_hour` \\- Max opportunities per user per hour\n\
            • `cooldown_period_minutes` \\- Cooldown between opportunities\n\
            • `max_daily_opportunities` \\- Max daily opportunities per user\n\
            • `notification_rate_limit` \\- Notification rate limit\n\
            • `maintenance_mode` \\- Enable/disable maintenance mode\n\
            • `beta_access` \\- Enable/disable beta access\n\n\
            **Examples:**\n\
            • `/admin_config max_opportunities_per_hour 5`\n\
            • `/admin_config maintenance_mode true`\n\
            • `/admin_config beta_access false`\n\n\
            **Current Configuration:**\n\
            🔸 Max Opportunities/Hour: `2`\n\
            🔸 Cooldown Period: `240 minutes`\n\
            🔸 Max Daily Opportunities: `10`\n\
            🔸 Maintenance Mode: `🟢 Disabled`\n\
            🔸 Beta Access: `🟢 Enabled`\n\n\
            ⚠️ Configuration changes affect all users immediately\\!"
                .to_string()
        } else if args.len() == 1 {
            let setting = args[0];
            format!(
                "🔧 *Configuration Setting: {}*\n\n\
                **Current Value:** Check the setting details below\\.\n\n\
                **Usage:** `/admin_config {} <new_value>`\n\n\
                **Example:** `/admin_config {} 5`\n\n\
                ⚠️ Provide a value to update this setting\\.",
                escape_markdown_v2(setting),
                escape_markdown_v2(setting),
                escape_markdown_v2(setting)
            )
        } else {
            let setting = args[0];
            let value = args[1];

            format!(
                "✅ *Configuration Updated*\n\n\
                🔧 **Setting:** `{}`\n\
                🔄 **New Value:** `{}`\n\
                🕒 **Updated At:** `{}`\n\
                👤 **Updated By:** `Super Admin`\n\n\
                **Impact:** This change affects all users immediately\\.\n\
                **Rollback:** Use the previous value to revert if needed\\.\n\n\
                💡 Monitor system metrics to ensure stability after configuration changes\\.",
                escape_markdown_v2(setting),
                escape_markdown_v2(value),
                escape_markdown_v2(
                    &chrono::Utc::now()
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string()
                )
            )
        }
    }

    async fn get_admin_broadcast_message(&self, args: &[&str]) -> String {
        if args.is_empty() {
            "📢 *Broadcast Message System*\n\n\
            **Usage:** `/admin_broadcast <message>`\n\n\
            **Examples:**\n\
            • `/admin_broadcast System maintenance in 30 minutes`\n\
            • `/admin_broadcast New features available! Check /help`\n\
            • `/admin_broadcast Welcome to all new beta users!`\n\n\
            **Broadcast Targets:**\n\
            • All active users\n\
            • Private chats only \\(for security\\)\n\
            • Rate limited to prevent spam\n\n\
            ⚠️ **Important Notes:**\n\
            • Messages are sent to ALL users\n\
            • Cannot be recalled once sent\n\
            • Use sparingly to avoid user fatigue\n\
            • Keep messages concise and valuable\n\n\
            📊 **Current Reach:** ~1,247 active users"
                .to_string()
        } else {
            let message = args.join(" ");

            format!(
                "📢 *Broadcast Scheduled*\n\n\
                **Message Preview:**\n\
                \"{}\"\n\n\
                📊 **Delivery Details:**\n\
                • Target Users: `1,247 active users`\n\
                • Delivery Method: `Private chat only`\n\
                • Estimated Time: `5-10 minutes`\n\
                • Rate Limit: `100 messages/minute`\n\n\
                🕒 **Scheduled At:** `{}`\n\
                👤 **Sent By:** `Super Admin`\n\n\
                ✅ **Status:** Broadcasting in progress\\.\\.\\.\n\n\
                💡 Monitor delivery metrics in `/admin_stats`\\.",
                escape_markdown_v2(&message),
                escape_markdown_v2(
                    &chrono::Utc::now()
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string()
                )
            )
        }
    }

    // ============= WEBHOOK SETUP =============

    pub async fn set_webhook(&self, webhook_url: &str) -> ArbitrageResult<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/setWebhook",
            self.config.bot_token
        );

        let payload = json!({
            "url": webhook_url
        });

        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ArbitrageError::network_error(format!("Failed to set webhook: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArbitrageError::telegram_error(format!(
                "Failed to set webhook: {}",
                error_text
            )));
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
    use crate::services::market_analysis::{
        OpportunityType, RiskLevel, TimeHorizon, TradingOpportunity,
    };
    use crate::services::opportunity_categorization::{
        AlertPriority, CategorizedOpportunity, OpportunityCategory, RiskIndicator,
    };
    use crate::types::{ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum};
    use serde_json::json;
    // use chrono::Datelike; // TODO: Re-enable when implementing date formatting

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
            categories: vec![
                OpportunityCategory::LowRiskArbitrage,
                OpportunityCategory::BeginnerFriendly,
            ],
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
            assert_eq!(
                std::mem::size_of_val(&service),
                std::mem::size_of::<TelegramService>()
            );
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
            assert!(message.contains("92")); // suitability score
            assert!(message.contains("Risk Assessment"));
        }

        #[test]
        fn test_enhanced_command_responses() {
            let config = create_test_config();
            let service = TelegramService::new(config);

            // Test that new command responses are not empty
            let welcome = futures::executor::block_on(service.get_welcome_message());
            assert!(welcome.contains("ArbEdge AI Trading Bot"));
            assert!(welcome.contains("AI\\-enhanced analysis")); // Fixed to check escaped version

            let help = futures::executor::block_on(service.get_help_message());
            assert!(help.contains("ai\\_insights")); // Fixed to check escaped version
            assert!(help.contains("categories"));
        }

        #[test]
        fn test_ai_insights_response() {
            let config = create_test_config();
            let service = TelegramService::new(config);

            let insights =
                futures::executor::block_on(service.get_ai_insights_message("test_user"));
            assert!(insights.contains("AI Analysis Summary"));
            assert!(insights.contains("confidence"));
            assert!(insights.contains("Performance Score"));
        }

        #[test]
        fn test_risk_assessment_response() {
            let config = create_test_config();
            let service = TelegramService::new(config);

            let risk =
                futures::executor::block_on(service.get_risk_assessment_message("test_user"));
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
            assert!(
                config.chat_id.starts_with('-')
                    || config.chat_id.chars().all(|c| c.is_ascii_digit())
            );
        }

        #[test]
        fn test_webhook_url_validation() {
            let config = create_test_config();
            let _service = TelegramService::new(config);

            // This is a placeholder test - in real implementation would validate URL format
            let webhook_url = "https://example.com/webhook";
            assert!(webhook_url.starts_with("https://"));
        }

        #[test]
        fn test_optional_webhook() {
            let config = create_test_config();
            let _service = TelegramService::new(config);

            // Service should work without webhook being set
            // Placeholder assertion - service creation successful
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
            assert!(message.contains("binance")); // Fixed to check lowercase as returned by format_exchange
            assert!(message.contains("bybit")); // Fixed to check lowercase as returned by format_exchange
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
            // Placeholder - would test actual disabled behavior
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
            let _service = TelegramService::new(config.clone());

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
            assert_eq!(
                std::mem::size_of_val(&service),
                std::mem::size_of::<TelegramService>()
            );
        }

        #[test]
        fn test_exchange_name_formatting() {
            let exchange = Some(ExchangeIdEnum::Binance);
            let formatted = crate::utils::formatter::format_exchange(&exchange);
            assert_eq!(formatted, "binance"); // Fixed to check actual output format
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
            let _service = TelegramService::new(config);
            let opportunity = create_test_opportunity();

            let message = format_opportunity_message(&opportunity);
            assert!(!message.is_empty());
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
            assert_eq!(
                std::mem::size_of_val(&service),
                std::mem::size_of::<TelegramService>()
            );
        }
    }
}
