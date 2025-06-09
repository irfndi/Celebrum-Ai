//! Storage Usage Analytics & Monitoring
//! 
//! Provides comprehensive storage analytics, usage tracking, growth predictions,
//! and real-time monitoring across all storage types (KV, D1, R2).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use worker::Env;

use crate::services::core::infrastructure::persistence::{ConnectionManager, StorageType, TransactionCoordinator};
use crate::utils::error::{ArbitrageError, ArbitrageResult, ErrorKind};

/// Storage usage metrics for a specific storage type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsageMetrics {
    /// Storage type
    pub storage_type: StorageType,
    /// Total storage size in bytes
    pub total_size_bytes: u64,
    /// Number of items stored
    pub item_count: u64,
    /// Average item size in bytes
    pub avg_item_size: f64,
    /// Total read operations
    pub read_operations: u64,
    /// Total write operations
    pub write_operations: u64,
    /// Total delete operations
    pub delete_operations: u64,
    /// Last access timestamp
    pub last_access: Option<DateTime<Utc>>,
    /// Growth rate (bytes per day)
    pub growth_rate_bytes_per_day: f64,
    /// Access frequency (operations per minute)
    pub access_frequency: f64,
    /// Collection timestamp
    pub collected_at: DateTime<Utc>,
}

/// Storage usage trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsageTrend {
    /// Storage type
    pub storage_type: StorageType,
    /// Time series data points
    pub data_points: VecDeque<StorageDataPoint>,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Growth velocity (bytes per day)
    pub growth_velocity: f64,
    /// Projected size in 30 days
    pub projected_size_30_days: u64,
    /// Capacity utilization percentage
    pub capacity_utilization: f64,
}

/// Individual data point in storage trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDataPoint {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// Storage size in bytes
    pub size_bytes: u64,
    /// Number of items
    pub item_count: u64,
    /// Operations per minute
    pub operations_per_minute: f64,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    /// Storage is growing
    Growing,
    /// Storage is shrinking
    Shrinking,
    /// Storage is stable
    Stable,
    /// Insufficient data
    Unknown,
}

/// Storage analytics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAnalyticsDashboard {
    /// Overall storage metrics
    pub overall_metrics: OverallStorageMetrics,
    /// Per-storage-type metrics
    pub storage_metrics: HashMap<StorageType, StorageUsageMetrics>,
    /// Usage trends
    pub usage_trends: HashMap<StorageType, StorageUsageTrend>,
    /// Top consumers by size
    pub top_consumers: Vec<TopConsumer>,
    /// Alerts and recommendations
    pub alerts: Vec<StorageAlert>,
    /// Dashboard last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Overall storage metrics across all types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStorageMetrics {
    /// Total storage across all types
    pub total_storage_bytes: u64,
    /// Total item count across all types
    pub total_item_count: u64,
    /// Total operations per day
    pub total_operations_per_day: u64,
    /// Cost estimate per month (in dollars)
    pub estimated_monthly_cost: f64,
    /// Growth rate (bytes per day)
    pub overall_growth_rate: f64,
}

/// Top storage consumer entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopConsumer {
    /// Consumer identifier
    pub identifier: String,
    /// Storage type
    pub storage_type: StorageType,
    /// Size in bytes
    pub size_bytes: u64,
    /// Percentage of total storage
    pub percentage_of_total: f64,
    /// Last access date
    pub last_access: Option<DateTime<Utc>>,
}

/// Storage alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAlert {
    /// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert type
    pub alert_type: AlertType,
    /// Storage type affected
    pub storage_type: StorageType,
    /// Alert message
    pub message: String,
    /// Alert threshold that was breached
    pub threshold: f64,
    /// Current value
    pub current_value: f64,
    /// Alert created timestamp
    pub created_at: DateTime<Utc>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Critical alert
    Critical,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    /// High storage usage
    HighUsage,
    /// Rapid growth detected
    RapidGrowth,
    /// Stale data detected
    StaleData,
    /// Unusual access pattern
    UnusualAccess,
    /// Cost threshold exceeded
    CostThreshold,
}

