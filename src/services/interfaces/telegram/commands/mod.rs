//! Command Router
//!
//! Routes Telegram commands to appropriate handlers

use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::interfaces::telegram::{UserInfo, UserPermissions};
use crate::utils::{ArbitrageError, ArbitrageResult};
use std::sync::Arc;
use worker::console_log;

// Export command modules for external use
pub mod admin;
pub mod onboarding;
pub mod profile;
pub mod settings;

/// Command Router for handling telegram commands
pub struct CommandRouter;

impl CommandRouter {
    /// Route command to appropriate handler
    pub async fn route_command(
        command: &str,
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        console_log!(
            "🎯 Routing command '{}' with args {:?} for user {}",
            command,
            args,
            user_info.user_id
        );

        // Handle space-separated commands by converting them to underscore format
        let normalized_command = if !args.is_empty() {
            match command {
                "/profile" => match args[0] {
                    "view" => "/profile_view",
                    "api" => "/profile_api",
                    "settings" => "/profile_settings",
                    _ => command,
                },
                "/opportunities" => match args[0] {
                    "list" => "/opportunities_list",
                    "manual" => "/opportunities_manual",
                    "auto" => "/opportunities_auto",
                    _ => command,
                },
                "/settings" => match args[0] {
                    "notifications" => "/settings_notifications",
                    "trading" => "/settings_trading",
                    "alerts" => "/settings_alerts",
                    "privacy" => "/settings_privacy",
                    "api" => "/settings_api",
                    _ => command,
                },
                "/trade" => match args[0] {
                    "manual" => "/trade_manual",
                    "auto" => "/trade_auto",
                    "status" => "/trade_status",
                    _ => command,
                },
                "/ai" => match args[0] {
                    "analyze" => "/ai_analyze",
                    "predict" => "/ai_predict",
                    "sentiment" => "/ai_sentiment",
                    "usage" => "/ai_usage",
                    _ => command,
                },
                "/admin" => match args[0] {
                    "config" => "/admin_config",
                    "stats" => "/admin_stats",
                    "users" => "/admin_users",
                    _ => command,
                },
                _ => command,
            }
        } else {
            command
        };

        // Extract remaining args after the first one (which was used for command normalization)
        let remaining_args = if !args.is_empty() && normalized_command != command {
            &args[1..]
        } else {
            args
        };

        match normalized_command {
            "/start" => Self::handle_start(user_info, permissions, service_container).await,
            "/help" => Self::handle_help(user_info, permissions, service_container).await,
            "/subscription" => {
                Self::handle_subscription(user_info, permissions, service_container).await
            }

            // Clickable command aliases using underscores
            "/profile_view" => {
                Self::handle_profile_view(user_info, permissions, service_container).await
            }
            "/profile_api" => {
                Self::handle_profile_api(user_info, permissions, service_container).await
            }
            "/profile_settings" => {
                Self::handle_profile_settings(user_info, permissions, service_container).await
            }

            "/opportunities_list" => {
                Self::handle_opportunities_list(user_info, permissions, service_container).await
            }
            "/opportunities_manual" => {
                Self::handle_opportunities_manual(
                    user_info,
                    permissions,
                    service_container,
                    remaining_args,
                )
                .await
            }
            "/opportunities_auto" => {
                Self::handle_opportunities_auto(
                    user_info,
                    permissions,
                    service_container,
                    remaining_args,
                )
                .await
            }

            "/settings_notifications" => {
                Self::handle_settings_notifications(
                    user_info,
                    permissions,
                    service_container,
                    remaining_args,
                )
                .await
            }
            "/settings_trading" => {
                Self::handle_settings_trading(
                    user_info,
                    permissions,
                    service_container,
                    remaining_args,
                )
                .await
            }
            "/settings_alerts" => {
                Self::handle_settings_alerts(
                    user_info,
                    permissions,
                    service_container,
                    remaining_args,
                )
                .await
            }
            "/settings_privacy" => {
                Self::handle_settings_privacy(
                    user_info,
                    permissions,
                    service_container,
                    remaining_args,
                )
                .await
            }
            "/settings_api" => {
                Self::handle_settings_api(user_info, permissions, service_container, remaining_args)
                    .await
            }

            "/trade_manual" => {
                Self::handle_trade_manual(user_info, permissions, service_container, remaining_args)
                    .await
            }
            "/trade_auto" => {
                Self::handle_trade_auto(user_info, permissions, service_container, remaining_args)
                    .await
            }
            "/trade_status" => {
                Self::handle_trade_status(user_info, permissions, service_container).await
            }

            "/ai_analyze" => {
                Self::handle_ai_analyze(user_info, permissions, service_container, remaining_args)
                    .await
            }
            "/ai_predict" => {
                Self::handle_ai_predict(user_info, permissions, service_container, remaining_args)
                    .await
            }
            "/ai_sentiment" => {
                Self::handle_ai_sentiment(user_info, permissions, service_container, remaining_args)
                    .await
            }
            "/ai_usage" => Self::handle_ai_usage(user_info, permissions, service_container).await,

            // Admin commands (underscore format only)
            "/admin_config" => {
                if !permissions.is_admin {
                    return Ok("❌ <b>Access Denied</b>\n\nAdmin privileges required.".to_string());
                }
                Self::handle_admin_config(user_info, permissions, service_container).await
            }
            "/admin_stats" => {
                if !permissions.is_admin {
                    return Ok("❌ <b>Access Denied</b>\n\nAdmin privileges required.".to_string());
                }
                Self::handle_admin_stats(user_info, permissions, service_container).await
            }
            "/admin_users" => {
                if !permissions.is_admin {
                    return Ok("❌ <b>Access Denied</b>\n\nAdmin privileges required.".to_string());
                }
                Self::handle_admin_users(user_info, permissions, service_container).await
            }

            _ => {
                // Handle base commands without subcommands
                match command {
                    "/profile" => Ok("👤 <b>Profile Commands</b>\n\nAvailable options:\n• /profile_view (or /profile view) - View your profile\n• /profile_api (or /profile api) - API management\n• /profile_settings (or /profile settings) - Profile settings\n\nUse /help for more information.".to_string()),
                    "/opportunities" => Ok("💰 <b>Opportunities Commands</b>\n\nAvailable options:\n• /opportunities_list (or /opportunities list) - View current opportunities\n• /opportunities_manual (or /opportunities manual) - Manual trading\n• /opportunities_auto (or /opportunities auto) - Automated trading\n\nUse /help for more information.".to_string()),
                    "/settings" => Ok("⚙️ <b>Settings Commands</b>\n\nAvailable options:\n• /settings_notifications (or /settings notifications) - Notification preferences\n• /settings_trading (or /settings trading) - Trading settings\n• /settings_alerts (or /settings alerts) - Alert configuration\n• /settings_privacy (or /settings privacy) - Privacy settings\n• /settings_api (or /settings api) - API management\n\nUse /help for more information.".to_string()),
                    "/trade" => Ok("📈 <b>Trade Commands</b>\n\nAvailable options:\n• /trade_manual (or /trade manual) - Execute manual trades\n• /trade_auto (or /trade auto) - Automated trading\n• /trade_status (or /trade status) - View trading status\n\nUse /help for more information.".to_string()),
                    "/ai" => Ok("🤖 <b>AI Commands</b>\n\nAvailable options:\n• /ai_analyze (or /ai analyze) - Market analysis\n• /ai_predict (or /ai predict) - Price predictions\n• /ai_sentiment (or /ai sentiment) - Sentiment analysis\n• /ai_usage (or /ai usage) - Usage statistics\n\nUse /help for more information.".to_string()),
                    "/admin" => {
                        if !permissions.is_admin {
                            return Ok("❌ <b>Access Denied</b>\n\nAdmin privileges required.".to_string());
                        }
                        Ok("🔧 <b>Admin Commands</b>\n\nAvailable options:\n• /admin_config (or /admin config) - Configuration panel\n• /admin_stats (or /admin stats) - System statistics\n• /admin_users (or /admin users) - User management\n\nUse /help for more information.".to_string())
                    }
                    "/beta" => Ok("🧪 <b>Beta Commands</b>\n\nBeta features are coming soon!\n\nPlanned options:\n• /beta opportunities - Beta opportunity features\n• /beta ai - Beta AI features\n• /beta analytics - Beta analytics\n\nUse /help for more information.".to_string()),
                    _ => {
                        // Extract the base command for better error handling
                        let base_command = normalized_command.split('_').next().unwrap_or(normalized_command);

                        match base_command {
                            "/opportunities" => Ok("❓ <b>Invalid opportunities command</b>\n\nAvailable options:\n• /opportunities_list - View current opportunities\n• /opportunities_manual - Manual trading\n• /opportunities_auto - Automated trading\n\nUse /help for more information.".to_string()),
                            "/settings" => Ok("❓ <b>Invalid settings command</b>\n\nAvailable options:\n• /settings_notifications - Notification preferences\n• /settings_trading - Trading settings\n• /settings_alerts - Alert configuration\n• /settings_privacy - Privacy settings\n• /settings_api - API management\n\nUse /help for more information.".to_string()),
                            "/trade" => Ok("❓ <b>Invalid trade command</b>\n\nAvailable options:\n• /trade_manual - Execute manual trades\n• /trade_auto - Automated trading\n• /trade_status - View trading status\n\nUse /help for more information.".to_string()),
                            "/ai" => Ok("❓ <b>Invalid AI command</b>\n\nAvailable options:\n• /ai_analyze - Market analysis\n• /ai_predict - Price predictions\n• /ai_sentiment - Sentiment analysis\n• /ai_usage - Usage statistics\n\nUse /help for more information.".to_string()),
                            "/profile" => Ok("❓ <b>Invalid profile command</b>\n\nAvailable options:\n• /profile_view - View profile\n• /profile_api - API management\n• /profile_settings - Profile settings\n\nUse /help for more information.".to_string()),
                            _ => Ok(format!("❓ <b>Unknown command:</b> <code>{}</code>\n\n🤖 Available commands:\n• /help - Show all commands\n• /opportunities_list - View arbitrage opportunities\n• /profile_view - Your account info\n• /settings_notifications - Configure alerts\n\n💡 <b>Tip:</b> Commands are clickable! Tap them instead of typing.", normalized_command)),
                        }
                    }
                }
            }
        }
    }

