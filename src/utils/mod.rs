// src/utils/mod.rs

pub mod calculations;
pub mod core_architecture;
pub mod error;
pub mod formatter;
pub mod helpers;
pub mod kv_standards;
pub mod logger;

// Re-export commonly used items
pub use core_architecture::{
    CoreServiceArchitecture, HealthCheckResult, HealthCheckable, ServiceConfig, ServiceDependency,
    ServiceInfo, ServiceLifecycle, ServiceRegistryEntry, ServiceStatus, ServiceType,
    SystemHealthOverview,
};
pub use error::{ArbitrageError, ArbitrageResult};
