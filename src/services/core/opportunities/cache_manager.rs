// src/services/core/opportunities/cache_manager.rs

use crate::log_info;
use crate::types::{ArbitrageOpportunity, GlobalOpportunity, TechnicalOpportunity};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde_json;
use std::collections::HashMap;
use worker::kv::KvStore;

/// Unified cache manager for all opportunity services
/// Consolidates caching logic and provides consistent cache management
pub struct OpportunityDataCache {
    kv_store: KvStore,
    default_ttl_seconds: u64,
    cache_prefixes: CachePrefixes,
}

#[derive(Clone)]
pub struct CachePrefixes {
    pub arbitrage_opportunities: String,
    pub technical_opportunities: String,
    pub global_opportunities: String,
    pub user_opportunities: String,
    pub group_opportunities: String,
    pub market_data: String,
    pub funding_rates: String,
    pub distribution_stats: String,
    pub user_access: String,
}

impl Default for CachePrefixes {
    fn default() -> Self {
        Self {
            arbitrage_opportunities: "arb_opp".to_string(),
            technical_opportunities: "tech_opp".to_string(),
            global_opportunities: "global_opp".to_string(),
            user_opportunities: "user_opp".to_string(),
            group_opportunities: "group_opp".to_string(),
            market_data: "market_data".to_string(),
            funding_rates: "funding_rates".to_string(),
            distribution_stats: "dist_stats".to_string(),
            user_access: "user_access".to_string(),
        }
    }
}

impl OpportunityDataCache {
    const DEFAULT_TTL_SECONDS: u64 = 300; // 5 minutes
    const LONG_TTL_SECONDS: u64 = 3600; // 1 hour
    const SHORT_TTL_SECONDS: u64 = 60; // 1 minute

    pub fn new(kv_store: KvStore) -> Self {
        Self {
            kv_store,
            default_ttl_seconds: Self::DEFAULT_TTL_SECONDS,
            cache_prefixes: CachePrefixes::default(),
        }
    }

    pub fn with_custom_prefixes(kv_store: KvStore, prefixes: CachePrefixes) -> Self {
        Self {
            kv_store,
            default_ttl_seconds: Self::DEFAULT_TTL_SECONDS,
            cache_prefixes: prefixes,
        }
    }

    // Arbitrage Opportunities Caching

