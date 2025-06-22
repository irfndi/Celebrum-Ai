use crate::services::core::ai::ai_intelligence::{
    AiIntelligenceConfig, AiOpportunityEnhancement, AiPerformanceInsights, AiPortfolioAnalysis,
    AiRiskAssessment, ParameterSuggestion, TradingFocus,
};
use crate::services::core::analysis::market_analysis::{
    OpportunityType, RiskLevel, TimeHorizon, TradingOpportunity,
};
use crate::types::*;
use crate::types::{PositionSide, PositionStatus};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AiIntelligenceConfig {
        AiIntelligenceConfig {
            enabled: true,
            ai_confidence_threshold: 0.6,
            max_ai_calls_per_hour: 100,
            cache_ttl_seconds: 1800,
            enable_performance_learning: true,
            enable_parameter_optimization: true,
            risk_assessment_frequency_hours: 6,
        }
    }

    #[allow(dead_code)]
    fn create_test_opportunity() -> TradingOpportunity {
        TradingOpportunity {
            opportunity_id: "test_opp_1".to_string(),
            opportunity_type: OpportunityType::Arbitrage,
            trading_pair: "BTCUSDT".to_string(),
            exchanges: vec!["binance".to_string(), "bybit".to_string()],
            entry_price: 50000.0,
            target_price: Some(51000.0),
            stop_loss: Some(49000.0),
            confidence_score: 0.8,
            risk_level: RiskLevel::Medium,
            expected_return: 0.02,
            time_horizon: TimeHorizon::Short,
            indicators_used: vec!["rsi".to_string(), "macd".to_string()],
            analysis_data: serde_json::json!({"signal": "bullish"}),
            created_at: chrono::Utc::now().timestamp() as u64,
            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 3600),
        }
    }

    #[test]
    fn test_ai_intelligence_config_creation() {
        let config = AiIntelligenceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.ai_confidence_threshold, 0.6);
        assert_eq!(config.max_ai_calls_per_hour, 100);
        assert_eq!(config.cache_ttl_seconds, 1800);
        assert!(config.enable_performance_learning);
        assert!(config.enable_parameter_optimization);
        assert_eq!(config.risk_assessment_frequency_hours, 6);
    }

    #[test]
    fn test_ai_opportunity_enhancement_structure() {
        let enhancement = AiOpportunityEnhancement {
            opportunity_id: "test_opp_1".to_string(),
            user_id: "user123".to_string(),
            ai_confidence_score: 0.85,
            ai_risk_assessment: AiRiskAssessment {
                overall_risk_score: 0.4,
                risk_factors: vec!["Market volatility".to_string()],
                portfolio_correlation_risk: 0.3,
                position_concentration_risk: 0.2,
                market_condition_risk: 0.4,
                volatility_risk: 0.5,
                liquidity_risk: 0.3,
                recommended_max_position: 1000.0,
            },
            ai_recommendations: vec!["Monitor closely".to_string()],
            position_sizing_suggestion: 500.0,
            timing_score: 0.8,
            technical_confirmation: 0.7,
            portfolio_impact_score: 0.6,
            ai_provider_used: "OpenAI".to_string(),
            analysis_timestamp: chrono::Utc::now().timestamp() as u64,
        };

        assert_eq!(enhancement.ai_confidence_score, 0.85);
        assert_eq!(enhancement.timing_score, 0.8);
        assert_eq!(enhancement.position_sizing_suggestion, 500.0);
        assert_eq!(enhancement.ai_risk_assessment.overall_risk_score, 0.4);
    }

    #[test]
    fn test_ai_risk_assessment_structure() {
        let risk_assessment = AiRiskAssessment {
            overall_risk_score: 0.6,
            risk_factors: vec!["Volatility".to_string(), "Liquidity".to_string()],
            portfolio_correlation_risk: 0.4,
            position_concentration_risk: 0.5,
            market_condition_risk: 0.3,
            volatility_risk: 0.7,
            liquidity_risk: 0.4,
            recommended_max_position: 2000.0,
        };

        assert_eq!(risk_assessment.overall_risk_score, 0.6);
        assert_eq!(risk_assessment.risk_factors.len(), 2);
        assert_eq!(risk_assessment.recommended_max_position, 2000.0);
    }

    #[test]
    fn test_ai_performance_insights_structure() {
        let insights = AiPerformanceInsights {
            user_id: "user123".to_string(),
            performance_score: 0.75,
            strengths: vec!["Good risk management".to_string()],
            weaknesses: vec!["Position sizing".to_string()],
            suggested_focus_adjustment: Some(TradingFocus::Arbitrage),
            parameter_optimization_suggestions: Vec::new(),
            learning_recommendations: vec!["Study technical analysis".to_string()],
            automation_readiness_score: 0.6,
            generated_at: chrono::Utc::now().timestamp() as u64,
        };

        assert_eq!(insights.performance_score, 0.75);
        assert_eq!(insights.automation_readiness_score, 0.6);
        assert_eq!(
            insights.suggested_focus_adjustment,
            Some(TradingFocus::Arbitrage)
        );
    }

    #[test]
    fn test_parameter_suggestion_structure() {
        let suggestion = ParameterSuggestion {
            parameter_name: "risk_tolerance".to_string(),
            current_value: "0.5".to_string(),
            suggested_value: "0.6".to_string(),
            rationale: "Based on performance, you can handle slightly higher risk".to_string(),
            impact_assessment: 0.7,
            confidence: 0.8,
        };

        assert_eq!(suggestion.parameter_name, "risk_tolerance");
        assert_eq!(suggestion.impact_assessment, 0.7);
        assert_eq!(suggestion.confidence, 0.8);
    }

    #[test]
    fn test_ai_portfolio_analysis_structure() {
        let analysis = AiPortfolioAnalysis {
            user_id: "user123".to_string(),
            correlation_risk_score: 0.4,
            concentration_risk_score: 0.6,
            diversification_score: 0.7,
            recommended_adjustments: vec!["Diversify more".to_string()],
            overexposure_warnings: vec!["High BTC exposure".to_string()],
            optimal_allocation_suggestions: HashMap::new(),
            analysis_timestamp: chrono::Utc::now().timestamp() as u64,
        };

        assert_eq!(analysis.correlation_risk_score, 0.4);
        assert_eq!(analysis.diversification_score, 0.7);
        assert_eq!(analysis.recommended_adjustments.len(), 1);
        assert_eq!(analysis.overexposure_warnings.len(), 1);
    }

    #[test]
    fn test_concentration_risk_calculation() {
        let positions = vec![
            create_test_position(1000.0),
            create_test_position(500.0),
            create_test_position(300.0),
        ];

        // Mock service for testing
        let config = create_test_config();
        let service = create_mock_service(config);

        let concentration_risk = service.calculate_concentration_risk(&positions);

        // Largest position (1000) / Total (1800) = 0.555...
        assert!((concentration_risk - 0.555).abs() < 0.01);
    }

    #[test]
    fn test_diversification_score_calculation() {
        let config = create_test_config();
        let service = create_mock_service(config);

        // Test with different numbers of positions
        assert_eq!(service.calculate_diversification_score(&[]), 0.2);
        assert_eq!(
            service.calculate_diversification_score(&[create_test_position(1000.0)]),
            0.2
        );

        let two_positions = vec![create_test_position(1000.0), create_test_position(500.0)];
        assert!((service.calculate_diversification_score(&two_positions) - 0.6).abs() < 0.0001);

        let five_positions = vec![
            create_test_position(1000.0),
            create_test_position(500.0),
            create_test_position(300.0),
            create_test_position(200.0),
            create_test_position(100.0),
        ];
        assert_eq!(
            service.calculate_diversification_score(&five_positions),
            0.8
        );
    }

    #[test]
    fn test_volatility_risk_calculation() {
        let config = create_test_config();
        let service = create_mock_service(config);

        let low_risk_opp = create_test_opportunity_with_risk(RiskLevel::Low);
        let medium_risk_opp = create_test_opportunity_with_risk(RiskLevel::Medium);
        let high_risk_opp = create_test_opportunity_with_risk(RiskLevel::High);

        assert_eq!(service.calculate_volatility_risk(&low_risk_opp), 0.2);
        assert_eq!(service.calculate_volatility_risk(&medium_risk_opp), 0.5);
        assert_eq!(service.calculate_volatility_risk(&high_risk_opp), 0.8);
    }

    #[test]
    fn test_automation_readiness_calculation() {
        let config = create_test_config();
        let service = create_mock_service(config);

        // High readiness: high win rate, many trades
        let high_readiness_data = PerformanceData {
            total_trades: 100,
            win_rate: 0.8,
            average_pnl: 50.0,
            _total_pnl: 5000.0,
        };
        assert_eq!(
            service.calculate_automation_readiness(&high_readiness_data),
            0.8
        );

        // Medium readiness: moderate win rate, some trades
        let medium_readiness_data = PerformanceData {
            total_trades: 30,
            win_rate: 0.65,
            average_pnl: 30.0,
            _total_pnl: 900.0,
        };
        assert_eq!(
            service.calculate_automation_readiness(&medium_readiness_data),
            0.6
        );

        // Low readiness: low win rate or few trades
        let low_readiness_data = PerformanceData {
            total_trades: 10,
            win_rate: 0.5,
            average_pnl: 20.0,
            _total_pnl: 200.0,
        };
        assert_eq!(
            service.calculate_automation_readiness(&low_readiness_data),
            0.3
        );
    }

    // Helper functions for testing
    fn create_test_position(value: f64) -> ArbitragePosition {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        ArbitragePosition {
            id: format!("pos_{}", value as u32),
            user_id: "test_user".to_string(),
            opportunity_id: "test_opp".to_string(),
            long_position: Position {
                info: serde_json::Value::Null,
                id: Some("long_pos_1".to_string()),
                symbol: "BTCUSDT".to_string(),
                timestamp: now,
                datetime: chrono::Utc::now().to_rfc3339(),
                isolated: Some(true),
                hedged: Some(false),
                side: "long".to_string(),
                amount: 0.1,
                contracts: Some(0.1),
                contract_size: Some(1.0),
                entry_price: Some(50000.0),
                mark_price: Some(50000.0),
                notional: Some(5000.0),
                leverage: Some(1.0),
                collateral: Some(5000.0),
                initial_margin: Some(5000.0),
                initial_margin_percentage: Some(1.0),
                maintenance_margin: Some(2500.0),
                maintenance_margin_percentage: Some(0.5),
                unrealized_pnl: Some(0.0),
                realized_pnl: Some(0.0),
                percentage: Some(0.0),
            },
            short_position: Position {
                info: serde_json::Value::Null,
                id: Some("short_pos_1".to_string()),
                symbol: "BTCUSDT".to_string(),
                timestamp: now,
                datetime: chrono::Utc::now().to_rfc3339(),
                isolated: Some(true),
                hedged: Some(false),
                side: "short".to_string(),
                amount: 0.1,
                contracts: Some(0.1),
                contract_size: Some(1.0),
                entry_price: Some(50100.0),
                mark_price: Some(50100.0),
                notional: Some(5010.0),
                leverage: Some(1.0),
                collateral: Some(5010.0),
                initial_margin: Some(5010.0),
                initial_margin_percentage: Some(1.0),
                maintenance_margin: Some(2505.0),
                maintenance_margin_percentage: Some(0.5),
                unrealized_pnl: Some(0.0),
                realized_pnl: Some(0.0),
                percentage: Some(0.0),
            },
            status: PositionStatus::Open,
            entry_time: now,
            exit_time: None,
            realized_pnl: 0.0,
            unrealized_pnl: 5.0,
            total_fees: 0.0,
            risk_score: 0.5,
            margin_used: 5000.0,
            symbol: "BTCUSDT".to_string(),
            side: PositionSide::Long,
            entry_price_long: 50000.0,
            entry_price_short: 50100.0,
            take_profit_price: Some(51000.0),
            volatility_score: Some(0.5),
            calculated_size_usd: Some(value), // Use the passed value parameter
            long_exchange: ExchangeIdEnum::Binance,
            short_exchange: ExchangeIdEnum::Bybit,
            size: Some(0.1),
            pnl: Some(5.0),
            unrealized_pnl_percentage: Some(0.01), // 5 / 50000 * 0.1 (assuming size is in BTC)
            max_drawdown: Some(0.0),
            created_at: now,
            holding_period_hours: Some(0.0),
            trailing_stop_distance: None,
            stop_loss_price: Some(49000.0),
            current_price: Some(50050.0),
            current_price_long: Some(50050.0),
            current_price_short: Some(50050.0),
            max_loss_usd: Some(100.0),
            exchange: ExchangeIdEnum::Binance, // Assuming primary exchange for the overall position
            pair: "BTC/USDT".to_string(),
            related_positions: Vec::new(),
            closed_at: None,
            updated_at: now,
            risk_reward_ratio: Some(2.0),
            last_optimization_check: None,
            hedge_position_id: None,
            position_group_id: None,
            current_state: Some("monitoring".to_string()),
            optimization_score: Some(0.0),
            recommended_action: Some("hold".to_string()),
            risk_percentage_applied: Some(0.01),
        }
    }

    fn create_test_opportunity_with_risk(risk_level: RiskLevel) -> TradingOpportunity {
        TradingOpportunity {
            opportunity_id: "test_opp_1".to_string(),
            opportunity_type: OpportunityType::Arbitrage,
            trading_pair: "BTCUSDT".to_string(),
            exchanges: vec!["binance".to_string(), "bybit".to_string()],
            entry_price: 50000.0,
            target_price: Some(51000.0),
            stop_loss: Some(49000.0),
            confidence_score: 0.8,
            risk_level,
            expected_return: 0.02,
            time_horizon: TimeHorizon::Short,
            indicators_used: vec!["rsi".to_string(), "macd".to_string()],
            analysis_data: serde_json::json!({"signal": "bullish"}),
            created_at: chrono::Utc::now().timestamp() as u64,
            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 3600),
        }
    }

    fn create_mock_service(config: AiIntelligenceConfig) -> MockAiIntelligenceService {
        MockAiIntelligenceService { config }
    }

    // Mock service for testing business logic
    #[allow(dead_code)]
    struct MockAiIntelligenceService {
        config: AiIntelligenceConfig,
    }

    impl MockAiIntelligenceService {
        fn calculate_concentration_risk(&self, positions: &[ArbitragePosition]) -> f64 {
            if positions.is_empty() {
                0.0
            } else {
                let total_value: f64 = positions.iter().filter_map(|p| p.calculated_size_usd).sum();
                let max_position = positions
                    .iter()
                    .filter_map(|p| p.calculated_size_usd)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                if total_value > 0.0 {
                    max_position / total_value
                } else {
                    0.0
                }
            }
        }

        fn calculate_diversification_score(&self, positions: &[ArbitragePosition]) -> f64 {
            if positions.len() <= 1 {
                0.2
            } else if positions.len() >= 5 {
                0.8
            } else {
                0.4 + (positions.len() as f64 * 0.1)
            }
        }

        fn calculate_volatility_risk(&self, opportunity: &TradingOpportunity) -> f64 {
            match opportunity.risk_level {
                RiskLevel::Low => 0.2,
                RiskLevel::Medium => 0.5,
                RiskLevel::High => 0.8,
            }
        }

        fn calculate_automation_readiness(&self, performance_data: &PerformanceData) -> f64 {
            if performance_data.win_rate > 0.7 && performance_data.total_trades > 50 {
                0.8
            } else if performance_data.win_rate > 0.6 && performance_data.total_trades > 20 {
                0.6
            } else {
                0.3
            }
        }
    }

    // Helper struct for testing
    #[allow(dead_code)]
    struct PerformanceData {
        total_trades: u32,
        win_rate: f64,
        average_pnl: f64,
        _total_pnl: f64,
    }
}
