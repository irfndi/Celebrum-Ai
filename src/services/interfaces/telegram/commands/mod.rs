//! Command Router
//!
//! Routes Telegram commands to appropriate handlers

use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::interfaces::telegram::{UserInfo, UserPermissions};
use crate::utils::ArbitrageResult;
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
        _args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        console_log!(
            "🎯 Routing command '{}' for user {}",
            command,
            user_info.user_id
        );

        match command {
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
                Self::handle_opportunities_manual(user_info, permissions, service_container, &[])
                    .await
            }
            "/opportunities_auto" => {
                Self::handle_opportunities_auto(user_info, permissions, service_container, &[])
                    .await
            }

            "/settings_notifications" => {
                Self::handle_settings_notifications(user_info, permissions, service_container, &[])
                    .await
            }
            "/settings_trading" => {
                Self::handle_settings_trading(user_info, permissions, service_container, &[]).await
            }
            "/settings_alerts" => {
                Self::handle_settings_alerts(user_info, permissions, service_container, &[]).await
            }
            "/settings_privacy" => {
                Self::handle_settings_privacy(user_info, permissions, service_container, &[]).await
            }
            "/settings_api" => {
                Self::handle_settings_api(user_info, permissions, service_container, &[]).await
            }

            "/trade_manual" => {
                Self::handle_trade_manual(user_info, permissions, service_container, &[]).await
            }
            "/trade_auto" => {
                Self::handle_trade_auto(user_info, permissions, service_container, &[]).await
            }
            "/trade_status" => {
                Self::handle_trade_status(user_info, permissions, service_container).await
            }

            "/ai_analyze" => {
                Self::handle_ai_analyze(user_info, permissions, service_container, &[]).await
            }
            "/ai_predict" => {
                Self::handle_ai_predict(user_info, permissions, service_container, &[]).await
            }
            "/ai_sentiment" => {
                Self::handle_ai_sentiment(user_info, permissions, service_container, &[]).await
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
                // Extract the base command for better error handling
                let base_command = command.split('_').next().unwrap_or(command);

                match base_command {
                    "/opportunities" => Ok("❓ <b>Invalid opportunities command</b>\n\nAvailable options:\n• /opportunities_list - View current opportunities\n• /opportunities_manual - Manual trading\n• /opportunities_auto - Automated trading\n\nUse /help for more information.".to_string()),
                    "/settings" => Ok("❓ <b>Invalid settings command</b>\n\nAvailable options:\n• /settings_notifications - Notification preferences\n• /settings_trading - Trading settings\n• /settings_alerts - Alert configuration\n• /settings_privacy - Privacy settings\n• /settings_api - API management\n\nUse /help for more information.".to_string()),
                    "/trade" => Ok("❓ <b>Invalid trade command</b>\n\nAvailable options:\n• /trade_manual - Execute manual trades\n• /trade_auto - Automated trading\n• /trade_status - View trading status\n\nUse /help for more information.".to_string()),
                    "/ai" => Ok("❓ <b>Invalid AI command</b>\n\nAvailable options:\n• /ai_analyze - Market analysis\n• /ai_predict - Price predictions\n• /ai_sentiment - Sentiment analysis\n• /ai_usage - Usage statistics\n\nUse /help for more information.".to_string()),
                    "/profile" => Ok("❓ <b>Invalid profile command</b>\n\nAvailable options:\n• /profile_view - View profile\n• /profile_api - API management\n• /profile_settings - Profile settings\n\nUse /help for more information.".to_string()),
                    _ => Ok(format!("❓ <b>Unknown command:</b> <code>{}</code>\n\n🤖 Available commands:\n• /help - Show all commands\n• /opportunities_list - View arbitrage opportunities\n• /profile_view - Your account info\n• /settings_notifications - Configure alerts\n\n💡 <b>Tip:</b> Commands are clickable! Tap them instead of typing.", command)),
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

    /// Handle profile view sub-command
    async fn handle_profile_view(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        if let Some(user_profile_service) = &service_container.user_profile_service {
            match user_profile_service
                .get_user_by_telegram_id(user_info.user_id)
                .await?
            {
                Some(profile) => {
                    let mut message = "👤 <b>Your Profile</b>\n\n".to_string();

                    message.push_str(&format!(
                        "🆔 <b>User ID:</b> <code>{}</code>\n",
                        profile.user_id
                    ));
                    if let Some(telegram_id) = profile.telegram_user_id {
                        message.push_str(&format!(
                            "📱 <b>Telegram ID:</b> <code>{}</code>\n",
                            telegram_id
                        ));
                    }
                    if let Some(username) = &profile.username {
                        message.push_str(&format!("👤 <b>Username:</b> @{}\n", username));
                    }

                    message.push_str(&format!(
                        "🏷️ <b>Access Level:</b> {}\n",
                        match profile.access_level {
                            crate::types::UserAccessLevel::Guest => "🆓 Guest",
                            crate::types::UserAccessLevel::Free => "🆓 Free",
                            crate::types::UserAccessLevel::Registered => "📝 Registered",
                            crate::types::UserAccessLevel::Verified => "✅ Verified",
                            crate::types::UserAccessLevel::Paid => "💰 Paid",
                            crate::types::UserAccessLevel::Premium => "💎 Premium",
                            crate::types::UserAccessLevel::Admin => "🔧 Admin",
                            crate::types::UserAccessLevel::SuperAdmin => "👑 Super Admin",
                            crate::types::UserAccessLevel::BetaUser => "🧪 Beta User",
                            crate::types::UserAccessLevel::FreeWithoutAPI => "🆓 Free (No API)",
                            crate::types::UserAccessLevel::FreeWithAPI => "🆓 Free (With API)",
                            crate::types::UserAccessLevel::SubscriptionWithAPI =>
                                "📊 Subscription (With API)",
                            crate::types::UserAccessLevel::Basic => "📋 Basic",
                            crate::types::UserAccessLevel::User => "👤 User",
                        }
                    ));

                    message.push_str(&format!(
                        "📊 <b>Subscription:</b> {}\n",
                        permissions.subscription_tier.to_uppercase()
                    ));

                    if permissions.beta_access {
                        message.push_str("🧪 <b>Beta Access:</b> ✅ Enabled\n");
                    }

                    message.push_str(&format!(
                        "📅 <b>Member Since:</b> {}\n\n",
                        chrono::DateTime::from_timestamp(profile.created_at as i64 / 1000, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    ));

                    message.push_str("🔧 <b>Quick Actions:</b>\n");
                    message.push_str("• /profile_api - Manage API keys\n");
                    message.push_str("• /profile_settings - Update preferences\n");
                    message.push_str("• /subscription - Manage subscription\n");

                    Ok(message)
                }
                None => Ok(
                    "❌ <b>Profile not found</b>\n\nPlease use /start to initialize your account."
                        .to_string(),
                ),
            }
        } else {
            Ok("⚠️ <b>Service Unavailable</b>\n\nProfile service is temporarily unavailable. Please try again later.".to_string())
        }
    }

    /// Handle profile API management sub-command
    async fn handle_profile_api(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        Ok(format!(
            "🔑 <b>API Key Management</b>\n\n\
            👤 <b>User:</b> {}\n\n\
            🚧 <b>No API keys configured</b>\n\n\
            💡 <b>Supported Exchanges:</b>\n\
            • Binance\n\
            • Bybit\n\
            • OKX\n\
            • Bitget\n\n\
            🔒 <b>Security:</b> All API keys are encrypted and stored securely.\n\n\
            📧 <b>Contact support to enable API key management for your account.</b>",
            user_info.user_id
        ))
    }

    /// Handle profile settings sub-command
    async fn handle_profile_settings(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        Ok(format!(
            "⚙️ <b>Profile Settings</b>\n\n\
            👤 <b>User:</b> {}\n\n\
            📋 <b>Available Settings:</b>\n\
            • /settings_notifications - Alert preferences\n\
            • /settings_trading - Trading preferences\n\
            • /settings_alerts - Price alerts\n\
            • /settings_privacy - Privacy settings\n\
            • /settings_api - API management\n\n\
            🚧 <b>Settings management coming soon!</b>\n\n\
            Current settings are managed through the web dashboard.",
            user_info.user_id
        ))
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

    /// Handle opportunities manual scan sub-command
    async fn handle_opportunities_manual(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !permissions.can_request_manual_scans() {
            return Ok("❌ <b>Access Denied</b>\n\nManual opportunity scanning requires a Paid subscription or higher.\n\nUse <code>/subscription</code> to upgrade your plan.".to_string());
        }

        let mut message = "🔍 <b>Manual Opportunity Scan</b>\n\n".to_string();

        message.push_str(&format!(
            "👤 <b>Requested by:</b> {}\n\n",
            user_info.user_id
        ));

        // TODO: Implement actual manual scan trigger
        message.push_str("🚧 <b>Manual Scan Feature Coming Soon!</b>\n\n");
        message.push_str("This feature will:\n");
        message.push_str("• Trigger immediate market scan\n");
        message.push_str("• Apply custom filters for exchanges/pairs\n");
        message.push_str("• Return fresh opportunities within 30 seconds\n");
        message.push_str("• Prioritize based on your trading preferences\n\n");

        message.push_str("📋 <b>Usage Examples:</b>\n");
        message.push_str("• /opportunities_manual - Scan all markets\n");
        message.push_str("• Filter specific exchanges or pairs\n\n");

        message.push_str(
            "💡 <b>Current Alternative:</b> Use /opportunities_list for existing opportunities",
        );

        Ok(message)
    }

    /// Handle opportunities automation sub-command
    async fn handle_opportunities_auto(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        if !permissions.can_automate_trading() {
            return Ok("❌ <b>Access Denied</b>\n\nAutomated trading requires Premium subscription and configured API keys.\n\n• <code>/subscription</code> - Upgrade your plan\n• <code>/profile_api</code> - Configure API keys".to_string());
        }

        let message = format!(
            "🤖 <b>Automated Opportunities</b>\n\n👤 <b>User:</b> {}\n",
            user_info.user_id
        );

        Ok(message)
    }

    /// Handle settings notifications sub-command
    async fn handle_settings_notifications(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "🔔 <b>Notification Settings</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );

        message.push_str("📱 <b>Current Settings:</b>\n");
        message.push_str("• Opportunity alerts: ✅ Enabled\n");
        message.push_str("• Trade confirmations: ✅ Enabled\n");
        message.push_str("• Error notifications: ✅ Enabled\n");
        message.push_str("• Weekly reports: ✅ Enabled\n");
        message.push_str("• Price alerts: ❌ Disabled\n\n");

        message.push_str("⚙️ <b>Notification Types:</b>\n");
        message.push_str("• 📊 Arbitrage opportunities\n");
        message.push_str("• 💰 Trade executions\n");
        message.push_str("• ⚠️ System alerts\n");
        message.push_str("• 📈 Market movements\n");
        message.push_str("• 🔧 Account changes\n\n");

        message.push_str("🚧 <b>Notification Management Coming Soon!</b>\n\n");
        message.push_str("This feature will allow you to:\n");
        message.push_str("• Toggle individual notification types\n");
        message.push_str("• Set quiet hours\n");
        message.push_str("• Configure alert thresholds\n");
        message.push_str("• Choose notification channels (Telegram/Email)\n\n");

        message.push_str("📋 <b>Related Commands:</b>\n");
        message.push_str("• /settings_alerts - Alert configuration\n");
        message.push_str("• /settings_trading - Trading preferences\n");
        message.push_str("• /profile_settings - Profile settings");

        Ok(message)
    }

    /// Handle settings trading sub-command
    async fn handle_settings_trading(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "⚙️ <b>Trading Settings</b>\n\n👤 <b>User:</b> {}\n",
            user_info.user_id
        );

        if !permissions.can_trade {
            message.push_str("\n❌ <b>Trading Access:</b> Requires subscription upgrade\n");
            message.push_str("• /subscription - View upgrade options\n\n");
        } else {
            message.push_str("\n✅ <b>Trading Access:</b> Enabled\n\n");
        }

        message.push_str("🎯 <b>Current Settings:</b>\n");
        message.push_str("• Auto-trading: ❌ Disabled\n");
        message.push_str("• Max position size: $1,000\n");
        message.push_str("• Stop loss: 2.0%\n");
        message.push_str("• Take profit: 5.0%\n");
        message.push_str("• Min profit threshold: 0.5%\n\n");

        message.push_str("🔧 <b>Risk Management:</b>\n");
        message.push_str("• Daily loss limit: $100\n");
        message.push_str("• Max open positions: 3\n");
        message.push_str("• Trading hours: 24/7\n");
        message.push_str("• Slippage tolerance: 0.1%\n\n");

        message.push_str("🚧 <b>Trading Configuration Coming Soon!</b>\n\n");
        message.push_str("This feature will allow you to:\n");
        message.push_str("• Configure risk parameters\n");
        message.push_str("• Set position sizing rules\n");
        message.push_str("• Define stop-loss/take-profit levels\n");
        message.push_str("• Set trading time restrictions\n");
        message.push_str("• Configure exchange preferences\n\n");

        message.push_str("📋 <b>Prerequisites:</b>\n");
        if !permissions.can_automate_trading() {
            message.push_str("• ❌ Premium subscription required\n");
        } else {
            message.push_str("• ✅ Premium subscription active\n");
        }
        message.push_str("• ⚠️ Exchange API keys required\n");
        message.push_str("• /profile_api - Manage API keys");

        Ok(message)
    }

    /// Handle settings alerts sub-command
    async fn handle_settings_alerts(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "🚨 <b>Alert Configuration</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );

        message.push_str("📊 <b>Active Alerts:</b>\n");
        message.push_str("• BTC/USDT: Profit > 1.0% ✅\n");
        message.push_str("• ETH/USDT: Profit > 0.8% ✅\n");
        message.push_str("• General: Profit > 1.5% ✅\n\n");

        message.push_str("⚙️ <b>Alert Types:</b>\n");
        message.push_str("• 💰 Profit threshold alerts\n");
        message.push_str("• 📈 Price movement alerts\n");
        message.push_str("• 🔄 Volume spike alerts\n");
        message.push_str("• ⚠️ Risk limit alerts\n");
        message.push_str("• 🤖 Trading bot status alerts\n\n");

        message.push_str("📱 <b>Delivery Methods:</b>\n");
        message.push_str("• Telegram: ✅ Enabled\n");
        message.push_str("• Email: ❌ Not configured\n");
        message.push_str("• Push notifications: ❌ Not available\n\n");

        message.push_str("🚧 <b>Alert Management Coming Soon!</b>\n\n");
        message.push_str("This feature will allow you to:\n");
        message.push_str("• Create custom profit threshold alerts\n");
        message.push_str("• Set price movement notifications\n");
        message.push_str("• Configure volume and volatility alerts\n");
        message.push_str("• Set up multi-channel delivery\n");
        message.push_str("• Manage alert frequency and timing\n\n");

        message.push_str("📋 <b>Related Commands:</b>\n");
        message.push_str("• /settings_notifications - Notification preferences\n");
        message.push_str("• /settings_trading - Trading settings\n");
        message.push_str("• /opportunities_list - View current opportunities");

        Ok(message)
    }

    /// Handle settings privacy sub-command
    async fn handle_settings_privacy(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "🔒 <b>Privacy Settings</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );

        message.push_str("📊 <b>Data Collection:</b>\n");
        message.push_str("• Trading analytics: ✅ Enabled\n");
        message.push_str("• Performance metrics: ✅ Enabled\n");
        message.push_str("• Usage statistics: ✅ Enabled\n");
        message.push_str("• Error reporting: ✅ Enabled\n\n");

        message.push_str("👥 <b>Data Sharing:</b>\n");
        message.push_str("• Anonymous analytics: ✅ Enabled\n");
        message.push_str("• Marketing communications: ❌ Disabled\n");
        message.push_str("• Third-party integrations: ❌ Disabled\n");
        message.push_str("• Research participation: ❌ Disabled\n\n");

        message.push_str("🔐 <b>Account Security:</b>\n");
        message.push_str("• Two-factor authentication: ⚠️ Not configured\n");
        message.push_str("• Session monitoring: ✅ Enabled\n");
        message.push_str("• Login alerts: ✅ Enabled\n");
        message.push_str("• API key rotation: ⚠️ Manual\n\n");

        message.push_str("🗄️ <b>Data Retention:</b>\n");
        message.push_str("• Trading history: 2 years\n");
        message.push_str("• Chat logs: 30 days\n");
        message.push_str("• Analytics data: 1 year\n");
        message.push_str("• Error logs: 90 days\n\n");

        message.push_str("🚧 <b>Privacy Controls Coming Soon!</b>\n\n");
        message.push_str("This feature will allow you to:\n");
        message.push_str("• Control data collection preferences\n");
        message.push_str("• Manage data sharing settings\n");
        message.push_str("• Configure security preferences\n");
        message.push_str("• Request data exports\n");
        message.push_str("• Schedule automatic data deletion\n\n");

        message.push_str("📋 <b>Related Commands:</b>\n");
        message.push_str("• /profile_settings - Account preferences\n");
        message.push_str("• /settings_api - API security settings");

        Ok(message)
    }

    /// Handle settings API management sub-command
    async fn handle_settings_api(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "🔑 <b>API Settings</b>\n\n👤 <b>User:</b> {}\n",
            user_info.user_id
        );

        if !permissions.can_access_api_features() {
            message.push_str("\n❌ <b>API Access:</b> Not available on your plan\n");
            message.push_str("• /subscription - Upgrade for API access\n\n");
        } else {
            message.push_str("\n✅ <b>API Access:</b> Enabled\n\n");
        }

        message.push_str("🔧 <b>API Configuration:</b>\n");
        message.push_str("• Rate limiting: ✅ Enabled (1000/hour)\n");
        message.push_str("• IP restrictions: ❌ Not configured\n");
        message.push_str("• Webhook endpoints: ❌ Not configured\n");
        message.push_str("• API versioning: v1 (latest)\n\n");

        message.push_str("🔐 <b>Security Settings:</b>\n");
        message.push_str("• API key rotation: Every 90 days\n");
        message.push_str("• Request signing: ✅ Required\n");
        message.push_str("• Timestamp validation: ✅ Enabled\n");
        message.push_str("• Audit logging: ✅ Enabled\n\n");

        message.push_str("📊 <b>Usage Monitoring:</b>\n");
        message.push_str("• Daily requests: 0 / 1000\n");
        message.push_str("• Error rate: 0.0%\n");
        message.push_str("• Average response time: N/A\n");
        message.push_str("• Last activity: Never\n\n");

        message.push_str("🚧 <b>API Management Coming Soon!</b>\n\n");
        message.push_str("This feature will allow you to:\n");
        message.push_str("• Generate and manage API keys\n");
        message.push_str("• Configure rate limits and restrictions\n");
        message.push_str("• Set up webhook endpoints\n");
        message.push_str("• Monitor API usage and performance\n");
        message.push_str("• Configure security policies\n\n");

        message.push_str("📋 <b>Related Commands:</b>\n");
        message.push_str("• /profile_api - Exchange API keys\n");
        message.push_str("• /settings_privacy - Privacy controls");

        Ok(message)
    }

    /// Handle trade manual sub-command
    async fn handle_trade_manual(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        let mut message = format!(
            "💼 <b>Manual Trading</b>\n\n👤 <b>User:</b> {}\n\n",
            user_info.user_id
        );

        message.push_str("🚧 <b>Manual Trading Feature Coming Soon!</b>\n\n");
        message.push_str("This feature will allow you to:\n");
        message.push_str("• Execute arbitrage trades manually\n");
        message.push_str("• Review opportunity details before trading\n");
        message.push_str("• Set custom position sizes\n");
        message.push_str("• Apply personal risk management\n");
        message.push_str("• Track trade performance\n\n");

        message.push_str("📋 <b>Prerequisites:</b>\n");
        message.push_str("• ⚠️ Exchange API keys required\n");
        message.push_str("• ✅ Sufficient account balance\n");
        message.push_str("• ✅ Risk management settings\n\n");

        message.push_str("💡 <b>Getting Started:</b>\n");
        message.push_str("1. Configure API keys: /profile_api\n");
        message.push_str("2. Set trading preferences: /settings_trading\n");
        message.push_str("3. Review opportunities: /opportunities_list\n");
        message.push_str("4. Execute trades: /trade_manual\n\n");

        message.push_str("🔧 <b>Current Alternative:</b>\n");
        message.push_str("Use /opportunities_list to view available arbitrage opportunities");

        Ok(message)
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
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        Ok("📊 <b>Admin Statistics</b>\n\n\
            🚧 <b>Admin Stats Feature Coming Soon!</b>\n\n\
            This feature will provide:\n\
            • Real-time system metrics\n\
            • User activity statistics\n\
            • Performance analytics\n\
            • Error tracking\n\
            • Resource utilization\n\n\
            📈 <b>Available Commands:</b>\n\
            • /admin_config - Configuration panel\n\
            • /admin_users - User management\n\
            • /admin_stats - System statistics"
            .to_string())
    }

    /// Handle admin users command
    async fn handle_admin_users(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        Ok("👥 <b>Admin User Management</b>\n\n\
            🚧 <b>User Management Feature Coming Soon!</b>\n\n\
            This feature will provide:\n\
            • User account management\n\
            • Access level controls\n\
            • Subscription management\n\
            • Activity monitoring\n\
            • Bulk operations\n\n\
            🔧 <b>Available Commands:</b>\n\
            • /admin_config - Configuration panel\n\
            • /admin_stats - System statistics\n\
            • /admin_users - User management"
            .to_string())
    }

    /// Handle admin config command
    async fn handle_admin_config(
        _user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
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

    /// Handle trade status display
    async fn handle_trade_status(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
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

        message.push_str("\n💼 <b>Active Trading:</b>\n");
        message.push_str("• Open positions: 0\n");
        message.push_str("• Pending orders: 0\n");
        message.push_str("• Auto-trading: ❌ Disabled\n\n");

        message.push_str("📈 <b>Today's Performance:</b>\n");
        message.push_str("• Trades executed: 0\n");
        message.push_str("• Total volume: $0.00\n");
        message.push_str("• P&L: $0.00 (0.00%)\n");
        message.push_str("• Success rate: N/A\n\n");

        message.push_str("🎯 <b>Recent Activity:</b>\n");
        message.push_str("• No recent trading activity\n\n");

        message.push_str("🚧 <b>Live Trading Data Coming Soon!</b>\n\n");
        message.push_str("This feature will display:\n");
        message.push_str("• Real-time position status\n");
        message.push_str("• Live P&L tracking\n");
        message.push_str("• Trade history and analytics\n");
        message.push_str("• Risk metrics and exposure\n");
        message.push_str("• Performance benchmarks\n\n");

        message.push_str("📋 <b>Related Commands:</b>\n");
        message.push_str("• /trade_manual - Execute manual trades\n");
        message.push_str("• /trade_auto - Automated trading\n");
        message.push_str("• /opportunities_list - View opportunities");

        Ok(message)
    }

    /// Handle AI analyze sub-command
    async fn handle_ai_analyze(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        Ok(format!(
            "📊 <b>AI Market Analysis</b>\n\n\
            �� <b>User:</b> {}\n\
            💱 <b>Pair:</b> BTC/USDT\n\n\
            🚧 <b>AI Analysis Feature Coming Soon!</b>\n\n\
            This feature will provide:\n\
            • 📈 Technical indicator analysis\n\
            • 📊 Support and resistance levels\n\
            • 🔄 Volume pattern recognition\n\
            • 📰 News sentiment integration\n\
            • 🎯 Entry/exit recommendations\n\n\
            🔑 <b>Requirements:</b>\n\
            • Configured AI API key (OpenAI/Anthropic)\n\
            • Sufficient API usage credits\n\
            • Real-time market data access\n\n\
            📋 <b>Setup:</b> /profile_api - Configure API keys",
            user_info.user_id
        ))
    }

    /// Handle AI predict sub-command
    async fn handle_ai_predict(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        Ok(format!(
            "🔮 <b>AI Price Prediction</b>\n\n\
            👤 <b>User:</b> {}\n\
            💱 <b>Pair:</b> BTC/USDT\n\
            ⏰ <b>Timeframe:</b> 1h\n\n\
            🚧 <b>AI Prediction Feature Coming Soon!</b>\n\n\
            This feature will provide:\n\
            • 📈 Price direction forecasts\n\
            • 📊 Confidence intervals\n\
            • 🎯 Target price levels\n\
            • ⚠️ Risk assessments\n\
            • 📰 Factor analysis (news, events)\n\n\
            🤖 <b>AI Models:</b>\n\
            • LSTM neural networks\n\
            • Transformer models\n\
            • Ensemble predictions\n\
            • Sentiment integration\n\n\
            📋 <b>Usage:</b> Use /ai_predict for price forecasts",
            user_info.user_id
        ))
    }

    /// Handle AI sentiment sub-command
    async fn handle_ai_sentiment(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        _args: &[&str],
    ) -> ArbitrageResult<String> {
        Ok(format!(
            "💭 <b>AI Sentiment Analysis</b>\n\n\
            👤 <b>User:</b> {}\n\
            💱 <b>Pair:</b> BTC/USDT\n\n\
            🚧 <b>AI Sentiment Feature Coming Soon!</b>\n\n\
            This feature will analyze:\n\
            • 🐦 Twitter/X sentiment trends\n\
            • 📰 News article sentiment\n\
            • 💬 Reddit discussions\n\
            • 📺 YouTube content analysis\n\
            • 📊 Trading volume correlations\n\n\
            📊 <b>Sentiment Metrics:</b>\n\
            • Overall sentiment score (-100 to +100)\n\
            • Fear & Greed index\n\
            • Social media momentum\n\
            • Influencer impact scores\n\n\
            📋 <b>Data Sources:</b> Twitter API, News APIs, Reddit API, YouTube API",
            user_info.user_id
        ))
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
        let access_level = &permissions.role; // Use the actual enum from permissions
        let daily_limits = match access_level {
            crate::types::UserAccessLevel::Free => (5, 25.0), // 5 calls, $25 limit
            crate::types::UserAccessLevel::Paid => (50, 100.0), // 50 calls, $100 limit
            crate::types::UserAccessLevel::Premium => (100, 200.0), // 100 calls, $200 limit
            crate::types::UserAccessLevel::Admin => (200, 500.0), // 200 calls, $500 limit
            crate::types::UserAccessLevel::SuperAdmin => (u32::MAX, f64::INFINITY), // Unlimited
            crate::types::UserAccessLevel::Guest => (1, 5.0), // 1 call, $5 limit
            crate::types::UserAccessLevel::Registered => (3, 10.0), // 3 calls, $10 limit
            crate::types::UserAccessLevel::Verified => (10, 25.0), // 10 calls, $25 limit
            crate::types::UserAccessLevel::BetaUser => (50, 100.0), // 50 calls, $100 limit
            crate::types::UserAccessLevel::FreeWithoutAPI => (0, 0.0), // No AI access without API
            crate::types::UserAccessLevel::FreeWithAPI => (5, 25.0), // 5 calls, $25 limit
            crate::types::UserAccessLevel::SubscriptionWithAPI => (u32::MAX, f64::INFINITY), // Unlimited
            crate::types::UserAccessLevel::Basic => (3, 10.0), // 3 calls, $10 limit
            crate::types::UserAccessLevel::User => (3, 10.0),  // 3 calls, $10 limit
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
        message.push_str(&format!(
            "• 👤 Access Level: {}\n",
            match access_level {
                crate::types::UserAccessLevel::Free => "Free",
                crate::types::UserAccessLevel::Paid => "Paid",
                crate::types::UserAccessLevel::Premium => "Premium",
                crate::types::UserAccessLevel::Admin => "Admin",
                crate::types::UserAccessLevel::SuperAdmin => "SuperAdmin",
                crate::types::UserAccessLevel::Guest => "Guest",
                crate::types::UserAccessLevel::Registered => "Registered",
                crate::types::UserAccessLevel::Verified => "Verified",
                crate::types::UserAccessLevel::BetaUser => "Beta User",
                crate::types::UserAccessLevel::FreeWithoutAPI => "Free (No API)",
                crate::types::UserAccessLevel::FreeWithAPI => "Free (With API)",
                crate::types::UserAccessLevel::SubscriptionWithAPI => "Subscription (With API)",
                crate::types::UserAccessLevel::Basic => "Basic",
                crate::types::UserAccessLevel::User => "User",
            }
        ));
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