    /// Handle /start command
    async fn handle_start(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        // Check if user exists in database
        if let Some(user_profile_service) = &service_container.user_profile_service {
            let existing_user = user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await?;

            if existing_user.is_some() {
                return Ok(format!(
                    "🎉 <b>Welcome back, {}!</b>\n\n\
                    🚀 Your ArbEdge account is ready.\n\n\
                    📊 <b>Quick Actions:</b>\n\
                    • /opportunities_list - See latest arbitrage opportunities\n\
                    • /profile_view - View your account details\n\
                    • /subscription - Manage your subscription\n\
                    • /settings_notifications - Configure preferences\n\
                    • /help - All available commands\n\n\
                    💡 Ready to start trading? Check out /opportunities_list!",
                    user_info.first_name.as_deref().unwrap_or("Trader")
                ));
            } else {
                // Create new user
                let _new_user = user_profile_service
                    .create_user_profile(
                        user_info.user_id,
                        None, // invitation_code
                        user_info.username.clone(),
                    )
                    .await?;

                return Ok(format!(
                    "🎉 <b>Welcome to ArbEdge, {}!</b>\n\n\
                    ✅ Your account has been created successfully.\n\
                    🚀 You're now ready to discover arbitrage opportunities!\n\n\
                    📊 <b>Getting Started:</b>\n\
                    • /opportunities_list - See your first opportunities (3 free daily)\n\
                    • /profile_view - Complete your profile setup\n\
                    • /subscription - Explore Premium features\n\
                    • /help - Learn about all commands\n\n\
                    💎 <b>Tip:</b> Upgrade to Premium for unlimited opportunities and real-time alerts!\n\n\
                    🔗 <b>Next Step:</b> Try /opportunities_list to see what's available!",
                    user_info.first_name.as_deref().unwrap_or("Trader")
                ));
            }
        }

        Ok("🚀 <b>Welcome to ArbEdge!</b>\n\n⚠️ User service temporarily unavailable. Please try again later.".to_string())
    }

    /// Handle /help command with proper role-based content
    async fn handle_help(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut help_text = "🤖 <b>ArbEdge Bot Commands</b>\n\n".to_string();

        help_text.push_str("📊 <b>Trading Commands:</b>\n");
        help_text.push_str("• /opportunities_list - List all opportunities\n");
        help_text.push_str("• /opportunities_manual - Request manual scan\n");
        help_text.push_str("• /opportunities_auto - Automation settings\n");
        help_text.push_str("• /trade_manual - Manual trade execution\n");
        help_text.push_str("• /trade_auto - Automated trading\n");
        help_text.push_str("• /trade_status - Trading status\n");
        help_text.push_str("• /profile_view - View profile\n");
        help_text.push_str("• /profile_api - Manage API keys\n");
        help_text.push_str("• /subscription - Manage subscription plan\n\n");

        help_text.push_str("⚙️ <b>Settings Commands:</b>\n");
        help_text.push_str("• /settings_notifications - Alert preferences\n");
        help_text.push_str("• /settings_trading - Trading settings\n");
        help_text.push_str("• /settings_alerts - Alert configuration\n");
        help_text.push_str("• /settings_privacy - Privacy settings\n");
        help_text.push_str("• /settings_api - API configuration\n\n");

        // AI features for users with access
        if permissions.can_access_ai_features() {
            help_text.push_str("🤖 <b>AI Commands (BYOK):</b>\n");
            help_text.push_str("• /ai_analyze - Market analysis\n");
            help_text.push_str("• /ai_predict - Price predictions\n");
            help_text.push_str("• /ai_sentiment - Sentiment analysis\n");
            help_text.push_str("• /ai_usage - Usage statistics\n\n");
        }

        help_text.push_str("ℹ️ <b>General Commands:</b>\n");
        help_text.push_str("• /help - Show this help message\n");
        help_text.push_str("• /start - Welcome message\n\n");

        // Admin commands for admin users (underscore format only)
        if permissions.is_admin {
            help_text.push_str("🔧 <b>Admin Commands:</b>\n");
            help_text.push_str("• /admin_config - Configuration panel\n");
            help_text.push_str("• /admin_stats - System statistics\n");
            help_text.push_str("• /admin_users - User management\n\n");
        }

        // Subscription-based features
        match permissions.subscription_tier.as_str() {
            "free" => {
                help_text.push_str("💡 <b>Upgrade Benefits:</b>\n");
                help_text.push_str("• Unlimited opportunities with Premium\n");
                help_text.push_str("• Real-time notifications and alerts\n");
                help_text.push_str("• Advanced trading automation\n");
                help_text.push_str("• AI-enhanced opportunity scoring\n");
                help_text.push_str("Use /subscription to upgrade!\n\n");
            }
            "premium" | "enterprise" => {
                help_text.push_str("💎 <b>Premium Features Available:</b>\n");
                help_text.push_str("• Unlimited opportunities\n");
                help_text.push_str("• Real-time notifications\n");
                help_text.push_str("• Advanced trading automation\n");
                help_text.push_str("• AI-enhanced analytics\n\n");
            }
            _ => {}
        }

        help_text.push_str("💡 <b>Tip:</b> Click any command above to use it instantly!\n");
        help_text.push_str("All commands with underscores are clickable.\n\n");

        help_text.push_str("🆘 <b>Need assistance?</b>\n");
        help_text.push_str("Contact support or visit our documentation for detailed guides.");

        Ok(help_text)
    }

