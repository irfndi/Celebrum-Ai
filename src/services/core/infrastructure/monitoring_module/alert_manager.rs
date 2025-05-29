// AlertManager - Intelligent alerting system with smart alerting and escalation policies
// Part of Monitoring Module replacing monitoring_observability.rs

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use worker::kv::KvStore;

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Emergency,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Error => "error",
            AlertSeverity::Critical => "critical",
            AlertSeverity::Emergency => "emergency",
        }
    }

    pub fn priority_score(&self) -> u8 {
        match self {
            AlertSeverity::Info => 1,
            AlertSeverity::Warning => 2,
            AlertSeverity::Error => 3,
            AlertSeverity::Critical => 4,
            AlertSeverity::Emergency => 5,
        }
    }

    pub fn escalation_timeout_seconds(&self) -> u64 {
        match self {
            AlertSeverity::Info => 3600,    // 1 hour
            AlertSeverity::Warning => 1800, // 30 minutes
            AlertSeverity::Error => 900,    // 15 minutes
            AlertSeverity::Critical => 300, // 5 minutes
            AlertSeverity::Emergency => 60, // 1 minute
        }
    }
}

/// Alert status tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
    Escalated,
    Expired,
}

impl AlertStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AlertStatus::Active => "active",
            AlertStatus::Acknowledged => "acknowledged",
            AlertStatus::Resolved => "resolved",
            AlertStatus::Suppressed => "suppressed",
            AlertStatus::Escalated => "escalated",
            AlertStatus::Expired => "expired",
        }
    }

    pub fn is_actionable(&self) -> bool {
        matches!(self, AlertStatus::Active | AlertStatus::Escalated)
    }
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub component: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub evaluation_interval_seconds: u64,
    pub enabled: bool,
    pub tags: HashMap<String, String>,
    pub notification_channels: Vec<String>,
    pub escalation_policy: Option<String>,
    pub suppression_rules: Vec<SuppressionRule>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl AlertRule {
    pub fn new(
        name: String,
        component: String,
        metric_name: String,
        condition: AlertCondition,
        severity: AlertSeverity,
        threshold: f64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            component,
            metric_name,
            condition,
            severity,
            threshold,
            duration_seconds: 300,           // 5 minutes default
            evaluation_interval_seconds: 60, // 1 minute default
            enabled: true,
            tags: HashMap::new(),
            notification_channels: Vec::new(),
            escalation_policy: None,
            suppression_rules: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_duration(mut self, duration_seconds: u64) -> Self {
        self.duration_seconds = duration_seconds;
        self
    }

    pub fn with_notification_channel(mut self, channel: String) -> Self {
        self.notification_channels.push(channel);
        self
    }

    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    pub fn evaluate(&self, metric_value: f64) -> bool {
        match self.condition {
            AlertCondition::GreaterThan => metric_value > self.threshold,
            AlertCondition::LessThan => metric_value < self.threshold,
            AlertCondition::Equal => (metric_value - self.threshold).abs() < f64::EPSILON,
            AlertCondition::NotEqual => (metric_value - self.threshold).abs() >= f64::EPSILON,
            AlertCondition::GreaterThanOrEqual => metric_value >= self.threshold,
            AlertCondition::LessThanOrEqual => metric_value <= self.threshold,
        }
    }
}

/// Alert condition types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Suppression rule for reducing alert noise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionRule {
    pub id: String,
    pub name: String,
    pub condition: String, // Expression to evaluate
    pub duration_seconds: u64,
    pub enabled: bool,
}

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub component: String,
    pub metric_name: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub message: String,
    pub current_value: f64,
    pub threshold: f64,
    pub condition: AlertCondition,
    pub tags: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub started_at: u64,
    pub acknowledged_at: Option<u64>,
    pub resolved_at: Option<u64>,
    pub escalated_at: Option<u64>,
    pub last_notification_at: Option<u64>,
    pub notification_count: u32,
    pub escalation_level: u8,
    pub correlation_id: Option<String>,
}

