//! Command Handlers Module
//!
//! This module contains all the command handlers for the Telegram bot.
//! Each handler implements the CommandHandler trait and handles specific commands.

pub mod admin_handler;
pub mod balance_handler;
pub mod help_handler;
pub mod opportunities_handler;
pub mod settings_handler;
pub mod start_handler;

// Re-export handlers for easy access
pub use admin_handler::AdminHandler;
pub use balance_handler::BalanceHandler;
pub use help_handler::HelpHandler;
pub use opportunities_handler::OpportunitiesHandler;
pub use settings_handler::SettingsHandler;
pub use start_handler::StartHandler;

use crate::core::command_router::CommandRouter;
use worker::console_log;

/// Initialize and register all command handlers
pub fn initialize_command_handlers() -> CommandRouter {
    console_log!("🔧 Initializing command handlers...");

    let mut router = CommandRouter::new();

    // Register basic commands
    router.register_handler(Box::new(StartHandler::new()));
    router.register_handler(Box::new(HelpHandler::new()));

    // Register trading commands
    router.register_handler(Box::new(OpportunitiesHandler::new()));
    router.register_handler(Box::new(BalanceHandler::new()));

    // Register user commands
    router.register_handler(Box::new(SettingsHandler::new()));

    // Register admin commands
    router.register_handler(Box::new(AdminHandler::new()));

    console_log!("✅ Command handlers initialized successfully");
    console_log!("📋 Registered commands: {:?}", router.get_command_names());

    router
}

// Removed unused functions get_available_commands and is_valid_command

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validation() {
        assert!(is_valid_command("/start"));
        assert!(is_valid_command("start"));
        assert!(is_valid_command("/help"));
        assert!(is_valid_command("opportunities"));
        assert!(!is_valid_command("invalid"));
        assert!(!is_valid_command("/unknown"));
    }

    #[test]
    fn test_available_commands() {
        let commands = get_available_commands();
        assert!(commands.contains(&"start"));
        assert!(commands.contains(&"help"));
        assert!(commands.contains(&"opportunities"));
        assert!(commands.contains(&"balance"));
        assert!(commands.contains(&"settings"));
        assert!(commands.contains(&"admin"));
    }

    #[tokio::test]
    async fn test_handler_initialization() {
        let router = initialize_command_handlers();
        let command_names = router.get_command_names();

        assert!(command_names.contains(&"start".to_string()));
        assert!(command_names.contains(&"help".to_string()));
        assert!(command_names.contains(&"opportunities".to_string()));
        assert!(command_names.contains(&"balance".to_string()));
        assert!(command_names.contains(&"settings".to_string()));
        assert!(command_names.contains(&"admin".to_string()));
    }
}
