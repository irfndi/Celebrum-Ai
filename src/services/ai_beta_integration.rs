use crate::types::{ArbitrageOpportunity, CommandPermission};
use crate::utils::ArbitrageResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI Enhancement Types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiEnhancementType {
    MarketSentimentAnalysis,
    OpportunityScoring,
    RiskAssessment,
    PatternRecognition,
    PersonalizedRecommendations,
    PortfolioOptimization,
    AutoParameterTuning,
    PredictiveAnalytics,
}

/// AI Confidence Levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiConfidenceLevel {
    Low,      // 0-40%
    Medium,   // 40-70%
    High,     // 70-90%
    VeryHigh, // 90-100%
}

impl From<f64> for AiConfidenceLevel {
    fn from(confidence: f64) -> Self {
        match confidence {
            c if c >= 0.9 => AiConfidenceLevel::VeryHigh,
            c if c >= 0.7 => AiConfidenceLevel::High,
            c if c >= 0.4 => AiConfidenceLevel::Medium,
            _ => AiConfidenceLevel::Low,
        }
    }
}

/// AI Enhanced Opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEnhancedOpportunity {
    pub base_opportunity: ArbitrageOpportunity,
    pub ai_score: f64, // 0.0 to 1.0
    pub confidence_level: AiConfidenceLevel,
    pub risk_adjusted_score: f64,
    pub market_sentiment: MarketSentiment,
    pub ai_insights: Vec<AiInsight>,
    pub personalization_score: f64,
    pub success_probability: f64,
    pub optimal_position_size: f64,
    pub time_sensitivity: TimeSensitivity,
    pub enhancement_metadata: serde_json::Value,
    pub enhanced_at: u64,
}

impl AiEnhancedOpportunity {
    pub fn new(opportunity: ArbitrageOpportunity, ai_score: f64) -> Self {
        Self {
            base_opportunity: opportunity,
            ai_score,
            confidence_level: AiConfidenceLevel::from(ai_score),
            risk_adjusted_score: ai_score,
            market_sentiment: MarketSentiment::Neutral,
            ai_insights: Vec::new(),
            personalization_score: 0.0,
            success_probability: ai_score,
            optimal_position_size: 0.0,
            time_sensitivity: TimeSensitivity::Medium,
            enhancement_metadata: serde_json::Value::Object(serde_json::Map::new()),
            enhanced_at: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    pub fn with_market_sentiment(mut self, sentiment: MarketSentiment) -> Self {
        self.market_sentiment = sentiment;
        self
    }

    pub fn with_insights(mut self, insights: Vec<AiInsight>) -> Self {
        self.ai_insights = insights;
        self
    }

    pub fn with_personalization_score(mut self, score: f64) -> Self {
        self.personalization_score = score;
        self
    }

    pub fn calculate_final_score(&self) -> f64 {
        // Weighted combination of AI score, risk adjustment, and personalization
        let weights = AiScoringWeights::default();

        (self.ai_score * weights.ai_score_weight)
            + (self.risk_adjusted_score * weights.risk_weight)
            + (self.personalization_score * weights.personalization_weight)
            + (self.market_sentiment.score() * weights.sentiment_weight)
    }
}

/// Market Sentiment Analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketSentiment {
    VeryBearish,
    Bearish,
    Neutral,
    Bullish,
    VeryBullish,
}

impl MarketSentiment {
    pub fn score(&self) -> f64 {
        match self {
            MarketSentiment::VeryBearish => -1.0,
            MarketSentiment::Bearish => -0.5,
            MarketSentiment::Neutral => 0.0,
            MarketSentiment::Bullish => 0.5,
            MarketSentiment::VeryBullish => 1.0,
        }
    }
}

/// Time Sensitivity for Opportunities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSensitivity {
    VeryHigh, // < 5 minutes
    High,     // 5-30 minutes
    Medium,   // 30 minutes - 2 hours
    Low,      // 2+ hours
}

/// AI Insight types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsight {
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub impact_level: ImpactLevel,
    pub recommendation: String,
    pub supporting_data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// AI Scoring Weights Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiScoringWeights {
    pub ai_score_weight: f64,
    pub risk_weight: f64,
    pub personalization_weight: f64,
    pub sentiment_weight: f64,
}

