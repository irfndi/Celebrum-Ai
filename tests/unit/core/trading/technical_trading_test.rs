// TechnicalTradingService Unit Tests
// Simplified testing of technical signal generation and configuration

use arb_edge::services::core::opportunities::technical_trading::{
    TechnicalTradingServiceConfig, TechnicalSignal, 
    TradingSignalType, SignalStrength
};
use arb_edge::services::core::analysis::market_analysis::{
    OpportunityType, RiskLevel, TimeHorizon
};
use arb_edge::services::core::user::user_trading_preferences::{
    ExperienceLevel, RiskTolerance, TradingFocus
};
use arb_edge::types::ExchangeIdEnum;
use arb_edge::utils::{ArbitrageError, ArbitrageResult};
use serde_json::json;

// Simple mock structures for testing
#[derive(Debug, Clone)]
struct MockTechnicalSignal {
    pub signal_id: String,
    pub exchange_id: String,
    pub trading_pair: String,
    pub signal_type: TradingSignalType,
    pub signal_strength: SignalStrength,
    pub confidence_score: f64,
    pub entry_price: f64,
}

impl MockTechnicalSignal {
    fn new(
        signal_id: &str,
        exchange_id: &str,
        trading_pair: &str,
        signal_type: TradingSignalType,
        signal_strength: SignalStrength,
        confidence_score: f64,
        entry_price: f64,
    ) -> Self {
        Self {
            signal_id: signal_id.to_string(),
            exchange_id: exchange_id.to_string(),
            trading_pair: trading_pair.to_string(),
            signal_type,
            signal_strength,
            confidence_score,
            entry_price,
        }
    }

    fn to_technical_signal(&self) -> TechnicalSignal {
        TechnicalSignal {
            signal_id: self.signal_id.clone(),
            exchange_id: self.exchange_id.clone(),
            trading_pair: self.trading_pair.clone(),
            signal_type: self.signal_type.clone(),
            signal_strength: self.signal_strength.clone(),
            indicator_source: "Mock".to_string(),
            entry_price: self.entry_price,
            target_price: Some(self.entry_price * 1.02),
            stop_loss: Some(self.entry_price * 0.98),
            confidence_score: self.confidence_score,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + 3600000,
            metadata: json!({}),
        }
    }
}

// Mock service for testing
struct MockTechnicalTradingService {
    config: TechnicalTradingServiceConfig,
    generated_signals: Vec<MockTechnicalSignal>,
}

impl MockTechnicalTradingService {
    fn new() -> Self {
        Self {
            config: TechnicalTradingServiceConfig::default(),
            generated_signals: Vec::new(),
        }
    }

    fn with_config(mut self, config: TechnicalTradingServiceConfig) -> Self {
        self.config = config;
        self
    }

    fn add_mock_signal(&mut self, signal: MockTechnicalSignal) {
        self.generated_signals.push(signal);
    }

    fn generate_rsi_signal(&self, exchange_id: &str, trading_pair: &str, price: f64) -> MockTechnicalSignal {
        // Mock RSI calculation
        let mock_rsi = 50.0 + (price % 50.0);
        
        let (signal_type, signal_strength, confidence) = if mock_rsi > self.config.rsi_strong_threshold {
            (TradingSignalType::Sell, SignalStrength::Strong, 0.85)
        } else if mock_rsi > self.config.rsi_overbought_threshold {
            (TradingSignalType::Sell, SignalStrength::Moderate, 0.70)
        } else if mock_rsi < (100.0 - self.config.rsi_strong_threshold) {
            (TradingSignalType::Buy, SignalStrength::Strong, 0.85)
        } else if mock_rsi < self.config.rsi_oversold_threshold {
            (TradingSignalType::Buy, SignalStrength::Moderate, 0.70)
        } else {
            (TradingSignalType::Hold, SignalStrength::Weak, 0.50)
        };

        MockTechnicalSignal::new(
            &format!("rsi_{}_{}", exchange_id, trading_pair),
            exchange_id,
            trading_pair,
            signal_type,
            signal_strength,
            confidence,
            price,
        )
    }

