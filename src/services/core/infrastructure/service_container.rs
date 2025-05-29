use crate::services::core::infrastructure::ai_services::{AICoordinator, AICoordinatorConfig};
use crate::services::core::infrastructure::data_access_layer::{
    DataAccessLayer, DataAccessLayerConfig,
};
use crate::services::core::infrastructure::data_ingestion_module::{
    DataIngestionModule, DataIngestionModuleConfig,
};
use crate::services::core::infrastructure::database_repositories::{
    DatabaseManager, DatabaseManagerConfig,
};
use crate::services::core::opportunities::{DistributionConfig, OpportunityDistributionService};
use crate::services::core::trading::ExchangeService;
use crate::services::core::user::{SessionManagementService, UserProfileService};
use crate::services::interfaces::telegram::TelegramService;
use crate::types;
use crate::utils::{ArbitrageError, ArbitrageResult};
use std::sync::Arc;
use worker::{kv::KvStore, Env};

/// Comprehensive service container for managing all application services
/// Provides centralized dependency injection and service lifecycle management
/// Addresses inconsistent service creation patterns across the application
pub struct ServiceContainer {
    pub session_service: SessionManagementService,
    pub distribution_service: OpportunityDistributionService,
    pub telegram_service: Option<Arc<TelegramService>>,
    pub exchange_service: Arc<ExchangeService>,
    pub user_profile_service: Option<Arc<UserProfileService>>,
    pub ai_coordinator: Option<Arc<AICoordinator>>,
    pub data_ingestion_module: Option<Arc<DataIngestionModule>>,
    pub database_manager: DatabaseManager,
    pub data_access_layer: DataAccessLayer,
}

impl ServiceContainer {
    /// Create a new comprehensive service container with all dependencies
    pub async fn new(env: &Env, kv_store: KvStore) -> ArbitrageResult<Self> {
        // Create custom environment wrapper
        let custom_env = types::Env::new(env.clone());

        // Get D1 database from environment
        let d1_database = env.d1("DB").map_err(|e| {
            ArbitrageError::configuration_error(format!("Failed to get D1 database: {}", e))
        })?;
        let d1_arc = Arc::new(d1_database);

        // Create database manager with configuration
        let db_config = DatabaseManagerConfig::default();
        let database_manager = DatabaseManager::new(d1_arc, db_config);

        // Create data access layer with configuration
        let dal_config = DataAccessLayerConfig::default();
        let data_access_layer = DataAccessLayer::new(dal_config, kv_store.clone())
            .await
            .map_err(|e| {
                ArbitrageError::configuration_error(format!(
                    "Failed to create DataAccessLayer: {}",
                    e
                ))
            })?;

        // Create exchange service (core service used by many endpoints)
        let exchange_service = Arc::new(ExchangeService::new(&custom_env)?);

        // Create session management service
        let session_service = SessionManagementService::new(
            database_manager.clone(),
            data_access_layer.get_kv_store(),
        );

        // Create opportunity distribution service
        let distribution_service = OpportunityDistributionService::new(
            database_manager.clone(),
            data_access_layer.clone(),
            session_service.clone(),
        );

        Ok(Self {
            session_service,
            distribution_service,
            telegram_service: None,
            exchange_service,
            user_profile_service: None,
            ai_coordinator: None,
            data_ingestion_module: None,
            database_manager,
            data_access_layer,
        })
    }

    /// Create with custom distribution configuration
    pub async fn with_distribution_config(
        env: &Env,
        kv_store: KvStore,
        config: DistributionConfig,
    ) -> ArbitrageResult<Self> {
        let mut container = Self::new(env, kv_store).await?;
        container.distribution_service = container.distribution_service.with_config(config);
        Ok(container)
    }

    /// Set the Telegram service for push notifications using Arc for shared ownership
    pub fn set_telegram_service(&mut self, telegram_service: TelegramService) {
        let arc_telegram_service = Arc::new(telegram_service);

        // Set telegram service in distribution service
        // self.distribution_service
        //     .set_notification_sender(Box::new(arc_telegram_service.clone()));

        // Store the Arc for container access
        self.telegram_service = Some(arc_telegram_service);
    }

