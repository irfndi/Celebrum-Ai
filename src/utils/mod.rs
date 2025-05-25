// src/utils/mod.rs

pub mod calculations;
pub mod core_architecture;
pub mod error;
pub mod formatter;
pub mod helpers;
pub mod kv_standards;
pub mod logger;

// Re-export commonly used items
pub use error::{ArbitrageError, ArbitrageResult};
pub use core_architecture::{ServiceStatus, ServiceType, HealthCheckResult, HealthCheckable, ServiceLifecycle, ServiceDependency, ServiceConfig, ServiceRegistryEntry, CoreServiceArchitecture, SystemHealthOverview, ServiceInfo};
