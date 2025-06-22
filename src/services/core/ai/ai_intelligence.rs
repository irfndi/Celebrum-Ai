use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============= CONFIGURATION =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIntelligenceConfig {
    pub enabled: bool,
    pub ai_confidence_threshold: f64,
    pub max_ai_calls_per_hour: u32,
    pub cache_ttl_seconds: u64,
    pub enable_performance_learning: bool,
    pub enable_parameter_optimization: bool,
    pub risk_assessment_frequency_hours: u32,
}

impl Default for AiIntelligenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ai_confidence_threshold: 0.6,
            max_ai_calls_per_hour: 100,
            cache_ttl_seconds: 1800, // 30 minutes
            enable_performance_learning: true,
            enable_parameter_optimization: true,
            risk_assessment_frequency_hours: 6,
        }
    }
}

// ============= CORE DATA STRUCTURES =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOpportunityEnhancement {
    pub opportunity_id: String,
    pub user_id: String,
    pub ai_confidence_score: f64,
    pub ai_risk_assessment: AiRiskAssessment,
    pub ai_recommendations: Vec<String>,
    pub position_sizing_suggestion: f64,
    pub timing_score: f64,
    pub technical_confirmation: f64,
    pub portfolio_impact_score: f64,
    pub ai_provider_used: String,
    pub analysis_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRiskAssessment {
    pub overall_risk_score: f64,
    pub risk_factors: Vec<String>,
    pub portfolio_correlation_risk: f64,
    pub position_concentration_risk: f64,
    pub market_condition_risk: f64,
    pub volatility_risk: f64,
    pub liquidity_risk: f64,
    pub recommended_max_position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPerformanceInsights {
    pub user_id: String,
    pub performance_score: f64,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub suggested_focus_adjustment: Option<TradingFocus>,
    pub parameter_optimization_suggestions: Vec<ParameterSuggestion>,
    pub learning_recommendations: Vec<String>,
    pub automation_readiness_score: f64,
    pub generated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingFocus {
    Arbitrage,
    Momentum,
    MeanReversion,
    Scalping,
    Swing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSuggestion {
    pub parameter_name: String,
    pub current_value: String,
    pub suggested_value: String,
    pub rationale: String,
    pub impact_assessment: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPortfolioAnalysis {
    pub user_id: String,
    pub correlation_risk_score: f64,
    pub concentration_risk_score: f64,
    pub diversification_score: f64,
    pub recommended_adjustments: Vec<String>,
    pub overexposure_warnings: Vec<String>,
    pub optimal_allocation_suggestions: HashMap<String, f64>,
    pub analysis_timestamp: u64,
}

// ============= HELPER DATA STRUCTURES =============

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PerformanceData {
    total_trades: u32,
    win_rate: f64,
    average_pnl: f64,
    _total_pnl: f64,
}

// ============= SERVICE IMPLEMENTATION =============

#[derive(Debug, Clone)]
pub struct AiIntelligenceService {
    pub config: AiIntelligenceConfig,
}

impl AiIntelligenceService {
    pub fn new(config: AiIntelligenceConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self {
            config: AiIntelligenceConfig::default(),
        }
    }

    pub fn calculate_concentration_risk(
        &self,
        positions: &[crate::types::ArbitragePosition],
    ) -> f64 {
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

    pub fn calculate_diversification_score(
        &self,
        positions: &[crate::types::ArbitragePosition],
    ) -> f64 {
        if positions.len() <= 1 {
            0.2
        } else if positions.len() >= 5 {
            0.8
        } else {
            0.4 + (positions.len() as f64 * 0.1)
        }
    }

    pub fn calculate_automation_readiness(&self, performance_data: &PerformanceData) -> f64 {
        if performance_data.win_rate > 0.7 && performance_data.total_trades > 50 {
            0.8
        } else if performance_data.win_rate > 0.6 && performance_data.total_trades > 20 {
            0.6
        } else {
            0.3
        }
    }
}

// Tests have been moved to packages/worker/tests/ai/ai_intelligence_test.rs
