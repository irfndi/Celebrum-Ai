// src/services/core/infrastructure/mod.rs

//! Infrastructure Services Module
//! 
//! This module contains core infrastructure services that support the application's
//! operational requirements including monitoring, database access, notifications,
//! and fund management.
//! 
//! ## Services
//! - `CloudflarePipelinesService`: High-volume data ingestion via Cloudflare Pipelines and R2
//! - `D1Service`: Database operations and management
//! - `FundMonitoringService`: Portfolio and fund tracking
//! - `KVService`: Key-Value store operations
//! - `MonitoringObservabilityService`: System monitoring and observability
//! - `NotificationService`: User notification delivery

pub mod cloudflare_pipelines;
pub mod d1_database;
pub mod fund_monitoring;
pub mod kv_service;
pub mod monitoring_observability;
pub mod notifications;
pub mod service_container;

pub use cloudflare_pipelines::CloudflarePipelinesService;
pub use d1_database::D1Service;
pub use fund_monitoring::FundMonitoringService;
pub use kv_service::KVService;
pub use monitoring_observability::MonitoringObservabilityService;
pub use notifications::NotificationService;
pub use service_container::{SessionDistributionServiceContainer, ServiceHealthStatus}; 