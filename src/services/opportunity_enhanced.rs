// src/services/opportunity_enhanced.rs
// Task 9.2: Enhanced arbitrage detection with technical analysis confirmation

use crate::log_error;
use crate::services::exchange::ExchangeInterface;
use crate::services::exchange::ExchangeService;
use crate::services::market_analysis::{
    MarketAnalysisService, OpportunityType, RiskLevel, TimeHorizon, TradingOpportunity,
};
use crate::services::telegram::TelegramService;
use crate::services::user_trading_preferences::{
    ExperienceLevel, RiskTolerance, TradingFocus, UserTradingPreferencesService,
};
use crate::types::{
    ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum, FundingRateInfo, StructuredTradingPair,
};
use crate::utils::{logger::Logger, ArbitrageResult};
use std::sync::Arc;

use futures::future::join_all;
use std::collections::HashMap;
// use serde_json::Value; // TODO: Re-enable when implementing JSON processing

#[derive(Clone)]
pub struct EnhancedOpportunityServiceConfig {
    pub exchanges: Vec<ExchangeIdEnum>,
    pub monitored_pairs: Vec<StructuredTradingPair>,
    pub threshold: f64,
    // Technical Analysis Enhancement Configuration
    pub enable_technical_analysis: bool,
    pub technical_confirmation_weight: f64, // 0.0 to 1.0, how much to weight technical analysis
    pub min_technical_confidence: f64,      // Minimum technical confidence score (0.0 to 1.0)
    pub volatility_threshold: f64,          // Maximum volatility allowed for arbitrage
    pub correlation_threshold: f64,         // Minimum price correlation between exchanges
    pub rsi_overbought: f64,                // RSI threshold for overbought (default: 70)
    pub rsi_oversold: f64,                  // RSI threshold for oversold (default: 30)
}

impl Default for EnhancedOpportunityServiceConfig {
    fn default() -> Self {
        Self {
            exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            monitored_pairs: vec![],
            threshold: 0.0001, // 0.01%
            enable_technical_analysis: true,
            technical_confirmation_weight: 0.3, // 30% weight to technical analysis
            min_technical_confidence: 0.6,      // 60% minimum confidence
            volatility_threshold: 0.05,         // 5% maximum volatility
            correlation_threshold: 0.7,         // 70% minimum correlation
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
        }
    }
}

pub struct EnhancedOpportunityService {
    config: EnhancedOpportunityServiceConfig,
    exchange_service: Arc<ExchangeService>,
    _telegram_service: Option<Arc<TelegramService>>,
    market_analysis_service: Arc<MarketAnalysisService>,
    preferences_service: Arc<UserTradingPreferencesService>,
    logger: Logger,
}

impl EnhancedOpportunityService {
    pub fn new(
        config: EnhancedOpportunityServiceConfig,
        exchange_service: Arc<ExchangeService>,
        telegram_service: Option<Arc<TelegramService>>,
        market_analysis_service: Arc<MarketAnalysisService>,
        preferences_service: Arc<UserTradingPreferencesService>,
        logger: Logger,
    ) -> Self {
        Self {
            config,
            exchange_service,
            _telegram_service: telegram_service,
            market_analysis_service,
            preferences_service,
            logger,
        }
    }