    /// Set the user profile service with encryption key
    pub fn set_user_profile_service(&mut self, encryption_key: String) {
        // UserProfileService expects raw KvStore, not DataAccessLayer
        // We'll need to get the KvStore from DataAccessLayer
        // For now, create a placeholder implementation
        let user_profile_service = UserProfileService::new(
            self.data_access_layer.get_kv_store(),
            self.database_manager.clone(),
            encryption_key,
        );
        self.user_profile_service = Some(Arc::new(user_profile_service));
    }

    /// Initialize AI Coordinator service with fallback mechanisms
    pub fn set_ai_coordinator(&mut self, env: &Env, config: Option<AICoordinatorConfig>) {
        let ai_config = config.unwrap_or_default();

        match AICoordinator::new(env, ai_config) {
            Ok(coordinator) => {
                self.ai_coordinator = Some(Arc::new(coordinator));
                worker::console_log!("AI Coordinator initialized successfully");
            }
            Err(e) => {
                worker::console_log!(
                    "Failed to initialize AI Coordinator: {} - continuing with fallback mode",
                    e
                );
                self.ai_coordinator = None;
            }
        }
    }

    /// Initialize Data Ingestion Module with fallback mechanisms
    pub async fn set_data_ingestion_module(
        &mut self,
        env: &Env,
        config: Option<DataIngestionModuleConfig>,
    ) {
        let ingestion_config = config.unwrap_or_default();

        match DataIngestionModule::new(ingestion_config, self.data_access_layer.get_kv_store(), env)
            .await
        {
            Ok(module) => {
                self.data_ingestion_module = Some(Arc::new(module));
                worker::console_log!(
                    "Data Ingestion Module initialized successfully with fallback support"
                );
            }
            Err(e) => {
                worker::console_log!(
                    "Failed to initialize Data Ingestion Module: {} - continuing with fallback mode",
                    e
                );
                self.data_ingestion_module = None;
            }
        }
    }

    /// Get exchange service (most commonly used service)
    pub fn exchange_service(&self) -> &Arc<ExchangeService> {
        &self.exchange_service
    }

    /// Get session management service
    pub fn session_service(&self) -> &SessionManagementService {
        &self.session_service
    }

    /// Get opportunity distribution service
    pub fn distribution_service(&self) -> &OpportunityDistributionService {
        &self.distribution_service
    }

    /// Get mutable opportunity distribution service
    pub fn distribution_service_mut(&mut self) -> &mut OpportunityDistributionService {
        &mut self.distribution_service
    }

    /// Get telegram service
    pub fn telegram_service(&self) -> Option<&Arc<TelegramService>> {
        self.telegram_service.as_ref()
    }

    /// Get user profile service
    pub fn user_profile_service(&self) -> Option<&Arc<UserProfileService>> {
        self.user_profile_service.as_ref()
    }

    /// Get AI coordinator service
    pub fn ai_coordinator(&self) -> Option<&Arc<AICoordinator>> {
        self.ai_coordinator.as_ref()
    }

    /// Get data ingestion module
    pub fn data_ingestion_module(&self) -> Option<&Arc<DataIngestionModule>> {
        self.data_ingestion_module.as_ref()
    }

    /// Get database manager
    pub fn database_manager(&self) -> &DatabaseManager {
        &self.database_manager
    }

    /// Get data access layer
    pub fn data_access_layer(&self) -> &DataAccessLayer {
        &self.data_access_layer
    }

    /// Validate that all required services are configured
    pub fn validate_configuration(&self) -> ArbitrageResult<()> {
        // Check if Telegram service is set for push notifications
        if self.telegram_service.is_none() {
            return Err(crate::utils::ArbitrageError::configuration_error(
                "Telegram service not configured for push notifications".to_string(),
            ));
        }

        Ok(())
    }

