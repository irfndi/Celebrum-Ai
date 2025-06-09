//! Profile Commands
//!
//! Commands for user profile management and subscription

use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::interfaces::telegram::{UserInfo, UserPermissions};
use crate::utils::ArbitrageResult;
use std::sync::Arc;

/// Handle profile command
pub async fn handle_profile_command(
    _service_container: &Arc<ServiceContainer>,
    user_info: &UserInfo,
    _permissions: &UserPermissions,
    _args: &[&str],
) -> ArbitrageResult<String> {
    let message = format!(
        "👤 <b>Your Profile</b>\n\n\
        🆔 <b>User ID:</b> <code>{}</code>\n\
        📱 <b>Telegram ID:</b> <code>{}</code>\n\
        👤 <b>Username:</b> {}\n\n\
        🚧 <b>Full profile coming soon!</b>\n\n\
        Planned features:\n\
        • Subscription management\n\
        • Trading statistics\n\
        • API key management\n\
        • Preference settings",
        user_info.user_id,
        user_info.user_id,
        user_info.username.as_deref().unwrap_or("Not set")
    );

    Ok(message)
}

/// Handle subscription command
pub async fn handle_subscription_command(
    _service_container: &Arc<ServiceContainer>,
    _user_info: &UserInfo,
    permissions: &UserPermissions,
    _args: &[&str],
) -> ArbitrageResult<String> {
    let message = format!(
        "💎 <b>Subscription Management</b>\n\n\
        📋 <b>Current Plan:</b> {}\n\n\
        🚧 <b>Subscription features coming soon!</b>\n\n\
        Planned features:\n\
        • Direct upgrade via bot\n\
        • Billing management\n\
        • Plan comparison\n\
        • Usage analytics",
        permissions.subscription_tier.to_uppercase()
    );

    Ok(message)
}
