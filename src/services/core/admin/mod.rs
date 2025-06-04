pub mod audit;
pub mod monitoring;
pub mod system_config;
pub mod user_management;

use crate::services::core::admin::audit::{AuditEvent, UserAuditAction};
use crate::services::core::admin::monitoring::SystemHealth;
use crate::services::core::admin::system_config::SystemConfig;
use crate::services::core::infrastructure::D1Service;
use crate::types::*;
use crate::utils::ArbitrageResult;
use std::sync::Arc;
use worker::Env;

// Re-export all admin services
pub use audit::AuditService;
pub use monitoring::MonitoringService;
pub use system_config::SystemConfigService;
pub use user_management::UserManagementService;

/// Comprehensive System Administration Service
/// Provides unified interface for all admin functionality
#[derive(Clone)]
pub struct AdminService {
    pub user_management: Arc<UserManagementService>,
    pub system_config: Arc<SystemConfigService>,
    pub monitoring: Arc<MonitoringService>,
    pub audit: Arc<AuditService>,
}

impl AdminService {
    /// Create new AdminService with all sub-services
    pub fn new(
        env: &Env,
        kv_store: worker::kv::KvStore,
        _d1_database: D1Service, // Not used yet, but reserved for future database operations
    ) -> Self {
        // Initialize all sub-services (all are sync constructors)
        let user_management = Arc::new(UserManagementService::new(env.clone(), kv_store.clone()));

        let system_config = Arc::new(SystemConfigService::new(env.clone(), kv_store.clone()));

        let monitoring = Arc::new(MonitoringService::new(env.clone(), kv_store.clone()));

        let audit = Arc::new(AuditService::new(env.clone(), kv_store.clone()));

        Self {
            user_management,
            system_config,
            monitoring,
            audit,
        }
    }

    /// Get user management service
    pub fn get_user_management(&self) -> Arc<UserManagementService> {
        self.user_management.clone()
    }

    /// Get system configuration service
    pub fn get_system_config(&self) -> Arc<SystemConfigService> {
        self.system_config.clone()
    }

    /// Get monitoring service
    pub fn get_monitoring(&self) -> Arc<MonitoringService> {
        self.monitoring.clone()
    }

    /// Get audit service
    pub fn get_audit(&self) -> Arc<AuditService> {
        self.audit.clone()
    }

    /// Validate admin permissions for a user
    pub async fn validate_admin_permissions(&self, user_id: &str) -> ArbitrageResult<bool> {
        // Get user profile to check role
        match self.user_management.get_user_by_id(user_id).await {
            Ok(Some(profile)) => {
                let role = profile.get_user_role();
                Ok(matches!(role, UserRole::SuperAdmin | UserRole::Admin))
            }
            Ok(None) => Ok(false), // User not found
            Err(_) => Ok(false),   // If error, deny access
        }
    }