/// Configuration for storage analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAnalyticsConfig {
    /// Analytics enabled status
    pub enabled: bool,
    /// Data collection interval in seconds
    pub collection_interval_seconds: u64,
    /// Dashboard refresh interval in seconds
    pub dashboard_refresh_seconds: u64,
    /// Historical data retention in days
    pub history_retention_days: u32,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Cost calculation settings
    pub cost_settings: CostSettings,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// High usage threshold (percentage)
    pub high_usage_threshold: f64,
    /// Rapid growth threshold (bytes per day)
    pub rapid_growth_threshold: u64,
    /// Stale data threshold (days)
    pub stale_data_threshold: u32,
    /// Cost threshold (dollars per month)
    pub monthly_cost_threshold: f64,
}

/// Cost calculation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSettings {
    /// KV storage cost per GB per month
    pub kv_cost_per_gb_month: f64,
    /// D1 storage cost per GB per month
    pub d1_cost_per_gb_month: f64,
    /// R2 storage cost per GB per month
    pub r2_cost_per_gb_month: f64,
    /// Operations cost per million operations
    pub operations_cost_per_million: f64,
}

impl Default for StorageAnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            collection_interval_seconds: 300, // 5 minutes
            dashboard_refresh_seconds: 60, // 1 minute
            history_retention_days: 90,
            alert_thresholds: AlertThresholds {
                high_usage_threshold: 80.0, // 80%
                rapid_growth_threshold: 1_000_000_000, // 1GB per day
                stale_data_threshold: 30, // 30 days
                monthly_cost_threshold: 100.0, // $100 per month
            },
            cost_settings: CostSettings {
                kv_cost_per_gb_month: 0.50,
                d1_cost_per_gb_month: 0.75,
                r2_cost_per_gb_month: 0.15,
                operations_cost_per_million: 1.0,
            },
        }
    }
}

/// Storage analytics service
#[derive(Debug)]
pub struct StorageAnalyticsService {
    config: StorageAnalyticsConfig,
    current_metrics: Arc<RwLock<HashMap<StorageType, StorageUsageMetrics>>>,
    usage_trends: Arc<RwLock<HashMap<StorageType, StorageUsageTrend>>>,
    dashboard_cache: Arc<RwLock<Option<StorageAnalyticsDashboard>>>,
    alerts: Arc<RwLock<Vec<StorageAlert>>>,
    connection_manager: Arc<ConnectionManager>,
    transaction_coordinator: Arc<TransactionCoordinator>,
    is_running: Arc<RwLock<bool>>,
    collection_history: Arc<Mutex<VecDeque<DateTime<Utc>>>>,
}

