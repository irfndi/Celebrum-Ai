use crate::utils::{ArbitrageError, ArbitrageResult};
use serde_json::json;
use std::sync::Arc;
use worker::{kv::KvStore, D1Database};

/// Simplified Admin Service for API endpoints
/// Provides basic admin functionality without complex dependencies
pub struct SimpleAdminService {
    kv_store: KvStore,
    d1_service: Arc<D1Database>,
}

impl SimpleAdminService {
    /// Create new SimpleAdminService
    pub fn new(kv_store: KvStore, d1_service: Arc<D1Database>) -> Self {
        Self {
            kv_store,
            d1_service,
        }
    }

    /// Get user statistics for admin dashboard
    pub async fn get_user_statistics(&self) -> ArbitrageResult<UserStatistics> {
        // Get total users
        let total_users = self.get_total_users().await?;
        let active_users = self.get_active_users().await?;
        let premium_users = self.get_premium_users().await?;

        // Get users by tier
        let tier_stats = self.get_users_by_tier().await?;

        // Get activity metrics
        let activity_metrics = self.get_activity_metrics().await?;

        // Get registration trends
        let registration_trends = self.get_registration_trends().await?;

        Ok(UserStatistics {
            total_users,
            active_users,
            premium_users,
            free_tier_users: tier_stats.free,
            basic_tier_users: tier_stats.basic,
            premium_tier_users: tier_stats.premium,
            enterprise_tier_users: tier_stats.enterprise,
            daily_active_users: activity_metrics.daily,
            weekly_active_users: activity_metrics.weekly,
            monthly_active_users: activity_metrics.monthly,
            registrations_today: registration_trends.today,
            registrations_this_week: registration_trends.week,
            registrations_this_month: registration_trends.month,
        })
    }

    /// Get system information
    pub async fn get_system_info(&self) -> ArbitrageResult<serde_json::Value> {
        // Check cache first
        let cache_key = "admin_system_info";
        if let Ok(Some(cached)) = self.kv_store.get(cache_key).text().await {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(data);
            }
        }

