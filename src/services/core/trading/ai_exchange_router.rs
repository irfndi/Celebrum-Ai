// use worker::{Request, Response, Env}; // TODO: Re-enable when implementing worker integration [Tracked: PR-24, Comment 94]
use crate::{
    services::{
        core::{
            ai::ai_integration::{
                AiAnalysisRequest, AiAnalysisResponse, AiIntegrationService, AiProvider,
            },
            infrastructure::{
                database_repositories::DatabaseManager, /* service_container::ServiceContainer, */
            },
            /* trading::exchange::ExchangeService, */
            user::user_profile::UserProfileService,
        },
        /* interfaces::telegram::TelegramService, */
    },
    types::{ArbitrageOpportunity, /* ExchangeIdEnum, */ GlobalOpportunity, UserProfile},
    utils::{ArbitrageError, ArbitrageResult},
};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use worker::console_log;
use worker::kv::KvStore;
// use regex::Regex; // TODO: Re-enable when implementing text parsing features [Tracked: PR-24, Comment 94]

/// Configuration for AI-Exchange Router
#[derive(Debug, Clone)]
pub struct AiExchangeRouterConfig {
    pub enabled: bool,
    pub max_analysis_timeout_seconds: u64,
    pub max_retries: u32,
    pub cache_ttl_seconds: u64,
    pub rate_limit_per_minute: u32,
}

impl Default for AiExchangeRouterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_analysis_timeout_seconds: 30,
            max_retries: 3,
            cache_ttl_seconds: 300, // 5 minutes
            rate_limit_per_minute: 20,
        }
    }
}

/// Market data structure for AI analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketDataSnapshot {
    pub timestamp: u64,
    pub opportunities: Vec<GlobalOpportunity>,
    pub exchange_data: HashMap<String, ExchangeMarketData>,
    pub context: MarketContext,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExchangeMarketData {
    pub exchange_id: String,
    pub funding_rates: HashMap<String, f64>,
    pub orderbook_depth: HashMap<String, OrderbookDepth>,
    pub volume_24h: HashMap<String, f64>,
    pub last_updated: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderbookDepth {
    pub bids_depth: f64,
    pub asks_depth: f64,
    pub spread: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketContext {
    pub volatility_index: f64,
    pub market_trend: String,
    pub global_sentiment: f64,
    pub active_pairs: Vec<String>,
}

/// AI Analysis result for opportunities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiOpportunityAnalysis {
    pub opportunity_id: String,
    pub user_id: String,
    pub ai_score: f64,
    pub viability_assessment: String,
    pub risk_factors: Vec<String>,
    pub recommended_position_size: f64,
    pub confidence_level: f64,
    pub analysis_timestamp: u64,
    pub ai_provider_used: String,
    pub custom_recommendations: Vec<String>,
}

/// Rate limit tracking for AI calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiCallRateLimit {
    pub user_id: String,
    pub calls_this_minute: u32,
    pub window_start: u64,
}

/// AI-Exchange Router Service
/// Implements secure API call routing through user's AI services
/// Stores audit trails in D1 and uses KV for caching and rate limiting
#[derive(Clone)]
pub struct AiExchangeRouterService {
    config: AiExchangeRouterConfig,
    ai_service: AiIntegrationService,
    user_service: UserProfileService,
    d1_service: DatabaseManager,
    kv_store: KvStore,
    _http_client: Client,
}

impl AiExchangeRouterService {
    /// Create new AI-Exchange Router service
    pub fn new(
        config: AiExchangeRouterConfig,
        ai_service: AiIntegrationService,
        user_service: UserProfileService,
        d1_service: DatabaseManager,
        kv_store: KvStore,
    ) -> Self {
        Self {
            config,
            ai_service,
            user_service,
            d1_service,
            kv_store,
            _http_client: Client::new(),
        }
    }

    /// Route market data analysis request through user's AI service
    /// with audit trail stored in D1 and rate limiting via KV
    pub async fn analyze_market_data(
        &self,
        user_id: &str,
        market_data: &MarketDataSnapshot,
        analysis_prompt: Option<String>,
    ) -> ArbitrageResult<AiAnalysisResponse> {
        if !self.config.enabled {
            return Err(ArbitrageError::config_error(
                "AI-Exchange routing is disabled",
            ));
        }

        // Check rate limits (KV for speed)
        self.check_and_update_rate_limit(user_id).await?;

        // Get user profile with AI providers
        let user_profile = self
            .user_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("User profile not found"))?;

        // Get user's preferred AI provider
        let ai_provider = self.get_user_ai_provider(&user_profile).await?;

