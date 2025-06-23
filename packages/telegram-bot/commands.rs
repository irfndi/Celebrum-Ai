//! Telegram Command Router
//!
//! Routes commands to appropriate modular handlers based on:
//! - User permissions and access levels
//! - Subscription tiers
//! - Beta feature access
//! - Command type (admin, user, trading, etc.)

use crate::types::UserAccessLevel;
use crate::core::bot_client::{TelegramError, TelegramResult};
use std::sync::Arc;
use worker::console_log;

// Import UserInfo and UserPermissions from parent module
use super::{UserInfo, UserPermissions};

/// Command Router for delegating to modular handlers
pub struct CommandRouter;

impl CommandRouter {
    /// Route command to appropriate handler based on command type and user permissions
    pub async fn route_command(
        command: &str,
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        console_log!("🔀 Routing command: {} for user: {}", command, user_info.user_id);

        // Remove leading slash if present
        let clean_command = command.trim_start_matches('/');

        match clean_command {
            // Core user commands
            "start" => Self::handle_start_command(user_info, permissions, service_container).await,
            "help" => Self::handle_help_command(user_info, permissions).await,
            "profile" => Self::handle_profile_command(user_info, permissions, service_container).await,
            "settings" => Self::handle_settings_command(args, user_info, permissions, service_container).await,
            
            // Trading and opportunities commands
            "opportunities" | "opps" => Self::handle_opportunities_command(args, user_info, permissions, service_container).await,
            "balance" => Self::handle_balance_command(user_info, permissions, service_container).await,
            "trading" => Self::handle_trading_command(args, user_info, permissions, service_container).await,
            
            // Beta features (requires beta access)
            cmd if cmd.starts_with("beta") => {
                Self::handle_beta_command(cmd, args, user_info, permissions, service_container).await
            },
            
            // Admin commands (requires admin access)
            "admin" => Self::handle_admin_command(args, user_info, permissions, service_container).await,
            "superadmin" => Self::handle_superadmin_command(args, user_info, permissions, service_container).await,
            
            // Analytics and insights
            "analytics" => Self::handle_analytics_command(args, user_info, permissions, service_container).await,
            "insights" => Self::handle_insights_command(user_info, permissions, service_container).await,
            
            // Subscription management
            "subscribe" | "upgrade" => Self::handle_subscription_command(args, user_info, permissions, service_container).await,
            
            // Unknown command
            _ => Ok(format!(
                "❓ Unknown command: /{}

Type /help to see available commands.",
                clean_command
            )),
        }
    }

    /// Handle start command - welcome message and onboarding
    async fn handle_start_command(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        let welcome_message = format!(
            "🚀 <b>Welcome to ArbEdge Trading Platform, {}!</b>\n\n\
            I'm your AI-powered arbitrage trading assistant.\n\n\
            📊 <b>Your Account:</b>\n\
            • Access Level: {:?}\n\
            • Subscription: {:?}\n\n\
            🛠️ <b>Quick Start:</b>\n\
            • /opportunities - View arbitrage opportunities\n\
            • /profile - Manage your profile\n\
            • /help - See all commands\n\n\
            <i>Ready to maximize your trading potential? 🎯</i>",
            user_info.first_name.as_deref().unwrap_or("Trader"),
            permissions.role,
            permissions.subscription_tier
        );
        Ok(welcome_message)
    }

    /// Handle help command - show available commands based on user permissions
    async fn handle_help_command(
        user_info: &UserInfo,
        permissions: &UserPermissions,
    ) -> TelegramResult<String> {
        let mut help_text = String::from("📋 <b>Available Commands:</b>\n\n");
        
        // Core commands for all users
        help_text.push_str(
            "🔹 <b>Core Commands:</b>\n\
            /start - Welcome message and account info\n\
            /help - Show this help message\n\
            /profile - View and edit your profile\n\
            /settings - Manage preferences\n\n"
        );
        
        // Trading commands based on subscription
        help_text.push_str(
            "📈 <b>Trading Commands:</b>\n\
            /opportunities - View arbitrage opportunities\n\
            /balance - Check trading balance\n\
            /trading - Trading tools and settings\n\
            /analytics - View trading analytics\n\n"
        );
        
        // Beta features for eligible users
        if permissions.beta_access {
            help_text.push_str(
                "🧪 <b>Beta Features:</b>\n\
                /beta_ai - Advanced AI analysis\n\
                /beta_signals - Early signal access\n\n"
            );
        }
        
        // Admin commands
        match permissions.role {
            UserAccessLevel::Admin => {
                help_text.push_str(
                    "👑 <b>Admin Commands:</b>\n\
                    /admin users - Manage users\n\
                    /admin config - System configuration\n\n"
                );
            },
            UserAccessLevel::SuperAdmin => {
                help_text.push_str(
                    "👑 <b>Admin Commands:</b>\n\
                    /admin users - Manage users\n\
                    /admin config - System configuration\n\
                    /superadmin system - System management\n\
                    /superadmin analytics - Platform analytics\n\n"
                );
            },
            _ => {}
        }
        
        help_text.push_str("💡 <i>Tip: Use command arguments for more specific actions!</i>");
        Ok(help_text)
    }

    /// Handle profile command - delegate to profile handler
    async fn handle_profile_command(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        // This would delegate to the profile handler in the future
        // For now, return basic profile info
        let profile_info = format!(
            "👤 <b>Your Profile:</b>\n\n\
            🆔 User ID: {}\n\
            📛 Display Name: {}\n\
            🎯 Access Level: {:?}\n\
            💎 Subscription: {:?}\n\
            🧪 Beta Access: {}\n\
            📅 Member Since: {}\n\n\
            Use /settings to modify your preferences.",
            user_info.user_id,
            user_info.first_name.as_deref().unwrap_or("Not set"),
            permissions.role,
            permissions.subscription_tier,
            if permissions.beta_access { "✅ Enabled" } else { "❌ Disabled" },
            "2024-01-01" // TODO: Get actual creation date from service
        );
        Ok(profile_info)
    }

