// src/services/technical_trading.rs
// Task 9.3: Technical Trading Opportunities Generation

use crate::services::market_analysis::{
    MarketAnalysisService, OpportunityType, PriceSeries, RiskLevel, TimeHorizon, TradingOpportunity,
};
use crate::services::user_trading_preferences::{
    ExperienceLevel, RiskTolerance, TradingFocus, UserTradingPreferencesService,
};
use crate::types::ExchangeIdEnum;
use crate::utils::{logger::Logger, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSignal {
    pub signal_id: String,
    pub exchange_id: String,
    pub trading_pair: String,
    pub signal_type: TradingSignalType,
    pub signal_strength: SignalStrength,
    pub indicator_source: String,
    pub entry_price: f64,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub confidence_score: f64,
    pub created_at: u64,
    pub expires_at: u64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingSignalType {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalStrength {
    Weak,     // Low confidence signal
    Moderate, // Medium confidence signal
    Strong,   // High confidence signal
    Extreme,  // Extremely high confidence signal
}

#[derive(Clone)]
pub struct TechnicalTradingServiceConfig {
    pub exchanges: Vec<ExchangeIdEnum>,
    pub monitored_pairs: Vec<String>,
    // RSI Configuration
    pub rsi_overbought_threshold: f64, // Default: 70.0
    pub rsi_oversold_threshold: f64,   // Default: 30.0
    pub rsi_strong_threshold: f64,     // Default: 80.0/20.0 for very strong signals
    // Moving Average Configuration
    pub ma_short_period: usize, // Default: 10 (short-term MA)
    pub ma_long_period: usize,  // Default: 20 (long-term MA)
    // Bollinger Bands Configuration
    pub bb_period: usize, // Default: 20
    pub bb_std_dev: f64,  // Default: 2.0
    // Signal Filtering
    pub min_confidence_score: f64, // Minimum confidence for signal generation
    pub signal_expiry_minutes: u64, // How long signals remain valid
    // Risk Management
    pub default_stop_loss_percentage: f64, // Default stop-loss as percentage
    pub default_take_profit_ratio: f64,    // Risk/reward ratio for take profit
}

impl Default for TechnicalTradingServiceConfig {
    fn default() -> Self {
        Self {
            exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            monitored_pairs: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            rsi_overbought_threshold: 70.0,
            rsi_oversold_threshold: 30.0,
            rsi_strong_threshold: 80.0,
            ma_short_period: 10,
            ma_long_period: 20,
            bb_period: 20,
            bb_std_dev: 2.0,
            min_confidence_score: 0.6,
            signal_expiry_minutes: 60,          // 1 hour signal validity
            default_stop_loss_percentage: 0.02, // 2% stop loss
            default_take_profit_ratio: 2.0,     // 2:1 reward/risk ratio
        }
    }
}

pub struct TechnicalTradingService {
    config: TechnicalTradingServiceConfig,
    market_analysis_service: Arc<MarketAnalysisService>,
    preferences_service: Arc<UserTradingPreferencesService>,
    logger: Logger,
}

impl TechnicalTradingService {
    pub fn new(
        config: TechnicalTradingServiceConfig,
        market_analysis_service: Arc<MarketAnalysisService>,
        preferences_service: Arc<UserTradingPreferencesService>,
        logger: Logger,
    ) -> Self {
        Self {
            config,
            market_analysis_service,
            preferences_service,
            logger,
        }
    }

    /// Generate technical trading opportunities for a user
    /// Task 9.3: Main entry point for technical trading opportunity detection
    pub async fn find_technical_opportunities(
        &self,
        exchange_ids: &[ExchangeIdEnum],
        pairs: &[String],
        user_id: Option<&str>,
    ) -> ArbitrageResult<Vec<TradingOpportunity>> {
        let mut opportunities = Vec::new();

        self.logger.info(&format!(
            "Starting technical trading opportunity detection for {} pairs across {} exchanges",
            pairs.len(),
            exchange_ids.len()
        ));

        // Get user preferences for filtering (read-only - don't create if missing)
        let user_preferences = if let Some(uid) = user_id {
            self.preferences_service.get_preferences(uid).await?
        } else {
            None
        };

        // Skip technical trading if user preference is arbitrage-only
        if let Some(ref prefs) = user_preferences {
            if prefs.trading_focus == TradingFocus::Arbitrage {
                self.logger.debug(
                    "User focus is arbitrage-only, skipping technical trading opportunities",
                );
                return Ok(opportunities);
            }
        }

        // Generate technical signals for each exchange-pair combination
        for exchange_id in exchange_ids {
            for pair in pairs {
                match self
                    .generate_technical_signals_for_pair(exchange_id, pair)
                    .await
                {
                    Ok(signals) => {
                        for signal in signals {
                            // Apply user preference filtering
                            if let Some(ref prefs) = user_preferences {
                                if !self.passes_user_preference_filter(&signal, prefs) {
                                    continue;
                                }
                            }

                            // Convert signal to trading opportunity
                            match self.convert_signal_to_opportunity(signal).await {
                                Ok(opportunity) => opportunities.push(opportunity),
                                Err(e) => {
                                    self.logger.warn(&format!(
                                        "Failed to convert signal to opportunity: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.logger.warn(&format!(
                            "Failed to generate signals for {}/{}: {}",
                            exchange_id, pair, e
                        ));
                    }
                }
            }
        }

        self.logger.info(&format!(
            "Generated {} technical trading opportunities",
            opportunities.len()
        ));

        Ok(opportunities)
    }

    /// Generate technical signals for a specific exchange-pair combination
    async fn generate_technical_signals_for_pair(
        &self,
        exchange_id: &ExchangeIdEnum,
        pair: &str,
    ) -> ArbitrageResult<Vec<TechnicalSignal>> {
        let mut signals = Vec::new();
        let exchange_str = exchange_id.to_string();

        // Get price series for the pair
        if let Some(price_series) = self
            .market_analysis_service
            .get_price_series(&exchange_str, pair)
        {
            // Generate different types of technical signals
            signals.extend(self.generate_rsi_signals(price_series)?);
            signals.extend(self.generate_moving_average_signals(price_series)?);
            signals.extend(self.generate_bollinger_band_signals(price_series)?);
            signals.extend(self.generate_momentum_signals(price_series)?);
        }

        // Filter signals by minimum confidence
        signals.retain(|signal| signal.confidence_score >= self.config.min_confidence_score);

        Ok(signals)
    }

    /// Generate RSI-based trading signals
    #[allow(clippy::result_large_err)]
    fn generate_rsi_signals(
        &self,
        price_series: &PriceSeries,
    ) -> ArbitrageResult<Vec<TechnicalSignal>> {
        let mut signals = Vec::new();

        // Calculate RSI using market analysis service
        let indicators = self
            .market_analysis_service
            .calculate_indicators(price_series, &["rsi_14"])?;

        if let Some(rsi_indicator) = indicators.iter().find(|i| i.indicator_name == "RSI_14") {
            if let Some(latest_rsi) = rsi_indicator.values.last() {
                let rsi_value = latest_rsi.value;
                let current_price = if let Some(latest_price) = price_series.data_points.last() {
                    latest_price.price
                } else {
                    return Ok(signals);
                };

                // Generate signals based on RSI levels
                if rsi_value <= self.config.rsi_oversold_threshold {
                    // Oversold - potential buy signal
                    let signal_strength = if rsi_value <= 20.0 {
                        SignalStrength::Extreme
                    } else if rsi_value <= 25.0 {
                        SignalStrength::Strong
                    } else {
                        SignalStrength::Moderate
                    };

                    let confidence_score =
                        self.calculate_rsi_confidence(rsi_value, TradingSignalType::Buy);

                    signals.push(self.create_technical_signal(
                        price_series,
                        TradingSignalType::Buy,
                        signal_strength,
                        current_price,
                        confidence_score,
                        "RSI_Oversold",
                        Some(serde_json::json!({
                            "rsi_value": rsi_value,
                            "threshold": self.config.rsi_oversold_threshold
                        })),
                    )?);
                } else if rsi_value >= self.config.rsi_overbought_threshold {
                    // Overbought - potential sell signal
                    let signal_strength = if rsi_value >= 80.0 {
                        SignalStrength::Extreme
                    } else if rsi_value >= 75.0 {
                        SignalStrength::Strong
                    } else {
                        SignalStrength::Moderate
                    };

                    let confidence_score =
                        self.calculate_rsi_confidence(rsi_value, TradingSignalType::Sell);

                    signals.push(self.create_technical_signal(
                        price_series,
                        TradingSignalType::Sell,
                        signal_strength,
                        current_price,
                        confidence_score,
                        "RSI_Overbought",
                        Some(serde_json::json!({
                            "rsi_value": rsi_value,
                            "threshold": self.config.rsi_overbought_threshold
                        })),
                    )?);
                }
            }
        }

        Ok(signals)
    }

    /// Generate moving average crossover signals
    #[allow(clippy::result_large_err)]
    fn generate_moving_average_signals(
        &self,
        price_series: &PriceSeries,
    ) -> ArbitrageResult<Vec<TechnicalSignal>> {
        let mut signals = Vec::new();

        // Calculate both short and long-term moving averages
        let sma_short_name = format!("sma_{}", self.config.ma_short_period);
        let sma_long_name = format!("sma_{}", self.config.ma_long_period);

        let indicators = self
            .market_analysis_service
            .calculate_indicators(price_series, &[&sma_short_name, &sma_long_name])?;

        let short_ma = indicators
            .iter()
            .find(|i| i.indicator_name == format!("sma_{}", self.config.ma_short_period));
        let long_ma = indicators
            .iter()
            .find(|i| i.indicator_name == format!("sma_{}", self.config.ma_long_period));

        if let (Some(short_indicator), Some(long_indicator)) = (short_ma, long_ma) {
            if short_indicator.values.len() >= 2 && long_indicator.values.len() >= 2 {
                let short_current = short_indicator.values.last().unwrap().value;
                let short_previous = short_indicator.values[short_indicator.values.len() - 2].value;
                let long_current = long_indicator.values.last().unwrap().value;
                let long_previous = long_indicator.values[long_indicator.values.len() - 2].value;

                let current_price = if let Some(latest_price) = price_series.data_points.last() {
                    latest_price.price
                } else {
                    return Ok(signals);
                };

                // Check for crossover signals
                if short_previous <= long_previous && short_current > long_current {
                    // Golden cross - bullish signal
                    let confidence_score = self.calculate_crossover_confidence(
                        short_current,
                        long_current,
                        TradingSignalType::Buy,
                    );

                    signals.push(self.create_technical_signal(
                        price_series,
                        TradingSignalType::Buy,
                        SignalStrength::Strong,
                        current_price,
                        confidence_score,
                        "MA_Golden_Cross",
                        Some(serde_json::json!({
                            "short_ma": short_current,
                            "long_ma": long_current,
                            "short_period": self.config.ma_short_period,
                            "long_period": self.config.ma_long_period
                        })),
                    )?);
                } else if short_previous >= long_previous && short_current < long_current {
                    // Death cross - bearish signal
                    let confidence_score = self.calculate_crossover_confidence(
                        short_current,
                        long_current,
                        TradingSignalType::Sell,
                    );

                    signals.push(self.create_technical_signal(
                        price_series,
                        TradingSignalType::Sell,
                        SignalStrength::Strong,
                        current_price,
                        confidence_score,
                        "MA_Death_Cross",
                        Some(serde_json::json!({
                            "short_ma": short_current,
                            "long_ma": long_current,
                            "short_period": self.config.ma_short_period,
                            "long_period": self.config.ma_long_period
                        })),
                    )?);
                }
            }
        }

        Ok(signals)
    }

    /// Generate Bollinger Band signals
    #[allow(clippy::result_large_err)]
    fn generate_bollinger_band_signals(
        &self,
        price_series: &PriceSeries,
    ) -> ArbitrageResult<Vec<TechnicalSignal>> {
        let mut signals = Vec::new();

        // Calculate Bollinger Bands
        let bb_name = format!("bb_{}", self.config.bb_period);
        let indicators = self
            .market_analysis_service
            .calculate_indicators(price_series, &[&bb_name])?;

        if let Some(bb_indicator) = indicators
            .iter()
            .find(|i| i.indicator_name == format!("bb_{}", self.config.bb_period))
        {
            if let Some(latest_bb) = bb_indicator.values.last() {
                let current_price = if let Some(latest_price) = price_series.data_points.last() {
                    latest_price.price
                } else {
                    return Ok(signals);
                };

                // Proper Bollinger Band calculation using standard deviation of price data
                let middle_band = latest_bb.value; // This should be the SMA

                // Calculate standard deviation of the recent price data
                let recent_prices: Vec<f64> = price_series
                    .data_points
                    .iter()
                    .rev()
                    .take(self.config.bb_period)
                    .map(|p| p.price)
                    .collect();

                let std_deviation = if recent_prices.len() >= self.config.bb_period {
                    self.calculate_standard_deviation(&recent_prices, middle_band)
                } else {
                    // Fallback for insufficient data - use simplified calculation
                    middle_band * 0.02 // 2% of middle band as fallback
                };

                let upper_band = middle_band + (std_deviation * self.config.bb_std_dev);
                let lower_band = middle_band - (std_deviation * self.config.bb_std_dev);

                // Check for band touch signals
                if current_price <= lower_band {
                    // Price touched lower band - potential buy signal
                    let confidence_score = self.calculate_bollinger_confidence(
                        current_price,
                        lower_band,
                        upper_band,
                        TradingSignalType::Buy,
                    );

                    signals.push(self.create_technical_signal(
                        price_series,
                        TradingSignalType::Buy,
                        SignalStrength::Moderate,
                        current_price,
                        confidence_score,
                        "BB_Lower_Touch",
                        Some(serde_json::json!({
                            "current_price": current_price,
                            "lower_band": lower_band,
                            "upper_band": upper_band,
                            "middle_band": middle_band
                        })),
                    )?);
                } else if current_price >= upper_band {
                    // Price touched upper band - potential sell signal
                    let confidence_score = self.calculate_bollinger_confidence(
                        current_price,
                        lower_band,
                        upper_band,
                        TradingSignalType::Sell,
                    );

                    signals.push(self.create_technical_signal(
                        price_series,
                        TradingSignalType::Sell,
                        SignalStrength::Moderate,
                        current_price,
                        confidence_score,
                        "BB_Upper_Touch",
                        Some(serde_json::json!({
                            "current_price": current_price,
                            "lower_band": lower_band,
                            "upper_band": upper_band,
                            "middle_band": middle_band
                        })),
                    )?);
                }
            }
        }

        Ok(signals)
    }

    /// Generate momentum-based signals
    #[allow(clippy::result_large_err)]
    fn generate_momentum_signals(
        &self,
        price_series: &PriceSeries,
    ) -> ArbitrageResult<Vec<TechnicalSignal>> {
        let mut signals = Vec::new();

        // Check if we have enough data for momentum analysis
        if price_series.data_points.len() < 10 {
            return Ok(signals);
        }

        let prices: Vec<f64> = price_series.data_points.iter().map(|p| p.price).collect();

        // Ensure we have enough data points (at least 6 for 5 periods ago calculation)
        if prices.len() < 6 {
            return Ok(signals);
        }

        let current_price = prices[prices.len() - 1];
        let price_5_periods_ago = prices[prices.len() - 6]; // 5 periods ago

        // Calculate price rate of change
        let price_change = (current_price - price_5_periods_ago) / price_5_periods_ago;
        let momentum_threshold = 0.02; // 2% change threshold

        if price_change.abs() > momentum_threshold {
            let signal_type = if price_change > 0.0 {
                TradingSignalType::Buy
            } else {
                TradingSignalType::Sell
            };

            let signal_strength = if price_change.abs() > 0.05 {
                SignalStrength::Strong
            } else {
                SignalStrength::Moderate
            };

            let confidence_score = self.calculate_momentum_confidence(price_change.abs());

            signals.push(self.create_technical_signal(
                price_series,
                signal_type,
                signal_strength,
                current_price,
                confidence_score,
                "Momentum_Signal",
                Some(serde_json::json!({
                    "price_change_percentage": price_change * 100.0,
                    "periods": 5,
                    "threshold": momentum_threshold * 100.0
                })),
            )?);
        }

        Ok(signals)
    }

    /// Create a technical signal with all necessary data
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::result_large_err)]
    fn create_technical_signal(
        &self,
        price_series: &PriceSeries,
        signal_type: TradingSignalType,
        signal_strength: SignalStrength,
        entry_price: f64,
        confidence_score: f64,
        indicator_source: &str,
        metadata: Option<serde_json::Value>,
    ) -> ArbitrageResult<TechnicalSignal> {
        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let expires_at = now + (self.config.signal_expiry_minutes * 60 * 1000); // Convert to milliseconds

        // Calculate target price and stop loss based on signal type
        let (target_price, stop_loss) = self.calculate_price_targets(entry_price, &signal_type);

        Ok(TechnicalSignal {
            signal_id: uuid::Uuid::new_v4().to_string(),
            exchange_id: price_series.exchange_id.clone(),
            trading_pair: price_series.trading_pair.clone(),
            signal_type,
            signal_strength,
            indicator_source: indicator_source.to_string(),
            entry_price,
            target_price,
            stop_loss,
            confidence_score,
            created_at: now,
            expires_at,
            metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
        })
    }

    /// Calculate price targets based on signal type and risk management rules
    fn calculate_price_targets(
        &self,
        entry_price: f64,
        signal_type: &TradingSignalType,
    ) -> (Option<f64>, Option<f64>) {
        match signal_type {
            TradingSignalType::Buy => {
                let stop_loss = entry_price * (1.0 - self.config.default_stop_loss_percentage);
                let target_price = entry_price
                    * (1.0
                        + (self.config.default_stop_loss_percentage
                            * self.config.default_take_profit_ratio));
                (Some(target_price), Some(stop_loss))
            }
            TradingSignalType::Sell => {
                let stop_loss = entry_price * (1.0 + self.config.default_stop_loss_percentage);
                let target_price = entry_price
                    * (1.0
                        - (self.config.default_stop_loss_percentage
                            * self.config.default_take_profit_ratio));
                (Some(target_price), Some(stop_loss))
            }
            TradingSignalType::Hold => (None, None),
        }
    }

    /// Calculate confidence score for RSI signals
    fn calculate_rsi_confidence(&self, rsi_value: f64, signal_type: TradingSignalType) -> f64 {
        match signal_type {
            TradingSignalType::Buy => {
                if rsi_value <= 20.0 {
                    0.9 // Very oversold
                } else if rsi_value <= 25.0 {
                    0.8 // Strongly oversold
                } else if rsi_value <= 30.0 {
                    0.7 // Moderately oversold
                } else {
                    0.5 // Neutral
                }
            }
            TradingSignalType::Sell => {
                if rsi_value >= 80.0 {
                    0.9 // Very overbought
                } else if rsi_value >= 75.0 {
                    0.8 // Strongly overbought
                } else if rsi_value >= 70.0 {
                    0.7 // Moderately overbought
                } else {
                    0.5 // Neutral
                }
            }
            TradingSignalType::Hold => 0.5,
        }
    }

    /// Calculate standard deviation of price data for Bollinger Bands
    fn calculate_standard_deviation(&self, prices: &[f64], mean: f64) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }

        let variance = prices
            .iter()
            .map(|price| {
                let diff = price - mean;
                diff * diff
            })
            .sum::<f64>()
            / prices.len() as f64;

        variance.sqrt()
    }

    /// Calculate confidence score for moving average crossover signals
    fn calculate_crossover_confidence(
        &self,
        short_ma: f64,
        long_ma: f64,
        signal_type: TradingSignalType,
    ) -> f64 {
        let ma_diff_percentage = ((short_ma - long_ma) / long_ma).abs();

        match signal_type {
            TradingSignalType::Buy | TradingSignalType::Sell => {
                if ma_diff_percentage > 0.02 {
                    0.8 // Strong crossover
                } else if ma_diff_percentage > 0.01 {
                    0.7 // Moderate crossover
                } else {
                    0.6 // Weak crossover
                }
            }
            TradingSignalType::Hold => 0.5,
        }
    }

    /// Calculate confidence score for Bollinger Band signals
    fn calculate_bollinger_confidence(
        &self,
        price: f64,
        lower_band: f64,
        upper_band: f64,
        signal_type: TradingSignalType,
    ) -> f64 {
        let band_width = upper_band - lower_band;

        match signal_type {
            TradingSignalType::Buy => {
                let distance_below = (lower_band - price) / band_width;
                if distance_below > 0.1 {
                    0.8 // Well below lower band
                } else {
                    0.6 // Just touching lower band
                }
            }
            TradingSignalType::Sell => {
                let distance_above = (price - upper_band) / band_width;
                if distance_above > 0.1 {
                    0.8 // Well above upper band
                } else {
                    0.6 // Just touching upper band
                }
            }
            TradingSignalType::Hold => 0.5,
        }
    }

    /// Calculate confidence score for momentum signals
    fn calculate_momentum_confidence(&self, momentum_strength: f64) -> f64 {
        if momentum_strength > 0.05 {
            0.8 // Strong momentum
        } else if momentum_strength > 0.03 {
            0.7 // Moderate momentum
        } else {
            0.6 // Weak momentum
        }
    }

    /// Check if a signal passes user preference filtering
    fn passes_user_preference_filter(
        &self,
        signal: &TechnicalSignal,
        preferences: &crate::services::user_trading_preferences::UserTradingPreferences,
    ) -> bool {
        // Filter by experience level
        let min_confidence = match preferences.experience_level {
            ExperienceLevel::Beginner => 0.8,     // High confidence required
            ExperienceLevel::Intermediate => 0.6, // Moderate confidence
            ExperienceLevel::Advanced => 0.4,     // Lower confidence acceptable
        };

        if signal.confidence_score < min_confidence {
            return false;
        }

        // Filter by risk tolerance
        match preferences.risk_tolerance {
            RiskTolerance::Conservative => {
                signal.confidence_score >= 0.8
                    && matches!(
                        signal.signal_strength,
                        SignalStrength::Strong | SignalStrength::Extreme
                    )
            }
            RiskTolerance::Balanced => signal.confidence_score >= 0.6,
            RiskTolerance::Aggressive => signal.confidence_score >= 0.4,
        }
    }

    /// Convert TechnicalSignal to TradingOpportunity
    async fn convert_signal_to_opportunity(
        &self,
        signal: TechnicalSignal,
    ) -> ArbitrageResult<TradingOpportunity> {
        let risk_level = match signal.signal_strength {
            SignalStrength::Extreme => RiskLevel::Low,
            SignalStrength::Strong => RiskLevel::Medium,
            SignalStrength::Moderate => RiskLevel::Medium,
            SignalStrength::Weak => RiskLevel::High,
        };

        let time_horizon = match signal.indicator_source.as_str() {
            s if s.contains("RSI") => TimeHorizon::Short,
            s if s.contains("MA") => TimeHorizon::Medium,
            s if s.contains("BB") => TimeHorizon::Short,
            s if s.contains("Momentum") => TimeHorizon::Short,
            _ => TimeHorizon::Medium,
        };

        // Calculate expected return based on target price
        let expected_return = if let Some(target) = signal.target_price {
            match signal.signal_type {
                TradingSignalType::Buy => {
                    ((target - signal.entry_price) / signal.entry_price) * 100.0
                }
                TradingSignalType::Sell => {
                    ((signal.entry_price - target) / signal.entry_price) * 100.0
                }
                TradingSignalType::Hold => 0.0,
            }
        } else {
            0.0
        };

        Ok(TradingOpportunity {
            opportunity_id: signal.signal_id,
            opportunity_type: OpportunityType::Technical,
            trading_pair: signal.trading_pair,
            exchanges: vec![signal.exchange_id],
            entry_price: signal.entry_price,
            target_price: signal.target_price,
            stop_loss: signal.stop_loss,
            confidence_score: signal.confidence_score,
            risk_level,
            expected_return,
            time_horizon,
            indicators_used: vec![signal.indicator_source],
            analysis_data: serde_json::json!({
                "signal_type": signal.signal_type,
                "signal_strength": signal.signal_strength,
                "technical_analysis": true,
                "signal_metadata": signal.metadata
            }),
            created_at: signal.created_at,
            expires_at: Some(signal.expires_at),
        })
    }

    pub fn get_config(&self) -> &TechnicalTradingServiceConfig {
        &self.config
    }
}
