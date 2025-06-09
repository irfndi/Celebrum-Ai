//! Fault Injection Module for Chaos Engineering

use crate::utils::error::{ArbitrageError, ArbitrageResult, ErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Removed unused imports - Duration and Instant not needed in main fault injection module
use worker::Env;

use super::ChaosEngineeringConfig;

// Re-export storage fault types from sibling modules
pub use super::d1_faults::D1FaultInjector;
pub use super::kv_faults::KvFaultInjector;
pub use super::network_chaos::NetworkChaosInjector;
pub use super::r2_faults::R2FaultInjector;
pub use super::resource_chaos::ResourceChaosInjector;

/// Types of faults to inject
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FaultType {
    /// Service unavailability
    Unavailability,
    /// High latency injection
    Latency,
    /// Connection timeouts
    Timeout,
    /// Data corruption simulation
    DataCorruption,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Intermittent failures
    IntermittentFailure,
    /// Gradual degradation
    GradualDegradation,
    /// Connection pool exhaustion
    ConnectionPoolExhaustion,
    /// Rate limiting simulation
    RateLimiting,
    /// Partial service unavailability
    PartialUnavailability,
}

/// Target systems for fault injection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InjectionTarget {
    /// Cloudflare KV store
    KvStore,
    /// D1 database
    D1Database,
    /// R2 object storage
    R2Storage,
    /// Network layer
    NetworkLayer,
    /// Memory allocation
    MemorySystem,
    /// CPU processing
    CpuSystem,
}

/// Fault injection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultConfig {
    /// Unique identifier for the fault
    pub id: String,
    /// Target system for injection
    pub target: InjectionTarget,
    /// Type of fault to inject as string
    pub fault_type: String,
    /// Intensity of the fault (0.0 to 1.0)
    pub intensity: f64,
    /// Duration in seconds
    pub duration_seconds: u64,
    /// Additional parameters
    pub parameters: HashMap<String, String>,
    /// When the fault was activated
    pub activated_at: Option<u64>,
    /// Whether fault is currently active
    pub is_active: bool,
}

/// Fault injection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultInjectionStatus {
    /// Total active faults
    pub total_active_faults: usize,
    /// Faults by target system
    pub faults_by_target: HashMap<String, usize>,
    /// Faults by type
    pub faults_by_type: HashMap<String, usize>,
    /// Average fault intensity
    pub average_intensity: f64,
    /// Storage system health impact
    pub storage_health_impact: StorageHealthImpact,
}

/// Storage system health impact metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealthImpact {
    /// KV store availability percentage
    pub kv_availability: f64,
    /// D1 database availability percentage
    pub d1_availability: f64,
    /// R2 storage availability percentage
    pub r2_availability: f64,
    /// Overall storage availability
    pub overall_availability: f64,
}

/// Fault Injector for chaos experiments
#[derive(Debug)]
pub struct FaultInjector {
    config: ChaosEngineeringConfig,
    active_faults: HashMap<String, FaultConfig>,
    kv_fault_injector: Option<KvFaultInjector>,
    d1_fault_injector: Option<D1FaultInjector>,
    r2_fault_injector: Option<R2FaultInjector>,
    network_chaos_injector: Option<NetworkChaosInjector>,
    resource_chaos_injector: Option<ResourceChaosInjector>,
    fault_counters: FaultCounters,
    is_initialized: bool,
}

/// Fault injection counters for metrics
#[derive(Debug, Default)]
struct FaultCounters {
    total_injected: u64,
    successful_injections: u64,
    failed_injections: u64,
    active_count: u64,
    storage_faults: u64,
    network_faults: u64,
    resource_faults: u64,
}

