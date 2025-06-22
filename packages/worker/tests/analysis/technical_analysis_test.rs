use arbedge_worker::analysis::technical_analysis::*;
use arbedge_worker::utils::logger::{Logger, LogLevel};
use arbedge_worker::types::{ExchangeIdEnum, Timeframe};

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
        let logger = Logger::new(LogLevel::Info);
        let service = TechnicalAnalysisService::new(config, logger);

        assert_eq!(service.active_signals.len(), 0);
        assert_eq!(service.signal_history.len(), 0);
    }

    #[tokio::test]
    async fn test_generate_global_signals() {
        let config = TechnicalAnalysisConfig::default();
        let logger = Logger::new(LogLevel::Info);
        let mut service = TechnicalAnalysisService::new(config, logger);

        // Generate test signals using mock data to avoid network calls
        let signals = service.generate_test_signals().await.unwrap();

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
        let logger = Logger::new(LogLevel::Info);
        let service = TechnicalAnalysisService::new(config, logger);

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