impl Default for AiScoringWeights {
    fn default() -> Self {
        Self {
            ai_score_weight: 0.4,
            risk_weight: 0.3,
            personalization_weight: 0.2,
            sentiment_weight: 0.1,
        }
    }
}

/// User Trading Profile for AI Personalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTradingProfile {
    pub user_id: String,
    pub risk_tolerance: f64, // 0.0 to 1.0
    pub experience_level: ExperienceLevel,
    pub preferred_strategies: Vec<String>,
    pub historical_performance: PerformanceMetrics,
    pub trading_patterns: TradingPatterns,
    pub personalization_features: HashMap<String, f64>,
    pub learning_data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_trades: u32,
    pub win_rate: f64,
    pub average_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub roi: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPatterns {
    pub preferred_timeframes: Vec<String>,
    pub average_position_size: f64,
    pub frequency_per_week: f64,
    pub preferred_pairs: Vec<String>,
    pub risk_per_trade: f64,
}

/// AI Beta Integration Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBetaConfig {
    pub enabled_features: Vec<AiEnhancementType>,
    pub min_confidence_threshold: f64,
    pub personalization_enabled: bool,
    pub advanced_analytics_enabled: bool,
    pub ml_predictions_enabled: bool,
    pub auto_optimization_enabled: bool,
    pub beta_user_access_only: bool,
    pub max_ai_opportunities_per_hour: u32,
}

impl Default for AiBetaConfig {
    fn default() -> Self {
        Self {
            enabled_features: vec![
                AiEnhancementType::OpportunityScoring,
                AiEnhancementType::MarketSentimentAnalysis,
                AiEnhancementType::RiskAssessment,
                AiEnhancementType::PersonalizedRecommendations,
            ],
            min_confidence_threshold: 0.75,
            personalization_enabled: true,
            advanced_analytics_enabled: true,
            ml_predictions_enabled: true,
            auto_optimization_enabled: false, // Disabled by default for safety
            beta_user_access_only: true,
            max_ai_opportunities_per_hour: 5,
        }
    }
}

/// AI Beta Integration Service
pub struct AiBetaIntegrationService {
    config: AiBetaConfig,
    user_profiles: HashMap<String, AiTradingProfile>,
    market_sentiment_cache: HashMap<String, MarketSentiment>,
    ai_model_metrics: AiModelMetrics,
}

impl AiBetaIntegrationService {
    pub fn new(config: AiBetaConfig) -> Self {
        Self {
            config,
            user_profiles: HashMap::new(),
            market_sentiment_cache: HashMap::new(),
            ai_model_metrics: AiModelMetrics::default(),
        }
    }

    /// Check if user has access to AI beta features
    pub fn check_beta_access(&self, user_permissions: &[CommandPermission]) -> bool {
        if !self.config.beta_user_access_only {
            return true;
        }

        user_permissions.contains(&CommandPermission::AIEnhancedOpportunities)
    }

