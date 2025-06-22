use crate::services::core::infrastructure::data_access_layer::unified_data_access_engine::*;
use std::collections::HashMap;

#[test]
fn test_unified_data_access_config_default() {
    let config = UnifiedDataAccessConfig::default();
    assert!(config.enable_caching);
    assert_eq!(config.cache_ttl_seconds, 300);
    assert!(config.enable_validation);
    assert!(config.compression_enabled);
}

#[test]
fn test_data_priority_ordering() {
    assert!(DataPriority::Critical > DataPriority::High);
    assert!(DataPriority::High > DataPriority::Medium);
    assert!(DataPriority::Medium > DataPriority::Low);
}

#[test]
fn test_create_simple_request() {
    let request = create_simple_request(
        DataSourceType::API,
        "test/endpoint".to_string(),
        Some("test_cache_key".to_string()),
    );

    assert_eq!(request.source_type, DataSourceType::API);
    assert_eq!(request.endpoint, "test/endpoint");
    assert_eq!(request.cache_key, Some("test_cache_key".to_string()));
    assert_eq!(request.priority, DataPriority::Medium);
}

#[test]
fn test_create_exchange_request() {
    let mut params = HashMap::new();
    params.insert("symbol".to_string(), "BTCUSDT".to_string());

    let request =
        create_exchange_request("binance".to_string(), "ticker/price".to_string(), params);

    assert!(matches!(request.source_type, DataSourceType::Exchange(_)));
    assert_eq!(request.priority, DataPriority::High);
    assert!(request.retry_config.is_some());
}

#[test]
fn test_unified_data_access_engine_builder() {
    let engine = UnifiedDataAccessEngineBuilder::new()
        .with_caching(true, 600)
        .with_compression(true, 2048)
        .with_validation(true, 30000)
        .with_rate_limiting(200)
        .with_circuit_breaker(true, 10)
        .build();

    assert!(engine.is_ok());
}

#[test]
fn test_cache_eviction_policy_serialization() {
    let policy = CacheEvictionPolicy::LRU;
    let serialized = serde_json::to_string(&policy).unwrap();
    let deserialized: CacheEvictionPolicy = serde_json::from_str(&serialized).unwrap();

    match (policy, deserialized) {
        (CacheEvictionPolicy::LRU, CacheEvictionPolicy::LRU) => {}
        _ => panic!("Serialization failed"),
    }
}