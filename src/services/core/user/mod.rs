// src/services/core/user/mod.rs

pub mod user_profile;
pub mod user_trading_preferences;
pub mod dynamic_config;
pub mod user_access;
pub mod ai_access;
pub mod user_exchange_api;

pub use user_profile::UserProfileService;
pub use user_trading_preferences::UserTradingPreferencesService;
pub use dynamic_config::DynamicConfigService;
pub use user_access::UserAccessService;
pub use ai_access::AIAccessService;
pub use user_exchange_api::{
    UserExchangeApiService, ApiKeyValidationResult, ExchangeCompatibilityResult,
    AddApiKeyRequest, UpdateApiKeyRequest, RateLimitInfo,
}; 