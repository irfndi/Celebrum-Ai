//! Tests for monitoring utilities
//! Extracted from src/services/core/infrastructure/monitoring.rs

use crate::infrastructure::monitoring::*;
use std::time::Duration;

#[tokio::test]
async fn test_metrics_collection() {
    let collector = MetricsCollector::new();
    
    // Test counter increment
    collector.increment_counter("test_counter", 1.0);
    collector.increment_counter("test_counter", 2.0);
    
    let metrics = collector.get_metrics().await;
    assert!(metrics.contains_key("test_counter"));
    assert_eq!(metrics["test_counter"], 3.0);
}

#[tokio::test]
async fn test_histogram_recording() {
    let collector = MetricsCollector::new();
    
    // Record some values
    collector.record_histogram("response_time", 100.0);
    collector.record_histogram("response_time", 200.0);
    collector.record_histogram("response_time", 150.0);
    
    let histogram = collector.get_histogram("response_time").await;
    assert!(histogram.is_some());
    
    let hist = histogram.unwrap();
    assert_eq!(hist.count, 3);
    assert_eq!(hist.sum, 450.0);
    assert!((hist.average - 150.0).abs() < 0.001);
}

#[tokio::test]
async fn test_gauge_updates() {
    let collector = MetricsCollector::new();
    
    // Set gauge values
    collector.set_gauge("memory_usage", 75.5);
    collector.set_gauge("cpu_usage", 45.2);
    
    let metrics = collector.get_metrics().await;
    assert_eq!(metrics["memory_usage"], 75.5);
    assert_eq!(metrics["cpu_usage"], 45.2);
    
    // Update gauge
    collector.set_gauge("memory_usage", 80.0);
    let updated_metrics = collector.get_metrics().await;
    assert_eq!(updated_metrics["memory_usage"], 80.0);
}

#[tokio::test]
async fn test_health_check_registration() {
    let monitor = HealthMonitor::new();
    
    // Register health checks
    monitor.register_check("database", Box::new(|| async {
        Ok(HealthStatus::Healthy)
    }));
    
    monitor.register_check("redis", Box::new(|| async {
        Ok(HealthStatus::Healthy)
    }));
    
    let health = monitor.check_all().await;
    assert_eq!(health.overall_status, HealthStatus::Healthy);
    assert_eq!(health.checks.len(), 2);
}

#[tokio::test]
async fn test_health_check_failure() {
    let monitor = HealthMonitor::new();
    
    // Register a failing health check
    monitor.register_check("failing_service", Box::new(|| async {
        Err("Service unavailable".to_string())
    }));
    
    let health = monitor.check_all().await;
    assert_eq!(health.overall_status, HealthStatus::Unhealthy);
    assert!(health.checks.iter().any(|c| c.status == HealthStatus::Unhealthy));
}

#[tokio::test]
async fn test_alert_threshold_monitoring() {
    let monitor = AlertMonitor::new();
    
    // Set up threshold alerts
    monitor.add_threshold_alert("cpu_usage", 80.0, AlertSeverity::Warning);
    monitor.add_threshold_alert("memory_usage", 90.0, AlertSeverity::Critical);
    
    // Test values below threshold
    let alerts = monitor.check_thresholds(&[
        ("cpu_usage", 70.0),
        ("memory_usage", 85.0),
    ]).await;
    assert!(alerts.is_empty());
    
    // Test values above threshold
    let alerts = monitor.check_thresholds(&[
        ("cpu_usage", 85.0),
        ("memory_usage", 95.0),
    ]).await;
    assert_eq!(alerts.len(), 2);
}

#[tokio::test]
async fn test_performance_tracking() {
    let tracker = PerformanceTracker::new();
    
    // Start tracking an operation
    let operation_id = tracker.start_operation("api_request");
    
    // Simulate some work
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // End tracking
    let duration = tracker.end_operation(operation_id);
    assert!(duration >= Duration::from_millis(10));
    
    // Check that metrics were recorded
    let metrics = tracker.get_operation_metrics("api_request").await;
    assert!(metrics.is_some());
    assert_eq!(metrics.unwrap().count, 1);
}