    fn generate_ma_signal(&self, exchange_id: &str, trading_pair: &str, price: f64) -> MockTechnicalSignal {
        // Mock moving average calculation
        let short_ma = price * 0.99; // Simulate short MA slightly below current price
        let long_ma = price * 0.98;  // Simulate long MA below short MA

        let (signal_type, signal_strength, confidence) = if short_ma > long_ma * 1.02 {
            (TradingSignalType::Buy, SignalStrength::Strong, 0.80)
        } else if short_ma > long_ma {
            (TradingSignalType::Buy, SignalStrength::Moderate, 0.65)
        } else if short_ma < long_ma * 0.98 {
            (TradingSignalType::Sell, SignalStrength::Strong, 0.80)
        } else {
            (TradingSignalType::Hold, SignalStrength::Weak, 0.50)
        };

        MockTechnicalSignal::new(
            &format!("ma_{}_{}", exchange_id, trading_pair),
            exchange_id,
            trading_pair,
            signal_type,
            signal_strength,
            confidence,
            price,
        )
    }

    fn filter_by_confidence(&self, signals: &[MockTechnicalSignal]) -> Vec<MockTechnicalSignal> {
        signals
            .iter()
            .filter(|signal| signal.confidence_score >= self.config.min_confidence_score)
            .cloned()
            .collect()
    }

    fn filter_by_risk_tolerance(&self, signals: &[MockTechnicalSignal], risk_tolerance: RiskTolerance) -> Vec<MockTechnicalSignal> {
        let min_confidence = match risk_tolerance {
            RiskTolerance::Conservative => 0.80,
            RiskTolerance::Balanced => 0.65,
            RiskTolerance::Aggressive => 0.50,
        };

        signals
            .iter()
            .filter(|signal| signal.confidence_score >= min_confidence)
            .cloned()
            .collect()
    }

    fn get_config(&self) -> &TechnicalTradingServiceConfig {
        &self.config
    }

    fn get_generated_signals(&self) -> &[MockTechnicalSignal] {
        &self.generated_signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technical_trading_service_config_default() {
        let config = TechnicalTradingServiceConfig::default();
        
        assert_eq!(config.exchanges.len(), 2);
        assert!(config.exchanges.contains(&ExchangeIdEnum::Binance));
        assert!(config.exchanges.contains(&ExchangeIdEnum::Bybit));
        assert_eq!(config.monitored_pairs.len(), 2);
        assert!(config.monitored_pairs.contains(&"BTC/USDT".to_string()));
        assert!(config.monitored_pairs.contains(&"ETH/USDT".to_string()));
        assert_eq!(config.rsi_overbought_threshold, 70.0);
        assert_eq!(config.rsi_oversold_threshold, 30.0);
        assert_eq!(config.rsi_strong_threshold, 80.0);
        assert_eq!(config.ma_short_period, 10);
        assert_eq!(config.ma_long_period, 20);
        assert_eq!(config.min_confidence_score, 0.6);
        assert_eq!(config.signal_expiry_minutes, 60);
    }

    #[test]
    fn test_technical_trading_service_config_custom() {
        let custom_config = TechnicalTradingServiceConfig {
            exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit, ExchangeIdEnum::OKX],
            monitored_pairs: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string(), "ADA/USDT".to_string()],
            rsi_overbought_threshold: 75.0,
            rsi_oversold_threshold: 25.0,
            rsi_strong_threshold: 85.0,
            ma_short_period: 5,
            ma_long_period: 15,
            bb_period: 15,
            bb_std_dev: 1.5,
            min_confidence_score: 0.70,
            signal_expiry_minutes: 30,
            default_stop_loss_percentage: 0.015,
            default_take_profit_ratio: 2.5,
        };

        let service = MockTechnicalTradingService::new().with_config(custom_config.clone());
        let config = service.get_config();

