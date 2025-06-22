use crate::services::core::infrastructure::data_ingestion_module::*;

#[test]
fn test_ingestion_event_type_defaults() {
    assert_eq!(IngestionEventType::MarketData.as_str(), "market_data");
    assert_eq!(
        IngestionEventType::MarketData.default_pipeline_id(),
        "prod-market-data-pipeline"
    );
    assert_eq!(
        IngestionEventType::Analytics.default_queue_name(),
        "analytics-queue"
    );
    assert_eq!(IngestionEventType::Audit.default_r2_prefix(), "audit");
}

#[test]
fn test_ingestion_event_creation() {
    let data = serde_json::json!({"test": "data"});
    let event = IngestionEvent::new(
        IngestionEventType::MarketData,
        "test_source".to_string(),
        data.clone(),
    );

    assert_eq!(event.event_type, IngestionEventType::MarketData);
    assert_eq!(event.source, "test_source");
    assert_eq!(event.data, data);
    assert_eq!(event.priority, 2);
    assert_eq!(event.retry_count, 0);
    assert_eq!(event.max_retries, 3);
}

#[test]
fn test_ingestion_event_expiration() {
    let mut event = IngestionEvent::new(
        IngestionEventType::Analytics,
        "test".to_string(),
        serde_json::json!({}),
    );

    // Event without TTL should not expire
    assert!(!event.is_expired());

    // Event with future TTL should not expire
    event = event.with_ttl(3600); // 1 hour
    assert!(!event.is_expired());

    // Simulate old timestamp
    event.timestamp = chrono::Utc::now().timestamp_millis() as u64 - 7200000; // 2 hours ago
    event = event.with_ttl(3600); // 1 hour TTL
    assert!(event.is_expired());
}

#[test]
fn test_data_ingestion_module_config_validation() {
    let mut config = DataIngestionModuleConfig::default();
    assert!(config.validate().is_ok());

    config.health_check_interval_seconds = 0;
    assert!(config.validate().is_err());

    config.health_check_interval_seconds = 30;
    config.r2_bucket_name = "".to_string();
    assert!(config.validate().is_err());

    config.r2_bucket_name = "test-bucket".to_string();
    config.compression_threshold_bytes = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_high_throughput_config() {
    let config = DataIngestionModuleConfig::high_throughput();
    assert!(config.enable_performance_optimization);
    assert_eq!(config.compression_threshold_bytes, 512);
}

#[test]
fn test_high_reliability_config() {
    let config = DataIngestionModuleConfig::high_reliability();
    assert!(config.enable_kv_fallback);
    assert_eq!(config.health_check_interval_seconds, 15);
}