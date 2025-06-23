//! Utility functions for Telegram Bot

// Utility functions for Telegram operations

/// Log macro for console output
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log::info!($($t)*));
}

// Utility functions for Telegram operations can be added here as needed