    /// Health check for all services
    pub async fn health_check(&self) -> ArbitrageResult<ServiceHealthStatus> {
        let mut status = ServiceHealthStatus::default();

        // Check session service health
        match self.session_service.get_active_session_count().await {
            Ok(_) => status.session_service_healthy = true,
            Err(e) => {
                status.session_service_healthy = false;
                status.errors.push(format!("Session service error: {}", e));
            }
        }

        // Check distribution service health
        match self.distribution_service.get_distribution_stats().await {
            Ok(_) => status.distribution_service_healthy = true,
            Err(e) => {
                status.distribution_service_healthy = false;
                status
                    .errors
                    .push(format!("Distribution service error: {}", e));
            }
        }

        // Check telegram service health
        if let Some(ref _telegram_service) = self.telegram_service {
            // Telegram service is available
            status.telegram_service_healthy = true;
        } else {
            status.telegram_service_healthy = false;
            status
                .errors
                .push("Telegram service not configured".to_string());
        }

        // Check exchange service health
        // Exchange service is always available if created
        status.exchange_service_healthy = true;

        // Check user profile service health
        if let Some(ref _user_profile_service) = self.user_profile_service {
            // User profile service is available
            status.user_profile_service_healthy = true;
        } else {
            status.user_profile_service_healthy = false;
            status
                .errors
                .push("User profile service not configured".to_string());
        }

        // Check AI coordinator health
        if let Some(ref ai_coordinator) = self.ai_coordinator {
            match ai_coordinator.health_check().await {
                Ok(health) => {
                    status.ai_coordinator_healthy = health.overall_health;
                    if !health.overall_health {
                        status.errors.push("AI coordinator unhealthy".to_string());
                    }
                }
                Err(_) => {
                    status.ai_coordinator_healthy = false;
                    status.errors.push("AI coordinator unhealthy".to_string());
                }
            }
        }

        // Check data ingestion module health
        if let Some(ref data_ingestion_module) = self.data_ingestion_module {
            match data_ingestion_module.health_check().await {
                Ok(true) => status.data_ingestion_module_healthy = true,
                Ok(false) | Err(_) => {
                    status.data_ingestion_module_healthy = false;
                    status
                        .errors
                        .push("Data ingestion module unhealthy".to_string());
                }
            }
        }

        // Overall health is true if no errors
        status.overall_healthy = status.errors.is_empty();

        Ok(status)
    }

    /// Cleanup expired sessions across all services
    pub async fn cleanup_expired_sessions(&self) -> ArbitrageResult<u32> {
        self.session_service.cleanup_expired_sessions().await
    }

    /// Distribute opportunities to eligible users
    pub async fn distribute_opportunities(
        &self,
        opportunities: &[crate::types::ArbitrageOpportunity],
    ) -> ArbitrageResult<u32> {
        let mut total_distributed = 0;

        for opportunity in opportunities {
            match self
                .distribution_service
                .distribute_opportunity(opportunity.clone())
                .await
            {
                Ok(count) => total_distributed += count,
                Err(e) => {
                    // Log error but continue with other opportunities
                    worker::console_log!("Failed to distribute opportunity: {}", e);
                }
            }
        }

        Ok(total_distributed)
    }
}

/// Health status for all services in the container
#[derive(Debug, Clone, Default)]
pub struct ServiceHealthStatus {
    pub overall_healthy: bool,
    pub session_service_healthy: bool,
    pub distribution_service_healthy: bool,
    pub telegram_service_healthy: bool,
    pub exchange_service_healthy: bool,
    pub user_profile_service_healthy: bool,
    pub ai_coordinator_healthy: bool,
    pub data_ingestion_module_healthy: bool,
    pub errors: Vec<String>,
}

impl ServiceHealthStatus {
    /// Get a summary of the health status
    pub fn summary(&self) -> String {
        if self.overall_healthy {
            "All services healthy".to_string()
        } else {
            format!("Service issues: {}", self.errors.join(", "))
        }
    }

    /// Get detailed status report
    pub fn detailed_report(&self) -> serde_json::Value {
        serde_json::json!({
            "overall_healthy": self.overall_healthy,
            "services": {
                "session_service": self.session_service_healthy,
                "distribution_service": self.distribution_service_healthy,
                "telegram_service": self.telegram_service_healthy,
                "exchange_service": self.exchange_service_healthy,
                "user_profile_service": self.user_profile_service_healthy,
                "ai_coordinator": self.ai_coordinator_healthy,
                "data_ingestion_module": self.data_ingestion_module_healthy
            },
            "errors": self.errors,
            "summary": self.summary()
        })
    }
}
