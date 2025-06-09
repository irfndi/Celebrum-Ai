//! Onboarding Commands
//!
//! User onboarding and account creation commands

use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::interfaces::telegram::{UserInfo, UserPermissions};
use crate::utils::ArbitrageResult;
use std::sync::Arc;

/// Handle start command
pub async fn handle_start_command(
    _service_container: &Arc<ServiceContainer>,
    user_info: &UserInfo,
    _permissions: &UserPermissions,
    _args: &[&str],
) -> ArbitrageResult<String> {
    let message = format!(
        "🎉 <b>Welcome to ArbEdge!</b>\n\n\
        Hello {}! 👋\n\n\
        🔍 <b>What is ArbEdge?</b>\n\
        ArbEdge is your AI-powered arbitrage opportunity detector. We scan multiple exchanges \
        to find profitable trading opportunities.\n\n\
        📱 <b>Available Commands:</b>\n\
        /help - Show all commands\n\
        /profile - View your profile\n\
        /opportunities - Browse opportunities\n\
        /settings - Configure preferences\n\n\
        🚀 <b>Get Started:</b>\n\
        Try /help to see all available commands!\n\n\
        🔐 Your account is automatically created and ready to use.",
        user_info.first_name.as_deref().unwrap_or("there")
    );

    Ok(message)
}
