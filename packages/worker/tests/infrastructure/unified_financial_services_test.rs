use crate::services::core::infrastructure::unified_financial_services::{
    UnifiedFinancialServicesConfig, BalanceTrackerConfig, FundAnalyzerConfig, FundAnalyzer
};
use std::collections::HashMap;

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