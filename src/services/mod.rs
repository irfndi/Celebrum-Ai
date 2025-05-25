// src/services/mod.rs

// Core services organized by domain
pub mod core {
    pub mod user {
    pub mod user_profile;
    pub mod user_access;
        pub mod user_trading_preferences;
        pub mod dynamic_config;
        
        pub use user_profile::UserProfileService;
        pub use user_trading_preferences::UserTradingPreferencesService;
        pub use dynamic_config::DynamicConfigService;
    }
    
    pub mod trading {
        pub mod exchange;
        pub mod exchange_availability;
        pub mod positions;
        pub mod ai_exchange_router;
        
        pub use exchange::ExchangeService;
        pub use exchange_availability::ExchangeAvailabilityService;
        pub use positions::PositionsService;
        pub use ai_exchange_router::AiExchangeRouterService;
    }
    
    pub mod opportunities {
        pub mod opportunity;
        pub mod global_opportunity;
        pub mod opportunity_enhanced;
        pub mod technical_trading;
        pub mod opportunity_categorization;
        
        pub use opportunity::OpportunityService;
        pub use global_opportunity::GlobalOpportunityService;
        pub use opportunity_enhanced::EnhancedOpportunityService;
        pub use technical_trading::TechnicalTradingService;
        pub use opportunity_categorization::OpportunityCategorizationService;
    }
    
    pub mod analysis {
        pub mod market_analysis;
        pub mod technical_analysis;
        pub mod correlation_analysis;
        
        pub use market_analysis::MarketAnalysisService;
        pub use technical_analysis::TechnicalAnalysisService;
        pub use correlation_analysis::CorrelationAnalysisService;
    }
    
    pub mod ai {
        pub mod ai_beta_integration;
        pub mod ai_intelligence;
        pub mod ai_integration;
        
        pub use ai_beta_integration::AiBetaIntegrationService;
        pub use ai_intelligence::AiIntelligenceService;
        pub use ai_integration::AiIntegrationService;
    }
    
    pub mod infrastructure {
        pub mod monitoring_observability;
        pub mod d1_database;
        pub mod notifications;
        pub mod fund_monitoring;
        
        pub use monitoring_observability::MonitoringObservabilityService;
        pub use d1_database::D1Service;
        pub use notifications::NotificationService;
        pub use fund_monitoring::FundMonitoringService;
    }
}

// Interface services for different platforms
pub mod interfaces {
    pub mod telegram {
        pub mod telegram;
        pub mod telegram_keyboard;
        
        pub use telegram::TelegramService;
        pub use telegram_keyboard::{InlineKeyboard, InlineKeyboardButton};
    }
    
    pub mod api {
        // Future API interface modules
    }
    
    pub mod discord {
        // Future Discord interface modules
    }
}

// Re-export commonly used services for backward compatibility
pub use core::user::{UserProfileService, UserTradingPreferencesService, DynamicConfigService};
pub use core::trading::{ExchangeService, ExchangeAvailabilityService, PositionsService, AiExchangeRouterService};
pub use core::opportunities::{OpportunityService, GlobalOpportunityService, EnhancedOpportunityService, TechnicalTradingService, OpportunityCategorizationService};
pub use core::analysis::{MarketAnalysisService, TechnicalAnalysisService, CorrelationAnalysisService};
pub use core::ai::{AiBetaIntegrationService, AiIntelligenceService, AiIntegrationService};
pub use core::infrastructure::{MonitoringObservabilityService, D1Service, NotificationService, FundMonitoringService};
pub use interfaces::telegram::{TelegramService, InlineKeyboard, InlineKeyboardButton};
