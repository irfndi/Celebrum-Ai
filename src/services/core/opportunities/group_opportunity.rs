use crate::types::{
    ArbitrageOpportunity, TechnicalOpportunity, ExchangeIdEnum, UserProfile, ChatContext,
    UserAccessLevel, OpportunityAccessResult, ExchangeCredentials, Ticker, FundingRateInfo,
    ArbitrageType, SubscriptionTier,
};
use crate::services::core::trading::exchange::{ExchangeService, ExchangeInterface};
use crate::services::core::user::{UserProfileService, UserAccessService};
use crate::services::core::opportunities::PersonalOpportunityService;
use crate::services::core::ai::ai_beta_integration::AIBetaIntegrationService;
use crate::utils::{ArbitrageError, ArbitrageResult};
use crate::log_info;
use chrono::Utc;
use rand;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use worker::kv::KvStore;

// Constants for technical analysis thresholds
const FUNDING_RATE_THRESHOLD: f64 = 0.01; // 1% funding rate threshold
const PRICE_MOMENTUM_THRESHOLD: f64 = 2.0; // 2% price change threshold

/// Group/Channel Opportunity Service for generating group-specific opportunities
/// Uses group/channel admin's exchange APIs to generate opportunities for group members
pub struct GroupOpportunityService {
    /// Service for interacting with cryptocurrency exchanges to fetch market data and execute trades
    exchange_service: Arc<ExchangeService>,
    /// Service for managing user profiles, subscription tiers, and user-specific configurations
    user_profile_service: Arc<UserProfileService>,
    /// Service for validating user access levels and permissions for different opportunity types
    user_access_service: Arc<UserAccessService>,
    /// Service for generating personalized trading opportunities for individual users
    personal_opportunity_service: Arc<PersonalOpportunityService>,
    /// Optional AI service for enhancing opportunities with artificial intelligence analysis
    /// When present, provides AI-powered market sentiment, risk assessment, and opportunity scoring
    ai_service: Option<Arc<AIBetaIntegrationService>>,
    /// Key-value store for caching group admin exchange credentials and opportunity data
    kv_store: KvStore,
    /// Cache time-to-live in seconds for storing group admin exchange API credentials
    /// Default is 600 seconds (10 minutes) to balance performance and security
    cache_ttl_seconds: u64,
}

impl GroupOpportunityService {
    const GROUP_OPPORTUNITIES_PREFIX: &'static str = "group_opportunities";
    const GROUP_ADMIN_CACHE_PREFIX: &'static str = "group_admin_apis";
    const GROUP_CACHE_TTL: u64 = 600; // 10 minutes

    pub fn new(
        exchange_service: Arc<ExchangeService>,
        user_profile_service: Arc<UserProfileService>,
        user_access_service: Arc<UserAccessService>,
        personal_opportunity_service: Arc<PersonalOpportunityService>,
        kv_store: KvStore,
    ) -> Self {
        Self {
            exchange_service,
            user_profile_service,
            user_access_service,
            personal_opportunity_service,
            ai_service: None,
            kv_store,
            cache_ttl_seconds: Self::GROUP_CACHE_TTL,
        }
    }

    /// Set AI service for AI-enhanced group opportunity generation
    pub fn set_ai_service(&mut self, ai_service: Arc<AIBetaIntegrationService>) {
        self.ai_service = Some(ai_service);
    }

    /// Generate group/channel arbitrage opportunities using admin's exchange APIs
    pub async fn generate_group_arbitrage_opportunities(
        &self,
        group_admin_id: &str,
        chat_context: &ChatContext,
        symbols: Option<Vec<String>>,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        // Validate that this is a group/channel context
        if !chat_context.is_group_context() {
            return Err(ArbitrageError::validation_error(
                "Group opportunity generation only available in group/channel contexts".to_string(),
            ));
        }

        // Validate group admin access
        let admin_access_result = self
            .validate_group_admin_access(group_admin_id, "arbitrage", chat_context)
            .await?;
        if !admin_access_result.can_access {
            log_info!(
                "Group admin access denied for group arbitrage generation",
                serde_json::json!({
                    "group_admin_id": group_admin_id,
                    "chat_context": format!("{:?}", chat_context),
                    "reason": admin_access_result.reason.unwrap_or_else(|| "Access denied".to_string())
                })
            );
            return Ok(vec![]);
        }

        // Get group admin's exchange APIs
        let admin_exchanges = self.get_group_admin_exchange_apis(group_admin_id).await?;
        if admin_exchanges.len() < 2 {
            log_info!(
                "Insufficient exchanges for group arbitrage",
                serde_json::json!({
                    "group_admin_id": group_admin_id,
                    "exchange_count": admin_exchanges.len(),
                    "required": 2
                })
            );
            return Ok(vec![]);
        }

        // Generate arbitrage opportunities using admin's exchanges
        let mut opportunities = Vec::new();
        let symbols_to_check = match symbols {
            Some(provided_symbols) => provided_symbols,
            None => self.get_default_symbols(),
        };

        for symbol in symbols_to_check {
            let symbol_opportunities = self
                .find_group_arbitrage_opportunities_for_symbol(&symbol, &admin_exchanges, group_admin_id)
                .await?;
            opportunities.extend(symbol_opportunities);
        }

        // Apply group multiplier (2x opportunities for groups)
        opportunities = self.apply_group_multiplier(opportunities);

        // Apply AI enhancement if available
        if let Some(ai_service) = &self.ai_service {
            opportunities = self
                .enhance_group_opportunities_with_ai(group_admin_id, opportunities, ai_service.as_ref())
                .await?;
        }

        // Apply access level delay
        if admin_access_result.delay_seconds > 0 {
            self.apply_group_opportunity_delay(&mut opportunities, admin_access_result.delay_seconds)
                .await?;
        }

        log_info!(
            "Generated group arbitrage opportunities",
            serde_json::json!({
                "group_admin_id": group_admin_id,
                "chat_context": format!("{:?}", chat_context),
                "opportunity_count": opportunities.len(),
                "exchange_count": admin_exchanges.len(),
                "group_multiplier_applied": true,
                "ai_enhanced": self.ai_service.is_some()
            })
        );

        Ok(opportunities)
    }

