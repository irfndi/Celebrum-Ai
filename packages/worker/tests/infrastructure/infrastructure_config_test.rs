use arbedge_worker::infrastructure::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infrastructure_config_default() {
        let config = InfrastructureConfig::default();
        assert_eq!(config.max_concurrent_users, 2500);
        assert!(config.enable_high_performance_mode);
        assert!(config.enable_comprehensive_monitoring);
        assert!(config.enable_intelligent_caching);
    }

    #[test]
    fn test_high_concurrency_config() {
        let config = InfrastructureConfig::high_concurrency();
        assert_eq!(config.max_concurrent_users, 2500);
        assert!(config.enable_high_performance_mode);
        assert_eq!(
            config.unified_core_config.circuit_breaker.failure_threshold,
            10
        );
    }

    #[test]
    fn test_high_reliability_config() {
        let config = InfrastructureConfig::high_reliability();
        assert!(config.enable_comprehensive_monitoring);
        assert_eq!(
            config.unified_core_config.circuit_breaker.failure_threshold,
            10
        );
        assert!(config.unified_core_config.failover.enable_auto_failover);
    }

    #[test]
    fn test_config_validation() {
        let mut config = InfrastructureConfig::default();
        assert!(config.validate().is_ok());

        config.max_concurrent_users = 0;
        assert!(config.validate().is_err());

        config.max_concurrent_users = 15000;
        assert!(config.validate().is_err());
    }
}