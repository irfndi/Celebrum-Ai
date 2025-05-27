use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::services::core::analysis::market_analysis::{MathUtils, PriceSeries};
use crate::services::core::infrastructure::cloudflare_pipelines::CloudflarePipelinesService;
use crate::services::core::user::user_trading_preferences::{TradingFocus, UserTradingPreferences};
use crate::utils::{logger::Logger, ArbitrageResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCorrelationData {
    pub exchange_a: String,
    pub exchange_b: String,
    pub correlation_coefficient: f64,
    pub confidence_level: f64,
    pub data_points: usize,
    pub analysis_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadershipAnalysis {
    pub leading_exchange: String,
    pub following_exchange: String,
    pub lag_seconds: i64,
    pub leadership_strength: f64,
    pub confidence: f64,
    pub analysis_window_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalCorrelation {
    pub exchange_a: String,
    pub exchange_b: String,
    pub rsi_correlation: f64,
    pub sma_correlation: f64,
    pub momentum_correlation: f64,
    pub overall_technical_correlation: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMetrics {
    pub trading_pair: String,
    pub price_correlations: Vec<ExchangeCorrelationData>,
    pub leadership_analysis: Vec<LeadershipAnalysis>,
    pub technical_correlations: Vec<TechnicalCorrelation>,
    pub analysis_timestamp: DateTime<Utc>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone)]
pub struct CorrelationAnalysisConfig {
    pub min_data_points: usize,
    pub max_lag_seconds: i64,
    pub correlation_threshold: f64,
    pub leadership_threshold: f64,
    pub technical_correlation_weight: f64,
    pub confidence_threshold: f64,
}

impl Default for CorrelationAnalysisConfig {
    fn default() -> Self {
        Self {
            min_data_points: 20,
            max_lag_seconds: 300, // 5 minutes
            correlation_threshold: 0.5,
            leadership_threshold: 0.6,
            technical_correlation_weight: 0.3,
            confidence_threshold: 0.7,
        }
    }
}

/// Historical correlation data event for pipeline storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationDataEvent {
    pub correlation_id: String,
    pub trading_pair: String,
    pub exchange_a: String,
    pub exchange_b: String,
    pub correlation_coefficient: f64,
    pub confidence_level: f64,
    pub data_points: usize,
    pub analysis_window_minutes: i64,
    pub timestamp: u64,
    pub data_type: String, // "correlation_analysis"
}

/// Leadership analysis event for pipeline storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadershipAnalysisEvent {
    pub analysis_id: String,
    pub trading_pair: String,
    pub leading_exchange: String,
    pub following_exchange: String,
    pub lag_seconds: i64,
    pub leadership_strength: f64,
    pub confidence: f64,
    pub analysis_window_minutes: i64,
    pub timestamp: u64,
    pub data_type: String, // "leadership_analysis"
}

pub struct CorrelationAnalysisService {
    config: CorrelationAnalysisConfig,
    pipelines_service: Option<CloudflarePipelinesService>, // For historical data consumption and results storage
    logger: Logger,
}

impl CorrelationAnalysisService {
    pub fn new(config: CorrelationAnalysisConfig, logger: Logger) -> Self {
        Self {
            config,
            pipelines_service: None,
            logger,
        }
    }

    /// Set pipelines service for historical data consumption and results storage
    pub fn set_pipelines_service(&mut self, pipelines_service: CloudflarePipelinesService) {
        self.pipelines_service = Some(pipelines_service);
    }

    /// Get historical correlation data from R2 storage via pipelines
    pub async fn get_historical_correlation_data(
        &self,
        trading_pair: &str,
        timeframe_hours: i64,
    ) -> ArbitrageResult<Vec<CorrelationDataEvent>> {
        if let Some(ref _pipelines_service) = self.pipelines_service {
            // In production, this would query R2 storage for historical correlation data
            self.logger.info(&format!(
                "Fetching historical correlation data from R2 for {} over {} hours",
                trading_pair, timeframe_hours
            ));

            // Simulate historical data retrieval from R2
            let historical_data = vec![CorrelationDataEvent {
                correlation_id: uuid::Uuid::new_v4().to_string(),
                trading_pair: trading_pair.to_string(),
                exchange_a: "binance".to_string(),
                exchange_b: "bybit".to_string(),
                correlation_coefficient: 0.85,
                confidence_level: 0.92,
                data_points: 1440, // 24 hours of minute data
                analysis_window_minutes: timeframe_hours * 60,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                data_type: "correlation_analysis".to_string(),
            }];

            Ok(historical_data)
        } else {
            self.logger
                .warn("Pipelines service not available for historical data retrieval");
            Ok(Vec::new())
        }
    }

    /// Store correlation analysis results to pipelines for historical tracking
    pub async fn store_correlation_results_to_pipeline(
        &self,
        correlation_data: &ExchangeCorrelationData,
        trading_pair: &str,
    ) -> ArbitrageResult<()> {
        if let Some(ref _pipelines_service) = self.pipelines_service {
            let _correlation_event = CorrelationDataEvent {
                correlation_id: uuid::Uuid::new_v4().to_string(),
                trading_pair: trading_pair.to_string(),
                exchange_a: correlation_data.exchange_a.clone(),
                exchange_b: correlation_data.exchange_b.clone(),
                correlation_coefficient: correlation_data.correlation_coefficient,
                confidence_level: correlation_data.confidence_level,
                data_points: correlation_data.data_points,
                analysis_window_minutes: 60, // Default 1 hour window
                timestamp: correlation_data.analysis_timestamp.timestamp_millis() as u64,
                data_type: "correlation_analysis".to_string(),
            };

            self.logger.info(&format!(
                "Storing correlation analysis to pipeline: {}/{} correlation: {:.3}",
                correlation_data.exchange_a,
                correlation_data.exchange_b,
                correlation_data.correlation_coefficient
            ));

            // In production, this would send to actual pipelines
        }
        Ok(())
    }

    /// Store leadership analysis results to pipelines
    pub async fn store_leadership_analysis_to_pipeline(
        &self,
        leadership_data: &LeadershipAnalysis,
        trading_pair: &str,
    ) -> ArbitrageResult<()> {
        if let Some(ref _pipelines_service) = self.pipelines_service {
            let _leadership_event = LeadershipAnalysisEvent {
                analysis_id: uuid::Uuid::new_v4().to_string(),
                trading_pair: trading_pair.to_string(),
                leading_exchange: leadership_data.leading_exchange.clone(),
                following_exchange: leadership_data.following_exchange.clone(),
                lag_seconds: leadership_data.lag_seconds,
                leadership_strength: leadership_data.leadership_strength,
                confidence: leadership_data.confidence,
                analysis_window_minutes: leadership_data.analysis_window_minutes,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                data_type: "leadership_analysis".to_string(),
            };

            self.logger.info(&format!(
                "Storing leadership analysis to pipeline: {} leads {} by {}s",
                leadership_data.leading_exchange,
                leadership_data.following_exchange,
                leadership_data.lag_seconds
            ));

            // In production, this would send to actual pipelines
        }
        Ok(())
    }

    /// Calculate price correlation between two exchanges
    pub fn calculate_price_correlation(
        &self,
        exchange_a_data: &PriceSeries,
        exchange_b_data: &PriceSeries,
        exchange_a_name: &str,
        exchange_b_name: &str,
    ) -> Result<ExchangeCorrelationData, String> {
        if exchange_a_data.data_points.len() < self.config.min_data_points
            || exchange_b_data.data_points.len() < self.config.min_data_points
        {
            return Err("Insufficient data points for correlation analysis".to_string());
        }

        // Align timestamps and extract price pairs
        let aligned_pairs = self.align_price_data(exchange_a_data, exchange_b_data)?;

        if aligned_pairs.len() < self.config.min_data_points {
            return Err("Insufficient aligned data points".to_string());
        }

        let prices_a: Vec<f64> = aligned_pairs.iter().map(|(a, _)| *a).collect();
        let prices_b: Vec<f64> = aligned_pairs.iter().map(|(_, b)| *b).collect();

        let correlation = MathUtils::price_correlation(&prices_a, &prices_b)
            .map_err(|e| format!("Correlation calculation error: {}", e))?;
        let confidence = self.calculate_correlation_confidence(&aligned_pairs);

        Ok(ExchangeCorrelationData {
            exchange_a: exchange_a_name.to_string(),
            exchange_b: exchange_b_name.to_string(),
            correlation_coefficient: correlation,
            confidence_level: confidence,
            data_points: aligned_pairs.len(),
            analysis_timestamp: Utc::now(),
        })
    }

    /// Perform lag correlation analysis to detect leadership
    pub fn analyze_exchange_leadership(
        &self,
        leader_data: &PriceSeries,
        follower_data: &PriceSeries,
        leader_name: &str,
        follower_name: &str,
    ) -> Result<LeadershipAnalysis, String> {
        let mut best_lag = 0i64;
        let mut best_correlation = 0.0f64;
        let mut best_confidence = 0.0f64;

        // Test different lag periods
        for lag_seconds in 0..=self.config.max_lag_seconds {
            if let Ok(correlation_data) =
                self.calculate_lagged_correlation(leader_data, follower_data, lag_seconds)
            {
                if correlation_data.correlation > best_correlation {
                    best_correlation = correlation_data.correlation;
                    best_lag = lag_seconds;
                    best_confidence = correlation_data.confidence;
                }
            }
        }

        let leadership_strength = if best_correlation > self.config.leadership_threshold {
            best_correlation
        } else {
            0.0
        };

        Ok(LeadershipAnalysis {
            leading_exchange: leader_name.to_string(),
            following_exchange: follower_name.to_string(),
            lag_seconds: best_lag,
            leadership_strength,
            confidence: best_confidence,
            analysis_window_minutes: self.config.max_lag_seconds / 60,
        })
    }

    /// Calculate technical indicator correlations between exchanges
    pub fn calculate_technical_correlation(
        &self,
        exchange_a_data: &PriceSeries,
        exchange_b_data: &PriceSeries,
        exchange_a_name: &str,
        exchange_b_name: &str,
    ) -> Result<TechnicalCorrelation, String> {
        let prices_a = exchange_a_data.price_values();
        let prices_b = exchange_b_data.price_values();

        if prices_a.len() < 30 || prices_b.len() < 30 {
            return Err("Insufficient data for technical correlation analysis".to_string());
        }

        // Calculate RSI for both exchanges
        let rsi_a = MathUtils::relative_strength_index(&prices_a, 14)
            .map_err(|e| format!("RSI calculation error for exchange A: {}", e))?;
        let rsi_b = MathUtils::relative_strength_index(&prices_b, 14)
            .map_err(|e| format!("RSI calculation error for exchange B: {}", e))?;

        // Calculate SMA for both exchanges
        let sma_a = MathUtils::simple_moving_average(&prices_a, 20)
            .map_err(|e| format!("SMA calculation error for exchange A: {}", e))?;
        let sma_b = MathUtils::simple_moving_average(&prices_b, 20)
            .map_err(|e| format!("SMA calculation error for exchange B: {}", e))?;

        // Calculate momentum for both exchanges
        let momentum_a = self.calculate_momentum(&prices_a, 10)?;
        let momentum_b = self.calculate_momentum(&prices_b, 10)?;

        // Calculate correlations between technical indicators
        let rsi_correlation = MathUtils::price_correlation(&rsi_a, &rsi_b).unwrap_or(0.0);
        let sma_correlation = MathUtils::price_correlation(&sma_a, &sma_b).unwrap_or(0.0);
        let momentum_correlation =
            MathUtils::price_correlation(&momentum_a, &momentum_b).unwrap_or(0.0);

        // Calculate overall technical correlation as weighted average
        let overall_correlation = (rsi_correlation + sma_correlation + momentum_correlation) / 3.0;
        let confidence = self.calculate_technical_correlation_confidence(
            rsi_correlation,
            sma_correlation,
            momentum_correlation,
        );

        Ok(TechnicalCorrelation {
            exchange_a: exchange_a_name.to_string(),
            exchange_b: exchange_b_name.to_string(),
            rsi_correlation,
            sma_correlation,
            momentum_correlation,
            overall_technical_correlation: overall_correlation,
            confidence,
        })
    }

    /// Generate comprehensive correlation metrics for a trading pair across multiple exchanges
    pub async fn generate_correlation_metrics(
        &self,
        trading_pair: &str,
        exchange_data: &HashMap<String, PriceSeries>,
        user_preferences: &UserTradingPreferences,
    ) -> Result<CorrelationMetrics, String> {
        if exchange_data.len() < 2 {
            return Err("Need at least 2 exchanges for correlation analysis".to_string());
        }

        let exchanges: Vec<String> = exchange_data.keys().cloned().collect();
        let mut price_correlations = Vec::new();
        let mut leadership_analysis = Vec::new();
        let mut technical_correlations = Vec::new();

        // Calculate all pairwise correlations
        for i in 0..exchanges.len() {
            for j in (i + 1)..exchanges.len() {
                let exchange_a = &exchanges[i];
                let exchange_b = &exchanges[j];
                let data_a = &exchange_data[exchange_a];
                let data_b = &exchange_data[exchange_b];

                // Price correlation
                if let Ok(price_corr) =
                    self.calculate_price_correlation(data_a, data_b, exchange_a, exchange_b)
                {
                    price_correlations.push(price_corr);
                }

                // Leadership analysis (both directions)
                if let Ok(leadership_ab) =
                    self.analyze_exchange_leadership(data_a, data_b, exchange_a, exchange_b)
                {
                    leadership_analysis.push(leadership_ab);
                }

                if let Ok(leadership_ba) =
                    self.analyze_exchange_leadership(data_b, data_a, exchange_b, exchange_a)
                {
                    leadership_analysis.push(leadership_ba);
                }

                // Technical correlation (if user is interested in technical analysis)
                if user_preferences.trading_focus != TradingFocus::Arbitrage {
                    if let Ok(tech_corr) =
                        self.calculate_technical_correlation(data_a, data_b, exchange_a, exchange_b)
                    {
                        technical_correlations.push(tech_corr);
                    }
                }
            }
        }

        let confidence_score = self.calculate_overall_confidence(
            &price_correlations,
            &leadership_analysis,
            &technical_correlations,
        );

        // Store correlation results to pipeline for historical tracking
        for correlation in &price_correlations {
            if let Err(e) = self
                .store_correlation_results_to_pipeline(correlation, trading_pair)
                .await
            {
                self.logger.warn(&format!(
                    "Failed to store correlation results to pipeline: {}",
                    e
                ));
            }
        }

        // Store leadership analysis to pipeline
        for leadership in &leadership_analysis {
            if let Err(e) = self
                .store_leadership_analysis_to_pipeline(leadership, trading_pair)
                .await
            {
                self.logger.warn(&format!(
                    "Failed to store leadership analysis to pipeline: {}",
                    e
                ));
            }
        }

        self.logger.info(&format!(
            "Generated correlation metrics for {}: {} price correlations, {} leadership analyses, {} technical correlations",
            trading_pair,
            price_correlations.len(),
            leadership_analysis.len(),
            technical_correlations.len()
        ));

        Ok(CorrelationMetrics {
            trading_pair: trading_pair.to_string(),
            price_correlations,
            leadership_analysis,
            technical_correlations,
            analysis_timestamp: Utc::now(),
            confidence_score,
        })
    }

    /// Helper: Align price data by timestamp
    fn align_price_data(
        &self,
        data_a: &PriceSeries,
        data_b: &PriceSeries,
    ) -> Result<Vec<(f64, f64)>, String> {
        let mut aligned_pairs = Vec::new();
        let tolerance_ms = 60000; // 1 minute tolerance in milliseconds

        for price_a in &data_a.data_points {
            for price_b in &data_b.data_points {
                let time_diff = (price_a.timestamp as i64 - price_b.timestamp as i64).abs();
                if time_diff <= tolerance_ms {
                    aligned_pairs.push((price_a.price, price_b.price));
                    break;
                }
            }
        }

        if aligned_pairs.is_empty() {
            return Err("No aligned price data found".to_string());
        }

        Ok(aligned_pairs)
    }

    /// Helper: Calculate lagged correlation
    fn calculate_lagged_correlation(
        &self,
        leader_data: &PriceSeries,
        follower_data: &PriceSeries,
        lag_seconds: i64,
    ) -> Result<LaggedCorrelationResult, String> {
        let lag_ms = lag_seconds * 1000; // Convert to milliseconds
        let mut leader_prices = Vec::new();
        let mut follower_prices = Vec::new();

        for leader_point in &leader_data.data_points {
            let target_time = leader_point.timestamp as i64 + lag_ms;

            // Find follower price closest to target time
            if let Some(follower_point) = follower_data
                .data_points
                .iter()
                .min_by_key(|p| (p.timestamp as i64 - target_time).abs())
            {
                let time_diff = (follower_point.timestamp as i64 - target_time).abs();
                if time_diff <= 30000 {
                    // 30 second tolerance in milliseconds
                    leader_prices.push(leader_point.price);
                    follower_prices.push(follower_point.price);
                }
            }
        }

        if leader_prices.len() < self.config.min_data_points {
            return Err("Insufficient aligned data for lag analysis".to_string());
        }

        let correlation = MathUtils::price_correlation(&leader_prices, &follower_prices)
            .map_err(|e| format!("Lag correlation calculation error: {}", e))?;
        let confidence =
            self.calculate_lag_correlation_confidence(&leader_prices, &follower_prices);

        Ok(LaggedCorrelationResult {
            correlation,
            confidence,
            _data_points: leader_prices.len(),
        })
    }

    /// Helper: Calculate momentum indicator
    fn calculate_momentum(&self, prices: &[f64], period: usize) -> Result<Vec<f64>, String> {
        if prices.len() < period {
            return Err("Insufficient data for momentum calculation".to_string());
        }

        let mut momentum = Vec::new();
        for i in period..prices.len() {
            let current = prices[i];
            let previous = prices[i - period];
            let mom = if previous != 0.0 {
                (current - previous) / previous * 100.0
            } else {
                0.0
            };
            momentum.push(mom);
        }

        Ok(momentum)
    }

    /// Helper: Calculate correlation confidence based on data points and variance
    fn calculate_correlation_confidence(&self, aligned_pairs: &[(f64, f64)]) -> f64 {
        let data_points = aligned_pairs.len() as f64;
        let base_confidence = (data_points / (self.config.min_data_points as f64 * 2.0)).min(1.0);

        // Adjust confidence based on price variance
        let prices_a: Vec<f64> = aligned_pairs.iter().map(|(a, _)| *a).collect();
        let prices_b: Vec<f64> = aligned_pairs.iter().map(|(_, b)| *b).collect();

        if let (Ok(std_a), Ok(std_b)) = (
            MathUtils::standard_deviation(&prices_a),
            MathUtils::standard_deviation(&prices_b),
        ) {
            let variance_factor = if std_a > 0.0 && std_b > 0.0 {
                1.0 // Good variance
            } else {
                0.5 // Low variance reduces confidence
            };
            base_confidence * variance_factor
        } else {
            base_confidence * 0.5
        }
    }

    /// Helper: Calculate lag correlation confidence
    fn calculate_lag_correlation_confidence(&self, prices_a: &[f64], _prices_b: &[f64]) -> f64 {
        let data_points = prices_a.len() as f64;
        (data_points / (self.config.min_data_points as f64)).min(1.0)
    }

    /// Helper: Calculate technical correlation confidence
    fn calculate_technical_correlation_confidence(
        &self,
        rsi_corr: f64,
        sma_corr: f64,
        momentum_corr: f64,
    ) -> f64 {
        let correlations = [rsi_corr.abs(), sma_corr.abs(), momentum_corr.abs()];
        let avg_correlation = correlations.iter().sum::<f64>() / correlations.len() as f64;
        let variance = correlations
            .iter()
            .map(|&x| (x - avg_correlation).powi(2))
            .sum::<f64>()
            / correlations.len() as f64;

        // Higher average correlation and lower variance = higher confidence
        let correlation_factor = avg_correlation;
        let consistency_factor = 1.0 - variance.min(1.0);

        (correlation_factor + consistency_factor) / 2.0
    }

    /// Helper: Calculate overall confidence score
    fn calculate_overall_confidence(
        &self,
        price_correlations: &[ExchangeCorrelationData],
        leadership_analysis: &[LeadershipAnalysis],
        technical_correlations: &[TechnicalCorrelation],
    ) -> f64 {
        let mut total_confidence = 0.0;
        let mut weight_sum = 0.0;

        // Price correlation confidence (weight: 0.5)
        if !price_correlations.is_empty() {
            let avg_price_confidence: f64 = price_correlations
                .iter()
                .map(|pc| pc.confidence_level)
                .sum::<f64>()
                / price_correlations.len() as f64;
            total_confidence += avg_price_confidence * 0.5;
            weight_sum += 0.5;
        }

        // Leadership analysis confidence (weight: 0.3)
        if !leadership_analysis.is_empty() {
            let avg_leadership_confidence: f64 = leadership_analysis
                .iter()
                .map(|la| la.confidence)
                .sum::<f64>()
                / leadership_analysis.len() as f64;
            total_confidence += avg_leadership_confidence * 0.3;
            weight_sum += 0.3;
        }

        // Technical correlation confidence (weight: 0.2)
        if !technical_correlations.is_empty() {
            let avg_technical_confidence: f64 = technical_correlations
                .iter()
                .map(|tc| tc.confidence)
                .sum::<f64>()
                / technical_correlations.len() as f64;
            total_confidence += avg_technical_confidence * 0.2;
            weight_sum += 0.2;
        }

        if weight_sum > 0.0 {
            total_confidence / weight_sum
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
struct LaggedCorrelationResult {
    correlation: f64,
    confidence: f64,
    _data_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::core::analysis::market_analysis::{PricePoint, PriceSeries, TimeFrame};
    use crate::services::core::user::user_trading_preferences::{
        AutomationLevel, AutomationScope, ExperienceLevel, RiskTolerance, TradingFocus,
        UserTradingPreferences,
    };
    use std::collections::HashMap;

    // Helper functions for testing

    fn create_test_price_series(
        base_time: u64,
        prices: Vec<f64>,
        time_intervals_ms: u64,
        exchange_id: &str,
        trading_pair: &str,
    ) -> PriceSeries {
        let mut price_series = PriceSeries::new(
            trading_pair.to_string(),
            exchange_id.to_string(),
            TimeFrame::OneMinute,
        );

        for (i, price) in prices.iter().enumerate() {
            let timestamp = base_time + (i as u64 * time_intervals_ms);
            let price_point = PricePoint {
                timestamp,
                price: *price,
                volume: Some(1000.0),
                exchange_id: exchange_id.to_string(),
                trading_pair: trading_pair.to_string(),
            };
            price_series.add_price_point(price_point);
        }

        price_series
    }

    fn create_correlated_price_series(
        base_series: &PriceSeries,
        correlation: f64,
        noise: f64,
        exchange_id: &str,
    ) -> PriceSeries {
        let mut correlated_series = PriceSeries::new(
            base_series.trading_pair.clone(),
            exchange_id.to_string(),
            base_series.timeframe.clone(),
        );

        for (i, point) in base_series.data_points.iter().enumerate() {
            let base_price = point.price;
            let correlated_price = base_price * correlation + (i as f64 * noise);
            let correlated_point = PricePoint {
                timestamp: point.timestamp,
                price: correlated_price,
                volume: point.volume,
                exchange_id: exchange_id.to_string(),
                trading_pair: point.trading_pair.clone(),
            };
            correlated_series.add_price_point(correlated_point);
        }

        correlated_series
    }

    fn create_lagged_price_series(
        base_series: &PriceSeries,
        lag_ms: u64,
        exchange_id: &str,
    ) -> PriceSeries {
        let mut lagged_series = PriceSeries::new(
            base_series.trading_pair.clone(),
            exchange_id.to_string(),
            base_series.timeframe.clone(),
        );

        for point in &base_series.data_points {
            let lagged_timestamp = point.timestamp + lag_ms;
            let lagged_point = PricePoint {
                timestamp: lagged_timestamp,
                price: point.price,
                volume: point.volume,
                exchange_id: exchange_id.to_string(),
                trading_pair: point.trading_pair.clone(),
            };
            lagged_series.add_price_point(lagged_point);
        }

        lagged_series
    }

    fn create_test_user_preferences(trading_focus: TradingFocus) -> UserTradingPreferences {
        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        UserTradingPreferences {
            preference_id: "test_pref".to_string(),
            user_id: "test_user".to_string(),
            trading_focus,
            experience_level: ExperienceLevel::Intermediate,
            risk_tolerance: RiskTolerance::Balanced,
            automation_level: AutomationLevel::Manual,
            automation_scope: AutomationScope::None,
            arbitrage_enabled: true,
            technical_enabled: true,
            advanced_analytics_enabled: false,
            preferred_notification_channels: vec!["telegram".to_string()],
            trading_hours_timezone: "UTC".to_string(),
            trading_hours_start: "00:00".to_string(),
            trading_hours_end: "23:59".to_string(),
            onboarding_completed: true,
            tutorial_steps_completed: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_create_correlation_analysis_service() {
        let config = CorrelationAnalysisConfig::default();
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(config, logger);

        // Service should be created successfully
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[test]
    fn test_calculate_price_correlation_high_correlation() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000; // 2024-01-01 00:00:00 UTC in milliseconds

        // Create highly correlated price series
        let prices_a = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0,
        ];
        let series_a = create_test_price_series(base_time, prices_a, 60000, "binance", "BTC/USDT");

        let prices_b = vec![
            99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
            111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0,
        ];
        let series_b = create_test_price_series(base_time, prices_b, 60000, "bybit", "BTC/USDT");

        let result = service.calculate_price_correlation(&series_a, &series_b, "binance", "bybit");

        assert!(result.is_ok());
        let correlation_data = result.unwrap();
        assert_eq!(correlation_data.exchange_a, "binance");
        assert_eq!(correlation_data.exchange_b, "bybit");
        assert!(correlation_data.correlation_coefficient > 0.95); // High correlation
        assert!(correlation_data.confidence_level > 0.0);
        assert!(correlation_data.data_points >= 20);
    }

    #[test]
    fn test_calculate_price_correlation_insufficient_data() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;

        // Create series with insufficient data points
        let prices = vec![100.0, 101.0, 102.0]; // Only 3 points, need 20
        let series_a =
            create_test_price_series(base_time, prices.clone(), 60000, "binance", "BTC/USDT");
        let series_b = create_test_price_series(base_time, prices, 60000, "bybit", "BTC/USDT");

        let result = service.calculate_price_correlation(&series_a, &series_b, "binance", "bybit");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data points"));
    }

    #[test]
    fn test_analyze_exchange_leadership() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;

        // Create leader and follower series
        let leader_prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0,
        ];
        let leader_series =
            create_test_price_series(base_time, leader_prices, 60000, "binance", "BTC/USDT");

        // Create follower series with 2-minute lag
        let follower_series = create_lagged_price_series(&leader_series, 120000, "bybit"); // 120000ms = 2 minutes

        let result = service.analyze_exchange_leadership(
            &leader_series,
            &follower_series,
            "binance",
            "bybit",
        );

        assert!(result.is_ok());
        let leadership = result.unwrap();
        assert_eq!(leadership.leading_exchange, "binance");
        assert_eq!(leadership.following_exchange, "bybit");
        assert!(leadership.lag_seconds >= 0);
        assert!(leadership.confidence > 0.0);
    }

    #[test]
    fn test_calculate_technical_correlation() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;

        // Create two highly correlated price series for technical analysis
        let prices_a = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0,
            124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0,
        ];
        let series_a = create_test_price_series(base_time, prices_a, 60000, "binance", "BTC/USDT");
        let series_b = create_correlated_price_series(&series_a, 0.98, 0.01, "bybit");

        let result =
            service.calculate_technical_correlation(&series_a, &series_b, "binance", "bybit");

        assert!(result.is_ok());
        let tech_correlation = result.unwrap();
        assert_eq!(tech_correlation.exchange_a, "binance");
        assert_eq!(tech_correlation.exchange_b, "bybit");
        assert!(tech_correlation.rsi_correlation.abs() <= 1.0);
        assert!(tech_correlation.sma_correlation.abs() <= 1.0);
        assert!(tech_correlation.momentum_correlation.abs() <= 1.0);
        assert!(tech_correlation.overall_technical_correlation.abs() <= 1.0);
        assert!(tech_correlation.confidence >= 0.0 && tech_correlation.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_generate_correlation_metrics_arbitrage_focus() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;
        let user_preferences = create_test_user_preferences(TradingFocus::Arbitrage);

        // Create exchange data
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0,
        ];
        let binance_series =
            create_test_price_series(base_time, prices, 60000, "binance", "BTC/USDT");
        let bybit_series = create_correlated_price_series(&binance_series, 0.95, 0.02, "bybit");
        let okx_series = create_correlated_price_series(&binance_series, 0.90, 0.05, "okx");

        let mut exchange_data = HashMap::new();
        exchange_data.insert("binance".to_string(), binance_series);
        exchange_data.insert("bybit".to_string(), bybit_series);
        exchange_data.insert("okx".to_string(), okx_series);

        let result = service
            .generate_correlation_metrics("BTC/USDT", &exchange_data, &user_preferences)
            .await;

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.trading_pair, "BTC/USDT");
        assert!(!metrics.price_correlations.is_empty());
        assert!(!metrics.leadership_analysis.is_empty());
        // For arbitrage-only focus, technical correlations should be empty
        assert!(metrics.technical_correlations.is_empty());
        assert!(metrics.confidence_score >= 0.0 && metrics.confidence_score <= 1.0);
    }

    #[tokio::test]
    async fn test_generate_correlation_metrics_hybrid_focus() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;
        let user_preferences = create_test_user_preferences(TradingFocus::Technical);

        // Create exchange data with enough data for technical analysis
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0,
            124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0,
        ];
        let binance_series =
            create_test_price_series(base_time, prices, 60000, "binance", "BTC/USDT");
        let bybit_series = create_correlated_price_series(&binance_series, 0.95, 0.02, "bybit");

        let mut exchange_data = HashMap::new();
        exchange_data.insert("binance".to_string(), binance_series);
        exchange_data.insert("bybit".to_string(), bybit_series);

        let result = service
            .generate_correlation_metrics("BTC/USDT", &exchange_data, &user_preferences)
            .await;

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.trading_pair, "BTC/USDT");
        assert!(!metrics.price_correlations.is_empty());
        assert!(!metrics.leadership_analysis.is_empty());
        // For technical focus, technical correlations should be included
        assert!(!metrics.technical_correlations.is_empty());
        assert!(metrics.confidence_score >= 0.0 && metrics.confidence_score <= 1.0);
    }

    #[tokio::test]
    async fn test_generate_correlation_metrics_insufficient_exchanges() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let user_preferences = create_test_user_preferences(TradingFocus::Arbitrage);

        // Create exchange data with only one exchange
        let base_time = 1672531200000;
        let prices = vec![100.0, 101.0, 102.0];
        let binance_series =
            create_test_price_series(base_time, prices, 60000, "binance", "BTC/USDT");

        let mut exchange_data = HashMap::new();
        exchange_data.insert("binance".to_string(), binance_series);

        let result = service
            .generate_correlation_metrics("BTC/USDT", &exchange_data, &user_preferences)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Need at least 2 exchanges"));
    }

    #[test]
    fn test_correlation_analysis_config_default() {
        let config = CorrelationAnalysisConfig::default();

        assert_eq!(config.min_data_points, 20);
        assert_eq!(config.max_lag_seconds, 300);
        assert_eq!(config.correlation_threshold, 0.5);
        assert_eq!(config.leadership_threshold, 0.6);
        assert_eq!(config.technical_correlation_weight, 0.3);
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[test]
    fn test_correlation_analysis_config_custom() {
        let config = CorrelationAnalysisConfig {
            min_data_points: 50,
            max_lag_seconds: 600,
            correlation_threshold: 0.8,
            leadership_threshold: 0.9,
            technical_correlation_weight: 0.5,
            confidence_threshold: 0.85,
        };

        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(config, logger);

        // Service should be created with custom config
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[test]
    fn test_exchange_correlation_data_serialization() {
        #[cfg(target_arch = "wasm32")]
        let now =
            chrono::DateTime::from_timestamp_millis(js_sys::Date::now() as i64).unwrap_or_default();
        #[cfg(not(target_arch = "wasm32"))]
        let now = chrono::Utc::now();

        let correlation_data = ExchangeCorrelationData {
            exchange_a: "binance".to_string(),
            exchange_b: "bybit".to_string(),
            correlation_coefficient: 0.85,
            confidence_level: 0.75,
            data_points: 100,
            analysis_timestamp: now,
        };

        // Test serialization
        let serialized = serde_json::to_string(&correlation_data);
        assert!(serialized.is_ok());

        // Test deserialization
        let deserialized: Result<ExchangeCorrelationData, _> =
            serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_data = deserialized.unwrap();
        assert_eq!(deserialized_data.exchange_a, "binance");
        assert_eq!(deserialized_data.exchange_b, "bybit");
        assert_eq!(deserialized_data.correlation_coefficient, 0.85);
    }

    #[test]
    fn test_correlation_confidence_calculation() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;

        // Create data with good variance
        let prices_a = vec![
            100.0, 105.0, 98.0, 110.0, 95.0, 112.0, 93.0, 115.0, 90.0, 118.0, 88.0, 120.0, 85.0,
            122.0, 83.0, 125.0, 80.0, 128.0, 78.0, 130.0, 75.0,
        ];
        let prices_b = vec![
            101.0, 106.0, 99.0, 111.0, 96.0, 113.0, 94.0, 116.0, 91.0, 119.0, 89.0, 121.0, 86.0,
            123.0, 84.0, 126.0, 81.0, 129.0, 79.0, 131.0, 76.0,
        ];

        let series_a = create_test_price_series(base_time, prices_a, 60000, "binance", "BTC/USDT");
        let series_b = create_test_price_series(base_time, prices_b, 60000, "bybit", "BTC/USDT");

        let result = service.calculate_price_correlation(&series_a, &series_b, "binance", "bybit");

        assert!(result.is_ok());
        let correlation_data = result.unwrap();
        // Should have reasonable confidence due to good data variance
        assert!(correlation_data.confidence_level > 0.5);
    }

    #[tokio::test]
    async fn test_multiple_exchange_correlation_analysis() {
        let logger = Logger::new(crate::utils::logger::LogLevel::Info);
        let service = CorrelationAnalysisService::new(CorrelationAnalysisConfig::default(), logger);
        let base_time = 1672531200000;
        let user_preferences = create_test_user_preferences(TradingFocus::Technical);

        // Create data for 4 exchanges with enough data for technical analysis
        let base_prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0,
            124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0,
        ];
        let binance_series =
            create_test_price_series(base_time, base_prices, 60000, "binance", "BTC/USDT");
        let bybit_series = create_correlated_price_series(&binance_series, 0.95, 0.02, "bybit");
        let okx_series = create_correlated_price_series(&binance_series, 0.90, 0.05, "okx");
        let bitget_series = create_correlated_price_series(&binance_series, 0.85, 0.08, "bitget");

        let mut exchange_data = HashMap::new();
        exchange_data.insert("binance".to_string(), binance_series);
        exchange_data.insert("bybit".to_string(), bybit_series);
        exchange_data.insert("okx".to_string(), okx_series);
        exchange_data.insert("bitget".to_string(), bitget_series);

        let result = service
            .generate_correlation_metrics("BTC/USDT", &exchange_data, &user_preferences)
            .await;

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // With 4 exchanges, we should have 6 price correlations (4 choose 2)
        assert_eq!(metrics.price_correlations.len(), 6);
        // We should have 12 leadership analyses (6 pairs × 2 directions)
        assert_eq!(metrics.leadership_analysis.len(), 12);
        // We should have 6 technical correlations for technical focus
        assert_eq!(metrics.technical_correlations.len(), 6);
        assert!(metrics.confidence_score > 0.0);
    }
}
