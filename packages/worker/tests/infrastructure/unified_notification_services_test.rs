use crate::services::core::infrastructure::unified_notification_services::*;

#[test]
fn test_notification_type_properties() {
    let security_alert = NotificationType::SecurityAlert;
    assert_eq!(security_alert.as_str(), "security_alert");
    assert_eq!(security_alert.default_priority(), 10);
    assert!(security_alert
        .default_channels()
        .contains(&NotificationChannel::Email));
}

#[test]
fn test_notification_priority_conversion() {
    let priority = NotificationPriority::High;
    assert_eq!(priority.as_u8(), 8);
    assert_eq!(NotificationPriority::from_u8(8), NotificationPriority::High);
}

#[test]
fn test_unified_notification_services_config_validation() {
    let config = UnifiedNotificationServicesConfig::default();
    assert!(config.validate().is_ok());

    let invalid_config = UnifiedNotificationServicesConfig {
        max_notifications_per_minute: 0,
        ..UnifiedNotificationServicesConfig::default()
    };
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_high_performance_config() {
    let config = UnifiedNotificationServicesConfig::high_performance();
    assert!(config.enable_high_performance_mode);
    assert_eq!(config.max_notifications_per_minute, 500);
}

#[test]
fn test_high_reliability_config() {
    let config = UnifiedNotificationServicesConfig::high_reliability();
    assert!(config.enable_high_reliability_mode);
    assert_eq!(config.max_notifications_per_minute, 50);
}

#[test]
fn test_notification_request_validation() {
    let request = NotificationRequest::new(
        "user123".to_string(),
        "opportunity_alert".to_string(),
        vec![NotificationChannel::Email],
    )
    .with_recipient(NotificationChannel::Email, "user@example.com".to_string())
    .with_custom_content("Test notification".to_string());

    assert!(request.validate().is_ok());

    let invalid_request = NotificationRequest::new(
        "user123".to_string(),
        "opportunity_alert".to_string(),
        vec![],
    );
    assert!(invalid_request.validate().is_err());
}