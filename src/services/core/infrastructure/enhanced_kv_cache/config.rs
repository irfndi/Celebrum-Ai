//! Cache Configuration
//!
//! Configuration management for the enhanced KV cache system

use super::{metadata::DataType, CacheTier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Comprehensive cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// General cache settings
    pub general: GeneralConfig,
    /// Tier-specific configurations
    pub tiers: TierConfig,
    /// Compression settings
    pub compression: CompressionConfig,
    /// Warming strategies
    pub warming: WarmingConfig,
    /// Cleanup policies
    pub cleanup: CleanupConfig,
    /// Performance targets
    pub performance: PerformanceConfig,
    /// Data type specific policies
    pub data_type_policies: HashMap<DataType, CachePolicy>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let mut data_type_policies = HashMap::new();

        // Configure policies for each data type
        data_type_policies.insert(
            DataType::MarketData,
            CachePolicy {
                default_tier: CacheTier::Hot,
                compression_threshold: 512,
                max_age: Duration::from_secs(300),
                promotion_threshold: 3,
                warming_enabled: true,
            },
        );

        data_type_policies.insert(
            DataType::UserProfile,
            CachePolicy {
                default_tier: CacheTier::Warm,
                compression_threshold: 1024,
                max_age: Duration::from_secs(3600),
                promotion_threshold: 2,
                warming_enabled: true,
            },
        );

        data_type_policies.insert(
            DataType::Opportunities,
            CachePolicy {
                default_tier: CacheTier::Hot,
                compression_threshold: 1024,
                max_age: Duration::from_secs(600),
                promotion_threshold: 2,
                warming_enabled: true,
            },
        );

        data_type_policies.insert(
            DataType::Session,
            CachePolicy {
                default_tier: CacheTier::Warm,
                compression_threshold: 2048,
                max_age: Duration::from_secs(1800),
                promotion_threshold: 1,
                warming_enabled: false,
            },
        );

        data_type_policies.insert(
            DataType::Configuration,
            CachePolicy {
                default_tier: CacheTier::Cold,
                compression_threshold: 4096,
                max_age: Duration::from_secs(86400),
                promotion_threshold: 5,
                warming_enabled: true,
            },
        );

        data_type_policies.insert(
            DataType::Analytics,
            CachePolicy {
                default_tier: CacheTier::Cold,
                compression_threshold: 2048,
                max_age: Duration::from_secs(3600),
                promotion_threshold: 1,
                warming_enabled: false,
            },
        );

        data_type_policies.insert(
            DataType::AiResponse,
            CachePolicy {
                default_tier: CacheTier::Warm,
                compression_threshold: 1024,
                max_age: Duration::from_secs(7200),
                promotion_threshold: 2,
                warming_enabled: true,
            },
        );

        data_type_policies.insert(
            DataType::Historical,
            CachePolicy {
                default_tier: CacheTier::Cold,
                compression_threshold: 512,
                max_age: Duration::from_secs(604800),
                promotion_threshold: 10,
                warming_enabled: false,
            },
        );

        data_type_policies.insert(
            DataType::Generic,
            CachePolicy {
                default_tier: CacheTier::Warm,
                compression_threshold: 1024,
                max_age: Duration::from_secs(3600),
                promotion_threshold: 2,
                warming_enabled: false,
            },
        );

        Self {
            general: GeneralConfig::default(),
            tiers: TierConfig::default(),
            compression: CompressionConfig::default(),
            warming: WarmingConfig::default(),
            cleanup: CleanupConfig::default(),
            performance: PerformanceConfig::default(),
            data_type_policies,
        }
    }
}

/// General cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Enable cache system
    pub enabled: bool,
    /// Default namespace for cache keys
    pub namespace: String,
    /// Maximum cache size in bytes (0 = unlimited)
    pub max_cache_size_bytes: usize,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Connection pool size for KV operations
    pub connection_pool_size: u32,
    /// Timeout for cache operations in milliseconds
    pub operation_timeout_ms: u64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            namespace: "arbedge_cache".to_string(),
            max_cache_size_bytes: 256 * 1024 * 1024, // 256MB
            metrics_enabled: true,
            debug_logging: false,
            connection_pool_size: 20,
            operation_timeout_ms: 5000,
        }
    }
}

