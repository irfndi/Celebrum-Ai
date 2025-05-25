use crate::services::user_profile::UserProfileService;
use crate::types::CommandPermission;
use serde_json::{json, Value};

/// Represents a single inline keyboard button with optional permission requirements
#[derive(Debug, Clone)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub callback_data: String,
    pub required_permission: Option<CommandPermission>,
}

/// Represents an inline keyboard with role-based filtering
#[derive(Debug, Clone)]
pub struct InlineKeyboard {
    pub buttons: Vec<Vec<InlineKeyboardButton>>, // Rows of buttons
}

impl Default for InlineKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineKeyboard {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
        }
    }

    /// Add a row of buttons
    pub fn add_row(&mut self, buttons: Vec<InlineKeyboardButton>) -> &mut Self {
        self.buttons.push(buttons);
        self
    }

    /// Add a single button as a new row
    pub fn add_button(&mut self, button: InlineKeyboardButton) -> &mut Self {
        self.buttons.push(vec![button]);
        self
    }

    /// Filter buttons based on user permissions
    pub async fn filter_by_permissions(
        &self,
        user_profile_service: &Option<UserProfileService>,
        user_id: &str,
    ) -> InlineKeyboard {
        let mut filtered_keyboard = InlineKeyboard::new();

        for row in &self.buttons {
            let mut filtered_row = Vec::new();

            for button in row {
                // Check if button requires permission
                if let Some(required_permission) = &button.required_permission {
                    // Check if user has permission
                    if Self::check_user_permission_static(
                        user_profile_service,
                        user_id,
                        required_permission,
                    )
                    .await
                    {
                        filtered_row.push(button.clone());
                    }
                } else {
                    // No permission required, always show
                    filtered_row.push(button.clone());
                }
            }

            // Only add row if it has buttons
            if !filtered_row.is_empty() {
                filtered_keyboard.add_row(filtered_row);
            }
        }

        filtered_keyboard
    }

    /// Convert to Telegram JSON format
    pub fn to_json(&self) -> Value {
        let keyboard_rows: Vec<Value> = self
            .buttons
            .iter()
            .map(|row| {
                let row_buttons: Vec<Value> = row
                    .iter()
                    .map(|button| {
                        json!({
                            "text": button.text,
                            "callback_data": button.callback_data
                        })
                    })
                    .collect();
                json!(row_buttons)
            })
            .collect();

        json!({
            "inline_keyboard": keyboard_rows
        })
    }

    /// Static permission check helper
    async fn check_user_permission_static(
        user_profile_service: &Option<UserProfileService>,
        user_id: &str,
        permission: &CommandPermission,
    ) -> bool {
        let Some(ref user_service) = user_profile_service else {
            return false;
        };

        // Helper function to redact sensitive user data for logging
        fn redact_user_id(user_id: &str) -> String {
            if user_id.len() <= 4 {
                "*".repeat(user_id.len())
            } else {
                format!("{}***{}", &user_id[..2], &user_id[user_id.len() - 2..])
            }
        }

        fn redact_telegram_id(telegram_id: i64) -> String {
            let id_str = telegram_id.to_string();
            if id_str.len() <= 4 {
                "*".repeat(id_str.len())
            } else {
                format!("{}***{}", &id_str[..2], &id_str[id_str.len() - 2..])
            }
        }

        // Safely parse user ID - return false for invalid IDs
        let telegram_id = match user_id.parse::<i64>() {
            Ok(id) if id > 0 => id, // Telegram user IDs start from 1
            Ok(_) => {
                log::warn!(
                    "Invalid user ID: user IDs must be positive: {}",
                    redact_user_id(user_id)
                );
                return false;
            }
            Err(e) => {
                log::warn!(
                    "Failed to parse user ID '{}': {}",
                    redact_user_id(user_id),
                    e
                );
                return false;
            }
        };

        let user_profile = match user_service.get_user_by_telegram_id(telegram_id).await {
            Ok(Some(profile)) => profile,
            Ok(None) => {
                log::warn!(
                    "User not found in database: telegram_id={}",
                    redact_telegram_id(telegram_id)
                );
                return false;
            }
            Err(e) => {
                log::warn!(
                    "Database error while fetching user profile for telegram_id={}: {}",
                    redact_telegram_id(telegram_id),
                    e
                );
                return false;
            }
        };

        user_profile.has_permission(permission.clone())
    }

    /// Create main menu keyboard with role-based filtering
    pub fn create_main_menu() -> Self {
        let mut keyboard = InlineKeyboard::new();

        // Row 1: Basic features (available to all users)
        keyboard.add_row(vec![
            InlineKeyboardButton::new("📊 Opportunities", "opportunities"),
            InlineKeyboardButton::new("📈 Categories", "categories"),
        ]);

        // Row 2: AI features (require specific permissions)
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "🤖 AI Insights",
                "ai_insights",
                CommandPermission::AIEnhancedOpportunities,
            ),
            InlineKeyboardButton::with_permission(
                "⚡ Risk Assessment",
                "risk_assessment",
                CommandPermission::AdvancedAnalytics,
            ),
        ]);

        // Row 3: Trading features (require trading permissions)
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "💰 Balance",
                "balance",
                CommandPermission::AdvancedAnalytics,
            ),
            InlineKeyboardButton::with_permission(
                "📋 Orders",
                "orders",
                CommandPermission::AdvancedAnalytics,
            ),
            InlineKeyboardButton::with_permission(
                "📊 Positions",
                "positions",
                CommandPermission::AdvancedAnalytics,
            ),
        ]);

        // Row 4: Trading actions (require manual trading permissions)
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "🛒 Buy",
                "buy",
                CommandPermission::ManualTrading,
            ),
            InlineKeyboardButton::with_permission(
                "💸 Sell",
                "sell",
                CommandPermission::ManualTrading,
            ),
        ]);

        // Row 5: Admin features (require admin permissions)
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "👥 Users",
                "admin_users",
                CommandPermission::SystemAdministration,
            ),
            InlineKeyboardButton::with_permission(
                "📊 Stats",
                "admin_stats",
                CommandPermission::SystemAdministration,
            ),
            InlineKeyboardButton::with_permission(
                "⚙️ Config",
                "admin_config",
                CommandPermission::SystemAdministration,
            ),
        ]);

        // Row 6: Settings (available to all users)
        keyboard.add_row(vec![
            InlineKeyboardButton::new("⚙️ Settings", "settings"),
            InlineKeyboardButton::new("❓ Help", "help"),
        ]);

        keyboard
    }

    /// Create opportunities keyboard with filtering
    pub fn create_opportunities_menu() -> Self {
        let mut keyboard = InlineKeyboard::new();

        // Row 1: Basic opportunity viewing
        keyboard.add_row(vec![
            InlineKeyboardButton::new("📊 All Opportunities", "opportunities_all"),
            InlineKeyboardButton::new("🔥 Top Opportunities", "opportunities_top"),
        ]);

        // Row 2: Advanced analytics
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "📈 Enhanced Analysis",
                "opportunities_enhanced",
                CommandPermission::AdvancedAnalytics,
            ),
            InlineKeyboardButton::with_permission(
                "🤖 AI Enhanced",
                "opportunities_ai",
                CommandPermission::AIEnhancedOpportunities,
            ),
        ]);

        // Row 3: Auto trading
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "⚡ Auto Enable",
                "auto_enable",
                CommandPermission::AutomatedTrading,
            ),
            InlineKeyboardButton::with_permission(
                "🛑 Auto Disable",
                "auto_disable",
                CommandPermission::AutomatedTrading,
            ),
            InlineKeyboardButton::with_permission(
                "⚙️ Auto Config",
                "auto_config",
                CommandPermission::AutomatedTrading,
            ),
        ]);

        keyboard.add_row(vec![InlineKeyboardButton::new("🔙 Back", "main_menu")]);

        keyboard
    }

    /// Create admin keyboard with all admin functions
    pub fn create_admin_menu() -> Self {
        let mut keyboard = InlineKeyboard::new();

        // All buttons require SystemAdministration permission
        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "👥 User Management",
                "admin_users",
                CommandPermission::SystemAdministration,
            ),
            InlineKeyboardButton::with_permission(
                "📊 System Stats",
                "admin_stats",
                CommandPermission::SystemAdministration,
            ),
        ]);

        keyboard.add_row(vec![
            InlineKeyboardButton::with_permission(
                "⚙️ Configuration",
                "admin_config",
                CommandPermission::SystemAdministration,
            ),
            InlineKeyboardButton::with_permission(
                "📢 Broadcast",
                "admin_broadcast",
                CommandPermission::SystemAdministration,
            ),
        ]);

        keyboard.add_row(vec![InlineKeyboardButton::with_permission(
            "🏢 Group Config",
            "admin_group_config",
            CommandPermission::SystemAdministration,
        )]);

        keyboard.add_row(vec![InlineKeyboardButton::new("🔙 Back", "main_menu")]);

        keyboard
    }
}

