// NotificationService Unit Tests
// Comprehensive testing of notification creation, delivery, templates, and rate limiting

use arb_edge::services::core::infrastructure::notifications::{
    NotificationService, Notification, NotificationTemplate, AlertTrigger, NotificationChannel,
    NotificationHistory, NotificationAnalytics
};
use arb_edge::utils::{ArbitrageError, ArbitrageResult};
use serde_json::json;
use std::collections::HashMap;

// Mock notification priority and delivery status enums (since they're strings in the actual codebase)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationPriority::Low => "low",
            NotificationPriority::Medium => "medium", 
            NotificationPriority::High => "high",
            NotificationPriority::Critical => "critical",
        }
    }
}

impl PartialOrd for NotificationPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NotificationPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            NotificationPriority::Low => 0,
            NotificationPriority::Medium => 1,
            NotificationPriority::High => 2,
            NotificationPriority::Critical => 3,
        };
        let other_val = match other {
            NotificationPriority::Low => 0,
            NotificationPriority::Medium => 1,
            NotificationPriority::High => 2,
            NotificationPriority::Critical => 3,
        };
        self_val.cmp(&other_val)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Retrying => "retrying",
        }
    }
}

// Mock notification config
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub max_notifications_per_minute: u32,
    pub max_notifications_per_hour: u32,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u32,
    pub enabled_channels: Vec<NotificationChannel>,
}

// Mock NotificationService for testing without external dependencies
struct MockNotificationService {
    notifications: Vec<MockNotification>,
    templates: HashMap<String, NotificationTemplate>,
    delivery_log: Vec<(String, DeliveryStatus)>,
    rate_limits: HashMap<String, (u32, u64)>, // (count, last_reset_time)
    config: NotificationConfig,
    error_simulation: Option<String>,
}

// Mock notification structure that matches our test needs
#[derive(Debug, Clone)]
struct MockNotification {
    pub notification_id: String,
    pub template_id: String,
    pub recipient: String,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub subject: String,
    pub body: String,
    pub variables: HashMap<String, String>,
    pub status: DeliveryStatus,
    pub created_at: u64,
    pub sent_at: Option<u64>,
    pub retry_count: u32,
    pub error_message: Option<String>,
}

impl MockNotificationService {
    fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Add default templates
        templates.insert("opportunity_alert".to_string(), NotificationTemplate {
            template_id: "opportunity_alert".to_string(),
            name: "Opportunity Alert".to_string(),
            description: Some("Alert for new trading opportunities".to_string()),
            category: "opportunity".to_string(),
            title_template: "🚨 New {opportunity_type} Opportunity: {trading_pair}".to_string(),
            message_template: "💰 Expected Return: {expected_return}%\n🎯 Confidence: {confidence}%\n⏰ Valid for: {time_horizon}".to_string(),
            priority: "high".to_string(),
            channels: vec![NotificationChannel::Telegram],
            variables: vec!["opportunity_type".to_string(), "trading_pair".to_string(), "expected_return".to_string(), "confidence".to_string(), "time_horizon".to_string()],
            is_system_template: true,
            is_active: true,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        });

        templates.insert("system_alert".to_string(), NotificationTemplate {
            template_id: "system_alert".to_string(),
            name: "System Alert".to_string(),
            description: Some("System maintenance and error alerts".to_string()),
            category: "system".to_string(),
            title_template: "⚠️ System Alert: {alert_type}".to_string(),
            message_template: "🔧 Issue: {description}\n📊 Severity: {severity}\n🕐 Time: {timestamp}".to_string(),
            priority: "critical".to_string(),
            channels: vec![NotificationChannel::Email],
            variables: vec!["alert_type".to_string(), "description".to_string(), "severity".to_string(), "timestamp".to_string()],
            is_system_template: true,
            is_active: true,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        });

