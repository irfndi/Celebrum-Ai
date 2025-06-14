use crate::utils::{ArbitrageError, ArbitrageResult};
use serde_json::json;
use std::sync::Arc;
use worker::{kv::KvStore, D1Database};

/// Analytics Service for dashboard analytics
/// Provides analytics functionality for API endpoints
pub struct AnalyticsService {
    kv_store: KvStore,
    d1_service: Arc<D1Database>,
}

impl AnalyticsService {
    /// Create new AnalyticsService
    pub fn new(kv_store: KvStore, d1_service: Arc<D1Database>) -> Self {
        Self {
            kv_store,
            d1_service,
        }
    }

    /// Get dashboard analytics for a user
    pub async fn get_dashboard_analytics(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<serde_json::Value> {
        // Check cache first
        let cache_key = format!("dashboard_analytics:{}", user_id);
        if let Ok(Some(cached)) = self.kv_store.get(&cache_key).text().await {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(data);
            }
        }

        // Get user statistics from database
        let user_stats = self.get_user_statistics(user_id).await?;
        let opportunity_stats = self.get_opportunity_statistics(user_id).await?;
        let trading_stats = self.get_trading_statistics(user_id).await?;

        let analytics = json!({
            "user_id": user_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "user_stats": user_stats,
            "opportunity_stats": opportunity_stats,
            "trading_stats": trading_stats,
            "performance": {
                "total_opportunities": opportunity_stats["total_count"],
                "profitable_opportunities": opportunity_stats["profitable_count"],
                "success_rate": if opportunity_stats["total_count"].as_u64().unwrap_or(0) > 0 {
                    (opportunity_stats["profitable_count"].as_f64().unwrap_or(0.0) /
                     opportunity_stats["total_count"].as_f64().unwrap_or(1.0)) * 100.0
                } else { 0.0 },
                "avg_profit_percentage": opportunity_stats["avg_profit"],
                "total_volume": trading_stats["total_volume"]
            }
        });

        // Cache for 5 minutes
        let _ = self
            .kv_store
            .put(&cache_key, analytics.to_string())?
            .expiration_ttl(300)
            .execute()
            .await;

        Ok(analytics)
    }

    /// Get user statistics
    async fn get_user_statistics(&self, user_id: &str) -> ArbitrageResult<serde_json::Value> {
        let stmt = self.d1_service.prepare(
            "
            SELECT 
                created_at,
                access_level,
                subscription_tier,
                last_login_at
            FROM user_profiles 
            WHERE user_id = ?
        ",
        );

        let result = stmt
            .bind(&[user_id.into()])?
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to get user stats: {}", e))
            })?;

        Ok(result.unwrap_or_else(|| {
            json!({
                "created_at": null,
                "access_level": "basic",
                "subscription_tier": "free",
                "last_login_at": null
            })
        }))
    }

    /// Get opportunity statistics
    async fn get_opportunity_statistics(
        &self,
        _user_id: &str,
    ) -> ArbitrageResult<serde_json::Value> {
        let stmt = self.d1_service.prepare(
            "
            SELECT 
                COUNT(*) as total_count,
                COUNT(CASE WHEN profit_percentage > 0 THEN 1 END) as profitable_count,
                AVG(profit_percentage) as avg_profit,
                MAX(profit_percentage) as max_profit,
                MIN(profit_percentage) as min_profit
            FROM opportunities 
            WHERE created_at >= datetime('now', '-30 days')
        ",
        );

        let result = stmt.first::<serde_json::Value>(None).await.map_err(|e| {
            ArbitrageError::database_error(format!("Failed to get opportunity stats: {}", e))
        })?;

        Ok(result.unwrap_or_else(|| {
            json!({
                "total_count": 0,
                "profitable_count": 0,
                "avg_profit": 0.0,
                "max_profit": 0.0,
                "min_profit": 0.0
            })
        }))
    }

    /// Get trading statistics
    async fn get_trading_statistics(&self, _user_id: &str) -> ArbitrageResult<serde_json::Value> {
        // For now, return mock data since we don't have trading history table
        // This would be replaced with real trading data in production
        Ok(json!({
            "total_volume": 0.0,
            "total_trades": 0,
            "successful_trades": 0,
            "failed_trades": 0,
            "avg_trade_size": 0.0,
            "last_trade_at": null
        }))
    }

    /// Get portfolio analytics
    pub async fn get_portfolio_analytics(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<serde_json::Value> {
        // Check cache first
        let cache_key = format!("portfolio_analytics:{}", user_id);
        if let Ok(Some(cached)) = self.kv_store.get(&cache_key).text().await {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(data);
            }
        }

        let portfolio = json!({
            "user_id": user_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "total_balance": 0.0,
            "available_balance": 0.0,
            "locked_balance": 0.0,
            "positions": [],
            "pnl": {
                "daily": 0.0,
                "weekly": 0.0,
                "monthly": 0.0,
                "total": 0.0
            },
            "risk_metrics": {
                "var_95": 0.0,
                "max_drawdown": 0.0,
                "sharpe_ratio": 0.0,
                "volatility": 0.0
            }
        });

        // Cache for 2 minutes
        let _ = self
            .kv_store
            .put(&cache_key, portfolio.to_string())?
            .expiration_ttl(120)
            .execute()
            .await;

        Ok(portfolio)
    }
}
