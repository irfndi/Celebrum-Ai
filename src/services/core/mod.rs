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
pub use infrastructure::*;
pub use invitation::*;
pub use market_data::*;
pub use opportunities::*;
pub use trading::*;
pub use user::*;

// Re-export admin services for easier access
pub use admin::{
    AdminService, AuditService, MonitoringService, SystemConfigService, UserManagementService,
};