        // Create analysis request
        let prompt = analysis_prompt.unwrap_or_else(|| self.create_default_analysis_prompt());
        let analysis_request = AiAnalysisRequest {
            prompt,
            market_data: serde_json::to_value(market_data).map_err(|e| {
                ArbitrageError::parse_error(format!("Failed to serialize market data: {}", e))
            })?,
            user_context: Some(self.create_user_context(&user_profile)?),
            max_tokens: Some(1000),
            temperature: Some(0.7),
        };

        // Call AI provider with retry logic
        let analysis_result = self
            .call_ai_with_retries(&ai_provider, &analysis_request)
            .await?;

        // Store audit trail in D1
        self.store_ai_analysis_audit(user_id, &ai_provider, &analysis_request, &analysis_result)
            .await?;

        // Cache result in KV for faster subsequent access
        self.cache_analysis_result(user_id, &analysis_result)
            .await?;

        Ok(analysis_result)
    }

    /// Analyze specific opportunities with AI and provide customized recommendations
    pub async fn analyze_opportunities(
        &self,
        user_id: &str,
        opportunities: &[GlobalOpportunity],
        user_context: Option<Value>,
    ) -> ArbitrageResult<Vec<AiOpportunityAnalysis>> {
        if !self.config.enabled {
            return Err(ArbitrageError::config_error(
                "AI-Exchange routing is disabled",
            ));
        }

        // Check rate limits
        self.check_and_update_rate_limit(user_id).await?;

        // Get user profile
        let user_profile = self
            .user_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("User profile not found"))?;

        // Get AI provider
        let ai_provider = self.get_user_ai_provider(&user_profile).await?;

        let mut analyses = Vec::new();

        for opportunity in opportunities {
            // Create opportunity-specific analysis request
            let analysis_request = self.create_opportunity_analysis_request(
                opportunity,
                &user_profile,
                user_context.as_ref(),
            )?;

            // Call AI provider
            let ai_response = self
                .call_ai_with_retries(&ai_provider, &analysis_request)
                .await?;

            // Parse AI response into structured analysis
            let analysis = self.parse_ai_opportunity_response(
                user_id,
                opportunity,
                &ai_response,
                &ai_provider,
            )?;

            // Store in D1 for analytics
            self.store_opportunity_analysis(&analysis).await?;

            analyses.push(analysis);
        }

        Ok(analyses)
    }

    /// Get real-time AI recommendations for active trading decisions
    pub async fn get_real_time_recommendations(
        &self,
        user_id: &str,
        current_positions: &[ArbitrageOpportunity],
        market_snapshot: &MarketDataSnapshot,
    ) -> ArbitrageResult<AiAnalysisResponse> {
        if !self.config.enabled {
            return Err(ArbitrageError::config_error(
                "AI-Exchange routing is disabled",
            ));
        }

        // Check cache first (KV)
        if let Some(cached_result) = self.get_cached_recommendations(user_id).await? {
            return Ok(cached_result);
        }

        // Get user and AI provider
        let user_profile = self
            .user_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("User profile not found"))?;

        let ai_provider = self.get_user_ai_provider(&user_profile).await?;

        // Create real-time analysis request
        let prompt = format!(
            "Analyze current positions and market data to provide immediate trading recommendations. \
             Current positions: {} active trades. Market volatility: {:.2}%. \
             Provide specific actions: HOLD, CLOSE, or ADJUST with reasoning.",
            current_positions.len(),
            market_snapshot.context.volatility_index * 100.0
        );

        let analysis_request = AiAnalysisRequest {
            prompt,
            market_data: json!({
                "current_positions": current_positions,
                "market_snapshot": market_snapshot
            }),
            user_context: Some(self.create_user_context(&user_profile)?),
            max_tokens: Some(800),
            temperature: Some(0.3), // Lower temperature for trading decisions
        };

        // Get AI analysis
        let analysis = self
            .call_ai_with_retries(&ai_provider, &analysis_request)
            .await?;

        // Cache for 2 minutes (real-time data)
        self.cache_recommendations(user_id, &analysis, 120).await?;

        // Store audit in D1
        self.store_ai_analysis_audit(user_id, &ai_provider, &analysis_request, &analysis)
            .await?;

        Ok(analysis)
    }

    /// Check and update rate limits using KV store
    async fn check_and_update_rate_limit(&self, user_id: &str) -> ArbitrageResult<()> {
        let rate_limit_key = format!("ai_rate_limit:{}", user_id);
        let current_time = chrono::Utc::now().timestamp() as u64;
        let window_duration = 60; // 1 minute

        let rate_limit = match self.kv_store.get(&rate_limit_key).text().await {
            Ok(Some(data)) => {
                serde_json::from_str::<AiCallRateLimit>(&data).unwrap_or_else(|_| AiCallRateLimit {
                    user_id: user_id.to_string(),
                    calls_this_minute: 0,
                    window_start: current_time,
                })
            }
            _ => AiCallRateLimit {
                user_id: user_id.to_string(),
                calls_this_minute: 0,
                window_start: current_time,
            },
        };

        // Check if we're in a new time window
        let updated_limit = if current_time - rate_limit.window_start >= window_duration {
            AiCallRateLimit {
                user_id: user_id.to_string(),
                calls_this_minute: 1,
                window_start: current_time,
            }
        } else {
            if rate_limit.calls_this_minute >= self.config.rate_limit_per_minute {
                return Err(ArbitrageError::rate_limit_error(
                    "AI analysis rate limit exceeded",
                ));
            }
            AiCallRateLimit {
                user_id: user_id.to_string(),
                calls_this_minute: rate_limit.calls_this_minute + 1,
                window_start: rate_limit.window_start,
            }
        };

        // Update rate limit in KV
        let rate_limit_data = serde_json::to_string(&updated_limit).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize rate limit: {}", e))
        })?;

        self.kv_store
            .put(&rate_limit_key, rate_limit_data)
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create KV put request: {}", e))
            })?
            .expiration_ttl(120) // 2 minutes expiration
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to update rate limit: {}", e))
            })?;

        Ok(())
    }

    /// Get user's AI provider for analysis
    async fn get_user_ai_provider(
        &self,
        user_profile: &UserProfile,
    ) -> ArbitrageResult<AiProvider> {
        // Find first active AI API key
        let ai_key = user_profile
            .api_keys
            .iter()
            .find(|key| {
                key.is_active
                    && matches!(
                        key.provider,
                        crate::types::ApiKeyProvider::OpenAI
                            | crate::types::ApiKeyProvider::Anthropic
                            | crate::types::ApiKeyProvider::Custom
                    )
            })
            .ok_or_else(|| ArbitrageError::not_found("No active AI API key found for user"))?;

        // Create AI provider from the key
        self.ai_service.create_ai_provider(ai_key)
    }

    /// Call AI provider with retry logic
    async fn call_ai_with_retries(
        &self,
        provider: &AiProvider,
        request: &AiAnalysisRequest,
    ) -> ArbitrageResult<AiAnalysisResponse> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            match self.ai_service.call_ai_provider(provider, request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);

                    // Add minimal delay between retries to avoid overwhelming rate-limited AI APIs
                    // Even in Cloudflare Workers, this prevents burst requests that could trigger rate limits
                    if attempt < self.config.max_retries {
                        // Minimal delay: 100-500ms to respect rate limits while staying responsive
                        let delay_ms = 100 + ((attempt - 1) * 100); // 100ms, 200ms, 300ms progression
                        self.delay_async(delay_ms).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| ArbitrageError::network_error("All AI provider calls failed")))
    }

    /// Store AI analysis audit trail in D1
    async fn store_ai_analysis_audit(
        &self,
        user_id: &str,
        ai_provider: &AiProvider,
        request: &AiAnalysisRequest,
        response: &AiAnalysisResponse,
    ) -> ArbitrageResult<()> {
        let start_time = std::time::Instant::now();

        let provider_name = match ai_provider {
            AiProvider::OpenAI { .. } => "OpenAI",
            AiProvider::Anthropic { .. } => "Anthropic",
            AiProvider::Custom { .. } => "Custom",
        };

        let request_data = serde_json::to_value(request).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize request: {}", e))
        })?;

        let response_data = serde_json::to_value(response).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize response: {}", e))
        })?;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        // Combine request and response data for audit
        let audit_data = serde_json::json!({
            "request": request_data,
            "response": response_data,
            "processing_time_ms": processing_time_ms,
            "provider": provider_name
        });

        let confidence_score = response.confidence.unwrap_or(0.5);

        // Store audit trail in D1 database
        self.d1_service
            .store_ai_analysis_audit(
                user_id,
                "ai_exchange_analysis",
                &audit_data,
                confidence_score.into(),
            )
            .await?;

        console_log!(
            "AI Analysis Audit stored: user={}, provider={}, processing_time_ms={}",
            user_id,
            provider_name,
            processing_time_ms
        );

        Ok(())
    }

    /// Async delay for retry backoff in Cloudflare Workers environment
    async fn delay_async(&self, delay_ms: u32) {
        #[cfg(target_arch = "wasm32")]
        {
            // Use worker's sleep for WASM environment
            use worker::Delay;
            Delay::from(std::time::Duration::from_millis(delay_ms as u64)).await;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use tokio's sleep for native environment
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
        }
    }

    /// Cache analysis result in KV store
    async fn cache_analysis_result(
        &self,
        user_id: &str,
        analysis: &AiAnalysisResponse,
    ) -> ArbitrageResult<()> {
        let cache_key = format!("ai_analysis_cache:{}", user_id);
        let cache_data = serde_json::to_string(analysis).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize analysis: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, cache_data)
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create KV put request: {}", e))
            })?
            .expiration_ttl(self.config.cache_ttl_seconds)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to cache analysis: {}", e))
            })?;

        Ok(())
    }

    /// Create default analysis prompt for market data
    fn create_default_analysis_prompt(&self) -> String {
        "Analyze the provided cryptocurrency arbitrage market data. \
         Focus on identifying high-probability opportunities, risk factors, \
         and optimal position sizing. Provide specific actionable insights \
         for arbitrage trading decisions. Consider market volatility, \
         liquidity, and funding rate dynamics."
            .to_string()
    }

    /// Create user context for AI analysis
    fn create_user_context(&self, user_profile: &UserProfile) -> ArbitrageResult<Value> {
        Ok(json!({
            "user_id": user_profile.user_id,
            "subscription_tier": user_profile.subscription.tier,
            "total_trades": user_profile.total_trades,
            "total_pnl": user_profile.total_pnl_usdt,
            "risk_tolerance": user_profile.configuration.risk_tolerance_percentage,
            "max_position_size": user_profile.configuration.max_entry_size_usdt,
            "min_position_size": user_profile.configuration.max_entry_size_usdt * 0.1, // 10% of max as min
        }))
    }

    /// Create opportunity-specific analysis request
    fn create_opportunity_analysis_request(
        &self,
        opportunity: &GlobalOpportunity,
        user_profile: &UserProfile,
        user_context: Option<&Value>,
    ) -> ArbitrageResult<AiAnalysisRequest> {
        let prompt = format!(
            "Analyze this specific arbitrage opportunity: {:?} with {:.2}% rate difference. \
             Consider user's risk tolerance ({:.2}%) and max position size (${:.2}). \
             Provide viability score (0-100), risk factors, and recommended position size.",
            opportunity.opportunity_data,
            if let crate::types::OpportunityData::Arbitrage(arb_opp) = &opportunity.opportunity_data
            {
                arb_opp.rate_difference * 100.0
            } else {
                0.0
            },
            user_profile.configuration.risk_tolerance_percentage,
            user_profile.configuration.max_entry_size_usdt
        );

        Ok(AiAnalysisRequest {
            prompt,
            market_data: serde_json::to_value(opportunity).map_err(|e| {
                ArbitrageError::parse_error(format!("Failed to serialize opportunity: {}", e))
            })?,
            user_context: user_context
                .cloned()
                .or_else(|| self.create_user_context(user_profile).ok()),
            max_tokens: Some(600),
            temperature: Some(0.5),
        })
    }

    /// Parse AI response into structured opportunity analysis
    fn parse_ai_opportunity_response(
        &self,
        user_id: &str,
        opportunity: &GlobalOpportunity,
        ai_response: &AiAnalysisResponse,
        ai_provider: &AiProvider,
    ) -> ArbitrageResult<AiOpportunityAnalysis> {
        // Extract score from AI response (simple regex-based extraction for now)
        let ai_score = self.extract_score_from_analysis(&ai_response.analysis);
        let confidence = ai_response.confidence.unwrap_or(0.5) as f64;

        Ok(AiOpportunityAnalysis {
            opportunity_id: match &opportunity.opportunity_data {
                crate::types::OpportunityData::Arbitrage(arb) => arb.id.clone(),
                crate::types::OpportunityData::Technical(tech) => tech.id.clone(),
                crate::types::OpportunityData::AI(ai) => ai.id.clone(),
            },
            user_id: user_id.to_string(),
            ai_score,
            viability_assessment: ai_response.analysis.clone(),
            risk_factors: self.extract_risk_factors(&ai_response.analysis),
            recommended_position_size: self.extract_position_size(&ai_response.analysis),
            confidence_level: confidence,
            analysis_timestamp: chrono::Utc::now().timestamp() as u64,
            ai_provider_used: match ai_provider {
                AiProvider::OpenAI { .. } => "OpenAI".to_string(),
                AiProvider::Anthropic { .. } => "Anthropic".to_string(),
                AiProvider::Custom { .. } => "Custom".to_string(),
            },
            custom_recommendations: ai_response.recommendations.clone(),
        })
    }

    /// Extract numeric score from AI analysis text
    fn extract_score_from_analysis(&self, analysis: &str) -> f64 {
        let score_regex =
            regex::Regex::new(r"(?i)(?:score|viability):\s*(\d+)|(\d+)/100|(\d+)%").unwrap();

        if let Some(captures) = score_regex.captures(analysis) {
            let score_str = captures
                .get(1)
                .or_else(|| captures.get(2))
                .or_else(|| captures.get(3))
                .map(|m| m.as_str())
                .unwrap_or("50");

            score_str.parse::<f64>().unwrap_or(50.0) / 100.0
        } else {
            0.5 // Default neutral score
        }
    }

    /// Extract risk factors from AI analysis
    fn extract_risk_factors(&self, analysis: &str) -> Vec<String> {
        let mut risk_factors = Vec::new();
        let analysis_lower = analysis.to_lowercase();

        if analysis_lower.contains("volatility") {
            risk_factors.push("High market volatility".to_string());
        }
        if analysis_lower.contains("liquidity") {
            risk_factors.push("Liquidity constraints".to_string());
        }
        if analysis_lower.contains("regulation") {
            risk_factors.push("Regulatory uncertainty".to_string());
        }
        if analysis_lower.contains("slippage") {
            risk_factors.push("Price slippage risk".to_string());
        }

        risk_factors
    }

    /// Extract recommended position size from analysis
    fn extract_position_size(&self, analysis: &str) -> f64 {
        let usdt_regex =
            regex::Regex::new(r"(?i)(?:size\s*:?\s*\$|use\s+)(\d+(?:\.\d+)?)\s*(?:usdt|usd|\$?)\b")
                .unwrap();
        let percentage_regex =
            regex::Regex::new(r"(?i)position\s*(?:size|amount)\s*:?\s*(\d+(?:\.\d+)?)\s*%")
                .unwrap();

        // First try to match USDT/dollar amounts
        if let Some(captures) = usdt_regex.captures(analysis) {
            if let Some(matched) = captures.get(1) {
                if let Ok(amount) = matched.as_str().parse::<f64>() {
                    return amount; // Return the USDT amount directly
                }
            }
        }

        // Then try percentage patterns
        if let Some(captures) = percentage_regex.captures(analysis) {
            if let Some(matched) = captures.get(1) {
                if let Ok(percentage) = matched.as_str().parse::<f64>() {
                    return percentage / 100.0; // Convert percentage to decimal
                }
            }
        }

        100.0 // Default $100 position size (changed from 0.05)
    }

    /// Store opportunity analysis in D1
    async fn store_opportunity_analysis(
        &self,
        analysis: &AiOpportunityAnalysis,
    ) -> ArbitrageResult<()> {
        // Convert analysis to serde_json::Value for database storage
        let analysis_value = serde_json::to_value(analysis).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize analysis: {}", e))
        })?;

        // Store opportunity analysis in D1 database
        self.d1_service
            .store_opportunity_analysis(&analysis_value)
            .await?;

        console_log!(
            "AI Opportunity Analysis stored: opportunity_id={}, user_id={}, score={:.2}",
            analysis.opportunity_id,
            analysis.user_id,
            analysis.ai_score
        );

        Ok(())
    }

    /// Get cached recommendations from KV
    async fn get_cached_recommendations(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<AiAnalysisResponse>> {
        let cache_key = format!("ai_recommendations_cache:{}", user_id);

        match self.kv_store.get(&cache_key).text().await {
            Ok(Some(data)) => match serde_json::from_str::<AiAnalysisResponse>(&data) {
                Ok(response) => Ok(Some(response)),
                Err(_) => Ok(None),
            },
            _ => Ok(None),
        }
    }

    /// Cache recommendations in KV
    async fn cache_recommendations(
        &self,
        user_id: &str,
        analysis: &AiAnalysisResponse,
        ttl_seconds: u64,
    ) -> ArbitrageResult<()> {
        let cache_key = format!("ai_recommendations_cache:{}", user_id);
        let cache_data = serde_json::to_string(analysis).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize recommendations: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, cache_data)
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create KV put request: {}", e))
            })?
            .expiration_ttl(ttl_seconds)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to cache recommendations: {}", e))
            })?;

        Ok(())
    }
}

// Tests have been moved to packages/worker/tests/trading/ai_exchange_router_test.rs