impl FaultInjector {
    pub async fn new(config: &ChaosEngineeringConfig, env: &Env) -> ArbitrageResult<Self> {
        let mut injector = Self {
            config: config.clone(),
            active_faults: HashMap::new(),
            kv_fault_injector: None,
            d1_fault_injector: None,
            r2_fault_injector: None,
            network_chaos_injector: None,
            resource_chaos_injector: None,
            fault_counters: FaultCounters::default(),
            is_initialized: false,
        };

        // Initialize storage-specific fault injectors if feature flags are enabled
        if config.feature_flags.storage_fault_injection {
            injector.kv_fault_injector = Some(KvFaultInjector::new(config, env).await?);
            injector.d1_fault_injector = Some(D1FaultInjector::new(config, env).await?);
            injector.r2_fault_injector = Some(R2FaultInjector::new(config, env).await?);
        }

        // Initialize network chaos injector if feature flag is enabled
        if config.feature_flags.network_chaos_simulation {
            let mut network_injector = NetworkChaosInjector::new(config.clone());
            network_injector.initialize(env).await?;
            injector.network_chaos_injector = Some(network_injector);
        }

        // Initialize resource chaos injector if feature flag is enabled
        if config.feature_flags.resource_exhaustion_testing {
            let mut resource_injector = ResourceChaosInjector::new(config.clone());
            resource_injector.initialize(env).await?;
            injector.resource_chaos_injector = Some(resource_injector);
        }

        injector.is_initialized = true;
        Ok(injector)
    }

    /// Inject a fault into a target system
    pub async fn inject_fault(
        &mut self,
        fault_id: String,
        mut fault_config: FaultConfig,
    ) -> ArbitrageResult<()> {
        // Validate fault intensity
        if fault_config.intensity > self.config.max_fault_intensity {
            self.fault_counters.failed_injections += 1;
            return Err(ArbitrageError::new(
                ErrorKind::ValidationError,
                format!(
                    "Fault intensity {} exceeds maximum allowed {}",
                    fault_config.intensity, self.config.max_fault_intensity
                ),
            ));
        }

        // Check feature flags for the specific fault type
        let feature_enabled = match fault_config.target {
            InjectionTarget::KvStore | InjectionTarget::D1Database | InjectionTarget::R2Storage => {
                self.config.feature_flags.storage_fault_injection
            }
            InjectionTarget::NetworkLayer => self.config.feature_flags.network_chaos_simulation,
            InjectionTarget::MemorySystem | InjectionTarget::CpuSystem => {
                self.config.feature_flags.resource_exhaustion_testing
            }
        };

        if !feature_enabled {
            self.fault_counters.failed_injections += 1;
            return Err(ArbitrageError::new(
                ErrorKind::ValidationError,
                format!(
                    "Fault injection not enabled for target {:?}",
                    fault_config.target
                ),
            ));
        }

        // Set activation timestamp and status
        fault_config.activated_at = Some(chrono::Utc::now().timestamp() as u64);
        fault_config.is_active = true;

        // Delegate to storage-specific injectors
        match fault_config.target {
            InjectionTarget::KvStore => {
                if let Some(injector) = &mut self.kv_fault_injector {
                    injector.inject_fault(&fault_id, &fault_config).await?;
                }
                self.fault_counters.storage_faults += 1;
            }
            InjectionTarget::D1Database => {
                if let Some(injector) = &mut self.d1_fault_injector {
                    injector.inject_fault(&fault_id, &fault_config).await?;
                }
                self.fault_counters.storage_faults += 1;
            }
            InjectionTarget::R2Storage => {
                if let Some(injector) = &mut self.r2_fault_injector {
                    injector.inject_fault(&fault_id, &fault_config).await?;
                }
                self.fault_counters.storage_faults += 1;
            }
            InjectionTarget::NetworkLayer => {
                self.fault_counters.network_faults += 1;
            }
            InjectionTarget::MemorySystem | InjectionTarget::CpuSystem => {
                self.fault_counters.resource_faults += 1;
            }
        }

        // Store active fault for tracking
        self.active_faults.insert(fault_id, fault_config);
        self.fault_counters.total_injected += 1;
        self.fault_counters.successful_injections += 1;
        self.fault_counters.active_count += 1;

        Ok(())
    }