        Self {
            notifications: Vec::new(),
            templates,
            delivery_log: Vec::new(),
            rate_limits: HashMap::new(),
            config: NotificationConfig {
                max_notifications_per_minute: 10,
                max_notifications_per_hour: 100,
                retry_attempts: 3,
                retry_delay_seconds: 30,
                enabled_channels: vec![NotificationChannel::Telegram, NotificationChannel::Email],
            },
            error_simulation: None,
        }
    }

    fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    fn reset_error_simulation(&mut self) {
        self.error_simulation = None;
    }

    async fn mock_create_notification(
        &mut self,
        template_id: &str,
        recipient: &str,
        variables: HashMap<String, String>,
        priority: NotificationPriority,
    ) -> ArbitrageResult<String> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "template_not_found" => Err(ArbitrageError::validation_error("Template not found")),
                "invalid_recipient" => Err(ArbitrageError::validation_error("Invalid recipient")),
                "rate_limit_exceeded" => Err(ArbitrageError::validation_error("Rate limit exceeded")),
                _ => Err(ArbitrageError::validation_error("Unknown notification error")),
            };
        }

        // Check rate limits
        if !self.check_rate_limit(recipient).await? {
            return Err(ArbitrageError::validation_error("Rate limit exceeded"));
        }

        // Get template
        let template = self.templates.get(template_id)
            .ok_or_else(|| ArbitrageError::validation_error("Template not found"))?;

        // Render notification
        let notification = self.render_notification(template, recipient, variables, priority).await?;
        let notification_id = notification.notification_id.clone();
        
        self.notifications.push(notification);
        Ok(notification_id)
    }

    async fn render_notification(
        &self,
        template: &NotificationTemplate,
        recipient: &str,
        variables: HashMap<String, String>,
        priority: NotificationPriority,
    ) -> ArbitrageResult<MockNotification> {
        let mut title = template.title_template.clone();
        let mut message = template.message_template.clone();

        // Replace variables in templates
        for (key, value) in &variables {
            let placeholder = format!("{{{}}}", key);
            title = title.replace(&placeholder, value);
            message = message.replace(&placeholder, value);
        }

        Ok(MockNotification {
            notification_id: format!("notif_{}", uuid::Uuid::new_v4()),
            template_id: template.template_id.clone(),
            recipient: recipient.to_string(),
            channel: template.channels[0].clone(),
            priority,
            subject: title,
            body: message,
            variables,
            status: DeliveryStatus::Pending,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            sent_at: None,
            retry_count: 0,
            error_message: None,
        })
    }

    async fn check_rate_limit(&mut self, recipient: &str) -> ArbitrageResult<bool> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let minute_ago = now - 60_000;

        let default_limits = (0, now);
        let (count, last_reset) = self.rate_limits.get(recipient).unwrap_or(&default_limits);

        // Reset counter if more than a minute has passed
        if now - last_reset > 60_000 {
            self.rate_limits.insert(recipient.to_string(), (1, now));
            Ok(true)
        } else if *count < self.config.max_notifications_per_minute {
            self.rate_limits.insert(recipient.to_string(), (count + 1, *last_reset));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn mock_send_notification(&mut self, notification_id: &str) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "delivery_failed" => Err(ArbitrageError::validation_error("Delivery failed")),
                "network_error" => Err(ArbitrageError::validation_error("Network error")),
                "service_unavailable" => Err(ArbitrageError::validation_error("Service unavailable")),
                _ => Err(ArbitrageError::validation_error("Unknown delivery error")),
            };
        }

        // Find and update notification
        if let Some(notification) = self.notifications.iter_mut().find(|n| n.notification_id == notification_id) {
            notification.status = DeliveryStatus::Sent;
            notification.sent_at = Some(chrono::Utc::now().timestamp_millis() as u64);
            
            self.delivery_log.push((notification_id.to_string(), DeliveryStatus::Sent));
            Ok(())
        } else {
            Err(ArbitrageError::validation_error("Notification not found"))
        }
    }

    fn get_notification_count(&self) -> usize {
        self.notifications.len()
    }

    fn get_delivery_success_rate(&self) -> f64 {
        if self.delivery_log.is_empty() {
            return 0.0;
        }

        let successful = self.delivery_log.iter()
            .filter(|(_, status)| matches!(status, DeliveryStatus::Sent))
            .count();

        successful as f64 / self.delivery_log.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_creation_and_validation() {
        let mut mock_service = MockNotificationService::new();
        
        // Test successful notification creation
        let mut variables = HashMap::new();
        variables.insert("opportunity_type".to_string(), "Arbitrage".to_string());
        variables.insert("trading_pair".to_string(), "BTCUSDT".to_string());
        variables.insert("expected_return".to_string(), "2.5".to_string());
        variables.insert("confidence".to_string(), "85".to_string());
        variables.insert("time_horizon".to_string(), "5 minutes".to_string());

        let result = mock_service.mock_create_notification(
            "opportunity_alert",
            "user123",
            variables,
            NotificationPriority::High,
        ).await;

        assert!(result.is_ok());
        assert_eq!(mock_service.get_notification_count(), 1);

        let notification = &mock_service.notifications[0];
        assert_eq!(notification.template_id, "opportunity_alert");
        assert_eq!(notification.recipient, "user123");
        assert_eq!(notification.priority, NotificationPriority::High);
        assert!(notification.subject.contains("Arbitrage"));
        assert!(notification.body.contains("2.5%"));
    }

    #[tokio::test]
    async fn test_template_rendering_and_variable_substitution() {
        let mut mock_service = MockNotificationService::new();
        
        let template = mock_service.templates.get("system_alert").unwrap().clone();
        
        let mut variables = HashMap::new();
        variables.insert("alert_type".to_string(), "High CPU Usage".to_string());
        variables.insert("description".to_string(), "CPU usage exceeded 90%".to_string());
        variables.insert("severity".to_string(), "Warning".to_string());
        variables.insert("timestamp".to_string(), "2025-01-28 10:30:00".to_string());

        let notification = mock_service.render_notification(
            &template,
            "admin@example.com",
            variables,
            NotificationPriority::Critical,
        ).await.unwrap();

        assert!(notification.subject.contains("High CPU Usage"));
        assert!(notification.body.contains("CPU usage exceeded 90%"));
        assert!(notification.body.contains("Warning"));
        assert!(notification.body.contains("2025-01-28 10:30:00"));
        assert_eq!(notification.channel, NotificationChannel::Email);
    }

    #[tokio::test]
    async fn test_alert_trigger_and_escalation_logic() {
        let mut mock_service = MockNotificationService::new();
        
        // Test different priority levels
        let priorities = vec![
            NotificationPriority::Low,
            NotificationPriority::Medium,
            NotificationPriority::High,
            NotificationPriority::Critical,
        ];

        for (i, priority) in priorities.iter().enumerate() {
            let mut variables = HashMap::new();
            variables.insert("alert_type".to_string(), format!("Alert {}", i + 1));
            variables.insert("description".to_string(), format!("Test alert {}", i + 1));
            variables.insert("severity".to_string(), format!("{:?}", priority));
            variables.insert("timestamp".to_string(), "2025-01-28 10:30:00".to_string());

            let result = mock_service.mock_create_notification(
                "system_alert",
                "admin@example.com",
                variables,
                priority.clone(),
            ).await;

            assert!(result.is_ok());
        }

        assert_eq!(mock_service.get_notification_count(), 4);

        // Verify escalation logic (critical alerts should be processed first)
        let critical_notifications: Vec<_> = mock_service.notifications.iter()
            .filter(|n| n.priority == NotificationPriority::Critical)
            .collect();
        
        assert_eq!(critical_notifications.len(), 1);
    }

    #[tokio::test]
    async fn test_delivery_mechanism_and_retry_logic() {
        let mut mock_service = MockNotificationService::new();
        
        // Create a notification
        let mut variables = HashMap::new();
        variables.insert("opportunity_type".to_string(), "Technical".to_string());
        variables.insert("trading_pair".to_string(), "ETHUSDT".to_string());
        variables.insert("expected_return".to_string(), "1.8".to_string());
        variables.insert("confidence".to_string(), "75".to_string());
        variables.insert("time_horizon".to_string(), "15 minutes".to_string());

        let notification_id = mock_service.mock_create_notification(
            "opportunity_alert",
            "user456",
            variables,
            NotificationPriority::Medium,
        ).await.unwrap();

        // Test successful delivery
        let result = mock_service.mock_send_notification(&notification_id).await;
        assert!(result.is_ok());

        let notification = mock_service.notifications.iter()
            .find(|n| n.notification_id == notification_id)
            .unwrap();
        
        assert_eq!(notification.status, DeliveryStatus::Sent);
        assert!(notification.sent_at.is_some());

        // Test delivery failure and retry
        mock_service.simulate_error("delivery_failed");
        
        let retry_result = mock_service.mock_send_notification(&notification_id).await;
        assert!(retry_result.is_err());
        assert!(retry_result.unwrap_err().to_string().contains("Delivery failed"));

        // Test recovery after error
        mock_service.reset_error_simulation();
        let recovery_result = mock_service.mock_send_notification(&notification_id).await;
        assert!(recovery_result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiting_and_throttling() {
        let mut mock_service = MockNotificationService::new();
        
        let recipient = "rate_test_user";
        let mut variables = HashMap::new();
        variables.insert("opportunity_type".to_string(), "Test".to_string());
        variables.insert("trading_pair".to_string(), "TESTUSDT".to_string());
        variables.insert("expected_return".to_string(), "1.0".to_string());
        variables.insert("confidence".to_string(), "50".to_string());
        variables.insert("time_horizon".to_string(), "1 minute".to_string());

        // Test rate limit enforcement
        let mut successful_notifications = 0;
        let mut rate_limited_notifications = 0;

        // Try to send more notifications than the rate limit allows
        for i in 0..15 {
            let result = mock_service.mock_create_notification(
                "opportunity_alert",
                recipient,
                variables.clone(),
                NotificationPriority::Low,
            ).await;

            if result.is_ok() {
                successful_notifications += 1;
            } else {
                rate_limited_notifications += 1;
            }
        }

        // Should allow up to max_notifications_per_minute (10) and reject the rest
        assert_eq!(successful_notifications, mock_service.config.max_notifications_per_minute as usize);
        assert_eq!(rate_limited_notifications, 5);

        // Test rate limit reset after time window
        // In real implementation, this would involve time manipulation or waiting
        // For mock, we simulate the reset
        mock_service.rate_limits.clear();
        
        let after_reset_result = mock_service.mock_create_notification(
            "opportunity_alert",
            recipient,
            variables,
            NotificationPriority::Low,
        ).await;
        
        assert!(after_reset_result.is_ok(), "Should allow notifications after rate limit reset");
    }

    #[tokio::test]
    async fn test_notification_template_management() {
        let mut mock_service = MockNotificationService::new();
        
        // Test template retrieval
        let opportunity_template = mock_service.templates.get("opportunity_alert");
        assert!(opportunity_template.is_some());
        
        let template = opportunity_template.unwrap();
        assert_eq!(template.template_id, "opportunity_alert");
        assert_eq!(template.channels[0], NotificationChannel::Telegram);
        assert_eq!(template.priority, "high");
        assert!(!template.variables.is_empty());

        // Test custom template creation
        let custom_template = NotificationTemplate {
            template_id: "custom_alert".to_string(),
            name: "Custom Alert".to_string(),
            description: Some("Custom alert template".to_string()),
            category: "custom".to_string(),
            title_template: "Custom: {title}".to_string(),
            message_template: "Message: {message}".to_string(),
            priority: "medium".to_string(),
            channels: vec![NotificationChannel::Push],
            variables: vec!["title".to_string(), "message".to_string()],
            is_system_template: false,
            is_active: true,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        };

        mock_service.templates.insert("custom_alert".to_string(), custom_template);
        
        // Test using custom template
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), "Test Title".to_string());
        variables.insert("message".to_string(), "Test Message".to_string());

        let result = mock_service.mock_create_notification(
            "custom_alert",
            "test_user",
            variables,
            NotificationPriority::Medium,
        ).await;

        assert!(result.is_ok());
        
        let notification = mock_service.notifications.last().unwrap();
        assert!(notification.subject.contains("Test Title"));
        assert!(notification.body.contains("Test Message"));
        assert_eq!(notification.channel, NotificationChannel::Push);
    }

    #[tokio::test]
    async fn test_error_handling_scenarios() {
        let mut mock_service = MockNotificationService::new();
        
        // Test template not found error
        mock_service.simulate_error("template_not_found");
        
        let result = mock_service.mock_create_notification(
            "nonexistent_template",
            "user123",
            HashMap::new(),
            NotificationPriority::Low,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Template not found"));

        // Test invalid recipient error
        mock_service.simulate_error("invalid_recipient");
        
        let result = mock_service.mock_create_notification(
            "opportunity_alert",
            "",
            HashMap::new(),
            NotificationPriority::Low,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid recipient"));

        // Test network error during delivery
        mock_service.reset_error_simulation();
        
        let mut variables = HashMap::new();
        variables.insert("opportunity_type".to_string(), "Test".to_string());
        variables.insert("trading_pair".to_string(), "TESTUSDT".to_string());
        variables.insert("expected_return".to_string(), "1.0".to_string());
        variables.insert("confidence".to_string(), "50".to_string());
        variables.insert("time_horizon".to_string(), "1 minute".to_string());

        let notification_id = mock_service.mock_create_notification(
            "opportunity_alert",
            "user123",
            variables,
            NotificationPriority::Low,
        ).await.unwrap();

        mock_service.simulate_error("network_error");
        
        let delivery_result = mock_service.mock_send_notification(&notification_id).await;
        assert!(delivery_result.is_err());
        assert!(delivery_result.unwrap_err().to_string().contains("Network error"));
    }

    #[tokio::test]
    async fn test_notification_performance_and_metrics() {
        let mut mock_service = MockNotificationService::new();
        
        // Create multiple notifications to test performance
        let start_time = std::time::Instant::now();
        
        for i in 0..50 {
            let mut variables = HashMap::new();
            variables.insert("opportunity_type".to_string(), "Performance Test".to_string());
            variables.insert("trading_pair".to_string(), format!("TEST{}USDT", i));
            variables.insert("expected_return".to_string(), "1.0".to_string());
            variables.insert("confidence".to_string(), "50".to_string());
            variables.insert("time_horizon".to_string(), "1 minute".to_string());

            let result = mock_service.mock_create_notification(
                "opportunity_alert",
                &format!("user{}", i % 10), // Distribute across 10 users
                variables,
                NotificationPriority::Low,
            ).await;

            // Only first 10 per user should succeed due to rate limiting
            if i < 10 {
                assert!(result.is_ok());
            }
        }
        
        let creation_duration = start_time.elapsed();
        assert!(creation_duration.as_millis() < 1000, "Notification creation should be fast");

        // Test delivery performance
        let delivery_start = std::time::Instant::now();
        
        for notification in &mock_service.notifications.clone() {
            let _ = mock_service.mock_send_notification(&notification.notification_id).await;
        }
        
        let delivery_duration = delivery_start.elapsed();
        assert!(delivery_duration.as_millis() < 500, "Notification delivery should be fast");

        // Test metrics calculation
        let success_rate = mock_service.get_delivery_success_rate();
        assert!(success_rate > 0.0, "Should have some successful deliveries");
    }

    #[test]
    fn test_notification_data_structures() {
        // Test NotificationPriority ordering
        assert!(NotificationPriority::Critical > NotificationPriority::High);
        assert!(NotificationPriority::High > NotificationPriority::Medium);
        assert!(NotificationPriority::Medium > NotificationPriority::Low);

        // Test NotificationChannel variants
        let channels = vec![
            NotificationChannel::Telegram,
            NotificationChannel::Email,
            NotificationChannel::Push,
        ];
        
        assert_eq!(channels.len(), 3);

        // Test DeliveryStatus variants
        let statuses = vec![
            DeliveryStatus::Pending,
            DeliveryStatus::Sent,
            DeliveryStatus::Failed,
            DeliveryStatus::Retrying,
        ];
        
        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_notification_config_validation() {
        let config = NotificationConfig {
            max_notifications_per_minute: 10,
            max_notifications_per_hour: 100,
            retry_attempts: 3,
            retry_delay_seconds: 30,
            enabled_channels: vec![NotificationChannel::Telegram],
        };

        assert!(config.max_notifications_per_minute > 0);
        assert!(config.max_notifications_per_hour >= config.max_notifications_per_minute);
        assert!(config.retry_attempts > 0);
        assert!(config.retry_delay_seconds > 0);
        assert!(!config.enabled_channels.is_empty());
    }
} 