impl Alert {
    pub fn new(rule: &AlertRule, current_value: f64, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            component: rule.component.clone(),
            metric_name: rule.metric_name.clone(),
            severity: rule.severity.clone(),
            status: AlertStatus::Active,
            message,
            current_value,
            threshold: rule.threshold,
            condition: rule.condition.clone(),
            tags: rule.tags.clone(),
            annotations: HashMap::new(),
            started_at: chrono::Utc::now().timestamp_millis() as u64,
            acknowledged_at: None,
            resolved_at: None,
            escalated_at: None,
            last_notification_at: None,
            notification_count: 0,
            escalation_level: 0,
            correlation_id: None,
        }
    }

    pub fn acknowledge(&mut self) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_at = Some(chrono::Utc::now().timestamp_millis() as u64);
    }

    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(chrono::Utc::now().timestamp_millis() as u64);
    }

    pub fn escalate(&mut self) {
        self.status = AlertStatus::Escalated;
        self.escalated_at = Some(chrono::Utc::now().timestamp_millis() as u64);
        self.escalation_level += 1;
    }

    pub fn suppress(&mut self) {
        self.status = AlertStatus::Suppressed;
    }

    pub fn add_annotation(&mut self, key: String, value: String) {
        self.annotations.insert(key, value);
    }

    pub fn duration_seconds(&self) -> u64 {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        (now - self.started_at) / 1000
    }

    pub fn should_escalate(&self, escalation_timeout: u64) -> bool {
        if self.status != AlertStatus::Active {
            return false;
        }

        let now = chrono::Utc::now().timestamp_millis() as u64;
        let time_since_start = (now - self.started_at) / 1000;

        time_since_start >= escalation_timeout
    }
}

/// Alert manager health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHealth {
    pub is_healthy: bool,
    pub active_alerts_count: u64,
    pub critical_alerts_count: u64,
    pub alert_processing_rate_per_second: f64,
    pub notification_success_rate_percent: f32,
    pub escalation_rate_percent: f32,
    pub suppression_rate_percent: f32,
    pub avg_alert_resolution_time_seconds: f64,
    pub last_alert_timestamp: u64,
    pub rules_count: u64,
    pub enabled_rules_count: u64,
    pub kv_store_available: bool,
    pub last_error: Option<String>,
}

impl Default for AlertHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            active_alerts_count: 0,
            critical_alerts_count: 0,
            alert_processing_rate_per_second: 0.0,
            notification_success_rate_percent: 0.0,
            escalation_rate_percent: 0.0,
            suppression_rate_percent: 0.0,
            avg_alert_resolution_time_seconds: 0.0,
            last_alert_timestamp: 0,
            rules_count: 0,
            enabled_rules_count: 0,
            kv_store_available: false,
            last_error: None,
        }
    }
}

/// Alert manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    pub evaluation_interval_seconds: u64,
    pub max_alerts_in_memory: usize,
    pub alert_retention_days: u32,
    pub enable_smart_grouping: bool,
    pub grouping_window_seconds: u64,
    pub enable_escalation: bool,
    pub default_escalation_timeout_seconds: u64,
    pub enable_suppression: bool,
    pub enable_correlation: bool,
    pub correlation_window_seconds: u64,
    pub enable_kv_storage: bool,
    pub kv_key_prefix: String,
    pub notification_channels: Vec<NotificationChannel>,
    pub escalation_policies: Vec<EscalationPolicy>,
    pub enable_alert_analytics: bool,
    pub max_notification_retries: u32,
    pub notification_retry_delay_seconds: u64,
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            evaluation_interval_seconds: 60,
            max_alerts_in_memory: 10000,
            alert_retention_days: 30,
            enable_smart_grouping: true,
            grouping_window_seconds: 300, // 5 minutes
            enable_escalation: true,
            default_escalation_timeout_seconds: 900, // 15 minutes
            enable_suppression: true,
            enable_correlation: true,
            correlation_window_seconds: 600, // 10 minutes
            enable_kv_storage: true,
            kv_key_prefix: "alerts:".to_string(),
            notification_channels: Vec::new(),
            escalation_policies: Vec::new(),
            enable_alert_analytics: true,
            max_notification_retries: 3,
            notification_retry_delay_seconds: 60,
        }
    }
}

impl AlertManagerConfig {
    pub fn high_performance() -> Self {
        Self {
            evaluation_interval_seconds: 30,
            max_alerts_in_memory: 50000,
            grouping_window_seconds: 120,
            correlation_window_seconds: 300,
            max_notification_retries: 5,
            notification_retry_delay_seconds: 30,
            ..Default::default()
        }
    }