/// Tier-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// Hot tier settings
    pub hot: TierSettings,
    /// Warm tier settings
    pub warm: TierSettings,
    /// Cold tier settings
    pub cold: TierSettings,
    /// Enable automatic tier promotion
    pub auto_promotion_enabled: bool,
    /// Enable automatic tier demotion
    pub auto_demotion_enabled: bool,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            hot: TierSettings {
                max_entries: 10000,
                max_size_bytes: 50 * 1024 * 1024, // 50MB
                default_ttl: Duration::from_secs(300),
                promotion_threshold: 5,
                access_frequency_threshold: 2.0,
            },
            warm: TierSettings {
                max_entries: 50000,
                max_size_bytes: 150 * 1024 * 1024, // 150MB
                default_ttl: Duration::from_secs(3600),
                promotion_threshold: 2,
                access_frequency_threshold: 0.5,
            },
            cold: TierSettings {
                max_entries: 100000,
                max_size_bytes: 500 * 1024 * 1024, // 500MB
                default_ttl: Duration::from_secs(86400),
                promotion_threshold: 1,
                access_frequency_threshold: 0.1,
            },
            auto_promotion_enabled: true,
            auto_demotion_enabled: true,
        }
    }
}

/// Settings for individual cache tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierSettings {
    /// Maximum number of entries in this tier
    pub max_entries: usize,
    /// Maximum size in bytes for this tier
    pub max_size_bytes: usize,
    /// Default TTL for entries in this tier
    pub default_ttl: Duration,
    /// Number of accesses required for promotion
    pub promotion_threshold: u32,
    /// Access frequency threshold (accesses per hour)
    pub access_frequency_threshold: f64,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Compression algorithm to use
    pub algorithm: CompressionAlgorithm,
    /// Minimum size threshold for compression
    pub size_threshold_bytes: usize,
    /// Compression level (1-9, higher = better compression but slower)
    pub compression_level: u32,
    /// Maximum compression ratio to accept (if worse, don't compress)
    pub max_compression_ratio: f64,
    /// Enable compression effectiveness tracking
    pub track_effectiveness: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Gzip,
            size_threshold_bytes: 1024, // 1KB
            compression_level: 6,       // Balanced
            max_compression_ratio: 0.9, // Only compress if at least 10% reduction
            track_effectiveness: true,
        }
    }
}

/// Available compression algorithms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// Gzip compression (good balance)
    Gzip,
    /// Brotli compression (better compression, slower)
    Brotli,
    /// LZ4 compression (faster, less compression)
    Lz4,
    /// No compression
    None,
}

impl CompressionAlgorithm {
    /// Get file extension for this algorithm
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Gzip => ".gz",
            CompressionAlgorithm::Brotli => ".br",
            CompressionAlgorithm::Lz4 => ".lz4",
            CompressionAlgorithm::None => "",
        }
    }

    /// Check if algorithm is available
    pub fn is_available(&self) -> bool {
        match self {
            CompressionAlgorithm::Gzip => true,    // Always available
            CompressionAlgorithm::Brotli => false, // Would need brotli crate
            CompressionAlgorithm::Lz4 => false,    // Would need lz4 crate
            CompressionAlgorithm::None => true,
        }
    }
}

/// Cache warming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingConfig {
    /// Enable cache warming
    pub enabled: bool,
    /// Enable predictive warming based on access patterns
    pub predictive_warming: bool,
    /// Enable scheduled warming
    pub scheduled_warming: bool,
    /// Warming batch size
    pub batch_size: usize,
    /// Maximum warming operations per minute
    pub max_warming_rate: u32,
    /// Warming priority queue size
    pub priority_queue_size: usize,
    /// Hours of access history to analyze for predictions
    pub prediction_history_hours: u32,
    /// Minimum access frequency to trigger warming
    pub min_access_frequency: f64,
}

impl Default for WarmingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            predictive_warming: true,
            scheduled_warming: false,
            batch_size: 10,
            max_warming_rate: 100,
            priority_queue_size: 1000,
            prediction_history_hours: 24,
            min_access_frequency: 0.5,
        }
    }
}

/// Cache cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Enable automatic cleanup
    pub enabled: bool,
    /// Cleanup interval in seconds
    pub cleanup_interval_seconds: u64,
    /// Space utilization threshold to trigger cleanup (0.0-1.0)
    pub space_threshold: f64,
    /// Age threshold for cleanup (entries older than this may be cleaned)
    pub max_age_seconds: u64,
    /// Maximum number of entries to clean per batch
    pub max_cleanup_batch_size: usize,
    /// Enable aggressive cleanup when space is low
    pub aggressive_cleanup: bool,
    /// Preserve frequently accessed items during cleanup
    pub preserve_frequent_access: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cleanup_interval_seconds: 300,  // 5 minutes
            space_threshold: 0.85,          // 85% full
            max_age_seconds: 7 * 24 * 3600, // 7 days
            max_cleanup_batch_size: 1000,
            aggressive_cleanup: true,
            preserve_frequent_access: true,
        }
    }
}