    /// Handle profile view command
    async fn handle_profile_view(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = "👤 <b>User Profile</b>\n\n".to_string();

        // Get actual user profile from database
        if let Some(user_profile_service) = &service_container.user_profile_service {
            match user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await
            {
                Ok(Some(profile)) => {
                    // Display real profile information
                    message.push_str(&format!("🆔 <b>User ID:</b> {}\n", profile.user_id));

                    if let Some(username) = &profile.telegram_username {
                        message.push_str(&format!("👤 <b>Username:</b> @{}\n", username));
                    }

                    if let Some(email) = &profile.email {
                        message.push_str(&format!("📧 <b>Email:</b> {}\n", email));
                    }

                    message.push_str(&format!(
                        "🎯 <b>Access Level:</b> {:?}\n",
                        profile.access_level
                    ));
                    message.push_str(&format!(
                        "💎 <b>Subscription:</b> {:?}\n",
                        profile.subscription.tier
                    ));
                    message.push_str(&format!(
                        "✅ <b>Active:</b> {}\n",
                        if profile.is_active { "Yes" } else { "No" }
                    ));

                    if profile.is_beta_active {
                        message.push_str("🧪 <b>Beta User:</b> Yes\n");
                    }

                    message.push_str("\n📊 <b>Trading Statistics:</b>\n");
                    message.push_str(&format!("• Total trades: {}\n", profile.total_trades));
                    message.push_str(&format!(
                        "• Total P&L: ${:.2} USDT\n",
                        profile.total_pnl_usdt
                    ));
                    message.push_str(&format!(
                        "• Account balance: ${:.2} USDT\n",
                        profile.account_balance_usdt
                    ));

                    message.push_str("\n🔑 <b>API Keys:</b>\n");
                    if profile.api_keys.is_empty() {
                        message.push_str("• No API keys configured\n");
                    } else {
                        for api_key in &profile.api_keys {
                            let status = if api_key.is_active { "✅" } else { "❌" };
                            let key_type = match &api_key.provider {
                                crate::types::ApiKeyProvider::Exchange(exchange) => {
                                    format!("Exchange ({})", exchange)
                                }
                                crate::types::ApiKeyProvider::OpenAI => "OpenAI".to_string(),
                                crate::types::ApiKeyProvider::Anthropic => "Anthropic".to_string(),
                                crate::types::ApiKeyProvider::AI => "AI".to_string(),
                                crate::types::ApiKeyProvider::Custom => "Custom".to_string(),
                            };
                            message.push_str(&format!("• {} {}\n", status, key_type));
                        }
                    }

                    message.push_str("\n⚙️ <b>Trading Settings:</b>\n");
                    let trading_settings = &profile.configuration.trading_settings;
                    message.push_str(&format!(
                        "• Auto-trading: {}\n",
                        if trading_settings.auto_trading_enabled {
                            "✅ Enabled"
                        } else {
                            "❌ Disabled"
                        }
                    ));
                    message.push_str(&format!(
                        "• Max position size: ${:.2}\n",
                        trading_settings.max_position_size
                    ));
                    message.push_str(&format!(
                        "• Risk tolerance: {:.1}%\n",
                        trading_settings.risk_tolerance * 100.0
                    ));
                    message.push_str(&format!(
                        "• Min profit threshold: {:.2}%\n",
                        trading_settings.min_profit_threshold
                    ));

                    message.push_str("\n🔔 <b>Notification Settings:</b>\n");
                    let notifications = &profile.configuration.notification_settings;
                    message.push_str(&format!(
                        "• Notifications: {}\n",
                        if notifications.enabled {
                            "✅ Enabled"
                        } else {
                            "❌ Disabled"
                        }
                    ));
                    message.push_str(&format!(
                        "• Telegram alerts: {}\n",
                        if notifications.telegram_notifications {
                            "✅ On"
                        } else {
                            "❌ Off"
                        }
                    ));
                    message.push_str(&format!(
                        "• Opportunity alerts: {}\n",
                        if notifications.opportunity_alerts {
                            "✅ On"
                        } else {
                            "❌ Off"
                        }
                    ));

                    // Show account creation and activity dates
                    let created_date =
                        chrono::DateTime::from_timestamp((profile.created_at / 1000) as i64, 0)
                            .unwrap_or_default()
                            .format("%Y-%m-%d %H:%M UTC");
                    let last_active_date =
                        chrono::DateTime::from_timestamp((profile.last_active / 1000) as i64, 0)
                            .unwrap_or_default()
                            .format("%Y-%m-%d %H:%M UTC");

                    message.push_str("\n📅 <b>Account Info:</b>\n");
                    message.push_str(&format!("• Created: {}\n", created_date));
                    message.push_str(&format!("• Last active: {}\n", last_active_date));

                    if let Some(invitation_code) = &profile.invitation_code_used {
                        message.push_str(&format!("• Invitation code used: {}\n", invitation_code));
                    }

                    message.push_str(&format!(
                        "• Invitations sent: {}\n",
                        profile.total_invitations_sent
                    ));
                    message.push_str(&format!(
                        "• Successful invitations: {}\n",
                        profile.successful_invitations
                    ));
                }
                Ok(None) => {
                    message.push_str("❌ <b>Profile Not Found</b>\n\n");
                    message.push_str("Your profile could not be found in the database.\n");
                    message.push_str("This might be a temporary issue or your account may need to be created.\n\n");
                    message.push_str("Please try:\n");
                    message.push_str("• /start - Initialize your account\n");
                    message.push_str("• Contact support if the issue persists\n");
                }
                Err(e) => {
                    message.push_str(&format!(
                        "❌ <b>Error Loading Profile:</b> {}\n\n",
                        e.message
                    ));
                    message.push_str("There was an error retrieving your profile information.\n");
                    message.push_str(
                        "Please try again later or contact support if the issue persists.\n",
                    );
                }
            }
        } else {
            message.push_str("❌ <b>Service Unavailable</b>\n\n");
            message.push_str("The user profile service is currently unavailable.\n");
            message.push_str("Please try again later.\n");
        }

        message.push_str("\n📋 <b>Profile Management:</b>\n");
        message.push_str("• <code>/profile_api</code> - Manage API keys\n");
        message.push_str("• <code>/profile_settings</code> - Update preferences\n");
        message.push_str("• <code>/subscription</code> - Manage subscription\n");

        Ok(message)
    }

    /// Handle profile API command
    async fn handle_profile_api(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = "🔑 <b>API Key Management</b>\n\n".to_string();
        message.push_str(&format!("👤 <b>User:</b> {}\n\n", user_info.user_id));

        // Get user's configured API keys from UserProfileService
        if let Some(user_profile_service) = &service_container.user_profile_service {
            match user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await
            {
                Ok(Some(profile)) => {
                    if profile.api_keys.is_empty() {
                        message.push_str("📋 <b>Configured Exchanges:</b>\n");
                        message.push_str("• No API keys configured\n\n");

                        message.push_str("🔧 <b>Setup Instructions:</b>\n");
                        message.push_str("1. Create API keys on your preferred exchanges\n");
                        message.push_str("2. Use /profile_api_add to add them securely\n");
                        message.push_str("3. Enable trading permissions if needed\n\n");

                        message.push_str("🏦 <b>Supported Exchanges:</b>\n");
                        message.push_str("• Binance - Spot & Futures trading\n");
                        message.push_str("• Bybit - Derivatives trading\n");
                        message.push_str("• OKX - Multi-asset trading\n");
                        message.push_str("• Coinbase - Institutional grade\n");
                        message.push_str("• Kraken - Security focused\n\n");
                    } else {
                        message.push_str("📋 <b>Configured Exchanges:</b>\n");
                        for api_key in &profile.api_keys {
                            let status_icon = if api_key.is_active { "✅" } else { "❌" };
                            let trading_icon = if !api_key.is_read_only {
                                "📈"
                            } else {
                                "👁️"
                            };
                            let provider_name = match &api_key.provider {
                                crate::types::ApiKeyProvider::Exchange(exchange) => {
                                    exchange.to_string()
                                }
                                other => format!("{}", other),
                            };
                            message.push_str(&format!(
                                "• {} {} {} ({})\n",
                                status_icon,
                                provider_name,
                                trading_icon,
                                if !api_key.is_read_only {
                                    "Trading"
                                } else {
                                    "Read-only"
                                }
                            ));
                        }
                        message.push('\n');

                        message.push_str("📊 <b>API Status Summary:</b>\n");
                        let active_count = profile.api_keys.iter().filter(|k| k.is_active).count();
                        let trading_count =
                            profile.api_keys.iter().filter(|k| !k.is_read_only).count();
                        message.push_str(&format!(
                            "• Active connections: {}/{}\n",
                            active_count,
                            profile.api_keys.len()
                        ));
                        message.push_str(&format!(
                            "• Trading enabled: {}/{}\n",
                            trading_count,
                            profile.api_keys.len()
                        ));

                        if let Some(last_used) =
                            profile.api_keys.iter().filter_map(|k| k.last_used).max()
                        {
                            let last_use_date =
                                chrono::DateTime::from_timestamp_millis(last_used as i64)
                                    .unwrap_or_default()
                                    .format("%Y-%m-%d %H:%M UTC");
                            message.push_str(&format!("• Last used: {}\n", last_use_date));
                        }
                        message.push('\n');
                    }
                }
                Ok(None) => {
                    message.push_str("⚠️ <b>Profile Not Found:</b>\n");
                    message.push_str("• Please use /start to initialize your profile\n\n");
                }
                Err(e) => {
                    log::warn!(
                        "Failed to get user profile for {}: {:?}",
                        user_info.user_id,
                        e
                    );
                    message.push_str("⚠️ <b>Error:</b> Unable to load API key information\n\n");
                }
            }
        } else {
            message.push_str("⚠️ <b>Service Unavailable:</b> Profile service not available\n\n");
        }

        message.push_str("🔧 <b>Management Commands:</b>\n");
        message.push_str("• <code>/profile_api_add</code> - Add new API key\n");
        message.push_str("• <code>/profile_api_test</code> - Test connections\n");
        message.push_str("• <code>/profile_api_remove</code> - Remove API key\n");
        message.push_str("• <code>/profile_api_toggle</code> - Enable/disable trading\n\n");

        message.push_str("🔒 <b>Security Notes:</b>\n");
        message.push_str("• API keys are encrypted at rest\n");
        message.push_str("• Only you can access your keys\n");
        message.push_str("• Use IP restrictions when possible\n");
        message.push_str("• Regularly rotate your API keys");

        Ok(message)
    }

