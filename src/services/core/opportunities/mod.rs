// src/services/core/opportunities/mod.rs

pub mod opportunity;
pub mod global_opportunity;
pub mod personal_opportunity;
pub mod group_opportunity;
pub mod opportunity_enhanced;
pub mod technical_trading;
pub mod opportunity_categorization;

pub use opportunity::OpportunityService;
pub use global_opportunity::GlobalOpportunityService;
pub use personal_opportunity::PersonalOpportunityService;
pub use group_opportunity::GroupOpportunityService;
pub use opportunity_enhanced::EnhancedOpportunityService;
pub use technical_trading::TechnicalTradingService;
pub use opportunity_categorization::OpportunityCategorizationService; 