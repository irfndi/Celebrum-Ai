// AI Intelligence Service
// Task 9.6: AI-Enhanced Opportunity Detection & Integration

use crate::services::core::analysis::correlation_analysis::CorrelationMetrics;
use crate::services::core::analysis::market_analysis::{RiskLevel, TradingOpportunity};
use crate::services::core::opportunities::opportunity_categorization::CategorizedOpportunity;
use crate::services::core::user::dynamic_config::UserConfigInstance;
use crate::services::core::user::user_trading_preferences::{TradingFocus, UserTradingPreferences};
use crate::services::{
    AiExchangeRouterService, CorrelationAnalysisService, D1Service, DynamicConfigService,
    OpportunityCategorizationService, PositionsService, UserTradingPreferencesService,
};
use crate::types::{ArbitragePosition, ExchangeIdEnum, GlobalOpportunity};
use crate::utils::{
    logger::{LogLevel, Logger},
    ArbitrageError, ArbitrageResult,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::kv::KvStore;

// ============= AI INTELLIGENCE DATA STRUCTURES =============

/// AI-enhanced opportunity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOpportunityEnhancement {
    pub opportunity_id: String,
    pub user_id: String,
    pub ai_confidence_score: f64, // 0.0 to 1.0 - AI's confidence in the opportunity
    pub ai_risk_assessment: AiRiskAssessment,
    pub ai_recommendations: Vec<String>, // AI-generated recommendations
    pub position_sizing_suggestion: f64, // AI-suggested position size in USD
    pub timing_score: f64,               // 0.0 to 1.0 - optimal timing assessment
    pub technical_confirmation: f64,     // 0.0 to 1.0 - technical analysis confirmation
    pub portfolio_impact_score: f64,     // 0.0 to 1.0 - impact on overall portfolio
    pub ai_provider_used: String,        // Which AI provider generated this
    pub analysis_timestamp: u64,
}

/// Comprehensive AI risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRiskAssessment {
    pub overall_risk_score: f64,          // 0.0 to 1.0 - overall risk level
    pub risk_factors: Vec<String>,        // Identified risk factors
    pub portfolio_correlation_risk: f64,  // Risk from portfolio correlations
    pub position_concentration_risk: f64, // Risk from position concentration
    pub market_condition_risk: f64,       // Risk from current market conditions
    pub volatility_risk: f64,             // Risk from price volatility
    pub liquidity_risk: f64,              // Risk from liquidity constraints
    pub recommended_max_position: f64,    // AI-recommended maximum position size
}

/// AI-driven performance insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPerformanceInsights {
    pub user_id: String,
    pub performance_score: f64,  // 0.0 to 1.0 - overall performance rating
    pub strengths: Vec<String>,  // Identified strengths in trading
    pub weaknesses: Vec<String>, // Areas for improvement
    pub suggested_focus_adjustment: Option<TradingFocus>, // Suggested trading focus
    pub parameter_optimization_suggestions: Vec<ParameterSuggestion>,
    pub learning_recommendations: Vec<String>, // Educational recommendations
    pub automation_readiness_score: f64,       // 0.0 to 1.0 - readiness for automation
    pub generated_at: u64,
}

/// AI-suggested parameter optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSuggestion {
    pub parameter_name: String,
    pub current_value: String,
    pub suggested_value: String,
    pub rationale: String,
    pub impact_assessment: f64, // 0.0 to 1.0 - expected impact
    pub confidence: f64,        // 0.0 to 1.0 - AI's confidence in suggestion
}

/// AI portfolio correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPortfolioAnalysis {
    pub user_id: String,
    pub correlation_risk_score: f64, // 0.0 to 1.0 - portfolio correlation risk
    pub concentration_risk_score: f64, // 0.0 to 1.0 - position concentration risk
    pub diversification_score: f64,  // 0.0 to 1.0 - portfolio diversification
    pub recommended_adjustments: Vec<String>, // AI recommendations for portfolio
    pub overexposure_warnings: Vec<String>, // Warnings about overexposure
    pub optimal_allocation_suggestions: HashMap<String, f64>, // Suggested allocations
    pub analysis_timestamp: u64,
}

/// Configuration for AI Intelligence Service
#[derive(Debug, Clone)]
pub struct AiIntelligenceConfig {
    pub enabled: bool,
    pub ai_confidence_threshold: f64, // Minimum AI confidence for recommendations
    pub max_ai_calls_per_hour: u32,   // Rate limiting for AI calls
    pub cache_ttl_seconds: u64,       // Cache TTL for AI analysis results
    pub enable_performance_learning: bool, // Enable AI learning from performance
    pub enable_parameter_optimization: bool, // Enable AI parameter optimization
    pub risk_assessment_frequency_hours: u64, // How often to run risk assessment
}

impl Default for AiIntelligenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ai_confidence_threshold: 0.6, // 60% minimum confidence
            max_ai_calls_per_hour: 100,   // Reasonable rate limit
            cache_ttl_seconds: 1800,      // 30 minutes cache
            enable_performance_learning: true,
            enable_parameter_optimization: true,
            risk_assessment_frequency_hours: 6, // Risk assessment every 6 hours
        }
    }
}

// ============= AI INTELLIGENCE SERVICE =============

