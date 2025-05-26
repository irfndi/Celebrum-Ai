use crate::services::core::infrastructure::{D1Service, KVService};
use crate::services::core::user::session_management::SessionManagementService;
use crate::services::core::opportunities::opportunity_distribution::{OpportunityDistributionService, DistributionConfig};
use crate::services::interfaces::telegram::telegram::TelegramService;
use crate::utils::ArbitrageResult;
use worker::{Env, kv::KvStore};
use std::sync::Arc;

/// Service container for managing session and opportunity distribution services
/// Provides centralized dependency injection and service lifecycle management
pub struct SessionDistributionServiceContainer {
    pub session_service: SessionManagementService,
    pub distribution_service: OpportunityDistributionService,
    pub telegram_service: Option<Arc<TelegramService>>,
}

impl SessionDistributionServiceContainer {
    /// Create a new service container with all dependencies
    pub fn new(env: &Env, kv_store: KvStore) -> ArbitrageResult<Self> {
        // Create infrastructure services
        let d1_service = D1Service::new(env)?;
        let kv_service = KVService::new(kv_store);
        
        // Create session management service
        let session_service = SessionManagementService::new(d1_service.clone(), kv_service.clone());
        
        // Create opportunity distribution service
        let distribution_service = OpportunityDistributionService::new(
            d1_service,
            kv_service,
            session_service.clone(),
        );
        
        Ok(Self {
            session_service,
            distribution_service,
            telegram_service: None,
        })
    }

    /// Create with custom distribution configuration
    pub fn with_distribution_config(env: &Env, kv_store: KvStore, config: DistributionConfig) -> ArbitrageResult<Self> {
        let mut container = Self::new(env, kv_store)?;
        container.distribution_service = container.distribution_service.with_config(config);
        Ok(container)
    }

    /// Set the Telegram service for push notifications using Arc for shared ownership
    pub fn set_telegram_service(&mut self, telegram_service: TelegramService) {
        let arc_telegram_service = Arc::new(telegram_service);
        
        // Set telegram service in distribution service (when it supports Arc)
        // Note: This requires updating OpportunityDistributionService to accept Arc<TelegramService>
        // For now, we'll store it in the container
        self.telegram_service = Some(arc_telegram_service);
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

    /// Validate that all required services are configured
    pub fn validate_configuration(&self) -> ArbitrageResult<()> {
        // Check if Telegram service is set for push notifications
        if self.telegram_service.is_none() {
            return Err(crate::utils::ArbitrageError::configuration_error(
                "Telegram service not configured for push notifications".to_string()
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
                status.errors.push(format!("Distribution service error: {}", e));
            }
        }
        
        // Check Telegram service health
        status.telegram_service_healthy = self.telegram_service.is_some();
        if !status.telegram_service_healthy {
            status.errors.push("Telegram service not configured".to_string());
        }
        
        status.overall_healthy = status.session_service_healthy 
            && status.distribution_service_healthy 
            && status.telegram_service_healthy;
        
        Ok(status)
    }

    /// Cleanup expired sessions across all services
    pub async fn cleanup_expired_sessions(&self) -> ArbitrageResult<u32> {
        self.session_service.cleanup_expired_sessions().await
    }

    /// Distribute opportunities to eligible users
    pub async fn distribute_opportunities(&self, opportunities: &[crate::types::ArbitrageOpportunity]) -> ArbitrageResult<u32> {
        let mut total_distributed = 0;
        
        for opportunity in opportunities {
            match self.distribution_service.distribute_opportunity(opportunity.clone()).await {
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
} 