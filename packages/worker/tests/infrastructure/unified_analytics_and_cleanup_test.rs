//! Tests for unified analytics and cleanup services
//! Extracted from src/services/core/infrastructure/unified_analytics_and_cleanup.rs

use std::collections::HashMap;

// Note: These imports will need to be updated based on the actual module structure
// after the monorepo reorganization is complete
use crate::infrastructure::unified_analytics_and_cleanup::*;

#[tokio::test]
async fn test_analytics_event_tracking() {
    let config = UnifiedAnalyticsAndCleanupConfig::default();
    let service = UnifiedAnalyticsAndCleanup::new(config);

    let event = AnalyticsData {
        timestamp: 1234567890,
        event_type: "test_event".to_string(),
        user_id: Some("user123".to_string()),
        session_id: Some("session456".to_string()),
        data: HashMap::new(),
        metadata: AnalyticsMetadata {
            source: "test".to_string(),
            version: "1.0".to_string(),
            environment: "test".to_string(),
            region: None,
        },
    };

    let result = service.track_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cleanup_scheduling() {
    let config = UnifiedAnalyticsAndCleanupConfig::default();
    let service = UnifiedAnalyticsAndCleanup::new(config);

    let policy = CleanupPolicy {
        name: "test_cleanup".to_string(),
        resource_type: ResourceType::TempFiles,
        retention_period_days: 7,
        size_threshold_mb: Some(100),
        priority: CleanupPriority::Medium,
    };

    let result = service.schedule_cleanup(policy).await;
    assert!(result.is_ok());

    let operation_id = result.unwrap();
    assert!(operation_id.starts_with("cleanup_"));
}

#[tokio::test]
async fn test_metrics_collection() {
    let config = UnifiedAnalyticsAndCleanupConfig::default();
    let service = UnifiedAnalyticsAndCleanup::new(config);

    // Test metrics collection functionality
    // This test would be expanded based on the actual metrics collection implementation
    let result = service.collect_metrics().await;
    assert!(result.is_ok());
}