    /// Cache arbitrage opportunities for a user
    pub async fn cache_user_arbitrage_opportunities(
        &self,
        user_id: &str,
        opportunities: &[ArbitrageOpportunity],
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:user:{}:arbitrage",
            self.cache_prefixes.user_opportunities, user_id
        );
        self.cache_opportunities(
            &cache_key,
            opportunities,
            ttl_seconds.unwrap_or(self.default_ttl_seconds),
        )
        .await
    }

    /// Get cached arbitrage opportunities for a user
    pub async fn get_cached_user_arbitrage_opportunities(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<Vec<ArbitrageOpportunity>>> {
        let cache_key = format!(
            "{}:user:{}:arbitrage",
            self.cache_prefixes.user_opportunities, user_id
        );
        self.get_cached_opportunities(&cache_key).await
    }

    /// Cache group arbitrage opportunities
    pub async fn cache_group_arbitrage_opportunities(
        &self,
        group_id: &str,
        opportunities: &[ArbitrageOpportunity],
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:group:{}:arbitrage",
            self.cache_prefixes.group_opportunities, group_id
        );
        self.cache_opportunities(
            &cache_key,
            opportunities,
            ttl_seconds.unwrap_or(self.default_ttl_seconds),
        )
        .await
    }

    /// Get cached group arbitrage opportunities
    pub async fn get_cached_group_arbitrage_opportunities(
        &self,
        group_id: &str,
    ) -> ArbitrageResult<Option<Vec<ArbitrageOpportunity>>> {
        let cache_key = format!(
            "{}:group:{}:arbitrage",
            self.cache_prefixes.group_opportunities, group_id
        );
        self.get_cached_opportunities(&cache_key).await
    }

    // Technical Opportunities Caching

    /// Cache technical opportunities for a user
    pub async fn cache_user_technical_opportunities(
        &self,
        user_id: &str,
        opportunities: &[TechnicalOpportunity],
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:user:{}:technical",
            self.cache_prefixes.user_opportunities, user_id
        );
        self.cache_technical_opportunities(
            &cache_key,
            opportunities,
            ttl_seconds.unwrap_or(self.default_ttl_seconds),
        )
        .await
    }

    /// Get cached technical opportunities for a user
    pub async fn get_cached_user_technical_opportunities(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<Vec<TechnicalOpportunity>>> {
        let cache_key = format!(
            "{}:user:{}:technical",
            self.cache_prefixes.user_opportunities, user_id
        );
        self.get_cached_technical_opportunities(&cache_key).await
    }

    // Global Opportunities Caching

    /// Cache global opportunities
    pub async fn cache_global_opportunities(
        &self,
        opportunities: &[GlobalOpportunity],
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!("{}:current", self.cache_prefixes.global_opportunities);
        self.cache_global_opportunities_internal(
            &cache_key,
            opportunities,
            ttl_seconds.unwrap_or(self.default_ttl_seconds),
        )
        .await
    }

    /// Get cached global opportunities
    pub async fn get_cached_global_opportunities(
        &self,
    ) -> ArbitrageResult<Option<Vec<GlobalOpportunity>>> {
        let cache_key = format!("{}:current", self.cache_prefixes.global_opportunities);
        self.get_cached_global_opportunities_internal(&cache_key)
            .await
    }

    // Market Data Caching

    /// Cache market data for an exchange and symbol
    pub async fn cache_market_data(
        &self,
        exchange: &str,
        symbol: &str,
        data: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:{}:{}",
            self.cache_prefixes.market_data, exchange, symbol
        );
        let data_str = serde_json::to_string(data).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize market data: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, data_str)?
            .expiration_ttl(ttl_seconds.unwrap_or(Self::SHORT_TTL_SECONDS))
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to cache market data: {}", e))
            })?;

        log_info!(
            "Cached market data",
            serde_json::json!({
                "exchange": exchange,
                "symbol": symbol,
                "ttl_seconds": ttl_seconds.unwrap_or(Self::SHORT_TTL_SECONDS)
            })
        );

        Ok(())
    }

    /// Get cached market data for an exchange and symbol
    pub async fn get_cached_market_data(
        &self,
        exchange: &str,
        symbol: &str,
    ) -> ArbitrageResult<Option<serde_json::Value>> {
        let cache_key = format!(
            "{}:{}:{}",
            self.cache_prefixes.market_data, exchange, symbol
        );

        match self.kv_store.get(&cache_key).text().await {
            Ok(Some(data)) => {
                match serde_json::from_str(&data) {
                    Ok(parsed_data) => {
                        log_info!(
                            "Retrieved cached market data",
                            serde_json::json!({
                                "exchange": exchange,
                                "symbol": symbol,
                                "cache_hit": true
                            })
                        );
                        Ok(Some(parsed_data))
                    }
                    Err(_) => Ok(None), // Invalid cache data
                }
            }
            Ok(None) => Ok(None), // Cache miss
            Err(_) => Ok(None),   // Cache error, treat as miss
        }
    }

    // Funding Rates Caching

    /// Cache funding rates for an exchange and symbol
    pub async fn cache_funding_rates(
        &self,
        exchange: &str,
        symbol: &str,
        rates: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:{}:{}",
            self.cache_prefixes.funding_rates, exchange, symbol
        );
        let data_str = serde_json::to_string(rates).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize funding rates: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, data_str)?
            .expiration_ttl(ttl_seconds.unwrap_or(Self::DEFAULT_TTL_SECONDS))
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to cache funding rates: {}", e))
            })?;

        Ok(())
    }

    /// Get cached funding rates for an exchange and symbol
    pub async fn get_cached_funding_rates(
        &self,
        exchange: &str,
        symbol: &str,
    ) -> ArbitrageResult<Option<serde_json::Value>> {
        let cache_key = format!(
            "{}:{}:{}",
            self.cache_prefixes.funding_rates, exchange, symbol
        );

        match self.kv_store.get(&cache_key).text().await {
            Ok(Some(data)) => match serde_json::from_str(&data) {
                Ok(parsed_data) => Ok(Some(parsed_data)),
                Err(_) => Ok(None),
            },
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    // Distribution Stats Caching

    /// Cache distribution statistics
    pub async fn cache_distribution_stats(
        &self,
        stats: &HashMap<String, serde_json::Value>,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        for (key, value) in stats {
            let cache_key = format!("{}:{}", self.cache_prefixes.distribution_stats, key);
            let data_str = serde_json::to_string(value).map_err(|e| {
                ArbitrageError::serialization_error(format!(
                    "Failed to serialize distribution stat: {}",
                    e
                ))
            })?;

            self.kv_store
                .put(&cache_key, data_str)?
                .expiration_ttl(ttl_seconds.unwrap_or(Self::LONG_TTL_SECONDS))
                .execute()
                .await
                .map_err(|e| {
                    ArbitrageError::database_error(format!(
                        "Failed to cache distribution stat: {}",
                        e
                    ))
                })?;
        }

        log_info!(
            "Cached distribution stats",
            serde_json::json!({
                "stats_count": stats.len(),
                "ttl_seconds": ttl_seconds.unwrap_or(Self::LONG_TTL_SECONDS)
            })
        );

        Ok(())
    }

    /// Get cached distribution statistics
    pub async fn get_cached_distribution_stats(
        &self,
    ) -> ArbitrageResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();

        // Common distribution stat keys
        let stat_keys = vec![
            "opportunities_today",
            "active_users",
            "avg_time_ms",
            "total_distributed",
        ];

        for key in stat_keys {
            let cache_key = format!("{}:{}", self.cache_prefixes.distribution_stats, key);
            if let Some(data) = self.kv_store.get(&cache_key).text().await? {
                if let Ok(parsed_data) = serde_json::from_str(&data) {
                    stats.insert(key.to_string(), parsed_data);
                }
            }
        }

        Ok(stats)
    }

    // User Access Caching

    /// Cache user access information
    pub async fn cache_user_access(
        &self,
        user_id: &str,
        access_data: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let cache_key = format!("{}:{}", self.cache_prefixes.user_access, user_id);
        let data_str = serde_json::to_string(access_data).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize user access: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, data_str)?
            .expiration_ttl(ttl_seconds.unwrap_or(Self::DEFAULT_TTL_SECONDS))
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to cache user access: {}", e))
            })?;

        Ok(())
    }

    /// Get cached user access information
    pub async fn get_cached_user_access(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<serde_json::Value>> {
        let cache_key = format!("{}:{}", self.cache_prefixes.user_access, user_id);

        match self.kv_store.get(&cache_key).text().await {
            Ok(Some(data)) => match serde_json::from_str(&data) {
                Ok(parsed_data) => Ok(Some(parsed_data)),
                Err(_) => Ok(None),
            },
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    // Cache Invalidation

    /// Invalidate user-specific caches
    pub async fn invalidate_user_cache(&self, user_id: &str) -> ArbitrageResult<()> {
        let keys_to_delete = vec![
            format!(
                "{}:user:{}:arbitrage",
                self.cache_prefixes.user_opportunities, user_id
            ),
            format!(
                "{}:user:{}:technical",
                self.cache_prefixes.user_opportunities, user_id
            ),
            format!("{}:{}", self.cache_prefixes.user_access, user_id),
        ];

        for key in keys_to_delete {
            let _ = self.kv_store.delete(&key).await; // Ignore errors for cache invalidation
        }

        log_info!(
            "Invalidated user cache",
            serde_json::json!({
                "user_id": user_id
            })
        );

        Ok(())
    }

    /// Invalidate group-specific caches
    pub async fn invalidate_group_cache(&self, group_id: &str) -> ArbitrageResult<()> {
        let keys_to_delete = vec![
            format!(
                "{}:group:{}:arbitrage",
                self.cache_prefixes.group_opportunities, group_id
            ),
            format!(
                "{}:group:{}:technical",
                self.cache_prefixes.group_opportunities, group_id
            ),
        ];

        for key in keys_to_delete {
            let _ = self.kv_store.delete(&key).await;
        }

        log_info!(
            "Invalidated group cache",
            serde_json::json!({
                "group_id": group_id
            })
        );

        Ok(())
    }

    /// Invalidate market data cache for an exchange and symbol
    pub async fn invalidate_market_data_cache(
        &self,
        exchange: &str,
        symbol: &str,
    ) -> ArbitrageResult<()> {
        let keys_to_delete = vec![
            format!(
                "{}:{}:{}",
                self.cache_prefixes.market_data, exchange, symbol
            ),
            format!(
                "{}:{}:{}",
                self.cache_prefixes.funding_rates, exchange, symbol
            ),
        ];

        for key in keys_to_delete {
            let _ = self.kv_store.delete(&key).await;
        }

        Ok(())
    }

    // Private helper methods

    async fn cache_opportunities<T: serde::Serialize>(
        &self,
        cache_key: &str,
        opportunities: &[T],
        ttl_seconds: u64,
    ) -> ArbitrageResult<()> {
        let data = serde_json::to_string(opportunities).map_err(|e| {
            ArbitrageError::serialization_error(format!("Failed to serialize opportunities: {}", e))
        })?;

        self.kv_store
            .put(cache_key, data)?
            .expiration_ttl(ttl_seconds)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to cache opportunities: {}", e))
            })?;

        Ok(())
    }

    async fn get_cached_opportunities<T: serde::de::DeserializeOwned>(
        &self,
        cache_key: &str,
    ) -> ArbitrageResult<Option<Vec<T>>> {
        match self.kv_store.get(cache_key).text().await {
            Ok(Some(data)) => {
                match serde_json::from_str(&data) {
                    Ok(opportunities) => Ok(Some(opportunities)),
                    Err(_) => Ok(None), // Invalid cache data
                }
            }
            Ok(None) => Ok(None), // Cache miss
            Err(_) => Ok(None),   // Cache error
        }
    }

    async fn cache_technical_opportunities(
        &self,
        cache_key: &str,
        opportunities: &[TechnicalOpportunity],
        ttl_seconds: u64,
    ) -> ArbitrageResult<()> {
        self.cache_opportunities(cache_key, opportunities, ttl_seconds)
            .await
    }

    async fn get_cached_technical_opportunities(
        &self,
        cache_key: &str,
    ) -> ArbitrageResult<Option<Vec<TechnicalOpportunity>>> {
        self.get_cached_opportunities(cache_key).await
    }

    async fn cache_global_opportunities_internal(
        &self,
        cache_key: &str,
        opportunities: &[GlobalOpportunity],
        ttl_seconds: u64,
    ) -> ArbitrageResult<()> {
        self.cache_opportunities(cache_key, opportunities, ttl_seconds)
            .await
    }

    async fn get_cached_global_opportunities_internal(
        &self,
        cache_key: &str,
    ) -> ArbitrageResult<Option<Vec<GlobalOpportunity>>> {
        self.get_cached_opportunities(cache_key).await
    }
}

// Tests moved to packages/worker/tests/opportunities/cache_manager_test.rs
