//! Read Migration Manager
//!
//! Intelligent read migration system that gradually shifts read operations from legacy
//! to new systems based on performance metrics, validation results, and configurable
//! rollout strategies with automatic failover capabilities.

use super::shared_types::{LegacySystemType, MigrationEvent, SystemIdentifier};
use crate::services::core::infrastructure::{
    legacy_system_integration::shared_types::PerformanceMetrics,
    shared_types::{CircuitBreaker, CircuitBreakerState, ComponentHealth},
};
use crate::utils::ArbitrageError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Read migration phases for gradual transition
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReadMigrationPhase {
    /// All reads from legacy system (0% new)
    LegacyOnly,
    /// 10% reads from new system for canary testing
    Canary,
    /// 25% reads from new system for initial rollout
    InitialRollout,
    /// 50% reads from new system for balanced testing
    Balanced,
    /// 75% reads from new system for advanced rollout
    AdvancedRollout,
    /// 90% reads from new system for final validation
    FinalValidation,
    /// All reads from new system (100% new)
    NewSystemOnly,
}

impl ReadMigrationPhase {
    /// Get the percentage of traffic that should go to new system
    pub fn new_system_percentage(&self) -> f64 {
        match self {
            ReadMigrationPhase::LegacyOnly => 0.0,
            ReadMigrationPhase::Canary => 0.1,
            ReadMigrationPhase::InitialRollout => 0.25,
            ReadMigrationPhase::Balanced => 0.5,
            ReadMigrationPhase::AdvancedRollout => 0.75,
            ReadMigrationPhase::FinalValidation => 0.9,
            ReadMigrationPhase::NewSystemOnly => 1.0,
        }
    }

    /// Get next phase in migration progression
    pub fn next_phase(&self) -> Option<Self> {
        match self {
            ReadMigrationPhase::LegacyOnly => Some(ReadMigrationPhase::Canary),
            ReadMigrationPhase::Canary => Some(ReadMigrationPhase::InitialRollout),
            ReadMigrationPhase::InitialRollout => Some(ReadMigrationPhase::Balanced),
            ReadMigrationPhase::Balanced => Some(ReadMigrationPhase::AdvancedRollout),
            ReadMigrationPhase::AdvancedRollout => Some(ReadMigrationPhase::FinalValidation),
            ReadMigrationPhase::FinalValidation => Some(ReadMigrationPhase::NewSystemOnly),
            ReadMigrationPhase::NewSystemOnly => None,
        }
    }

    /// Get previous phase for rollback
    pub fn previous_phase(&self) -> Option<Self> {
        match self {
            ReadMigrationPhase::LegacyOnly => None,
            ReadMigrationPhase::Canary => Some(ReadMigrationPhase::LegacyOnly),
            ReadMigrationPhase::InitialRollout => Some(ReadMigrationPhase::Canary),
            ReadMigrationPhase::Balanced => Some(ReadMigrationPhase::InitialRollout),
            ReadMigrationPhase::AdvancedRollout => Some(ReadMigrationPhase::Balanced),
            ReadMigrationPhase::FinalValidation => Some(ReadMigrationPhase::AdvancedRollout),
            ReadMigrationPhase::NewSystemOnly => Some(ReadMigrationPhase::FinalValidation),
        }
    }
}

/// Read routing strategies for intelligent traffic distribution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ReadRoutingStrategy {
    /// Random distribution based on percentage
    Random,
    /// Hash-based consistent routing for user sessions
    ConsistentHashing,
    /// Performance-based routing prioritizing faster system
    PerformanceBased,
    /// Geographic routing based on user location
    Geographic,
    /// Time-based routing for gradual migration windows
    TimeBased,
    /// A/B testing with control groups
    ABTesting,
}

/// Routing decision made by the migration manager
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoutingDecision {
    /// Whether to route to new system (true) or legacy (false)
    pub use_new_system: bool,
    /// Reason for the routing decision
    pub reason: String,
    /// Confidence score for the decision (0.0 to 1.0)
    pub confidence: f64,
    /// Timestamp of the decision
    pub timestamp: u64,
    /// Session identifier for consistent routing
    pub session_id: Option<String>,
}

