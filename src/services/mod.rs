// src/services/mod.rs

// Core services organized by domain
pub mod core;

// Interface services for different platforms
pub mod interfaces;

// Re-export commonly used services for backward compatibility
pub use core::ai::{AiBetaIntegrationService, AiIntegrationService, AiIntelligenceService};
pub use core::analysis::{
    CorrelationAnalysisService, MarketAnalysisService, TechnicalAnalysisService,
};
pub use core::infrastructure::CacheManager;
pub use core::infrastructure::{
    // All services now fully modularized
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
pub use interfaces::telegram::{ModularTelegramService, UserInfo, UserPermissions};
pub use interfaces::{InlineKeyboard, InlineKeyboardButton};
