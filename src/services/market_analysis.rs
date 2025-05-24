// Market Analysis Service
// Task 9.1: Technical Indicators Foundation for Hybrid Trading Platform
// Supports both arbitrage enhancement and standalone technical trading

use crate::services::user_trading_preferences::{TradingFocus, UserTradingPreferences};
use crate::services::{D1Service, UserTradingPreferencesService};
use crate::utils::{logger::Logger, ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use worker::*; // TODO: Re-enable when implementing worker functionality

// ============= CORE DATA STRUCTURES =============

/// Price data point with timestamp
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PricePoint {
    pub timestamp: u64,       // Unix timestamp in milliseconds
    pub price: f64,           // Price value
    pub volume: Option<f64>,  // Trading volume (optional)
    pub exchange_id: String,  // Exchange identifier
    pub trading_pair: String, // Trading pair (e.g., "BTC/USDT")
}

/// Time series of price data for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSeries {
    pub trading_pair: String,
    pub exchange_id: String,
    pub timeframe: TimeFrame,
    pub data_points: Vec<PricePoint>,
    pub last_updated: u64,
}

impl PriceSeries {
    pub fn new(trading_pair: String, exchange_id: String, timeframe: TimeFrame) -> Self {
        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            trading_pair,
            exchange_id,
            timeframe,
            data_points: Vec::new(),
            last_updated: now,
        }
    }

    pub fn add_price_point(&mut self, point: PricePoint) {
        self.data_points.push(point);
        // Keep data sorted by timestamp
        self.data_points
            .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        #[cfg(target_arch = "wasm32")]
        {
            self.last_updated = js_sys::Date::now() as u64;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_updated = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
    }

    /// Get the latest price point
    pub fn latest_price(&self) -> Option<&PricePoint> {
        self.data_points.last()
    }

    /// Get prices within a specific time range
    pub fn price_range(&self, start_time: u64, end_time: u64) -> Vec<&PricePoint> {
        self.data_points
            .iter()
            .filter(|point| point.timestamp >= start_time && point.timestamp <= end_time)
            .collect()
    }

    /// Get just the price values for calculations
    pub fn price_values(&self) -> Vec<f64> {
        self.data_points.iter().map(|p| p.price).collect()
    }
}

/// Time frame for price data aggregation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeFrame {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "1d")]
    OneDay,
}

impl TimeFrame {
    /// Get the duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        match self {
            TimeFrame::OneMinute => 60 * 1000,
            TimeFrame::FiveMinutes => 5 * 60 * 1000,
            TimeFrame::FifteenMinutes => 15 * 60 * 1000,
            TimeFrame::OneHour => 60 * 60 * 1000,
            TimeFrame::FourHours => 4 * 60 * 60 * 1000,
            TimeFrame::OneDay => 24 * 60 * 60 * 1000,
        }
    }
}

/// Result of a technical indicator calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResult {
    pub indicator_name: String,
    pub values: Vec<IndicatorValue>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub calculated_at: u64,
}

/// Individual indicator value with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorValue {
    pub timestamp: u64,
    pub value: f64,
    pub signal: Option<SignalType>, // Buy/Sell/Hold signal if applicable
}

/// Signal types for trading decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    StrongBuy,
    StrongSell,
}

/// Types of trading opportunities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpportunityType {
    #[serde(rename = "arbitrage")]
    Arbitrage, // Cross-exchange price differences
    #[serde(rename = "technical")]
    Technical, // Technical analysis signals
    #[serde(rename = "arbitrage_technical")]
    ArbitrageTechnical, // Arbitrage enhanced with technical analysis
}

/// Trading opportunity with analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingOpportunity {
    pub opportunity_id: String,
    pub opportunity_type: OpportunityType,
    pub trading_pair: String,
    pub exchanges: Vec<String>, // Exchanges involved
    pub entry_price: f64,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub confidence_score: f64, // 0.0 to 1.0
    pub risk_level: RiskLevel,
    pub expected_return: f64, // Percentage
    pub time_horizon: TimeHorizon,
    pub indicators_used: Vec<String>,
    pub analysis_data: serde_json::Value,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low, // Conservative, arbitrage-focused
    #[serde(rename = "medium")]
    Medium, // Balanced risk-reward
    #[serde(rename = "high")]
    High, // Higher risk, higher potential return
}