    /// Perform comprehensive health check of all admin services
    pub async fn health_check(&self) -> ArbitrageResult<AdminHealthStatus> {
        let user_mgmt_healthy = self.user_management.health_check().await?;
        let system_config_healthy = self.system_config.health_check().await?;
        let monitoring_healthy = self.monitoring.health_check().await?;
        let audit_healthy = self.audit.health_check().await?;

        let overall_healthy =
            user_mgmt_healthy && system_config_healthy && monitoring_healthy && audit_healthy;

        Ok(AdminHealthStatus {
            overall_healthy,
            user_management_healthy: user_mgmt_healthy,
            system_config_healthy: system_config_healthy,
            monitoring_healthy: monitoring_healthy,
            audit_healthy: audit_healthy,
            last_check: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Get comprehensive admin dashboard data
    pub async fn get_dashboard_data(
        &self,
        admin_user_id: &str,
    ) -> ArbitrageResult<AdminDashboardData> {
        // Verify admin permissions
        if !self.user_management.is_super_admin(admin_user_id).await? {
            return Err(crate::utils::ArbitrageError::authorization_error(
                "Admin access required",
            ));
        }

        // Collect data from all services
        let user_stats = self.user_management.get_user_statistics().await?;
        let system_health = self.monitoring.get_system_health().await?;
        let recent_audit_events = self.audit.get_recent_events(50).await?;
        let system_config = self.system_config.get_current_config().await?;

        Ok(AdminDashboardData {
            user_statistics: user_stats,
            system_health,
            recent_audit_events,
            system_configuration: system_config,
            generated_at: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Execute admin action with full audit logging
    pub async fn execute_admin_action(
        &self,
        admin_user_id: &str,
        action: AdminAction,
    ) -> ArbitrageResult<AdminActionResult> {
        // Validate admin permissions
        if !self.validate_admin_permissions(admin_user_id).await? {
            return Err(ArbitrageError::Unauthorized(
                "Insufficient admin permissions".to_string(),
            ));
        }

        // Clone action for logging
        let action_clone = action.clone();

        // Log the action attempt
        let audit_action = UserAuditAction::new(
            admin_user_id.to_string(),
            "admin_action_attempted".to_string(),
            format!("Admin action attempted: {:?}", action_clone),
        );
        self.audit
            .log_user_action(
                admin_user_id,
                audit_action,
                Some(format!("Action: {:?}", action_clone)),
            )
            .await?;

        // Execute the action based on type
        let result = match action {
            AdminAction::CreateUser { user_data } => {
                let user_profile = self.user_management.create_user(user_data).await?;
                AdminActionResult::UserCreated {
                    user_id: user_profile.user_id,
                }
            }
            AdminAction::UpdateUserAccess {
                user_id,
                access_level,
            } => {
                self.user_management
                    .update_user_access_level(&user_id, access_level)
                    .await?;
                AdminActionResult::UserAccessUpdated { user_id }
            }
            AdminAction::UpdateSystemConfig {
                config_key,
                config_value,
            } => {
                self.system_config
                    .update_config(&config_key, config_value)
                    .await?;
                AdminActionResult::SystemConfigUpdated { config_key }
            }
            AdminAction::EnableMaintenanceMode => {
                self.system_config.enable_maintenance_mode().await?;
                AdminActionResult::MaintenanceModeEnabled
            }
            AdminAction::DisableMaintenanceMode => {
                self.system_config.disable_maintenance_mode().await?;
                AdminActionResult::MaintenanceModeDisabled
            }
        };

        // Log successful action
        self.audit
            .log_user_action(
                admin_user_id,
                UserAuditAction::new(
                    admin_user_id.to_string(),
                    "admin_action_completed".to_string(),
                    format!("Action: {:?}, Result: {:?}", action_clone, result),
                ),
                Some(format!("Action: {:?}, Result: {:?}", action_clone, result)),
            )
            .await?;

        Ok(result)
    }
}

/// Admin health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdminHealthStatus {
    pub overall_healthy: bool,
    pub user_management_healthy: bool,
    pub system_config_healthy: bool,
    pub monitoring_healthy: bool,
    pub audit_healthy: bool,
    pub last_check: u64,
}

/// Admin dashboard data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdminDashboardData {
    pub user_statistics: UserStatistics,
    pub system_health: SystemHealth,
    pub recent_audit_events: Vec<AuditEvent>,
    pub system_configuration: SystemConfig,
    pub generated_at: u64,
}

/// Admin actions that can be executed
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AdminAction {
    CreateUser {
        user_data: CreateUserData,
    },
    UpdateUserAccess {
        user_id: String,
        access_level: UserAccessLevel,
    },
    UpdateSystemConfig {
        config_key: String,
        config_value: serde_json::Value,
    },
    EnableMaintenanceMode,
    DisableMaintenanceMode,
}

/// Results of admin actions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AdminActionResult {
    UserCreated { user_id: String },
    UserAccessUpdated { user_id: String },
    SystemConfigUpdated { config_key: String },
    MaintenanceModeEnabled,
    MaintenanceModeDisabled,
}

/// Data for creating a new user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateUserData {
    pub telegram_user_id: Option<i64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub subscription_tier: SubscriptionTier,
    pub access_level: UserAccessLevel,
    pub invitation_code: Option<String>,
}
