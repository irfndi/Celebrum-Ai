//! Enhanced KV Cache System
//!
//! Multi-tier hierarchical caching with compression, warming, and metadata tracking
//! Provides production-ready caching infrastructure for ArbEdge platform

// NEW: Unified data access service (replaces complex multi-file structure)
pub mod unified_data_access;

// Original data access layer modules (to be deprecated)
pub mod api_connector;
pub mod cache_layer;
pub mod data_coordinator;
pub mod data_source_manager;
pub mod data_validator;

// Enhanced KV cache modules
pub mod cache_manager;
pub mod compression;
pub mod config;
pub mod metadata;
pub mod warming;

// Simple data access module
pub mod simple_data_access;

// Re-export main components for easy access

// NEW: Unified data access components (recommended for new code)
pub use unified_data_access::{
    DataAccessResult, DataSource, UnifiedDataAccessBuilder, UnifiedDataAccessConfig,
    UnifiedDataAccessMetrics, UnifiedDataAccessService,
};

// Original data access layer components (legacy, will be deprecated)
pub use api_connector::{APIConnector, APIConnectorConfig};
pub use cache_layer::{CacheLayer, CacheLayerConfig};
pub use data_coordinator::{DataCoordinator as DataAccessDataCoordinator, DataCoordinatorConfig};
pub use data_source_manager::{DataSourceManager, DataSourceManagerConfig};
pub use data_validator::{DataValidator, DataValidatorConfig};

// Simple data access components
pub use simple_data_access::{
    DataType as SimpleDataType, SimpleDataAccessConfig, SimpleDataAccessService, SimpleDataRequest,
    SimpleDataResponse,
};

// Enhanced KV cache components
pub use cache_manager::{
    BatchOperation, BatchResult, CacheManagerMetrics, CompressionStats, KvCacheManager, TierStats,
    WarmingStats,
};
pub use compression::{
    CompressionEngine, CompressionEnvelope, CompressionMetrics, CompressionMiddleware,
    CompressionResult, ContentAnalyzer, ContentType, StorageData,
};
pub use config::{
    CacheConfig as EnhancedCacheConfig, CleanupConfig, CompressionConfig, GeneralConfig,
    TierConfig, WarmingConfig,
};
pub use metadata::{
    AccessPattern, AlertSeverity, CacheAnalyticsReport, CacheMetadata, CleanupCandidate,
    CleanupRecommendations, DataType, HotPathMetrics, MetadataTracker, PerformanceAlert,
    PerformanceAnalysis, Priority, TierInsights, TopEntriesAnalysis, TrendAnalysis,
};
pub use warming::{CacheWarmingService, WarmingRequest, WarmingStats as WarmingServiceStats};

use crate::utils::ArbitrageResult;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unified Data Access Layer Configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataAccessLayerConfig {
    pub api_connector_config: APIConnectorConfig,
    pub cache_layer_config: CacheLayerConfig,
    pub data_coordinator_config: DataCoordinatorConfig,
    pub data_source_manager_config: DataSourceManagerConfig,
    pub data_validator_config: DataValidatorConfig,
    pub simple_data_access_config: SimpleDataAccessConfig,
    pub enhanced_cache_config: EnhancedCacheConfig,
}

impl DataAccessLayerConfig {
    pub fn high_concurrency() -> Self {
        Self::default() // Simple implementation for now
    }

    pub fn high_reliability() -> Self {
        Self::default() // Simple implementation for now
    }
}

/// Unified Data Access Layer that wraps all data access components
#[derive(Clone)]
pub struct DataAccessLayer {
    config: DataAccessLayerConfig,
    kv_store: worker::kv::KvStore,
}

impl DataAccessLayer {
    pub async fn new(
        config: DataAccessLayerConfig,
        kv_store: worker::kv::KvStore,
    ) -> ArbitrageResult<Self> {
        Ok(Self { config, kv_store })
    }

    pub fn config(&self) -> &DataAccessLayerConfig {
        &self.config
    }

    pub fn get_kv_store(&self) -> &worker::kv::KvStore {
        &self.kv_store
    }

    // Access methods for compatibility
    pub async fn api_connector(&self) -> ArbitrageResult<APIConnector> {
        APIConnector::new(self.config.api_connector_config.clone())
    }

    pub async fn cache_layer(&self) -> ArbitrageResult<CacheLayer> {
        CacheLayer::new(self.config.cache_layer_config.clone())
    }

    pub async fn data_coordinator(&self) -> ArbitrageResult<DataAccessDataCoordinator> {
        DataAccessDataCoordinator::new(
            self.config.data_coordinator_config.clone(),
            self.config.data_source_manager_config.clone(),
            self.config.cache_layer_config.clone(),
            self.config.api_connector_config.clone(),
            self.config.data_validator_config.clone(),
            self.kv_store.clone(),
        )
        .await
    }

    pub async fn data_source_manager(&self) -> ArbitrageResult<DataSourceManager> {
        DataSourceManager::new(self.config.data_source_manager_config.clone())
    }

    pub async fn data_validator(&self) -> ArbitrageResult<DataValidator> {
        DataValidator::new(self.config.data_validator_config.clone())
    }