impl InlineKeyboardButton {
    pub fn new(text: &str, callback_data: &str) -> Self {
        Self {
            text: text.to_string(),
            callback_data: callback_data.to_string(),
            required_permission: None,
        }
    }

    pub fn with_permission(text: &str, callback_data: &str, permission: CommandPermission) -> Self {
        Self {
            text: text.to_string(),
            callback_data: callback_data.to_string(),
            required_permission: Some(permission),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_keyboard_creation() {
        let keyboard = InlineKeyboard::create_main_menu();

        // Should have multiple rows
        assert!(!keyboard.buttons.is_empty());

        // First row should have basic features
        assert!(keyboard.buttons[0].len() >= 2);
        assert_eq!(keyboard.buttons[0][0].text, "📊 Opportunities");
        assert_eq!(keyboard.buttons[0][1].text, "📈 Categories");
    }

    #[test]
    fn test_button_permission_assignment() {
        let button_public = InlineKeyboardButton::new("Public", "public");
        let button_private = InlineKeyboardButton::with_permission(
            "Private",
            "private",
            CommandPermission::ManualTrading,
        );

        assert!(button_public.required_permission.is_none());
        assert!(button_private.required_permission.is_some());
        assert_eq!(
            button_private.required_permission.unwrap(),
            CommandPermission::ManualTrading
        );
    }

    #[test]
    fn test_keyboard_json_conversion() {
        let mut keyboard = InlineKeyboard::new();
        keyboard.add_row(vec![InlineKeyboardButton::new("Test", "test_callback")]);

        let json = keyboard.to_json();
        assert!(json["inline_keyboard"].is_array());
        assert!(json["inline_keyboard"][0].is_array());
        assert_eq!(json["inline_keyboard"][0][0]["text"], "Test");
        assert_eq!(
            json["inline_keyboard"][0][0]["callback_data"],
            "test_callback"
        );
    }
}