    pub fn high_reliability() -> Self {
        Self {
            evaluation_interval_seconds: 15,
            max_alerts_in_memory: 100000,
            alert_retention_days: 90,
            grouping_window_seconds: 60,
            default_escalation_timeout_seconds: 300,
            correlation_window_seconds: 180,
            max_notification_retries: 10,
            notification_retry_delay_seconds: 15,
            ..Default::default()
        }
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.evaluation_interval_seconds == 0 {
            return Err(ArbitrageError::configuration_error(
                "Evaluation interval must be greater than 0".to_string(),
            ));
        }

        if self.max_alerts_in_memory == 0 {
            return Err(ArbitrageError::configuration_error(
                "Max alerts in memory must be greater than 0".to_string(),
            ));
        }

        if self.alert_retention_days == 0 {
            return Err(ArbitrageError::configuration_error(
                "Alert retention days must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String, // "email", "slack", "webhook", "telegram"
    pub endpoint: String,
    pub enabled: bool,
    pub severity_filter: Vec<AlertSeverity>,
    pub rate_limit_per_hour: u32,
    pub retry_config: RetryConfig,
}

/// Escalation policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub id: String,
    pub name: String,
    pub steps: Vec<EscalationStep>,
    pub enabled: bool,
}

/// Escalation step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    pub level: u8,
    pub timeout_seconds: u64,
    pub notification_channels: Vec<String>,
    pub actions: Vec<String>,
}

/// Retry configuration for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_seconds: u64,
    pub max_delay_seconds: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 60,
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
        }
    }
}

/// Intelligent alert manager with smart alerting and escalation policies
pub struct AlertManager {
    config: AlertManagerConfig,
    logger: crate::utils::logger::Logger,
    kv_store: KvStore,

    // Alert storage
    alert_rules: Arc<Mutex<HashMap<String, AlertRule>>>,
    active_alerts: Arc<Mutex<HashMap<String, Alert>>>,
    alert_history: Arc<Mutex<Vec<Alert>>>,

    // Grouping and correlation
    alert_groups: Arc<Mutex<HashMap<String, Vec<String>>>>,
    correlation_map: Arc<Mutex<HashMap<String, String>>>,

    // Health and performance tracking
    health: Arc<Mutex<AlertHealth>>,
    alert_count: Arc<Mutex<u64>>,
    notification_count: Arc<Mutex<u64>>,

    // Performance metrics
    startup_time: u64,
}

impl AlertManager {
    /// Create new alert manager with intelligent alerting
    pub async fn new(
        config: AlertManagerConfig,
        kv_store: KvStore,
        _env: &worker::Env,
    ) -> ArbitrageResult<Self> {
        let startup_start = chrono::Utc::now().timestamp_millis() as u64;

        // Validate configuration
        config.validate()?;

        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
        logger.info("Initializing AlertManager with intelligent alerting");

        let startup_time = chrono::Utc::now().timestamp_millis() as u64 - startup_start;

        logger.info(&format!(
            "AlertManager initialized successfully in {}ms",
            startup_time
        ));

        Ok(Self {
            config,
            logger,
            kv_store,
            alert_rules: Arc::new(Mutex::new(HashMap::new())),
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
            alert_history: Arc::new(Mutex::new(Vec::new())),
            alert_groups: Arc::new(Mutex::new(HashMap::new())),
            correlation_map: Arc::new(Mutex::new(HashMap::new())),
            health: Arc::new(Mutex::new(AlertHealth::default())),
            alert_count: Arc::new(Mutex::new(0)),
            notification_count: Arc::new(Mutex::new(0)),
            startup_time,
        })
    }

    /// Add alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> ArbitrageResult<()> {
        let mut rules = self.alert_rules.lock().unwrap();
        rules.insert(rule.id.clone(), rule.clone());

        self.logger
            .info(&format!("Added alert rule: {} ({})", rule.name, rule.id));

        // Store in KV if enabled
        if self.config.enable_kv_storage {
            self.store_rule_in_kv(&rule).await?;
        }

