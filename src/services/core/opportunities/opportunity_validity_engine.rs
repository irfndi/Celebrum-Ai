use std::sync::Arc;

use crate::services::core::infrastructure::database_repositories::DatabaseManager;
use crate::services::core::market_data::funding_rate_service::FundingRateService;
use crate::services::core::trading::exchange::ExchangeService;
use crate::types::ArbitrageOpportunity;
use crate::utils::{ArbitrageError, ArbitrageResult};

/// Configuration for validity evaluation thresholds.
#[derive(Clone)]
pub struct ValidityConfig {
    /// Minimum profit % required to remain valid.
    pub min_profit_percent: f64,
    /// Funding-rate differential threshold: if current diff < this ⇒ invalid.
    pub min_funding_rate_diff: f64,
}

impl Default for ValidityConfig {
    fn default() -> Self {
        Self {
            min_profit_percent: 0.1,     // 0.1%
            min_funding_rate_diff: 0.02, // 0.02 (2%) annualised diff
        }
    }
}

/// Engine that periodically checks stored opportunities and marks them
/// `invalid` or `expired` according to profitability / funding-rate refreshes.
pub struct OpportunityValidityEngine {
    db: DatabaseManager,
    funding_service: FundingRateService,
    config: ValidityConfig,
}

impl OpportunityValidityEngine {
    pub fn new(
        db: DatabaseManager,
        exchange_service: Arc<ExchangeService>,
        cache_manager: Arc<crate::services::CacheManager>,
        config: Option<ValidityConfig>,
    ) -> Self {
        let funding_service = FundingRateService::new(exchange_service, cache_manager, None);
        Self {
            db,
            funding_service,
            config: config.unwrap_or_default(),
        }
    }

    /// Refresh validity for all non-expired opportunities.
    pub async fn refresh_all(&self) -> ArbitrageResult<u32> {
        let opportunities = self.db.get_all_opportunities().await?;
        let mut updated = 0u32;
        for opp in opportunities {
            if !self.evaluate_validity(&opp).await? {
                self.mark_invalid(&opp).await?;
                updated += 1;
            }
        }
        Ok(updated)
    }

    /// Evaluate a single opportunity – returns `true` if still valid.
    async fn evaluate_validity(&self, opportunity: &ArbitrageOpportunity) -> ArbitrageResult<bool> {
        // Check expiry first
        if let Some(exp) = opportunity.expires_at {
            let now = chrono::Utc::now().timestamp_millis() as u64;
            if now > exp {
                return Ok(false);
            }
        }

        // Check profit percent
        if opportunity.rate_difference * 100.0 < self.config.min_profit_percent {
            return Ok(false);
        }

        // Funding-rate check (only if rates present)
        if let (Some(long_rate), Some(short_rate)) = (opportunity.long_rate, opportunity.short_rate)
        {
            if (short_rate - long_rate).abs() < self.config.min_funding_rate_diff {
                return Ok(false);
            }
        } else {
            // Attempt live fetch if missing
            let long_rate_info = self
                .funding_service
                .get_funding_rate(&opportunity.long_exchange, &opportunity.pair)
                .await?
                .funding_rate;
            let short_rate_info = self
                .funding_service
                .get_funding_rate(&opportunity.short_exchange, &opportunity.pair)
                .await?
                .funding_rate;
            if (short_rate_info - long_rate_info).abs() < self.config.min_funding_rate_diff {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Mark opportunity invalid in DB (soft update).
    async fn mark_invalid(&self, opportunity: &ArbitrageOpportunity) -> ArbitrageResult<()> {
        let query = "UPDATE opportunities SET is_valid = 0 WHERE id = ?";
        let stmt = self.db.prepare(query);
        stmt.bind(&[opportunity.id.clone().into()])
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;
        Ok(())
    }
}
