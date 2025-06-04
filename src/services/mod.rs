// src/services/mod.rs

// Core services organized by domain
pub mod core;

// Interface services for different platforms
pub mod interfaces {
    pub mod telegram {
        #[allow(clippy::module_inception)]
        pub mod telegram;
        pub mod telegram_keyboard;

        pub use telegram::TelegramService;
        pub use telegram_keyboard::{InlineKeyboard, InlineKeyboardButton};
    }

    pub mod api {
        // TODO: Implement REST API interface modules
        // - api_service.rs: Core API service for HTTP endpoints
        // - api_middleware.rs: Authentication and rate limiting middleware
        // - api_routes.rs: Route definitions and handlers
    }

    pub mod discord {
        // TODO: Implement Discord bot interface modules
        // - discord_service.rs: Core Discord bot service
        // - discord_commands.rs: Discord slash commands and interactions
        // - discord_embeds.rs: Rich embed formatting for Discord messages
    }
}

// Re-export commonly used services for backward compatibility
pub use core::ai::{AiBetaIntegrationService, AiIntegrationService, AiIntelligenceService};
pub use core::analysis::{
    CorrelationAnalysisService, MarketAnalysisService, TechnicalAnalysisService,
};
pub use core::infrastructure::CacheManager;
pub use core::infrastructure::{
    // Legacy services still in use
    AnalyticsEngineService,
    DataAccessLayer,
    DataIngestionModule,
    DatabaseManager,
    FinancialModule,

    GlobalRateLimiterDO,
    MarketDataCoordinatorDO,
    MonitoringModule,
    // New modular infrastructure components
    NotificationModule,
    OpportunityCoordinatorDO,
    ServiceContainer,
    UserOpportunityQueueDO,
};
pub use core::invitation::{AffiliationService, InvitationService, ReferralService};
pub use core::opportunities::{
    AIEnhancer,
    AccessManager,
    MarketAnalyzer,
    OpportunityBuilder,
    OpportunityCategorizationService,
    OpportunityConfig,
    OpportunityContext,
    OpportunityDistributionService,
    // New modular components
    OpportunityEngine,
};
pub use core::trading::{AiExchangeRouterService, ExchangeService, PositionsService};
pub use core::user::{
    DynamicConfigService, SessionManagementService, UserProfileService,
    UserTradingPreferencesService,
};
pub use interfaces::telegram::{InlineKeyboard, InlineKeyboardButton, TelegramService};