        Ok(())
    }

    /// Remove alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> ArbitrageResult<bool> {
        let mut rules = self.alert_rules.lock().unwrap();
        let removed = rules.remove(rule_id).is_some();

        if removed {
            self.logger
                .info(&format!("Removed alert rule: {}", rule_id));
        }

        Ok(removed)
    }

    /// Evaluate metric against alert rules
    pub async fn evaluate_metric(
        &self,
        component: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> ArbitrageResult<Vec<Alert>> {
        let rules = self.alert_rules.lock().unwrap();
        let mut triggered_alerts = Vec::new();

        for rule in rules.values() {
            if rule.enabled
                && rule.component == component
                && rule.metric_name == metric_name
                && rule.evaluate(metric_value)
            {
                let message = format!(
                    "Alert triggered: {} {} {} (current: {}, threshold: {})",
                    metric_name,
                    match rule.condition {
                        AlertCondition::GreaterThan => ">",
                        AlertCondition::LessThan => "<",
                        AlertCondition::Equal => "==",
                        AlertCondition::NotEqual => "!=",
                        AlertCondition::GreaterThanOrEqual => ">=",
                        AlertCondition::LessThanOrEqual => "<=",
                    },
                    rule.threshold,
                    metric_value,
                    rule.threshold
                );

                let alert = Alert::new(rule, metric_value, message);
                triggered_alerts.push(alert);
            }
        }

        // Process triggered alerts
        for alert in &triggered_alerts {
            self.process_alert(alert.clone()).await?;
        }

        Ok(triggered_alerts)
    }

    /// Process alert (grouping, correlation, notification)
    async fn process_alert(&self, mut alert: Alert) -> ArbitrageResult<()> {
        // Check for correlation
        if self.config.enable_correlation {
            if let Some(correlation_id) = self.find_correlation(&alert).await {
                alert.correlation_id = Some(correlation_id);
            }
        }

        // Check for suppression
        if self.config.enable_suppression && self.should_suppress(&alert).await {
            alert.suppress();
            self.logger.info(&format!("Alert suppressed: {}", alert.id));
            return Ok(());
        }

        // Add to active alerts
        {
            let mut active_alerts = self.active_alerts.lock().unwrap();
            active_alerts.insert(alert.id.clone(), alert.clone());

            // Check memory limits
            if active_alerts.len() > self.config.max_alerts_in_memory {
                self.cleanup_old_alerts().await?;
            }
        }

        // Group alerts if enabled
        if self.config.enable_smart_grouping {
            self.group_alert(&alert).await;
        }

        // Send notifications
        self.send_notifications(&alert).await?;

        // Update statistics
        {
            let mut count = self.alert_count.lock().unwrap();
            *count += 1;
        }

        self.logger.info(&format!(
            "Processed alert: {} ({})",
            alert.rule_name, alert.id
        ));

        Ok(())
    }

    /// Find correlation for alert
    async fn find_correlation(&self, alert: &Alert) -> Option<String> {
        let correlation_map = self.correlation_map.lock().unwrap();

        // Simple correlation based on component and time window
        let correlation_key = format!("{}:{}", alert.component, alert.severity.as_str());

        if let Some(existing_correlation) = correlation_map.get(&correlation_key) {
            Some(existing_correlation.clone())
        } else {
            None
        }
    }

    /// Check if alert should be suppressed
    async fn should_suppress(&self, alert: &Alert) -> bool {
        let rules = self.alert_rules.lock().unwrap();

        if let Some(rule) = rules.get(&alert.rule_id) {
            for suppression_rule in &rule.suppression_rules {
                if suppression_rule.enabled {
                    // Simple suppression logic - in practice, this would evaluate the condition
                    if alert.severity == AlertSeverity::Info {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Group alert with similar alerts
    async fn group_alert(&self, alert: &Alert) {
        let group_key = format!(
            "{}:{}:{}",
            alert.component,
            alert.metric_name,
            alert.severity.as_str()
        );

        let mut groups = self.alert_groups.lock().unwrap();
        let group = groups.entry(group_key).or_insert_with(Vec::new);
        group.push(alert.id.clone());
    }

    /// Send notifications for alert
    async fn send_notifications(&self, alert: &Alert) -> ArbitrageResult<()> {
        for channel in &self.config.notification_channels {
            if channel.enabled && channel.severity_filter.contains(&alert.severity) {
                if let Err(e) = self.send_notification(channel, alert).await {
                    self.logger.error(&format!(
                        "Failed to send notification via {}: {}",
                        channel.name, e
                    ));
                } else {
                    let mut count = self.notification_count.lock().unwrap();
                    *count += 1;
                }
            }
        }

        Ok(())
    }

    /// Send notification via specific channel
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        alert: &Alert,
    ) -> ArbitrageResult<()> {
        // In a real implementation, this would send actual notifications
        self.logger.info(&format!(
            "Sending {} notification for alert {} via {}",
            alert.severity.as_str(),
            alert.id,
            channel.name
        ));

        // Simulate notification sending
        Ok(())
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> ArbitrageResult<bool> {
        let mut active_alerts = self.active_alerts.lock().unwrap();

        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.acknowledge();
            self.logger
                .info(&format!("Alert acknowledged: {}", alert_id));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) -> ArbitrageResult<bool> {
        let mut active_alerts = self.active_alerts.lock().unwrap();

        if let Some(mut alert) = active_alerts.remove(alert_id) {
            alert.resolve();

            // Move to history
            {
                let mut history = self.alert_history.lock().unwrap();
                history.push(alert);
            }

            self.logger.info(&format!("Alert resolved: {}", alert_id));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let active_alerts = self.active_alerts.lock().unwrap();
        active_alerts.values().cloned().collect()
    }

    /// Get alerts by severity
    pub async fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<Alert> {
        let active_alerts = self.active_alerts.lock().unwrap();
        active_alerts
            .values()
            .filter(|alert| alert.severity == severity)
            .cloned()
            .collect()
    }

    /// Store alert rule in KV
    async fn store_rule_in_kv(&self, rule: &AlertRule) -> ArbitrageResult<()> {
        let kv_key = format!("{}rule:{}", self.config.kv_key_prefix, rule.id);
        let serialized = serde_json::to_string(rule)
            .map_err(|e| ArbitrageError::serialization_error(e.to_string()))?;

        self.kv_store
            .put(&kv_key, serialized)
            .map_err(|e| ArbitrageError::storage_error(format!("KV put failed: {:?}", e)))?
            .execute()
            .await
            .map_err(|e| ArbitrageError::storage_error(format!("KV execute failed: {:?}", e)))?;

        Ok(())
    }

    /// Cleanup old alerts
    async fn cleanup_old_alerts(&self) -> ArbitrageResult<u64> {
        let mut active_alerts = self.active_alerts.lock().unwrap();
        let mut cleaned_count = 0;

        let cutoff_time = chrono::Utc::now().timestamp_millis() as u64
            - (self.config.alert_retention_days as u64 * 24 * 3600 * 1000);

        active_alerts.retain(|_, alert| {
            if alert.started_at < cutoff_time {
                cleaned_count += 1;
                false
            } else {
                true
            }
        });

        if cleaned_count > 0 {
            self.logger
                .info(&format!("Cleaned up {} old alerts", cleaned_count));
        }

        Ok(cleaned_count)
    }

    /// Get alert manager health
    pub async fn get_health(&self) -> AlertHealth {
        let mut health = self.health.lock().unwrap();

        // Update health metrics
        let active_alerts = self.active_alerts.lock().unwrap();
        let rules = self.alert_rules.lock().unwrap();

        let active_count = active_alerts.len() as u64;
        let critical_count = active_alerts
            .values()
            .filter(|alert| {
                matches!(
                    alert.severity,
                    AlertSeverity::Critical | AlertSeverity::Emergency
                )
            })
            .count() as u64;

        let rules_count = rules.len() as u64;
        let enabled_rules_count = rules.values().filter(|rule| rule.enabled).count() as u64;

        // Calculate processing rate
        let alert_count = *self.alert_count.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let time_since_startup = (now - self.startup_time) / 1000; // seconds
        let processing_rate = if time_since_startup > 0 {
            alert_count as f64 / time_since_startup as f64
        } else {
            0.0
        };

        // Determine overall health
        let is_healthy = critical_count == 0 && active_count < 1000;

        health.is_healthy = is_healthy;
        health.active_alerts_count = active_count;
        health.critical_alerts_count = critical_count;
        health.alert_processing_rate_per_second = processing_rate;
        health.rules_count = rules_count;
        health.enabled_rules_count = enabled_rules_count;
        health.kv_store_available = self.config.enable_kv_storage;

        health.clone()
    }

    /// Perform health check
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        let health = self.get_health().await;
        Ok(health.is_healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Emergency > AlertSeverity::Critical);
        assert!(AlertSeverity::Critical > AlertSeverity::Error);
        assert!(AlertSeverity::Error > AlertSeverity::Warning);
        assert!(AlertSeverity::Warning > AlertSeverity::Info);
    }

    #[test]
    fn test_alert_rule_evaluation() {
        let rule = AlertRule::new(
            "CPU High".to_string(),
            "system".to_string(),
            "cpu_usage".to_string(),
            AlertCondition::GreaterThan,
            AlertSeverity::Warning,
            80.0,
        );

        assert!(rule.evaluate(85.0));
        assert!(!rule.evaluate(75.0));
    }

    #[test]
    fn test_alert_status_transitions() {
        let rule = AlertRule::new(
            "Test Alert".to_string(),
            "test".to_string(),
            "test_metric".to_string(),
            AlertCondition::GreaterThan,
            AlertSeverity::Error,
            100.0,
        );

        let mut alert = Alert::new(&rule, 150.0, "Test message".to_string());

        assert_eq!(alert.status, AlertStatus::Active);
        assert!(alert.status.is_actionable());

        alert.acknowledge();
        assert_eq!(alert.status, AlertStatus::Acknowledged);
        assert!(!alert.status.is_actionable());

        alert.resolve();
        assert_eq!(alert.status, AlertStatus::Resolved);
    }

    #[test]
    fn test_alert_manager_config_validation() {
        let mut config = AlertManagerConfig::default();
        assert!(config.validate().is_ok());

        config.evaluation_interval_seconds = 0;
        assert!(config.validate().is_err());
    }
}
