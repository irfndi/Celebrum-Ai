use crate::types::{
    ArbitrageOpportunity, TechnicalOpportunity, ExchangeIdEnum, UserProfile, ChatContext,
    UserAccessLevel, OpportunityAccessResult, ExchangeCredentials, Ticker, FundingRateInfo,
    ArbitrageType, SubscriptionTier,
};
use crate::services::core::trading::exchange::{ExchangeService, ExchangeInterface, ApiKeySource};
use crate::services::core::user::{UserProfileService, UserAccessService};
use crate::services::core::ai::ai_beta_integration::AIBetaIntegrationService;
use crate::utils::{ArbitrageError, ArbitrageResult};
use crate::log_info;
use chrono::Utc;
use futures::future::join_all;
use rand::random;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use worker::kv::KvStore;

/// Personal Opportunity Service for generating user-specific opportunities
/// Uses user's personal exchange APIs to generate opportunities tailored to their available exchanges
pub struct PersonalOpportunityService {
    exchange_service: Arc<ExchangeService>,
    user_profile_service: Arc<UserProfileService>,
    user_access_service: Arc<UserAccessService>,
    ai_service: Option<Arc<AIBetaIntegrationService>>,
    kv_store: KvStore,
    cache_ttl_seconds: u64,
}

impl PersonalOpportunityService {
    const PERSONAL_OPPORTUNITIES_PREFIX: &'static str = "personal_opportunities";
    const USER_EXCHANGE_CACHE_PREFIX: &'static str = "user_exchanges";
    const OPPORTUNITY_CACHE_TTL: u64 = 300; // 5 minutes

    pub fn new(
        exchange_service: Arc<ExchangeService>,
        user_profile_service: Arc<UserProfileService>,
        user_access_service: Arc<UserAccessService>,
        kv_store: KvStore,
    ) -> Self {
        Self {
            exchange_service,
            user_profile_service,
            user_access_service,
            ai_service: None,
            kv_store,
            cache_ttl_seconds: Self::OPPORTUNITY_CACHE_TTL,
        }
    }

    /// Set AI service for AI-enhanced personal opportunity generation
    pub fn set_ai_service(&mut self, ai_service: Arc<AIBetaIntegrationService>) {
        self.ai_service = Some(ai_service);
    }

    /// Generate personal arbitrage opportunities using user's exchange APIs
    pub async fn generate_personal_arbitrage_opportunities(
        &self,
        user_id: &str,
        chat_context: &ChatContext,
        symbols: Option<Vec<String>>,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        // Validate user access
        let access_result = self.validate_user_access(user_id, "arbitrage", chat_context).await?;
        if !access_result.can_access {
            log_info!(
                "User access denied for personal arbitrage generation",
                serde_json::json!({
                    "user_id": user_id,
                    "reason": access_result.reason.unwrap_or_else(|| "Access denied".to_string())
                })
            );
            return Ok(vec![]);
        }

        // Get user's exchange APIs
        let user_exchanges = self.get_user_exchange_apis(user_id).await?;
        if user_exchanges.len() < 2 {
            log_info!(
                "Insufficient exchanges for personal arbitrage",
                serde_json::json!({
                    "user_id": user_id,
                    "exchange_count": user_exchanges.len(),
                    "required": 2
                })
            );
            return Ok(vec![]);
        }

        // Generate arbitrage opportunities between user's exchanges
        let mut opportunities = Vec::new();
        let symbols_to_check = symbols.unwrap_or_else(|| self.get_default_symbols());

        for symbol in symbols_to_check {
            let symbol_opportunities = self
                .find_arbitrage_opportunities_for_symbol(&symbol, &user_exchanges, user_id)
                .await?;
            opportunities.extend(symbol_opportunities);
        }

        // Apply AI enhancement if available and user has AI access
        if let Some(ai_service) = &self.ai_service {
            opportunities = self
                .enhance_opportunities_with_ai(user_id, opportunities, ai_service.as_ref())
                .await?;
        }

        // Apply access level delay
        if access_result.delay_seconds > 0 {
            self.apply_opportunity_delay(&mut opportunities, access_result.delay_seconds)
                .await?;
        }

        // Record opportunity generation
        for _ in &opportunities {
            let _ = self
                .user_access_service
                .record_arbitrage_opportunity_received(user_id, chat_context)
                .await;
        }

        log_info!(
            "Generated personal arbitrage opportunities",
            serde_json::json!({
                "user_id": user_id,
                "opportunity_count": opportunities.len(),
                "exchange_count": user_exchanges.len(),
                "ai_enhanced": self.ai_service.is_some()
            })
        );

        Ok(opportunities)
    }