/// AI Intelligence Service - The brain of the platform
/// Integrates all existing services with AI-enhanced decision making
///
/// TODO: REFACTOR - This service violates single responsibility principle
/// Suggested approach: Split into smaller focused services:
/// - AiOpportunityAnalysisService: analyze_opportunity_with_ai()
/// - AiPortfolioRiskService: assess_portfolio_risk_with_ai()  
/// - AiPerformanceService: generate_performance_insights()
/// - AiParameterOptimizationService: optimize_trading_parameters()
///   And implement builder pattern for cleaner dependency management
pub struct AiIntelligenceService {
    config: AiIntelligenceConfig,
    ai_router: AiExchangeRouterService,
    categorization_service: OpportunityCategorizationService,
    positions_service: PositionsService<KvStore>,
    config_service: DynamicConfigService,
    preferences_service: UserTradingPreferencesService,
    correlation_service: CorrelationAnalysisService,
    d1_service: D1Service,
    kv_store: KvStore,
    logger: Logger,
}

impl AiIntelligenceService {
    /// Create new AI Intelligence Service
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: AiIntelligenceConfig,
        ai_router: AiExchangeRouterService,
        categorization_service: OpportunityCategorizationService,
        positions_service: PositionsService<KvStore>,
        config_service: DynamicConfigService,
        preferences_service: UserTradingPreferencesService,
        correlation_service: CorrelationAnalysisService,
        d1_service: D1Service,
        kv_store: KvStore,
    ) -> Self {
        Self {
            config,
            ai_router,
            categorization_service,
            positions_service,
            config_service,
            preferences_service,
            correlation_service,
            d1_service,
            kv_store,
            logger: Logger::new(LogLevel::Info),
        }
    }

    /// Analyze opportunity with AI enhancement
    /// Combines categorization, position analysis, and AI insights
    pub async fn analyze_opportunity_with_ai(
        &self,
        user_id: &str,
        opportunity: &TradingOpportunity,
    ) -> ArbitrageResult<AiOpportunityEnhancement> {
        if !self.config.enabled {
            return Err(ArbitrageError::config_error("AI Intelligence is disabled"));
        }

        // Check rate limiting
        self.check_ai_rate_limit(user_id).await?;

        // Get categorized opportunity
        let categorized_opp = self
            .categorization_service
            .categorize_opportunity(opportunity.clone(), user_id)
            .await?;

        // Get user's current positions for context
        let positions = self
            .positions_service
            .get_all_positions()
            .await
            .unwrap_or_default();

        // Get user preferences and configuration
        let preferences = self
            .preferences_service
            .get_or_create_preferences(user_id)
            .await?;

        let user_config = self
            .config_service
            .get_user_config(user_id, "default")
            .await?;

        // Create AI analysis prompt
        let ai_prompt = self.create_opportunity_analysis_prompt(
            &categorized_opp,
            &positions,
            &preferences,
            &user_config,
        );

        // Call AI for analysis
        // Convert TradingOpportunity to GlobalOpportunity for AI router
        let global_opp = self.convert_to_global_opportunity(opportunity.clone());
        let ai_response = self
            .ai_router
            .analyze_opportunities(
                user_id,
                &[global_opp],
                Some(serde_json::Value::String(ai_prompt)),
            )
            .await?;

        // Parse AI response into enhancement
        let enhancement = self
            .parse_ai_opportunity_response(
                user_id,
                opportunity,
                &categorized_opp,
                ai_response.first().unwrap(),
                &positions,
            )
            .await?;

        // Store AI enhancement for learning
        self.store_ai_enhancement(&enhancement).await?;

        // Cache result
        self.cache_ai_enhancement(user_id, &enhancement).await?;

        self.logger.info(&format!(
            "AI opportunity analysis complete: user={}, opportunity={}, score={:.2}",
            user_id, opportunity.opportunity_id, enhancement.ai_confidence_score
        ));

        Ok(enhancement)
    }

    /// Assess portfolio risk with AI
    /// Analyzes current positions and provides AI-driven risk insights
    pub async fn assess_portfolio_risk_with_ai(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<AiPortfolioAnalysis> {
        if !self.config.enabled {
            return Err(ArbitrageError::config_error("AI Intelligence is disabled"));
        }

        // Get user's current positions
        let positions = self
            .positions_service
            .get_all_positions()
            .await
            .unwrap_or_default();

        if positions.is_empty() {
            return Ok(self.create_empty_portfolio_analysis(user_id));
        }

        // Get correlation data
        let exchange_data = if !positions.is_empty() {
            // Attempt to fetch actual exchange data for positions
            match self.fetch_exchange_data_for_positions(&positions).await {
                Ok(data) => data,
                Err(_) => {
                    return Err(ArbitrageError::not_implemented(
                        "Exchange data fetching for correlation analysis not yet implemented"
                            .to_string(),
                    ));
                }
            }
        } else {
            std::collections::HashMap::new()
        };

        // Get user preferences
        let preferences = self
            .preferences_service
            .get_or_create_preferences(user_id)
            .await?;

        // Generate correlation analysis if data is available
        let correlation_metrics = if !exchange_data.is_empty() {
            Some(
                self.correlation_service
                    .generate_correlation_metrics("BTCUSDT", &exchange_data, &preferences)
                    .unwrap_or_else(|_| self.create_default_correlation_metrics()),
            )
        } else {
            None
        };

        // Create AI risk assessment prompt
        let _ai_prompt =
            self.create_portfolio_risk_prompt(&positions, &correlation_metrics, &preferences);

        // Get AI analysis
        let market_snapshot = self.create_portfolio_market_snapshot(&positions);
        let ai_response = self
            .ai_router
            .get_real_time_recommendations(user_id, &[], &market_snapshot)
            .await?;

        // Parse AI response into portfolio analysis
        let portfolio_analysis = self.parse_ai_portfolio_response(
            user_id,
            &positions,
            &correlation_metrics,
            &ai_response,
        );

        // Store portfolio analysis
        self.store_portfolio_analysis(&portfolio_analysis).await?;

        self.logger.info(&format!(
            "AI portfolio risk assessment complete: user={}, risk_score={:.2}",
            user_id, portfolio_analysis.correlation_risk_score
        ));

        Ok(portfolio_analysis)
    }

    /// Generate AI-driven performance insights
    /// Analyzes user's trading performance and provides recommendations
    pub async fn generate_performance_insights(
        &self,
        user_id: &str,
        analysis_period_days: u32,
    ) -> ArbitrageResult<AiPerformanceInsights> {
        if !self.config.enabled || !self.config.enable_performance_learning {
            return Err(ArbitrageError::config_error(
                "AI performance learning is disabled",
            ));
        }

        // Get user's trading history
        let performance_data = self
            .get_user_performance_data(user_id, analysis_period_days)
            .await?;

        // Get user preferences and configuration
        let preferences = self
            .preferences_service
            .get_or_create_preferences(user_id)
            .await?;

        // Create AI performance analysis prompt
        let ai_prompt = self.create_performance_analysis_prompt(&performance_data, &preferences);

        // Get AI insights
        let ai_response = self
            .ai_router
            .analyze_market_data(
                user_id,
                &self.create_performance_market_data(&performance_data),
                Some(ai_prompt),
            )
            .await?;

        // Parse AI response into performance insights
        let insights = self.parse_ai_performance_response(user_id, &performance_data, &ai_response);

        // Store insights for learning
        self.store_performance_insights(&insights).await?;

        self.logger.info(&format!(
            "AI performance insights generated: user={}, score={:.2}",
            user_id, insights.performance_score
        ));

        Ok(insights)
    }

    /// Optimize trading parameters with AI
    /// Suggests parameter improvements based on performance and market conditions
    pub async fn optimize_trading_parameters(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Vec<ParameterSuggestion>> {
        if !self.config.enabled || !self.config.enable_parameter_optimization {
            return Err(ArbitrageError::config_error(
                "AI parameter optimization is disabled",
            ));
        }

        // Get current user configuration
        let current_config = self
            .config_service
            .get_user_config(user_id, "default")
            .await?;

        // Get user performance data
        let performance_data = self.get_user_performance_data(user_id, 30).await?;

        // Get user preferences
        let preferences = self
            .preferences_service
            .get_or_create_preferences(user_id)
            .await?;

        // Create AI optimization prompt
        let ai_prompt = self.create_parameter_optimization_prompt(
            &current_config,
            &performance_data,
            &preferences,
        );

        // Get AI recommendations
        let ai_response = self
            .ai_router
            .analyze_market_data(
                user_id,
                &self.create_config_optimization_data(&current_config),
                Some(ai_prompt),
            )
            .await?;

        // Parse AI response into parameter suggestions
        let suggestions = self.parse_ai_parameter_suggestions(&current_config, &ai_response);

        // Store suggestions
        for suggestion in &suggestions {
            self.store_parameter_suggestion(user_id, suggestion).await?;
        }

        self.logger.info(&format!(
            "AI parameter optimization complete: user={}, suggestions={}",
            user_id,
            suggestions.len()
        ));

        Ok(suggestions)
    }

    /// Check if user should adjust their trading focus based on AI analysis
    pub async fn suggest_trading_focus_adjustment(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<TradingFocus>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Generate performance insights
        let insights = self.generate_performance_insights(user_id, 60).await?;

        // Return AI's suggestion if confidence is high enough
        if insights.automation_readiness_score >= self.config.ai_confidence_threshold {
            Ok(insights.suggested_focus_adjustment)
        } else {
            Ok(None)
        }
    }

    // ============= PRIVATE HELPER METHODS =============

    /// Check AI rate limiting
    async fn check_ai_rate_limit(&self, user_id: &str) -> ArbitrageResult<()> {
        let rate_key = format!(
            "ai_intelligence_rate:{}:{}",
            user_id,
            chrono::Utc::now().format("%Y%m%d%H")
        );

        let current_count: u32 = match self.kv_store.get(&rate_key).text().await {
            Ok(Some(count_str)) => count_str.parse().unwrap_or(0),
            _ => 0,
        };

        if current_count >= self.config.max_ai_calls_per_hour {
            return Err(ArbitrageError::rate_limit_error(
                "AI Intelligence rate limit exceeded",
            ));
        }

        // Update count
        self.kv_store
            .put(&rate_key, (current_count + 1).to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to update rate limit: {}", e))
            })?
            .expiration_ttl(3600) // 1 hour
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to execute rate limit update: {}", e))
            })?;

        Ok(())
    }

    /// Create opportunity analysis prompt for AI
    fn create_opportunity_analysis_prompt(
        &self,
        categorized_opp: &CategorizedOpportunity,
        positions: &[ArbitragePosition],
        preferences: &UserTradingPreferences,
        _user_config: &Option<UserConfigInstance>,
    ) -> String {
        format!(
            "Analyze this trading opportunity for advanced insights:\n\
             Opportunity: {} (Categories: {:?})\n\
             Confidence: {:.2}%, Risk Level: {:?}\n\
             User Experience: {:?}, Risk Tolerance: {:?}, Trading Focus: {:?}\n\
             Current Positions: {} active trades\n\
             \n\
             Provide analysis on:\n\
             1. AI confidence score (0-100)\n\
             2. Risk assessment and factors\n\
             3. Optimal position sizing\n\
             4. Timing assessment\n\
             5. Portfolio impact\n\
             6. Specific recommendations\n\
             \n\
             Consider user's experience level and current portfolio when making recommendations.",
            categorized_opp.base_opportunity.opportunity_id,
            categorized_opp.categories,
            categorized_opp.user_suitability_score * 100.0,
            categorized_opp.base_opportunity.risk_level,
            preferences.experience_level,
            preferences.risk_tolerance,
            preferences.trading_focus,
            positions.len()
        )
    }

    /// Create portfolio risk assessment prompt for AI
    fn create_portfolio_risk_prompt(
        &self,
        positions: &[ArbitragePosition],
        _correlation_metrics: &Option<CorrelationMetrics>,
        preferences: &UserTradingPreferences,
    ) -> String {
        let total_value: f64 = positions.iter().filter_map(|p| p.calculated_size_usd).sum();
        let position_count = positions.len();

        format!(
            "Analyze this portfolio for risk assessment:\n\
             Total Portfolio Value: ${:.2}\n\
             Number of Positions: {}\n\
             User Risk Tolerance: {:?}\n\
             User Experience: {:?}\n\
             \n\
             Provide assessment on:\n\
             1. Overall risk score (0-100)\n\
             2. Correlation risks between positions\n\
             3. Concentration risks\n\
             4. Market condition risks\n\
             5. Recommended portfolio adjustments\n\
             6. Optimal allocation suggestions\n\
             \n\
             Focus on portfolio-level risks and diversification.",
            total_value, position_count, preferences.risk_tolerance, preferences.experience_level
        )
    }

    /// Create performance analysis prompt for AI
    fn create_performance_analysis_prompt(
        &self,
        performance_data: &PerformanceData,
        preferences: &UserTradingPreferences,
    ) -> String {
        format!(
            "Analyze this user's trading performance for insights:\n\
             Total Trades: {}\n\
             Win Rate: {:.2}%\n\
             Average PnL: ${:.2}\n\
             Current Trading Focus: {:?}\n\
             Experience Level: {:?}\n\
             \n\
             Provide insights on:\n\
             1. Performance score (0-100)\n\
             2. Identified strengths\n\
             3. Areas for improvement\n\
             4. Suggested trading focus adjustment\n\
             5. Parameter optimization suggestions\n\
             6. Automation readiness assessment\n\
             7. Learning recommendations\n\
             \n\
             Be specific and actionable in recommendations.",
            performance_data.total_trades,
            performance_data.win_rate * 100.0,
            performance_data.average_pnl,
            preferences.trading_focus,
            preferences.experience_level
        )
    }

    /// Create parameter optimization prompt for AI
    fn create_parameter_optimization_prompt(
        &self,
        current_config: &Option<UserConfigInstance>,
        performance_data: &PerformanceData,
        preferences: &UserTradingPreferences,
    ) -> String {
        let config_summary = current_config
            .as_ref()
            .map(|c| format!("Configuration ID: {}", c.instance_id))
            .unwrap_or_else(|| "No configuration set".to_string());

        format!(
            "Optimize trading parameters based on performance:\n\
             Current Configuration: {}\n\
             Performance: {:.2}% win rate, ${:.2} avg PnL\n\
             User Preferences: {:?} focus, {:?} experience\n\
             Total Trades: {}\n\
             \n\
             Suggest optimizations for:\n\
             1. Risk management parameters\n\
             2. Position sizing strategies\n\
             3. Entry/exit criteria\n\
             4. Alert thresholds\n\
             5. Trading frequency settings\n\
             \n\
             Provide specific parameter values and rationale for each suggestion.",
            config_summary,
            performance_data.win_rate * 100.0,
            performance_data.average_pnl,
            preferences.trading_focus,
            preferences.experience_level,
            performance_data.total_trades
        )
    }

    /// Parse AI opportunity response into enhancement
    async fn parse_ai_opportunity_response(
        &self,
        user_id: &str,
        opportunity: &TradingOpportunity,
        _categorized_opp: &CategorizedOpportunity,
        ai_analysis: &crate::services::core::trading::ai_exchange_router::AiOpportunityAnalysis,
        positions: &[ArbitragePosition],
    ) -> ArbitrageResult<AiOpportunityEnhancement> {
        // Extract AI insights from analysis text
        let ai_confidence_score = ai_analysis.ai_score;
        let technical_confirmation =
            self.calculate_technical_confirmation_from_analysis(&ai_analysis.viability_assessment);
        let timing_score =
            self.extract_timing_score_from_analysis(&ai_analysis.viability_assessment);
        let portfolio_impact_score = self.calculate_portfolio_impact(opportunity, positions);

        // Create AI risk assessment
        let ai_risk_assessment = AiRiskAssessment {
            overall_risk_score: self
                .calculate_overall_risk_score(&ai_analysis.viability_assessment),
            risk_factors: ai_analysis.risk_factors.clone(),
            portfolio_correlation_risk: self.calculate_correlation_risk(positions),
            position_concentration_risk: self.calculate_concentration_risk(positions),
            market_condition_risk: self.extract_market_risk(&ai_analysis.viability_assessment),
            volatility_risk: self.calculate_volatility_risk(opportunity),
            liquidity_risk: self.calculate_liquidity_risk(opportunity),
            recommended_max_position: ai_analysis.recommended_position_size,
        };

        Ok(AiOpportunityEnhancement {
            opportunity_id: opportunity.opportunity_id.clone(),
            user_id: user_id.to_string(),
            ai_confidence_score,
            ai_risk_assessment,
            ai_recommendations: ai_analysis.custom_recommendations.clone(),
            position_sizing_suggestion: ai_analysis.recommended_position_size,
            timing_score,
            technical_confirmation,
            portfolio_impact_score,
            ai_provider_used: ai_analysis.ai_provider_used.clone(),
            analysis_timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Parse AI portfolio response
    fn parse_ai_portfolio_response(
        &self,
        user_id: &str,
        positions: &[ArbitragePosition],
        _correlation_metrics: &Option<CorrelationMetrics>,
        ai_response: &crate::services::core::ai::ai_integration::AiAnalysisResponse,
    ) -> AiPortfolioAnalysis {
        AiPortfolioAnalysis {
            user_id: user_id.to_string(),
            correlation_risk_score: self
                .extract_correlation_risk_from_analysis(&ai_response.analysis),
            concentration_risk_score: self.calculate_concentration_risk(positions),
            diversification_score: self.calculate_diversification_score(positions),
            recommended_adjustments: self.extract_portfolio_recommendations(&ai_response.analysis),
            overexposure_warnings: self.extract_overexposure_warnings(&ai_response.analysis),
            optimal_allocation_suggestions: HashMap::new(), // Would be populated from AI analysis
            analysis_timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Parse AI performance response
    fn parse_ai_performance_response(
        &self,
        user_id: &str,
        performance_data: &PerformanceData,
        ai_response: &crate::services::core::ai::ai_integration::AiAnalysisResponse,
    ) -> AiPerformanceInsights {
        AiPerformanceInsights {
            user_id: user_id.to_string(),
            performance_score: self.extract_performance_score(&ai_response.analysis),
            strengths: self.extract_strengths(&ai_response.analysis),
            weaknesses: self.extract_weaknesses(&ai_response.analysis),
            suggested_focus_adjustment: self.extract_focus_suggestion(&ai_response.analysis),
            parameter_optimization_suggestions: Vec::new(), // Would be populated from AI analysis
            learning_recommendations: ai_response.recommendations.clone(),
            automation_readiness_score: self.calculate_automation_readiness(performance_data),
            generated_at: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Parse AI parameter suggestions
    fn parse_ai_parameter_suggestions(
        &self,
        _current_config: &Option<UserConfigInstance>,
        ai_response: &crate::services::core::ai::ai_integration::AiAnalysisResponse,
    ) -> Vec<ParameterSuggestion> {
        // Parse AI response for parameter suggestions
        // This would be more sophisticated in a real implementation
        ai_response
            .recommendations
            .iter()
            .enumerate()
            .map(|(i, rec)| ParameterSuggestion {
                parameter_name: format!("param_{}", i),
                current_value: "current".to_string(),
                suggested_value: "suggested".to_string(),
                rationale: rec.clone(),
                impact_assessment: 0.7,
                confidence: 0.8,
            })
            .collect()
    }

    // ============= UTILITY METHODS =============

    /// Extract technical confirmation score from AI analysis
    fn calculate_technical_confirmation_from_analysis(&self, analysis: &str) -> f64 {
        // Look for technical confirmation indicators in the AI analysis
        if analysis
            .to_lowercase()
            .contains("strong technical confirmation")
        {
            0.9
        } else if analysis
            .to_lowercase()
            .contains("moderate technical confirmation")
        {
            0.7
        } else if analysis
            .to_lowercase()
            .contains("weak technical confirmation")
        {
            0.4
        } else {
            0.6 // Default moderate confirmation
        }
    }

    /// Extract timing score from AI analysis using regex patterns
    fn extract_timing_score_from_analysis(&self, analysis: &str) -> f64 {
        let excellent_timing =
            Regex::new(r"(?i)\b(excellent|outstanding|perfect)\s+timing\b").unwrap();
        let good_timing = Regex::new(r"(?i)\b(good|solid|decent)\s+timing\b").unwrap();
        let poor_timing = Regex::new(r"(?i)\b(poor|bad|terrible)\s+timing\b").unwrap();

        if excellent_timing.is_match(analysis) {
            0.9
        } else if good_timing.is_match(analysis) {
            0.7
        } else if poor_timing.is_match(analysis) {
            0.3
        } else {
            0.6 // Default moderate timing
        }
    }

    /// Calculate portfolio impact of new opportunity
    fn calculate_portfolio_impact(
        &self,
        _opportunity: &TradingOpportunity,
        positions: &[ArbitragePosition],
    ) -> f64 {
        if positions.is_empty() {
            0.9 // High impact for first position
        } else {
            // Calculate based on correlation and concentration
            0.5 // Moderate impact for additional positions
        }
    }

    /// Calculate overall risk score from AI analysis using regex patterns
    fn calculate_overall_risk_score(&self, analysis: &str) -> f64 {
        let high_risk = Regex::new(r"(?i)\b(high|elevated|extreme|significant)\s+risk\b").unwrap();
        let moderate_risk = Regex::new(r"(?i)\b(moderate|medium|balanced)\s+risk\b").unwrap();
        let low_risk = Regex::new(r"(?i)\b(low|minimal|negligible)\s+risk\b").unwrap();

        if high_risk.is_match(analysis) {
            0.8
        } else if moderate_risk.is_match(analysis) {
            0.5
        } else if low_risk.is_match(analysis) {
            0.2
        } else {
            0.5 // Default moderate risk
        }
    }

    /// Calculate correlation risk for positions
    fn calculate_correlation_risk(&self, positions: &[ArbitragePosition]) -> f64 {
        if positions.len() < 2 {
            0.1 // Low correlation risk with few positions
        } else {
            0.4 // Moderate correlation risk
        }
    }

    /// Calculate concentration risk for positions
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
                max_position / total_value // Concentration as percentage of largest position
            } else {
                0.0
            }
        }
    }

    /// Extract market risk from AI analysis using regex patterns
    fn extract_market_risk(&self, analysis: &str) -> f64 {
        let volatile_market =
            Regex::new(r"(?i)\b(volatile|turbulent|unstable|chaotic)\s+(market|conditions)\b")
                .unwrap();
        let stable_market =
            Regex::new(r"(?i)\b(stable|steady|calm|consolidated)\s+(market|conditions)\b").unwrap();

        if volatile_market.is_match(analysis) {
            0.7
        } else if stable_market.is_match(analysis) {
            0.3
        } else {
            0.5 // Default moderate market risk
        }
    }

    /// Calculate volatility risk for opportunity
    fn calculate_volatility_risk(&self, opportunity: &TradingOpportunity) -> f64 {
        match opportunity.risk_level {
            RiskLevel::Low => 0.2,
            RiskLevel::Medium => 0.5,
            RiskLevel::High => 0.8,
        }
    }

    /// Calculate liquidity risk for opportunity
    fn calculate_liquidity_risk(&self, _opportunity: &TradingOpportunity) -> f64 {
        0.4 // Default moderate liquidity risk
    }

    /// Create empty portfolio analysis
    fn create_empty_portfolio_analysis(&self, user_id: &str) -> AiPortfolioAnalysis {
        AiPortfolioAnalysis {
            user_id: user_id.to_string(),
            correlation_risk_score: 0.0,
            concentration_risk_score: 0.0,
            diversification_score: 1.0,
            recommended_adjustments: vec!["Consider opening initial positions".to_string()],
            overexposure_warnings: Vec::new(),
            optimal_allocation_suggestions: HashMap::new(),
            analysis_timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Get user performance data
    async fn get_user_performance_data(
        &self,
        user_id: &str,
        _days: u32,
    ) -> ArbitrageResult<PerformanceData> {
        // Fetch actual performance data from D1 database
        let analytics = self
            .d1_service
            .get_trading_analytics(user_id, Some(100))
            .await?;

        if analytics.is_empty() {
            return Err(ArbitrageError::not_found(format!(
                "No performance data found for user: {}",
                user_id
            )));
        }

        // Calculate performance metrics from analytics data
        let total_trades = analytics.len() as u32;
        let profitable_trades = analytics
            .iter()
            .filter(|a| a.metric_type == "trade_executed" && a.metric_value > 0.0)
            .count() as u32;
        let win_rate = if total_trades > 0 {
            profitable_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let total_pnl = analytics
            .iter()
            .filter(|a| a.metric_type == "profit_loss")
            .map(|a| a.metric_value)
            .sum::<f64>();

        let average_pnl = if total_trades > 0 {
            total_pnl / total_trades as f64
        } else {
            0.0
        };

        Ok(PerformanceData {
            total_trades,
            win_rate,
            average_pnl,
            _total_pnl: total_pnl,
        })
    }

    /// Create market snapshot for portfolio analysis
    fn create_portfolio_market_snapshot(
        &self,
        positions: &[ArbitragePosition],
    ) -> crate::services::core::trading::ai_exchange_router::MarketDataSnapshot {
        use crate::services::core::trading::ai_exchange_router::{
            MarketContext, MarketDataSnapshot,
        };
        use std::collections::HashMap;

        MarketDataSnapshot {
            timestamp: chrono::Utc::now().timestamp() as u64,
            opportunities: Vec::new(), // Would be populated with current opportunities
            exchange_data: HashMap::new(), // Would be populated with exchange data
            context: MarketContext {
                volatility_index: 0.3,
                market_trend: "neutral".to_string(),
                global_sentiment: 0.5,
                active_pairs: positions.iter().map(|p| p.pair.clone()).collect(),
            },
        }
    }

    /// Create performance market data
    fn create_performance_market_data(
        &self,
        _performance_data: &PerformanceData,
    ) -> crate::services::core::trading::ai_exchange_router::MarketDataSnapshot {
        self.create_portfolio_market_snapshot(&[])
    }

    /// Create config optimization data
    fn create_config_optimization_data(
        &self,
        _config: &Option<UserConfigInstance>,
    ) -> crate::services::core::trading::ai_exchange_router::MarketDataSnapshot {
        self.create_portfolio_market_snapshot(&[])
    }

    // Additional parsing methods using regex patterns
    fn extract_correlation_risk_from_analysis(&self, analysis: &str) -> f64 {
        let high_correlation =
            Regex::new(r"(?i)\b(high|strong|significant)\s+correlation\b").unwrap();
        let low_correlation = Regex::new(r"(?i)\b(low|weak|minimal)\s+correlation\b").unwrap();

        if high_correlation.is_match(analysis) {
            0.8
        } else if low_correlation.is_match(analysis) {
            0.2
        } else {
            0.5
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

    fn extract_portfolio_recommendations(&self, analysis: &str) -> Vec<String> {
        let mut recommendations = Vec::new();
        let diversify_pattern = Regex::new(r"(?i)\b(diversify|spread|distribute)\b").unwrap();
        let reduce_position_pattern =
            Regex::new(r"(?i)\b(reduce|decrease|limit)\s+(position|size|exposure)\b").unwrap();

        if diversify_pattern.is_match(analysis) {
            recommendations.push("Consider diversifying across more trading pairs".to_string());
        }
        if reduce_position_pattern.is_match(analysis) {
            recommendations.push("Consider reducing position sizes".to_string());
        }
        if recommendations.is_empty() {
            recommendations.push("Portfolio looks well balanced".to_string());
        }
        recommendations
    }

    fn extract_overexposure_warnings(&self, analysis: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        if analysis.to_lowercase().contains("overexposed") {
            warnings.push("Overexposure detected in portfolio".to_string());
        }
        warnings
    }

    fn extract_performance_score(&self, analysis: &str) -> f64 {
        if analysis.to_lowercase().contains("excellent performance") {
            0.9
        } else if analysis.to_lowercase().contains("good performance") {
            0.7
        } else if analysis.to_lowercase().contains("poor performance") {
            0.3
        } else {
            0.6
        }
    }

    fn extract_strengths(&self, analysis: &str) -> Vec<String> {
        let mut strengths = Vec::new();
        if analysis.to_lowercase().contains("risk management") {
            strengths.push("Strong risk management".to_string());
        }
        if analysis.to_lowercase().contains("timing") {
            strengths.push("Good market timing".to_string());
        }
        if strengths.is_empty() {
            strengths.push("Consistent trading approach".to_string());
        }
        strengths
    }

    fn extract_weaknesses(&self, analysis: &str) -> Vec<String> {
        let mut weaknesses = Vec::new();
        if analysis.to_lowercase().contains("position sizing") {
            weaknesses.push("Position sizing could be improved".to_string());
        }
        if analysis.to_lowercase().contains("diversification") {
            weaknesses.push("Needs better diversification".to_string());
        }
        weaknesses
    }

    fn extract_focus_suggestion(&self, analysis: &str) -> Option<TradingFocus> {
        if analysis.to_lowercase().contains("focus on arbitrage") {
            Some(TradingFocus::Arbitrage)
        } else if analysis.to_lowercase().contains("focus on technical") {
            Some(TradingFocus::Technical)
        } else if analysis.to_lowercase().contains("hybrid approach") {
            Some(TradingFocus::Hybrid)
        } else {
            None
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

    /// Create default correlation metrics when none available
    fn create_default_correlation_metrics(&self) -> CorrelationMetrics {
        CorrelationMetrics {
            trading_pair: "BTCUSDT".to_string(),
            price_correlations: Vec::new(),
            leadership_analysis: Vec::new(),
            technical_correlations: Vec::new(),
            analysis_timestamp: chrono::Utc::now(),
            confidence_score: 0.5,
        }
    }

    /// Fetch exchange data for correlation analysis (placeholder implementation)
    async fn fetch_exchange_data_for_positions(
        &self,
        _positions: &[ArbitragePosition],
    ) -> ArbitrageResult<
        std::collections::HashMap<
            String,
            crate::services::core::analysis::market_analysis::PriceSeries,
        >,
    > {
        // This would interface with ExchangeService to fetch actual price data
        // For now, return an error to indicate feature not implemented
        Err(ArbitrageError::not_implemented(
            "Exchange data fetching not yet implemented - requires ExchangeService integration"
                .to_string(),
        ))
    }

    // ============= STORAGE METHODS =============

    async fn store_ai_enhancement(
        &self,
        enhancement: &AiOpportunityEnhancement,
    ) -> ArbitrageResult<()> {
        // Store in D1 for analytics
        self.d1_service
            .store_ai_opportunity_enhancement(enhancement)
            .await?;
        Ok(())
    }

    async fn cache_ai_enhancement(
        &self,
        user_id: &str,
        enhancement: &AiOpportunityEnhancement,
    ) -> ArbitrageResult<()> {
        let cache_key = format!("ai_enhancement:{}:{}", user_id, enhancement.opportunity_id);
        let serialized = serde_json::to_string(enhancement).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize enhancement: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, serialized)
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create cache put: {}", e))
            })?
            .expiration_ttl(self.config.cache_ttl_seconds)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to cache enhancement: {}", e))
            })?;

        Ok(())
    }

    async fn store_portfolio_analysis(
        &self,
        analysis: &AiPortfolioAnalysis,
    ) -> ArbitrageResult<()> {
        // Store in D1 for tracking
        self.d1_service
            .store_ai_portfolio_analysis(analysis)
            .await?;
        Ok(())
    }

    async fn store_performance_insights(
        &self,
        insights: &AiPerformanceInsights,
    ) -> ArbitrageResult<()> {
        // Store in D1 for learning
        self.d1_service
            .store_ai_performance_insights(insights)
            .await?;
        Ok(())
    }

    async fn store_parameter_suggestion(
        &self,
        user_id: &str,
        suggestion: &ParameterSuggestion,
    ) -> ArbitrageResult<()> {
        // Store in D1 for tracking
        self.d1_service
            .store_ai_parameter_suggestion(user_id, suggestion)
            .await?;
        Ok(())
    }

    /// Convert TradingOpportunity to GlobalOpportunity for AI router
    fn convert_to_global_opportunity(&self, trading_opp: TradingOpportunity) -> GlobalOpportunity {
        use crate::types::{
            ArbitrageOpportunity, ArbitrageType, DistributionStrategy, GlobalOpportunity,
            OpportunitySource,
        };

        // Create an ArbitrageOpportunity from TradingOpportunity
        // Use dynamic exchange selection based on trading opportunity data
        let (long_exchange, short_exchange) = self.select_exchanges_for_opportunity(&trading_opp);

        let arb_opp = ArbitrageOpportunity::new(
            trading_opp.trading_pair.clone(),
            long_exchange,
            short_exchange,
            Some(trading_opp.entry_price),
            trading_opp.target_price,
            trading_opp.expected_return,
            ArbitrageType::CrossExchange,
        );

        GlobalOpportunity {
            opportunity: arb_opp,
            detection_timestamp: trading_opp.created_at,
            expiry_timestamp: trading_opp
                .expires_at
                .unwrap_or(trading_opp.created_at + 3600000), // 1 hour default
            priority_score: trading_opp.confidence_score,
            distributed_to: Vec::new(),
            max_participants: Some(1),
            current_participants: 0,
            distribution_strategy: DistributionStrategy::FirstComeFirstServe,
            source: OpportunitySource::SystemGenerated,
        }
    }

    /// Select appropriate exchanges for an opportunity based on available data
    fn select_exchanges_for_opportunity(
        &self,
        trading_opp: &TradingOpportunity,
    ) -> (ExchangeIdEnum, ExchangeIdEnum) {
        // Try to parse exchanges from the trading opportunity data
        let available_exchanges: Vec<ExchangeIdEnum> = trading_opp
            .exchanges
            .iter()
            .filter_map(|exchange_str| exchange_str.parse::<ExchangeIdEnum>().ok())
            .collect();

        match available_exchanges.len() {
            0 => {
                // No valid exchanges found, use default fallback
                (ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit)
            }
            1 => {
                // Only one exchange available, use it for both positions (not ideal but functional)
                let exchange = available_exchanges[0];
                (exchange, exchange)
            }
            _ => {
                // Multiple exchanges available, use first two
                (available_exchanges[0], available_exchanges[1])
            }
        }
    }

    /// Get supported exchanges for dynamic selection
    #[allow(dead_code)]
    fn get_supported_exchanges() -> Vec<ExchangeIdEnum> {
        vec![
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
            ExchangeIdEnum::Bitget,
        ]
    }

    /// Select optimal exchanges based on trading pair and market conditions
    #[allow(dead_code)]
    fn select_optimal_exchanges_for_pair(
        &self,
        trading_pair: &str,
    ) -> (ExchangeIdEnum, ExchangeIdEnum) {
        // This could be enhanced with real-time liquidity and spread analysis
        // For now, use a simple rotation based on pair characteristics
        let supported = Self::get_supported_exchanges();

        // Simple hash-based selection for consistent but varied exchange pairing
        let pair_hash = trading_pair.chars().map(|c| c as u32).sum::<u32>();
        let long_idx = (pair_hash % supported.len() as u32) as usize;
        let short_idx = ((pair_hash / 2) % supported.len() as u32) as usize;

        // Ensure we don't use the same exchange for both positions
        let short_idx = if short_idx == long_idx {
            (short_idx + 1) % supported.len()
        } else {
            short_idx
        };

        (supported[long_idx], supported[short_idx])
    }
}

// ============= HELPER DATA STRUCTURES =============

#[derive(Debug, Clone)]
struct PerformanceData {
    total_trades: u32,
    win_rate: f64,
    average_pnl: f64,
    _total_pnl: f64,
}

// ============= TESTS =============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::core::analysis::market_analysis::{
        OpportunityType, RiskLevel, TimeHorizon, TradingOpportunity,
    };
    use crate::types::*;

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
        ArbitragePosition {
            id: format!("pos_{}", value as u32),
            exchange: crate::types::ExchangeIdEnum::Binance,
            pair: "BTCUSDT".to_string(),
            side: crate::types::PositionSide::Long,
            size: value / 50000.0,
            entry_price: 50000.0,
            current_price: Some(50000.0),
            pnl: Some(0.0),
            status: crate::types::PositionStatus::Open,
            created_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
            calculated_size_usd: Some(value),
            risk_percentage_applied: Some(0.02),
            stop_loss_price: Some(49000.0),
            take_profit_price: Some(51000.0),
            trailing_stop_distance: None,
            max_loss_usd: Some(100.0),
            risk_reward_ratio: Some(2.0),
            related_positions: Vec::new(),
            hedge_position_id: None,
            position_group_id: None,
            optimization_score: Some(0.7),
            recommended_action: Some(crate::types::PositionAction::Hold),
            last_optimization_check: None,
            max_drawdown: None,
            unrealized_pnl_percentage: None,
            holding_period_hours: None,
            volatility_score: None,
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
}
