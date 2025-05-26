use crate::types::{ArbitrageOpportunity, CommandPermission, ExchangeIdEnum};
use crate::utils::ArbitrageResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Technical Analysis Signal Types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    RsiDivergence,
    SupportResistance,
    MovingAverageCrossover,
    BollingerBandBreakout,
    VolumeSpike,
    TrendConfirmation,
    PatternRecognition,
}

/// Signal Strength Levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

/// Trading Signal Direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalDirection {
    Buy,
    Sell,
    Hold,
    Neutral,
}

/// Timeframe for analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    M1,  // 1 minute
    M5,  // 5 minutes
    M15, // 15 minutes
    M30, // 30 minutes
    H1,  // 1 hour
    H4,  // 4 hours
    H12, // 12 hours
    D1,  // 1 day
    W1,  // 1 week
}

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timeframe::M1 => write!(f, "1m"),
            Timeframe::M5 => write!(f, "5m"),
            Timeframe::M15 => write!(f, "15m"),
            Timeframe::M30 => write!(f, "30m"),
            Timeframe::H1 => write!(f, "1h"),
            Timeframe::H4 => write!(f, "4h"),
            Timeframe::H12 => write!(f, "12h"),
            Timeframe::D1 => write!(f, "1d"),
            Timeframe::W1 => write!(f, "1w"),
        }
    }
}

/// Technical Analysis Signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSignal {
    pub id: String,
    pub pair: String,
    pub exchange: ExchangeIdEnum,
    pub signal_type: SignalType,
    pub direction: SignalDirection,
    pub strength: SignalStrength,
    pub timeframe: Timeframe,
    pub current_price: f64,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub confidence: f64, // 0.0 to 1.0
    pub description: String,
    pub generated_at: u64, // timestamp
    pub expires_at: u64,   // timestamp
    pub metadata: serde_json::Value,
}