/// Time horizon for opportunity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeHorizon {
    #[serde(rename = "immediate")]
    Immediate, // < 5 minutes
    #[serde(rename = "short")]
    Short, // 5-60 minutes
    #[serde(rename = "medium")]
    Medium, // 1-24 hours
    #[serde(rename = "long")]
    Long, // > 24 hours
}

// ============= MATHEMATICAL FOUNDATION =============

/// Mathematical utility functions for technical indicators
pub struct MathUtils;

impl MathUtils {
    /// Calculate Simple Moving Average
    #[allow(clippy::result_large_err)]
    pub fn simple_moving_average(prices: &[f64], period: usize) -> ArbitrageResult<Vec<f64>> {
        if prices.len() < period {
            return Err(ArbitrageError::validation_error(
                "Insufficient data for SMA calculation",
            ));
        }

        let mut sma_values = Vec::new();

        for i in (period - 1)..prices.len() {
            let sum: f64 = prices[(i + 1 - period)..=i].iter().sum();
            sma_values.push(sum / period as f64);
        }

        Ok(sma_values)
    }

    /// Calculate Exponential Moving Average
    #[allow(clippy::result_large_err)]
    pub fn exponential_moving_average(prices: &[f64], period: usize) -> ArbitrageResult<Vec<f64>> {
        if prices.is_empty() {
            return Err(ArbitrageError::validation_error("No price data provided"));
        }

        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema_values = Vec::with_capacity(prices.len());

        // First EMA value is the first price
        ema_values.push(prices[0]);

        // Calculate subsequent EMA values
        for i in 1..prices.len() {
            let ema = alpha * prices[i] + (1.0 - alpha) * ema_values[i - 1];
            ema_values.push(ema);
        }

        Ok(ema_values)
    }

