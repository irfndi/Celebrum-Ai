// src/services/mod.rs

pub mod ai_beta_integration;
pub mod ai_exchange_router;
pub mod ai_integration;
pub mod ai_intelligence;
pub mod core_architecture;
pub mod correlation_analysis;
pub mod d1_database;
pub mod dynamic_config;
pub mod exchange;
pub mod fund_monitoring;
pub mod global_opportunity;
pub mod market_analysis;
pub mod notifications;
pub mod opportunity;
pub mod opportunity_categorization;
pub mod opportunity_enhanced;
pub mod positions;
pub mod technical_analysis;
pub mod technical_trading;
pub mod telegram;
pub mod user_profile;
pub mod user_trading_preferences;

// Re-export commonly used services
pub use ai_beta_integration::AiBetaIntegrationService;
pub use ai_exchange_router::AiExchangeRouterService;
pub use ai_integration::AiIntegrationService;
pub use ai_intelligence::AiIntelligenceService;
pub use core_architecture::CoreServiceArchitecture;
pub use correlation_analysis::CorrelationAnalysisService;
pub use d1_database::D1Service;
pub use dynamic_config::DynamicConfigService;
pub use exchange::ExchangeService;
pub use fund_monitoring::FundMonitoringService;
pub use global_opportunity::GlobalOpportunityService;
pub use market_analysis::MarketAnalysisService;
pub use notifications::NotificationService;
pub use opportunity::OpportunityService;
pub use opportunity_categorization::OpportunityCategorizationService;
pub use opportunity_enhanced::EnhancedOpportunityService;
pub use positions::PositionsService;
pub use technical_analysis::TechnicalAnalysisService;
pub use technical_trading::TechnicalTradingService;
pub use telegram::TelegramService;
pub use user_profile::UserProfileService;
pub use user_trading_preferences::UserTradingPreferencesService;