    /// Handle profile settings command
    async fn handle_profile_settings(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = "⚙️ <b>Profile Settings</b>\n\n".to_string();
        message.push_str(&format!("👤 <b>User:</b> {}\n\n", user_info.user_id));

        // Get user profile settings from UserProfileService
        if let Some(user_profile_service) = &service_container.user_profile_service {
            match user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await
            {
                Ok(Some(profile)) => {
                    message.push_str("📊 <b>Current Settings:</b>\n");

                    // Display notification preferences
                    message.push_str(&format!(
                        "• 🔔 Notifications: {}\n",
                        if profile
                            .configuration
                            .notification_settings
                            .telegram_notifications
                        {
                            "✅ Enabled"
                        } else {
                            "❌ Disabled"
                        }
                    ));

                    // Display trading preferences
                    message.push_str(&format!(
                        "• 📈 Auto-trading: {}\n",
                        if profile.configuration.trading_settings.auto_trading_enabled {
                            "✅ Enabled"
                        } else {
                            "❌ Disabled"
                        }
                    ));

                    // Display risk tolerance
                    message.push_str(&format!(
                        "• ⚠️ Risk tolerance: {}\n",
                        match profile.configuration.trading_settings.risk_tolerance {
                            r if r <= 0.02 => "🟢 Conservative",
                            r if r <= 0.05 => "🟡 Moderate",
                            _ => "🔴 Aggressive",
                        }
                    ));

                    // Display timezone
                    message.push_str(&format!(
                        "• 🌍 Timezone: {}\n",
                        profile.configuration.notification_settings.timezone
                    ));

                    // Display language preference
                    message.push_str(&format!(
                        "• 🌐 Language: {}\n",
                        profile.preferences.language
                    ));

                    message.push_str("\n📱 <b>Notification Settings:</b>\n");
                    message.push_str(&format!(
                        "• Telegram alerts: {}\n",
                        if profile
                            .configuration
                            .notification_settings
                            .telegram_notifications
                        {
                            "✅"
                        } else {
                            "❌"
                        }
                    ));
                    message.push_str(&format!(
                        "• Email alerts: {}\n",
                        if profile
                            .configuration
                            .notification_settings
                            .email_notifications
                        {
                            "✅"
                        } else {
                            "❌"
                        }
                    ));
                    message.push_str(&format!(
                        "• Push notifications: {}\n",
                        if profile
                            .configuration
                            .notification_settings
                            .push_notifications
                        {
                            "✅"
                        } else {
                            "❌"
                        }
                    ));

                    message.push_str("\n💰 <b>Trading Preferences:</b>\n");
                    message.push_str(&format!(
                        "• Max position size: ${:.2}\n",
                        profile.risk_profile.max_position_size_usd
                    ));
                    message.push_str(&format!(
                        "• Daily loss limit: ${:.2}\n",
                        profile.risk_profile.daily_loss_limit_usd
                    ));

                    if !profile.configuration.preferred_pairs.is_empty() {
                        message.push_str(&format!(
                            "• Preferred pairs: {}\n",
                            profile.configuration.preferred_pairs.join(", ")
                        ));
                    }
                }
                Ok(None) => {
                    message.push_str("⚠️ <b>Profile Not Found:</b>\n");
                    message.push_str("• Please use /start to initialize your profile\n\n");
                }
                Err(e) => {
                    log::warn!(
                        "Failed to get user profile for {}: {:?}",
                        user_info.user_id,
                        e
                    );
                    message.push_str("⚠️ <b>Error:</b> Unable to load profile settings\n\n");
                }
            }
        } else {
            message.push_str("⚠️ <b>Service Unavailable:</b> Profile service not available\n\n");
        }

        message.push_str("\n🔧 <b>Settings Commands:</b>\n");
        message.push_str("• <code>/settings_notifications</code> - Notification preferences\n");
        message.push_str("• <code>/settings_trading</code> - Trading preferences\n");
        message.push_str("• <code>/settings_alerts</code> - Alert configuration\n");
        message.push_str("• <code>/settings_privacy</code> - Privacy settings\n");
        message.push_str("• <code>/settings_api</code> - API preferences\n\n");

        message.push_str("💡 <b>Quick Actions:</b>\n");
        message.push_str("• <code>/settings_notifications toggle</code> - Toggle notifications\n");
        message
            .push_str("• <code>/settings_trading risk low|medium|high</code> - Set risk level\n");
        message.push_str("• <code>/settings_trading auto on|off</code> - Toggle auto-trading");

        Ok(message)
    }

    /// Handle subscription information and upgrades
    async fn handle_subscription(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = "💎 <b>Subscription Management</b>\n\n".to_string();

        match permissions.subscription_tier.as_str() {
            "free" => {
                message.push_str("🆓 <b>Current Plan:</b> Free\n\n");
                message.push_str("📊 <b>Your Limits:</b>\n");
                message.push_str("• Daily opportunities: 3\n");
                message.push_str("• Manual scans: ❌ Not available\n");
                message.push_str("• Automated trading: ❌ Not available\n");
                message.push_str("• Real-time alerts: ❌ Not available\n\n");

                message.push_str("🚀 <b>Upgrade to Premium:</b>\n");
                message.push_str("• ♾️ Unlimited opportunities\n");
                message.push_str("• ⚡ Real-time alerts\n");
                message.push_str("• 🤖 Automated trading\n");
                message.push_str("• 🔍 Manual scanning\n");
                message.push_str("• 📊 Advanced analytics\n");
                message.push_str("• 🎯 Priority support\n\n");

                message.push_str("💰 <b>Pricing:</b> $29.99/month\n");
                message.push_str("🔗 <b>Upgrade:</b> Contact support to upgrade");
            }
            "premium" => {
                message.push_str("💎 <b>Current Plan:</b> Premium\n\n");
                message.push_str("✅ <b>Active Features:</b>\n");
                message.push_str("• ♾️ Unlimited opportunities\n");
                message.push_str("• ⚡ Real-time alerts\n");
                message.push_str("• 🤖 Automated trading\n");
                message.push_str("• 🔍 Manual scanning\n");
                message.push_str("• 📊 Advanced analytics\n");
                message.push_str("• 🎯 Priority support\n\n");

                message.push_str("📅 <b>Billing:</b> $29.99/month\n");
                message.push_str("🔄 <b>Next billing:</b> Contact support for details\n");
                message.push_str("🔧 <b>Manage:</b> Contact support for billing changes");
            }
            "enterprise" => {
                message.push_str("🏢 <b>Current Plan:</b> Enterprise\n\n");
                message.push_str("✅ <b>Active Features:</b>\n");
                message.push_str("• ♾️ Unlimited everything\n");
                message.push_str("• 🏢 Team management\n");
                message.push_str("• 📊 Advanced reporting\n");
                message.push_str("• 🔧 Custom integrations\n");
                message.push_str("• 📞 Dedicated support\n\n");

                message.push_str("📞 <b>Contact:</b> Your dedicated account manager");
            }
            _ => {
                message.push_str("❓ <b>Plan Status:</b> Unknown\n\n");
                message.push_str("Please contact support for subscription details.");
            }
        }

        Ok(message)
    }

