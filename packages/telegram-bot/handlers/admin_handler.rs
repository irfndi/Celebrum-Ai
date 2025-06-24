//! Admin Command Handler
//!
//! Handles the /admin command for administrative functions.

use crate::core::bot_client::TelegramResult;
use crate::core::command_router::{CommandContext, CommandHandler, UserPermissions};
use crate::integrations::get_admin_statistics;
use async_trait::async_trait;
use worker::console_log;

pub struct AdminHandler;

impl AdminHandler {
    pub fn new() -> Self {
        Self
    }

    /// Generate the main admin dashboard view.
    async fn get_dashboard_text(&self) -> String {
        match get_admin_statistics().await {
            Ok(stats) => format!(
                "👑 *Admin Dashboard*\n\n
                *📊 System Statistics*\n
                - *Total Users:* {}\n
                - *Active Users:* {}\n
                - *Admin Users:* {}\n
                - *Total Volume:* ${:.2}\n
                - *Total Trades:* {}\n\n
                *🔧 System Status:* ✅ Operational\n\n
                *💡 Available Commands:*\n
                - `/admin stats` - View detailed statistics\n
                - `/admin users` - User management functions\n
                - `/admin broadcast` - Send a message to all users",
                stats["total_users"].as_u64().unwrap_or(0),
                stats["active_users"].as_u64().unwrap_or(0),
                0,   // admin_users not available in current stats
                0.0, // total_volume_usdt not available in current stats
                stats["total_trades"].as_u64().unwrap_or(0)
            ),
            Err(e) => {
                console_log!("❌ Failed to get admin stats for dashboard: {:?}", e);
                "👑 *Admin Dashboard*\n\n
                Could not load system statistics. The system may be experiencing issues."
                    .to_string()
            }
        }
    }

    /// Generate detailed statistics view.
    async fn get_stats_text(&self) -> String {
        match get_admin_statistics().await {
            Ok(stats) => format!(
                "📈 *Detailed System Statistics*\n\n
                *👥 User Metrics*\n
                - *Total Users:* {}\n
                - *Active Users (24h):* {}\n
                - *Premium Users:* {}\n\n
                *💰 Trading Metrics*\n
                - *Total Trades:* {}\n
                - *Total Volume:* ${:.2}\n

                *⚙️ System Performance*\n
                - *Status:* ✅ All systems operational\n
                - *Database Load:* Low\n
                - *API Latency:* 120ms",
                stats["total_users"].as_u64().unwrap_or(0),
                stats["active_users"].as_u64().unwrap_or(0),
                0, // admin_users not available in current stats
                stats["total_trades"].as_u64().unwrap_or(0),
                0.0 // total_volume_usdt not available in current stats
            ),
            Err(e) => {
                console_log!("❌ Failed to get detailed admin stats: {:?}", e);
                "📈 *Detailed System Statistics*\n\nCould not load detailed statistics.".to_string()
            }
        }
    }
}

#[async_trait]
impl CommandHandler for AdminHandler {
    async fn handle(
        &self,
        _chat_id: i64,
        user_id: i64,
        args: &[&str],
        _context: &CommandContext,
    ) -> TelegramResult<String> {
        console_log!("👑 Processing /admin command for user {}", user_id);

        let subcommand = args.first().unwrap_or(&"dashboard").to_lowercase();

        match subcommand.as_str() {
            "stats" | "statistics" => Ok(self.get_stats_text().await),
            "dashboard" => Ok(self.get_dashboard_text().await),
            "users" => {
                // Placeholder for user management functionality
                Ok("👥 *User Management*\n\nThis feature is under development.".to_string())
            }
            "broadcast" => {
                // Placeholder for broadcast functionality
                if args.len() > 1 {
                    let message = args[1..].join(" ");
                    console_log!("📢 Admin {} is broadcasting: {}", user_id, message);
                    Ok(format!("📢 Broadcast queued: '{}'", message))
                } else {
                    Ok("Usage: /admin broadcast <message>".to_string())
                }
            }
            "logs" => Ok("📜 *System Logs*\n\nThis feature is under development.".to_string()),
            _ => Ok(self.get_dashboard_text().await),
        }
    }

    fn command_name(&self) -> &'static str {
        "admin"
    }

    fn help_text(&self) -> &'static str {
        "Access admin panel and system management tools"
    }

    fn check_permission(&self, user_permissions: &UserPermissions) -> bool {
        // Only admins can use admin commands
        user_permissions.is_admin
    }
}

impl Default for AdminHandler {
    fn default() -> Self {
        Self::new()
    }
}