    /// Handle settings command
    async fn handle_settings_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        if args.is_empty() {
            return Ok(
                "⚙️ <b>Settings Menu:</b>\n\n\
                /settings notifications - Notification preferences\n\
                /settings trading - Trading preferences\n\
                /settings privacy - Privacy settings\n\
                /settings timezone - Timezone settings\n\n\
                <i>Choose a category to configure.</i>".to_string()
            );
        }
        
        match args[0] {
            "notifications" => Ok("🔔 Notification settings coming soon!".to_string()),
            "trading" => Ok("📈 Trading settings coming soon!".to_string()),
            "privacy" => Ok("🔒 Privacy settings coming soon!".to_string()),
            "timezone" => Ok("🌍 Timezone settings coming soon!".to_string()),
            _ => Ok("❓ Unknown setting category. Use /settings to see options.".to_string()),
        }
    }

    /// Handle opportunities command
    async fn handle_opportunities_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        // This would delegate to opportunities handler
        let action = args.first().unwrap_or(&"list");
        
        match *action {
            "list" => Ok("📊 Loading your personalized opportunities...".to_string()),
            "generate" => Ok("🔄 Generating new opportunities...".to_string()),
            "auto" => Ok("🤖 Auto-notification settings...".to_string()),
            _ => Ok("📊 Available: /opportunities list, /opportunities generate, /opportunities auto".to_string()),
        }
    }

    /// Handle balance command
    async fn handle_balance_command(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        // This would delegate to trading handler
        Ok("💰 Loading your trading balance...".to_string())
    }

    /// Handle trading command
    async fn handle_trading_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        let action = args.first().unwrap_or(&"status");
        
        match *action {
            "status" => Ok("📈 Trading status and active positions...".to_string()),
            "settings" => Ok("⚙️ Trading configuration options...".to_string()),
            "history" => Ok("📜 Trading history and performance...".to_string()),
            _ => Ok("📈 Available: /trading status, /trading settings, /trading history".to_string()),
        }
    }

    /// Handle beta commands (requires beta access)
    async fn handle_beta_command(
        command: &str,
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        if !permissions.beta_access {
            return Ok("🧪 Beta features are not enabled for your account. Contact support for access.".to_string());
        }
        
        match command {
            "beta_ai" => Ok("🤖 Advanced AI analysis coming soon!".to_string()),
            "beta_signals" => Ok("📡 Early signal access coming soon!".to_string()),
            _ => Ok("🧪 Available beta commands: /beta_ai, /beta_signals".to_string()),
        }
    }

    /// Handle admin commands (requires admin access)
    async fn handle_admin_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        if !matches!(permissions.role, UserAccessLevel::Admin | UserAccessLevel::SuperAdmin) {
            return Ok("🚫 Admin access required for this command.".to_string());
        }
        
        let action = args.first().unwrap_or(&"help");
        
        match *action {
            "users" => Ok("👥 User management interface...".to_string()),
            "config" => Ok("⚙️ System configuration panel...".to_string()),
            "help" => Ok("👑 Admin commands: /admin users, /admin config".to_string()),
            _ => Ok("👑 Available: /admin users, /admin config".to_string()),
        }
    }

    /// Handle superadmin commands (requires superadmin access)
    async fn handle_superadmin_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        if !matches!(permissions.role, UserAccessLevel::SuperAdmin) {
            return Ok("🚫 SuperAdmin access required for this command.".to_string());
        }
        
        let action = args.first().unwrap_or(&"help");
        
        match *action {
            "system" => Ok("🖥️ System management interface...".to_string()),
            "analytics" => Ok("📊 Platform analytics dashboard...".to_string()),
            "help" => Ok("👑 SuperAdmin commands: /superadmin system, /superadmin analytics".to_string()),
            _ => Ok("👑 Available: /superadmin system, /superadmin analytics".to_string()),
        }
    }

    /// Handle analytics command
    async fn handle_analytics_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        let period = args.first().unwrap_or(&"week");
        
        match *period {
            "day" => Ok("📊 Daily analytics report...".to_string()),
            "week" => Ok("📊 Weekly analytics report...".to_string()),
            "month" => Ok("📊 Monthly analytics report...".to_string()),
            _ => Ok("📊 Available: /analytics day, /analytics week, /analytics month".to_string()),
        }
    }

    /// Handle insights command
    async fn handle_insights_command(
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        Ok("💡 Personalized trading insights coming soon!".to_string())
    }

    /// Handle subscription command
    async fn handle_subscription_command(
        args: &[&str],
        user_info: &UserInfo,
        permissions: &UserPermissions,
        _service_container: &Arc<ServiceContainer>,
    ) -> TelegramResult<String> {
        let action = args.first().unwrap_or(&"info");
        
        match *action {
            "info" => Ok(format!(
                "💎 <b>Subscription Info:</b>\n\n\
                Current Plan: {}\n\
                Status: Active\n\n\
                Use /subscribe upgrade to see available plans.",
                permissions.subscription_tier
            )),
            "upgrade" => Ok("⬆️ Subscription upgrade options coming soon!".to_string()),
            _ => Ok("💎 Available: /subscribe info, /subscribe upgrade".to_string()),
        }
    }
}