    /// Enhance opportunities with AI analysis
    pub async fn enhance_opportunities(
        &mut self,
        opportunities: Vec<ArbitrageOpportunity>,
        user_id: &str,
    ) -> ArbitrageResult<Vec<AiEnhancedOpportunity>> {
        let mut enhanced_opportunities = Vec::new();

        for opportunity in opportunities {
            if let Ok(enhanced) = self.enhance_single_opportunity(opportunity, user_id).await {
                if enhanced.ai_score >= self.config.min_confidence_threshold {
                    enhanced_opportunities.push(enhanced);
                }
            }
        }

        // Sort by final AI score
        enhanced_opportunities.sort_by(|a, b| {
            b.calculate_final_score()
                .partial_cmp(&a.calculate_final_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to configured maximum
        let max_opportunities = self.config.max_ai_opportunities_per_hour as usize;
        enhanced_opportunities.truncate(max_opportunities);

        Ok(enhanced_opportunities)
    }

    /// Enhance a single opportunity with AI analysis
    async fn enhance_single_opportunity(
        &mut self,
        opportunity: ArbitrageOpportunity,
        user_id: &str,
    ) -> ArbitrageResult<AiEnhancedOpportunity> {
        // Calculate base AI score
        let ai_score = self.calculate_ai_score(&opportunity).await;

        // Create enhanced opportunity
        let mut enhanced = AiEnhancedOpportunity::new(opportunity.clone(), ai_score);

        // Add market sentiment analysis
        let sentiment = self.analyze_market_sentiment(&opportunity.pair).await;
        enhanced = enhanced.with_market_sentiment(sentiment);

        // Generate AI insights
        let insights = self.generate_ai_insights(&opportunity, ai_score).await;
        enhanced = enhanced.with_insights(insights);

        // Add personalization if user profile exists
        if let Some(profile) = self.user_profiles.get(user_id) {
            let personalization_score = self.calculate_personalization_score(&opportunity, profile);
            enhanced = enhanced.with_personalization_score(personalization_score);
        }

        // Calculate additional metrics
        enhanced.risk_adjusted_score = self.calculate_risk_adjusted_score(&enhanced);
        enhanced.success_probability = self.predict_success_probability(&enhanced).await;
        enhanced.optimal_position_size = self.calculate_optimal_position_size(&enhanced, user_id);
        enhanced.time_sensitivity = self.assess_time_sensitivity(&enhanced);

        Ok(enhanced)
    }

    /// Calculate AI score for an opportunity
    async fn calculate_ai_score(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        // Mock AI scoring algorithm
        // In production, this would use ML models, market data analysis, etc.

        let mut score = 0.5; // Base score

        // Rate difference impact
        score += (opportunity.rate_difference * 100.0).min(0.3);

        // Pair popularity (BTC/ETH get higher scores)
        if opportunity.pair.contains("BTC") || opportunity.pair.contains("ETH") {
            score += 0.1;
        }

        // Exchange reliability
        if let Some(exchange) = &opportunity.long_exchange {
            score += match exchange {
                crate::types::ExchangeIdEnum::Binance => 0.15,
                crate::types::ExchangeIdEnum::Bybit => 0.1,
                crate::types::ExchangeIdEnum::OKX => 0.1,
                _ => 0.05,
            };
        }

        // Random factor to simulate ML uncertainty
        let random_factor = (opportunity.timestamp % 100) as f64 / 1000.0;
        score += random_factor;

        score.min(1.0)
    }

    /// Analyze market sentiment for a trading pair
    async fn analyze_market_sentiment(&mut self, pair: &str) -> MarketSentiment {
        // Check cache first
        if let Some(sentiment) = self.market_sentiment_cache.get(pair) {
            return sentiment.clone();
        }

        // Mock sentiment analysis
        let sentiment = match pair {
            p if p.contains("BTC") => MarketSentiment::Bullish,
            p if p.contains("ETH") => MarketSentiment::Bullish,
            p if p.contains("ADA") => MarketSentiment::Neutral,
            p if p.contains("SOL") => MarketSentiment::VeryBullish,
            _ => MarketSentiment::Neutral,
        };

        // Cache the result
        self.market_sentiment_cache
            .insert(pair.to_string(), sentiment.clone());
        sentiment
    }

    /// Generate AI insights for an opportunity
    async fn generate_ai_insights(
        &self,
        opportunity: &ArbitrageOpportunity,
        ai_score: f64,
    ) -> Vec<AiInsight> {
        let mut insights = Vec::new();

        // Score-based insights
        if ai_score > 0.9 {
            insights.push(AiInsight {
                insight_type: "high_confidence_prediction".to_string(),
                title: "High AI Confidence".to_string(),
                description: "AI models show very high confidence in this opportunity".to_string(),
                confidence: ai_score,
                impact_level: ImpactLevel::High,
                recommendation: "Consider increasing position size for this high-confidence opportunity".to_string(),
                supporting_data: serde_json::json!({"model_consensus": 0.95, "historical_success_rate": 0.87}),
            });
        }

        // Rate difference insights
        if opportunity.rate_difference > 0.005 {
            insights.push(AiInsight {
                insight_type: "rate_arbitrage_potential".to_string(),
                title: "Significant Rate Difference".to_string(),
                description: format!("Rate difference of {:.3}% detected", opportunity.rate_difference * 100.0),
                confidence: 0.9,
                impact_level: ImpactLevel::Medium,
                recommendation: "Monitor for quick execution as rate differences may close rapidly".to_string(),
                supporting_data: serde_json::json!({"rate_difference": opportunity.rate_difference}),
            });
        }

        // Pair-specific insights
        if opportunity.pair.contains("BTC") {
            insights.push(AiInsight {
                insight_type: "market_leader_signal".to_string(),
                title: "Bitcoin Market Movement".to_string(),
                description: "BTC movements often influence broader market sentiment".to_string(),
                confidence: 0.8,
                impact_level: ImpactLevel::Medium,
                recommendation: "Consider market correlation effects in position sizing"
                    .to_string(),
                supporting_data: serde_json::json!({"market_correlation": 0.75}),
            });
        }

        insights
    }

    /// Calculate personalization score based on user profile
    fn calculate_personalization_score(
        &self,
        opportunity: &ArbitrageOpportunity,
        profile: &AiTradingProfile,
    ) -> f64 {
        let mut score = 0.5; // Base score

        // Risk tolerance alignment
        let opportunity_risk = self.assess_opportunity_risk(opportunity);
        let risk_alignment = 1.0 - (opportunity_risk - profile.risk_tolerance).abs();
        score += risk_alignment * 0.3;

        // Preferred pairs
        if profile
            .trading_patterns
            .preferred_pairs
            .contains(&opportunity.pair)
        {
            score += 0.2;
        }

        // Experience level adjustment
        match profile.experience_level {
            ExperienceLevel::Beginner => {
                // Prefer safer opportunities
                if opportunity.rate_difference < 0.01 {
                    score += 0.1;
                }
            }
            ExperienceLevel::Expert => {
                // Can handle more complex opportunities
                if opportunity.rate_difference > 0.02 {
                    score += 0.1;
                }
            }
            _ => {}
        }

        score.min(1.0)
    }

    /// Assess risk level of an opportunity
    fn assess_opportunity_risk(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        let mut risk = 0.3; // Base risk

        // Rate difference risk (higher rate diff = higher risk)
        risk += (opportunity.rate_difference * 10.0).min(0.4);

        // Exchange risk
        let exchange_risk = match &opportunity.long_exchange {
            Some(crate::types::ExchangeIdEnum::Binance) => 0.05,
            Some(crate::types::ExchangeIdEnum::Bybit) => 0.1,
            Some(crate::types::ExchangeIdEnum::OKX) => 0.1,
            _ => 0.2,
        };
        risk += exchange_risk;

        risk.min(1.0)
    }

    /// Calculate risk-adjusted score
    fn calculate_risk_adjusted_score(&self, enhanced: &AiEnhancedOpportunity) -> f64 {
        let risk = self.assess_opportunity_risk(&enhanced.base_opportunity);
        let risk_penalty = risk * 0.5;

        (enhanced.ai_score - risk_penalty).max(0.0)
    }

    /// Predict success probability using ML models (mock implementation)
    async fn predict_success_probability(&self, enhanced: &AiEnhancedOpportunity) -> f64 {
        // Mock ML prediction
        let base_probability = enhanced.ai_score;
        let sentiment_adjustment = enhanced.market_sentiment.score() * 0.1;
        let risk_adjustment = -self.assess_opportunity_risk(&enhanced.base_opportunity) * 0.2;

        (base_probability + sentiment_adjustment + risk_adjustment).clamp(0.1, 0.99)
    }

    /// Calculate optimal position size
    fn calculate_optimal_position_size(
        &self,
        enhanced: &AiEnhancedOpportunity,
        user_id: &str,
    ) -> f64 {
        if let Some(profile) = self.user_profiles.get(user_id) {
            let base_size = profile.trading_patterns.average_position_size;
            let confidence_multiplier = enhanced.ai_score;
            let risk_adjustment = 1.0 - self.assess_opportunity_risk(&enhanced.base_opportunity);

            base_size * confidence_multiplier * risk_adjustment
        } else {
            // Default conservative size
            100.0
        }
    }

    /// Assess time sensitivity of opportunity
    fn assess_time_sensitivity(&self, enhanced: &AiEnhancedOpportunity) -> TimeSensitivity {
        let rate_diff = enhanced.base_opportunity.rate_difference;

        match rate_diff {
            r if r > 0.01 => TimeSensitivity::VeryHigh,
            r if r > 0.005 => TimeSensitivity::High,
            r if r > 0.002 => TimeSensitivity::Medium,
            _ => TimeSensitivity::Low,
        }
    }

    /// Create or update user trading profile
    pub fn update_user_profile(&mut self, user_id: String, profile: AiTradingProfile) {
        self.user_profiles.insert(user_id, profile);
    }

    /// Get AI statistics and metrics
    pub fn get_ai_metrics(&self) -> &AiModelMetrics {
        &self.ai_model_metrics
    }

    /// Generate personalized AI recommendations
    pub async fn get_personalized_recommendations(&self, user_id: &str) -> Vec<AiRecommendation> {
        let mut recommendations = Vec::new();

        if let Some(profile) = self.user_profiles.get(user_id) {
            // Risk management recommendations
            if profile.historical_performance.max_drawdown > 0.2 {
                recommendations.push(AiRecommendation {
                    recommendation_type: "risk_management".to_string(),
                    title: "Reduce Position Sizes".to_string(),
                    description:
                        "Your maximum drawdown suggests reducing position sizes to preserve capital"
                            .to_string(),
                    priority: RecommendationPriority::High,
                    estimated_impact: 0.8,
                    action_items: vec![
                        "Reduce position sizes by 25%".to_string(),
                        "Implement stricter stop losses".to_string(),
                        "Diversify across more opportunities".to_string(),
                    ],
                });
            }

            // Performance improvement recommendations
            if profile.historical_performance.win_rate < 0.6 {
                recommendations.push(AiRecommendation {
                    recommendation_type: "strategy_optimization".to_string(),
                    title: "Improve Trade Selection".to_string(),
                    description: "AI analysis suggests focusing on higher-confidence opportunities"
                        .to_string(),
                    priority: RecommendationPriority::Medium,
                    estimated_impact: 0.6,
                    action_items: vec![
                        "Increase minimum confidence threshold to 80%".to_string(),
                        "Focus on familiar trading pairs".to_string(),
                        "Wait for stronger technical confirmations".to_string(),
                    ],
                });
            }
        }

        recommendations
    }
}

/// AI Model Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelMetrics {
    pub total_predictions: u64,
    pub successful_predictions: u64,
    pub accuracy_rate: f64,
    pub average_confidence: f64,
    pub model_version: String,
    pub last_updated: u64,
}

impl Default for AiModelMetrics {
    fn default() -> Self {
        Self {
            total_predictions: 0,
            successful_predictions: 0,
            accuracy_rate: 0.0,
            average_confidence: 0.0,
            model_version: "beta-1.0".to_string(),
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// AI Recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRecommendation {
    pub recommendation_type: String,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_impact: f64,
    pub action_items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ArbitrageType;

    fn create_test_opportunity() -> ArbitrageOpportunity {
        ArbitrageOpportunity::new(
            "BTCUSDT".to_string(),
            Some(crate::types::ExchangeIdEnum::Binance),
            Some(crate::types::ExchangeIdEnum::Bybit),
            Some(43250.0),
            Some(44000.0),
            2.5,
            ArbitrageType::CrossExchange,
        )
    }

    fn create_test_profile() -> AiTradingProfile {
        AiTradingProfile {
            user_id: "test_user".to_string(),
            risk_tolerance: 0.6,
            experience_level: ExperienceLevel::Intermediate,
            preferred_strategies: vec!["arbitrage".to_string()],
            historical_performance: PerformanceMetrics {
                total_trades: 50,
                win_rate: 0.75,
                average_return: 0.15,
                max_drawdown: 0.1,
                sharpe_ratio: 1.2,
                roi: 0.25,
            },
            trading_patterns: TradingPatterns {
                preferred_timeframes: vec!["4h".to_string()],
                average_position_size: 500.0,
                frequency_per_week: 10.0,
                preferred_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
                risk_per_trade: 0.02,
            },
            personalization_features: HashMap::new(),
            learning_data: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_ai_enhanced_opportunity_creation() {
        let opportunity = create_test_opportunity();
        let enhanced = AiEnhancedOpportunity::new(opportunity.clone(), 0.85);

        assert_eq!(enhanced.base_opportunity.pair, "BTCUSDT");
        assert_eq!(enhanced.ai_score, 0.85);
        assert_eq!(enhanced.confidence_level, AiConfidenceLevel::High);
    }

    #[test]
    fn test_confidence_level_conversion() {
        assert_eq!(AiConfidenceLevel::from(0.95), AiConfidenceLevel::VeryHigh);
        assert_eq!(AiConfidenceLevel::from(0.75), AiConfidenceLevel::High);
        assert_eq!(AiConfidenceLevel::from(0.55), AiConfidenceLevel::Medium);
        assert_eq!(AiConfidenceLevel::from(0.25), AiConfidenceLevel::Low);
    }

    #[test]
    fn test_market_sentiment_scoring() {
        assert_eq!(MarketSentiment::VeryBullish.score(), 1.0);
        assert_eq!(MarketSentiment::Bullish.score(), 0.5);
        assert_eq!(MarketSentiment::Neutral.score(), 0.0);
        assert_eq!(MarketSentiment::Bearish.score(), -0.5);
        assert_eq!(MarketSentiment::VeryBearish.score(), -1.0);
    }

    #[tokio::test]
    async fn test_ai_beta_service_creation() {
        let config = AiBetaConfig::default();
        let service = AiBetaIntegrationService::new(config);

        assert_eq!(service.user_profiles.len(), 0);
        assert_eq!(service.market_sentiment_cache.len(), 0);
    }

    #[test]
    fn test_beta_access_check() {
        let config = AiBetaConfig::default();
        let service = AiBetaIntegrationService::new(config);

        let permissions = vec![CommandPermission::AIEnhancedOpportunities];
        assert!(service.check_beta_access(&permissions));

        let no_permissions = vec![CommandPermission::BasicCommands];
        assert!(!service.check_beta_access(&no_permissions));
    }

    #[tokio::test]
    async fn test_enhance_opportunities() {
        let config = AiBetaConfig::default();
        let mut service = AiBetaIntegrationService::new(config);

        let opportunities = vec![create_test_opportunity()];
        let enhanced = service
            .enhance_opportunities(opportunities, "test_user")
            .await
            .unwrap();

        assert!(!enhanced.is_empty());
        assert!(enhanced[0].ai_score > 0.0);
    }

    #[test]
    fn test_personalization_score_calculation() {
        let config = AiBetaConfig::default();
        let service = AiBetaIntegrationService::new(config);

        let opportunity = create_test_opportunity();
        let profile = create_test_profile();

        let score = service.calculate_personalization_score(&opportunity, &profile);
        assert!(score > 0.0 && score <= 1.0);

        // Should get bonus for preferred pair
        assert!(score > 0.5); // Base score + preferred pair bonus
    }

    #[test]
    fn test_final_score_calculation() {
        let opportunity = create_test_opportunity();
        let enhanced = AiEnhancedOpportunity::new(opportunity, 0.8)
            .with_market_sentiment(MarketSentiment::Bullish)
            .with_personalization_score(0.9);

        let final_score = enhanced.calculate_final_score();
        assert!(final_score > 0.0 && final_score <= 1.0);
        assert!(final_score > enhanced.ai_score); // Should be higher due to good sentiment and personalization
    }

    #[tokio::test]
    async fn test_personalized_recommendations() {
        let config = AiBetaConfig::default();
        let mut service = AiBetaIntegrationService::new(config);

        let mut profile = create_test_profile();
        profile.historical_performance.max_drawdown = 0.3; // High drawdown
        service.update_user_profile("test_user".to_string(), profile);

        let recommendations = service.get_personalized_recommendations("test_user").await;
        assert!(!recommendations.is_empty());

        // Should recommend risk management due to high drawdown
        let risk_rec = recommendations
            .iter()
            .find(|r| r.recommendation_type == "risk_management");
        assert!(risk_rec.is_some());
    }
}