    /// Remove a fault from a target system
    pub async fn remove_fault(&mut self, fault_id: &str) -> ArbitrageResult<()> {
        if let Some(fault_config) = self.active_faults.get(fault_id) {
            // Delegate to storage-specific injectors for removal
            match fault_config.target {
                InjectionTarget::KvStore => {
                    if let Some(injector) = &mut self.kv_fault_injector {
                        injector.remove_fault(fault_id).await?;
                    }
                }
                InjectionTarget::D1Database => {
                    if let Some(injector) = &mut self.d1_fault_injector {
                        injector.remove_fault(fault_id).await?;
                    }
                }
                InjectionTarget::R2Storage => {
                    if let Some(injector) = &mut self.r2_fault_injector {
                        injector.remove_fault(fault_id).await?;
                    }
                }
                _ => {}
            }

            self.active_faults.remove(fault_id);
            self.fault_counters.active_count -= 1;
        }

        Ok(())
    }

    /// Get all active faults
    pub fn get_active_faults(&self) -> &HashMap<String, FaultConfig> {
        &self.active_faults
    }

    /// Get fault injection status
    pub fn get_injection_status(&self) -> FaultInjectionStatus {
        let mut faults_by_target = HashMap::new();
        let mut faults_by_type = HashMap::new();
        let mut total_intensity = 0.0;

        for fault in self.active_faults.values() {
            let target_name = format!("{:?}", fault.target);
            let type_name = format!("{:?}", fault.fault_type);

            *faults_by_target.entry(target_name).or_insert(0) += 1;
            *faults_by_type.entry(type_name).or_insert(0) += 1;
            total_intensity += fault.intensity;
        }

        let average_intensity = if !self.active_faults.is_empty() {
            total_intensity / self.active_faults.len() as f64
        } else {
            0.0
        };

        let storage_health_impact = self.calculate_storage_health_impact();

        FaultInjectionStatus {
            total_active_faults: self.active_faults.len(),
            faults_by_target,
            faults_by_type,
            average_intensity,
            storage_health_impact,
        }
    }

    /// Calculate storage system health impact
    fn calculate_storage_health_impact(&self) -> StorageHealthImpact {
        let kv_availability = self.calculate_system_availability(&InjectionTarget::KvStore);
        let d1_availability = self.calculate_system_availability(&InjectionTarget::D1Database);
        let r2_availability = self.calculate_system_availability(&InjectionTarget::R2Storage);

        let overall_availability = (kv_availability + d1_availability + r2_availability) / 3.0;

        StorageHealthImpact {
            kv_availability,
            d1_availability,
            r2_availability,
            overall_availability,
        }
    }

    /// Calculate availability for a specific storage system
    fn calculate_system_availability(&self, target: &InjectionTarget) -> f64 {
        let system_faults: Vec<&FaultConfig> = self
            .active_faults
            .values()
            .filter(|fault| &fault.target == target)
            .collect();

        if system_faults.is_empty() {
            return 100.0;
        }

        let total_impact: f64 = system_faults
            .iter()
            .map(|fault| match fault.fault_type.as_str() {
                "unavailability" => fault.intensity * 100.0,
                "partial_unavailability" => fault.intensity * 50.0,
                "timeout" | "latency" => fault.intensity * 30.0,
                "data_corruption" => fault.intensity * 40.0,
                "intermittent_failure" => fault.intensity * 25.0,
                "gradual_degradation" => fault.intensity * 20.0,
                "connection_pool_exhaustion" => fault.intensity * 60.0,
                "rate_limiting" => fault.intensity * 35.0,
                "resource_exhaustion" => fault.intensity * 45.0,
                _ => fault.intensity * 30.0, // Default impact
            })
            .sum();

        (100.0 - total_impact.min(100.0)).max(0.0)
    }

    /// Check if any storage faults are active
    pub fn has_storage_faults(&self) -> bool {
        self.active_faults.values().any(|fault| {
            matches!(
                fault.target,
                InjectionTarget::KvStore | InjectionTarget::D1Database | InjectionTarget::R2Storage
            )
        })
    }

    /// Check if any network faults are active
    pub fn has_network_faults(&self) -> bool {
        self.active_faults
            .values()
            .any(|fault| matches!(fault.target, InjectionTarget::NetworkLayer))
    }