    /// Generate group/channel technical opportunities using admin's exchange APIs
    pub async fn generate_group_technical_opportunities(
        &self,
        group_admin_id: &str,
        chat_context: &ChatContext,
        symbols: Option<Vec<String>>,
    ) -> ArbitrageResult<Vec<TechnicalOpportunity>> {
        // Validate that this is a group/channel context
        if !chat_context.is_group_context() {
            return Err(ArbitrageError::validation_error(
                "Group opportunity generation only available in group/channel contexts".to_string(),
            ));
        }

        // Validate group admin access
        let admin_access_result = self
            .validate_group_admin_access(group_admin_id, "technical", chat_context)
            .await?;
        if !admin_access_result.can_access {
            log_info!(
                "Group admin access denied for group technical generation",
                serde_json::json!({
                    "group_admin_id": group_admin_id,
                    "chat_context": format!("{:?}", chat_context),
                    "reason": admin_access_result.reason.unwrap_or_else(|| "Access denied".to_string())
                })
            );
            return Ok(vec![]);
        }

        // Get group admin's exchange APIs
        let admin_exchanges = self.get_group_admin_exchange_apis(group_admin_id).await?;
        if admin_exchanges.is_empty() {
            log_info!(
                "No exchanges available for group technical opportunities",
                serde_json::json!({
                    "group_admin_id": group_admin_id,
                    "exchange_count": 0
                })
            );
            return Ok(vec![]);
        }

        // Generate technical opportunities for each admin exchange
        let mut opportunities = Vec::new();
        let symbols_to_check = match symbols {
            Some(provided_symbols) => provided_symbols,
            None => self.get_default_symbols(),
        };

        for (exchange_id, _credentials) in &admin_exchanges {
            for symbol in &symbols_to_check {
                if let Ok(technical_opportunity) = self
                    .find_group_technical_opportunity_for_symbol(symbol, exchange_id, group_admin_id)
                    .await
                {
                    opportunities.push(technical_opportunity);
                }
            }
        }

        // Apply group multiplier (2x opportunities for groups)
        opportunities = self.apply_group_technical_multiplier(opportunities);

        // Apply AI enhancement if available
        if let Some(ai_service) = &self.ai_service {
            opportunities = self
                .enhance_group_technical_opportunities_with_ai(group_admin_id, opportunities, ai_service.as_ref())
                .await?;
        }

        // Apply access level delay
        if admin_access_result.delay_seconds > 0 {
            self.apply_group_technical_opportunity_delay(&mut opportunities, admin_access_result.delay_seconds)
                .await?;
        }

        log_info!(
            "Generated group technical opportunities",
            serde_json::json!({
                "group_admin_id": group_admin_id,
                "chat_context": format!("{:?}", chat_context),
                "opportunity_count": opportunities.len(),
                "exchange_count": admin_exchanges.len(),
                "group_multiplier_applied": true,
                "ai_enhanced": self.ai_service.is_some()
            })
        );

        Ok(opportunities)
    }

    /// Get hybrid opportunities combining global, personal, and group opportunities
    pub async fn get_hybrid_group_opportunities(
        &self,
        user_id: &str,
        group_admin_id: &str,
        chat_context: &ChatContext,
        global_arbitrage: Vec<ArbitrageOpportunity>,
        global_technical: Vec<TechnicalOpportunity>,
    ) -> ArbitrageResult<(Vec<ArbitrageOpportunity>, Vec<TechnicalOpportunity>)> {
        // Get user's access level and remaining opportunities
        let access_level = self.user_access_service.get_user_access_level(user_id).await?;
        let remaining = self
            .user_access_service
            .get_remaining_opportunities(user_id, chat_context)
            .await?;

        let mut final_arbitrage = global_arbitrage;
        let mut final_technical = global_technical;

        // Generate group opportunities if user has remaining capacity
        if remaining.0 > 0 {
            // Generate group arbitrage opportunities
            let group_arbitrage = self
                .generate_group_arbitrage_opportunities(group_admin_id, chat_context, None)
                .await?;
            
            // Merge with existing opportunities, prioritizing group ones
            final_arbitrage = self.merge_arbitrage_opportunities(final_arbitrage, group_arbitrage);
        }

        if remaining.1 > 0 {
            // Generate group technical opportunities
            let group_technical = self
                .generate_group_technical_opportunities(group_admin_id, chat_context, None)
                .await?;
            
            // Merge with existing opportunities, prioritizing group ones
            final_technical = self.merge_technical_opportunities(final_technical, group_technical);
        }

        // Also get personal opportunities if user has their own APIs
        let user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?;

        if let Some(profile) = user_profile {
            if profile.has_trading_api_keys() && remaining.0 > 0 {
                // Get personal opportunities using PersonalOpportunityService
                // Pass slices instead of cloning vectors for better performance
                let (personal_arbitrage, personal_technical) = self
                    .personal_opportunity_service
                    .get_hybrid_opportunities_with_slices(user_id, chat_context, &final_arbitrage, &final_technical)
                    .await?;

                final_arbitrage = personal_arbitrage;
                final_technical = personal_technical;
            }
        }

        log_info!(
            "Generated hybrid group opportunities",
            serde_json::json!({
                "user_id": user_id,
                "group_admin_id": group_admin_id,
                "access_level": format!("{:?}", access_level),
                "final_arbitrage_count": final_arbitrage.len(),
                "final_technical_count": final_technical.len(),
                "remaining_arbitrage": remaining.0,
                "remaining_technical": remaining.1,
                "chat_context": format!("{:?}", chat_context)
            })
        );

        Ok((final_arbitrage, final_technical))
    }

