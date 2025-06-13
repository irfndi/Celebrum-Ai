//! Enhanced KV Cache System
//!
//! Multi-tier hierarchical caching with compression, warming, and metadata tracking
//! Provides production-ready caching infrastructure for ArbEdge platform

pub mod cache_manager;
pub mod compression;
pub mod config;
pub mod metadata;
pub mod warming;

// Re-export main components for easy access
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

use serde::{Deserialize, Serialize};
use std::time::Duration;

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

    /// Get tier priority (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            CacheTier::Hot => 3,
            CacheTier::Warm => 2,
            CacheTier::Cold => 1,
        }
    }

    /// Get string representation for key building
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheTier::Hot => "hot",
            CacheTier::Warm => "warm",
            CacheTier::Cold => "cold",
        }
    }

    /// Check if data should be promoted to this tier based on access patterns
    pub fn should_promote(&self, access_count: u32, last_access_age: Duration) -> bool {
        match self {
            CacheTier::Hot => access_count >= 5 && last_access_age < Duration::from_secs(300),
            CacheTier::Warm => access_count >= 2 && last_access_age < Duration::from_secs(3600),
            CacheTier::Cold => true, // Always accept data
        }
    }
}

/// Cache entry with tier information and metadata (compatible with cache_manager)
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
    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now > self.expires_at
    }

    /// Record an access to this entry
    pub fn record_access(&mut self) {
        self.last_accessed = chrono::Utc::now().timestamp_millis() as u64;
        self.access_count += 1;
    }

    /// Get the age of this entry in milliseconds
    pub fn age_ms(&self) -> u64 {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now.saturating_sub(self.created_at)
    }

    /// Get time since last access in milliseconds
    pub fn last_access_age_ms(&self) -> u64 {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now.saturating_sub(self.last_accessed)
    }
}

/// Cache operation types used in enhanced cache system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Enhanced cache statistics compatible with cache_manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.hit_count as f64 / self.total_operations as f64
        }
    }

    /// Update stats with new operation
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
                self.promotion_count += 1;
            }
            CacheOperation::Compress => {
                self.compression_count += 1;
            }
            CacheOperation::Warm => {
                self.warming_count += 1;
            }
            _ => {}
        }

        // Update average response time
        self.average_response_time_ms = (self.average_response_time_ms + latency_ms) / 2.0;

        self.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }
}

// Type alias for convenience
pub type CacheStats = EnhancedCacheStats;