    /// Get fault count by target type
    pub fn get_fault_count_by_target(&self, target: &InjectionTarget) -> usize {
        self.active_faults
            .values()
            .filter(|fault| &fault.target == target)
            .count()
    }

    /// Get fault counters for metrics
    pub fn get_fault_counters(&self) -> (u64, u64, u64, u64) {
        (
            self.fault_counters.total_injected,
            self.fault_counters.successful_injections,
            self.fault_counters.failed_injections,
            self.fault_counters.active_count,
        )
    }

    /// Check if a specific storage system has faults
    pub fn has_kv_faults(&self) -> bool {
        self.get_fault_count_by_target(&InjectionTarget::KvStore) > 0
    }

    pub fn has_d1_faults(&self) -> bool {
        self.get_fault_count_by_target(&InjectionTarget::D1Database) > 0
    }

    pub fn has_r2_faults(&self) -> bool {
        self.get_fault_count_by_target(&InjectionTarget::R2Storage) > 0
    }

    /// Cleanup expired faults
    pub async fn cleanup_expired_faults(&mut self) -> ArbitrageResult<()> {
        let current_timestamp = chrono::Utc::now().timestamp() as u64;
        let mut expired_faults = Vec::new();

        for (fault_id, fault_config) in &self.active_faults {
            if let Some(activated_at) = fault_config.activated_at {
                if current_timestamp - activated_at > fault_config.duration_seconds {
                    expired_faults.push(fault_id.clone());
                }
            }
        }

        for fault_id in expired_faults {
            self.remove_fault(&fault_id).await?;
        }

        Ok(())
    }

    pub async fn is_healthy(&self) -> ArbitrageResult<bool> {
        if !self.is_initialized {
            return Ok(false);
        }

        // Check health of storage-specific injectors
        if let Some(injector) = &self.kv_fault_injector {
            if !injector.is_healthy().await? {
                return Ok(false);
            }
        }

        if let Some(injector) = &self.d1_fault_injector {
            if !injector.is_healthy().await? {
                return Ok(false);
            }
        }

        if let Some(injector) = &self.r2_fault_injector {
            if !injector.is_healthy().await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn shutdown(&mut self) -> ArbitrageResult<()> {
        // Shutdown storage-specific injectors
        if let Some(injector) = &mut self.kv_fault_injector {
            injector.shutdown().await?;
        }
        if let Some(injector) = &mut self.d1_fault_injector {
            injector.shutdown().await?;
        }
        if let Some(injector) = &mut self.r2_fault_injector {
            injector.shutdown().await?;
        }

        // Remove all active faults
        self.active_faults.clear();
        self.is_initialized = false;
        Ok(())
    }
}

impl Default for StorageHealthImpact {
    fn default() -> Self {
        Self {
            kv_availability: 100.0,
            d1_availability: 100.0,
            r2_availability: 100.0,
            overall_availability: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ChaosEngineeringConfig {
        let mut config = ChaosEngineeringConfig::default();
        config.feature_flags.storage_fault_injection = true;
        config.feature_flags.network_chaos_simulation = true;
        config.feature_flags.resource_exhaustion_testing = true;
        config
    }

    #[test]
    fn test_fault_config_creation() {
        let fault_config = FaultConfig {
            id: "test_fault_001".to_string(),
            target: InjectionTarget::KvStore,
            fault_type: "latency".to_string(),
            intensity: 0.5,
            duration_seconds: 60,
            parameters: HashMap::new(),
            activated_at: None,
            is_active: false,
        };

        assert_eq!(fault_config.target, InjectionTarget::KvStore);
        assert_eq!(fault_config.fault_type, "latency");
        assert_eq!(fault_config.intensity, 0.5);
        assert!(!fault_config.is_active);
    }

    #[test]
    fn test_fault_intensity_validation() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_storage_health_impact_calculation() {
        let impact = StorageHealthImpact::default();
        assert_eq!(impact.kv_availability, 100.0);
        assert_eq!(impact.d1_availability, 100.0);
        assert_eq!(impact.r2_availability, 100.0);
        assert_eq!(impact.overall_availability, 100.0);
    }
}