/// Configuration for read migration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMigrationConfig {
    /// Enable read migration functionality
    pub enabled: bool,
    /// Current migration phase
    pub current_phase: ReadMigrationPhase,
    /// Read routing strategy
    pub routing_strategy: ReadRoutingStrategy,
    /// Phase transition thresholds
    pub phase_thresholds: HashMap<ReadMigrationPhase, u32>,
    /// Session routing settings
    pub session_routing: bool,
    pub session_consistency: bool,
    /// Health monitoring settings
    pub health_monitoring: bool,
    pub health_check_interval_seconds: u64,
    /// Circuit breaker settings
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
}

impl Default for ReadMigrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            current_phase: ReadMigrationPhase::LegacyOnly,
            routing_strategy: ReadRoutingStrategy::Random,
            phase_thresholds: HashMap::new(),
            session_routing: false,
            session_consistency: false,
            health_monitoring: false,
            health_check_interval_seconds: 30,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 10,
        }
    }
}

/// System-specific metrics tracking
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SystemMetrics {
    legacy_metrics: PerformanceMetrics,
    new_system_metrics: PerformanceMetrics,
    last_comparison: Instant,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            legacy_metrics: PerformanceMetrics::default(),
            new_system_metrics: PerformanceMetrics::default(),
            last_comparison: Instant::now(),
        }
    }
}

/// Read Migration Manager for intelligent traffic routing
pub struct ReadMigrationManager {
    config: ReadMigrationConfig,
    current_phase: Arc<Mutex<ReadMigrationPhase>>,
    legacy_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    new_system_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    session_routing: Arc<Mutex<HashMap<String, bool>>>, // true = new system
    migration_events: Arc<Mutex<Vec<MigrationEvent>>>,
    health_status: Arc<Mutex<ComponentHealth>>,
    start_time: Instant,
}

impl ReadMigrationManager {
    /// Create new ReadMigrationManager
    pub fn new(config: ReadMigrationConfig) -> Result<Self, ArbitrageError> {
        let legacy_cb = CircuitBreaker::new(
            config.circuit_breaker_threshold,
            config.circuit_breaker_timeout_seconds,
        );

        let new_system_cb = CircuitBreaker::new(
            config.circuit_breaker_threshold,
            config.circuit_breaker_timeout_seconds,
        );

        Ok(Self {
            current_phase: Arc::new(Mutex::new(config.current_phase)),
            config,
            legacy_circuit_breaker: Arc::new(Mutex::new(legacy_cb)),
            new_system_circuit_breaker: Arc::new(Mutex::new(new_system_cb)),
            session_routing: Arc::new(Mutex::new(HashMap::new())),
            migration_events: Arc::new(Mutex::new(Vec::new())),
            health_status: Arc::new(Mutex::new(ComponentHealth::new(
                true,                               // is_healthy
                "ReadMigrationManager".to_string(), // component_name
                0,                                  // uptime_seconds
                1.0,                                // performance_score
                0,                                  // error_count
                0,                                  // warning_count
            ))),
            start_time: Instant::now(),
        })
    }