/// Performance targets and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target cache hit ratio (0.0-1.0)
    pub target_hit_ratio: f64,
    /// Target response time in milliseconds
    pub target_response_time_ms: u64,
    /// Maximum acceptable response time
    pub max_response_time_ms: u64,
    /// Enable performance monitoring
    pub monitoring_enabled: bool,
    /// Performance sample rate (0.0-1.0)
    pub sample_rate: f64,
    /// Enable alerting when performance degrades
    pub alerting_enabled: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_hit_ratio: 0.95,      // 95% hit ratio
            target_response_time_ms: 50, // 50ms target
            max_response_time_ms: 200,   // 200ms max acceptable
            monitoring_enabled: true,
            sample_rate: 0.1,        // Sample 10% of operations
            alerting_enabled: false, // Disable by default
        }
    }
}

/// Data type specific cache policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePolicy {
    /// Default tier for this data type
    pub default_tier: CacheTier,
    /// Size threshold for compression
    pub compression_threshold: usize,
    /// Maximum age before cleanup
    pub max_age: Duration,
    /// Access count threshold for promotion
    pub promotion_threshold: u32,
    /// Enable warming for this data type
    pub warming_enabled: bool,
}

impl CacheConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: CacheConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get policy for a specific data type
    pub fn get_policy(&self, data_type: &DataType) -> &CachePolicy {
        self.data_type_policies
            .get(data_type)
            .unwrap_or_else(|| self.data_type_policies.get(&DataType::Generic).unwrap())
    }

    /// Get tier settings for a specific tier
    pub fn get_tier_settings(&self, tier: CacheTier) -> &TierSettings {
        match tier {
            CacheTier::Hot => &self.tiers.hot,
            CacheTier::Warm => &self.tiers.warm,
            CacheTier::Cold => &self.tiers.cold,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check that tier sizes make sense
        if self.tiers.hot.max_size_bytes > self.general.max_cache_size_bytes {
            return Err("Hot tier size exceeds total cache size".to_string());
        }

        let total_tier_size = self.tiers.hot.max_size_bytes
            + self.tiers.warm.max_size_bytes
            + self.tiers.cold.max_size_bytes;

        if total_tier_size > self.general.max_cache_size_bytes {
            return Err("Sum of tier sizes exceeds total cache size".to_string());
        }

        // Check compression settings
        if !self.compression.algorithm.is_available() {
            return Err(format!(
                "Compression algorithm {:?} is not available",
                self.compression.algorithm
            ));
        }

        // Check performance targets
        if self.performance.target_hit_ratio > 1.0 || self.performance.target_hit_ratio < 0.0 {
            return Err("Target hit ratio must be between 0.0 and 1.0".to_string());
        }

        if self.performance.target_response_time_ms > self.performance.max_response_time_ms {
            return Err("Target response time cannot exceed maximum response time".to_string());
        }

        Ok(())
    }

    /// Create configuration optimized for production
    pub fn production_optimized() -> Self {
        let mut config = Self::default();

        // Production optimizations
        config.general.max_cache_size_bytes = 1024 * 1024 * 1024; // 1GB
        config.general.connection_pool_size = 50;
        config.general.debug_logging = false;

        // More aggressive tiers for production
        config.tiers.hot.max_entries = 50000;
        config.tiers.warm.max_entries = 200000;
        config.tiers.cold.max_entries = 500000;

        // Better compression
        config.compression.compression_level = 8;
        config.compression.size_threshold_bytes = 512;

        // More frequent cleanup
        config.cleanup.cleanup_interval_seconds = 120; // 2 minutes
        config.cleanup.space_threshold = 0.80; // 80% full

        // Higher performance targets
        config.performance.target_hit_ratio = 0.98; // 98% hit ratio
        config.performance.target_response_time_ms = 25; // 25ms target

        config
    }

    /// Create configuration optimized for development
    pub fn development_optimized() -> Self {
        let mut config = Self::default();

        // Development optimizations
        config.general.max_cache_size_bytes = 128 * 1024 * 1024; // 128MB
        config.general.debug_logging = true;
        config.general.metrics_enabled = true;

        // Smaller tiers for development
        config.tiers.hot.max_entries = 1000;
        config.tiers.warm.max_entries = 5000;
        config.tiers.cold.max_entries = 10000;

        // Less aggressive compression
        config.compression.compression_level = 4;
        config.compression.size_threshold_bytes = 2048;

        // More frequent cleanup for testing
        config.cleanup.cleanup_interval_seconds = 60; // 1 minute

        // Relaxed performance targets
        config.performance.target_hit_ratio = 0.90; // 90% hit ratio
        config.performance.target_response_time_ms = 100; // 100ms target

        config
    }
}
