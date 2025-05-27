// src/services/mod.rs

// Core services organized by domain
pub mod core {
    pub mod user {
        pub mod dynamic_config;
        pub mod session_management;
        pub mod user_access;
        pub mod user_profile;
        pub mod user_trading_preferences;

        pub use dynamic_config::DynamicConfigService;
        pub use session_management::SessionManagementService;
        pub use user_profile::UserProfileService;
        pub use user_trading_preferences::UserTradingPreferencesService;
    }

    pub mod trading {
        pub mod ai_exchange_router;
        pub mod exchange;
        pub mod exchange_availability;
        pub mod positions;

        pub use ai_exchange_router::AiExchangeRouterService;
        pub use exchange::ExchangeService;
        pub use exchange_availability::ExchangeAvailabilityService;
        pub use positions::PositionsService;
    }

    pub mod opportunities {
        pub mod global_opportunity;
        pub mod opportunity;
        pub mod opportunity_categorization;
        pub mod opportunity_distribution;
        pub mod opportunity_enhanced;
        pub mod technical_trading;

        pub use global_opportunity::GlobalOpportunityService;
        pub use opportunity::OpportunityService;
        pub use opportunity_categorization::OpportunityCategorizationService;
        pub use opportunity_distribution::{
            DistributionConfig, DistributionStats, OpportunityDistributionService,
        };
        pub use opportunity_enhanced::EnhancedOpportunityService;
        pub use technical_trading::TechnicalTradingService;
    }

    pub mod analysis {
        pub mod correlation_analysis;
        pub mod market_analysis;
        pub mod technical_analysis;

        pub use correlation_analysis::CorrelationAnalysisService;
        pub use market_analysis::MarketAnalysisService;
        pub use technical_analysis::TechnicalAnalysisService;
    }

    pub mod ai {
        pub mod ai_beta_integration;
        pub mod ai_integration;
        pub mod ai_intelligence;

        pub use ai_beta_integration::AiBetaIntegrationService;
        pub use ai_integration::AiIntegrationService;
        pub use ai_intelligence::AiIntelligenceService;
    }

    // TODO: Fix invitation services - multiple compilation errors need to be resolved:
    // 1. Missing ArbitrageError::unauthorized method
    // 2. Missing D1Service::query and D1Service::execute methods
    // 3. Type mismatches with unwrap_or_default() usage
    // 4. Missing .await on async calls
    // pub mod invitation {
    //     pub mod affiliation_service;
    //     pub mod invitation_service;
    //     pub mod referral_service;

    //     pub use affiliation_service::AffiliationService;
    //     pub use invitation_service::InvitationService;
    //     pub use referral_service::ReferralService;
    // }

    pub mod infrastructure {
        pub mod ai_gateway;
        pub mod analytics_engine;
        pub mod cloudflare_pipelines;
        pub mod cloudflare_queues;
        pub mod d1_database;
        pub mod durable_objects;
        pub mod fund_monitoring;
        pub mod hybrid_data_access;
        pub mod kv_service;
        pub mod monitoring_observability;
        pub mod notifications;
        pub mod service_container;
        pub mod vectorize_service;

        pub use ai_gateway::{
            AIGatewayConfig, AIGatewayService, AIModelConfig, AIRequest, AIResponse,
            ModelRequirements, RoutingDecision,
        };
        pub use analytics_engine::{
            AnalyticsEngineConfig, AnalyticsEngineService, RealTimeMetrics, UserAnalytics,
        };
        pub use cloudflare_pipelines::CloudflarePipelinesService;
        pub use cloudflare_queues::{
            CloudflareQueuesConfig, CloudflareQueuesService, DistributionStrategy, MessagePriority,
        };
        pub use d1_database::D1Service;
        pub use durable_objects::{
            GlobalRateLimiterDO, MarketDataCoordinatorDO, OpportunityCoordinatorDO,
            UserOpportunityQueueDO,
        };
        pub use fund_monitoring::FundMonitoringService;
        pub use hybrid_data_access::{
            HybridDataAccessConfig, HybridDataAccessService, MarketDataSnapshot,
            SuperAdminApiConfig,
        };
        pub use kv_service::KVService;
        pub use monitoring_observability::MonitoringObservabilityService;
        pub use notifications::NotificationService;
        pub use service_container::{ServiceContainer, ServiceHealthStatus};
        pub use vectorize_service::{
            OpportunityEmbedding, RankedOpportunity, SimilarityResult, UserPreferenceVector,
            VectorizeConfig, VectorizeService,
        };
    }
}

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
pub use core::infrastructure::{
    AIGatewayService, AnalyticsEngineService, CloudflarePipelinesService, CloudflareQueuesService,
    D1Service, DistributionStrategy, FundMonitoringService, GlobalRateLimiterDO,
    HybridDataAccessService, KVService, MarketDataCoordinatorDO, MessagePriority,
    MonitoringObservabilityService, NotificationService, OpportunityCoordinatorDO,
    ServiceContainer, UserOpportunityQueueDO, VectorizeService,
};
// TODO: Re-enable when invitation services compilation errors are fixed
// pub use core::invitation::{AffiliationService, InvitationService, ReferralService};
pub use core::opportunities::{
    EnhancedOpportunityService, GlobalOpportunityService, OpportunityCategorizationService,
    OpportunityDistributionService, OpportunityService, TechnicalTradingService,
};
pub use core::trading::{
    AiExchangeRouterService, ExchangeAvailabilityService, ExchangeService, PositionsService,
};
pub use core::user::{
    DynamicConfigService, SessionManagementService, UserProfileService,
    UserTradingPreferencesService,
};
pub use interfaces::telegram::{InlineKeyboard, InlineKeyboardButton, TelegramService};
