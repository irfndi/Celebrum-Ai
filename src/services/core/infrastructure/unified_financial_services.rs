// Unified Financial Services - Consolidated Financial Management System
// Consolidates: financial_module (4 files → 1 unified module)

use crate::types::{Balance, Balances};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use worker::Env;

// =============================================================================
// Core Financial Types
// =============================================================================

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
    pub action_needed: String,
    pub priority: String,
    pub estimated_impact: f64,
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
    pub expected_improvement: f64,
    pub implementation_priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnalytics {
    pub total_value_usd: f64,
    pub total_value_change_24h: f64,
    pub total_value_change_percentage: f64,
    pub best_performing_asset: String,
    pub worst_performing_asset: String,
    pub portfolio_diversity_score: f64,
    pub risk_score: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub exchange_distribution: HashMap<String, f64>,
    pub asset_distribution: HashMap<String, f64>,
}

// =============================================================================
// Balance Tracker
// =============================================================================

#[derive(Debug, Clone)]
pub struct BalanceTrackerConfig {
    pub enable_real_time_tracking: bool,
    pub enable_historical_data: bool,
    pub enable_cross_exchange_analysis: bool,
    pub update_interval_seconds: u64,
    pub history_retention_days: u32,
    pub max_exchanges_per_user: u32,
    pub enable_anomaly_detection: bool,
    pub anomaly_threshold_percentage: f64,
}

impl Default for BalanceTrackerConfig {
    fn default() -> Self {
        Self {
            enable_real_time_tracking: true,
            enable_historical_data: true,
            enable_cross_exchange_analysis: true,
            update_interval_seconds: 30,
            history_retention_days: 90,
            max_exchanges_per_user: 10,
            enable_anomaly_detection: true,
            anomaly_threshold_percentage: 10.0,
        }
    }
}

impl BalanceTrackerConfig {
    pub fn high_performance() -> Self {
        Self {
            update_interval_seconds: 10,
            history_retention_days: 30,
            max_exchanges_per_user: 20,
            ..Self::default()
        }
    }