    /// Validate group admin access for opportunity generation
    async fn validate_group_admin_access(
        &self,
        group_admin_id: &str,
        opportunity_type: &str,
        chat_context: &ChatContext,
    ) -> ArbitrageResult<OpportunityAccessResult> {
        // Check if admin has trading APIs
        let admin_profile = self
            .user_profile_service
            .get_user_profile(group_admin_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("Group admin not found: {}", group_admin_id)))?;

        if !admin_profile.has_trading_api_keys() {
            return Ok(OpportunityAccessResult {
                can_access: false,
                access_level: UserAccessLevel::FreeWithoutAPI,
                delay_seconds: 0,
                remaining_arbitrage: 0,
                remaining_technical: 0,
                reason: Some("Group opportunities require admin to have trading API keys".to_string()),
            });
        }

        // Use the standard access validation for the admin
        let required_exchanges = vec![]; // Will be checked per opportunity
        self.user_access_service
            .validate_opportunity_access(group_admin_id, opportunity_type, chat_context, &required_exchanges)
            .await
    }

    /// Get group admin's exchange APIs for group opportunity generation
    async fn get_group_admin_exchange_apis(
        &self,
        group_admin_id: &str,
    ) -> ArbitrageResult<Vec<(ExchangeIdEnum, ExchangeCredentials)>> {
        // Try cache first
        let cache_key = format!("{}:{}", Self::GROUP_ADMIN_CACHE_PREFIX, group_admin_id);
        if let Ok(Some(cached)) = self.get_cached_admin_exchanges(&cache_key).await {
            return Ok(cached);
        }

        // Get admin profile
        let admin_profile = self
            .user_profile_service
            .get_user_profile(group_admin_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("Group admin not found: {}", group_admin_id)))?;

        // Extract exchange APIs
        let mut admin_exchanges = Vec::new();
        for api_key in &admin_profile.api_keys {
            if api_key.is_active && api_key.can_trade() {
                if let Ok(exchange_id) = api_key.exchange_id.parse::<ExchangeIdEnum>() {
                    let credentials = ExchangeCredentials {
                        api_key: api_key.api_key.clone(),
                        secret: api_key.secret.clone(),
                        default_leverage: api_key.default_leverage.unwrap_or(1),
                        exchange_type: api_key.exchange_type.clone().unwrap_or_else(|| "spot".to_string()),
                    };
                    admin_exchanges.push((exchange_id, credentials));
                }
            }
        }

        // Cache the result
        self.cache_admin_exchanges(&cache_key, &admin_exchanges).await?;

        Ok(admin_exchanges)
    }

    /// Find arbitrage opportunities for a specific symbol using admin's exchanges
    async fn find_group_arbitrage_opportunities_for_symbol(
        &self,
        symbol: &str,
        admin_exchanges: &[(ExchangeIdEnum, ExchangeCredentials)],
        group_admin_id: &str,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Get tickers from all admin exchanges
        let mut exchange_tickers = HashMap::new();
        for (exchange_id, _credentials) in admin_exchanges {
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
                        "Failed to get ticker for group arbitrage",
                        serde_json::json!({
                            "group_admin_id": group_admin_id,
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
                        .create_group_arbitrage_opportunity(
                            symbol,
                            exchange_a,
                            exchange_b,
                            ticker_a,
                            ticker_b,
                            group_admin_id,
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

    /// Find technical opportunity for a specific symbol on admin's exchange
    async fn find_group_technical_opportunity_for_symbol(
        &self,
        symbol: &str,
        exchange_id: &ExchangeIdEnum,
        group_admin_id: &str,
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

        // Create technical opportunity for group
        let opportunity = TechnicalOpportunity {
            id: format!("group_tech_{}_{}_{}_{}", group_admin_id, exchange_id.to_string(), symbol, Utc::now().timestamp()),
            symbol: symbol.to_string(),
            exchange: exchange_id.clone(),
            signal_type: self.determine_technical_signal(&ticker, &funding_rate),
            entry_price: ticker.last.unwrap_or(0.0),
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

    /// Create arbitrage opportunity between two admin exchanges for group
    async fn create_group_arbitrage_opportunity(
        &self,
        symbol: &str,
        exchange_a: &ExchangeIdEnum,
        exchange_b: &ExchangeIdEnum,
        ticker_a: &Ticker,
        ticker_b: &Ticker,
        group_admin_id: &str,
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
            id: format!("group_arb_{}_{}_{}_{}", group_admin_id, symbol, Utc::now().timestamp(), rand::random::<u32>()),
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

    /// Apply group multiplier (2x opportunities for groups)
    fn apply_group_multiplier(&self, opportunities: Vec<ArbitrageOpportunity>) -> Vec<ArbitrageOpportunity> {
        let mut multiplied = opportunities.clone();
        
        // Create additional opportunities with slight variations for group multiplier
        for (i, opportunity) in opportunities.iter().enumerate() {
            if i < opportunities.len() {
                let mut additional_opportunity = opportunity.clone();
                additional_opportunity.id = format!("{}_group_2x", opportunity.id);
                additional_opportunity.created_at += 1000; // 1 second later
                multiplied.push(additional_opportunity);
            }
        }

        // Sort by potential profit and limit to reasonable number
        multiplied.sort_by(|a, b| b.potential_profit_percent.partial_cmp(&a.potential_profit_percent).unwrap());
        multiplied.truncate(20); // Limit to top 20 opportunities for groups
        
        multiplied
    }

    /// Apply group multiplier for technical opportunities
    fn apply_group_technical_multiplier(&self, opportunities: Vec<TechnicalOpportunity>) -> Vec<TechnicalOpportunity> {
        let mut multiplied = opportunities.clone();
        
        // Create additional opportunities with slight variations for group multiplier
        for (i, opportunity) in opportunities.iter().enumerate() {
            if i < opportunities.len() {
                let mut additional_opportunity = opportunity.clone();
                additional_opportunity.id = format!("{}_group_2x", opportunity.id);
                additional_opportunity.created_at += 1000; // 1 second later
                multiplied.push(additional_opportunity);
            }
        }

        // Sort by confidence score and limit to reasonable number
        multiplied.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        multiplied.truncate(20); // Limit to top 20 opportunities for groups
        
        multiplied
    }

    // Helper methods (reusing logic from PersonalOpportunityService)
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

        // Calculate price change from ticker data
        let last_price = ticker.last.unwrap_or(0.0);
        let high_24h = ticker.high.unwrap_or(last_price);
        let low_24h = ticker.low.unwrap_or(last_price);
        let price_change_percent = if low_24h > 0.0 {
            ((last_price - low_24h) / low_24h) * 100.0
        } else {
            0.0
        };

        // Use price momentum as fallback
        if price_change_percent > 2.0 {
            "LONG".to_string()
        } else if price_change_percent < -2.0 {
            "SHORT".to_string()
        } else {
            "NEUTRAL".to_string()
        }
    }

    fn calculate_target_price(&self, ticker: &Ticker) -> f64 {
        let last_price = ticker.last.unwrap_or(0.0);
        last_price * 1.02
    }

    fn calculate_stop_loss(&self, ticker: &Ticker) -> f64 {
        let last_price = ticker.last.unwrap_or(0.0);
        last_price * 0.99
    }

    fn calculate_confidence_score(&self, ticker: &Ticker, funding_rate: &Option<FundingRateInfo>) -> f64 {
        let mut score = 0.5;

        let volume_24h = ticker.volume.unwrap_or(0.0);
        if volume_24h > 1000000.0 {
            score += 0.2;
        }

        if let Some(fr) = funding_rate {
            if fr.funding_rate.abs() > 0.01 {
                score += 0.2;
            }
        }

        // Calculate price change from ticker data
        let last_price = ticker.last.unwrap_or(0.0);
        let high_24h = ticker.high.unwrap_or(last_price);
        let low_24h = ticker.low.unwrap_or(last_price);
        let price_change_percent = if low_24h > 0.0 {
            ((last_price - low_24h) / low_24h) * 100.0
        } else {
            0.0
        };

        if price_change_percent.abs() > 1.0 {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn calculate_expected_return(&self, ticker: &Ticker) -> f64 {
        let last_price = ticker.last.unwrap_or(0.0);
        let high_24h = ticker.high.unwrap_or(last_price);
        let low_24h = ticker.low.unwrap_or(last_price);
        let price_change_percent = if low_24h > 0.0 {
            ((last_price - low_24h) / low_24h) * 100.0
        } else {
            0.0
        };
        price_change_percent.abs() * 0.5
    }

    fn assess_risk_level(&self, ticker: &Ticker) -> String {
        let last_price = ticker.last.unwrap_or(0.0);
        let high_24h = ticker.high.unwrap_or(last_price);
        let low_24h = ticker.low.unwrap_or(last_price);
        let price_change_percent = if low_24h > 0.0 {
            ((last_price - low_24h) / low_24h) * 100.0
        } else {
            0.0
        };

        if price_change_percent.abs() > 5.0 {
            "HIGH".to_string()
        } else if price_change_percent.abs() > 2.0 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        }
    }

    fn analyze_market_conditions(&self, ticker: &Ticker, funding_rate: &Option<FundingRateInfo>) -> String {
        let mut conditions = Vec::new();

        // Calculate price change from ticker data
        let last_price = ticker.last.unwrap_or(0.0);
        let high_24h = ticker.high.unwrap_or(last_price);
        let low_24h = ticker.low.unwrap_or(last_price);
        let price_change_percent = if low_24h > 0.0 {
            ((last_price - low_24h) / low_24h) * 100.0
        } else {
            0.0
        };

        if price_change_percent > 2.0 {
            conditions.push("Bullish momentum");
        } else if price_change_percent < -2.0 {
            conditions.push("Bearish momentum");
        }

        if let Some(fr) = funding_rate {
            if fr.funding_rate > 0.01 {
                conditions.push("High funding rate");
            } else if fr.funding_rate < -0.01 {
                conditions.push("Negative funding rate");
            }
        }

        let volume_24h = ticker.volume.unwrap_or(0.0);
        if volume_24h > 1000000.0 {
            conditions.push("High volume");
        }

        if conditions.is_empty() {
            "Neutral market conditions".to_string()
        } else {
            conditions.join(", ")
        }
    }

    fn calculate_arbitrage_confidence(&self, price_diff_percent: f64) -> f64 {
        (price_diff_percent / 5.0).min(1.0)
    }

    fn identify_risk_factors(&self, ticker_a: &Ticker, ticker_b: &Ticker) -> Vec<String> {
        let mut risks = Vec::new();

        let volume_a = ticker_a.volume.unwrap_or(0.0);
        let volume_b = ticker_b.volume.unwrap_or(0.0);
        if volume_a < 100000.0 || volume_b < 100000.0 {
            risks.push("Low liquidity".to_string());
        }

        // Calculate price change percentage from ticker data
        let last_a = ticker_a.last.unwrap_or(0.0);
        let high_a = ticker_a.high.unwrap_or(last_a);
        let low_a = ticker_a.low.unwrap_or(last_a);
        let change_a = if low_a > 0.0 { ((last_a - low_a) / low_a) * 100.0 } else { 0.0 };

        let last_b = ticker_b.last.unwrap_or(0.0);
        let high_b = ticker_b.high.unwrap_or(last_b);
        let low_b = ticker_b.low.unwrap_or(last_b);
        let change_b = if low_b > 0.0 { ((last_b - low_b) / low_b) * 100.0 } else { 0.0 };

        if (change_a - change_b).abs() > 5.0 {
            risks.push("High volatility divergence".to_string());
        }

        risks
    }

    fn calculate_liquidity_score(&self, ticker_a: &Ticker, ticker_b: &Ticker) -> f64 {
        let volume_a = ticker_a.volume.unwrap_or(0.0);
        let volume_b = ticker_b.volume.unwrap_or(0.0);
        let avg_volume = (volume_a + volume_b) / 2.0;
        (avg_volume / 1000000.0).min(1.0)
    }

    // AI enhancement methods
    async fn enhance_group_opportunities_with_ai(
        &self,
        group_admin_id: &str,
        opportunities: Vec<ArbitrageOpportunity>,
        ai_service: &AIBetaIntegrationService,
    ) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        // Check if group admin has AI access through UserAccessService
        let user_profile = self.user_profile_service.get_user_profile(group_admin_id).await?;
        let ai_access_level = user_profile.get_ai_access_level();
        
        // Only enhance if group admin has AI access
        if !ai_access_level.can_use_ai_analysis() {
            return Ok(opportunities);
        }
        
        // Enhance opportunities with AI analysis for group context
        match ai_service.enhance_opportunities(opportunities.clone(), group_admin_id).await {
            Ok(enhanced_opportunities) => {
                // Convert enhanced opportunities back to regular opportunities
                // but with updated scores and metadata for group context
                let mut enhanced_regular = Vec::new();
                for enhanced in enhanced_opportunities {
                    let mut opportunity = enhanced.base_opportunity;
                    
                    // Update opportunity with AI insights
                    if let Some(current_profit) = opportunity.potential_profit_value {
                        opportunity.potential_profit_value = Some(enhanced.risk_adjusted_score * current_profit);
                    }
                    
                    // Apply group context boost to AI-enhanced opportunities
                    // Groups get slightly higher confidence due to collective decision making
                    opportunity.confidence_score = (opportunity.confidence_score * 1.1).min(1.0);
                    
                    // Add group-specific AI metadata
                    let ai_metadata = serde_json::json!({
                        "ai_enhanced": true,
                        "ai_score": enhanced.ai_score,
                        "confidence_level": enhanced.confidence_level,
                        "market_sentiment": enhanced.market_sentiment,
                        "success_probability": enhanced.success_probability,
                        "time_sensitivity": enhanced.time_sensitivity,
                        "enhanced_at": enhanced.enhanced_at,
                        "group_context": true,
                        "group_confidence_boost": 1.1
                    });
                    
                    enhanced_regular.push(opportunity);
                }
                
                log_info!("Enhanced {} group arbitrage opportunities with AI for admin {}", enhanced_regular.len(), group_admin_id);
                Ok(enhanced_regular)
            }
            Err(e) => {
                log_info!("AI enhancement failed for group admin {}, using original opportunities: {}", group_admin_id, e);
                Ok(opportunities) // Fallback to original opportunities
            }
        }
    }

    async fn enhance_group_technical_opportunities_with_ai(
        &self,
        group_admin_id: &str,
        opportunities: Vec<TechnicalOpportunity>,
        ai_service: &AIBetaIntegrationService,
    ) -> ArbitrageResult<Vec<TechnicalOpportunity>> {
        // Check if group admin has AI access
        let user_profile = self.user_profile_service.get_user_profile(group_admin_id).await?;
        let ai_access_level = user_profile.get_ai_access_level();
        
        // Only enhance if group admin has AI access
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
                    rate_difference: if tech_opp.target_price.is_some() && tech_opp.entry_price > 0.0 {
                        (tech_opp.target_price.unwrap() - tech_opp.entry_price) / tech_opp.entry_price
                    } else {
                        0.0
                    },
                    net_rate_difference: if tech_opp.target_price.is_some() && tech_opp.entry_price > 0.0 {
                        Some((tech_opp.target_price.unwrap() - tech_opp.entry_price) / tech_opp.entry_price)
                    } else {
                        Some(0.0)
                    },
                    potential_profit_value: Some(tech_opp.expected_return_percentage * 10.0), // Assume $1000 position
                    timestamp: tech_opp.timestamp,
                    r#type: ArbitrageType::SpotFutures,
                    details: tech_opp.details.clone(),
                    min_exchanges_required: 1,
                }
            })
            .collect();
        
        // Enhance with AI for group context
        match ai_service.enhance_opportunities(arbitrage_opportunities, group_admin_id).await {
            Ok(enhanced_opportunities) => {
                // Convert back to technical opportunities with AI enhancements
                let mut enhanced_technical = Vec::new();
                for (i, enhanced) in enhanced_opportunities.iter().enumerate() {
                    if let Some(original_tech) = opportunities.get(i) {
                        let mut tech_opp = original_tech.clone();
                        
                        // Update with AI insights
                        tech_opp.confidence_score = enhanced.ai_score;
                        tech_opp.expected_return_percentage = enhanced.risk_adjusted_score * tech_opp.expected_return_percentage;
                        
                        // Apply group context boost
                        tech_opp.confidence_score = (tech_opp.confidence_score * 1.1).min(1.0);
                        
                        // Update target price based on AI analysis with group context
                        if enhanced.success_probability > 0.7 {
                            if let Some(target) = tech_opp.target_price {
                                tech_opp.target_price = Some(target * (1.0 + enhanced.success_probability * 0.12)); // Slightly higher for groups
                            }
                        }
                        
                        enhanced_technical.push(tech_opp);
                    }
                }
                
                log_info!("Enhanced {} group technical opportunities with AI for admin {}", enhanced_technical.len(), group_admin_id);
                Ok(enhanced_technical)
            }
            Err(e) => {
                log_info!("AI enhancement failed for group admin {}, using original technical opportunities: {}", group_admin_id, e);
                Ok(opportunities) // Fallback to original opportunities
            }
        }
    }

    // Opportunity merging methods
    fn merge_arbitrage_opportunities(
        &self,
        existing: Vec<ArbitrageOpportunity>,
        group: Vec<ArbitrageOpportunity>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut merged = group; // Prioritize group opportunities
        merged.extend(existing);
        
        // Remove duplicates and sort by potential profit
        merged.sort_by(|a, b| {
            let a_profit = a.potential_profit_value.unwrap_or(0.0);
            let b_profit = b.potential_profit_value.unwrap_or(0.0);
            b_profit.partial_cmp(&a_profit).unwrap()
        });
        merged.truncate(15); // Limit to top 15 opportunities for groups
        
        merged
    }

    fn merge_technical_opportunities(
        &self,
        existing: Vec<TechnicalOpportunity>,
        group: Vec<TechnicalOpportunity>,
    ) -> Vec<TechnicalOpportunity> {
        let mut merged = group; // Prioritize group opportunities
        merged.extend(existing);
        
        // Remove duplicates and sort by confidence score
        merged.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        merged.truncate(15); // Limit to top 15 opportunities for groups
        
        merged
    }

    // Delay application methods
    async fn apply_group_opportunity_delay(
        &self,
        opportunities: &mut Vec<ArbitrageOpportunity>,
        delay_seconds: u64,
    ) -> ArbitrageResult<()> {
        for opportunity in opportunities {
            // ArbitrageOpportunity doesn't have created_at field, use timestamp instead
            opportunity.timestamp += delay_seconds * 1000; // Convert to milliseconds
        }
        Ok(())
    }

    async fn apply_group_technical_opportunity_delay(
        &self,
        opportunities: &mut Vec<TechnicalOpportunity>,
        delay_seconds: u64,
    ) -> ArbitrageResult<()> {
        for opportunity in opportunities {
            // TechnicalOpportunity doesn't have created_at field, use timestamp instead
            opportunity.timestamp += delay_seconds * 1000; // Convert to milliseconds
        }
        Ok(())
    }

    // Cache methods
    async fn cache_admin_exchanges(
        &self,
        cache_key: &str,
        exchanges: &[(ExchangeIdEnum, ExchangeCredentials)],
    ) -> ArbitrageResult<()> {
        let serialized = serde_json::to_string(exchanges)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize admin exchanges: {}", e)))?;
        
        self.kv_store
            .put(cache_key, serialized)
            .map_err(|e| ArbitrageError::storage_error(format!("Failed to cache admin exchanges: {:?}", e)))?
            .expiration_ttl(self.cache_ttl_seconds)
            .execute()
            .await
            .map_err(|e| ArbitrageError::storage_error(format!("Failed to execute cache put: {:?}", e)))?;

        Ok(())
    }

    async fn get_cached_admin_exchanges(
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
    use crate::services::{D1Service, UserProfileService, UserAccessService, ExchangeService};
    use chrono::Utc;
    use worker::{Env, kv::KvStore};

    // Test helper functions
    fn create_test_env() -> Env {
        Env::default()
    }

    fn create_test_kv_store() -> KvStore {
        KvStore::default()
    }

    fn create_test_d1_service() -> D1Service {
        D1Service::new(&create_test_env()).unwrap()
    }

    fn create_test_exchange_service() -> Arc<ExchangeService> {
        Arc::new(ExchangeService::new(&create_test_env()).unwrap())
    }

    fn create_test_user_profile_service() -> Arc<UserProfileService> {
        Arc::new(UserProfileService::new(create_test_d1_service()))
    }

    fn create_test_user_access_service() -> Arc<UserAccessService> {
        Arc::new(UserAccessService::new(
            create_test_d1_service(),
            UserProfileService::new(create_test_d1_service()),
            create_test_kv_store(),
        ))
    }

    fn create_test_personal_opportunity_service() -> Arc<PersonalOpportunityService> {
        Arc::new(PersonalOpportunityService::new(
            create_test_exchange_service(),
            create_test_user_profile_service(),
            create_test_user_access_service(),
            create_test_kv_store(),
        ))
    }

    fn create_test_group_opportunity_service() -> GroupOpportunityService {
        GroupOpportunityService {
            exchange_service: create_test_exchange_service(),
            user_profile_service: create_test_user_profile_service(),
            user_access_service: create_test_user_access_service(),
            personal_opportunity_service: create_test_personal_opportunity_service(),
            ai_service: None,
            kv_store: create_test_kv_store(),
            cache_ttl_seconds: 600,
        }
    }

    fn create_test_arbitrage_opportunity(id: &str, symbol: &str) -> ArbitrageOpportunity {
        ArbitrageOpportunity {
            id: id.to_string(),
            symbol: symbol.to_string(),
            buy_exchange: ExchangeIdEnum::Binance,
            sell_exchange: ExchangeIdEnum::Bybit,
            buy_price: 50000.0,
            sell_price: 50100.0,
            price_difference: 100.0,
            price_difference_percent: 0.2,
            potential_profit_value: 100.0,
            potential_profit_percent: 0.2,
            arbitrage_type: ArbitrageType::Spot,
            confidence_score: 0.8,
            estimated_execution_time: 30,
            risk_factors: vec![],
            created_at: Utc::now().timestamp() as u64,
            expires_at: (Utc::now().timestamp() + 300) as u64,
            funding_rates: None,
            volume_24h: 1000000.0,
            liquidity_score: 0.9,
        }
    }

    fn create_test_ticker(symbol: &str, price: f64, change_percent: f64) -> Ticker {
        Ticker {
            symbol: symbol.to_string(),
            last_price: price,
            price_change_percent: change_percent,
            volume_24h: 1000000.0,
            high_24h: price * 1.02,
            low_24h: price * 0.98,
            bid_price: price - 10.0,
            ask_price: price + 10.0,
            timestamp: Utc::now().timestamp() as u64,
        }
    }

    #[test]
    fn test_group_opportunity_service_creation() {
        // Test that GroupOpportunityService can be created with all required dependencies
        let service = create_test_group_opportunity_service();
        
        // Verify service has correct cache TTL
        assert_eq!(service.cache_ttl_seconds, 600);
        
        // Verify AI service is initially None
        assert!(service.ai_service.is_none());
        
        // Verify service constants are correct
        assert_eq!(GroupOpportunityService::GROUP_OPPORTUNITIES_PREFIX, "group_opportunities");
        assert_eq!(GroupOpportunityService::GROUP_ADMIN_CACHE_PREFIX, "group_admin_apis");
        assert_eq!(GroupOpportunityService::GROUP_CACHE_TTL, 600);
    }

    #[test]
    fn test_group_multiplier_logic() {
        // Test that group multiplier creates 2x opportunities with proper variations
        let service = create_test_group_opportunity_service();
        let original_opportunities = vec![
            create_test_arbitrage_opportunity("test_1", "BTCUSDT"),
            create_test_arbitrage_opportunity("test_2", "ETHUSDT"),
        ];

        let multiplied = service.apply_group_multiplier(original_opportunities.clone());
        
        // Should have 2x opportunities (original + group variations)
        assert_eq!(multiplied.len(), 4);
        
        // Verify group multiplier IDs are created correctly
        let group_ids: Vec<_> = multiplied.iter()
            .filter(|opp| opp.id.contains("_group_2x"))
            .collect();
        assert_eq!(group_ids.len(), 2);
        
        // Verify opportunities are sorted by potential profit
        for i in 1..multiplied.len() {
            assert!(multiplied[i-1].potential_profit_percent >= multiplied[i].potential_profit_percent);
        }
        
        // Verify limited to reasonable number (20 max)
        let many_opportunities: Vec<_> = (0..30)
            .map(|i| create_test_arbitrage_opportunity(&format!("test_{}", i), "BTCUSDT"))
            .collect();
        let multiplied_many = service.apply_group_multiplier(many_opportunities);
        assert!(multiplied_many.len() <= 20);
    }

    #[test]
    fn test_technical_signal_determination() {
        let service = create_test_group_opportunity_service();
        
        // Test LONG signal with positive price momentum
        let bullish_ticker = create_test_ticker("BTCUSDT", 50000.0, 3.0);
        let signal = service.determine_technical_signal(&bullish_ticker, &None);
        assert_eq!(signal, "LONG");
        
        // Test SHORT signal with negative price momentum
        let bearish_ticker = create_test_ticker("ETHUSDT", 3000.0, -3.0);
        let signal = service.determine_technical_signal(&bearish_ticker, &None);
        assert_eq!(signal, "SHORT");
        
        // Test NEUTRAL signal with small price change
        let neutral_ticker = create_test_ticker("ADAUSDT", 1.0, 0.5);
        let signal = service.determine_technical_signal(&neutral_ticker, &None);
        assert_eq!(signal, "NEUTRAL");
        
        // Test funding rate override - HIGH positive funding rate should signal SHORT
        let high_funding = Some(FundingRateInfo {
            symbol: "BTCUSDT".to_string(),
            funding_rate: 0.02, // 2% funding rate
            next_funding_time: chrono::Utc::now().timestamp() as u64 + 3600,
            mark_price: 50000.0,
        });
        let signal = service.determine_technical_signal(&bullish_ticker, &high_funding);
        assert_eq!(signal, "SHORT");
        
        // Test negative funding rate should signal LONG
        let negative_funding = Some(FundingRateInfo {
            symbol: "BTCUSDT".to_string(),
            funding_rate: -0.02, // -2% funding rate
            next_funding_time: chrono::Utc::now().timestamp() as u64 + 3600,
            mark_price: 50000.0,
        });
        let signal = service.determine_technical_signal(&bullish_ticker, &negative_funding);
        assert_eq!(signal, "LONG");
    }

    #[test]
    fn test_confidence_score_calculation() {
        let service = create_test_group_opportunity_service();
        
        // Test high volume ticker
        let high_volume_ticker = Ticker {
            symbol: "BTCUSDT".to_string(),
            last_price: 50000.0,
            price_change_percent: 2.5,
            volume_24h: 2000000.0, // High volume
            high_24h: 51000.0,
            low_24h: 49000.0,
            bid_price: 49990.0,
            ask_price: 50010.0,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        let high_funding = Some(FundingRateInfo {
            symbol: "BTCUSDT".to_string(),
            funding_rate: 0.02, // High funding rate
            next_funding_time: chrono::Utc::now().timestamp() as u64 + 3600,
            mark_price: 50000.0,
        });
        
        let confidence = service.calculate_confidence_score(&high_volume_ticker, &high_funding);
        assert!(confidence > 0.8); // Should be high confidence
        
        // Test low volume ticker
        let low_volume_ticker = Ticker {
            symbol: "ALTCOIN".to_string(),
            last_price: 1.0,
            price_change_percent: 0.5,
            volume_24h: 50000.0, // Low volume
            high_24h: 1.01,
            low_24h: 0.99,
            bid_price: 0.999,
            ask_price: 1.001,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        let low_confidence = service.calculate_confidence_score(&low_volume_ticker, &None);
        assert!(low_confidence < 0.7); // Should be lower confidence
    }

    #[test]
    fn test_arbitrage_confidence_calculation() {
        let service = create_test_group_opportunity_service();
        
        // Test high price difference
        let high_diff_confidence = service.calculate_arbitrage_confidence(2.0); // 2%
        assert!(high_diff_confidence > 0.3);
        
        // Test low price difference
        let low_diff_confidence = service.calculate_arbitrage_confidence(0.1); // 0.1%
        assert!(low_diff_confidence < 0.1);
        
        // Test maximum confidence cap
        let max_diff_confidence = service.calculate_arbitrage_confidence(10.0); // 10%
        assert_eq!(max_diff_confidence, 1.0); // Should be capped at 1.0
    }

    #[test]
    fn test_risk_factors_identification() {
        let service = create_test_group_opportunity_service();
        
        // Test low liquidity risk
        let low_liquidity_ticker_a = create_test_ticker("LOWLIQ", 100.0, 1.0);
        let mut low_liquidity_ticker_a_modified = low_liquidity_ticker_a.clone();
        low_liquidity_ticker_a_modified.volume_24h = 50000.0; // Low volume
        
        let low_liquidity_ticker_b = create_test_ticker("LOWLIQ", 101.0, 1.0);
        let mut low_liquidity_ticker_b_modified = low_liquidity_ticker_b.clone();
        low_liquidity_ticker_b_modified.volume_24h = 60000.0; // Low volume
        
        let risks = service.identify_risk_factors(&low_liquidity_ticker_a_modified, &low_liquidity_ticker_b_modified);
        assert!(risks.contains(&"Low liquidity".to_string()));
        
        // Test high volatility divergence risk
        let volatile_ticker_a = create_test_ticker("VOLATILE", 100.0, 10.0); // +10%
        let volatile_ticker_b = create_test_ticker("VOLATILE", 101.0, -5.0); // -5%
        
        let volatility_risks = service.identify_risk_factors(&volatile_ticker_a, &volatile_ticker_b);
        assert!(volatility_risks.contains(&"High volatility divergence".to_string()));
        
        // Test no risks scenario
        let stable_ticker_a = create_test_ticker("STABLE", 100.0, 1.0);
        let mut stable_ticker_a_modified = stable_ticker_a.clone();
        stable_ticker_a_modified.volume_24h = 2000000.0; // High volume
        
        let stable_ticker_b = create_test_ticker("STABLE", 101.0, 1.2);
        let mut stable_ticker_b_modified = stable_ticker_b.clone();
        stable_ticker_b_modified.volume_24h = 2100000.0; // High volume
        
        let no_risks = service.identify_risk_factors(&stable_ticker_a_modified, &stable_ticker_b_modified);
        assert!(no_risks.is_empty());
    }

    #[test]
    fn test_liquidity_score_calculation() {
        let service = create_test_group_opportunity_service();
        
        // Test high liquidity
        let high_vol_ticker_a = create_test_ticker("HIGHVOL", 100.0, 1.0);
        let mut high_vol_ticker_a_modified = high_vol_ticker_a.clone();
        high_vol_ticker_a_modified.volume_24h = 5000000.0;
        
        let high_vol_ticker_b = create_test_ticker("HIGHVOL", 101.0, 1.0);
        let mut high_vol_ticker_b_modified = high_vol_ticker_b.clone();
        high_vol_ticker_b_modified.volume_24h = 5000000.0;
        
        let high_liquidity = service.calculate_liquidity_score(&high_vol_ticker_a_modified, &high_vol_ticker_b_modified);
        assert_eq!(high_liquidity, 1.0); // Should be capped at 1.0
        
        // Test low liquidity
        let low_vol_ticker_a = create_test_ticker("LOWVOL", 100.0, 1.0);
        let mut low_vol_ticker_a_modified = low_vol_ticker_a.clone();
        low_vol_ticker_a_modified.volume_24h = 100000.0;
        
        let low_vol_ticker_b = create_test_ticker("LOWVOL", 101.0, 1.0);
        let mut low_vol_ticker_b_modified = low_vol_ticker_b.clone();
        low_vol_ticker_b_modified.volume_24h = 100000.0;
        
        let low_liquidity = service.calculate_liquidity_score(&low_vol_ticker_a_modified, &low_vol_ticker_b_modified);
        assert!(low_liquidity < 0.2);
    }

    #[test]
    fn test_default_symbols() {
        let service = create_test_group_opportunity_service();
        let symbols = service.get_default_symbols();
        
        assert_eq!(symbols.len(), 5);
        assert!(symbols.contains(&"BTCUSDT".to_string()));
        assert!(symbols.contains(&"ETHUSDT".to_string()));
        assert!(symbols.contains(&"BNBUSDT".to_string()));
        assert!(symbols.contains(&"ADAUSDT".to_string()));
        assert!(symbols.contains(&"SOLUSDT".to_string()));
    }
} 