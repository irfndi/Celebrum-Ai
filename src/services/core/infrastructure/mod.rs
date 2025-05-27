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
//! - `VectorizeService`: AI-enhanced opportunity matching with Cloudflare Vectorize
//! - `AnalyticsEngineService`: Enhanced observability with Workers Analytics Engine
//! - `AIGatewayService`: Centralized AI model management and routing
//! - `CloudflareQueuesService`: Robust message queuing with retry logic

pub mod ai_gateway;
pub mod analytics_engine;
pub mod cloudflare_pipelines;
pub mod cloudflare_queues;
pub mod d1_database;
pub mod durable_objects;
pub mod fund_monitoring;
pub mod hybrid_data_access;
pub mod kv_service;
pub mod market_data_ingestion;
pub mod monitoring_observability;
pub mod notifications;
pub mod service_container;
pub mod vectorize_service;

pub use ai_gateway::{AIGatewayService, AIGatewayConfig, AIModelConfig, AIRequest, AIResponse, ModelRequirements, RoutingDecision};
pub use analytics_engine::{AnalyticsEngineService, AnalyticsEngineConfig, RealTimeMetrics, UserAnalytics};
pub use cloudflare_pipelines::CloudflarePipelinesService;
pub use cloudflare_queues::{CloudflareQueuesService, CloudflareQueuesConfig, MessagePriority, DistributionStrategy};
pub use d1_database::D1Service;
pub use fund_monitoring::FundMonitoringService;
pub use hybrid_data_access::{HybridDataAccessService, HybridDataAccessConfig, SuperAdminApiConfig, MarketDataSnapshot};
pub use kv_service::KVService;
pub use market_data_ingestion::{MarketDataIngestionService, MarketDataIngestionConfig, MarketDataSnapshot as IngestionSnapshot, IngestionStats};
pub use monitoring_observability::MonitoringObservabilityService;
pub use notifications::NotificationService;
pub use service_container::{SessionDistributionServiceContainer, ServiceHealthStatus};
pub use vectorize_service::{VectorizeService, VectorizeConfig, OpportunityEmbedding, UserPreferenceVector, SimilarityResult, RankedOpportunity};
pub use durable_objects::{OpportunityCoordinatorDO, UserOpportunityQueueDO, GlobalRateLimiterDO, MarketDataCoordinatorDO}; 