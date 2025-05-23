// src/services/mod.rs

pub mod exchange;
pub mod opportunity;
pub mod positions;
pub mod telegram;
pub mod user_profile;
pub mod global_opportunity;
pub mod ai_integration;
pub mod d1_database;
pub mod ai_exchange_router;
pub mod fund_monitoring;
pub mod dynamic_config;
pub mod notifications;
pub mod user_trading_preferences;
pub mod market_analysis;
pub mod opportunity_enhanced;

// Re-export main service structs
pub use exchange::{ExchangeInterface, ExchangeService};
pub use opportunity::{OpportunityService, OpportunityServiceConfig};
pub use positions::{PositionsService, CreatePositionData, UpdatePositionData};
pub use telegram::{TelegramService, TelegramConfig};
pub use user_profile::UserProfileService;
pub use global_opportunity::GlobalOpportunityService;
pub use ai_integration::{AiIntegrationService, AiIntegrationConfig, AiProvider, AiAnalysisRequest, AiAnalysisResponse};
pub use d1_database::D1Service;
pub use ai_exchange_router::{AiExchangeRouterService, AiExchangeRouterConfig, MarketDataSnapshot, AiOpportunityAnalysis};
pub use fund_monitoring::{FundMonitoringService, FundMonitoringConfig, ExchangeBalanceSnapshot, FundAllocation, BalanceHistoryEntry, FundOptimizationResult, BalanceAnalytics};
pub use dynamic_config::{DynamicConfigService, DynamicConfigTemplate, ConfigValidationResult, ConfigPreset};
pub use notifications::{NotificationService, NotificationTemplate, AlertTrigger, Notification, NotificationHistory, NotificationAnalytics, TriggerEvaluationContext};
pub use user_trading_preferences::*;
pub use market_analysis::*;
pub use opportunity_enhanced::*;