impl TechnicalSignal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pair: String,
        exchange: ExchangeIdEnum,
        signal_type: SignalType,
        direction: SignalDirection,
        strength: SignalStrength,
        timeframe: Timeframe,
        current_price: f64,
        confidence: f64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let expires_at = now + (24 * 60 * 60 * 1000); // 24 hours expiry

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pair,
            exchange,
            signal_type,
            direction,
            strength,
            timeframe,
            current_price,
            target_price: None,
            stop_loss: None,
            confidence,
            description: String::new(),
            generated_at: now,
            expires_at,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn with_target_price(mut self, target_price: f64) -> Self {
        self.target_price = Some(target_price);
        self
    }

    pub fn with_stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now > self.expires_at
    }

    pub fn calculate_profit_potential(&self) -> Option<f64> {
        if let Some(target) = self.target_price {
            match self.direction {
                SignalDirection::Buy => {
                    Some((target - self.current_price) / self.current_price * 100.0)
                }
                SignalDirection::Sell => {
                    Some((self.current_price - target) / self.current_price * 100.0)
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Global Technical Analysis Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAnalysisConfig {
    pub enabled_exchanges: Vec<ExchangeIdEnum>,
    pub monitored_pairs: Vec<String>,
    pub enabled_signals: Vec<SignalType>,
    pub min_confidence_threshold: f64,
    pub max_signals_per_hour: u32,
    pub signal_expiry_hours: u32,
    pub enable_multi_timeframe: bool,
    pub primary_timeframes: Vec<Timeframe>,
}

impl Default for TechnicalAnalysisConfig {
    fn default() -> Self {
        Self {
            enabled_exchanges: vec![
                ExchangeIdEnum::Binance,
                ExchangeIdEnum::Bybit,
                ExchangeIdEnum::OKX,
                ExchangeIdEnum::Bitget,
            ],
            monitored_pairs: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "ADAUSDT".to_string(),
                "SOLUSDT".to_string(),
                "BNBUSDT".to_string(),
                "MATICUSDT".to_string(),
            ],
            enabled_signals: vec![
                SignalType::RsiDivergence,
                SignalType::SupportResistance,
                SignalType::MovingAverageCrossover,
                SignalType::BollingerBandBreakout,
            ],
            min_confidence_threshold: 0.7,
            max_signals_per_hour: 10,
            signal_expiry_hours: 24,
            enable_multi_timeframe: true,
            primary_timeframes: vec![Timeframe::H1, Timeframe::H4, Timeframe::D1],
        }
    }
}

/// Technical Analysis Service for Global Signal Generation
pub struct TechnicalAnalysisService {
    config: TechnicalAnalysisConfig,
    active_signals: HashMap<String, TechnicalSignal>,
    signal_history: Vec<TechnicalSignal>,
}

impl TechnicalAnalysisService {
    pub fn new(config: TechnicalAnalysisConfig) -> Self {
        Self {
            config,
            active_signals: HashMap::new(),
            signal_history: Vec::new(),
        }
    }

    /// Generate technical analysis signals for all monitored pairs
    pub async fn generate_global_signals(&mut self) -> ArbitrageResult<Vec<TechnicalSignal>> {
        let mut signals = Vec::new();

        for pair in &self.config.monitored_pairs {
            for exchange in &self.config.enabled_exchanges {
                // Generate signals for different timeframes
                for timeframe in &self.config.primary_timeframes {
                    if let Ok(signal) = self.analyze_pair(pair, exchange, timeframe).await {
                        if signal.confidence >= self.config.min_confidence_threshold {
                            signals.push(signal);
                        }
                    }
                }
            }
        }

        // Update active signals
        self.update_active_signals(&signals);

        Ok(signals)
    }

    /// Analyze a specific trading pair for technical signals
    async fn analyze_pair(
        &self,
        pair: &str,
        exchange: &ExchangeIdEnum,
        timeframe: &Timeframe,
    ) -> ArbitrageResult<TechnicalSignal> {
        // TODO: In production, this would fetch real market data and perform actual TA
        // For now, generate mock signals based on different scenarios

        let current_price = self.get_mock_current_price(pair);
        let (signal_type, direction, strength, confidence) =
            self.get_mock_analysis(pair, timeframe);

        let mut signal = TechnicalSignal::new(
            pair.to_string(),
            *exchange,
            signal_type,
            direction,
            strength,
            timeframe.clone(),
            current_price,
            confidence,
        );

        signal = self.enhance_signal_with_targets(signal);

        Ok(signal)
    }

    /// Get mock current price for a trading pair
    fn get_mock_current_price(&self, pair: &str) -> f64 {
        match pair {
            "BTCUSDT" => 43250.50,
            "ETHUSDT" => 2634.75,
            "ADAUSDT" => 0.4821,
            "SOLUSDT" => 138.92,
            "BNBUSDT" => 312.45,
            "MATICUSDT" => 0.8567,
            _ => 100.0,
        }
    }

    /// Generate mock technical analysis for a pair
    fn get_mock_analysis(
        &self,
        pair: &str,
        timeframe: &Timeframe,
    ) -> (SignalType, SignalDirection, SignalStrength, f64) {
        // Generate different signals based on pair and timeframe
        match (pair, timeframe) {
            ("BTCUSDT", Timeframe::H4) => (
                SignalType::RsiDivergence,
                SignalDirection::Buy,
                SignalStrength::Strong,
                0.89,
            ),
            ("ETHUSDT", Timeframe::H1) => (
                SignalType::SupportResistance,
                SignalDirection::Sell,
                SignalStrength::Medium,
                0.76,
            ),
            ("ADAUSDT", Timeframe::D1) => (
                SignalType::MovingAverageCrossover,
                SignalDirection::Buy,
                SignalStrength::VeryStrong,
                0.94,
            ),
            ("SOLUSDT", Timeframe::H4) => (
                SignalType::BollingerBandBreakout,
                SignalDirection::Buy,
                SignalStrength::Strong,
                0.87,
            ),
            ("BNBUSDT", Timeframe::H1) => (
                SignalType::VolumeSpike,
                SignalDirection::Hold,
                SignalStrength::Weak,
                0.65,
            ),
            _ => (
                SignalType::TrendConfirmation,
                SignalDirection::Neutral,
                SignalStrength::Medium,
                0.72,
            ),
        }
    }

    /// Enhance signal with target prices and stop losses
    fn enhance_signal_with_targets(&self, mut signal: TechnicalSignal) -> TechnicalSignal {
        let signal_type = signal.signal_type.clone();
        let pair = signal.pair.clone();
        let timeframe = signal.timeframe.clone();

        match signal.direction {
            SignalDirection::Buy => {
                let target = signal.current_price * 1.04; // 4% target
                let stop_loss = signal.current_price * 0.98; // 2% stop loss
                signal = signal
                    .with_target_price(target)
                    .with_stop_loss(stop_loss)
                    .with_description(format!(
                        "{:?} signal detected on {} {}. Buy signal with 4% upside target.",
                        signal_type, pair, timeframe
                    ));
            }
            SignalDirection::Sell => {
                let target = signal.current_price * 0.96; // 4% target
                let stop_loss = signal.current_price * 1.02; // 2% stop loss
                signal = signal
                    .with_target_price(target)
                    .with_stop_loss(stop_loss)
                    .with_description(format!(
                        "{:?} signal detected on {} {}. Sell signal with 4% downside target.",
                        signal_type, pair, timeframe
                    ));
            }
            _ => {
                signal = signal.with_description(format!(
                    "{:?} signal detected on {} {}. Monitor for trend development.",
                    signal_type, pair, timeframe
                ));
            }
        }

        signal
    }

    /// Update active signals, removing expired ones
    fn update_active_signals(&mut self, new_signals: &[TechnicalSignal]) {
        // Remove expired signals
        self.active_signals.retain(|_, signal| !signal.is_expired());

        // Add new signals
        for signal in new_signals {
            self.active_signals
                .insert(signal.id.clone(), signal.clone());
        }

        // Move old signals to history
        let mut expired_signals = Vec::new();
        self.active_signals.retain(|_, signal| {
            if signal.is_expired() {
                expired_signals.push(signal.clone());
                false
            } else {
                true
            }
        });

        self.signal_history.extend(expired_signals);
    }

    /// Get active signals filtered by user permissions
    pub fn get_signals_for_user(
        &self,
        user_permissions: &[CommandPermission],
    ) -> Vec<TechnicalSignal> {
        // Check if user has technical analysis access
        if user_permissions.contains(&CommandPermission::TechnicalAnalysis) {
            self.active_signals.values().cloned().collect()
        } else {
            // Return empty for users without TA access
            Vec::new()
        }
    }

    /// Get signals for a specific pair
    pub fn get_signals_for_pair(&self, pair: &str) -> Vec<TechnicalSignal> {
        self.active_signals
            .values()
            .filter(|signal| signal.pair == pair)
            .cloned()
            .collect()
    }

    /// Get signals by timeframe
    pub fn get_signals_by_timeframe(&self, timeframe: &Timeframe) -> Vec<TechnicalSignal> {
        self.active_signals
            .values()
            .filter(|signal| signal.timeframe == *timeframe)
            .cloned()
            .collect()
    }

    /// Convert technical signal to arbitrage opportunity (for integration)
    /// Note: This creates a pseudo-arbitrage opportunity for technical signals
    /// In practice, technical signals should use TechnicalOpportunity instead
    pub fn signal_to_opportunity(&self, signal: &TechnicalSignal) -> ArbitrageOpportunity {
        let profit_potential = signal.calculate_profit_potential().unwrap_or(0.0);

        // For technical signals, we use the same exchange for both long and short
        // This is a temporary compatibility method - should use TechnicalOpportunity instead
        match ArbitrageOpportunity::new(
            signal.pair.clone(),
            signal.exchange, // Use same exchange for long
            signal.exchange, // Use same exchange for short (pseudo-arbitrage)
            Some(signal.current_price),
            signal.target_price,
            profit_potential,
            crate::types::ArbitrageType::CrossExchange, // Closest type for TA signals
        ) {
            Ok(opp) => opp.with_details(format!("Technical Analysis: {}", signal.description)),
            Err(_) => {
                // Fallback to a valid opportunity if creation fails
                ArbitrageOpportunity::new(
                    "BTCUSDT".to_string(), // Fallback pair
                    signal.exchange,
                    signal.exchange,
                    Some(signal.current_price),
                    signal.target_price,
                    profit_potential,
                    crate::types::ArbitrageType::CrossExchange,
                )
                .unwrap_or_else(|_| {
                    // Last resort fallback
                    ArbitrageOpportunity {
                        pair: signal.pair.clone(),
                        long_exchange: signal.exchange,
                        short_exchange: signal.exchange,
                        ..Default::default()
                    }
                })
                .with_details(format!("Technical Analysis: {}", signal.description))
            }
        }
    }

    /// Get technical analysis statistics
    pub fn get_statistics(&self) -> TechnicalAnalysisStats {
        let total_signals = self.active_signals.len();
        let buy_signals = self
            .active_signals
            .values()
            .filter(|s| s.direction == SignalDirection::Buy)
            .count();
        let sell_signals = self
            .active_signals
            .values()
            .filter(|s| s.direction == SignalDirection::Sell)
            .count();

        let avg_confidence = if total_signals > 0 {
            self.active_signals
                .values()
                .map(|s| s.confidence)
                .sum::<f64>()
                / total_signals as f64
        } else {
            0.0
        };

        TechnicalAnalysisStats {
            total_active_signals: total_signals,
            buy_signals,
            sell_signals,
            hold_signals: total_signals - buy_signals - sell_signals,
            average_confidence: avg_confidence,
            monitored_pairs: self.config.monitored_pairs.len(),
            enabled_exchanges: self.config.enabled_exchanges.len(),
        }
    }
}

/// Statistics for technical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAnalysisStats {
    pub total_active_signals: usize,
    pub buy_signals: usize,
    pub sell_signals: usize,
    pub hold_signals: usize,
    pub average_confidence: f64,
    pub monitored_pairs: usize,
    pub enabled_exchanges: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technical_signal_creation() {
        let signal = TechnicalSignal::new(
            "BTCUSDT".to_string(),
            ExchangeIdEnum::Binance,
            SignalType::RsiDivergence,
            SignalDirection::Buy,
            SignalStrength::Strong,
            Timeframe::H4,
            43250.0,
            0.89,
        );

        assert_eq!(signal.pair, "BTCUSDT");
        assert_eq!(signal.exchange, ExchangeIdEnum::Binance);
        assert_eq!(signal.signal_type, SignalType::RsiDivergence);
        assert_eq!(signal.direction, SignalDirection::Buy);
        assert_eq!(signal.strength, SignalStrength::Strong);
        assert_eq!(signal.confidence, 0.89);
        assert!(!signal.is_expired());
    }

    #[test]
    fn test_signal_profit_calculation() {
        let mut signal = TechnicalSignal::new(
            "ETHUSDT".to_string(),
            ExchangeIdEnum::Bybit,
            SignalType::SupportResistance,
            SignalDirection::Buy,
            SignalStrength::Medium,
            Timeframe::H1,
            2000.0,
            0.75,
        );

        signal = signal.with_target_price(2080.0);

        let profit = signal.calculate_profit_potential().unwrap();
        assert!((profit - 4.0).abs() < 0.01); // Should be 4%
    }

    #[test]
    fn test_technical_analysis_config_default() {
        let config = TechnicalAnalysisConfig::default();

        assert_eq!(config.enabled_exchanges.len(), 4);
        assert!(config.enabled_exchanges.contains(&ExchangeIdEnum::Binance));
        assert_eq!(config.monitored_pairs.len(), 6);
        assert!(config.monitored_pairs.contains(&"BTCUSDT".to_string()));
        assert_eq!(config.min_confidence_threshold, 0.7);
        assert_eq!(config.max_signals_per_hour, 10);
    }

    #[tokio::test]
    async fn test_technical_analysis_service_creation() {
        let config = TechnicalAnalysisConfig::default();
        let service = TechnicalAnalysisService::new(config);

        assert_eq!(service.active_signals.len(), 0);
        assert_eq!(service.signal_history.len(), 0);
    }

    #[tokio::test]
    async fn test_generate_global_signals() {
        let config = TechnicalAnalysisConfig::default();
        let mut service = TechnicalAnalysisService::new(config);

        let signals = service.generate_global_signals().await.unwrap();

        // Should generate signals for configured pairs and timeframes
        assert!(!signals.is_empty());

        // All signals should meet confidence threshold
        for signal in &signals {
            assert!(signal.confidence >= 0.7);
        }
    }

    #[test]
    fn test_timeframe_display() {
        assert_eq!(Timeframe::M1.to_string(), "1m");
        assert_eq!(Timeframe::H4.to_string(), "4h");
        assert_eq!(Timeframe::D1.to_string(), "1d");
        assert_eq!(Timeframe::W1.to_string(), "1w");
    }

    #[test]
    fn test_signal_to_opportunity_conversion() {
        let config = TechnicalAnalysisConfig::default();
        let service = TechnicalAnalysisService::new(config);

        let signal = TechnicalSignal::new(
            "BTCUSDT".to_string(),
            ExchangeIdEnum::Binance,
            SignalType::RsiDivergence,
            SignalDirection::Buy,
            SignalStrength::Strong,
            Timeframe::H4,
            43250.0,
            0.89,
        )
        .with_target_price(45000.0);

        let opportunity = service.signal_to_opportunity(&signal);

        assert_eq!(opportunity.pair, "BTCUSDT");
        assert_eq!(opportunity.long_exchange, ExchangeIdEnum::Binance);
        assert!(opportunity.details.is_some());
    }
}