    /// Make routing decision for a read request
    pub async fn route_read_request(
        &self,
        _request_id: String,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<RoutingDecision, ArbitrageError> {
        // Check circuit breaker states
        let legacy_available = {
            let mut legacy_cb = self.legacy_circuit_breaker.lock().unwrap();
            legacy_cb.can_execute()
        };

        let new_system_available = {
            let mut new_system_cb = self.new_system_circuit_breaker.lock().unwrap();
            new_system_cb.can_execute()
        };

        // If only one system is available, route to it
        if !legacy_available && new_system_available {
            return Ok(RoutingDecision {
                use_new_system: true,
                reason: "Legacy system circuit breaker open".to_string(),
                confidence: 1.0,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                session_id: session_id.clone(),
            });
        }

        if legacy_available && !new_system_available {
            return Ok(RoutingDecision {
                use_new_system: false,
                reason: "New system circuit breaker open".to_string(),
                confidence: 1.0,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                session_id: session_id.clone(),
            });
        }

        if !legacy_available && !new_system_available {
            return Err(ArbitrageError::infrastructure_error(
                "Both systems unavailable",
            ));
        }

        // Check session stickiness
        if let Some(session_id) = &session_id {
            let session_routing = self.session_routing.lock().unwrap();
            if let Some(&use_new_system) = session_routing.get(session_id) {
                return Ok(RoutingDecision {
                    use_new_system,
                    reason: "Session stickiness".to_string(),
                    confidence: 0.9,
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    session_id: Some(session_id.clone()),
                });
            }
        }

        // Get current phase and make routing decision
        let current_phase = {
            let phase = self.current_phase.lock().unwrap();
            *phase
        };

        let new_system_percentage = current_phase.new_system_percentage();

        let use_new_system = match self.config.routing_strategy {
            ReadRoutingStrategy::Random => {
                let random_value: f64 = rand::random();
                random_value < new_system_percentage
            }
            ReadRoutingStrategy::ConsistentHashing => {
                if let Some(user_id) = &user_id {
                    let hash = self.hash_string(user_id);
                    (hash % 100) as f64 / 100.0 < new_system_percentage
                } else {
                    let random_value: f64 = rand::random();
                    random_value < new_system_percentage
                }
            }
            ReadRoutingStrategy::PerformanceBased => {
                // Default to random for now - would need actual performance comparison
                let random_value: f64 = rand::random();
                random_value < new_system_percentage
            }
            _ => {
                // Default to random for other strategies
                let random_value: f64 = rand::random();
                random_value < new_system_percentage
            }
        };

        // Store session routing for stickiness
        if let Some(session_id) = &session_id {
            let mut session_routing = self.session_routing.lock().unwrap();
            session_routing.insert(session_id.clone(), use_new_system);
        }

        let decision = RoutingDecision {
            use_new_system,
            reason: format!(
                "Phase: {:?}, Strategy: {:?}",
                current_phase, self.config.routing_strategy
            ),
            confidence: if new_system_percentage == 0.0 || new_system_percentage == 1.0 {
                1.0
            } else {
                0.8
            },
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            session_id: session_id.clone(),
        };

        Ok(decision)
    }

    /// Record request result for performance tracking
    pub async fn record_request_result(
        &self,
        use_new_system: bool,
        _duration: Duration,
        success: bool,
    ) -> Result<(), ArbitrageError> {
        // Update circuit breakers
        if use_new_system {
            let mut cb = self.new_system_circuit_breaker.lock().unwrap();
            if success {
                cb.record_success();
            } else {
                cb.record_failure();
            }
        } else {
            let mut cb = self.legacy_circuit_breaker.lock().unwrap();
            if success {
                cb.record_success();
            } else {
                cb.record_failure();
            }
        }

        Ok(())
    }

    /// Set migration phase manually
    pub async fn set_migration_phase(
        &self,
        phase: ReadMigrationPhase,
    ) -> Result<(), ArbitrageError> {
        {
            let mut current_phase = self.current_phase.lock().unwrap();
            let old_phase = *current_phase;
            *current_phase = phase;

            // Clear session routing when changing phases significantly
            if (phase.new_system_percentage() - old_phase.new_system_percentage()).abs() > 0.25 {
                let mut session_routing = self.session_routing.lock().unwrap();
                session_routing.clear();
            }
        }

        self.record_migration_event(MigrationEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: crate::services::core::infrastructure::legacy_system_integration::shared_types::MigrationEventType::PhaseChanged,
            system_id: SystemIdentifier::new(
                LegacySystemType::Custom("read_migration_manager".to_string()),
                "read_migration_manager".to_string(),
                "1.0.0".to_string(),
            ),
            message: format!("Migration phase set to {:?}", phase),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            severity: crate::services::core::infrastructure::legacy_system_integration::shared_types::EventSeverity::Info,
            data: HashMap::new(),
        }).await?;

        Ok(())
    }

    /// Rollback to previous migration phase
    pub async fn rollback_phase(&self) -> Result<(), ArbitrageError> {
        let current_phase = {
            let phase = self.current_phase.lock().unwrap();
            *phase
        };

        if let Some(previous_phase) = current_phase.previous_phase() {
            self.set_migration_phase(previous_phase).await?;

            self.record_migration_event(MigrationEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                event_type: crate::services::core::infrastructure::legacy_system_integration::shared_types::MigrationEventType::RollbackStarted,
                system_id: SystemIdentifier::new(
                    LegacySystemType::Custom("read_migration_manager".to_string()),
                    "read_migration_manager".to_string(),
                    "1.0.0".to_string(),
                ),
                message: format!("Rolled back from {:?} to {:?}", current_phase, previous_phase),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                severity: crate::services::core::infrastructure::legacy_system_integration::shared_types::EventSeverity::Warning,
                data: HashMap::new(),
            }).await?;
        }

