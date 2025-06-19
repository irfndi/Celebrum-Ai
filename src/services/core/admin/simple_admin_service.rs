use crate::types::{ArbitrageResult, UserStatistics};
use worker::kv::KvStore;
use serde_json::json;
use std::sync::Arc;
use crate::utils::error::{ArbitrageError, ArbitrageResult};

/// Simplified Admin Service for API endpoints
/// Provides basic admin functionality without complex dependencies
pub struct SimpleAdminService {
    kv_store: KvStore,
    d1_service: Arc<worker::D1Database>,
}

impl SimpleAdminService {
    /// Create new SimpleAdminService
    pub fn new(kv_store: KvStore, d1_service: Arc<worker::D1Database>) -> Self {
        Self {
            kv_store,
            d1_service,
        }
    }

    /// Get user statistics for admin dashboard
    pub async fn get_user_statistics(&self) -> ArbitrageResult<UserStatistics> {
        // Get total users
        let total_users = self.get_total_users().await? as u32;
        let active_users = self.get_active_users().await? as u32;
        let premium_users = self.get_premium_users().await? as u32;

        // Get users by tier
        let tier_stats = self.get_users_by_tier().await?;

        // Get activity metrics
        let activity_metrics = self.get_activity_metrics().await?;

        Ok(UserStatistics {
            total_users,
            active_users,
            free_users: tier_stats.free as u32,
            paid_users: premium_users,
            admin_users: 0, // Would need separate query
            super_admin_users: 0, // Would need separate query
            other_users: 0, // Would need separate query
            recently_active_users: activity_metrics.daily as u32,
            total_trades: 0, // Would need separate query
            total_volume_usdt: 0.0, // Would need separate query
            generated_at: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Get system information
    pub async fn get_system_info(&self) -> ArbitrageResult<serde_json::Value> {
        // Check cache first
        let cache_key = "admin_system_info";
        if let Some(cached) = self.kv_store.get(cache_key).text().await? {
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
        if let Some(cached) = self.kv_store.get(cache_key).text().await? {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(data);
            }
        }

        let config = json!({
            "api": {
                "rate_limit": 1000,
                "timeout_seconds": 30,
                "max_request_size": "10MB"
            },
            "database": {
                "connection_pool_size": 10,
                "query_timeout_seconds": 30
            },
            "cache": {
                "default_ttl_seconds": 300,
                "max_size": "100MB"
            },
            "security": {
                "jwt_expiry_hours": 24,
                "password_min_length": 8,
                "max_login_attempts": 5
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
    pub async fn update_system_config(&self, updates: serde_json::Value) -> ArbitrageResult<serde_json::Value> {
        // Get current config
        let mut current_config = self.get_system_config().await?;
        
        // Merge updates into current config
        if let (Some(current_obj), Some(updates_obj)) = (current_config.as_object_mut(), updates.as_object()) {
            for (key, value) in updates_obj {
                current_obj.insert(key.clone(), value.clone());
            }
        }
        
        // Cache the updated config
        let cache_key = "admin_system_config";
        let _ = self
            .kv_store
            .put(cache_key, current_config.to_string())?
            .expiration_ttl(300)
            .execute()
            .await;
        
        Ok(current_config)
    }

    // Helper methods for user statistics
    async fn get_total_users(&self) -> ArbitrageResult<i64> {
        let result = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        Ok(result
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0))
    }

    async fn get_active_users(&self) -> ArbitrageResult<i64> {
        let result = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE last_login > datetime('now', '-30 days')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        Ok(result
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0))
    }

    async fn get_premium_users(&self) -> ArbitrageResult<i64> {
        let result = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE subscription_tier IN ('premium', 'enterprise')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        Ok(result
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0))
    }

    async fn get_users_by_tier(&self) -> ArbitrageResult<TierStats> {
        let result = self
            .d1_service
            .prepare("SELECT subscription_tier, COUNT(*) as count FROM users GROUP BY subscription_tier")
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        let mut tier_stats = TierStats::default();
        
        if let Ok(results) = result.results::<serde_json::Value>() {
            for row in results {
                if let (Some(tier), Some(count)) = (row.get("subscription_tier").and_then(|v| v.as_str()), row.get("count").and_then(|v| v.as_i64())) {
                    match tier {
                        "free" => tier_stats.free = count,
                        "basic" => tier_stats.basic = count,
                        "premium" => tier_stats.premium = count,
                        "enterprise" => tier_stats.enterprise = count,
                        _ => {}
                    }
                }
            }
        }

        Ok(tier_stats)
    }

    async fn get_activity_metrics(&self) -> ArbitrageResult<ActivityMetrics> {
        // Daily active users
        let daily = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE last_login > datetime('now', '-1 day')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Weekly active users
        let weekly = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE last_login > datetime('now', '-7 days')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Monthly active users
        let monthly = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE last_login > datetime('now', '-30 days')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(ActivityMetrics {
            daily,
            weekly,
            monthly,
        })
    }

    async fn get_registration_trends(&self) -> ArbitrageResult<RegistrationTrends> {
        // Today's registrations
        let today = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE created_at > datetime('now', '-1 day')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // This week's registrations
        let week = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE created_at > datetime('now', '-7 days')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // This month's registrations
        let month = self
            .d1_service
            .prepare("SELECT COUNT(*) as count FROM users WHERE created_at > datetime('now', '-30 days')")
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(RegistrationTrends {
            today,
            week,
            month,
        })
    }

    async fn check_database_health(&self) -> bool {
        self.d1_service
            .prepare("SELECT 1")
            .first::<serde_json::Value>(None)
            .await
            .is_ok()
    }

    async fn check_kv_health(&self) -> bool {
        self.kv_store
            .get("health_check")
            .text()
            .await
            .is_ok()
    }

    async fn get_feature_flags(&self) -> ArbitrageResult<serde_json::Value> {
        Ok(json!({
            "advanced_analytics": true,
            "real_time_notifications": true,
            "beta_features": false,
            "maintenance_mode": false
        }))
    }
}

#[derive(Default)]
struct TierStats {
    free: i64,
    basic: i64,
    premium: i64,
    enterprise: i64,
}

struct ActivityMetrics {
    daily: i64,
    weekly: i64,
    monthly: i64,
}

struct RegistrationTrends {
    today: i64,
    week: i64,
    month: i64,
}