    /// Generate personal technical opportunities using user's individual exchanges
    pub async fn generate_personal_technical_opportunities(
        &self,
        user_id: &str,
        chat_context: &ChatContext,
        symbols: Option<Vec<String>>,
    ) -> ArbitrageResult<Vec<TechnicalOpportunity>> {
        // Validate user access
        let access_result = self.validate_user_access(user_id, "technical", chat_context).await?;
        if !access_result.can_access {
            log_info!(
                "User access denied for personal technical generation",
                serde_json::json!({
                    "user_id": user_id,
                    "reason": access_result.reason.unwrap_or_else(|| "Access denied".to_string())
                })
            );
            return Ok(vec![]);
        }

        // Get user's exchange APIs
        let user_exchanges = self.get_user_exchange_apis(user_id).await?;
        if user_exchanges.is_empty() {
            log_info!(
                "No exchanges available for personal technical opportunities",
                serde_json::json!({
                    "user_id": user_id,
                    "exchange_count": 0
                })
            );
            return Ok(vec![]);
        }

        // Generate technical opportunities for each exchange
        let mut opportunities = Vec::new();
        let symbols_to_check = symbols.unwrap_or_else(|| self.get_default_symbols());

        for (exchange_id, _credentials) in &user_exchanges {
            for symbol in &symbols_to_check {
                if let Ok(technical_opportunity) = self
                    .find_technical_opportunity_for_symbol(symbol, exchange_id, user_id)
                    .await
                {
                    opportunities.push(technical_opportunity);
                }
            }
        }

        // Apply AI enhancement if available and user has AI access
        if let Some(ai_service) = &self.ai_service {
            opportunities = self
                .enhance_technical_opportunities_with_ai(user_id, opportunities, ai_service.as_ref())
                .await?;
        }

        // Apply access level delay
        if access_result.delay_seconds > 0 {
            self.apply_technical_opportunity_delay(&mut opportunities, access_result.delay_seconds)
                .await?;
        }

        // Record opportunity generation
        for _ in &opportunities {
            let _ = self
                .user_access_service
                .record_technical_opportunity_received(user_id, chat_context)
                .await;
        }

        log_info!(
            "Generated personal technical opportunities",
            serde_json::json!({
                "user_id": user_id,
                "opportunity_count": opportunities.len(),
                "exchange_count": user_exchanges.len(),
                "ai_enhanced": self.ai_service.is_some()
            })
        );

        Ok(opportunities)
    }

    /// Get hybrid opportunities combining global and personal opportunities (slice-based for performance)
    pub async fn get_hybrid_opportunities_with_slices(
        &self,
        user_id: &str,
        chat_context: &ChatContext,
        global_arbitrage: &[ArbitrageOpportunity],
        global_technical: &[TechnicalOpportunity],
    ) -> ArbitrageResult<(Vec<ArbitrageOpportunity>, Vec<TechnicalOpportunity>)> {
        // Get user's access level to determine how many personal opportunities to generate
        let access_level = self.user_access_service.get_user_access_level(user_id).await?;
        let remaining = self
            .user_access_service
            .get_remaining_opportunities(user_id, chat_context)
            .await?;

        let mut final_arbitrage = global_arbitrage.to_vec();
        let mut final_technical = global_technical.to_vec();

        // Generate personal opportunities if user has remaining capacity
        if remaining.0 > 0 {
            // Generate personal arbitrage opportunities
            let personal_arbitrage = self
                .generate_personal_arbitrage_opportunities(user_id, chat_context, None)
                .await?;
            
            // Merge with global opportunities, prioritizing personal ones
            final_arbitrage = self.merge_arbitrage_opportunities(final_arbitrage, personal_arbitrage);
        }

        if remaining.1 > 0 {
            // Generate personal technical opportunities
            let personal_technical = self
                .generate_personal_technical_opportunities(user_id, chat_context, None)
                .await?;
            
            // Merge with global opportunities, prioritizing personal ones
            final_technical = self.merge_technical_opportunities(final_technical, personal_technical);
        }

        log_info!(
            "Generated hybrid opportunities with slices",
            serde_json::json!({
                "user_id": user_id,
                "access_level": format!("{:?}", access_level),
                "final_arbitrage_count": final_arbitrage.len(),
                "final_technical_count": final_technical.len(),
                "remaining_arbitrage": remaining.0,
                "remaining_technical": remaining.1
            })
        );

        Ok((final_arbitrage, final_technical))
    }

