// src/services/core/infrastructure/analytics_module/mod.rs

//! Analytics Module - Revolutionary Real-Time Analytics and Reporting System
//!
//! This module provides comprehensive analytics capabilities for the ArbEdge platform,
//! replacing the monolithic analytics_engine.rs with a modular architecture optimized
//! for high-concurrency trading operations (1000-2500 concurrent users).
//!
//! ## Modular Architecture (4 Components):
//!
//! 1. **DataProcessor** - Real-time data processing and stream analytics
//! 2. **ReportGenerator** - Automated report generation and export
//! 3. **MetricsAggregator** - Business metrics aggregation and KPI tracking
//! 4. **AnalyticsCoordinator** - Main orchestrator for all analytics operations
//!
//! ## Revolutionary Features:
//! - **Real-Time Processing**: Sub-second analytics with stream processing
//! - **Multi-Format Export**: PDF, CSV, JSON report generation
//! - **Business Intelligence**: KPI tracking and predictive analytics
//! - **Performance Optimization**: Batch processing and intelligent caching
//! - **Chaos Engineering**: Circuit breakers and fallback strategies

pub mod analytics_coordinator;
pub mod data_processor;
pub mod metrics_aggregator;
pub mod report_generator;

pub use analytics_coordinator::{
    AnalyticsCoordinator, AnalyticsCoordinatorConfig, AnalyticsCoordinatorHealth,
    AnalyticsCoordinatorMetrics, AnalyticsQuery, AnalyticsQueryResult, QueryMetadata,
    QueryPriority, TimeRange,
};
pub use data_processor::{
    DataProcessor, DataProcessorConfig, DataProcessorHealth, DataProcessorMetrics,
};
pub use metrics_aggregator::{
    MetricsAggregator, MetricsAggregatorConfig, MetricsAggregatorHealth, MetricsAggregatorMetrics,
};
pub use report_generator::{
    ReportGenerator, ReportGeneratorConfig, ReportGeneratorHealth, ReportGeneratorMetrics,
};

/// Analytics Module Configuration for High-Performance Analytics
#[derive(Debug, Clone)]
pub struct AnalyticsModuleConfig {
    // Core analytics settings
    pub enable_real_time_processing: bool,
    pub enable_automated_reporting: bool,
    pub enable_business_intelligence: bool,
    pub enable_predictive_analytics: bool,

    // Performance settings optimized for 1000-2500 concurrent users
    pub max_concurrent_queries: u32,
    pub batch_processing_size: usize,
    pub cache_ttl_seconds: u64,
    pub retention_days: u32,

    // Component configurations
    pub data_processor_config: DataProcessorConfig,
    pub report_generator_config: ReportGeneratorConfig,
    pub metrics_aggregator_config: MetricsAggregatorConfig,
    pub analytics_coordinator_config: AnalyticsCoordinatorConfig,
}

impl Default for AnalyticsModuleConfig {
    fn default() -> Self {
        Self {
            enable_real_time_processing: true,
            enable_automated_reporting: true,
            enable_business_intelligence: true,
            enable_predictive_analytics: true,
            max_concurrent_queries: 100,
            batch_processing_size: 200,
            cache_ttl_seconds: 300,
            retention_days: 90,
            data_processor_config: DataProcessorConfig::default(),
            report_generator_config: ReportGeneratorConfig::default(),
            metrics_aggregator_config: MetricsAggregatorConfig::default(),
            analytics_coordinator_config: AnalyticsCoordinatorConfig::default(),
        }
    }
}

impl AnalyticsModuleConfig {
    /// High-performance configuration for 1000-2500 concurrent users
    pub fn high_performance() -> Self {
        Self {
            enable_real_time_processing: true,
            enable_automated_reporting: true,
            enable_business_intelligence: true,
            enable_predictive_analytics: true,
            max_concurrent_queries: 200,
            batch_processing_size: 500,
            cache_ttl_seconds: 180,
            retention_days: 90,
            data_processor_config: DataProcessorConfig::high_performance(),
            report_generator_config: ReportGeneratorConfig::high_performance(),
            metrics_aggregator_config: MetricsAggregatorConfig::high_performance(),
            analytics_coordinator_config: AnalyticsCoordinatorConfig::high_performance(),
        }
    }

    /// High-reliability configuration for critical analytics
    pub fn high_reliability() -> Self {
        Self {
            enable_real_time_processing: false,
            enable_automated_reporting: true,
            enable_business_intelligence: true,
            enable_predictive_analytics: false,
            max_concurrent_queries: 50,
            batch_processing_size: 100,
            cache_ttl_seconds: 600,
            retention_days: 180,
            data_processor_config: DataProcessorConfig::high_reliability(),
            report_generator_config: ReportGeneratorConfig::high_reliability(),
            metrics_aggregator_config: MetricsAggregatorConfig::high_reliability(),
            analytics_coordinator_config: AnalyticsCoordinatorConfig::high_reliability(),
        }
    }
}

/// Analytics Module utility functions
pub mod utils {
    use super::*;

    /// Create high-performance analytics configuration
    pub fn create_high_performance_config() -> AnalyticsModuleConfig {
        AnalyticsModuleConfig::high_performance()
    }

    /// Create high-reliability analytics configuration
    pub fn create_high_reliability_config() -> AnalyticsModuleConfig {
        AnalyticsModuleConfig::high_reliability()
    }

    /// Create development analytics configuration
    pub fn create_development_config() -> AnalyticsModuleConfig {
        AnalyticsModuleConfig {
            max_concurrent_queries: 10,
            batch_processing_size: 50,
            retention_days: 30,
            enable_predictive_analytics: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_module_config_default() {
        let config = AnalyticsModuleConfig::default();
        assert!(config.enable_real_time_processing);
        assert!(config.enable_automated_reporting);
        assert_eq!(config.max_concurrent_queries, 100);
        assert_eq!(config.batch_processing_size, 200);
    }

    #[test]
    fn test_high_performance_config() {
        let config = AnalyticsModuleConfig::high_performance();
        assert_eq!(config.max_concurrent_queries, 200);
        assert_eq!(config.batch_processing_size, 500);
        assert_eq!(config.cache_ttl_seconds, 180);
    }

    #[test]
    fn test_high_reliability_config() {
        let config = AnalyticsModuleConfig::high_reliability();
        assert_eq!(config.max_concurrent_queries, 50);
        assert_eq!(config.retention_days, 365);
        assert!(!config.enable_predictive_analytics);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AnalyticsModuleConfig::default();
        assert!(config.validate().is_ok());

        config.max_concurrent_queries = 0;
        assert!(config.validate().is_err());
    }
}