    /// Calculate standard deviation
    #[allow(clippy::result_large_err)]
    pub fn standard_deviation(values: &[f64]) -> ArbitrageResult<f64> {
        if values.is_empty() {
            return Err(ArbitrageError::validation_error("No values provided"));
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

        Ok(variance.sqrt())
    }

    /// Calculate Relative Strength Index (RSI)
    /// Calculate Relative Strength Index (RSI) using Cutler's method
    ///
    /// This implementation uses simple moving averages for gain/loss calculation (Cutler's method)
    /// rather than exponential moving averages (Wilder's method). Cutler's method is more responsive
    /// to recent price changes and is commonly used in modern trading platforms.
    ///
    /// Formula: RSI = 100 - (100 / (1 + RS))
    /// Where RS = Average Gain / Average Loss over the specified period
    #[allow(clippy::result_large_err)]
    pub fn relative_strength_index(prices: &[f64], period: usize) -> ArbitrageResult<Vec<f64>> {
        if prices.len() < period + 1 {
            return Err(ArbitrageError::validation_error(
                "Insufficient data for RSI calculation",
            ));
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        // Calculate price changes
        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let mut rsi_values = Vec::new();

        // Calculate RSI for each period
        for i in (period - 1)..gains.len() {
            let avg_gain: f64 = gains[(i + 1 - period)..=i].iter().sum::<f64>() / period as f64;
            let avg_loss: f64 = losses[(i + 1 - period)..=i].iter().sum::<f64>() / period as f64;

            let rs = if avg_loss == 0.0 {
                100.0 // If no losses, RSI is 100
            } else {
                avg_gain / avg_loss
            };

            let rsi = 100.0 - (100.0 / (1.0 + rs));
            rsi_values.push(rsi);
        }

        Ok(rsi_values)
    }

    /// Calculate Bollinger Bands
    #[allow(clippy::result_large_err)]
    pub fn bollinger_bands(
        prices: &[f64],
        period: usize,
        std_dev_multiplier: f64,
    ) -> ArbitrageResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let sma = Self::simple_moving_average(prices, period)?;

        if prices.len() < period {
            return Err(ArbitrageError::validation_error(
                "Insufficient data for Bollinger Bands",
            ));
        }

        let mut upper_band = Vec::new();
        let mut lower_band = Vec::new();

        for i in (period - 1)..prices.len() {
            let price_slice = &prices[(i + 1 - period)..=i];
            let std_dev = Self::standard_deviation(price_slice)?;
            let sma_index = i - (period - 1);

            upper_band.push(sma[sma_index] + (std_dev_multiplier * std_dev));
            lower_band.push(sma[sma_index] - (std_dev_multiplier * std_dev));
        }

        Ok((upper_band, sma, lower_band))
    }

    /// Calculate price correlation between two price series
    #[allow(clippy::result_large_err)]
    pub fn price_correlation(prices1: &[f64], prices2: &[f64]) -> ArbitrageResult<f64> {
        if prices1.len() != prices2.len() || prices1.is_empty() {
            return Err(ArbitrageError::validation_error(
                "Price series must be same length and non-empty",
            ));
        }

        let n = prices1.len() as f64;
        let mean1 = prices1.iter().sum::<f64>() / n;
        let mean2 = prices2.iter().sum::<f64>() / n;

        let numerator: f64 = prices1
            .iter()
            .zip(prices2.iter())
            .map(|(x, y)| (x - mean1) * (y - mean2))
            .sum();

        let sum_sq1: f64 = prices1.iter().map(|x| (x - mean1).powi(2)).sum();
        let sum_sq2: f64 = prices2.iter().map(|y| (y - mean2).powi(2)).sum();

        let denominator = (sum_sq1 * sum_sq2).sqrt();

        if denominator == 0.0 {
            Ok(0.0)
        } else {
            Ok(numerator / denominator)
        }
    }
}

// ============= MARKET ANALYSIS SERVICE =============

/// Main service for market analysis and technical indicators
pub struct MarketAnalysisService {
    _d1_service: D1Service,
    preferences_service: UserTradingPreferencesService,
    logger: Logger,
    price_cache: HashMap<String, PriceSeries>, // In-memory cache for recent price data
    cache_max_size: usize,                     // Maximum number of cached series
    cache_ttl_ms: u64,                         // Time-to-live for cache entries in milliseconds
}

impl MarketAnalysisService {
    pub fn new(
        d1_service: D1Service,
        preferences_service: UserTradingPreferencesService,
        logger: Logger,
    ) -> Self {
        Self {
            _d1_service: d1_service,
            preferences_service,
            logger,
            price_cache: HashMap::new(),
            cache_max_size: 1000, // Default: cache up to 1000 price series
            cache_ttl_ms: 24 * 60 * 60 * 1000, // Default: 24 hours TTL
        }
    }

    /// Store price data for analysis
    pub async fn store_price_data(&mut self, price_point: PricePoint) -> ArbitrageResult<()> {
        let cache_key = format!("{}_{}", price_point.exchange_id, price_point.trading_pair);

        // Evict expired entries before adding new data
        self.evict_expired_cache_entries();

        // Check if cache is at capacity and evict LRU entries if needed
        if self.price_cache.len() >= self.cache_max_size
            && !self.price_cache.contains_key(&cache_key)
        {
            self.evict_lru_cache_entries(1);
        }

        // Update in-memory cache
        match self.price_cache.get_mut(&cache_key) {
            Some(series) => {
                series.add_price_point(price_point.clone());
            }
            None => {
                let mut series = PriceSeries::new(
                    price_point.trading_pair.clone(),
                    price_point.exchange_id.clone(),
                    TimeFrame::OneMinute, // Default timeframe
                );
                series.add_price_point(price_point.clone());
                self.price_cache.insert(cache_key, series);
            }
        }

        self.logger.info(&format!(
            "Stored price data for {}/{}: ${}",
            price_point.exchange_id, price_point.trading_pair, price_point.price
        ));

        Ok(())
    }

