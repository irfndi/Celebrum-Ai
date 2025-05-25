// src/services/core/infrastructure/mod.rs

//! Infrastructure Services Module
//! 
//! This module contains core infrastructure services that support the application's
//! operational requirements including monitoring, database access, notifications,
//! and fund management.
//! 
//! ## Services
//! - `MonitoringObservabilityService`: System monitoring and observability
//! - `D1Service`: Database operations and management
//! - `NotificationService`: User notification delivery
//! - `FundMonitoringService`: Portfolio and fund tracking

pub mod monitoring_observability;
pub mod d1_database;
pub mod notifications;
pub mod fund_monitoring;

pub use monitoring_observability::MonitoringObservabilityService;
pub use d1_database::D1Service;
pub use notifications::NotificationService;
pub use fund_monitoring::FundMonitoringService; 