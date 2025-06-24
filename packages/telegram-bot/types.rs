//! Types module for the Telegram bot package
//!
//! This module defines types used within the telegram-bot package.
//! These are simplified versions of the main crate types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserAccessLevel {
    // Core simplified roles
    Free,       // Basic access with limited features
    Pro,        // Enhanced features with moderate limits
    Ultra,      // Premium features with high limits
    Admin,      // Administrative access to system management
    SuperAdmin, // Full system access and control

    // Legacy support (deprecated - will be migrated)
    Guest,
    Registered,
    Verified,
    Paid,
    Premium,
    BetaUser,
    FreeWithoutAPI,
    FreeWithAPI,
    SubscriptionWithAPI,
    Basic,
    User,
}

impl UserAccessLevel {
    pub fn can_trade(&self) -> bool {
        match self {
            UserAccessLevel::Free => false,
            UserAccessLevel::Pro | UserAccessLevel::Ultra | UserAccessLevel::Admin | UserAccessLevel::SuperAdmin => true,
            // Legacy support
            UserAccessLevel::Guest | UserAccessLevel::FreeWithoutAPI => false,
            _ => true, // Default to true for other legacy roles
        }
    }
}

/// Alias for UserAccessLevel for contexts where UserRole is more semantically appropriate.
pub type UserRole = UserAccessLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandPermission {
    ViewOpportunities,
    BasicTrading,
    BasicOpportunities,
    AdvancedAnalytics,
    ManualTrading,
    AutomatedTrading,
    AIEnhancedOpportunities,
    TechnicalAnalysis,
    PremiumFeatures,
    AdminAccess,
    SystemAdministration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Paid,
    Basic,
    Pro,
    Ultra,
    Premium,
    Enterprise,
    Beta,
    Admin,
    SuperAdmin,
}

pub type ArbitrageResult<T> = Result<T, TelegramError>;

#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Permission denied: {0}")]
    Permission(String),
}