        Ok(())
    }

    /// Get current migration phase
    pub fn get_current_phase(&self) -> ReadMigrationPhase {
        let phase = self.current_phase.lock().unwrap();
        *phase
    }

    /// Get system health status
    pub async fn get_health(&self) -> Result<ComponentHealth, ArbitrageError> {
        let circuit_breaker_health = {
            let legacy_cb = self.legacy_circuit_breaker.lock().unwrap();
            let new_system_cb = self.new_system_circuit_breaker.lock().unwrap();
            legacy_cb.state == CircuitBreakerState::Closed
                && new_system_cb.state == CircuitBreakerState::Closed
        };

        let mut health = self.health_status.lock().unwrap().clone();

        let overall_healthy = circuit_breaker_health;

        health.is_healthy = overall_healthy;
        health.last_check = chrono::Utc::now().timestamp_millis() as u64;

        // Clean up old events (keep last 1000)
        {
            let mut events = self.migration_events.lock().unwrap();
            let len = events.len();
            if len > 1000 {
                events.drain(0..len - 1000);
            }
        }

        Ok(health)
    }

    /// Record migration event
    async fn record_migration_event(&self, event: MigrationEvent) -> Result<(), ArbitrageError> {
        let mut events = self.migration_events.lock().unwrap();
        events.push(event);

        // Keep only last 1000 events
        if events.len() > 1000 {
            let len = events.len();
            if len > 1000 {
                events.drain(0..len - 1000);
            }
        }

        Ok(())
    }

    /// Simple string hash function for consistent hashing
    fn hash_string(&self, s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Get migration events
    pub fn get_migration_events(&self) -> Vec<MigrationEvent> {
        let events = self.migration_events.lock().unwrap();
        events.clone()
    }

    /// Clear old session routing entries
    pub async fn cleanup_sessions(&self) -> Result<(), ArbitrageError> {
        let mut session_routing = self.session_routing.lock().unwrap();

        // In a real implementation, you'd check session timestamps
        // For now, just clear sessions older than the stickiness duration
        if self.start_time.elapsed().as_secs() > self.config.circuit_breaker_timeout_seconds {
            session_routing.clear();
        }

        Ok(())
    }
}

impl Default for ReadMigrationManager {
    fn default() -> Self {
        Self::new(ReadMigrationConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_phase_progression() {
        assert_eq!(
            ReadMigrationPhase::LegacyOnly.next_phase(),
            Some(ReadMigrationPhase::Canary)
        );
        assert_eq!(ReadMigrationPhase::NewSystemOnly.next_phase(), None);
        assert_eq!(
            ReadMigrationPhase::Canary.previous_phase(),
            Some(ReadMigrationPhase::LegacyOnly)
        );
        assert_eq!(ReadMigrationPhase::LegacyOnly.previous_phase(), None);
    }

    #[test]
    fn test_phase_percentages() {
        assert_eq!(ReadMigrationPhase::LegacyOnly.new_system_percentage(), 0.0);
        assert_eq!(ReadMigrationPhase::Canary.new_system_percentage(), 0.1);
        assert_eq!(ReadMigrationPhase::Balanced.new_system_percentage(), 0.5);
        assert_eq!(
            ReadMigrationPhase::NewSystemOnly.new_system_percentage(),
            1.0
        );
    }

    #[tokio::test]
    async fn test_read_migration_manager_creation() {
        let config = ReadMigrationConfig::default();
        let manager = ReadMigrationManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_routing_decision() {
        let config = ReadMigrationConfig::default();
        let manager = ReadMigrationManager::new(config).unwrap();

        let decision = manager
            .route_read_request(
                "test-request".to_string(),
                Some("test-user".to_string()),
                Some("test-session".to_string()),
            )
            .await;

        assert!(decision.is_ok());
        let decision = decision.unwrap();
        assert!(!decision.use_new_system); // Should be false for LegacyOnly phase
    }

    #[tokio::test]
    async fn test_phase_setting() {
        let config = ReadMigrationConfig::default();
        let manager = ReadMigrationManager::new(config).unwrap();

        assert_eq!(manager.get_current_phase(), ReadMigrationPhase::LegacyOnly);

        let result = manager
            .set_migration_phase(ReadMigrationPhase::Balanced)
            .await;
        assert!(result.is_ok());
        assert_eq!(manager.get_current_phase(), ReadMigrationPhase::Balanced);
    }
}