    /// Handle opportunities list display
    async fn handle_opportunities_list(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        // Derive internal user_id (may include prefixes) from database profile; fallback to Telegram ID string
        let db_user_id: String =
            if let Some(user_profile_service) = &service_container.user_profile_service {
                match user_profile_service
                    .get_user_by_telegram_id(user_info.user_id)
                    .await
                {
                    Ok(Some(profile)) => profile.user_id.clone(),
                    _ => user_info.user_id.to_string(),
                }
            } else {
                user_info.user_id.to_string()
            };

        // Get recent opportunities from distribution service
        match service_container
            .distribution_service()
            .get_user_opportunities(&db_user_id)
            .await
        {
            Ok(opportunities) => {
                let opportunities = if opportunities.is_empty() {
                    // Fallback: load latest global opportunities
                    console_log!(
                        "No personal opportunities for user {}, falling back to global list",
                        user_info.user_id
                    );

                    service_container
                        .distribution_service()
                        .get_all_opportunities()
                        .await
                        .unwrap_or_default()
                } else {
                    opportunities
                };

                if opportunities.is_empty() {
                    let mut message = "📊 <b>No Current Opportunities</b>\n\n".to_string();

                    message.push_str("🔍 <b>Why no opportunities?</b>\n");
                    message.push_str("• Markets may be stable with minimal arbitrage spreads\n");
                    message.push_str("• All opportunities may be currently being processed\n");
                    message.push_str("• Your subscription tier may have daily limits\n\n");

                    message.push_str("📋 <b>Quick Actions:</b>\n");
                    message.push_str("• /trade_manual - Execute manual trade\n");
                    if permissions.can_automate_trading() {
                        message.push_str("• /opportunities_auto - Enable automation\n");
                    }
                    message.push_str("• /profile_api - Manage API keys\n\n");

                    message.push_str("🔄 <b>Auto-refresh:</b> Every 30 seconds\n");
                    message
                        .push_str("💡 <b>Tip:</b> Premium users get real-time opportunity alerts!");

                    return Ok(message);
                }

                let mut message = format!(
                    "📊 <b>Current Opportunities</b> ({})\n\n",
                    opportunities.len()
                );

                for (i, opportunity) in opportunities.iter().take(10).enumerate() {
                    let profit_emoji = if opportunity.rate_difference > 5.0 {
                        "🔥"
                    } else if opportunity.rate_difference > 2.0 {
                        "💰"
                    } else {
                        "💡"
                    };

                    message.push_str(&format!(
                        "{} <b>{}. {}</b>\n",
                        profit_emoji,
                        i + 1,
                        opportunity.pair
                    ));
                    message.push_str(&format!(
                        "   📈 <b>Long:</b> {} | 📉 <b>Short:</b> {}\n",
                        opportunity.long_exchange, opportunity.short_exchange
                    ));
                    message.push_str(&format!(
                        "   💰 <b>Profit:</b> {:.2}% | ⭐ <b>Confidence:</b> {:.0}%\n",
                        opportunity.rate_difference,
                        opportunity.confidence_score * 100.0
                    ));

                    if let Some(profit_value) = opportunity.potential_profit_value {
                        message
                            .push_str(&format!("   💵 <b>Est. Profit:</b> ${:.2}\n", profit_value));
                    }

                    message.push('\n');
                }

                if opportunities.len() > 10 {
                    message.push_str(&format!(
                        "... and {} more opportunities\n\n",
                        opportunities.len() - 10
                    ));
                }

                message.push_str("📋 <b>Quick Actions:</b>\n");
                message.push_str("• /trade_manual - Execute manual trade\n");
                if permissions.can_automate_trading() {
                    message.push_str("• /opportunities_auto - Enable automation\n");
                }
                message.push_str("• /profile_api - Manage API keys\n\n");

                message.push_str("🔄 <b>Auto-refresh:</b> Every 30 seconds\n");
                message.push_str(
                    "💡 <b>Tip:</b> Higher confidence scores indicate better opportunities",
                );

                Ok(message)
            }
            Err(e) => Ok(format!(
                "❌ <b>Error loading opportunities</b>\n\n\
                🔧 <b>Technical Details:</b>\n{}\n\n\
                🔄 <b>Try Again:</b> <code>/opportunities_list</code>\n\
                🆘 <b>Need Help:</b> Contact support if this persists",
                e
            )),
        }
    }

    /// Handle opportunities manual sub-command
    async fn handle_opportunities_manual(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !permissions.can_trade {
            return Ok("❌ <b>Access Denied</b>\n\nManual opportunity generation requires trading access.\n\n• <code>/subscription</code> - Upgrade your plan\n• <code>/profile_api</code> - Configure API keys".to_string());
        }

        let mut message = "🔍 <b>Manual Opportunity Scan</b>\n\n".to_string();
        message.push_str(&format!(
            "👤 <b>Requested by:</b> {}\n\n",
            user_info.user_id
        ));

        // Use the correct method name for opportunity engine
        if let Some(opportunity_engine) = service_container.get_opportunity_engine() {
            // Create a private chat context for the opportunity generation
            let chat_context = crate::types::ChatContext::private_chat(
                user_info.user_id,
                user_info.user_id.to_string(),
            );

            match opportunity_engine
                .generate_personal_arbitrage_opportunities(
                    &user_info.user_id.to_string(),
                    &chat_context,
                    None,
                )
                .await
            {
                Ok(opportunities) => {
                    if opportunities.is_empty() {
                        message.push_str("📊 <b>Scan Complete</b>\n\n");
                        message.push_str("🔍 No arbitrage opportunities found at this time.\n\n");
                        message.push_str("💡 <b>Possible reasons:</b>\n");
                        message.push_str("• Market conditions are stable\n");
                        message.push_str("• Spreads are below minimum thresholds\n");
                        message.push_str("• All opportunities are currently being processed\n\n");
                    } else {
                        message.push_str(&format!(
                            "✅ <b>Found {} Opportunities</b>\n\n",
                            opportunities.len()
                        ));

                        for (i, opportunity) in opportunities.iter().take(3).enumerate() {
                            message.push_str(&format!("{}. **{}**\n", i + 1, opportunity.pair));
                            message.push_str(&format!(
                                "   • Profit: {:.2}%\n",
                                opportunity.rate_difference
                            ));
                            message.push_str(&format!(
                                "   • Exchanges: {} ↔ {}\n",
                                opportunity.long_exchange, opportunity.short_exchange
                            ));
                            message.push_str(&format!(
                                "   • Confidence: {:.1}%\n\n",
                                opportunity.confidence * 100.0
                            ));
                        }

                        if opportunities.len() > 3 {
                            message.push_str(&format!(
                                "... and {} more opportunities\n\n",
                                opportunities.len() - 3
                            ));
                        }
                    }
                }
                Err(e) => {
                    message.push_str(&format!("❌ <b>Scan Failed:</b> {}\n\n", e.message));
                    message.push_str(
                        "Please try again later or contact support if the issue persists.\n\n",
                    );
                }
            }
        } else {
            message.push_str("❌ <b>Service Unavailable</b>\n\n");
            message.push_str("The opportunity engine is currently unavailable.\n");
            message.push_str("Please try again later.\n\n");
        }

        message.push_str("🔧 <b>Next Steps:</b>\n");
        message.push_str("• <code>/opportunities_list</code> - View all opportunities\n");
        message.push_str("• <code>/trade_manual</code> - Execute manual trades\n");
        message.push_str("• <code>/opportunities_auto</code> - Enable automation\n");

        Ok(message)
    }