    /// Get price series for analysis
    pub fn get_price_series(&self, exchange_id: &str, trading_pair: &str) -> Option<&PriceSeries> {
        let cache_key = format!("{}_{}", exchange_id, trading_pair);
        self.price_cache.get(&cache_key)
    }

    /// Evict expired cache entries based on TTL
    fn evict_expired_cache_entries(&mut self) {
        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let expired_keys: Vec<String> = self
            .price_cache
            .iter()
            .filter(|(_, series)| now.saturating_sub(series.last_updated) > self.cache_ttl_ms)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.price_cache.remove(&key);
            self.logger
                .debug(&format!("Evicted expired cache entry: {}", key));
        }
    }

    /// Evict least recently used cache entries
    fn evict_lru_cache_entries(&mut self, count: usize) {
        // Find the oldest entries by last_updated timestamp
        let mut entries: Vec<(String, u64)> = self
            .price_cache
            .iter()
            .map(|(key, series)| (key.clone(), series.last_updated))
            .collect();

        // Sort by last_updated (oldest first)
        entries.sort_by_key(|(_, timestamp)| *timestamp);

        // Remove the oldest entries
        for (key, _) in entries.into_iter().take(count) {
            self.price_cache.remove(&key);
            self.logger
                .debug(&format!("Evicted LRU cache entry: {}", key));
        }
    }

