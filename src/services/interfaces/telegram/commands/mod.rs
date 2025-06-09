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
pub mod opportunities;
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
            "🎯 Routing command '{}' for user {}",
            command,
            user_info.user_id
        );

        match command {
            "/start" => Self::handle_start(user_info, permissions, service_container).await,
            "/help" => Self::handle_help(user_info, permissions, service_container).await,
            "/profile" => Self::handle_profile(user_info, permissions, service_container).await,
            "/subscription" => {
                Self::handle_subscription(user_info, permissions, service_container).await
            }
            "/opportunities" => {
                Self::handle_opportunities(user_info, permissions, service_container).await
            }
            "/beta" => Self::handle_beta(user_info, permissions, service_container, args).await,
            "/settings" => {
                Self::handle_settings(user_info, permissions, service_container, args).await
            }
            "/admin" => {
                if !permissions.is_admin {
                    return Ok("❌ <b>Access Denied</b>\n\nAdmin privileges required.".to_string());
                }
                Self::handle_admin(user_info, permissions, service_container, args).await
            }
            _ => Ok("❓ <b>Unknown command.</b>\n\nType /help for available commands.".to_string()),
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
                    • /opportunities - See latest arbitrage opportunities\n\
                    • /profile - View your account details\n\
                    • /subscription - Manage your subscription\n\
                    • /settings - Configure preferences\n\
                    • /help - All available commands\n\n\
                    💡 Ready to start trading? Check out /opportunities!",
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
                    • /opportunities - See your first opportunities (3 free daily)\n\
                    • /profile - Complete your profile setup\n\
                    • /subscription - Explore Premium features\n\
                    • /help - Learn about all commands\n\n\
                    💎 <b>Tip:</b> Upgrade to Premium for unlimited opportunities and real-time alerts!\n\n\
                    🔗 <b>Next Step:</b> Try /opportunities to see what's available!",
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
        help_text.push_str("• /opportunities - View latest arbitrage opportunities\n");
        help_text.push_str("• /profile - View your account details\n");
        help_text.push_str("• /subscription - Manage your subscription\n");
        help_text.push_str("• /settings - Configure your preferences\n\n");

        help_text.push_str("ℹ️ <b>General Commands:</b>\n");
        help_text.push_str("• /start - Initialize your account\n");
        help_text.push_str("• /help - Show this help message\n\n");

        // Beta features for beta users
        if permissions.beta_access {
            help_text.push_str("🧪 <b>Beta Commands:</b>\n");
            help_text.push_str("• /beta - Access beta features menu\n");
            help_text.push_str("• /beta opportunities - Enhanced opportunity analysis\n");
            help_text.push_str("• /beta ai - Advanced AI features\n");
            help_text.push_str("• /beta analytics - Performance analytics\n\n");
        }

        // Admin commands for admin users
        if permissions.is_admin {
            help_text.push_str("🔧 <b>Admin Commands:</b>\n");
            help_text.push_str("• /admin - Access admin panel\n");

            // SuperAdmin specific features
            if matches!(permissions.role, crate::types::UserAccessLevel::SuperAdmin) {
                help_text.push_str("• /admin system - System management\n");
                help_text.push_str("• /admin users - User management\n");
            }
            help_text.push('\n');
        }

        // Subscription-based features
        match permissions.subscription_tier.as_str() {
            "free" => {
                help_text.push_str("💡 <b>Upgrade Benefits:</b>\n");
                help_text.push_str("• Unlimited opportunities with Premium\n");
                help_text.push_str("• Real-time notifications\n");
                help_text.push_str("• Advanced analytics\n");
                help_text.push_str("Use /subscription to upgrade!\n\n");
            }
            "premium" | "enterprise" => {
                help_text.push_str("💎 <b>Premium Features Available:</b>\n");
                help_text.push_str("• Unlimited opportunities\n");
                help_text.push_str("• Real-time notifications\n");
                help_text.push_str("• Advanced analytics\n\n");
            }
            _ => {}
        }

        help_text.push_str("💡 <b>Need more help?</b>\n");
        help_text.push_str("Contact support or visit our documentation.");

        Ok(help_text)
    }

    /// Handle /profile command
    async fn handle_profile(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
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

                    if let Some(username) = &profile.telegram_username {
                        message.push_str(&format!("👤 <b>Username:</b> @{}\n", username));
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
                        "📊 <b>Status:</b> {}\n",
                        if profile.subscription.is_active {
                            "✅ Active"
                        } else {
                            "❌ Inactive"
                        }
                    ));

                    let dt = chrono::DateTime::from_timestamp(profile.last_active as i64, 0)
                        .unwrap_or_else(chrono::Utc::now);
                    message.push_str(&format!(
                        "🕒 <b>Last Active:</b> {}\n",
                        dt.format("%Y-%m-%d %H:%M UTC")
                    ));

                    message.push_str(&format!(
                        "📈 <b>Total Trades:</b> {}\n",
                        profile.total_trades
                    ));
                    message.push_str(&format!(
                        "💰 <b>Total P&L:</b> ${:.2} USDT\n",
                        profile.total_pnl_usdt
                    ));

                    Ok(message)
                }
                None => Ok(
                    "❌ <b>Profile not found.</b>\n\nUse /start to create your account."
                        .to_string(),
                ),
            }
        } else {
            Ok("⚠️ <b>Profile service unavailable.</b>\n\nPlease try again later.".to_string())
        }
    }

    /// Handle /subscription command
    async fn handle_subscription(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        let mut message = "💎 <b>Subscription Management</b>\n\n".to_string();

        message.push_str(&format!(
            "📋 <b>Current Plan:</b> {}\n\n",
            permissions.subscription_tier.to_uppercase()
        ));

        match permissions.subscription_tier.as_str() {
            "free" => {
                message.push_str("🆓 <b>Free Plan Features:</b>\n");
                message.push_str("• 3 opportunities per day\n");
                message.push_str("• 5-minute delayed alerts\n");
                message.push_str("• Basic market data\n\n");

                message.push_str("💎 <b>Upgrade to Premium:</b>\n");
                message.push_str("• Unlimited opportunities\n");
                message.push_str("• Real-time alerts\n");
                message.push_str("• Advanced analytics\n");
                message.push_str("• Priority support\n\n");

                message.push_str("🚧 <b>Coming Soon:</b> Direct upgrade via bot!");
            }
            "premium" => {
                message.push_str("💎 <b>Premium Plan Active:</b>\n");
                message.push_str("• ✅ Unlimited opportunities\n");
                message.push_str("• ✅ Real-time alerts\n");
                message.push_str("• ✅ Advanced analytics\n");
                message.push_str("• ✅ Priority support\n\n");

                message.push_str("🔄 <b>Subscription Management:</b>\n");
                message.push_str("Visit our website to manage your subscription.");
            }
            _ => {
                message
                    .push_str("🔧 <b>Subscription status unknown.</b>\n\nPlease contact support.");
            }
        }

        Ok(message)
    }

    /// Handle /opportunities command
    async fn handle_opportunities(
        user_info: &UserInfo,
        _permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> ArbitrageResult<String> {
        // Get recent opportunities from distribution service
        let distribution_service = &service_container.distribution_service;

        match distribution_service.get_user_opportunities(&user_info.user_id.to_string()).await {
            Ok(opportunities) => {
                if opportunities.is_empty() {
                    return Ok("📊 <b>No opportunities available</b>\n\n🔄 Check back in a few minutes for new arbitrage opportunities.".to_string());
                }

                let mut message = "💰 <b>Latest Arbitrage Opportunities</b>\n\n".to_string();

                for (i, opp) in opportunities.iter().take(5).enumerate() {
                    message.push_str(&format!(
                        "🔹 <b>Opportunity #{}</b>\n\
                        💱 <b>Pair:</b> {}\n\
                        📈 <b>Profit:</b> {:.2}%\n\
                        💵 <b>Volume:</b> ${:.2}\n\
                        🏪 <b>Exchanges:</b> {} → {}\n\
                        ⭐ <b>Confidence:</b> {:.1}/10\n\n",
                        i + 1,
                        opp.trading_pair,
                        opp.profit_percentage,
                        opp.volume,
                        opp.buy_exchange,
                        opp.sell_exchange,
                        opp.confidence_score
                    ));
                }

                if opportunities.len() > 5 {
                    message.push_str(&format!("📋 <i>+{} more opportunities available</i>\n\n", opportunities.len() - 5));
                }

                message.push_str("🔄 <b>Auto-refresh:</b> Every 30 seconds\n");
                message.push_str("💡 <b>Tip:</b> Higher confidence scores indicate better opportunities");

                Ok(message)
            }
            Err(_) => Ok("⚠️ <b>Unable to fetch opportunities</b>\n\nThe opportunity service is temporarily unavailable. Please try again later.".to_string()),
        }
    }

    /// Handle /beta command
    async fn handle_beta(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        if !permissions.beta_access {
            return Ok("🚫 <b>Beta Access Required</b>\n\nBeta features are available to invited users only.\nContact support to request beta access.".to_string());
        }

        if args.is_empty() {
            return Ok("🧪 <b>Beta Features Menu</b>\n\n\
                🎯 <b>Available Beta Commands:</b>\n\
                • /beta opportunities - Enhanced opportunity analysis\n\
                • /beta ai - Advanced AI trading insights\n\
                • /beta analytics - Performance analytics dashboard\n\
                • /beta feedback - Send beta feedback\n\n\
                💡 <b>Note:</b> Beta features are experimental and may change."
                .to_string());
        }

        match args[0] {
            "opportunities" => Ok("🧪 <b>Beta Opportunities</b>\n\n🚧 Enhanced opportunity analysis coming soon!\n\nPlanned features:\n• Advanced filtering\n• Risk assessment\n• Historical performance data\n• Custom alerts".to_string()),
            "ai" => Ok("🧪 <b>Beta AI Features</b>\n\n🚧 AI trading insights coming soon!\n\nPlanned features:\n• Market sentiment analysis\n• Predictive modeling\n• Trading recommendations\n• Risk optimization".to_string()),
            "analytics" => Ok("🧪 <b>Beta Analytics</b>\n\n🚧 Performance analytics coming soon!\n\nPlanned features:\n• Trading performance metrics\n• Profit/loss tracking\n• Strategy backtesting\n• Custom dashboards".to_string()),
            "feedback" => Ok("🧪 <b>Beta Feedback</b>\n\n📝 We value your feedback!\n\nPlease send your thoughts and suggestions about:\n• Feature requests\n• Bug reports\n• User experience improvements\n• Performance issues\n\nContact our beta support team with your feedback.".to_string()),
            _ => Ok("❓ <b>Unknown beta command.</b>\n\nUse /beta to see available beta features.".to_string()),
        }
    }

    /// Handle /settings command
    async fn handle_settings(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        if args.is_empty() {
            let mut message = "⚙️ <b>Settings & Configuration</b>\n\n".to_string();

            message.push_str("🎯 <b>Available Settings:</b>\n");
            message.push_str("• /settings notifications - Notification preferences\n");
            message.push_str("• /settings trading - Trading configuration\n");
            message.push_str("• /settings preferences - General preferences\n\n");

            if permissions.is_admin {
                message.push_str("🔧 <b>Admin Settings:</b>\n");
                message.push_str("• /admin settings - System configuration\n\n");
            }

            match permissions.subscription_tier.as_str() {
                "free" => {
                    message.push_str("💡 <b>Upgrade for More Settings:</b>\n");
                    message.push_str("• Advanced trading preferences\n");
                    message.push_str("• Custom notification rules\n");
                    message.push_str("Use /subscription to upgrade!\n\n");
                }
                "premium" | "enterprise" => {
                    message.push_str("💎 <b>Premium Settings Available:</b>\n");
                    message.push_str("• Advanced trading rules\n");
                    message.push_str("• Custom notifications\n\n");
                }
                _ => {}
            }

            message.push_str("🚧 <b>Note:</b> Advanced settings coming soon!");
            return Ok(message);
        }

        match args[0] {
            "notifications" => Ok("🔔 <b>Notification Settings</b>\n\n🚧 Notification settings coming soon!\n\nPlanned features:\n• Alert preferences\n• Custom triggers\n• Delivery methods\n• Quiet hours".to_string()),
            "trading" => Ok("📈 <b>Trading Settings</b>\n\n🚧 Trading settings coming soon!\n\nPlanned features:\n• Risk tolerance\n• Position sizing\n• Stop-loss rules\n• Auto-trading preferences".to_string()),
            "preferences" => Ok("👤 <b>User Preferences</b>\n\n🚧 User preferences coming soon!\n\nPlanned features:\n• Language settings\n• Timezone configuration\n• Display preferences\n• Privacy settings".to_string()),
            _ => Ok("❓ <b>Unknown settings option.</b>\n\nUse /settings to see available options.".to_string()),
        }
    }

    /// Handle /admin command
    async fn handle_admin(
        _user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
        args: &[&str],
    ) -> ArbitrageResult<String> {
        if args.is_empty() {
            let mut message = "🔧 <b>Admin Panel</b>\n\n".to_string();

            message.push_str("🎯 <b>Available Admin Commands:</b>\n");
            message.push_str("• /admin users - User management\n");
            message.push_str("• /admin system - System status\n");
            message.push_str("• /admin settings - Configuration\n\n");

            if matches!(permissions.role, crate::types::UserAccessLevel::SuperAdmin) {
                message.push_str("👑 <b>SuperAdmin Features:</b>\n");
                message.push_str("• /admin database - Database management\n");
                message.push_str("• /admin monitoring - System monitoring\n\n");
            }

            message.push_str("🚧 <b>Note:</b> Admin features are being developed.");
            return Ok(message);
        }

        match args[0] {
            "users" => Ok("👥 <b>User Management</b>\n\n🚧 User management coming soon!\n\nPlanned features:\n• View user statistics\n• Manage subscriptions\n• Handle support requests\n• Monitor user activity".to_string()),
            "system" => Ok("🖥️ <b>System Status</b>\n\n🚧 System monitoring coming soon!\n\nPlanned features:\n• Service health checks\n• Performance metrics\n• Error monitoring\n• Resource usage".to_string()),
            "settings" => Ok("⚙️ <b>Admin Settings</b>\n\n🚧 Admin configuration coming soon!\n\nPlanned features:\n• System configuration\n• Feature flags\n• Maintenance mode\n• Global settings".to_string()),
            "database" if matches!(permissions.role, crate::types::UserAccessLevel::SuperAdmin) => {
                Ok("🗄️ <b>Database Management</b>\n\n🚧 Database tools coming soon!\n\nPlanned features:\n• Database health\n• Query analytics\n• Backup status\n• Migration tools".to_string())
            }
            "monitoring" if matches!(permissions.role, crate::types::UserAccessLevel::SuperAdmin) => {
                Ok("📊 <b>System Monitoring</b>\n\n🚧 Advanced monitoring coming soon!\n\nPlanned features:\n• Real-time metrics\n• Alert management\n• Log analysis\n• Performance dashboards".to_string())
            }
            _ => Ok("❓ <b>Unknown admin command.</b>\n\nUse /admin to see available options.".to_string()),
        }
    }
}
