// src/services/core/admin/audit.rs

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::{kv::KvStore, Env};

/// Configuration for Audit Service retention periods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub user_action_ttl_seconds: u64,
    pub system_event_ttl_seconds: u64,
    pub security_event_ttl_seconds: u64,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            user_action_ttl_seconds: 365 * 24 * 60 * 60,  // 1 year
            system_event_ttl_seconds: 365 * 24 * 60 * 60, // 1 year
            security_event_ttl_seconds: 2 * 365 * 24 * 60 * 60, // 2 years
        }
    }
}

/// Audit event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Security severity levels for security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    UserAction(UserAuditAction),
    SystemEvent(SystemAuditEvent),
    SecurityEvent(SecurityAuditEvent),
}

/// Main audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub timestamp: u64,
    pub details: Option<String>,
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub severity: AuditSeverity,
}

/// System configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub max_concurrent_users: u32,
    pub rate_limit_per_minute: u32,
    pub maintenance_mode: bool,
    pub feature_flags: HashMap<String, bool>,
    pub api_version: String,
    pub last_updated: u64,
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub active_connections: u32,
    pub requests_per_minute: u32,
    pub error_rate_percent: f64,
    pub uptime_seconds: u64,
    pub last_updated: u64,
}

/// Audit service for tracking all system activities and security events
#[derive(Clone)]
pub struct AuditService {
    kv_store: KvStore,
    #[allow(dead_code)] // Will be used for environment configuration
    env: Env,
    config: AuditConfig,
}

impl AuditService {
    pub fn new(env: Env, kv_store: KvStore, config: AuditConfig) -> Self {
        Self {
            kv_store,
            env,
            config,
        }
    }

    /// Log user action for audit trail
    pub async fn log_user_action(
        &self,
        user_id: &str,
        action: UserAuditAction,
        details: Option<String>,
    ) -> ArbitrageResult<()> {
        let audit_event = AuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: AuditEventType::UserAction(action),
            user_id: Some(user_id.to_string()),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            details,
            metadata: serde_json::Value::Null,
            ip_address: None,
            user_agent: None,
            session_id: None,
            severity: AuditSeverity::Info,
        };

        let audit_key = format!(
            "audit_user_action:{}:{}",
            audit_event.timestamp, audit_event.event_id
        );
        let audit_data = serde_json::to_string(&audit_event).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize audit action: {}", e))
        })?;

        self.kv_store
            .put(&audit_key, &audit_data)?
            .expiration_ttl(self.config.user_action_ttl_seconds)
            .execute()
            .await?;

        Ok(())
    }

    /// Log system event for audit trail
    pub async fn log_system_event(&self, event: SystemAuditEvent) -> ArbitrageResult<()> {
        let audit_event = AuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: AuditEventType::SystemEvent(event),
            user_id: None,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            details: None,
            metadata: serde_json::Value::Null,
            ip_address: None,
            user_agent: None,
            session_id: None,
            severity: AuditSeverity::Info,
        };

        let audit_key = format!(
            "audit_system_event:{}:{}",
            audit_event.timestamp, audit_event.event_id
        );
        let audit_data = serde_json::to_string(&audit_event).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize audit event: {}", e))
        })?;

        self.kv_store
            .put(&audit_key, &audit_data)?
            .expiration_ttl(self.config.system_event_ttl_seconds)
            .execute()
            .await?;

        Ok(())
    }

    /// Log security event for audit trail
    pub async fn log_security_event(&self, event: SecurityAuditEvent) -> ArbitrageResult<()> {
        let audit_event = AuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: AuditEventType::SecurityEvent(event),
            user_id: None,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            details: None,
            metadata: serde_json::Value::Null,
            ip_address: None,
            user_agent: None,
            session_id: None,
            severity: AuditSeverity::Critical,
        };

        let audit_key = format!(
            "audit_security_event:{}:{}",
            audit_event.timestamp, audit_event.event_id
        );
        let security_data = serde_json::to_string(&audit_event).map_err(|e| {
            ArbitrageError::serialization_error(format!(
                "Failed to serialize security event: {}",
                e
            ))
        })?;

        self.kv_store
            .put(&audit_key, &security_data)?
            .expiration_ttl(self.config.security_event_ttl_seconds)
            .execute()
            .await?;

        Ok(())
    }

    /// Get recent audit events
    pub async fn get_recent_events(&self, limit: u32) -> ArbitrageResult<Vec<AuditEvent>> {
        let mut events = Vec::new();
        let limit = limit.min(1000);

        // Get recent events (simplified implementation)
        for i in 0..limit {
            let audit_key = format!(
                "audit_user_action:{}:event_{}",
                chrono::Utc::now().timestamp_millis() as u64 - (i as u64 * 60000),
                i
            );

            if let Some(audit_data) = self.kv_store.get(&audit_key).text().await? {
                if let Ok(event) = serde_json::from_str::<AuditEvent>(&audit_data) {
                    events.push(event);
                }
            }
        }

        // Sort by timestamp (most recent first)
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(events)
    }

    /// Health check for audit service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        // Test KV store connectivity
        let test_key = "audit_health_check";
        let test_value = "test";

        match self.kv_store.put(test_key, test_value) {
            Ok(put_builder) => match put_builder.execute().await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            },
            Err(_) => Ok(false),
        }
    }
}