    /// Enhanced arbitrage detection with technical analysis confirmation
    /// Task 9.2: Combines traditional arbitrage with technical indicators for better timing
    pub async fn find_enhanced_opportunities(
        &self,
        exchange_ids: &[ExchangeIdEnum],
        pairs: &[String],
        threshold: f64,
        user_id: Option<&str>,
    ) -> ArbitrageResult<Vec<TradingOpportunity>> {
        let mut enhanced_opportunities = Vec::new();

        self.logger.info(&format!(
            "Starting enhanced arbitrage detection for {} pairs across {} exchanges",
            pairs.len(),
            exchange_ids.len()
        ));

        // Step 1: Get traditional arbitrage opportunities (reuse existing logic)
        let arbitrage_opportunities = self
            .find_basic_arbitrage_opportunities(exchange_ids, pairs, threshold)
            .await?;

        if arbitrage_opportunities.is_empty() {
            self.logger.info("No basic arbitrage opportunities found");
            return Ok(enhanced_opportunities);
        }

        // Step 2: Get user preferences for filtering
        let user_preferences = if let Some(uid) = user_id {
            Some(
                self.preferences_service
                    .get_or_create_preferences(uid)
                    .await?,
            )
        } else {
            None
        };

        // Skip technical analysis if user preference is arbitrage-only and technical analysis is disabled
        if let Some(ref prefs) = user_preferences {
            if prefs.trading_focus == TradingFocus::Arbitrage
                && !self.config.enable_technical_analysis
            {
                // Convert arbitrage opportunities to trading opportunities without technical analysis
                for arb_opp in arbitrage_opportunities {
                    let trading_opp = self
                        .convert_arbitrage_to_trading_opportunity(arb_opp, 1.0, "arbitrage_only")
                        .await?;
                    enhanced_opportunities.push(trading_opp);
                }
                return Ok(enhanced_opportunities);
            }
        }

        self.logger.info(&format!(
            "Analyzing {} arbitrage opportunities with technical indicators",
            arbitrage_opportunities.len()
        ));

        // Step 3: Enhance each arbitrage opportunity with technical analysis
        for arb_opportunity in &arbitrage_opportunities {
            match self
                .enhance_arbitrage_with_technical_analysis(
                    arb_opportunity,
                    user_preferences.as_ref(),
                )
                .await
            {
                Ok(Some(enhanced_opp)) => {
                    enhanced_opportunities.push(enhanced_opp);
                }
                Ok(None) => {
                    self.logger.debug(&format!(
                        "Arbitrage opportunity {} filtered out by technical analysis",
                        arb_opportunity.id
                    ));
                }
                Err(e) => {
                    self.logger.warn(&format!(
                        "Failed to analyze opportunity {}: {}",
                        arb_opportunity.id, e
                    ));
                }
            }
        }

        self.logger.info(&format!(
            "Enhanced {} out of {} arbitrage opportunities passed technical analysis",
            enhanced_opportunities.len(),
            arbitrage_opportunities.len()
        ));

        Ok(enhanced_opportunities)
    }

