use std::sync::Arc;

use crate::services::core::infrastructure::CacheManager;
use crate::services::ExchangeService;
use crate::types::{ExchangeIdEnum, FundingRateInfo};
use crate::utils::{ArbitrageError, ArbitrageResult};

/// Default cache time-to-live for funding-rate data (in seconds).
/// Most exchanges publish funding rates every 8 hours, however some
/// (e.g. Bybit hourly) update more frequently.  A conservative 120-minute
/// TTL avoids excessive API calls while ensuring rates remain fresh.
const DEFAULT_TTL_SECONDS: u64 = 7_200; // 2 hours

/// Production-ready funding-rate retrieval & caching layer.
///
/// This service is **exchange-agnostic** – it delegates network calls to
/// `ExchangeService` while handling:
///   • Caching via `CacheManager` (to minimise external API calls)
///   • Unified key construction (so other modules don't duplicate logic)
///   • Graceful fall-backs & error propagation
///   • Optional immediate refresh bypassing cache
///
/// The implementation intentionally avoids assuming any *mock* data – all
/// requests go to live exchange endpoints via `ExchangeService::get_funding_rate_direct`.
#[derive(Clone)]
pub struct FundingRateService {
    exchange_service: Arc<ExchangeService>,
    cache_manager: Arc<CacheManager>,
    ttl_seconds: u64,
}

impl FundingRateService {
    /// Build a new service instance.
    pub fn new(
        exchange_service: Arc<ExchangeService>,
        cache_manager: Arc<CacheManager>,
        ttl_seconds: Option<u64>,
    ) -> Self {
        Self {
            exchange_service,
            cache_manager,
            ttl_seconds: ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS),
        }
    }

    /// Retrieve the latest funding-rate for the given exchange/symbol.
    ///
    /// • Checks cache first and returns cached value if still valid.
    /// • If no cache or expired, fetches fresh data and caches it.
    pub async fn get_funding_rate(
        &self,
        exchange: &ExchangeIdEnum,
        symbol: &str,
    ) -> ArbitrageResult<FundingRateInfo> {
        let cache_key = Self::cache_key(exchange, symbol);

        // Attempt cache hit
        if let Ok(Some(cached)) = self
            .cache_manager
            .kv_get(&cache_key)
            .await
            .map(|s| s.and_then(|v| serde_json::from_str::<FundingRateInfo>(&v).ok()))
        {
            return Ok(cached);
        }

        // Miss – fetch from exchange
        let fresh = self.refresh_funding_rate(exchange, symbol).await?;
        Ok(fresh)
    }

    /// Force refresh – bypass cache & fetch directly from exchange.
    pub async fn refresh_funding_rate(
        &self,
        exchange: &ExchangeIdEnum,
        symbol: &str,
    ) -> ArbitrageResult<FundingRateInfo> {
        let rate_info = self
            .exchange_service
            .get_funding_rate_direct(exchange.as_str(), symbol)
            .await
            .map_err(|e| {
                ArbitrageError::api_error(format!(
                    "Failed to fetch funding rate from {}: {}",
                    exchange, e
                ))
            })?;

        // Persist in cache (ignore cache errors – non-critical)
        let _ = self
            .cache_manager
            .kv_put(
                &Self::cache_key(exchange, symbol),
                &serde_json::to_string(&rate_info).unwrap_or_default(),
                Some(self.ttl_seconds),
            )
            .await;

        Ok(rate_info)
    }

    /// Build a deterministic cache key for (exchange, symbol).
    fn cache_key(exchange: &ExchangeIdEnum, symbol: &str) -> String {
        format!(
            "funding_rate:{}:{}",
            exchange.as_str().to_lowercase(),
            symbol.to_uppercase()
        )
    }
}

// Tests have been moved to packages/worker/tests/market_data/funding_rate_service_test.rs