    pub fn high_reliability() -> Self {
        Self {
            update_interval_seconds: 60,
            history_retention_days: 180,
            max_exchanges_per_user: 5,
            enable_anomaly_detection: true,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.update_interval_seconds == 0 {
            return Err(ArbitrageError::configuration_error(
                "update_interval_seconds must be greater than 0".to_string(),
            ));
        }
        if self.max_exchanges_per_user == 0 {
            return Err(ArbitrageError::configuration_error(
                "max_exchanges_per_user must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceTrackerHealth {
    pub is_healthy: bool,
    pub active_users: u64,
    pub monitored_exchanges: u64,
    pub total_tracked_balances: u64,
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub last_update_timestamp: u64,
    pub average_update_latency_ms: f64,
    pub anomalies_detected: u64,
    pub last_health_check: u64,
}

impl Default for BalanceTrackerHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            active_users: 0,
            monitored_exchanges: 0,
            total_tracked_balances: 0,
            successful_updates: 0,
            failed_updates: 0,
            last_update_timestamp: 0,
            average_update_latency_ms: 0.0,
            anomalies_detected: 0,
            last_health_check: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceTrackerMetrics {
    pub total_balance_updates: u64,
    pub updates_per_second: f64,
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub average_processing_time_ms: f64,
    pub exchanges_by_status: HashMap<String, u64>,
    pub users_by_activity: HashMap<String, u64>,
    pub balance_changes_24h: HashMap<String, f64>,
    pub anomaly_alerts: u64,
    pub last_updated: u64,
}

impl Default for BalanceTrackerMetrics {
    fn default() -> Self {
        Self {
            total_balance_updates: 0,
            updates_per_second: 0.0,
            successful_updates: 0,
            failed_updates: 0,
            average_processing_time_ms: 0.0,
            exchanges_by_status: HashMap::new(),
            users_by_activity: HashMap::new(),
            balance_changes_24h: HashMap::new(),
            anomaly_alerts: 0,
            last_updated: 0,
        }
    }
}

#[allow(dead_code)]
pub struct BalanceTracker {
    config: BalanceTrackerConfig,
    health: Arc<Mutex<BalanceTrackerHealth>>,
    metrics: Arc<Mutex<BalanceTrackerMetrics>>,
    current_balances: Arc<Mutex<HashMap<String, HashMap<String, ExchangeBalanceSnapshot>>>>,
    balance_history: Arc<Mutex<HashMap<String, Vec<BalanceHistoryEntry>>>>,
    startup_time: u64,
}

impl BalanceTracker {
    pub fn new(config: BalanceTrackerConfig) -> ArbitrageResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            health: Arc::new(Mutex::new(BalanceTrackerHealth::default())),
            metrics: Arc::new(Mutex::new(BalanceTrackerMetrics::default())),
            current_balances: Arc::new(Mutex::new(HashMap::new())),
            balance_history: Arc::new(Mutex::new(HashMap::new())),
            startup_time: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    pub async fn track_balance(
        &self,
        user_id: &str,
        exchange_id: &str,
        balances: Balances,
    ) -> ArbitrageResult<ExchangeBalanceSnapshot> {
        let snapshot = ExchangeBalanceSnapshot {
            exchange_id: exchange_id.to_string(),
            balances: balances.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            total_usd_value: self.calculate_total_usd_value(&balances).await?,
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        {
            let mut current_balances = self.current_balances.lock().unwrap();
            current_balances
                .entry(user_id.to_string())
                .or_default()
                .insert(exchange_id.to_string(), snapshot.clone());
        }

        if self.config.enable_historical_data {
            self.add_to_history(user_id, &snapshot).await?;
        }

        self.update_metrics(&snapshot).await;
        Ok(snapshot)
    }

    async fn calculate_total_usd_value(&self, balances: &Balances) -> ArbitrageResult<f64> {
        let mut total_value = 0.0;
        for (asset, balance) in balances {
            let usd_price = self.get_asset_price_usd(asset).await.unwrap_or(0.0);
            total_value += balance.free * usd_price;
            total_value += balance.used * usd_price;
        }
        Ok(total_value)
    }

    async fn get_asset_price_usd(&self, _asset: &str) -> ArbitrageResult<f64> {
        // In real implementation, fetch from price feed
        Ok(1.0)
    }

    async fn add_to_history(
        &self,
        user_id: &str,
        snapshot: &ExchangeBalanceSnapshot,
    ) -> ArbitrageResult<()> {
        let mut history = self.balance_history.lock().unwrap();
        let user_history = history.entry(user_id.to_string()).or_default();

        for (asset, balance) in &snapshot.balances {
            let entry = BalanceHistoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                exchange_id: snapshot.exchange_id.clone(),
                asset: asset.clone(),
                balance: balance.clone(),
                usd_value: balance.free + balance.used,
                timestamp: snapshot.timestamp,
                snapshot_id: format!("{}:{}", snapshot.exchange_id, snapshot.timestamp),
            };
            user_history.push(entry);
        }

        // Cleanup old entries
        let cutoff_time = chrono::Utc::now().timestamp_millis() as u64
            - (self.config.history_retention_days as u64 * 24 * 60 * 60 * 1000);
        user_history.retain(|entry| entry.timestamp > cutoff_time);

        Ok(())
    }

    async fn update_metrics(&self, _snapshot: &ExchangeBalanceSnapshot) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_balance_updates += 1;
        metrics.successful_updates += 1;
        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    pub async fn get_current_balances(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<HashMap<String, ExchangeBalanceSnapshot>> {
        let current_balances = self.current_balances.lock().unwrap();
        Ok(current_balances.get(user_id).cloned().unwrap_or_default())
    }

    pub async fn get_balance_history(
        &self,
        user_id: &str,
        exchange_id: Option<&str>,
        limit: Option<u32>,
    ) -> ArbitrageResult<Vec<BalanceHistoryEntry>> {
        let history = self.balance_history.lock().unwrap();
        let user_history = history.get(user_id).cloned().unwrap_or_default();

        let mut filtered_history: Vec<_> = user_history
            .into_iter()
            .filter(|entry| exchange_id.is_none_or(|ex_id| entry.exchange_id == ex_id))
            .collect();

        filtered_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered_history.truncate(limit as usize);
        }

        Ok(filtered_history)
    }

    pub async fn health_check(&self) -> ArbitrageResult<BalanceTrackerHealth> {
        let mut health = self.health.lock().unwrap();
        health.is_healthy = true;
        health.last_health_check = chrono::Utc::now().timestamp_millis() as u64;
        Ok(health.clone())
    }

    pub async fn get_metrics(&self) -> BalanceTrackerMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

// =============================================================================
// Fund Analyzer
// =============================================================================

#[derive(Debug, Clone)]
pub struct FundAnalyzerConfig {
    pub enable_portfolio_analysis: bool,
    pub enable_risk_assessment: bool,
    pub enable_performance_tracking: bool,
    pub enable_optimization_recommendations: bool,
    pub analysis_window_hours: u32,
    pub risk_tolerance_level: f64,
    pub minimum_portfolio_value: f64,
    pub rebalancing_threshold_percentage: f64,
}

impl Default for FundAnalyzerConfig {
    fn default() -> Self {
        Self {
            enable_portfolio_analysis: true,
            enable_risk_assessment: true,
            enable_performance_tracking: true,
            enable_optimization_recommendations: true,
            analysis_window_hours: 24,
            risk_tolerance_level: 0.5,
            minimum_portfolio_value: 100.0,
            rebalancing_threshold_percentage: 5.0,
        }
    }
}

impl FundAnalyzerConfig {
    pub fn high_performance() -> Self {
        Self {
            analysis_window_hours: 12,
            rebalancing_threshold_percentage: 3.0,
            ..Self::default()
        }
    }

    pub fn high_reliability() -> Self {
        Self {
            analysis_window_hours: 48,
            risk_tolerance_level: 0.3,
            rebalancing_threshold_percentage: 10.0,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.analysis_window_hours == 0 {
            return Err(ArbitrageError::configuration_error(
                "analysis_window_hours must be greater than 0".to_string(),
            ));
        }
        if self.risk_tolerance_level < 0.0 || self.risk_tolerance_level > 1.0 {
            return Err(ArbitrageError::configuration_error(
                "risk_tolerance_level must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAnalyzerHealth {
    pub is_healthy: bool,
    pub analyses_completed: u64,
    pub portfolios_analyzed: u64,
    pub optimization_recommendations: u64,
    pub risk_assessments_completed: u64,
    pub average_analysis_time_ms: f64,
    pub last_analysis_timestamp: u64,
    pub last_health_check: u64,
}

impl Default for FundAnalyzerHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            analyses_completed: 0,
            portfolios_analyzed: 0,
            optimization_recommendations: 0,
            risk_assessments_completed: 0,
            average_analysis_time_ms: 0.0,
            last_analysis_timestamp: 0,
            last_health_check: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAnalyzerMetrics {
    pub total_analyses: u64,
    pub analyses_per_hour: f64,
    pub successful_analyses: u64,
    pub failed_analyses: u64,
    pub average_portfolio_value: f64,
    pub portfolios_by_risk_level: HashMap<String, u64>,
    pub asset_allocation_efficiency: f64,
    pub rebalancing_recommendations: u64,
    pub last_updated: u64,
}

impl Default for FundAnalyzerMetrics {
    fn default() -> Self {
        Self {
            total_analyses: 0,
            analyses_per_hour: 0.0,
            successful_analyses: 0,
            failed_analyses: 0,
            average_portfolio_value: 0.0,
            portfolios_by_risk_level: HashMap::new(),
            asset_allocation_efficiency: 0.0,
            rebalancing_recommendations: 0,
            last_updated: 0,
        }
    }
}

#[allow(dead_code)]
pub struct FundAnalyzer {
    config: FundAnalyzerConfig,
    health: Arc<Mutex<FundAnalyzerHealth>>,
    metrics: Arc<Mutex<FundAnalyzerMetrics>>,
    startup_time: u64,
}

impl FundAnalyzer {
    pub fn new(config: FundAnalyzerConfig) -> ArbitrageResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            health: Arc::new(Mutex::new(FundAnalyzerHealth::default())),
            metrics: Arc::new(Mutex::new(FundAnalyzerMetrics::default())),
            startup_time: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    pub async fn analyze_portfolio(
        &self,
        _user_id: &str,
        snapshots: &HashMap<String, ExchangeBalanceSnapshot>,
    ) -> ArbitrageResult<PortfolioAnalytics> {
        let start_time = chrono::Utc::now().timestamp_millis() as u64;

        let mut total_value = 0.0;
        let mut exchange_distribution = HashMap::new();
        let mut asset_distribution = HashMap::new();

        for (exchange_id, snapshot) in snapshots {
            total_value += snapshot.total_usd_value;
            exchange_distribution.insert(exchange_id.clone(), snapshot.total_usd_value);

            for (asset, balance) in &snapshot.balances {
                let asset_value = balance.free + balance.used;
                *asset_distribution.entry(asset.clone()).or_insert(0.0) += asset_value;
            }
        }

        let diversity_score = self.calculate_diversity_score(&asset_distribution);
        let risk_score = self.calculate_risk_score(&asset_distribution);

        let analytics = PortfolioAnalytics {
            total_value_usd: total_value,
            total_value_change_24h: 0.0, // Would calculate from historical data
            total_value_change_percentage: 0.0,
            best_performing_asset: self.find_best_performing_asset(&asset_distribution),
            worst_performing_asset: self.find_worst_performing_asset(&asset_distribution),
            portfolio_diversity_score: diversity_score,
            risk_score,
            sharpe_ratio: 0.0, // Would calculate based on returns and volatility
            max_drawdown: 0.0,
            exchange_distribution,
            asset_distribution,
        };

        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        self.update_metrics(end_time - start_time).await;

        Ok(analytics)
    }

    fn calculate_diversity_score(&self, asset_distribution: &HashMap<String, f64>) -> f64 {
        if asset_distribution.is_empty() {
            return 0.0;
        }

        let total_value: f64 = asset_distribution.values().sum();
        if total_value == 0.0 {
            return 0.0;
        }

        let mut herfindahl_index = 0.0;
        for value in asset_distribution.values() {
            let share = value / total_value;
            herfindahl_index += share * share;
        }

        // Convert to diversity score (0-1, higher is more diverse)
        1.0 - herfindahl_index
    }

    fn calculate_risk_score(&self, asset_distribution: &HashMap<String, f64>) -> f64 {
        // Simplified risk calculation based on asset types
        let mut risk_score = 0.0;
        let total_value: f64 = asset_distribution.values().sum();

        if total_value == 0.0 {
            return 0.0;
        }

        for (asset, value) in asset_distribution {
            let asset_risk = match asset.as_str() {
                "BTC" | "ETH" => 0.3,
                "USDT" | "USDC" | "DAI" => 0.1,
                _ => 0.5,
            };
            risk_score += (value / total_value) * asset_risk;
        }

        risk_score
    }

    fn find_best_performing_asset(&self, asset_distribution: &HashMap<String, f64>) -> String {
        asset_distribution
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(asset, _)| asset.clone())
            .unwrap_or_else(|| "N/A".to_string())
    }

    fn find_worst_performing_asset(&self, asset_distribution: &HashMap<String, f64>) -> String {
        asset_distribution
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(asset, _)| asset.clone())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub async fn optimize_allocation(
        &self,
        _user_id: &str,
        current_allocation: &HashMap<String, f64>,
        target_allocation: &HashMap<String, f64>,
    ) -> ArbitrageResult<FundOptimizationResult> {
        let mut allocations = Vec::new();
        let mut total_variance = 0.0;

        for (asset, &target_amount) in target_allocation {
            let current_amount = current_allocation.get(asset).copied().unwrap_or(0.0);
            let variance = ((target_amount - current_amount) / current_amount.max(1.0)) * 100.0;
            total_variance += variance.abs();

            let action = if variance > self.config.rebalancing_threshold_percentage {
                "buy".to_string()
            } else if variance < -self.config.rebalancing_threshold_percentage {
                "sell".to_string()
            } else {
                "hold".to_string()
            };

            allocations.push(FundAllocation {
                exchange_id: "consolidated".to_string(),
                asset: asset.clone(),
                current_amount,
                optimal_amount: target_amount,
                variance_percentage: variance,
                action_needed: action,
                priority: if variance.abs() > 10.0 {
                    "high"
                } else {
                    "medium"
                }
                .to_string(),
                estimated_impact: variance.abs() / 100.0,
            });
        }

        let optimization_score =
            1.0 - (total_variance / (target_allocation.len() as f64 * 100.0)).min(1.0);

        Ok(FundOptimizationResult {
            allocations,
            total_portfolio_value: current_allocation.values().sum(),
            optimization_score,
            recommendations: vec![
                "Consider rebalancing assets with high variance".to_string(),
                "Monitor risk exposure regularly".to_string(),
            ],
            risk_assessment: "Moderate".to_string(),
            expected_improvement: total_variance / target_allocation.len() as f64,
            implementation_priority: if total_variance > 50.0 {
                "high"
            } else {
                "medium"
            }
            .to_string(),
        })
    }

    async fn update_metrics(&self, processing_time: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_analyses += 1;
        metrics.successful_analyses += 1;

        let total = metrics.total_analyses as f64;
        metrics.average_portfolio_value =
            (metrics.average_portfolio_value * (total - 1.0) + processing_time as f64) / total;

        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    pub async fn health_check(&self) -> ArbitrageResult<FundAnalyzerHealth> {
        let mut health = self.health.lock().unwrap();
        health.is_healthy = true;
        health.last_health_check = chrono::Utc::now().timestamp_millis() as u64;
        Ok(health.clone())
    }

    pub async fn get_metrics(&self) -> FundAnalyzerMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

// =============================================================================
// Financial Coordinator
// =============================================================================

#[derive(Debug, Clone)]
pub struct FinancialCoordinatorConfig {
    pub enable_coordinator: bool,
    pub enable_automated_rebalancing: bool,
    pub enable_risk_monitoring: bool,
    pub enable_performance_alerts: bool,
    pub coordination_interval_seconds: u64,
    pub risk_threshold_percentage: f64,
    pub rebalancing_frequency_hours: u32,
    pub minimum_trade_amount: f64,
}

impl Default for FinancialCoordinatorConfig {
    fn default() -> Self {
        Self {
            enable_coordinator: true,
            enable_automated_rebalancing: false,
            enable_risk_monitoring: true,
            enable_performance_alerts: true,
            coordination_interval_seconds: 300,
            risk_threshold_percentage: 15.0,
            rebalancing_frequency_hours: 24,
            minimum_trade_amount: 10.0,
        }
    }
}

impl FinancialCoordinatorConfig {
    pub fn high_performance() -> Self {
        Self {
            coordination_interval_seconds: 60,
            rebalancing_frequency_hours: 6,
            enable_automated_rebalancing: true,
            ..Self::default()
        }
    }

    pub fn high_reliability() -> Self {
        Self {
            coordination_interval_seconds: 600,
            risk_threshold_percentage: 10.0,
            enable_automated_rebalancing: false,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.coordination_interval_seconds == 0 {
            return Err(ArbitrageError::configuration_error(
                "coordination_interval_seconds must be greater than 0".to_string(),
            ));
        }
        if self.risk_threshold_percentage < 0.0 || self.risk_threshold_percentage > 100.0 {
            return Err(ArbitrageError::configuration_error(
                "risk_threshold_percentage must be between 0.0 and 100.0".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialCoordinatorHealth {
    pub is_healthy: bool,
    pub balance_tracker_healthy: bool,
    pub fund_analyzer_healthy: bool,
    pub coordination_cycles_completed: u64,
    pub rebalancing_operations: u64,
    pub risk_alerts_triggered: u64,
    pub last_coordination_timestamp: u64,
    pub average_coordination_time_ms: f64,
    pub last_health_check: u64,
}

impl Default for FinancialCoordinatorHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            balance_tracker_healthy: false,
            fund_analyzer_healthy: false,
            coordination_cycles_completed: 0,
            rebalancing_operations: 0,
            risk_alerts_triggered: 0,
            last_coordination_timestamp: 0,
            average_coordination_time_ms: 0.0,
            last_health_check: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialCoordinatorMetrics {
    pub total_coordination_cycles: u64,
    pub successful_cycles: u64,
    pub failed_cycles: u64,
    pub rebalancing_recommendations: u64,
    pub risk_assessments_completed: u64,
    pub portfolio_optimizations: u64,
    pub average_processing_time_ms: f64,
    pub coordination_efficiency: f64,
    pub last_updated: u64,
}

impl Default for FinancialCoordinatorMetrics {
    fn default() -> Self {
        Self {
            total_coordination_cycles: 0,
            successful_cycles: 0,
            failed_cycles: 0,
            rebalancing_recommendations: 0,
            risk_assessments_completed: 0,
            portfolio_optimizations: 0,
            average_processing_time_ms: 0.0,
            coordination_efficiency: 0.0,
            last_updated: 0,
        }
    }
}

#[allow(dead_code)]
pub struct FinancialCoordinator {
    config: FinancialCoordinatorConfig,
    balance_tracker: Arc<BalanceTracker>,
    fund_analyzer: Arc<FundAnalyzer>,
    health: Arc<Mutex<FinancialCoordinatorHealth>>,
    metrics: Arc<Mutex<FinancialCoordinatorMetrics>>,
    startup_time: u64,
}

impl FinancialCoordinator {
    pub fn new(
        config: FinancialCoordinatorConfig,
        balance_tracker: Arc<BalanceTracker>,
        fund_analyzer: Arc<FundAnalyzer>,
    ) -> ArbitrageResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            balance_tracker,
            fund_analyzer,
            health: Arc::new(Mutex::new(FinancialCoordinatorHealth::default())),
            metrics: Arc::new(Mutex::new(FinancialCoordinatorMetrics::default())),
            startup_time: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    pub async fn coordinate_financial_operations(&self, user_id: &str) -> ArbitrageResult<()> {
        let start_time = chrono::Utc::now().timestamp_millis() as u64;

        // Get current balances
        let current_balances = self.balance_tracker.get_current_balances(user_id).await?;

        if current_balances.is_empty() {
            return Ok(());
        }

        // Analyze portfolio
        let analytics = self
            .fund_analyzer
            .analyze_portfolio(user_id, &current_balances)
            .await?;

        // Check for risk alerts
        if analytics.risk_score > self.config.risk_threshold_percentage / 100.0 {
            self.trigger_risk_alert(user_id, &analytics).await?;
        }

        // Check for rebalancing opportunities
        if self.config.enable_automated_rebalancing {
            self.check_rebalancing_opportunities(user_id, &analytics)
                .await?;
        }

        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        self.update_metrics(end_time - start_time, true).await;

        Ok(())
    }

    async fn trigger_risk_alert(
        &self,
        _user_id: &str,
        _analytics: &PortfolioAnalytics,
    ) -> ArbitrageResult<()> {
        let mut health = self.health.lock().unwrap();
        health.risk_alerts_triggered += 1;

        // In real implementation, would send notification
        Ok(())
    }

    async fn check_rebalancing_opportunities(
        &self,
        _user_id: &str,
        _analytics: &PortfolioAnalytics,
    ) -> ArbitrageResult<()> {
        if _analytics.portfolio_diversity_score < 0.5 {
            // Portfolio needs rebalancing
            let mut health = self.health.lock().unwrap();
            health.rebalancing_operations += 1;
        }

        Ok(())
    }

    async fn update_metrics(&self, processing_time: u64, success: bool) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_coordination_cycles += 1;

        if success {
            metrics.successful_cycles += 1;
        } else {
            metrics.failed_cycles += 1;
        }

        let total = metrics.total_coordination_cycles as f64;
        metrics.average_processing_time_ms =
            (metrics.average_processing_time_ms * (total - 1.0) + processing_time as f64) / total;

        metrics.coordination_efficiency = metrics.successful_cycles as f64 / total;
        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn health_check(&self) -> ArbitrageResult<FinancialCoordinatorHealth> {
        let mut health = self.health.lock().unwrap();

        let balance_tracker_health = self.balance_tracker.health_check().await?;
        let fund_analyzer_health = self.fund_analyzer.health_check().await?;

        health.balance_tracker_healthy = balance_tracker_health.is_healthy;
        health.fund_analyzer_healthy = fund_analyzer_health.is_healthy;
        health.is_healthy = health.balance_tracker_healthy && health.fund_analyzer_healthy;
        health.last_health_check = chrono::Utc::now().timestamp_millis() as u64;

        Ok(health.clone())
    }

    pub async fn get_metrics(&self) -> FinancialCoordinatorMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

// =============================================================================
// Unified Financial Services
// =============================================================================

#[derive(Debug, Clone)]
pub struct UnifiedFinancialServicesConfig {
    pub enable_financial_services: bool,
    pub enable_high_performance_mode: bool,
    pub enable_high_reliability_mode: bool,
    pub balance_tracker_config: BalanceTrackerConfig,
    pub fund_analyzer_config: FundAnalyzerConfig,
    pub financial_coordinator_config: FinancialCoordinatorConfig,
}

impl Default for UnifiedFinancialServicesConfig {
    fn default() -> Self {
        Self {
            enable_financial_services: true,
            enable_high_performance_mode: false,
            enable_high_reliability_mode: true,
            balance_tracker_config: BalanceTrackerConfig::default(),
            fund_analyzer_config: FundAnalyzerConfig::default(),
            financial_coordinator_config: FinancialCoordinatorConfig::default(),
        }
    }
}

impl UnifiedFinancialServicesConfig {
    pub fn high_performance() -> Self {
        Self {
            enable_financial_services: true,
            enable_high_performance_mode: true,
            enable_high_reliability_mode: false,
            balance_tracker_config: BalanceTrackerConfig::high_performance(),
            fund_analyzer_config: FundAnalyzerConfig::high_performance(),
            financial_coordinator_config: FinancialCoordinatorConfig::high_performance(),
        }
    }

    pub fn high_reliability() -> Self {
        Self {
            enable_financial_services: true,
            enable_high_performance_mode: false,
            enable_high_reliability_mode: true,
            balance_tracker_config: BalanceTrackerConfig::high_reliability(),
            fund_analyzer_config: FundAnalyzerConfig::high_reliability(),
            financial_coordinator_config: FinancialCoordinatorConfig::high_reliability(),
        }
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        self.balance_tracker_config.validate()?;
        self.fund_analyzer_config.validate()?;
        self.financial_coordinator_config.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedFinancialServicesHealth {
    pub is_healthy: bool,
    pub balance_tracker_healthy: bool,
    pub fund_analyzer_healthy: bool,
    pub financial_coordinator_healthy: bool,
    pub total_users_monitored: u64,
    pub total_portfolios_analyzed: u64,
    pub total_optimizations_completed: u64,
    pub average_processing_time_ms: f64,
    pub last_health_check: u64,
}

impl Default for UnifiedFinancialServicesHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            balance_tracker_healthy: false,
            fund_analyzer_healthy: false,
            financial_coordinator_healthy: false,
            total_users_monitored: 0,
            total_portfolios_analyzed: 0,
            total_optimizations_completed: 0,
            average_processing_time_ms: 0.0,
            last_health_check: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedFinancialServicesMetrics {
    pub total_balance_updates: u64,
    pub total_portfolio_analyses: u64,
    pub total_optimizations: u64,
    pub average_portfolio_value: f64,
    pub balance_tracking_efficiency: f64,
    pub analysis_efficiency: f64,
    pub optimization_success_rate: f64,
    pub risk_alerts_triggered: u64,
    pub rebalancing_recommendations: u64,
    pub last_updated: u64,
}

impl Default for UnifiedFinancialServicesMetrics {
    fn default() -> Self {
        Self {
            total_balance_updates: 0,
            total_portfolio_analyses: 0,
            total_optimizations: 0,
            average_portfolio_value: 0.0,
            balance_tracking_efficiency: 0.0,
            analysis_efficiency: 0.0,
            optimization_success_rate: 0.0,
            risk_alerts_triggered: 0,
            rebalancing_recommendations: 0,
            last_updated: 0,
        }
    }
}

pub struct UnifiedFinancialServices {
    config: UnifiedFinancialServicesConfig,
    balance_tracker: Arc<BalanceTracker>,
    fund_analyzer: Arc<FundAnalyzer>,
    financial_coordinator: FinancialCoordinator,
    health: Arc<Mutex<UnifiedFinancialServicesHealth>>,
    metrics: Arc<Mutex<UnifiedFinancialServicesMetrics>>,
    startup_time: u64,
}

impl UnifiedFinancialServices {
    pub async fn new(config: UnifiedFinancialServicesConfig, _env: &Env) -> ArbitrageResult<Self> {
        config.validate()?;

        let balance_tracker = Arc::new(BalanceTracker::new(config.balance_tracker_config.clone())?);
        let fund_analyzer = Arc::new(FundAnalyzer::new(config.fund_analyzer_config.clone())?);
        let financial_coordinator = FinancialCoordinator::new(
            config.financial_coordinator_config.clone(),
            balance_tracker.clone(),
            fund_analyzer.clone(),
        )?;

        Ok(Self {
            config,
            balance_tracker,
            fund_analyzer,
            financial_coordinator,
            health: Arc::new(Mutex::new(UnifiedFinancialServicesHealth::default())),
            metrics: Arc::new(Mutex::new(UnifiedFinancialServicesMetrics::default())),
            startup_time: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    pub async fn track_balance(
        &self,
        user_id: &str,
        exchange_id: &str,
        balances: Balances,
    ) -> ArbitrageResult<ExchangeBalanceSnapshot> {
        if !self.config.enable_financial_services {
            return Err(ArbitrageError::service_unavailable(
                "Financial services are disabled",
            ));
        }

        let snapshot = self
            .balance_tracker
            .track_balance(user_id, exchange_id, balances)
            .await?;
        self.update_metrics_balance_update().await;
        Ok(snapshot)
    }

    pub async fn analyze_portfolio(&self, user_id: &str) -> ArbitrageResult<PortfolioAnalytics> {
        let current_balances = self.balance_tracker.get_current_balances(user_id).await?;
        let analytics = self
            .fund_analyzer
            .analyze_portfolio(user_id, &current_balances)
            .await?;
        self.update_metrics_portfolio_analysis().await;
        Ok(analytics)
    }

    pub async fn optimize_portfolio(
        &self,
        user_id: &str,
        target_allocation: &HashMap<String, f64>,
    ) -> ArbitrageResult<FundOptimizationResult> {
        let current_balances = self.balance_tracker.get_current_balances(user_id).await?;
        let current_allocation: HashMap<String, f64> = current_balances
            .iter()
            .flat_map(|(_, snapshot)| &snapshot.balances)
            .map(|(asset, balance)| (asset.clone(), balance.free + balance.used))
            .collect();

        let optimization = self
            .fund_analyzer
            .optimize_allocation(user_id, &current_allocation, target_allocation)
            .await?;
        self.update_metrics_optimization().await;
        Ok(optimization)
    }

    pub async fn coordinate_financial_operations(&self, user_id: &str) -> ArbitrageResult<()> {
        self.financial_coordinator
            .coordinate_financial_operations(user_id)
            .await
    }

    pub async fn get_balance_history(
        &self,
        user_id: &str,
        exchange_id: Option<&str>,
        limit: Option<u32>,
    ) -> ArbitrageResult<Vec<BalanceHistoryEntry>> {
        self.balance_tracker
            .get_balance_history(user_id, exchange_id, limit)
            .await
    }

    async fn update_metrics_balance_update(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_balance_updates += 1;
        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    async fn update_metrics_portfolio_analysis(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_portfolio_analyses += 1;
        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    async fn update_metrics_optimization(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_optimizations += 1;
        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn health_check(&self) -> ArbitrageResult<UnifiedFinancialServicesHealth> {
        let mut health = self.health.lock().unwrap();

        let balance_tracker_health = self.balance_tracker.health_check().await?;
        let fund_analyzer_health = self.fund_analyzer.health_check().await?;
        let coordinator_health = self.financial_coordinator.health_check().await?;

        health.balance_tracker_healthy = balance_tracker_health.is_healthy;
        health.fund_analyzer_healthy = fund_analyzer_health.is_healthy;
        health.financial_coordinator_healthy = coordinator_health.is_healthy;

        health.is_healthy = health.balance_tracker_healthy
            && health.fund_analyzer_healthy
            && health.financial_coordinator_healthy;

        health.last_health_check = chrono::Utc::now().timestamp_millis() as u64;

        Ok(health.clone())
    }

    pub async fn get_metrics(&self) -> UnifiedFinancialServicesMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }

    pub fn get_uptime_seconds(&self) -> u64 {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        (now - self.startup_time) / 1000
    }

    pub async fn generate_rebalance_suggestions(
        &self,
        _user_id: &str,
    ) -> Result<Vec<FundOptimizationResult>, ArbitrageError> {
        // Implementation needed
        unimplemented!()
    }

    pub async fn save_portfolio_analytics(
        &self,
        _user_id: &str,
        _analytics: &PortfolioAnalytics,
    ) -> Result<(), ArbitrageError> {
        // Implementation needed
        unimplemented!()
    }

    pub async fn get_portfolio_analytics(
        &self,
        _user_id: &str,
    ) -> Result<Option<PortfolioAnalytics>, ArbitrageError> {
        // Implementation needed
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_financial_services_config_validation() {
        let config = UnifiedFinancialServicesConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_high_performance_config() {
        let config = UnifiedFinancialServicesConfig::high_performance();
        assert!(config.enable_high_performance_mode);
        assert_eq!(config.balance_tracker_config.update_interval_seconds, 10);
    }

    #[test]
    fn test_high_reliability_config() {
        let config = UnifiedFinancialServicesConfig::high_reliability();
        assert!(config.enable_high_reliability_mode);
        assert_eq!(config.balance_tracker_config.update_interval_seconds, 60);
    }

    #[test]
    fn test_balance_tracker_config_validation() {
        let mut config = BalanceTrackerConfig::default();
        assert!(config.validate().is_ok());

        config.update_interval_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_fund_analyzer_config_validation() {
        let mut config = FundAnalyzerConfig::default();
        assert!(config.validate().is_ok());

        config.risk_tolerance_level = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_portfolio_analytics_calculation() {
        let mut asset_distribution = HashMap::new();
        asset_distribution.insert("BTC".to_string(), 50.0);
        asset_distribution.insert("ETH".to_string(), 30.0);
        asset_distribution.insert("USDT".to_string(), 20.0);

        let analyzer = FundAnalyzer::new(FundAnalyzerConfig::default()).unwrap();
        let diversity = analyzer.calculate_diversity_score(&asset_distribution);
        assert!(diversity > 0.5);

        let risk = analyzer.calculate_risk_score(&asset_distribution);
        assert!(risk > 0.1 && risk < 0.5);
    }
}