impl StorageAnalyticsService {
    /// Create a new storage analytics service
    pub async fn new(
        config: StorageAnalyticsConfig,
        connection_manager: Arc<ConnectionManager>,
        transaction_coordinator: Arc<TransactionCoordinator>,
    ) -> ArbitrageResult<Self> {
        Ok(Self {
            config,
            current_metrics: Arc::new(RwLock::new(HashMap::new())),
            usage_trends: Arc::new(RwLock::new(HashMap::new())),
            dashboard_cache: Arc::new(RwLock::new(None)),
            alerts: Arc::new(RwLock::new(Vec::new())),
            connection_manager,
            transaction_coordinator,
            is_running: Arc::new(RwLock::new(false)),
            collection_history: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// Start the analytics service
    pub async fn start(&self, env: &Env) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(ArbitrageError::new(
                ErrorKind::ValidationError,
                "Storage analytics service is already running".to_string(),
            ));
        }
        *is_running = true;
        drop(is_running);

        // Start data collection loop
        let service = self.clone();
        let env_clone = env.clone();
        tokio::spawn(async move {
            service.collection_loop(&env_clone).await;
        });

        Ok(())
    }

    /// Stop the analytics service
    pub async fn stop(&self) -> ArbitrageResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    /// Get current dashboard data
    pub async fn get_dashboard(&self) -> Option<StorageAnalyticsDashboard> {
        self.dashboard_cache.read().await.clone()
    }

    /// Get usage trends for a specific storage type
    pub async fn get_usage_trend(&self, storage_type: &StorageType) -> Option<StorageUsageTrend> {
        self.usage_trends.read().await.get(storage_type).cloned()
    }

    /// Get current alerts
    pub async fn get_alerts(&self) -> Vec<StorageAlert> {
        self.alerts.read().await.clone()
    }

    /// Get storage metrics for a specific type
    pub async fn get_storage_metrics(&self, storage_type: &StorageType) -> Option<StorageUsageMetrics> {
        self.current_metrics.read().await.get(storage_type).cloned()
    }

    /// Data collection loop
    async fn collection_loop(&self, env: &Env) {
        let mut interval = interval(Duration::from_secs(self.config.collection_interval_seconds));
        
        while *self.is_running.read().await {
            interval.tick().await;
            
            if let Err(e) = self.collect_storage_metrics(env).await {
                eprintln!("Storage metrics collection error: {:?}", e);
            }
        }
    }

    /// Collect storage metrics for all storage types
    async fn collect_storage_metrics(&self, _env: &Env) -> ArbitrageResult<()> {
        let now = Utc::now();
        let storage_types = vec![StorageType::KV, StorageType::D1, StorageType::R2];
        
        for storage_type in storage_types {
            let metrics = self.collect_storage_type_metrics(&storage_type, _env).await?;
            
            // Update current metrics
            {
                let mut current_metrics = self.current_metrics.write().await;
                current_metrics.insert(storage_type.clone(), metrics.clone());
            }
            
            // Update usage trends
            self.update_usage_trend(&storage_type, &metrics).await;
        }

        // Record collection timestamp
        {
            let mut history = self.collection_history.lock().await;
            history.push_back(now);
            
            // Limit history size
            let max_history = (self.config.history_retention_days * 24 * 60 / 
                (self.config.collection_interval_seconds / 60)) as usize;
            while history.len() > max_history {
                history.pop_front();
            }
        }

        Ok(())
    }

    /// Collect metrics for a specific storage type
    async fn collect_storage_type_metrics(
        &self,
        storage_type: &StorageType,
        _env: &Env,
    ) -> ArbitrageResult<StorageUsageMetrics> {
        let now = Utc::now();
        
        // Mock data for demonstration - in production this would query actual storage systems
        let (total_size, item_count, read_ops, write_ops, delete_ops) = match storage_type {
            StorageType::KV => (1_024_000, 1000, 5000, 1000, 100),
            StorageType::D1 => (10_240_000, 50000, 10000, 2000, 500),
            StorageType::R2 => (104_857_600, 100, 1000, 200, 10),
        };

        Ok(StorageUsageMetrics {
            storage_type: storage_type.clone(),
            total_size_bytes: total_size,
            item_count,
            avg_item_size: if item_count > 0 { total_size as f64 / item_count as f64 } else { 0.0 },
            read_operations: read_ops,
            write_operations: write_ops,
            delete_operations: delete_ops,
            last_access: Some(now),
            growth_rate_bytes_per_day: total_size as f64 * 0.01, // 1% growth per day
            access_frequency: (read_ops + write_ops + delete_ops) as f64 / 60.0,
            collected_at: now,
        })
    }

    /// Update usage trend for a storage type
    async fn update_usage_trend(&self, storage_type: &StorageType, metrics: &StorageUsageMetrics) {
        let mut trends = self.usage_trends.write().await;
        
        let trend = trends.entry(storage_type.clone()).or_insert_with(|| StorageUsageTrend {
            storage_type: storage_type.clone(),
            data_points: VecDeque::new(),
            trend_direction: TrendDirection::Unknown,
            growth_velocity: 0.0,
            projected_size_30_days: metrics.total_size_bytes,
            capacity_utilization: 0.0,
        });

        // Add new data point
        trend.data_points.push_back(StorageDataPoint {
            timestamp: metrics.collected_at,
            size_bytes: metrics.total_size_bytes,
            item_count: metrics.item_count,
            operations_per_minute: metrics.access_frequency,
        });

        // Keep only recent data points
        let max_points = (self.config.history_retention_days * 24 * 60 / 
            (self.config.collection_interval_seconds / 60)) as usize;
        while trend.data_points.len() > max_points {
            trend.data_points.pop_front();
        }

        // Calculate trends
        self.calculate_trend_metrics(trend);
    }

    /// Calculate trend metrics from data points
    fn calculate_trend_metrics(&self, trend: &mut StorageUsageTrend) {
        if trend.data_points.len() < 2 {
            trend.trend_direction = TrendDirection::Unknown;
            return;
        }

        let data_points: Vec<_> = trend.data_points.iter().collect();
        let first = data_points.first().unwrap();
        let last = data_points.last().unwrap();
        
        let time_diff = last.timestamp.signed_duration_since(first.timestamp).num_seconds() as f64;
        let size_diff = last.size_bytes as i64 - first.size_bytes as i64;
        
        if time_diff > 0.0 {
            // Calculate growth velocity (bytes per day)
            trend.growth_velocity = (size_diff as f64 / time_diff) * 86400.0;
            
            // Determine trend direction
            trend.trend_direction = if trend.growth_velocity > 1000.0 {
                TrendDirection::Growing
            } else if trend.growth_velocity < -1000.0 {
                TrendDirection::Shrinking
            } else {
                TrendDirection::Stable
            };
            
            // Project size in 30 days
            let projected_growth = trend.growth_velocity * 30.0;
            trend.projected_size_30_days = (last.size_bytes as f64 + projected_growth).max(0.0) as u64;
        }
    }

    /// Check if service is healthy
    pub async fn is_healthy(&self) -> ArbitrageResult<bool> {
        if !*self.is_running.read().await {
            return Ok(false);
        }

        // Check if recent data collection occurred
        let history = self.collection_history.lock().await;
        if let Some(last_collection) = history.back() {
            let elapsed = Utc::now().signed_duration_since(*last_collection);
            let max_interval = chrono::Duration::seconds(self.config.collection_interval_seconds as i64 * 2);
            if elapsed > max_interval {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Clone for StorageAnalyticsService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            current_metrics: Arc::clone(&self.current_metrics),
            usage_trends: Arc::clone(&self.usage_trends),
            dashboard_cache: Arc::clone(&self.dashboard_cache),
            alerts: Arc::clone(&self.alerts),
            connection_manager: Arc::clone(&self.connection_manager),
            transaction_coordinator: Arc::clone(&self.transaction_coordinator),
            is_running: Arc::clone(&self.is_running),
            collection_history: Arc::clone(&self.collection_history),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_analytics_service_creation() {
        let config = StorageAnalyticsConfig::default();
        let connection_manager = Arc::new(ConnectionManager::new().await.unwrap());
        let transaction_coordinator = Arc::new(TransactionCoordinator::new().await.unwrap());
        
        let service = StorageAnalyticsService::new(config, connection_manager, transaction_coordinator)
            .await
            .unwrap();

        assert!(!*service.is_running.read().await);
        assert!(service.get_dashboard().await.is_none());
    }

    #[tokio::test]
    async fn test_trend_calculation() {
        let config = StorageAnalyticsConfig::default();
        let connection_manager = Arc::new(ConnectionManager::new().await.unwrap());
        let transaction_coordinator = Arc::new(TransactionCoordinator::new().await.unwrap());
        
        let service = StorageAnalyticsService::new(config, connection_manager, transaction_coordinator)
            .await
            .unwrap();

        let mut trend = StorageUsageTrend {
            storage_type: StorageType::KV,
            data_points: VecDeque::new(),
            trend_direction: TrendDirection::Unknown,
            growth_velocity: 0.0,
            projected_size_30_days: 0,
            capacity_utilization: 0.0,
        };

        // Add test data points
        let now = Utc::now();
        trend.data_points.push_back(StorageDataPoint {
            timestamp: now - chrono::Duration::hours(1),
            size_bytes: 1000,
            item_count: 10,
            operations_per_minute: 5.0,
        });
        
        trend.data_points.push_back(StorageDataPoint {
            timestamp: now,
            size_bytes: 2000,
            item_count: 20,
            operations_per_minute: 10.0,
        });

        service.calculate_trend_metrics(&mut trend);

        assert_eq!(trend.trend_direction, TrendDirection::Growing);
        assert!(trend.growth_velocity > 0.0);
    }
}