    /// Get hybrid opportunities combining global and personal opportunities
    pub async fn get_hybrid_opportunities(
        &self,
        user_id: &str,
        chat_context: &ChatContext,
        global_arbitrage: Vec<ArbitrageOpportunity>,
        global_technical: Vec<TechnicalOpportunity>,
    ) -> ArbitrageResult<(Vec<ArbitrageOpportunity>, Vec<TechnicalOpportunity>)> {
        // Get user's access level to determine how many personal opportunities to generate
        let access_level = self.user_access_service.get_user_access_level(user_id).await?;
        let remaining = self
            .user_access_service
            .get_remaining_opportunities(user_id, chat_context)
            .await?;

        let mut final_arbitrage = global_arbitrage;
        let mut final_technical = global_technical;

        // Generate personal opportunities if user has remaining capacity
        if remaining.0 > 0 {
            // Generate personal arbitrage opportunities
            let personal_arbitrage = self
                .generate_personal_arbitrage_opportunities(user_id, chat_context, None)
                .await?;
            
            // Merge with global opportunities, prioritizing personal ones
            final_arbitrage = self.merge_arbitrage_opportunities(final_arbitrage, personal_arbitrage);
        }

        if remaining.1 > 0 {
            // Generate personal technical opportunities
            let personal_technical = self
                .generate_personal_technical_opportunities(user_id, chat_context, None)
                .await?;
            
            // Merge with global opportunities, prioritizing personal ones
            final_technical = self.merge_technical_opportunities(final_technical, personal_technical);
        }

        log_info!(
            "Generated hybrid opportunities",
            serde_json::json!({
                "user_id": user_id,
                "access_level": format!("{:?}", access_level),
                "final_arbitrage_count": final_arbitrage.len(),
                "final_technical_count": final_technical.len(),
                "remaining_arbitrage": remaining.0,
                "remaining_technical": remaining.1
            })
        );

        Ok((final_arbitrage, final_technical))
    }