/// User audit action for tracking user activities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAuditAction {
    pub action_id: String,
    pub user_id: String,
    pub action_type: String, // "login", "logout", "profile_update", "api_key_added", etc.
    pub description: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl UserAuditAction {
    pub fn new(user_id: String, action_type: String, description: String) -> Self {
        Self {
            action_id: format!("action_{}", uuid::Uuid::new_v4()),
            user_id,
            action_type,
            description,
            ip_address: None,
            user_agent: None,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            success: true,
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_request_info(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// System audit event for tracking system activities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemAuditEvent {
    pub event_id: String,
    pub event_type: String, // "service_start", "service_stop", "configuration_change", etc.
    pub description: String,
    pub service_name: String,
    pub timestamp: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl SystemAuditEvent {
    pub fn new(event_type: String, description: String, service_name: String) -> Self {
        Self {
            event_id: format!("event_{}", uuid::Uuid::new_v4()),
            event_type,
            description,
            service_name,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            success: true,
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Security audit event for tracking security-related activities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAuditEvent {
    pub event_id: String,
    pub event_type: String, // "failed_login", "suspicious_activity", "unauthorized_access", etc.
    pub description: String,
    pub severity: SecuritySeverity,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

impl SecurityAuditEvent {
    pub fn new(event_type: String, description: String, severity: SecuritySeverity) -> Self {
        Self {
            event_id: format!("security_{}", uuid::Uuid::new_v4()),
            event_type,
            description,
            severity,
            user_id: None,
            ip_address: None,
            user_agent: None,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            metadata: HashMap::new(),
        }
    }

    pub fn with_user_info(
        mut self,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.user_id = user_id;
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Daily audit summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyAuditSummary {
    pub date: String,
    pub total_user_actions: u32,
    pub total_system_events: u32,
    pub total_security_events: u32,
    pub login_attempts: u32,
    pub logout_events: u32,
    pub profile_updates: u32,
    pub api_key_changes: u32,
    pub subscription_changes: u32,
    pub service_events: u32,
    pub configuration_changes: u32,
    pub maintenance_events: u32,
    pub error_events: u32,
    pub other_actions: u32,
    pub other_events: u32,
}

impl DailyAuditSummary {
    pub fn new(date: String) -> Self {
        Self {
            date,
            total_user_actions: 0,
            total_system_events: 0,
            total_security_events: 0,
            login_attempts: 0,
            logout_events: 0,
            profile_updates: 0,
            api_key_changes: 0,
            subscription_changes: 0,
            service_events: 0,
            configuration_changes: 0,
            maintenance_events: 0,
            error_events: 0,
            other_actions: 0,
            other_events: 0,
        }
    }
}

/// Audit search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSearchCriteria {
    pub user_id: Option<String>,
    pub start_date: Option<u64>,
    pub end_date: Option<u64>,
    pub action_types: Option<Vec<String>>,
    pub include_user_actions: bool,
    pub include_system_events: bool,
    pub include_security_events: bool,
    pub limit: Option<u32>,
}

/// Audit search results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSearchResults {
    pub user_actions: Vec<UserAuditAction>,
    pub system_events: Vec<SystemAuditEvent>,
    pub security_events: Vec<SecurityAuditEvent>,
}

/// Security alert for critical events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAlert {
    pub alert_id: String,
    pub event_id: String,
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: u64,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<u64>,
}