#[tokio::test]
async fn test_error_rate_monitoring() {
    let monitor = ErrorRateMonitor::new();
    
    // Record some successful operations
    monitor.record_success("api_endpoint");
    monitor.record_success("api_endpoint");
    monitor.record_success("api_endpoint");
    
    // Record some errors
    monitor.record_error("api_endpoint", "timeout");
    
    let error_rate = monitor.get_error_rate("api_endpoint", Duration::from_secs(60)).await;
    assert!((error_rate - 0.25).abs() < 0.001); // 1 error out of 4 total = 25%
}

#[tokio::test]
async fn test_custom_metrics_export() {
    let exporter = MetricsExporter::new();
    
    // Add custom metrics
    exporter.add_custom_metric("business_metric", 42.0, &[
        ("environment", "test"),
        ("service", "worker"),
    ]);
    
    let exported = exporter.export_prometheus_format().await;
    assert!(exported.contains("business_metric"));
    assert!(exported.contains("environment=\"test\""));
    assert!(exported.contains("service=\"worker\""));
}

#[tokio::test]
async fn test_log_aggregation() {
    let aggregator = LogAggregator::new();
    
    // Add log entries
    aggregator.add_log_entry(LogLevel::Info, "test_service", "Operation completed");
    aggregator.add_log_entry(LogLevel::Error, "test_service", "Operation failed");
    aggregator.add_log_entry(LogLevel::Warning, "test_service", "Operation slow");
    
    let summary = aggregator.get_log_summary(Duration::from_secs(60)).await;
    assert_eq!(summary.total_entries, 3);
    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.warning_count, 1);
    assert_eq!(summary.info_count, 1);
}

#[tokio::test]
async fn test_resource_usage_monitoring() {
    let monitor = ResourceMonitor::new();
    
    let usage = monitor.get_current_usage().await;
    
    // Basic sanity checks
    assert!(usage.cpu_percent >= 0.0 && usage.cpu_percent <= 100.0);
    assert!(usage.memory_bytes > 0);
    assert!(usage.disk_usage_percent >= 0.0 && usage.disk_usage_percent <= 100.0);
}

#[tokio::test]
async fn test_distributed_tracing() {
    let tracer = DistributedTracer::new();
    
    // Start a trace
    let trace_id = tracer.start_trace("user_request");
    
    // Add spans
    let span1 = tracer.start_span(trace_id, "database_query");
    tokio::time::sleep(Duration::from_millis(5)).await;
    tracer.end_span(span1);
    
    let span2 = tracer.start_span(trace_id, "external_api_call");
    tokio::time::sleep(Duration::from_millis(10)).await;
    tracer.end_span(span2);
    
    // End trace
    let trace = tracer.end_trace(trace_id);
    
    assert_eq!(trace.spans.len(), 2);
    assert!(trace.total_duration >= Duration::from_millis(15));
}

#[tokio::test]
async fn test_monitoring_dashboard_data() {
    let dashboard = MonitoringDashboard::new();
    
    // Simulate adding various metrics
    dashboard.update_metric("requests_per_second", 150.0);
    dashboard.update_metric("average_response_time", 250.0);
    dashboard.update_metric("error_rate", 2.5);
    
    let dashboard_data = dashboard.get_dashboard_data().await;
    
    assert!(dashboard_data.contains_key("requests_per_second"));
    assert!(dashboard_data.contains_key("average_response_time"));
    assert!(dashboard_data.contains_key("error_rate"));
    
    assert_eq!(dashboard_data["requests_per_second"], 150.0);
    assert_eq!(dashboard_data["average_response_time"], 250.0);
    assert_eq!(dashboard_data["error_rate"], 2.5);
}