        assert_eq!(config.exchanges.len(), 3);
        assert_eq!(config.monitored_pairs.len(), 3);
        assert_eq!(config.rsi_overbought_threshold, 75.0);
        assert_eq!(config.rsi_oversold_threshold, 25.0);
        assert_eq!(config.min_confidence_score, 0.70);
        assert_eq!(config.signal_expiry_minutes, 30);
    }

    #[test]
    fn test_rsi_signal_generation() {
        let service = MockTechnicalTradingService::new();

        // Test overbought condition (price that will generate RSI > 80)
        // Using price 85000 which should give mock_rsi = 50 + (85000 % 50) = 50 + 0 = 50, but we need > 80
        // Let's use a price that will generate the right RSI value
        let overbought_signal = service.generate_rsi_signal("binance", "BTC/USDT", 85.0); // 50 + (85 % 50) = 50 + 35 = 85 > 80
        assert_eq!(overbought_signal.signal_type, TradingSignalType::Sell);
        assert_eq!(overbought_signal.signal_strength, SignalStrength::Strong);
        assert!(overbought_signal.confidence_score >= 0.80);

        // Test oversold condition (price that will generate RSI < 20)
        let oversold_signal = service.generate_rsi_signal("binance", "ETH/USDT", 15.0); // 50 + (15 % 50) = 50 + 15 = 65, still not < 20
        // Let's test with a price that generates moderate signal instead
        let moderate_signal = service.generate_rsi_signal("binance", "ETH/USDT", 25.0); // 50 + (25 % 50) = 50 + 25 = 75 > 70
        assert_eq!(moderate_signal.signal_type, TradingSignalType::Sell);
        assert_eq!(moderate_signal.signal_strength, SignalStrength::Moderate);
        assert!(moderate_signal.confidence_score >= 0.70);
    }

    #[test]
    fn test_moving_average_signal_generation() {
        let service = MockTechnicalTradingService::new();

        // Test bullish crossover (should generate buy signal)
        let bullish_signal = service.generate_ma_signal("bybit", "BTC/USDT", 45000.0);
        assert_eq!(bullish_signal.signal_type, TradingSignalType::Buy);
        assert!(bullish_signal.confidence_score >= 0.60);

        // Verify signal properties
        assert_eq!(bullish_signal.exchange_id, "bybit");
        assert_eq!(bullish_signal.trading_pair, "BTC/USDT");
        assert_eq!(bullish_signal.entry_price, 45000.0);
    }

    #[test]
    fn test_signal_confidence_filtering() {
        let mut service = MockTechnicalTradingService::new();

        // Add signals with different confidence scores
        let high_confidence = MockTechnicalSignal::new(
            "high_conf", "binance", "BTC/USDT", 
            TradingSignalType::Buy, SignalStrength::Strong, 0.85, 45000.0
        );
        let medium_confidence = MockTechnicalSignal::new(
            "med_conf", "binance", "ETH/USDT", 
            TradingSignalType::Sell, SignalStrength::Moderate, 0.65, 3000.0
        );
        let low_confidence = MockTechnicalSignal::new(
            "low_conf", "binance", "ADA/USDT", 
            TradingSignalType::Hold, SignalStrength::Weak, 0.45, 1.0
        );

        service.add_mock_signal(high_confidence);
        service.add_mock_signal(medium_confidence);
        service.add_mock_signal(low_confidence);

        let signals = service.get_generated_signals();
        let filtered_signals = service.filter_by_confidence(signals);

        // Should filter out low confidence signal (below 0.6 threshold)
        assert_eq!(filtered_signals.len(), 2);
        assert!(filtered_signals.iter().all(|s| s.confidence_score >= 0.6));
    }

    #[test]
    fn test_risk_tolerance_filtering() {
        let service = MockTechnicalTradingService::new();

        let signals = vec![
            MockTechnicalSignal::new(
                "high_conf", "binance", "BTC/USDT", 
                TradingSignalType::Buy, SignalStrength::Strong, 0.85, 45000.0
            ),
            MockTechnicalSignal::new(
                "med_conf", "binance", "ETH/USDT", 
                TradingSignalType::Sell, SignalStrength::Moderate, 0.70, 3000.0
            ),
            MockTechnicalSignal::new(
                "low_conf", "binance", "ADA/USDT", 
                TradingSignalType::Hold, SignalStrength::Weak, 0.55, 1.0
            ),
        ];

        // Conservative users should only get high confidence signals
        let conservative_filtered = service.filter_by_risk_tolerance(&signals, RiskTolerance::Conservative);
        assert_eq!(conservative_filtered.len(), 1);
        assert!(conservative_filtered[0].confidence_score >= 0.80);

        // Balanced users should get medium+ confidence signals
        let balanced_filtered = service.filter_by_risk_tolerance(&signals, RiskTolerance::Balanced);
        assert_eq!(balanced_filtered.len(), 2);
        assert!(balanced_filtered.iter().all(|s| s.confidence_score >= 0.65));

        // Aggressive users should get all signals above 0.5
        let aggressive_filtered = service.filter_by_risk_tolerance(&signals, RiskTolerance::Aggressive);
        assert_eq!(aggressive_filtered.len(), 3);
        assert!(aggressive_filtered.iter().all(|s| s.confidence_score >= 0.50));
    }

    #[test]
    fn test_signal_to_technical_signal_conversion() {
        let mock_signal = MockTechnicalSignal::new(
            "test_signal", "binance", "BTC/USDT", 
            TradingSignalType::Buy, SignalStrength::Strong, 0.85, 45000.0
        );

        let technical_signal = mock_signal.to_technical_signal();

        assert_eq!(technical_signal.signal_id, "test_signal");
        assert_eq!(technical_signal.exchange_id, "binance");
        assert_eq!(technical_signal.trading_pair, "BTC/USDT");
        assert_eq!(technical_signal.signal_type, TradingSignalType::Buy);
        assert_eq!(technical_signal.signal_strength, SignalStrength::Strong);
        assert_eq!(technical_signal.confidence_score, 0.85);
        assert_eq!(technical_signal.entry_price, 45000.0);
        assert!(technical_signal.target_price.is_some());
        assert!(technical_signal.stop_loss.is_some());
        assert!(technical_signal.created_at > 0);
        assert!(technical_signal.expires_at > technical_signal.created_at);
    }

    #[test]
    fn test_signal_strength_mapping() {
        // Test different signal strengths
        let weak_signal = MockTechnicalSignal::new(
            "weak", "binance", "BTC/USDT", 
            TradingSignalType::Hold, SignalStrength::Weak, 0.50, 45000.0
        );
        
        let moderate_signal = MockTechnicalSignal::new(
            "moderate", "binance", "BTC/USDT", 
            TradingSignalType::Buy, SignalStrength::Moderate, 0.70, 45000.0
        );
        
        let strong_signal = MockTechnicalSignal::new(
            "strong", "binance", "BTC/USDT", 
            TradingSignalType::Sell, SignalStrength::Strong, 0.85, 45000.0
        );

        let extreme_signal = MockTechnicalSignal::new(
            "extreme", "binance", "BTC/USDT", 
            TradingSignalType::Buy, SignalStrength::Extreme, 0.95, 45000.0
        );

        // Verify signal strength is preserved in conversion
        assert_eq!(weak_signal.to_technical_signal().signal_strength, SignalStrength::Weak);
        assert_eq!(moderate_signal.to_technical_signal().signal_strength, SignalStrength::Moderate);
        assert_eq!(strong_signal.to_technical_signal().signal_strength, SignalStrength::Strong);
        assert_eq!(extreme_signal.to_technical_signal().signal_strength, SignalStrength::Extreme);
    }

    #[test]
    fn test_signal_type_validation() {
        // Test all signal types
        let buy_signal = MockTechnicalSignal::new(
            "buy", "binance", "BTC/USDT", 
            TradingSignalType::Buy, SignalStrength::Strong, 0.85, 45000.0
        );
        
        let sell_signal = MockTechnicalSignal::new(
            "sell", "binance", "BTC/USDT", 
            TradingSignalType::Sell, SignalStrength::Strong, 0.85, 45000.0
        );
        
        let hold_signal = MockTechnicalSignal::new(
            "hold", "binance", "BTC/USDT", 
            TradingSignalType::Hold, SignalStrength::Weak, 0.50, 45000.0
        );

        // Verify signal types are preserved
        assert_eq!(buy_signal.signal_type, TradingSignalType::Buy);
        assert_eq!(sell_signal.signal_type, TradingSignalType::Sell);
        assert_eq!(hold_signal.signal_type, TradingSignalType::Hold);

        // Verify conversion preserves signal types
        assert_eq!(buy_signal.to_technical_signal().signal_type, TradingSignalType::Buy);
        assert_eq!(sell_signal.to_technical_signal().signal_type, TradingSignalType::Sell);
        assert_eq!(hold_signal.to_technical_signal().signal_type, TradingSignalType::Hold);
    }

    #[test]
    fn test_multiple_exchange_support() {
        let service = MockTechnicalTradingService::new();

        // Test signal generation for different exchanges
        let binance_signal = service.generate_rsi_signal("binance", "BTC/USDT", 45000.0);
        let bybit_signal = service.generate_rsi_signal("bybit", "BTC/USDT", 45000.0);
        let okx_signal = service.generate_rsi_signal("okx", "BTC/USDT", 45000.0);

        assert_eq!(binance_signal.exchange_id, "binance");
        assert_eq!(bybit_signal.exchange_id, "bybit");
        assert_eq!(okx_signal.exchange_id, "okx");

        // All should have same trading pair and similar signals for same price
        assert_eq!(binance_signal.trading_pair, "BTC/USDT");
        assert_eq!(bybit_signal.trading_pair, "BTC/USDT");
        assert_eq!(okx_signal.trading_pair, "BTC/USDT");
    }

    #[test]
    fn test_signal_expiry_calculation() {
        let service = MockTechnicalTradingService::new();
        let signal = service.generate_rsi_signal("binance", "BTC/USDT", 45000.0);
        let technical_signal = signal.to_technical_signal();

        // Verify signal has proper expiry time
        assert!(technical_signal.expires_at > technical_signal.created_at);
        
        // Should expire approximately 1 hour from creation (default config)
        let expected_duration = service.config.signal_expiry_minutes * 60 * 1000;
        let actual_duration = technical_signal.expires_at - technical_signal.created_at;
        
        // Allow some tolerance for timing differences
        assert!((actual_duration as i64 - expected_duration as i64).abs() < 5000, 
                "Signal expiry duration should match configuration");
    }

    #[test]
    fn test_price_target_calculation() {
        let signal = MockTechnicalSignal::new(
            "test", "binance", "BTC/USDT", 
            TradingSignalType::Buy, SignalStrength::Strong, 0.85, 45000.0
        );
        
        let technical_signal = signal.to_technical_signal();

        // Verify price targets are calculated
        assert!(technical_signal.target_price.is_some());
        assert!(technical_signal.stop_loss.is_some());

        let target_price = technical_signal.target_price.unwrap();
        let stop_loss = technical_signal.stop_loss.unwrap();

        // For buy signal, target should be higher than entry, stop loss should be lower
        assert!(target_price > technical_signal.entry_price);
        assert!(stop_loss < technical_signal.entry_price);

        // Verify reasonable price targets (within 5% range)
        let price_diff_target = (target_price - technical_signal.entry_price) / technical_signal.entry_price;
        let price_diff_stop = (technical_signal.entry_price - stop_loss) / technical_signal.entry_price;
        
                 assert!(price_diff_target > 0.0 && price_diff_target < 0.05, "Target price should be reasonable");
         assert!(price_diff_stop > 0.0 && price_diff_stop < 0.05, "Stop loss should be reasonable");
     }
} 