    /// Basic arbitrage opportunity detection (traditional funding rate arbitrage)
    async fn find_basic_arbitrage_opportunities(
        &self,
        exchange_ids: &[ExchangeIdEnum],
        pairs: &[String],
        threshold: f64,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Step 1: Fetch funding rates for all pairs and exchanges
        let mut funding_rate_data: HashMap<
            String,
            HashMap<ExchangeIdEnum, Option<FundingRateInfo>>,
        > = HashMap::new();

        // Initialize maps
        for pair in pairs {
            funding_rate_data.insert(pair.clone(), HashMap::new());
        }

        // Collect funding rate fetch operations
        let mut funding_tasks = Vec::new();

        for pair in pairs {
            for exchange_id in exchange_ids {
                let exchange_service = Arc::clone(&self.exchange_service);
                let pair = pair.clone();
                let exchange_id = *exchange_id;

                let task = Box::pin(async move {
                    let result = exchange_service
                        .fetch_funding_rates(&exchange_id.to_string(), Some(&pair))
                        .await;

                    let funding_info = match result {
                        Ok(rates) => {
                            if let Some(rate_data) = rates.first() {
                                match rate_data["fundingRate"].as_str() {
                                    Some(rate_str) => match rate_str.parse::<f64>() {
                                        Ok(funding_rate) => Some(FundingRateInfo {
                                            symbol: pair.clone(),
                                            funding_rate,
                                            timestamp: Some(chrono::Utc::now()),
                                            datetime: Some(chrono::Utc::now().to_rfc3339()),
                                            next_funding_time: None,
                                            estimated_rate: None,
                                        }),
                                        Err(parse_err) => {
                                            log_error!(
                                                "Failed to parse funding rate",
                                                serde_json::json!({
                                                    "exchange": exchange_id.to_string(),
                                                    "pair": pair,
                                                    "raw_value": rate_str,
                                                    "error": parse_err.to_string()
                                                })
                                            );
                                            None
                                        }
                                    },
                                    None => {
                                        log_error!(
                                            "Missing fundingRate field in response",
                                            serde_json::json!({
                                                "exchange": exchange_id.to_string(),
                                                "pair": pair,
                                                "response": rate_data
                                            })
                                        );
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    };

                    (pair, exchange_id, funding_info)
                });
                funding_tasks.push(task);
            }
        }

        // Execute all funding rate fetch operations concurrently
        let funding_results = join_all(funding_tasks).await;

        // Process funding rate results
        for (pair, exchange_id, funding_info) in funding_results {
            if let Some(pair_map) = funding_rate_data.get_mut(&pair) {
                pair_map.insert(exchange_id, funding_info);
            }
        }

        // Step 2: Identify opportunities
        for pair in pairs {
            if let Some(pair_funding_rates) = funding_rate_data.get(pair) {
                let available_exchanges: Vec<ExchangeIdEnum> = pair_funding_rates
                    .iter()
                    .filter_map(|(exchange_id, rate_info)| {
                        if rate_info.is_some() {
                            Some(*exchange_id)
                        } else {
                            None
                        }
                    })
                    .collect();

                if available_exchanges.len() < 2 {
                    continue;
                }

                // Compare all pairs of exchanges
                for i in 0..available_exchanges.len() {
                    for j in (i + 1)..available_exchanges.len() {
                        let exchange_a = available_exchanges[i];
                        let exchange_b = available_exchanges[j];

                        if let (Some(Some(rate_a)), Some(Some(rate_b))) = (
                            pair_funding_rates.get(&exchange_a),
                            pair_funding_rates.get(&exchange_b),
                        ) {
                            let rate_diff = (rate_a.funding_rate - rate_b.funding_rate).abs();

                            if rate_diff >= threshold {
                                let (long_exchange, short_exchange, long_rate, short_rate) =
                                    if rate_a.funding_rate > rate_b.funding_rate {
                                        (
                                            exchange_b,
                                            exchange_a,
                                            rate_b.funding_rate,
                                            rate_a.funding_rate,
                                        )
                                    } else {
                                        (
                                            exchange_a,
                                            exchange_b,
                                            rate_a.funding_rate,
                                            rate_b.funding_rate,
                                        )
                                    };

                                let opportunity = ArbitrageOpportunity {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    pair: pair.clone(),
                                    r#type: ArbitrageType::FundingRate,
                                    long_exchange: Some(long_exchange),
                                    short_exchange: Some(short_exchange),
                                    long_rate: Some(long_rate),
                                    short_rate: Some(short_rate),
                                    rate_difference: rate_diff,
                                    net_rate_difference: Some(rate_diff),
                                    potential_profit_value: None,
                                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                                    details: Some(format!(
                                        "Funding rate arbitrage: Long {} ({}%) / Short {} ({}%)",
                                        long_exchange,
                                        (long_rate * 100.0 * 10000.0).round() / 10000.0,
                                        short_exchange,
                                        (short_rate * 100.0 * 10000.0).round() / 10000.0
                                    )),
                                };

                                opportunities.push(opportunity);
                            }
                        }
                    }
                }
            }
        }

        Ok(opportunities)
    }

    /// Enhance a single arbitrage opportunity with technical analysis
    async fn enhance_arbitrage_with_technical_analysis(
        &self,
        arbitrage_opportunity: &ArbitrageOpportunity,
        user_preferences: Option<
            &crate::services::user_trading_preferences::UserTradingPreferences,
        >,
    ) -> ArbitrageResult<Option<TradingOpportunity>> {
        let pair = &arbitrage_opportunity.pair;

        // Get price data for technical analysis
        let technical_score = self
            .calculate_technical_confirmation_score(
                pair,
                arbitrage_opportunity.long_exchange.as_ref(),
                arbitrage_opportunity.short_exchange.as_ref(),
            )
            .await?;

        // Apply technical analysis filtering
        if technical_score < self.config.min_technical_confidence {
            return Ok(None); // Filtered out by technical analysis
        }

        // Calculate combined confidence score
        let arbitrage_confidence = self.calculate_arbitrage_confidence(arbitrage_opportunity);
        let combined_confidence =
            self.combine_confidence_scores(arbitrage_confidence, technical_score);

        // Apply user preference filtering
        if let Some(prefs) = user_preferences {
            if !self.passes_user_preference_filter(combined_confidence, technical_score, prefs) {
                return Ok(None);
            }
        }

        // Convert to enhanced trading opportunity
        let enhanced_opportunity = self
            .convert_arbitrage_to_trading_opportunity(
                arbitrage_opportunity.clone(),
                combined_confidence,
                &format!("arbitrage_technical_score_{:.3}", technical_score),
            )
            .await?;

        Ok(Some(enhanced_opportunity))
    }

    /// Calculate technical confirmation score for an arbitrage opportunity
    async fn calculate_technical_confirmation_score(
        &self,
        pair: &str,
        long_exchange: Option<&ExchangeIdEnum>,
        short_exchange: Option<&ExchangeIdEnum>,
    ) -> ArbitrageResult<f64> {
        let mut total_score = 0.0;
        let mut score_count = 0;

        // Get price series for both exchanges if available
        let exchanges_to_analyze = [long_exchange, short_exchange]
            .iter()
            .filter_map(|&ex| ex.map(|e| e.to_string()))
            .collect::<Vec<_>>();

        if exchanges_to_analyze.is_empty() {
            return Ok(0.5); // Neutral score if no exchange data
        }

        for exchange_id in &exchanges_to_analyze {
            if let Some(price_series) = self
                .market_analysis_service
                .get_price_series(exchange_id, pair)
            {
                // Calculate technical indicators
                let indicators = self
                    .market_analysis_service
                    .calculate_indicators(price_series, &["sma_20", "rsi_14"])?;

                // Analyze RSI for market timing
                if let Some(rsi_indicator) =
                    indicators.iter().find(|i| i.indicator_name == "RSI_14")
                {
                    if let Some(latest_rsi) = rsi_indicator.values.last() {
                        let rsi_score = self.calculate_rsi_score(latest_rsi.value);
                        total_score += rsi_score;
                        score_count += 1;
                    }
                }

                // Analyze volatility using price series
                let volatility_score = self.calculate_volatility_score(price_series)?;
                total_score += volatility_score;
                score_count += 1;
            }
        }

        // Calculate price correlation between exchanges if we have both
        if exchanges_to_analyze.len() == 2 {
            let correlation_score = self
                .calculate_correlation_score(
                    &exchanges_to_analyze[0],
                    &exchanges_to_analyze[1],
                    pair,
                )
                .await?;
            total_score += correlation_score;
            score_count += 1;
        }

        if score_count == 0 {
            Ok(0.5) // Neutral score if no technical data available
        } else {
            Ok(total_score / score_count as f64)
        }
    }

    /// Calculate RSI-based score for market timing
    fn calculate_rsi_score(&self, rsi_value: f64) -> f64 {
        if rsi_value > self.config.rsi_overbought || rsi_value < self.config.rsi_oversold {
            0.3 // Extreme RSI values indicate potential reversal - lower score for arbitrage
        } else if (40.0..=60.0).contains(&rsi_value) {
            0.8 // Neutral RSI range - good for arbitrage
        } else {
            0.6 // Moderate RSI values - decent for arbitrage
        }
    }

    /// Calculate volatility-based score
    #[allow(clippy::result_large_err)]
    fn calculate_volatility_score(
        &self,
        price_series: &crate::services::market_analysis::PriceSeries,
    ) -> ArbitrageResult<f64> {
        if price_series.data_points.len() < 10 {
            return Ok(0.5); // Neutral score for insufficient data
        }

        let prices: Vec<f64> = price_series.data_points.iter().map(|p| p.price).collect();
        let volatility = crate::services::market_analysis::MathUtils::standard_deviation(&prices)?;

        // Calculate volatility as percentage of mean price
        let mean_price = prices.iter().sum::<f64>() / prices.len() as f64;
        let volatility_percentage = volatility / mean_price;

        if volatility_percentage > self.config.volatility_threshold {
            Ok(0.2) // High volatility - risky for arbitrage
        } else if volatility_percentage < self.config.volatility_threshold * 0.5 {
            Ok(0.9) // Low volatility - good for arbitrage
        } else {
            Ok(0.6) // Moderate volatility - acceptable for arbitrage
        }
    }

    /// Calculate price correlation score between exchanges
    async fn calculate_correlation_score(
        &self,
        exchange1: &str,
        exchange2: &str,
        pair: &str,
    ) -> ArbitrageResult<f64> {
        let series1 = self
            .market_analysis_service
            .get_price_series(exchange1, pair);
        let series2 = self
            .market_analysis_service
            .get_price_series(exchange2, pair);

        if let (Some(s1), Some(s2)) = (series1, series2) {
            let prices1 = s1.price_values();
            let prices2 = s2.price_values();

            // Ensure we have enough data and same length
            let min_length = std::cmp::min(prices1.len(), prices2.len());
            if min_length < 10 {
                return Ok(0.5); // Neutral score for insufficient data
            }

            let correlation = crate::services::market_analysis::MathUtils::price_correlation(
                &prices1[..min_length],
                &prices2[..min_length],
            )?;

            // Convert correlation to score
            if correlation >= self.config.correlation_threshold {
                Ok(0.9) // High correlation - good for arbitrage
            } else if correlation >= 0.5 {
                Ok(0.6) // Moderate correlation - acceptable
            } else {
                Ok(0.2) // Low correlation - risky for arbitrage
            }
        } else {
            Ok(0.5) // Neutral score if no data available
        }
    }

    /// Calculate base arbitrage confidence score
    fn calculate_arbitrage_confidence(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        let rate_diff = opportunity.rate_difference;

        if rate_diff >= self.config.threshold * 5.0 {
            0.9 // Very high rate difference
        } else if rate_diff >= self.config.threshold * 3.0 {
            0.8 // High rate difference
        } else if rate_diff >= self.config.threshold * 2.0 {
            0.7 // Good rate difference
        } else {
            0.6 // Minimal rate difference
        }
    }

    /// Combine arbitrage and technical confidence scores
    fn combine_confidence_scores(&self, arbitrage_score: f64, technical_score: f64) -> f64 {
        let arbitrage_weight = 1.0 - self.config.technical_confirmation_weight;
        let technical_weight = self.config.technical_confirmation_weight;

        arbitrage_score * arbitrage_weight + technical_score * technical_weight
    }

    /// Check if opportunity passes user preference filtering
    fn passes_user_preference_filter(
        &self,
        combined_confidence: f64,
        technical_score: f64,
        preferences: &crate::services::user_trading_preferences::UserTradingPreferences,
    ) -> bool {
        // Filter by experience level
        let min_confidence = match preferences.experience_level {
            ExperienceLevel::Beginner => 0.8,     // High confidence required
            ExperienceLevel::Intermediate => 0.6, // Moderate confidence
            ExperienceLevel::Advanced => 0.4,     // Lower confidence acceptable
        };

        if combined_confidence < min_confidence {
            return false;
        }

        // Filter by risk tolerance
        match preferences.risk_tolerance {
            RiskTolerance::Conservative => technical_score >= 0.7 && combined_confidence >= 0.8,
            RiskTolerance::Balanced => technical_score >= 0.5 && combined_confidence >= 0.6,
            RiskTolerance::Aggressive => combined_confidence >= 0.4,
        }
    }

    /// Convert ArbitrageOpportunity to TradingOpportunity
    async fn convert_arbitrage_to_trading_opportunity(
        &self,
        arbitrage_opportunity: ArbitrageOpportunity,
        confidence_score: f64,
        analysis_type: &str,
    ) -> ArbitrageResult<TradingOpportunity> {
        let risk_level = if confidence_score >= 0.8 {
            RiskLevel::Low
        } else if confidence_score >= 0.6 {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        };

        let time_horizon = TimeHorizon::Short; // Arbitrage is typically short-term
        let expected_return = arbitrage_opportunity.rate_difference * 100.0; // Convert to percentage
        let target_price = arbitrage_opportunity
            .long_rate
            .map(|rate| rate * (1.0 + arbitrage_opportunity.rate_difference));

        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut exchanges = Vec::new();
        if let Some(long_ex) = arbitrage_opportunity.long_exchange {
            exchanges.push(long_ex.to_string());
        }
        if let Some(short_ex) = arbitrage_opportunity.short_exchange {
            exchanges.push(short_ex.to_string());
        }

        Ok(TradingOpportunity {
            opportunity_id: arbitrage_opportunity.id,
            opportunity_type: OpportunityType::Arbitrage,
            trading_pair: arbitrage_opportunity.pair,
            exchanges,
            entry_price: arbitrage_opportunity.long_rate.unwrap_or(0.0),
            target_price,
            stop_loss: None, // Arbitrage typically doesn't use stop losses
            confidence_score,
            risk_level,
            expected_return,
            time_horizon,
            indicators_used: vec!["funding_rate".to_string(), analysis_type.to_string()],
            analysis_data: serde_json::json!({
                "arbitrage_type": "funding_rate",
                "rate_difference": arbitrage_opportunity.rate_difference,
                "technical_analysis_enabled": self.config.enable_technical_analysis,
                "analysis_type": analysis_type
            }),
            created_at: now,
            expires_at: Some(now + 300_000), // 5 minutes expiration for arbitrage
        })
    }

    pub fn get_config(&self) -> &EnhancedOpportunityServiceConfig {
        &self.config
    }
}