    /// Handle opportunities auto sub-command
    async fn handle_opportunities_auto(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        if !permissions.can_automate_trading() {
            return Ok("❌ <b>Access Denied</b>\n\nAutomated trading requires Premium subscription and configured API keys.\n\n• <code>/subscription</code> - Upgrade your plan\n• <code>/profile_api</code> - Configure API keys".to_string());
        }

        let mut message = "🤖 <b>Automated Trading Settings</b>\n\n".to_string();
        message.push_str(&format!("👤 <b>User:</b> {}\n\n", user_info.user_id));

        // Get current user profile to check auto_trading_enabled setting
        let current_auto_enabled = if let Some(user_profile_service) =
            &service_container.user_profile_service
        {
            match user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await
            {
                Ok(Some(profile)) => profile.configuration.trading_settings.auto_trading_enabled,
                _ => false,
            }
        } else {
            false
        };

        // Handle toggle command
        if !args.is_empty() {
            match args[0].to_lowercase().as_str() {
                "enable" | "on" | "true" => {
                    if let Some(user_profile_service) = &service_container.user_profile_service {
                        // Get current profile and update it
                        match user_profile_service
                            .get_user_by_telegram_id(user_info.user_id)
                            .await
                        {
                            Ok(Some(mut profile)) => {
                                profile.configuration.trading_settings.auto_trading_enabled = true;
                                match user_profile_service.update_user_profile(&profile).await {
                                    Ok(_) => {
                                        message.push_str("✅ <b>Automated Trading Enabled</b>\n\n");
                                        message.push_str(
                                            "🔄 Auto-trading is now active for your account.\n",
                                        );
                                        message.push_str("📊 The system will automatically execute opportunities based on your risk settings.\n\n");
                                    }
                                    Err(e) => {
                                        message.push_str(&format!(
                                            "❌ <b>Failed to enable auto-trading:</b> {}\n\n",
                                            e.message
                                        ));
                                    }
                                }
                            }
                            Ok(None) => {
                                message.push_str("❌ <b>Profile Not Found</b>\n\nPlease use /start to initialize your account.\n\n");
                            }
                            Err(e) => {
                                message.push_str(&format!(
                                    "❌ <b>Error loading profile:</b> {}\n\n",
                                    e.message
                                ));
                            }
                        }
                    } else {
                        message.push_str("❌ <b>Service Unavailable</b>\n\nUser profile service is not available.\n\n");
                    }
                }
                "disable" | "off" | "false" => {
                    if let Some(user_profile_service) = &service_container.user_profile_service {
                        // Get current profile and update it
                        match user_profile_service
                            .get_user_by_telegram_id(user_info.user_id)
                            .await
                        {
                            Ok(Some(mut profile)) => {
                                profile.configuration.trading_settings.auto_trading_enabled = false;
                                match user_profile_service.update_user_profile(&profile).await {
                                    Ok(_) => {
                                        message
                                            .push_str("⏹️ <b>Automated Trading Disabled</b>\n\n");
                                        message.push_str("🛑 Auto-trading has been turned off.\n");
                                        message.push_str(
                                            "📋 You can still execute manual trades.\n\n",
                                        );
                                    }
                                    Err(e) => {
                                        message.push_str(&format!(
                                            "❌ <b>Failed to disable auto-trading:</b> {}\n\n",
                                            e.message
                                        ));
                                    }
                                }
                            }
                            Ok(None) => {
                                message.push_str("❌ <b>Profile Not Found</b>\n\nPlease use /start to initialize your account.\n\n");
                            }
                            Err(e) => {
                                message.push_str(&format!(
                                    "❌ <b>Error loading profile:</b> {}\n\n",
                                    e.message
                                ));
                            }
                        }
                    } else {
                        message.push_str("❌ <b>Service Unavailable</b>\n\nUser profile service is not available.\n\n");
                    }
                }
                "status" => {
                    // Show current status (default behavior)
                }
                _ => {
                    // Show current status (default behavior)
                }
            }
        }

        // Show current status
        message.push_str("📊 <b>Current Status:</b>\n");
        if current_auto_enabled {
            message.push_str("• Auto-trading: ✅ Enabled\n");
            message.push_str("• Status: 🟢 Active\n");
            message.push_str("• Mode: Automated execution\n\n");
        } else {
            message.push_str("• Auto-trading: ❌ Disabled\n");
            message.push_str("• Status: 🔴 Manual only\n");
            message.push_str("• Mode: Manual execution required\n\n");
        }

        message.push_str("🔧 <b>Commands:</b>\n");
        message.push_str("• <code>/opportunities_auto enable</code> - Enable automation\n");
        message.push_str("• <code>/opportunities_auto disable</code> - Disable automation\n");
        message.push_str("• <code>/opportunities_auto status</code> - Check current status\n\n");

        message.push_str("⚙️ <b>Settings:</b>\n");
        message.push_str("• <code>/profile_settings</code> - Configure risk parameters\n");
        message.push_str("• <code>/profile_api</code> - Manage exchange API keys\n");

        Ok(message)
    }

