//! Shared Types for Legacy System Integration
//!
//! Common types, enums, and data structures used across all legacy system
//! integration components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Legacy system types supported by the integration framework
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegacySystemType {
    /// Legacy Telegram service
    TelegramService,
    /// Legacy opportunity engine
    OpportunityEngine,
    /// Legacy analytics engine
    AnalyticsEngine,
    /// Legacy trading services
    TradingServices,
    /// Legacy user services
    UserServices,
    /// Legacy market data services
    MarketDataServices,
    /// Legacy notification services
    NotificationServices,
    /// Legacy database services
    DatabaseServices,
    /// Custom legacy system
    Custom(String),
}

impl std::fmt::Display for LegacySystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LegacySystemType::TelegramService => write!(f, "telegram_service"),
            LegacySystemType::OpportunityEngine => write!(f, "opportunity_engine"),
            LegacySystemType::AnalyticsEngine => write!(f, "analytics_engine"),
            LegacySystemType::TradingServices => write!(f, "trading_services"),
            LegacySystemType::UserServices => write!(f, "user_services"),
            LegacySystemType::MarketDataServices => write!(f, "market_data_services"),
            LegacySystemType::NotificationServices => write!(f, "notification_services"),
            LegacySystemType::DatabaseServices => write!(f, "database_services"),
            LegacySystemType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// System identifier for migration operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemIdentifier {
    /// System type
    pub system_type: LegacySystemType,
    /// System instance ID
    pub instance_id: String,
    /// System version
    pub version: String,
}

impl SystemIdentifier {
    pub fn new(system_type: LegacySystemType, instance_id: String, version: String) -> Self {
        Self {
            system_type,
            instance_id,
            version,
        }
    }

    pub fn key(&self) -> String {
        format!("{}:{}:{}", self.system_type, self.instance_id, self.version)
    }
}

/// Migration system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationSystemHealth {
    /// System identifier
    pub system_id: SystemIdentifier,
    /// Health status
    pub status: HealthStatus,
    /// Health metrics
    pub metrics: HealthMetrics,
    /// Last health check timestamp
    pub last_check: u64,
    /// Health check interval in seconds
    pub check_interval_seconds: u64,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy and operational
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is unhealthy but not critical
    Unhealthy,
    /// System is in critical state
    Critical,
    /// System is unavailable
    Unavailable,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Critical => write!(f, "critical"),
            HealthStatus::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// Health metrics for system monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            response_time_ms: 0.0,
            error_rate: 0.0,
            success_rate: 1.0,
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            active_connections: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Migration metrics for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetrics {
    /// Migration operation ID
    pub operation_id: String,
    /// System identifier
    pub system_id: SystemIdentifier,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp (None if ongoing)
    pub end_time: Option<u64>,
    /// Duration in milliseconds (calculated or estimated)
    pub duration_ms: f64,
    /// Operation type
    pub operation_type: MigrationOperationType,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Migration operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationOperationType {
    /// Dual-write operation
    DualWrite,
    /// Read migration operation
    ReadMigration,
    /// Data validation operation
    DataValidation,
    /// Feature flag toggle
    FeatureFlagToggle,
    /// System adaptation
    SystemAdaptation,
    /// Rollback operation
    Rollback,
}

impl std::fmt::Display for MigrationOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationOperationType::DualWrite => write!(f, "dual_write"),
            MigrationOperationType::ReadMigration => write!(f, "read_migration"),
            MigrationOperationType::DataValidation => write!(f, "data_validation"),
            MigrationOperationType::FeatureFlagToggle => write!(f, "feature_flag_toggle"),
            MigrationOperationType::SystemAdaptation => write!(f, "system_adaptation"),
            MigrationOperationType::Rollback => write!(f, "rollback"),
        }
    }
}

/// Performance metrics for migration operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    /// 99th percentile latency
    pub p99_latency_ms: f64,
    /// Error count
    pub error_count: u64,
    /// Total operations processed
    pub total_operations: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU time in milliseconds
    pub cpu_time_ms: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            throughput_ops_per_sec: 0.0,
            latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            error_count: 0,
            total_operations: 0,
            memory_usage_bytes: 0,
            cpu_time_ms: 0.0,
        }
    }
}

/// Migration event for logging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEvent {
    /// Event ID
    pub event_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: MigrationEventType,
    /// System identifier
    pub system_id: SystemIdentifier,
    /// Event message
    pub message: String,
    /// Event severity
    pub severity: EventSeverity,
    /// Additional event data
    pub data: HashMap<String, serde_json::Value>,
}

/// Migration event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationEventType {
    /// Migration started
    MigrationStarted,
    /// Migration completed successfully
    MigrationCompleted,
    /// Migration failed
    MigrationFailed,
    /// Migration paused
    MigrationPaused,
    /// Migration resumed
    MigrationResumed,
    /// Rollback started
    RollbackStarted,
    /// Rollback completed
    RollbackCompleted,
    /// Validation passed
    ValidationPassed,
    /// Validation failed
    ValidationFailed,
    /// Feature flag toggled
    FeatureFlagToggled,
    /// System health changed
    HealthStatusChanged,
    /// Performance threshold breached
    PerformanceThresholdBreached,
    /// Data written to system
    DataWritten,
    /// Migration phase changed
    PhaseChanged,
}

/// Event severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical events
    Critical,
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Info => write!(f, "info"),
            EventSeverity::Warning => write!(f, "warning"),
            EventSeverity::Error => write!(f, "error"),
            EventSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Migration error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationError {
    /// Configuration error
    ConfigurationError(String),
    /// Validation error
    ValidationError(String),
    /// System unavailable error
    SystemUnavailableError(String),
    /// Timeout error
    TimeoutError(String),
    /// Data consistency error
    DataConsistencyError(String),
    /// Feature flag error
    FeatureFlagError(String),
    /// Circuit breaker error
    CircuitBreakerError(String),
    /// Unknown error
    UnknownError(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::ConfigurationError(msg) => write!(f, "Configuration Error: {}", msg),
            MigrationError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            MigrationError::SystemUnavailableError(msg) => write!(f, "System Unavailable: {}", msg),
            MigrationError::TimeoutError(msg) => write!(f, "Timeout Error: {}", msg),
            MigrationError::DataConsistencyError(msg) => {
                write!(f, "Data Consistency Error: {}", msg)
            }
            MigrationError::FeatureFlagError(msg) => write!(f, "Feature Flag Error: {}", msg),
            MigrationError::CircuitBreakerError(msg) => write!(f, "Circuit Breaker Error: {}", msg),
            MigrationError::UnknownError(msg) => write!(f, "Unknown Error: {}", msg),
        }
    }
}

impl std::error::Error for MigrationError {}

/// Conversion to ArbitrageError for compatibility
impl From<MigrationError> for crate::utils::error::ArbitrageError {
    fn from(error: MigrationError) -> Self {
        crate::utils::error::ArbitrageError::infrastructure_error(error.to_string())
    }
}
