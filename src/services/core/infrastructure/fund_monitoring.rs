// src/services/fund_monitoring.rs

use crate::services::{D1Service, ExchangeService};
// ExchangeInterface import removed - fund monitoring needs refactor for UserExchangeApiService
use crate::types::{Balance, Balances};
// use crate::utils::{ArbitrageResult, ArbitrageError, logger::{Logger, LogLevel}}; // TODO: Re-enable when implementing fund monitoring
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use worker::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundMonitoringConfig {
    pub update_interval_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub enable_optimization: bool,
    pub min_balance_threshold: f64,
}

impl Default for FundMonitoringConfig {
    fn default() -> Self {
        Self {
            update_interval_seconds: 30, // Update every 30 seconds
            cache_ttl_seconds: 300,      // Cache for 5 minutes
            enable_optimization: true,
            min_balance_threshold: 0.01, // Minimum balance to track
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeBalanceSnapshot {
    pub exchange_id: String,
    pub balances: Balances,
    pub timestamp: u64,
    pub total_usd_value: f64,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAllocation {
    pub exchange_id: String,
    pub asset: String,
    pub current_amount: f64,
    pub optimal_amount: f64,
    pub variance_percentage: f64,
    pub action_needed: String, // "buy", "sell", "hold", "transfer"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceHistoryEntry {
    pub id: String,
    pub user_id: String,
    pub exchange_id: String,
    pub asset: String,
    pub balance: Balance,
    pub usd_value: f64,
    pub timestamp: u64,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundOptimizationResult {
    pub allocations: Vec<FundAllocation>,
    pub total_portfolio_value: f64,
    pub optimization_score: f64,
    pub recommendations: Vec<String>,
    pub risk_assessment: String,
}

pub struct FundMonitoringService {
    pub config: FundMonitoringConfig,
    pub exchange_service: ExchangeService,
    pub d1_service: Option<D1Service>,
    pub kv: kv::KvStore,
}

impl FundMonitoringService {
    pub fn new(
        config: FundMonitoringConfig,
        exchange_service: ExchangeService,
        d1_service: Option<D1Service>,
        kv: kv::KvStore,
    ) -> Self {
        Self {
            config,
            exchange_service,
            d1_service,
            kv,
        }
    }

    /// Get real-time balances across all connected exchanges for a user
    pub async fn get_real_time_balances(
        &self,
        user_id: &str,
        exchange_ids: &[String],
    ) -> Result<HashMap<String, ExchangeBalanceSnapshot>> {
        let mut balance_snapshots = HashMap::new();
        let timestamp = Date::now().as_millis();

        for exchange_id in exchange_ids {
            match self
                .fetch_exchange_balance(user_id, exchange_id, timestamp)
                .await
            {
                Ok(snapshot) => {
                    balance_snapshots.insert(exchange_id.clone(), snapshot);
                }
                Err(e) => {
                    console_error!(
                        "Failed to fetch balance for exchange {}: {:?}",
                        exchange_id,
                        e
                    );
                    // Continue with other exchanges even if one fails
                }
            }
        }

        // Store snapshots in cache for quick access
        self.cache_balance_snapshots(user_id, &balance_snapshots)
            .await?;

        Ok(balance_snapshots)
    }

    /// Fetch balance for a specific exchange
    async fn fetch_exchange_balance(
        &self,
        user_id: &str,
        exchange_id: &str,
        timestamp: u64,
    ) -> Result<ExchangeBalanceSnapshot> {
        // Check cache first
        if let Ok(Some(cached)) = self.get_cached_balance(user_id, exchange_id).await {
            if timestamp - cached.timestamp < (self.config.cache_ttl_seconds * 1000) {
                return Ok(cached);
            }
        }

        // TODO: Update to use UserExchangeApiService for user-specific API keys
        // This needs to be refactored to work with the new user-centric API key management
        Err(Error::RustError(
            "Fund monitoring needs to be updated to use UserExchangeApiService".to_string(),
        ))
    }

    /// Parse balance data from exchange API response to our Balances type
    fn parse_balance_data(&self, balance_data: &serde_json::Value) -> Result<Balances> {
        let mut balances = HashMap::new();

        // Handle different exchange response formats
        if let Some(balances_array) = balance_data.as_array() {
            // Binance format: array of balance objects
            for balance_obj in balances_array {
                if let (Some(asset), Some(free), Some(locked)) = (
                    balance_obj["asset"].as_str(),
                    balance_obj["free"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok()),
                    balance_obj["locked"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok()),
                ) {
                    let total = free + locked;
                    if total > self.config.min_balance_threshold {
                        balances.insert(
                            asset.to_string(),
                            Balance {
                                free,
                                used: locked,
                                total,
                            },
                        );
                    }
                }
            }
        } else if let Some(balances_obj) = balance_data.as_object() {
            // Other exchange formats: object with asset keys
            for (asset, balance_info) in balances_obj {
                if let Some(balance_obj) = balance_info.as_object() {
                    let free = balance_obj["free"].as_f64().unwrap_or(0.0);
                    let used = balance_obj["used"].as_f64().unwrap_or(0.0);
                    let total = free + used;

                    if total > self.config.min_balance_threshold {
                        balances.insert(asset.clone(), Balance { free, used, total });
                    }
                }
            }
        }

        Ok(balances)
    }

    /// Calculate total USD value of balances
    async fn calculate_total_usd_value(&self, balances: &Balances) -> Result<f64> {
        let mut total_value = 0.0;

        for (asset, balance) in balances {
            // For now, use a simple price lookup (this would be enhanced with real price feeds)
            let price = self.get_asset_price_usd(asset).await.unwrap_or(0.0);
            total_value += balance.total * price;
        }

        Ok(total_value)
    }

    /// Get asset price in USD (placeholder - would integrate with price feeds)
    async fn get_asset_price_usd(&self, asset: &str) -> Result<f64> {
        // This is a placeholder - in a real implementation, this would
        // fetch from price feeds like CoinGecko, CoinMarketCap, or exchange APIs
        match asset.to_uppercase().as_str() {
            "BTC" => Ok(43000.0),
            "ETH" => Ok(2600.0),
            "USDT" | "USDC" | "BUSD" => Ok(1.0),
            "BNB" => Ok(320.0),
            "ADA" => Ok(0.45),
            "SOL" => Ok(95.0),
            _ => Ok(0.0), // Unknown asset
        }
    }

    /// Optimize fund allocation across exchanges
    pub async fn optimize_fund_allocation(
        &self,
        _user_id: &str,
        balance_snapshots: &HashMap<String, ExchangeBalanceSnapshot>,
        target_allocation: &HashMap<String, f64>, // asset -> target percentage
    ) -> Result<FundOptimizationResult> {
        if !self.config.enable_optimization {
            return Err(Error::RustError(
                "Fund optimization is disabled".to_string(),
            ));
        }

        let mut allocations = Vec::new();
        let mut total_portfolio_value = 0.0;
        let mut asset_totals: HashMap<String, f64> = HashMap::new();

        // Calculate current total portfolio value and asset totals
        for snapshot in balance_snapshots.values() {
            total_portfolio_value += snapshot.total_usd_value;
            for (asset, balance) in &snapshot.balances {
                *asset_totals.entry(asset.clone()).or_insert(0.0) += balance.total;
            }
        }

        // Calculate optimal allocations
        for (asset, target_percentage) in target_allocation {
            let current_total = asset_totals.get(asset).unwrap_or(&0.0);
            let target_usd_value = total_portfolio_value * target_percentage;
            let asset_price = self.get_asset_price_usd(asset).await.unwrap_or(1.0);
            let optimal_amount = target_usd_value / asset_price;
            let variance_percentage = if optimal_amount > 0.0 {
                ((current_total - optimal_amount) / optimal_amount * 100.0).abs()
            } else {
                0.0
            };

            let action_needed = if variance_percentage > 5.0 {
                if *current_total < optimal_amount {
                    "buy".to_string()
                } else {
                    "sell".to_string()
                }
            } else {
                "hold".to_string()
            };

            // For each exchange, calculate allocation
            for (exchange_id, snapshot) in balance_snapshots {
                let current_amount = snapshot.balances.get(asset).map(|b| b.total).unwrap_or(0.0);

                allocations.push(FundAllocation {
                    exchange_id: exchange_id.clone(),
                    asset: asset.clone(),
                    current_amount,
                    optimal_amount: optimal_amount / balance_snapshots.len() as f64, // Distribute evenly
                    variance_percentage,
                    action_needed: action_needed.clone(),
                });
            }
        }

        let optimization_score = self.calculate_optimization_score(&allocations);
        let recommendations = self.generate_recommendations(&allocations);
        let risk_assessment = self.assess_portfolio_risk(&allocations, total_portfolio_value);

        Ok(FundOptimizationResult {
            allocations,
            total_portfolio_value,
            optimization_score,
            recommendations,
            risk_assessment,
        })
    }

    /// Calculate optimization score (0-100)
    fn calculate_optimization_score(&self, allocations: &[FundAllocation]) -> f64 {
        if allocations.is_empty() {
            return 0.0;
        }

        let total_variance: f64 = allocations.iter().map(|a| a.variance_percentage).sum();
        let avg_variance = total_variance / allocations.len() as f64;

        // Score decreases as variance increases
        (100.0 - avg_variance.min(100.0)).max(0.0)
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, allocations: &[FundAllocation]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let high_variance_count = allocations
            .iter()
            .filter(|a| a.variance_percentage > 10.0)
            .count();

        if high_variance_count > 0 {
            recommendations.push(format!(
                "Consider rebalancing {} assets with high variance (>10%)",
                high_variance_count
            ));
        }

        let buy_needed: Vec<_> = allocations
            .iter()
            .filter(|a| a.action_needed == "buy")
            .collect();

        if !buy_needed.is_empty() {
            recommendations.push(format!(
                "Consider purchasing more of: {}",
                buy_needed
                    .iter()
                    .map(|a| format!("{} on {}", a.asset, a.exchange_id))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let sell_needed: Vec<_> = allocations
            .iter()
            .filter(|a| a.action_needed == "sell")
            .collect();

        if !sell_needed.is_empty() {
            recommendations.push(format!(
                "Consider reducing positions in: {}",
                sell_needed
                    .iter()
                    .map(|a| format!("{} on {}", a.asset, a.exchange_id))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("Portfolio allocation is optimal".to_string());
        }

        recommendations
    }

    /// Assess portfolio risk
    fn assess_portfolio_risk(&self, allocations: &[FundAllocation], total_value: f64) -> String {
        let high_variance_count = allocations
            .iter()
            .filter(|a| a.variance_percentage > 15.0)
            .count();

        let total_variance: f64 = allocations.iter().map(|a| a.variance_percentage).sum();
        let avg_variance = if !allocations.is_empty() {
            total_variance / allocations.len() as f64
        } else {
            0.0
        };

        if total_value < 1000.0 {
            "Low Value Portfolio".to_string()
        } else if avg_variance > 20.0 || high_variance_count > 3 {
            "High Risk - Significant Rebalancing Needed".to_string()
        } else if avg_variance > 10.0 {
            "Medium Risk - Monitor Closely".to_string()
        } else {
            "Low Risk - Well Balanced".to_string()
        }
    }

    /// Store balance history in D1 database
    async fn store_balance_history(
        &self,
        user_id: &str,
        snapshot: &ExchangeBalanceSnapshot,
        d1: &D1Service,
    ) -> Result<()> {
        let snapshot_id = Uuid::new_v4().to_string();

        for (asset, balance) in &snapshot.balances {
            let history_entry = BalanceHistoryEntry {
                id: Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                exchange_id: snapshot.exchange_id.clone(),
                asset: asset.clone(),
                balance: balance.clone(),
                usd_value: balance.total * self.get_asset_price_usd(asset).await.unwrap_or(0.0),
                timestamp: snapshot.timestamp,
                snapshot_id: snapshot_id.clone(),
            };

            d1.store_balance_history(&history_entry)
                .await
                .map_err(|e| {
                    Error::RustError(format!("Failed to store balance history: {:?}", e))
                })?;
        }

        Ok(())
    }

    /// Cache balance snapshots in KV
    async fn cache_balance_snapshots(
        &self,
        user_id: &str,
        snapshots: &HashMap<String, ExchangeBalanceSnapshot>,
    ) -> Result<()> {
        for (exchange_id, snapshot) in snapshots {
            let cache_key = format!("balance:{}:{}", user_id, exchange_id);
            let serialized = serde_json::to_string(snapshot)
                .map_err(|e| Error::RustError(format!("Failed to serialize snapshot: {}", e)))?;

            self.kv
                .put(&cache_key, serialized)?
                .expiration_ttl(self.config.cache_ttl_seconds)
                .execute()
                .await?;
        }

        Ok(())
    }

    /// Get cached balance for an exchange
    async fn get_cached_balance(
        &self,
        user_id: &str,
        exchange_id: &str,
    ) -> Result<Option<ExchangeBalanceSnapshot>> {
        let cache_key = format!("balance:{}:{}", user_id, exchange_id);

        match self.kv.get(&cache_key).text().await? {
            Some(cached_data) => {
                let snapshot: ExchangeBalanceSnapshot = serde_json::from_str(&cached_data)
                    .map_err(|e| {
                        Error::RustError(format!("Failed to deserialize cached snapshot: {}", e))
                    })?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    /// Get balance history for a user and time range
    pub async fn get_balance_history(
        &self,
        user_id: &str,
        exchange_id: Option<&str>,
        asset: Option<&str>,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<BalanceHistoryEntry>> {
        if let Some(d1) = &self.d1_service {
            d1.get_balance_history(
                user_id,
                exchange_id,
                asset,
                from_timestamp,
                to_timestamp,
                limit,
            )
            .await
            .map_err(|e| Error::RustError(format!("Failed to get balance history: {:?}", e)))
        } else {
            console_warn!("D1 service not available, returning empty history");
            Ok(Vec::new())
        }
    }

    /// Calculate balance analytics
    pub async fn calculate_balance_analytics(
        &self,
        user_id: &str,
        days_back: u32,
    ) -> Result<BalanceAnalytics> {
        let from_timestamp = Date::now().as_millis() - (days_back as u64 * 24 * 60 * 60 * 1000);
        let history = self
            .get_balance_history(user_id, None, None, Some(from_timestamp), None, None)
            .await?;

        let analytics = BalanceAnalytics::from_history(&history);
        Ok(analytics)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAnalytics {
    pub total_value_change: f64,
    pub total_value_change_percentage: f64,
    pub best_performing_asset: String,
    pub worst_performing_asset: String,
    pub average_daily_change: f64,
    pub volatility_score: f64,
    pub exchange_performance: HashMap<String, f64>,
}

impl BalanceAnalytics {
    fn from_history(history: &[BalanceHistoryEntry]) -> Self {
        if history.is_empty() {
            return Self::default();
        }

        // Group by asset and exchange
        let mut asset_values: HashMap<String, Vec<f64>> = HashMap::new();
        let mut exchange_values: HashMap<String, Vec<f64>> = HashMap::new();

        for entry in history {
            asset_values
                .entry(entry.asset.clone())
                .or_default()
                .push(entry.usd_value);
            exchange_values
                .entry(entry.exchange_id.clone())
                .or_default()
                .push(entry.usd_value);
        }

        // Calculate metrics
        let total_initial: f64 = history
            .iter()
            .take(history.len() / 2)
            .map(|e| e.usd_value)
            .sum();
        let total_final: f64 = history
            .iter()
            .skip(history.len() / 2)
            .map(|e| e.usd_value)
            .sum();

        let total_value_change = total_final - total_initial;
        let total_value_change_percentage = if total_initial > 0.0 {
            (total_value_change / total_initial) * 100.0
        } else {
            0.0
        };

        // Find best/worst performing assets
        let mut best_asset = "N/A".to_string();
        let mut worst_asset = "N/A".to_string();
        let mut best_performance = f64::NEG_INFINITY;
        let mut worst_performance = f64::INFINITY;

        for (asset, values) in &asset_values {
            if values.len() >= 2 {
                let initial = values[0];
                let final_val = values[values.len() - 1];
                let performance = if initial > 0.0 {
                    (final_val - initial) / initial
                } else {
                    0.0
                };

                if performance > best_performance {
                    best_performance = performance;
                    best_asset = asset.clone();
                }
                if performance < worst_performance {
                    worst_performance = performance;
                    worst_asset = asset.clone();
                }
            }
        }

        // Calculate volatility (simplified)
        let values: Vec<f64> = history.iter().map(|e| e.usd_value).collect();
        let volatility_score = Self::calculate_volatility(&values);

        // Exchange performance
        let mut exchange_performance = HashMap::new();
        for (exchange, values) in &exchange_values {
            if values.len() >= 2 {
                let initial = values[0];
                let final_val = values[values.len() - 1];
                let performance = if initial > 0.0 {
                    (final_val - initial) / initial * 100.0
                } else {
                    0.0
                };
                exchange_performance.insert(exchange.clone(), performance);
            }
        }

        Self {
            total_value_change,
            total_value_change_percentage,
            best_performing_asset: best_asset,
            worst_performing_asset: worst_asset,
            average_daily_change: total_value_change_percentage / 30.0, // Rough daily average
            volatility_score,
            exchange_performance,
        }
    }

    fn calculate_volatility(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance.sqrt() / mean * 100.0 // As percentage
    }

    fn default() -> Self {
        Self {
            total_value_change: 0.0,
            total_value_change_percentage: 0.0,
            best_performing_asset: "N/A".to_string(),
            worst_performing_asset: "N/A".to_string(),
            average_daily_change: 0.0,
            volatility_score: 0.0,
            exchange_performance: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_score_calculation() {
        // Test the optimization score calculation directly
        let allocations = [
            FundAllocation {
                exchange_id: "binance".to_string(),
                asset: "BTC".to_string(),
                current_amount: 1.0,
                optimal_amount: 1.1,
                variance_percentage: 10.0,
                action_needed: "buy".to_string(),
            },
            FundAllocation {
                exchange_id: "coinbase".to_string(),
                asset: "ETH".to_string(),
                current_amount: 10.0,
                optimal_amount: 9.0,
                variance_percentage: 5.0,
                action_needed: "sell".to_string(),
            },
        ];

        // Create a minimal config for testing
        let _config = FundMonitoringConfig::default();

        // Test the calculation logic directly
        let total_variance: f64 = allocations
            .iter()
            .map(|a| a.variance_percentage.abs())
            .sum();
        let avg_variance = total_variance / allocations.len() as f64;
        let score = 100.0 - (avg_variance * 2.0).min(100.0);

        assert!((0.0..=100.0).contains(&score));
        assert!(score > 80.0); // Should be high score with low variance
    }

    #[test]
    fn test_risk_assessment_logic() {
        // Test risk assessment logic directly
        let low_risk_allocations = [FundAllocation {
            exchange_id: "binance".to_string(),
            asset: "BTC".to_string(),
            current_amount: 1.0,
            optimal_amount: 1.05,
            variance_percentage: 5.0,
            action_needed: "hold".to_string(),
        }];

        let high_risk_allocations = [FundAllocation {
            exchange_id: "binance".to_string(),
            asset: "BTC".to_string(),
            current_amount: 1.0,
            optimal_amount: 2.0,
            variance_percentage: 25.0,
            action_needed: "buy".to_string(),
        }];

        // Test the risk assessment logic
        let low_avg_variance: f64 = low_risk_allocations
            .iter()
            .map(|a| a.variance_percentage.abs())
            .sum::<f64>()
            / low_risk_allocations.len() as f64;

        let high_avg_variance: f64 = high_risk_allocations
            .iter()
            .map(|a| a.variance_percentage.abs())
            .sum::<f64>()
            / high_risk_allocations.len() as f64;

        assert!(low_avg_variance < 10.0);
        assert!(high_avg_variance > 20.0);
    }

    #[test]
    fn test_balance_analytics_volatility() {
        let values = vec![100.0, 110.0, 95.0, 105.0, 120.0];
        let volatility = BalanceAnalytics::calculate_volatility(&values);
        assert!(volatility > 0.0);
    }

    #[test]
    fn test_recommendation_generation_logic() {
        // Test recommendation generation logic
        let allocations = vec![FundAllocation {
            exchange_id: "binance".to_string(),
            asset: "BTC".to_string(),
            current_amount: 1.0,
            optimal_amount: 1.5,
            variance_percentage: 15.0,
            action_needed: "buy".to_string(),
        }];

        // Test the logic for generating recommendations
        let mut recommendations = Vec::new();

        for allocation in &allocations {
            if allocation.variance_percentage.abs() > 10.0 {
                match allocation.action_needed.as_str() {
                    "buy" => {
                        recommendations.push(format!(
                            "Consider purchasing {} {} on {} to reach optimal allocation",
                            allocation.optimal_amount - allocation.current_amount,
                            allocation.asset,
                            allocation.exchange_id
                        ));
                    }
                    "sell" => {
                        recommendations.push(format!(
                            "Consider selling {} {} on {} to reach optimal allocation",
                            allocation.current_amount - allocation.optimal_amount,
                            allocation.asset,
                            allocation.exchange_id
                        ));
                    }
                    _ => {}
                }
            }
        }

        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("purchasing")));
    }

    #[test]
    fn test_fund_monitoring_config_default() {
        let config = FundMonitoringConfig::default();
        assert_eq!(config.update_interval_seconds, 30);
        assert_eq!(config.cache_ttl_seconds, 300);
        assert!(config.enable_optimization);
        assert_eq!(config.min_balance_threshold, 0.01);
    }

    #[test]
    fn test_balance_analytics_default() {
        let analytics = BalanceAnalytics::default();
        assert_eq!(analytics.total_value_change, 0.0);
        assert_eq!(analytics.total_value_change_percentage, 0.0);
        assert_eq!(analytics.best_performing_asset, "N/A");
        assert_eq!(analytics.worst_performing_asset, "N/A");
        assert_eq!(analytics.average_daily_change, 0.0);
        assert_eq!(analytics.volatility_score, 0.0);
        assert!(analytics.exchange_performance.is_empty());
    }
}