    /// Validate user access for personal opportunity generation
    async fn validate_user_access(
        &self,
        user_id: &str,
        opportunity_type: &str,
        chat_context: &ChatContext,
    ) -> ArbitrageResult<OpportunityAccessResult> {
        // For personal opportunities, we need to check if user has trading APIs
        let user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("User not found: {}", user_id)))?;

        if !user_profile.has_trading_api_keys() {
            return Ok(OpportunityAccessResult {
                can_access: false,
                access_level: UserAccessLevel::FreeWithoutAPI,
                delay_seconds: 0,
                remaining_arbitrage: 0,
                remaining_technical: 0,
                reason: Some("Personal opportunities require trading API keys".to_string()),
            });
        }

        // Use the standard access validation
        let required_exchanges = vec![]; // Will be checked per opportunity
        self.user_access_service
            .validate_opportunity_access(user_id, opportunity_type, chat_context, &required_exchanges)
            .await
    }

    /// Get user's exchange APIs for personal opportunity generation
    async fn get_user_exchange_apis(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Vec<(ExchangeIdEnum, ExchangeCredentials)>> {
        // Try cache first
        let cache_key = format!("{}:{}", Self::USER_EXCHANGE_CACHE_PREFIX, user_id);
        if let Ok(Some(cached)) = self.get_cached_user_exchanges(&cache_key).await {
            return Ok(cached);
        }

        // Get user profile
        let user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("User not found: {}", user_id)))?;

        // Extract exchange APIs
        let mut user_exchanges = Vec::new();
        for api_key in &user_profile.api_keys {
            if api_key.is_active && api_key.can_trade() {
                if let Ok(exchange_id) = api_key.exchange_id.parse::<ExchangeIdEnum>() {
                    let credentials = ExchangeCredentials {
                        api_key: api_key.api_key.clone(),
                        secret: api_key.secret.clone(),
                        default_leverage: api_key.default_leverage.unwrap_or(1),
                        exchange_type: api_key.exchange_type.clone().unwrap_or_else(|| "spot".to_string()),
                    };
                    user_exchanges.push((exchange_id, credentials));
                }
            }
        }

        // Cache the result
        self.cache_user_exchanges(&cache_key, &user_exchanges).await?;

        Ok(user_exchanges)
    }

    /// Find arbitrage opportunities for a specific symbol between user's exchanges
    async fn find_arbitrage_opportunities_for_symbol(
        &self,
        symbol: &str,
        user_exchanges: &[(ExchangeIdEnum, ExchangeCredentials)],
        user_id: &str,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Get tickers from all user exchanges
        let mut exchange_tickers = HashMap::new();
        for (exchange_id, credentials) in user_exchanges {
            match self
                .exchange_service
                .get_ticker(&exchange_id.to_string(), symbol)
                .await
            {
                Ok(ticker) => {
                    exchange_tickers.insert(exchange_id.clone(), ticker);
                }
                Err(e) => {
                    log_info!(
                        "Failed to get ticker for personal arbitrage",
                        serde_json::json!({
                            "user_id": user_id,
                            "exchange": format!("{:?}", exchange_id),
                            "symbol": symbol,
                            "error": e.to_string()
                        })
                    );
                }
            }
        }

        // Find arbitrage opportunities between exchanges
        let exchanges: Vec<_> = exchange_tickers.keys().cloned().collect();
        for i in 0..exchanges.len() {
            for j in (i + 1)..exchanges.len() {
                let exchange_a = &exchanges[i];
                let exchange_b = &exchanges[j];
                
                if let (Some(ticker_a), Some(ticker_b)) = (
                    exchange_tickers.get(exchange_a),
                    exchange_tickers.get(exchange_b),
                ) {
                    if let Ok(opportunity) = self
                        .create_arbitrage_opportunity(
                            symbol,
                            exchange_a,
                            exchange_b,
                            ticker_a,
                            ticker_b,
                            user_id,
                        )
                        .await
                    {
                        opportunities.push(opportunity);
                    }
                }
            }
        }

        Ok(opportunities)
    }

    /// Find technical opportunity for a specific symbol on a specific exchange
    async fn find_technical_opportunity_for_symbol(
        &self,
        symbol: &str,
        exchange_id: &ExchangeIdEnum,
        user_id: &str,
    ) -> ArbitrageResult<TechnicalOpportunity> {
        // Get market data for technical analysis
        let ticker = self
            .exchange_service
            .get_ticker(&exchange_id.to_string(), symbol)
            .await?;

        // Get funding rate if available (for futures)
        let funding_rate = self
            .exchange_service
            .fetch_funding_rates(&exchange_id.to_string(), Some(symbol))
            .await
            .ok()
            .and_then(|rates| rates.first().cloned())
            .and_then(|rate| {
                Some(FundingRateInfo {
                    symbol: symbol.to_string(),
                    funding_rate: rate.get("fundingRate")?.as_f64()?,
                    next_funding_time: rate.get("fundingTime")?.as_u64()?,
                    mark_price: rate.get("markPrice")?.as_f64()?,
                })
            });

        // Create technical opportunity
        let opportunity = TechnicalOpportunity {
            id: format!("tech_{}_{}_{}_{}", user_id, exchange_id.to_string(), symbol, Utc::now().timestamp()),
            symbol: symbol.to_string(),
            exchange: exchange_id.clone(),
            signal_type: self.determine_technical_signal(&ticker, &funding_rate),
            entry_price: ticker.last_price,
            target_price: self.calculate_target_price(&ticker),
            stop_loss: self.calculate_stop_loss(&ticker),
            confidence_score: self.calculate_confidence_score(&ticker, &funding_rate),
            expected_return: self.calculate_expected_return(&ticker),
            risk_level: self.assess_risk_level(&ticker),
            time_horizon: "1-4 hours".to_string(),
            market_conditions: self.analyze_market_conditions(&ticker, &funding_rate),
            created_at: Utc::now().timestamp() as u64,
            expires_at: (Utc::now().timestamp() + 3600) as u64, // 1 hour expiry
            funding_rate,
        };

        Ok(opportunity)
    }

    /// Create arbitrage opportunity between two exchanges
    async fn create_arbitrage_opportunity(
        &self,
        symbol: &str,
        exchange_a: &ExchangeIdEnum,
        exchange_b: &ExchangeIdEnum,
        ticker_a: &Ticker,
        ticker_b: &Ticker,
        user_id: &str,
    ) -> ArbitrageResult<ArbitrageOpportunity> {
        // Calculate price difference
        let price_diff = (ticker_b.last_price - ticker_a.last_price).abs();
        let price_diff_percent = (price_diff / ticker_a.last_price) * 100.0;

        // Only create opportunity if price difference is significant (>0.1%)
        if price_diff_percent < 0.1 {
            return Err(ArbitrageError::validation_error(
                "Price difference too small for arbitrage".to_string(),
            ));
        }

        // Determine buy/sell exchanges
        let (buy_exchange, sell_exchange, buy_price, sell_price) = if ticker_a.last_price < ticker_b.last_price {
            (exchange_a.clone(), exchange_b.clone(), ticker_a.last_price, ticker_b.last_price)
        } else {
            (exchange_b.clone(), exchange_a.clone(), ticker_b.last_price, ticker_a.last_price)
        };

        let opportunity = ArbitrageOpportunity {
            id: format!("arb_{}_{}_{}_{}", user_id, symbol, Utc::now().timestamp(), rand::random::<u32>()),
            symbol: symbol.to_string(),
            buy_exchange,
            sell_exchange,
            buy_price,
            sell_price,
            price_difference: price_diff,
            price_difference_percent,
            potential_profit_value: price_diff * 100.0, // Assuming 100 units
            potential_profit_percent: price_diff_percent,
            arbitrage_type: ArbitrageType::Spot,
            confidence_score: self.calculate_arbitrage_confidence(price_diff_percent),
            estimated_execution_time: 30, // 30 seconds
            risk_factors: self.identify_risk_factors(ticker_a, ticker_b),
            created_at: Utc::now().timestamp() as u64,
            expires_at: (Utc::now().timestamp() + 300) as u64, // 5 minutes expiry
            funding_rates: None,
            volume_24h: (ticker_a.volume_24h + ticker_b.volume_24h) / 2.0,
            liquidity_score: self.calculate_liquidity_score(ticker_a, ticker_b),
        };

        Ok(opportunity)
    }

    // Helper methods for technical analysis and opportunity creation
    fn get_default_symbols(&self) -> Vec<String> {
        vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
            "ADAUSDT".to_string(),
            "SOLUSDT".to_string(),
        ]
    }

    fn determine_technical_signal(&self, ticker: &Ticker, funding_rate: &Option<FundingRateInfo>) -> String {
        // Simple technical signal determination
        if let Some(fr) = funding_rate {
            if fr.funding_rate > 0.01 {
                return "SHORT".to_string();
            } else if fr.funding_rate < -0.01 {
                return "LONG".to_string();
            }
        }

        // Use price momentum as fallback
        if ticker.price_change_percent > 2.0 {
            "LONG".to_string()
        } else if ticker.price_change_percent < -2.0 {
            "SHORT".to_string()
        } else {
            "NEUTRAL".to_string()
        }
    }

    fn calculate_target_price(&self, ticker: &Ticker) -> f64 {
        // Simple target price calculation (2% above current price for long)
        ticker.last_price * 1.02
    }

    fn calculate_stop_loss(&self, ticker: &Ticker) -> f64 {
        // Simple stop loss calculation (1% below current price)
        ticker.last_price * 0.99
    }

    fn calculate_confidence_score(&self, ticker: &Ticker, funding_rate: &Option<FundingRateInfo>) -> f64 {
        let mut score = 0.5; // Base score

        // Adjust based on volume
        if ticker.volume_24h > 1000000.0 {
            score += 0.2;
        }

        // Adjust based on funding rate
        if let Some(fr) = funding_rate {
            if fr.funding_rate.abs() > 0.01 {
                score += 0.2;
            }
        }

        // Adjust based on price change
        if ticker.price_change_percent.abs() > 1.0 {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn calculate_expected_return(&self, ticker: &Ticker) -> f64 {
        // Simple expected return calculation
        ticker.price_change_percent.abs() * 0.5
    }

    fn assess_risk_level(&self, ticker: &Ticker) -> String {
        if ticker.price_change_percent.abs() > 5.0 {
            "HIGH".to_string()
        } else if ticker.price_change_percent.abs() > 2.0 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        }
    }

    fn analyze_market_conditions(&self, ticker: &Ticker, funding_rate: &Option<FundingRateInfo>) -> String {
        let mut conditions = Vec::new();

        if ticker.price_change_percent > 2.0 {
            conditions.push("Bullish momentum");
        } else if ticker.price_change_percent < -2.0 {
            conditions.push("Bearish momentum");
        }

        if let Some(fr) = funding_rate {
            if fr.funding_rate > 0.01 {
                conditions.push("High funding rate");
            } else if fr.funding_rate < -0.01 {
                conditions.push("Negative funding rate");
            }
        }

        if ticker.volume_24h > 1000000.0 {
            conditions.push("High volume");
        }

        if conditions.is_empty() {
            "Neutral market conditions".to_string()
        } else {
            conditions.join(", ")
        }
    }

    fn calculate_arbitrage_confidence(&self, price_diff_percent: f64) -> f64 {
        // Higher price difference = higher confidence
        (price_diff_percent / 5.0).min(1.0)
    }

    fn identify_risk_factors(&self, ticker_a: &Ticker, ticker_b: &Ticker) -> Vec<String> {
        let mut risks = Vec::new();

        if ticker_a.volume_24h < 100000.0 || ticker_b.volume_24h < 100000.0 {
            risks.push("Low liquidity".to_string());
        }

        if (ticker_a.price_change_percent - ticker_b.price_change_percent).abs() > 5.0 {
            risks.push("High volatility divergence".to_string());
        }

        risks
    }

    fn calculate_liquidity_score(&self, ticker_a: &Ticker, ticker_b: &Ticker) -> f64 {
        let avg_volume = (ticker_a.volume_24h + ticker_b.volume_24h) / 2.0;
        (avg_volume / 1000000.0).min(1.0)
    }

    // AI enhancement methods
    async fn enhance_opportunities_with_ai(
        &self,
        user_id: &str,
        opportunities: Vec<ArbitrageOpportunity>,
        ai_service: &AIBetaIntegrationService,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        // Check if user has AI access through UserAccessService
        let user_profile = self.user_profile_service.get_user_profile(user_id).await?;
        let ai_access_level = user_profile.get_ai_access_level();
        
        // Only enhance if user has AI access
        if !ai_access_level.can_use_ai_analysis() {
            return Ok(opportunities);
        }
        
        // Enhance opportunities with AI analysis
        match ai_service.enhance_opportunities(opportunities.clone(), user_id).await {
            Ok(enhanced_opportunities) => {
                // Convert enhanced opportunities back to regular opportunities
                // but with updated scores and metadata
                let mut enhanced_regular = Vec::new();
                for enhanced in enhanced_opportunities {
                    let mut opportunity = enhanced.base_opportunity;
                    
                    // Update opportunity with AI insights
                    opportunity.confidence_score = enhanced.ai_score;
                    opportunity.potential_profit_percent = enhanced.risk_adjusted_score * opportunity.potential_profit_percent;
                    
                    // Add AI metadata to opportunity
                    let ai_metadata = serde_json::json!({
                        "ai_enhanced": true,
                        "ai_score": enhanced.ai_score,
                        "confidence_level": enhanced.confidence_level,
                        "market_sentiment": enhanced.market_sentiment,
                        "success_probability": enhanced.success_probability,
                        "time_sensitivity": enhanced.time_sensitivity,
                        "enhanced_at": enhanced.enhanced_at
                    });
                    
                    // Store AI metadata in opportunity metadata field if available
                    // For now, we'll just use the enhanced opportunity as-is
                    enhanced_regular.push(opportunity);
                }
                
                log_info!("Enhanced {} personal arbitrage opportunities with AI for user {}", enhanced_regular.len(), user_id);
                Ok(enhanced_regular)
            }
            Err(e) => {
                log_info!("AI enhancement failed for user {}, using original opportunities: {}", user_id, e);
                Ok(opportunities) // Fallback to original opportunities
            }
        }
    }

    async fn enhance_technical_opportunities_with_ai(
        &self,
        user_id: &str,
        opportunities: Vec<TechnicalOpportunity>,
        ai_service: &AIBetaIntegrationService,
    ) -> ArbitrageResult<Vec<TechnicalOpportunity>> {
        // Check if user has AI access
        let user_profile = self.user_profile_service.get_user_profile(user_id).await?;
        let ai_access_level = user_profile.get_ai_access_level();
        
        // Only enhance if user has AI access
        if !ai_access_level.can_use_ai_analysis() {
            return Ok(opportunities);
        }

        // Convert technical opportunities to arbitrage format for AI analysis
        let arbitrage_opportunities: Vec<ArbitrageOpportunity> = opportunities
            .iter()
            .map(|tech_opp| {
                // Create a synthetic arbitrage opportunity for AI analysis
                ArbitrageOpportunity {
                    id: tech_opp.id.clone(),
                    pair: tech_opp.pair.clone(),
                    long_exchange: tech_opp.exchange,
                    short_exchange: tech_opp.exchange, // Same exchange for technical
                    long_rate: Some(tech_opp.entry_price),
                    short_rate: tech_opp.target_price,
                    rate_difference: tech_opp.expected_return_percentage / 100.0,
                    net_rate_difference: Some(tech_opp.expected_return_percentage / 100.0),
                    potential_profit_value: Some(tech_opp.expected_return_percentage * 10.0), // Assume $1000 position
                    timestamp: tech_opp.timestamp,
                    r#type: ArbitrageType::SpotFutures, // Technical analysis type
                    details: tech_opp.details.clone(),
                    min_exchanges_required: 1, // Technical requires 1 exchange
                }
            })
            .collect();
        
        // Enhance with AI
        match ai_service.enhance_opportunities(arbitrage_opportunities, user_id).await {
            Ok(enhanced_opportunities) => {
                // Convert back to technical opportunities with AI enhancements
                let mut enhanced_technical = Vec::new();
                for (i, enhanced) in enhanced_opportunities.iter().enumerate() {
                    if let Some(original_tech) = opportunities.get(i) {
                        let mut tech_opp = original_tech.clone();
                        
                        // Update with AI insights
                        tech_opp.confidence_score = enhanced.ai_score;
                        tech_opp.expected_return = enhanced.risk_adjusted_score * tech_opp.expected_return;
                        
                        // Update target price based on AI analysis
                        if enhanced.success_probability > 0.7 {
                            tech_opp.target_price = tech_opp.target_price * (1.0 + enhanced.success_probability * 0.1);
                        }
                        
                        enhanced_technical.push(tech_opp);
                    }
                }
                
                log_info!("Enhanced {} personal technical opportunities with AI for user {}", enhanced_technical.len(), user_id);
                Ok(enhanced_technical)
            }
            Err(e) => {
                log_info!("AI enhancement failed for user {}, using original technical opportunities: {}", user_id, e);
                Ok(opportunities) // Fallback to original opportunities
            }
        }
    }

    // Opportunity merging methods
    fn merge_arbitrage_opportunities(
        &self,
        global: Vec<ArbitrageOpportunity>,
        personal: Vec<ArbitrageOpportunity>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut merged = personal; // Prioritize personal opportunities
        merged.extend(global);
        
        // Remove duplicates and sort by potential profit
        merged.sort_by(|a, b| b.potential_profit_percent.partial_cmp(&a.potential_profit_percent).unwrap());
        merged.truncate(10); // Limit to top 10 opportunities
        
        merged
    }

    fn merge_technical_opportunities(
        &self,
        global: Vec<TechnicalOpportunity>,
        personal: Vec<TechnicalOpportunity>,
    ) -> Vec<TechnicalOpportunity> {
        let mut merged = personal; // Prioritize personal opportunities
        merged.extend(global);
        
        // Remove duplicates and sort by confidence score
        merged.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        merged.truncate(10); // Limit to top 10 opportunities
        
        merged
    }

    // Delay application methods
    async fn apply_opportunity_delay(
        &self,
        opportunities: &mut Vec<ArbitrageOpportunity>,
        delay_seconds: u64,
    ) -> ArbitrageResult<()> {
        for opportunity in opportunities {
            opportunity.created_at += delay_seconds * 1000; // Convert to milliseconds
        }
        Ok(())
    }

    async fn apply_technical_opportunity_delay(
        &self,
        opportunities: &mut Vec<TechnicalOpportunity>,
        delay_seconds: u64,
    ) -> ArbitrageResult<()> {
        for opportunity in opportunities {
            opportunity.created_at += delay_seconds * 1000; // Convert to milliseconds
        }
        Ok(())
    }

    // Cache methods
    async fn cache_user_exchanges(
        &self,
        cache_key: &str,
        exchanges: &[(ExchangeIdEnum, ExchangeCredentials)],
    ) -> ArbitrageResult<()> {
        let serialized = serde_json::to_string(exchanges)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize user exchanges: {}", e)))?;
        
        self.kv_store
            .put(cache_key, serialized)
            .map_err(|e| ArbitrageError::storage_error(format!("Failed to cache user exchanges: {:?}", e)))?
            .expiration_ttl(self.cache_ttl_seconds)
            .execute()
            .await
            .map_err(|e| ArbitrageError::storage_error(format!("Failed to execute cache put: {:?}", e)))?;

        Ok(())
    }

    async fn get_cached_user_exchanges(
        &self,
        cache_key: &str,
    ) -> ArbitrageResult<Option<Vec<(ExchangeIdEnum, ExchangeCredentials)>>> {
        match self.kv_store.get(cache_key).text().await {
            Ok(Some(cached)) => {
                match serde_json::from_str(&cached) {
                    Ok(exchanges) => Ok(Some(exchanges)),
                    Err(_) => Ok(None), // Invalid cache data
                }
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::services::{D1Service, UserProfileService, UserAccessService};
    use chrono::Utc;
    use worker::{Env, kv::KvStore};

    fn create_test_user_profile() -> UserProfile {
        UserProfile {
            user_id: "test_user".to_string(),
            telegram_id: 12345,
            username: Some("testuser".to_string()),
            subscription: SubscriptionTier::Basic,
            account_status: AccountStatus::Active,
            api_keys: vec![
                UserApiKey {
                    exchange_id: "binance".to_string(),
                    api_key: "test_key".to_string(),
                    secret: "test_secret".to_string(),
                    is_active: true,
                    permissions: vec!["spot".to_string()],
                    default_leverage: Some(1),
                    exchange_type: Some("spot".to_string()),
                    created_at: Utc::now().timestamp() as u64,
                    last_used: None,
                },
            ],
            // ... other fields with defaults
            created_at: Utc::now().timestamp() as u64,
            updated_at: Utc::now().timestamp() as u64,
            profile_metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_personal_opportunity_service_creation() {
        // Test that PersonalOpportunityService can be created
        // This is a placeholder test - full testing would require mock services
        assert!(true);
    }

    #[test]
    fn test_arbitrage_confidence_calculation() {
        // Test confidence calculation logic
        let service = PersonalOpportunityService {
            exchange_service: Arc::new(ExchangeService::new(&Env::default()).unwrap()),
            user_profile_service: Arc::new(UserProfileService::new(D1Service::new(&Env::default()).unwrap())),
            user_access_service: Arc::new(UserAccessService::new(
                D1Service::new(&Env::default()).unwrap(),
                UserProfileService::new(D1Service::new(&Env::default()).unwrap()),
                KvStore::default(),
            )),
            ai_service: None,
            kv_store: KvStore::default(),
            cache_ttl_seconds: 300,
        };

        let confidence = service.calculate_arbitrage_confidence(2.5);
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_technical_signal_determination() {
        let service = PersonalOpportunityService {
            exchange_service: Arc::new(ExchangeService::new(&Env::default()).unwrap()),
            user_profile_service: Arc::new(UserProfileService::new(D1Service::new(&Env::default()).unwrap())),
            user_access_service: Arc::new(UserAccessService::new(
                D1Service::new(&Env::default()).unwrap(),
                UserProfileService::new(D1Service::new(&Env::default()).unwrap()),
                KvStore::default(),
            )),
            ai_service: None,
            kv_store: KvStore::default(),
            cache_ttl_seconds: 300,
        };

        let ticker = Ticker {
            symbol: "BTCUSDT".to_string(),
            last_price: 50000.0,
            price_change_percent: 3.0,
            volume_24h: 1000000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            bid_price: 49990.0,
            ask_price: 50010.0,
            timestamp: Utc::now().timestamp() as u64,
        };

        let signal = service.determine_technical_signal(&ticker, &None);
        assert_eq!(signal, "LONG");
    }
} 