    /// Configure cache settings
    pub fn configure_cache(&mut self, max_size: usize, ttl_ms: u64) {
        self.cache_max_size = max_size;
        self.cache_ttl_ms = ttl_ms;
        self.logger.info(&format!(
            "Cache configured: max_size={}, ttl_ms={}",
            max_size, ttl_ms
        ));
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert(
            "cache_size".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.price_cache.len())),
        );
        stats.insert(
            "cache_max_size".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.cache_max_size)),
        );
        stats.insert(
            "cache_ttl_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.cache_ttl_ms)),
        );

        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let expired_count = self
            .price_cache
            .values()
            .filter(|series| now.saturating_sub(series.last_updated) > self.cache_ttl_ms)
            .count();

        stats.insert(
            "expired_entries".to_string(),
            serde_json::Value::Number(serde_json::Number::from(expired_count)),
        );
        stats
    }

    /// Generate trading opportunities based on user preferences
    pub async fn generate_opportunities(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Vec<TradingOpportunity>> {
        let preferences = self
            .preferences_service
            .get_or_create_preferences(user_id)
            .await?;

        let mut opportunities = Vec::new();

        match preferences.trading_focus {
            TradingFocus::Arbitrage => {
                // Focus on arbitrage opportunities
                opportunities.extend(self.detect_arbitrage_opportunities(&preferences).await?);
            }
            TradingFocus::Technical => {
                // Focus on technical trading opportunities
                opportunities.extend(self.detect_technical_opportunities(&preferences).await?);
            }
            TradingFocus::Hybrid => {
                // Both arbitrage and technical opportunities
                opportunities.extend(self.detect_arbitrage_opportunities(&preferences).await?);
                opportunities.extend(self.detect_technical_opportunities(&preferences).await?);
                opportunities.extend(self.detect_hybrid_opportunities(&preferences).await?);
            }
        }

        // Filter by user's risk tolerance and experience level
        opportunities = self.filter_opportunities_by_preferences(opportunities, &preferences);

        self.logger.info(&format!(
            "Generated {} opportunities for user {} (focus: {:?})",
            opportunities.len(),
            user_id,
            preferences.trading_focus
        ));

        Ok(opportunities)
    }

    /// Detect arbitrage opportunities (enhanced with technical analysis)
    async fn detect_arbitrage_opportunities(
        &self,
        _preferences: &UserTradingPreferences,
    ) -> ArbitrageResult<Vec<TradingOpportunity>> {
        // TODO: Implement arbitrage detection logic
        // This will be enhanced with technical indicators for better timing
        Ok(Vec::new())
    }

    /// Detect technical trading opportunities
    async fn detect_technical_opportunities(
        &self,
        _preferences: &UserTradingPreferences,
    ) -> ArbitrageResult<Vec<TradingOpportunity>> {
        // TODO: Implement technical analysis opportunity detection
        Ok(Vec::new())
    }

    /// Detect hybrid opportunities (arbitrage + technical signals)
    async fn detect_hybrid_opportunities(
        &self,
        _preferences: &UserTradingPreferences,
    ) -> ArbitrageResult<Vec<TradingOpportunity>> {
        // TODO: Implement hybrid opportunity detection
        Ok(Vec::new())
    }

    /// Filter opportunities based on user preferences
    fn filter_opportunities_by_preferences(
        &self,
        opportunities: Vec<TradingOpportunity>,
        preferences: &UserTradingPreferences,
    ) -> Vec<TradingOpportunity> {
        opportunities
            .into_iter()
            .filter(|opp| {
                // Filter by risk tolerance
                match preferences.risk_tolerance {
                    crate::services::user_trading_preferences::RiskTolerance::Conservative => {
                        opp.risk_level == RiskLevel::Low
                    }
                    crate::services::user_trading_preferences::RiskTolerance::Balanced => {
                        opp.risk_level == RiskLevel::Low || opp.risk_level == RiskLevel::Medium
                    }
                    crate::services::user_trading_preferences::RiskTolerance::Aggressive => {
                        true // All risk levels allowed
                    }
                }
            })
            .filter(|opp| {
                // Filter by experience level (beginners get simpler opportunities)
                match preferences.experience_level {
                    crate::services::user_trading_preferences::ExperienceLevel::Beginner => {
                        opp.opportunity_type == OpportunityType::Arbitrage
                            && opp.confidence_score >= 0.8
                    }
                    crate::services::user_trading_preferences::ExperienceLevel::Intermediate => {
                        opp.confidence_score >= 0.6
                    }
                    crate::services::user_trading_preferences::ExperienceLevel::Advanced => {
                        true // All opportunities allowed
                    }
                }
            })
            .collect()
    }

    /// Calculate technical indicators for a price series
    #[allow(clippy::result_large_err)]
    pub fn calculate_indicators(
        &self,
        series: &PriceSeries,
        indicators: &[&str],
    ) -> ArbitrageResult<Vec<IndicatorResult>> {
        let prices = series.price_values();
        let mut results = Vec::new();

        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        for &indicator in indicators {
            match indicator {
                "sma_20" => {
                    if let Ok(sma_values) = MathUtils::simple_moving_average(&prices, 20) {
                        let indicator_values: Vec<IndicatorValue> = sma_values
                            .into_iter()
                            .enumerate()
                            .map(|(i, value)| IndicatorValue {
                                timestamp: series.data_points[i + 19].timestamp, // Offset by period
                                value,
                                signal: None,
                            })
                            .collect();

                        results.push(IndicatorResult {
                            indicator_name: "SMA_20".to_string(),
                            values: indicator_values,
                            metadata: HashMap::new(),
                            calculated_at: now,
                        });
                    }
                }
                "rsi_14" => {
                    if let Ok(rsi_values) = MathUtils::relative_strength_index(&prices, 14) {
                        let indicator_values: Vec<IndicatorValue> = rsi_values
                            .into_iter()
                            .enumerate()
                            .map(|(i, value)| {
                                let signal = if value > 70.0 {
                                    Some(SignalType::Sell)
                                } else if value < 30.0 {
                                    Some(SignalType::Buy)
                                } else {
                                    Some(SignalType::Hold)
                                };

                                IndicatorValue {
                                    timestamp: series.data_points[i + 14].timestamp, // Offset by period
                                    value,
                                    signal,
                                }
                            })
                            .collect();

                        results.push(IndicatorResult {
                            indicator_name: "RSI_14".to_string(),
                            values: indicator_values,
                            metadata: HashMap::new(),
                            calculated_at: now,
                        });
                    }
                }
                _ => {
                    self.logger
                        .warn(&format!("Unknown indicator requested: {}", indicator));
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_series_creation() {
        let series = PriceSeries::new(
            "BTC/USDT".to_string(),
            "binance".to_string(),
            TimeFrame::OneMinute,
        );

        assert_eq!(series.trading_pair, "BTC/USDT");
        assert_eq!(series.exchange_id, "binance");
        assert_eq!(series.timeframe, TimeFrame::OneMinute);
        assert!(series.data_points.is_empty());
    }

    #[test]
    fn test_price_point_addition() {
        let mut series = PriceSeries::new(
            "BTC/USDT".to_string(),
            "binance".to_string(),
            TimeFrame::OneMinute,
        );

        let point = PricePoint {
            timestamp: 1640995200000, // 2022-01-01 00:00:00 UTC
            price: 46000.0,
            volume: Some(1.5),
            exchange_id: "binance".to_string(),
            trading_pair: "BTC/USDT".to_string(),
        };

        series.add_price_point(point.clone());
        assert_eq!(series.data_points.len(), 1);
        assert_eq!(series.latest_price().unwrap().price, 46000.0);
    }

    #[test]
    fn test_simple_moving_average() {
        let prices = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0];
        let sma = MathUtils::simple_moving_average(&prices, 3).unwrap();

        assert_eq!(sma.len(), 4); // 6 prices - 3 period + 1
        assert_eq!(sma[0], 12.0); // (10+12+14)/3
        assert_eq!(sma[1], 14.0); // (12+14+16)/3
        assert_eq!(sma[2], 16.0); // (14+16+18)/3
        assert_eq!(sma[3], 18.0); // (16+18+20)/3
    }

    #[test]
    fn test_exponential_moving_average() {
        let prices = vec![10.0, 12.0, 14.0, 16.0, 18.0];
        let ema = MathUtils::exponential_moving_average(&prices, 3).unwrap();

        assert_eq!(ema.len(), 5);
        assert_eq!(ema[0], 10.0); // First value is the first price
                                  // Subsequent values should follow EMA formula
        assert!(ema[1] > 10.0 && ema[1] < 12.0);
    }

    #[test]
    fn test_rsi_calculation() {
        let prices = vec![
            44.0, 44.25, 44.5, 43.75, 44.5, 44.0, 44.25, 45.0, 47.0, 46.75, 46.5, 46.25, 47.75,
            47.5, 47.0, 46.5, 46.0, 47.0, 47.25, 48.0,
        ];

        let rsi = MathUtils::relative_strength_index(&prices, 14).unwrap();
        assert!(!rsi.is_empty());

        // RSI should be between 0 and 100
        for value in rsi {
            assert!((0.0..=100.0).contains(&value));
        }
    }

    #[test]
    fn test_bollinger_bands() {
        let prices = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0];
        let (upper, middle, lower) = MathUtils::bollinger_bands(&prices, 5, 2.0).unwrap();

        assert_eq!(upper.len(), middle.len());
        assert_eq!(middle.len(), lower.len());

        // Upper band should be above middle, lower should be below
        for i in 0..upper.len() {
            assert!(upper[i] > middle[i]);
            assert!(lower[i] < middle[i]);
        }
    }

    #[test]
    fn test_price_correlation() {
        let prices1 = vec![10.0, 12.0, 14.0, 16.0, 18.0];
        let prices2 = vec![20.0, 24.0, 28.0, 32.0, 36.0]; // Perfect positive correlation

        let correlation = MathUtils::price_correlation(&prices1, &prices2).unwrap();
        assert!((correlation - 1.0).abs() < 0.001); // Should be very close to 1.0
    }

    #[test]
    fn test_timeframe_duration() {
        assert_eq!(TimeFrame::OneMinute.duration_ms(), 60 * 1000);
        assert_eq!(TimeFrame::FiveMinutes.duration_ms(), 5 * 60 * 1000);
        assert_eq!(TimeFrame::OneHour.duration_ms(), 60 * 60 * 1000);
        assert_eq!(TimeFrame::OneDay.duration_ms(), 24 * 60 * 60 * 1000);
    }
}