    pub async fn simple_data_access(&self) -> ArbitrageResult<SimpleDataAccessService> {
        SimpleDataAccessService::new(
            self.config.simple_data_access_config.clone(),
            self.kv_store.clone(),
        )
    }

    pub async fn kv_cache_manager(&self) -> ArbitrageResult<KvCacheManager> {
        KvCacheManager::new(self.config.enhanced_cache_config.clone())
    }
}

/// Cache tier definitions with specific TTL and access patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheTier {
    /// Hot tier: Real-time data (30s-5min TTL)
    Hot,
    /// Warm tier: Recent data (5min-1hr TTL)
    Warm,
    /// Cold tier: Historical data (1hr-7days TTL)
    Cold,
}

impl CacheTier {
    /// Get default TTL for this tier
    pub fn default_ttl(&self) -> Duration {
        match self {
            CacheTier::Hot => Duration::from_secs(300),   // 5 minutes
            CacheTier::Warm => Duration::from_secs(3600), // 1 hour
            CacheTier::Cold => Duration::from_secs(86400), // 24 hours
        }
    }

    /// Get priority level for cache operations
    pub fn priority(&self) -> u8 {
        match self {
            CacheTier::Hot => 100,
            CacheTier::Warm => 50,
            CacheTier::Cold => 10,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CacheTier::Hot => "hot",
            CacheTier::Warm => "warm",
            CacheTier::Cold => "cold",
        }
    }

    pub fn should_promote(&self, access_count: u32, last_access_age: Duration) -> bool {
        match self {
            CacheTier::Cold => access_count > 5 && last_access_age < Duration::from_secs(300),
            CacheTier::Warm => access_count > 10 && last_access_age < Duration::from_secs(60),
            CacheTier::Hot => false, // Already at highest tier
        }
    }
}

/// Cache entry with metadata for multi-tier system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String, // Changed to String for compatibility with cache_manager
    pub tier: CacheTier,
    pub data_type: DataType,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_accessed: u64,
    pub access_count: u64, // Changed to u64 for compatibility
    pub size_bytes: u64,
    pub compressed: bool,
    pub ttl_seconds: u64,
}

impl CacheEntry {
    /// Check if this cache entry has expired
    pub fn is_expired(&self) -> bool {
        let now = crate::utils::time::get_current_timestamp();
        now > self.expires_at
    }

    /// Record access to this cache entry
    pub fn record_access(&mut self) {
        self.last_accessed = crate::utils::time::get_current_timestamp();
        self.access_count += 1;
    }

    /// Get age of cache entry in milliseconds
    pub fn age_ms(&self) -> u64 {
        let now = crate::utils::time::get_current_timestamp();
        now.saturating_sub(self.created_at)
    }

    /// Get time since last access in milliseconds
    pub fn last_access_age_ms(&self) -> u64 {
        let now = crate::utils::time::get_current_timestamp();
        now.saturating_sub(self.last_accessed)
    }
}

/// Cache operations for tracking and optimization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheOperation {
    Get,
    Put,
    Delete,
    Promote,
    Demote,
    Compress,
    Warm,
    BatchGet,
    BatchPut,
}

/// Comprehensive cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCacheStats {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub promotion_count: u64,
    pub compression_count: u64,
    pub warming_count: u64,
    pub total_size_bytes: u64,
    pub hot_tier_entries: u64,
    pub warm_tier_entries: u64,
    pub cold_tier_entries: u64,
    pub average_compression_ratio: f64,
    pub average_response_time_ms: f64,
    pub last_updated: u64,
}

impl EnhancedCacheStats {
    /// Calculate cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total_requests = self.hit_count + self.miss_count;
        if total_requests == 0 {
            0.0
        } else {
            self.hit_count as f64 / total_requests as f64
        }
    }

    /// Record a cache operation for statistics
    pub fn record_operation(
        &mut self,
        operation_type: &CacheOperation,
        success: bool,
        latency_ms: f64,
    ) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        match operation_type {
            CacheOperation::Get => {
                if success {
                    self.hit_count += 1;
                } else {
                    self.miss_count += 1;
                }
            }
            CacheOperation::Promote => {
                if success {
                    self.promotion_count += 1;
                }
            }
            CacheOperation::Compress => {
                if success {
                    self.compression_count += 1;
                }
            }
            CacheOperation::Warm => {
                if success {
                    self.warming_count += 1;
                }
            }
            _ => {}
        }

        // Update average response time
        let total_ops = self.total_operations as f64;
        self.average_response_time_ms =
            ((self.average_response_time_ms * (total_ops - 1.0)) + latency_ms) / total_ops;

        self.last_updated = crate::utils::time::get_current_timestamp();
    }
}

impl Default for EnhancedCacheStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            hit_count: 0,
            miss_count: 0,
            promotion_count: 0,
            compression_count: 0,
            warming_count: 0,
            total_size_bytes: 0,
            hot_tier_entries: 0,
            warm_tier_entries: 0,
            cold_tier_entries: 0,
            average_compression_ratio: 0.0,
            average_response_time_ms: 0.0,
            last_updated: crate::utils::time::get_current_timestamp(),
        }
    }
}

/// Type alias for backward compatibility
pub type CacheStats = EnhancedCacheStats;