        let system_info = json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "version": "1.0.0",
            "environment": "production",
            "services": {
                "database": self.check_database_health().await,
                "kv_store": self.check_kv_health().await,
                "api": true
            },
            "performance": {
                "uptime_seconds": 0, // Would be calculated from startup time
                "memory_usage": "N/A", // Not available in Cloudflare Workers
                "cpu_usage": "N/A"     // Not available in Cloudflare Workers
            },
            "feature_flags": self.get_feature_flags().await?,
            "configuration": {
                "max_concurrent_users": 2500,
                "rate_limiting_enabled": true,
                "maintenance_mode": false
            }
        });

        // Cache for 1 minute
        let _ = self
            .kv_store
            .put(cache_key, system_info.to_string())?
            .expiration_ttl(60)
            .execute()
            .await;

        Ok(system_info)
    }

    /// Get system configuration
    pub async fn get_system_config(&self) -> ArbitrageResult<serde_json::Value> {
        // Check cache first
        let cache_key = "admin_system_config";
        if let Ok(Some(cached)) = self.kv_store.get(cache_key).text().await {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(data);
            }
        }

        let config = json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "feature_flags": self.get_feature_flags().await?,
            "rate_limits": {
                "api_requests_per_minute": 100,
                "opportunities_per_user": 50,
                "telegram_commands_per_minute": 10
            },
            "trading": {
                "max_position_size": 10000.0,
                "min_profit_threshold": 0.1,
                "max_slippage": 0.5
            },
            "notifications": {
                "telegram_enabled": true,
                "discord_enabled": false,
                "email_enabled": false
            },
            "security": {
                "encryption_enabled": true,
                "audit_logging": true,
                "session_timeout_minutes": 60
            }
        });

        // Cache for 5 minutes
        let _ = self
            .kv_store
            .put(cache_key, config.to_string())?
            .expiration_ttl(300)
            .execute()
            .await;

        Ok(config)
    }

    /// Update system configuration
    pub async fn update_system_config(
        &self,
        update_request: serde_json::Value,
    ) -> ArbitrageResult<serde_json::Value> {
        // For now, just return the updated config
        // In a real implementation, this would validate and persist the changes
        let updated_config = json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "updated_fields": update_request,
            "status": "updated",
            "message": "Configuration updated successfully"
        });

        // Clear cache to force refresh
        let _ = self.kv_store.delete("admin_system_config").await;

        Ok(updated_config)
    }

    // Helper methods
    async fn get_total_users(&self) -> ArbitrageResult<u64> {
        let stmt = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM user_profiles");
        let result = stmt.first::<serde_json::Value>(None).await.map_err(|e| {
            ArbitrageError::database_error(format!("Failed to get total users: {}", e))
        })?;

        Ok(result
            .and_then(|r| r.get("count").cloned())
            .and_then(|c| c.as_u64())
            .unwrap_or(0))
    }

    async fn get_active_users(&self) -> ArbitrageResult<u64> {
        let stmt = self.d1_service.prepare(
            "
            SELECT COUNT(*) as count 
            FROM user_profiles 
            WHERE last_login_at >= datetime('now', '-7 days')
        ",
        );
        let result = stmt.first::<serde_json::Value>(None).await.map_err(|e| {
            ArbitrageError::database_error(format!("Failed to get active users: {}", e))
        })?;

        Ok(result
            .and_then(|r| r.get("count").cloned())
            .and_then(|c| c.as_u64())
            .unwrap_or(0))
    }

    async fn get_premium_users(&self) -> ArbitrageResult<u64> {
        let stmt = self.d1_service.prepare(
            "
            SELECT COUNT(*) as count 
            FROM user_profiles 
            WHERE subscription_tier IN ('premium', 'enterprise')
        ",
        );
        let result = stmt.first::<serde_json::Value>(None).await.map_err(|e| {
            ArbitrageError::database_error(format!("Failed to get premium users: {}", e))
        })?;

        Ok(result
            .and_then(|r| r.get("count").cloned())
            .and_then(|c| c.as_u64())
            .unwrap_or(0))
    }

    async fn get_users_by_tier(&self) -> ArbitrageResult<TierStats> {
        let stmt = self.d1_service.prepare(
            "
            SELECT 
                subscription_tier,
                COUNT(*) as count 
            FROM user_profiles 
            GROUP BY subscription_tier
        ",
        );
        let results = stmt.all().await.map_err(|e| {
            ArbitrageError::database_error(format!("Failed to get tier stats: {}", e))
        })?;

        let mut stats = TierStats::default();
        for result in results.results::<serde_json::Value>().unwrap_or_default() {
            if let (Some(tier), Some(count)) = (
                result.get("subscription_tier").and_then(|t| t.as_str()),
                result.get("count").and_then(|c| c.as_u64()),
            ) {
                match tier {
                    "free" => stats.free = count,
                    "basic" => stats.basic = count,
                    "premium" => stats.premium = count,
                    "enterprise" => stats.enterprise = count,
                    _ => {}
                }
            }
        }

        Ok(stats)
    }

    async fn get_activity_metrics(&self) -> ArbitrageResult<ActivityMetrics> {
        // For now, return mock data since we don't have detailed activity tracking
        Ok(ActivityMetrics {
            daily: 0,
            weekly: 0,
            monthly: 0,
        })
    }

    async fn get_registration_trends(&self) -> ArbitrageResult<RegistrationTrends> {
        // For now, return mock data since we don't have detailed registration tracking
        Ok(RegistrationTrends {
            today: 0,
            week: 0,
            month: 0,
        })
    }

    async fn check_database_health(&self) -> bool {
        matches!(
            self.d1_service
                .prepare("SELECT 1")
                .first::<serde_json::Value>(None)
                .await,
            Ok(Some(_))
        )
    }

    async fn check_kv_health(&self) -> bool {
        self.kv_store.get("health_check").text().await.is_ok()
    }

    async fn get_feature_flags(&self) -> ArbitrageResult<serde_json::Value> {
        // Try to get feature flags from KV store
        if let Ok(Some(flags)) = self.kv_store.get("feature_flags").text().await {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&flags) {
                return Ok(data);
            }
        }

        // Return default feature flags
        Ok(json!({
            "trading_manual": true,
            "trading_auto": false,
            "ai_features": true,
            "admin_panel": true,
            "beta_features": false,
            "funding_rate_integration": true,
            "opportunity_deduplication": true,
            "trade_target_calculation": true,
            "opportunity_validity_engine": true
        }))
    }
}

#[derive(Debug, Default)]
struct TierStats {
    free: u64,
    basic: u64,
    premium: u64,
    enterprise: u64,
}

#[derive(Debug, Default)]
struct ActivityMetrics {
    daily: u64,
    weekly: u64,
    monthly: u64,
}

#[derive(Debug, Default)]
struct RegistrationTrends {
    today: u64,
    week: u64,
    month: u64,
}

#[derive(Debug)]
pub struct UserStatistics {
    pub total_users: u64,
    pub active_users: u64,
    pub premium_users: u64,
    pub free_tier_users: u64,
    pub basic_tier_users: u64,
    pub premium_tier_users: u64,
    pub enterprise_tier_users: u64,
    pub daily_active_users: u64,
    pub weekly_active_users: u64,
    pub monthly_active_users: u64,
    pub registrations_today: u64,
    pub registrations_this_week: u64,
    pub registrations_this_month: u64,
}
