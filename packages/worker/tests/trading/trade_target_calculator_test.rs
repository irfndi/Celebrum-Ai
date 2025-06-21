use cerebrum_ai::services::core::opportunities::trade_target_calculator::{TradeTargetCalculator, TradeTargets};
use cerebrum_ai::types::TradingSettings;
use cerebrum_ai::utils::{ArbitrageError, ArbitrageResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_calculation() {
        let targets = TradeTargetCalculator::calculate(10_000.0, None, None).unwrap();
        assert_eq!(targets.stop_loss_price, 10_000.0 * 0.99);
        assert_eq!(targets.take_profit_price, 10_000.0 * 1.02);
        assert!((targets.projected_pl_usd - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_custom_trade_size() {
        let targets = TradeTargetCalculator::calculate(1_000.0, Some(500.0), None).unwrap();
        assert_eq!(targets.stop_loss_price, 1_000.0 * 0.99);
        assert_eq!(targets.take_profit_price, 1_000.0 * 1.02);
        assert!((targets.projected_pl_usd - 10.0).abs() < f64::EPSILON); // 500 * 0.02
    }

    #[test]
    fn test_with_trading_settings() {
        let settings = TradingSettings {
            auto_trading_enabled: true,
            max_position_size: 200.0,
            risk_tolerance: 0.7,
            stop_loss_percentage: 2.0, // 2%
            take_profit_percentage: 3.0, // 3%
            preferred_exchanges: vec![],
            preferred_trading_pairs: vec![],
            min_profit_threshold: 0.5,
            max_leverage: 10,
            daily_loss_limit: 1000.0,
        };

        let targets = TradeTargetCalculator::calculate(1_000.0, Some(300.0), Some(&settings)).unwrap();
        assert_eq!(targets.stop_loss_price, 1_000.0 * 0.98); // 1 - 0.02
        assert_eq!(targets.take_profit_price, 1_000.0 * 1.03); // 1 + 0.03
        assert!((targets.projected_pl_usd - 9.0).abs() < f64::EPSILON); // 300 * 0.03
        assert_eq!(targets.projected_pl_percent, 3.0);
    }

    #[test]
    fn test_invalid_price() {
        let result = TradeTargetCalculator::calculate(0.0, None, None);
        assert!(result.is_err());
        
        let result = TradeTargetCalculator::calculate(-100.0, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_minimum_percentage_clamping() {
        let settings = TradingSettings {
            auto_trading_enabled: true,
            max_position_size: 200.0,
            risk_tolerance: 0.7,
            stop_loss_percentage: 0.05, // Very low, should be clamped to 0.1
            take_profit_percentage: 0.1, // Very low, should be clamped to 0.2
            preferred_exchanges: vec![],
            preferred_trading_pairs: vec![],
            min_profit_threshold: 0.5,
            max_leverage: 10,
            daily_loss_limit: 1000.0,
        };

        let targets = TradeTargetCalculator::calculate(1_000.0, None, Some(&settings)).unwrap();
        assert_eq!(targets.stop_loss_price, 1_000.0 * 0.999); // 1 - 0.001 (0.1% clamped)
        assert_eq!(targets.take_profit_price, 1_000.0 * 1.002); // 1 + 0.002 (0.2% clamped)
    }
}