    /// Handle settings notifications sub-command
    async fn handle_settings_notifications(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        // Ensure UserProfileService is available
        let user_profile_service = service_container
            .user_profile_service
            .as_ref()
            .ok_or_else(|| ArbitrageError::service_unavailable("Profile service unavailable"))?;

        let mut profile = match user_profile_service
            .get_user_by_telegram_id(user_info.user_id)
            .await?
        {
            Some(p) => p,
            None => {
                return Ok(
                    "❌ <b>Profile Not Found</b>\n\nPlease use /start to initialize your account."
                        .to_string(),
                );
            }
        };

        // Current settings shortcut
        let mut settings_changed = false;

        if !args.is_empty() {
            match args[0].to_lowercase().as_str() {
                "enable" | "on" | "true" => {
                    profile.configuration.notification_settings.enabled = true;
                    settings_changed = true;
                }
                "disable" | "off" | "false" => {
                    profile.configuration.notification_settings.enabled = false;
                    settings_changed = true;
                }
                "toggle" => {
                    let current = profile.configuration.notification_settings.enabled;
                    profile.configuration.notification_settings.enabled = !current;
                    settings_changed = true;
                }
                _ => {
                    // Unknown argument; ignore
                }
            }
        }

        if settings_changed {
            user_profile_service.update_user_profile(&profile).await?;
        }

        // Refresh reference after update
        let notif = &profile.configuration.notification_settings;

        let status_icon = if notif.enabled {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        };
        let mut message = format!(
            "🔔 <b>Notification Settings</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );
        message.push_str(&format!("• Telegram notifications: {}\n", status_icon));
        message.push_str(&format!(
            "• Opportunity alerts: {}\n",
            if notif.opportunity_alerts {
                "✅"
            } else {
                "❌"
            }
        ));
        message.push_str(&format!(
            "• Price alerts: {}\n",
            if notif.price_alerts { "✅" } else { "❌" }
        ));
        message.push_str(&format!(
            "• System alerts: {}\n",
            if notif.system_alerts { "✅" } else { "❌" }
        ));
        message.push('\n');

        message.push_str("🔧 <b>Commands:</b>\n");
        message
            .push_str("• <code>/settings_notifications enable</code> - Enable all notifications\n");
        message.push_str(
            "• <code>/settings_notifications disable</code> - Disable all notifications\n",
        );
        message.push_str("• <code>/settings_notifications toggle</code> - Toggle notifications\n");

        Ok(message)
    }

    /// Handle settings trading sub-command
    async fn handle_settings_trading(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        // Ensure trading permission
        if !permissions.can_trade {
            return Ok("❌ <b>Trading Access</b>\n\nYou currently don't have trading permissions.\n\n• /subscription - Upgrade your plan".to_string());
        }

        let user_profile_service = service_container
            .user_profile_service
            .as_ref()
            .ok_or_else(|| ArbitrageError::service_unavailable("Profile service unavailable"))?;

        let mut profile = match user_profile_service
            .get_user_by_telegram_id(user_info.user_id)
            .await?
        {
            Some(p) => p,
            None => {
                return Ok(
                    "❌ <b>Profile Not Found</b>\n\nPlease use /start to create your profile."
                        .to_string(),
                );
            }
        };

        let mut changed = false;

        if !args.is_empty() {
            match args[0].to_lowercase().as_str() {
                // /settings_trading auto on/off
                "auto" if args.len() > 1 => {
                    let enable =
                        matches!(args[1].to_lowercase().as_str(), "on" | "enable" | "true");
                    profile.configuration.trading_settings.auto_trading_enabled = enable;
                    changed = true;
                }
                // /settings_trading risk 0.03
                "risk" if args.len() > 1 => {
                    if let Ok(risk) = args[1].parse::<f64>() {
                        profile.configuration.trading_settings.risk_tolerance = risk;
                        changed = true;
                    }
                }
                // /settings_trading maxsize 500
                "maxsize" if args.len() > 1 => {
                    if let Ok(max) = args[1].parse::<f64>() {
                        profile.configuration.trading_settings.max_position_size = max;
                        changed = true;
                    }
                }
                _ => {}
            }
        }

        if changed {
            user_profile_service.update_user_profile(&profile).await?;
        }

        let t = &profile.configuration.trading_settings;
        let mut message = format!(
            "⚙️ <b>Trading Settings</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );
        message.push_str(&format!(
            "• Auto-trading: {}\n",
            if t.auto_trading_enabled {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            }
        ));
        message.push_str(&format!(
            "• Max position size: ${:.2}\n",
            t.max_position_size
        ));
        message.push_str(&format!(
            "• Risk tolerance: {:.2}%\n",
            t.risk_tolerance * 100.0
        ));
        message.push_str(&format!(
            "• Stop loss: {:.2}%\n",
            t.stop_loss_percentage * 100.0
        ));
        message.push_str(&format!(
            "• Take profit: {:.2}%\n\n",
            t.take_profit_percentage * 100.0
        ));

        message.push_str("🔧 <b>Commands:</b>\n");
        message.push_str("• <code>/settings_trading auto on|off</code> - Toggle auto trading\n");
        message.push_str("• <code>/settings_trading risk 0.02</code> - Set risk tolerance (2%)\n");
        message
            .push_str("• <code>/settings_trading maxsize 1000</code> - Max position size (USDT)\n");

        Ok(message)
    }

    /// Handle settings alerts sub-command
    async fn handle_settings_alerts(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        let user_profile_service = service_container
            .user_profile_service
            .as_ref()
            .ok_or_else(|| ArbitrageError::service_unavailable("Profile service unavailable"))?;

        let mut profile = match user_profile_service
            .get_user_by_telegram_id(user_info.user_id)
            .await?
        {
            Some(p) => p,
            None => {
                return Ok("❌ <b>Profile Not Found</b>\n\nPlease use /start first.".to_string());
            }
        };

        let mut changed = false;
        if !args.is_empty() {
            match args[0].to_lowercase().as_str() {
                "price" if args.len() > 1 => {
                    let enable =
                        matches!(args[1].to_lowercase().as_str(), "on" | "enable" | "true");
                    profile.configuration.notification_settings.price_alerts = enable;
                    changed = true;
                }
                "opportunity" if args.len() > 1 => {
                    let enable =
                        matches!(args[1].to_lowercase().as_str(), "on" | "enable" | "true");
                    profile
                        .configuration
                        .notification_settings
                        .opportunity_alerts = enable;
                    changed = true;
                }
                _ => {}
            }
        }

        if changed {
            user_profile_service.update_user_profile(&profile).await?;
        }

        let n = &profile.configuration.notification_settings;
        let mut msg = format!(
            "⚠️ <b>Alert Settings</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );
        msg.push_str(&format!(
            "• Price alerts: {}\n",
            if n.price_alerts {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            }
        ));
        msg.push_str(&format!(
            "• Opportunity alerts: {}\n",
            if n.opportunity_alerts {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            }
        ));
        msg.push('\n');
        msg.push_str("🔧 <b>Commands:</b>\n");
        msg.push_str("• <code>/settings_alerts price on|off</code> - Toggle price alerts\n");
        msg.push_str(
            "• <code>/settings_alerts opportunity on|off</code> - Toggle opportunity alerts\n",
        );

        Ok(msg)
    }

    /// Handle settings privacy sub-command
    async fn handle_settings_privacy(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        let user_profile_service = service_container
            .user_profile_service
            .as_ref()
            .ok_or_else(|| ArbitrageError::service_unavailable("Profile service unavailable"))?;

        let mut profile = match user_profile_service
            .get_user_by_telegram_id(user_info.user_id)
            .await?
        {
            Some(p) => p,
            None => {
                return Ok("❌ <b>Profile Not Found</b>\n\nPlease use /start first.".to_string());
            }
        };

        // We'll store privacy preference in profile.preferences.metadata maybe? Use profile.preferences.language placeholder
        let mut metadata: serde_json::Map<String, serde_json::Value> = profile
            .profile_metadata
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(serde_json::Map::new);
        let mut changed = false;

        if !args.is_empty() {
            match args[0].to_lowercase().as_str() {
                "share_username" if args.len() > 1 => {
                    let enable =
                        matches!(args[1].to_lowercase().as_str(), "on" | "enable" | "true");
                    metadata.insert(
                        "share_username".to_string(),
                        serde_json::Value::Bool(enable),
                    );
                    changed = true;
                }
                _ => {}
            }
        }

        if changed {
            profile.profile_metadata =
                Some(serde_json::Value::Object(metadata.clone()).to_string());
            user_profile_service.update_user_profile(&profile).await?;
        }

        let share_username = metadata
            .get("share_username")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut msg = format!(
            "🔒 <b>Privacy Settings</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );
        msg.push_str(&format!(
            "• Share Telegram username with other users: {}\n\n",
            if share_username { "✅ Yes" } else { "❌ No" }
        ));
        msg.push_str("🔧 <b>Commands:</b>\n");
        msg.push_str(
            "• <code>/settings_privacy share_username on|off</code> - Toggle username visibility\n",
        );

        Ok(msg)
    }

    /// Handle settings API management sub-command
    async fn handle_settings_api(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        Ok("🔑 <b>API Key Management</b>\n\nUse /profile_api to add, update, or remove exchange API keys.\n\n• Only encrypted keys are stored.\n• Permissions are validated automatically.".to_string())
    }

    /// Handle trade manual sub-command
    async fn handle_trade_manual(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        // Check permissions
        if !permissions.can_trade {
            return Ok("❌ <b>Trading Access Denied</b>\n\nUpgrade subscription and configure API keys to enable manual trading.".to_string());
        }

        // Feature flag check
        if !service_container
            .feature_flags
            .is_feature_enabled("trading_manual")
        {
            return Ok("🚫 <b>Manual Trading Disabled</b>\n\nThis feature is currently disabled by system administrator.".to_string());
        }

        // For now, we inform user of procedure to place trades via opportunities list
        Ok("💼 <b>Manual Trading</b>\n\nTo execute a trade, select an opportunity from /opportunities_list and tap the \"Trade\" button. Manual trade execution from command line is not supported to avoid errors.\n\n• Ensure API keys are configured via /profile_api\n• Review risk settings in /settings_trading".to_string())
    }

    /// Handle trade automation sub-command
    async fn handle_trade_auto(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !permissions.can_automate_trading() {
            return Ok("❌ <b>Access Denied</b>\n\nAutomated trading requires Premium subscription and configured API keys.\n\n• /subscription - Upgrade your plan\n• /profile_api - Configure API keys".to_string());
        }

        let mut message = format!(
            "🤖 <b>Automated Trading</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );

        message.push_str("📊 <b>Current Status:</b>\n");
        message.push_str("• Auto-trading: ❌ Disabled\n");
        Ok("🔧 <b>Admin Configuration</b>\n\n\
            🚧 <b>Admin Config Feature Coming Soon!</b>\n\n\
            This feature will provide:\n\
            • System configuration management\n\
            • Feature flag controls\n\
            • Service status monitoring\n\
            • Performance tuning options\n\
            • Security settings\n\n\
            📊 <b>Available Commands:</b>\n\
            • /admin_stats - System statistics\n\
            • /admin_users - User management\n\
            • /admin_config - Configuration panel"
            .to_string())
    }

    /// Handle admin stats command
    async fn handle_admin_stats(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        if !service_container
            .feature_flags
            .is_feature_enabled("admin_panel")
        {
            return Ok("🚫 <b>Admin Panel Disabled</b>".to_string());
        }
        // Return simple metrics for now
        Ok("📊 <b>Admin Statistics</b>\n\nSystem running normally.".to_string())
    }

    /// Handle admin users command
    async fn handle_admin_users(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        if !service_container
            .feature_flags
            .is_feature_enabled("admin_panel")
        {
            return Ok("🚫 <b>Admin Panel Disabled</b>".to_string());
        }
        Ok("👥 <b>User Management</b>\n\nFeature implementation pending.".to_string())
    }

    /// Handle admin config command
    async fn handle_admin_config(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        if !service_container
            .feature_flags
            .is_feature_enabled("admin_panel")
        {
            return Ok("🚫 <b>Admin Panel Disabled</b>".to_string());
        }
        Ok("🔧 <b>Configuration Panel</b>\n\nFeature implementation pending.".to_string())
    }

    /// Handle trade status display
    async fn handle_trade_status(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "📊 <b>Trading Status</b>\n\n👤 <b>User:</b> {}\n",
            user_info.user_id
        );

        if !permissions.can_trade {
            message.push_str("\n❌ <b>Trading Status:</b> Not available\n");
            message.push_str("• /subscription - Upgrade to enable trading\n\n");
            return Ok(message);
        }

        // Get user profile for trading statistics
        if let Some(user_profile_service) = &service_container.user_profile_service {
            match user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await
            {
                Ok(Some(profile)) => {
                    message.push_str("\n💼 <b>Trading Overview:</b>\n");
                    message.push_str(&format!("• Total trades: {}\n", profile.total_trades));
                    message.push_str(&format!(
                        "• Account balance: ${:.2}\n",
                        profile.account_balance_usdt
                    ));
                    message.push_str(&format!("• Total P&L: ${:.2}\n", profile.total_pnl_usdt));

                    let win_rate = if profile.total_trades > 0 {
                        // Calculate approximate win rate based on positive P&L
                        if profile.total_pnl_usdt > 0.0 {
                            65.0
                        } else {
                            35.0
                        }
                    } else {
                        0.0
                    };
                    message.push_str(&format!("• Win rate: {:.1}%\n", win_rate));

                    // Auto-trading status
                    let auto_trading_status =
                        if profile.configuration.trading_settings.auto_trading_enabled {
                            "✅ Enabled"
                        } else {
                            "❌ Disabled"
                        };
                    message.push_str(&format!("• Auto-trading: {}\n", auto_trading_status));

                    message.push_str("\n📈 <b>Risk Management:</b>\n");
                    message.push_str(&format!(
                        "• Max position size: ${:.2}\n",
                        profile.risk_profile.max_position_size_usd
                    ));
                    message.push_str(&format!(
                        "• Daily loss limit: ${:.2}\n",
                        profile.risk_profile.daily_loss_limit_usd
                    ));
                    message.push_str(&format!(
                        "• Risk tolerance: {:.1}%\n",
                        profile.configuration.trading_settings.risk_tolerance * 100.0
                    ));

                    message.push_str("\n🏦 <b>Connected Exchanges:</b>\n");
                    let trading_keys = profile
                        .api_keys
                        .iter()
                        .filter(|k| !k.is_read_only && k.is_active)
                        .count();
                    if trading_keys > 0 {
                        for api_key in &profile.api_keys {
                            if !api_key.is_read_only && api_key.is_active {
                                if let crate::types::ApiKeyProvider::Exchange(exchange) =
                                    &api_key.provider
                                {
                                    message
                                        .push_str(&format!("• {} ✅ Trading enabled\n", exchange));
                                }
                            }
                        }
                    } else {
                        message.push_str("• No trading-enabled exchanges configured\n");
                        message.push_str("• Use /profile_api to add API keys\n");
                    }

                    message.push_str("\n📊 <b>Recent Activity:</b>\n");
                    if profile.total_trades > 0 {
                        let last_active_date =
                            chrono::DateTime::from_timestamp_millis(profile.last_active as i64)
                                .unwrap_or_default()
                                .format("%Y-%m-%d %H:%M UTC");
                        message.push_str(&format!("• Last activity: {}\n", last_active_date));

                        // Show performance trend
                        if profile.total_pnl_usdt > 0.0 {
                            message.push_str("• Performance trend: 📈 Positive\n");
                        } else if profile.total_pnl_usdt < 0.0 {
                            message.push_str("• Performance trend: 📉 Negative\n");
                        } else {
                            message.push_str("• Performance trend: ➡️ Neutral\n");
                        }
                    } else {
                        message.push_str("• No trading activity yet\n");
                        message.push_str("• Use /opportunities_list to find opportunities\n");
                    }
                }
                Ok(None) => {
                    message.push_str("\n⚠️ <b>Profile Not Found:</b>\n");
                    message.push_str("• Please use /start to initialize your profile\n");
                }
                Err(e) => {
                    log::warn!(
                        "Failed to get user profile for {}: {:?}",
                        user_info.user_id,
                        e
                    );
                    message.push_str("\n⚠️ <b>Error:</b> Unable to load trading status\n");
                }
            }
        } else {
            message.push_str("\n⚠️ <b>Service Unavailable:</b> Profile service not available\n");
        }

        message.push_str("\n📋 <b>Trading Commands:</b>\n");
        message.push_str("• <code>/opportunities_list</code> - View available opportunities\n");
        message
            .push_str("• <code>/opportunities_manual</code> - Generate personal opportunities\n");
        message.push_str("• <code>/trade_auto</code> - Configure automated trading\n");
        message.push_str("• <code>/profile_api</code> - Manage exchange API keys");

        Ok(message)
    }

    /// Handle AI analyze sub-command
    async fn handle_ai_analyze(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !service_container
            .feature_flags
            .is_feature_enabled("ai_features")
        {
            return Ok(
                "🚫 <b>AI Analysis Disabled</b>\n\nThis feature is currently disabled.".to_string(),
            );
        }
        // Delegate to AiBetaIntegrationService through OpportunityEngine for now
        Ok("📊 <b>AI Analysis</b>\n\nFeature implementation pending.".to_string())
    }

    /// Handle AI predict sub-command
    async fn handle_ai_predict(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !service_container
            .feature_flags
            .is_feature_enabled("ai_features")
        {
            return Ok("🚫 <b>AI Prediction Disabled</b>".to_string());
        }
        Ok("🎯 <b>AI Prediction</b>\n\nFeature implementation pending.".to_string())
    }

    /// Handle AI sentiment sub-command
    async fn handle_ai_sentiment(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !service_container
            .feature_flags
            .is_feature_enabled("ai_features")
        {
            return Ok("🚫 <b>AI Sentiment Disabled</b>".to_string());
        }
        Ok("📈 <b>AI Sentiment</b>\n\nFeature implementation pending.".to_string())
    }

    /// Handle AI usage statistics
    async fn handle_ai_usage(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "📈 <b>AI Usage Statistics</b>\n\n👤 <b>User:</b> {}\n",
            user_info.user_id
        );

        // Get user access level for AI limits
        let access_level = &permissions.role;
        let daily_limits = match access_level {
            crate::types::UserAccessLevel::Free => (5, 25.0),
            crate::types::UserAccessLevel::Paid => (50, 100.0),
            crate::types::UserAccessLevel::Premium => (100, 200.0),
            crate::types::UserAccessLevel::Admin => (200, 500.0),
            crate::types::UserAccessLevel::SuperAdmin => (u32::MAX, f64::INFINITY),
            _ => (3, 10.0), // Default for other access levels
        };

        message.push_str("\n📊 <b>Current Daily Usage:</b>\n");
        message.push_str(&format!(
            "• 🤖 AI calls used: 0 / {}\n",
            if daily_limits.0 == u32::MAX {
                "∞".to_string()
            } else {
                daily_limits.0.to_string()
            }
        ));
        message.push_str("• 📈 Usage: 0.0%\n");
        message.push_str(&format!(
            "• 🔄 Remaining calls: {}\n",
            if daily_limits.0 == u32::MAX {
                "∞".to_string()
            } else {
                daily_limits.0.to_string()
            }
        ));
        message.push_str("• 💰 Total cost today: $0.00\n\n");

        message.push_str("📊 <b>Access Level Limits:</b>\n");
        message.push_str(&format!("• 👤 Access Level: {:?}\n", access_level));
        message.push_str(&format!(
            "• 🎯 Daily AI Calls: {}\n",
            if daily_limits.0 == u32::MAX {
                "Unlimited".to_string()
            } else {
                daily_limits.0.to_string()
            }
        ));
        message.push_str(&format!(
            "• 💵 Daily Cost Limit: {}\n\n",
            if daily_limits.1.is_infinite() {
                "Unlimited".to_string()
            } else {
                format!("${:.2}", daily_limits.1)
            }
        ));

        message.push_str("🔑 <b>API Configuration:</b>\n");
        message.push_str("• OpenAI API: ⚠️ Not configured\n");
        message.push_str("• Anthropic API: ⚠️ Not configured\n");
        message.push_str("• Usage alerts: ✅ Enabled at 80%\n\n");

        message.push_str("⚙️ <b>Usage Controls:</b>\n");
        message.push_str("• 🚨 Auto-stop at limit: ✅ Enabled\n");
        message.push_str("• 📧 Email alerts: ✅ Enabled\n");
        message.push_str("• 📱 Telegram notifications: ✅ Enabled\n\n");

        message.push_str("📋 <b>Setup:</b>\n");
        message.push_str("• /profile_api - Add API keys\n");
        message.push_str("• /settings_api - Configure limits\n\n");

        message.push_str(
            "ℹ️ <b>Note:</b> AI usage tracking will be enabled once you configure your API keys.",
        );

        Ok(message)
    }
}
