// src/services/core/mod.rs

pub mod admin;
pub mod ai;
pub mod analysis;
pub mod auth;
pub mod infrastructure;
pub mod invitation;
pub mod market_data;
pub mod opportunities;
pub mod trading;
pub mod user;

// Re-export all services for convenience
pub use admin::*;
pub use ai::*;
pub use analysis::*;
pub use auth::*;
// Don't re-export infrastructure::* to avoid ServiceHealthStatus conflict with admin
pub use infrastructure::{
    AnalyticsEngineService, CacheManager, DatabaseManager, FinancialModule, InfrastructureEngine,
    MonitoringModule, NotificationModule, ServiceContainer,
}; // Only export specific items
pub use invitation::*;
pub use market_data::*;
pub use trading::*;
pub use user::*;

// Explicitly export items from the 'opportunities' module
pub use opportunities::access_manager::AccessManager;
pub use opportunities::ai_enhancer::AIEnhancer;
pub use opportunities::cache_manager::{CachePrefixes, OpportunityDataCache};
pub use opportunities::market_analyzer::MarketAnalyzer;
pub use opportunities::opportunity_builders::OpportunityBuilder;
pub use opportunities::opportunity_categorization::*;
pub use opportunities::opportunity_core::*;
pub use opportunities::opportunity_distribution::OpportunityDistributionService;
pub use opportunities::opportunity_engine::OpportunityEngine;

// Re-export admin services for easier access
pub use admin::{
    AdminService, AuditService, MonitoringService, SystemConfigService, UserManagementService,
};
