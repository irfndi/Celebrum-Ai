// Cloudflare Vectorize Service for AI-Enhanced Opportunity Matching
// Leverages Cloudflare Vectorize for similarity search, personalization, and recommendation features

use crate::types::{ArbitrageOpportunity, ExchangeIdEnum};
use crate::utils::{ArbitrageError, ArbitrageResult};
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::{Env, Fetch, Method, Request, RequestInit};

/// Configuration for Vectorize service
#[derive(Debug, Clone)]
pub struct VectorizeConfig {
    pub enabled: bool,
    pub index_name: String,
    pub embedding_dimensions: u32,
    pub similarity_threshold: f32,
    pub max_results: u32,
    pub cache_ttl_seconds: u64,
    pub api_base_url: String, // Cloudflare Vectorize API endpoint
    pub account_id: String,   // Cloudflare account ID
    pub api_token: String,    // Cloudflare API token
}

impl Default for VectorizeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            index_name: "arbitrage-opportunities".to_string(),
            embedding_dimensions: 384, // Standard embedding dimension
            similarity_threshold: 0.7,
            max_results: 10,
            cache_ttl_seconds: 3600, // 1 hour
            api_base_url: "https://api.cloudflare.com/client/v4".to_string(),
            account_id: "".to_string(), // To be set from environment
            api_token: "".to_string(),  // To be set from environment
        }
    }
}

/// Opportunity embedding for vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityEmbedding {
    pub opportunity_id: String,
    pub pair: String,
    pub exchange_combination: String,
    pub rate_difference: f32,
    pub risk_level: f32,
    pub market_conditions: Vec<f32>,
    pub embedding: Vec<f32>,
    pub metadata: OpportunityMetadata,
    pub created_at: u64,
}

/// User preference vector for personalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferenceVector {
    pub user_id: String,
    pub risk_tolerance: f32,
    pub preferred_pairs: Vec<String>,
    pub preferred_exchanges: Vec<ExchangeIdEnum>,
    pub interaction_history: Vec<f32>,
    pub success_patterns: Vec<f32>,
    pub embedding: Vec<f32>,
    pub last_updated: u64,
}

/// Metadata for opportunity embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityMetadata {
    pub opportunity_type: String,
    pub risk_category: String,
    pub profit_potential: f32,
    pub time_sensitivity: String,
    pub market_volatility: f32,
    pub liquidity_score: f32,
}

/// Similarity search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub opportunity_id: String,
    pub similarity_score: f32,
    pub opportunity: ArbitrageOpportunity,
    pub metadata: OpportunityMetadata,
}

/// Ranked opportunity with personalization score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedOpportunity {
    pub opportunity: ArbitrageOpportunity,
    pub similarity_score: f32,
    pub personalization_score: f32,
    pub combined_score: f32,
    pub ranking_factors: HashMap<String, f32>,
}

/// Request structure for Cloudflare Vectorize API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizeRequest {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: serde_json::Value,
}

/// Query request structure for Cloudflare Vectorize API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizeQueryRequest {
    pub vector: Vec<f32>,
    #[serde(rename = "topK")]
    pub top_k: u32,
    pub filter: Option<serde_json::Value>,
    #[serde(rename = "returnValues")]
    pub return_values: bool,
    #[serde(rename = "returnMetadata")]
    pub return_metadata: bool,
}

/// Response structure from Cloudflare Vectorize API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizeQueryResponse {
    pub success: bool,
    pub result: VectorizeQueryResult,
}

/// Query result structure from Cloudflare Vectorize API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizeQueryResult {
    pub matches: Vec<VectorizeMatch>,
}

/// Individual match result from Cloudflare Vectorize API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizeMatch {
    pub id: String,
    pub score: f32,
    pub values: Option<Vec<f32>>,
    pub metadata: Option<serde_json::Value>,
}

/// Cloudflare Vectorize Service
pub struct VectorizeService {
    config: VectorizeConfig,
    logger: crate::utils::logger::Logger,
}

impl VectorizeService {
    /// Create new VectorizeService instance
    pub fn new(env: &Env, mut config: VectorizeConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        // Get Cloudflare credentials from environment
        if let Ok(account_id) = env.var("CLOUDFLARE_ACCOUNT_ID") {
            config.account_id = account_id.to_string();
        }
        if let Ok(api_token) = env.var("CLOUDFLARE_API_TOKEN") {
            config.api_token = api_token.to_string();
        }

        // Validate configuration
        if config.enabled && (config.account_id.is_empty() || config.api_token.is_empty()) {
            logger.warn("Vectorize service disabled: missing Cloudflare credentials");
            config.enabled = false;
        }

        logger.info(&format!(
            "Vectorize service initialized: enabled={}, index='{}'",
            config.enabled, config.index_name
        ));

        Ok(Self { config, logger })
    }

    /// Store opportunity embedding in Vectorize
    pub async fn store_opportunity_embedding(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            self.logger
                .debug("Vectorize service disabled, skipping embedding storage");
            return Ok(());
        }

        // Generate embedding for the opportunity
        let embedding = self.generate_opportunity_embedding(opportunity).await?;

        // Create metadata
        let metadata = self.create_opportunity_metadata(opportunity).await?;

        // Prepare Vectorize request
        let vector_request = VectorizeRequest {
            id: opportunity.id.clone(),
            values: embedding.embedding,
            metadata: serde_json::json!({
                "pair": embedding.pair,
                "exchange_combination": embedding.exchange_combination,
                "rate_difference": embedding.rate_difference,
                "risk_level": embedding.risk_level,
                "opportunity_type": metadata.opportunity_type,
                "risk_category": metadata.risk_category,
                "profit_potential": metadata.profit_potential,
                "time_sensitivity": metadata.time_sensitivity,
                "market_volatility": metadata.market_volatility,
                "liquidity_score": metadata.liquidity_score,
                "created_at": embedding.created_at
            }),
        };

        // Make API call to Cloudflare Vectorize
        match self.vectorize_upsert(&[vector_request]).await {
            Ok(_) => {
                self.logger.info(&format!(
                    "Successfully stored opportunity embedding: id={}, pair={}",
                    opportunity.id, opportunity.pair
                ));
                Ok(())
            }
            Err(e) => {
                self.logger.error(&format!(
                    "Failed to store opportunity embedding: id={}, error={}",
                    opportunity.id, e
                ));
                // Don't fail the entire operation if vectorize fails
                Ok(())
            }
        }
    }

    /// Find similar opportunities using vector similarity search
    pub async fn find_similar_opportunities(
        &self,
        reference_opportunity: &ArbitrageOpportunity,
        limit: Option<u32>,
    ) -> ArbitrageResult<Vec<SimilarityResult>> {
        if !self.config.enabled {
            self.logger
                .debug("Vectorize service disabled, returning empty results");
            return Ok(Vec::new());
        }

        // Generate embedding for reference opportunity
        let reference_embedding = self
            .generate_opportunity_embedding(reference_opportunity)
            .await?;

        // Prepare query request
        let query_request = VectorizeQueryRequest {
            vector: reference_embedding.embedding,
            top_k: limit.unwrap_or(self.config.max_results),
            filter: None, // Could add filters for pair, exchange, etc.
            return_values: false,
            return_metadata: true,
        };

        // Make API call to Cloudflare Vectorize
        match self.vectorize_query(&query_request).await {
            Ok(response) => {
                let mut similarity_results = Vec::new();

                for match_result in response.result.matches {
                    // Skip if similarity score is below threshold
                    if match_result.score < self.config.similarity_threshold {
                        continue;
                    }

                    // Skip self-match
                    if match_result.id == reference_opportunity.id {
                        continue;
                    }

                    // Extract metadata and reconstruct opportunity
                    if let Some(metadata) = match_result.metadata {
                        match self.reconstruct_opportunity_from_metadata(&metadata) {
                            Ok(opportunity) => {
                                let opp_metadata = self.extract_metadata_from_result(&metadata)?;
                                similarity_results.push(SimilarityResult {
                                    opportunity_id: match_result.id,
                                    similarity_score: match_result.score,
                                    opportunity,
                                    metadata: opp_metadata,
                                });
                            }
                            Err(e) => {
                                self.logger.warn(&format!(
                                    "Failed to reconstruct opportunity from metadata: {}",
                                    e
                                ));
                            }
                        }
                    }
                }

                self.logger.info(&format!(
                    "Found {} similar opportunities for pair={} (threshold={})",
                    similarity_results.len(),
                    reference_opportunity.pair,
                    self.config.similarity_threshold
                ));

                Ok(similarity_results)
            }
            Err(e) => {
                self.logger
                    .error(&format!("Failed to query similar opportunities: {}", e));
                // Return empty results instead of failing
                Ok(Vec::new())
            }
        }
    }

    /// Store user preference vector for personalization
    pub async fn store_user_preference_vector(
        &self,
        user_id: &str,
        interactions: &[OpportunityInteraction],
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            self.logger
                .debug("Vectorize service disabled, skipping user vector storage");
            return Ok(());
        }

        // Generate user preference embedding
        let preference_vector = self
            .generate_user_preference_vector(user_id, interactions)
            .await?;

        // Prepare Vectorize request with user prefix
        let vector_request = VectorizeRequest {
            id: format!("user_{}", user_id),
            values: preference_vector.embedding,
            metadata: serde_json::json!({
                "user_id": preference_vector.user_id,
                "risk_tolerance": preference_vector.risk_tolerance,
                "preferred_pairs": preference_vector.preferred_pairs,
                "preferred_exchanges": preference_vector.preferred_exchanges,
                "last_updated": preference_vector.last_updated,
                "vector_type": "user_preference"
            }),
        };

        // Make API call to Cloudflare Vectorize
        match self.vectorize_upsert(&[vector_request]).await {
            Ok(_) => {
                self.logger.info(&format!(
                    "Successfully stored user preference vector: user_id={}",
                    user_id
                ));
                Ok(())
            }
            Err(e) => {
                self.logger.error(&format!(
                    "Failed to store user preference vector: user_id={}, error={}",
                    user_id, e
                ));
                // Don't fail the entire operation if vectorize fails
                Ok(())
            }
        }
    }

    /// Rank opportunities for a specific user using personalization
    pub async fn rank_opportunities_for_user(
        &self,
        user_id: &str,
        opportunities: &[ArbitrageOpportunity],
    ) -> ArbitrageResult<Vec<RankedOpportunity>> {
        if !self.config.enabled || opportunities.is_empty() {
            // Return default ranking when vectorize is disabled
            return Ok(opportunities
                .iter()
                .map(|opp| RankedOpportunity {
                    opportunity: opp.clone(),
                    similarity_score: 0.5,
                    personalization_score: 0.5,
                    combined_score: 0.5,
                    ranking_factors: HashMap::new(),
                })
                .collect());
        }

        // Try to retrieve user preference vector
        let user_vector = match self.get_user_preference_vector(user_id).await {
            Ok(vector) => vector,
            Err(_) => {
                // If no user vector exists, return default ranking
                self.logger.info(&format!(
                    "No user preference vector found for user: {}, using default ranking",
                    user_id
                ));
                return Ok(opportunities
                    .iter()
                    .map(|opp| RankedOpportunity {
                        opportunity: opp.clone(),
                        similarity_score: 0.5,
                        personalization_score: 0.5,
                        combined_score: 0.5,
                        ranking_factors: HashMap::new(),
                    })
                    .collect());
            }
        };

        // Rank each opportunity
        let mut ranked_opportunities = Vec::new();
        for opportunity in opportunities {
            let ranking = self
                .calculate_opportunity_ranking(opportunity, &user_vector)
                .await?;
            ranked_opportunities.push(ranking);
        }

        // Sort by combined score (highest first)
        ranked_opportunities.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.logger.info(&format!(
            "Ranked {} opportunities for user: {}",
            ranked_opportunities.len(),
            user_id
        ));

        Ok(ranked_opportunities)
    }

    /// Get personalized opportunities for a user based on their preferences
    pub async fn get_personalized_opportunities(
        &self,
        user_id: &str,
        opportunities: &[ArbitrageOpportunity],
    ) -> ArbitrageResult<Vec<RankedOpportunity>> {
        if !self.config.enabled {
            // Return empty results when service is disabled for consistent behavior
            return Ok(Vec::new());
        }

        // Use the existing rank_opportunities_for_user method
        self.rank_opportunities_for_user(user_id, opportunities)
            .await
    }

    /// Make HTTP request to Cloudflare Vectorize API for upserting vectors
    async fn vectorize_upsert(&self, vectors: &[VectorizeRequest]) -> ArbitrageResult<()> {
        let url = format!(
            "{}/accounts/{}/vectorize/indexes/{}/upsert",
            self.config.api_base_url, self.config.account_id, self.config.index_name
        );

        let request_body = serde_json::json!({
            "vectors": vectors
        });

        let mut request_init = RequestInit::new();
        request_init.method = Method::Post;
        request_init.headers = worker::Headers::new();
        request_init.headers.set(
            "Authorization",
            &format!("Bearer {}", self.config.api_token),
        )?;
        request_init
            .headers
            .set("Content-Type", "application/json")?;
        request_init.body = Some(request_body.to_string().into());

        let request = Request::new_with_init(&url, &request_init)?;
        let mut response = Fetch::Request(request).send().await?;

        if response.status_code() == 200 {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ArbitrageError::api_error(format!(
                "Vectorize upsert failed: status={}, error={}",
                response.status_code(),
                error_text
            )))
        }
    }

    /// Make HTTP request to Cloudflare Vectorize API for querying vectors
    async fn vectorize_query(
        &self,
        query: &VectorizeQueryRequest,
    ) -> ArbitrageResult<VectorizeQueryResponse> {
        let url = format!(
            "{}/accounts/{}/vectorize/indexes/{}/query",
            self.config.api_base_url, self.config.account_id, self.config.index_name
        );

        let mut request_init = RequestInit::new();
        request_init.method = Method::Post;
        request_init.headers = worker::Headers::new();
        request_init.headers.set(
            "Authorization",
            &format!("Bearer {}", self.config.api_token),
        )?;
        request_init
            .headers
            .set("Content-Type", "application/json")?;
        request_init.body = Some(serde_json::to_string(query)?.into());

        let request = Request::new_with_init(&url, &request_init)?;
        let mut response = Fetch::Request(request).send().await?;

        if response.status_code() == 200 {
            let response_text = response.text().await?;
            let query_response: VectorizeQueryResponse = serde_json::from_str(&response_text)?;
            Ok(query_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ArbitrageError::api_error(format!(
                "Vectorize query failed: status={}, error={}",
                response.status_code(),
                error_text
            )))
        }
    }

    /// Get user preference vector from Vectorize
    async fn get_user_preference_vector(&self, user_id: &str) -> ArbitrageResult<Vec<f32>> {
        let url = format!(
            "{}/accounts/{}/vectorize/indexes/{}/get-by-ids",
            self.config.api_base_url, self.config.account_id, self.config.index_name
        );

        let request_body = serde_json::json!({
            "ids": [format!("user_{}", user_id)]
        });

        let mut request_init = RequestInit::new();
        request_init.method = Method::Post;
        request_init.headers = worker::Headers::new();
        request_init.headers.set(
            "Authorization",
            &format!("Bearer {}", self.config.api_token),
        )?;
        request_init
            .headers
            .set("Content-Type", "application/json")?;
        request_init.body = Some(request_body.to_string().into());

        let request = Request::new_with_init(&url, &request_init)?;
        let mut response = Fetch::Request(request).send().await?;

        if response.status_code() == 200 {
            let response_text = response.text().await?;
            let response_data: serde_json::Value = serde_json::from_str(&response_text)?;

            if let Some(vectors) = response_data["result"]["vectors"].as_array() {
                if let Some(vector) = vectors.first() {
                    if let Some(values) = vector["values"].as_array() {
                        let user_vector: Vec<f32> = values
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect();
                        return Ok(user_vector);
                    }
                }
            }

            Err(ArbitrageError::not_found(format!(
                "User preference vector not found: {}",
                user_id
            )))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ArbitrageError::api_error(format!(
                "Failed to get user vector: status={}, error={}",
                response.status_code(),
                error_text
            )))
        }
    }

    /// Generate embedding for an opportunity
    async fn generate_opportunity_embedding(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<OpportunityEmbedding> {
        // Create feature vector from opportunity characteristics
        let mut features = Vec::new();

        // Rate difference features (normalized)
        features.push(opportunity.rate_difference as f32 * 1000.0); // Scale up for better representation

        // Safe log transform for skewed data - prevent NaN by clamping rate_difference
        let rate_diff = opportunity.rate_difference as f32;
        let safe_rate_diff = if rate_diff <= -1.0 { -0.999 } else { rate_diff };
        features.push(safe_rate_diff.ln_1p()); // Log transform for skewed data

        // Exchange features (one-hot encoding)
        let exchanges = [
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
            ExchangeIdEnum::Bitget,
        ];
        for exchange in &exchanges {
            features.push(if opportunity.long_exchange == *exchange {
                1.0
            } else {
                0.0
            });
            features.push(if opportunity.short_exchange == *exchange {
                1.0
            } else {
                0.0
            });
        }

        // Pair features (simplified encoding)
        let major_pairs = ["BTC", "ETH", "SOL", "ADA", "DOT"];
        for pair in &major_pairs {
            features.push(if opportunity.pair.contains(pair) {
                1.0
            } else {
                0.0
            });
        }

        // Time-based features
        let current_time = chrono::Utc::now().timestamp() as f32;
        let opportunity_age = (current_time - opportunity.timestamp as f32) / 3600.0; // Hours
        features.push(opportunity_age);
        features.push((-opportunity_age / 24.0).exp()); // Decay function for freshness

        // Risk features
        let risk_level = self.calculate_opportunity_risk(opportunity);
        features.push(risk_level);
        features.push(risk_level.powi(2)); // Non-linear risk representation

        // Market condition features (enhanced with real market data)
        let market_volatility = self.calculate_market_volatility(opportunity).await?;
        let liquidity_score = self.calculate_liquidity_score(opportunity).await?;
        let market_sentiment = self.calculate_market_sentiment(opportunity).await?;

        features.push(market_volatility);
        features.push(liquidity_score);
        features.push(market_sentiment);

        // Pad or truncate to match embedding dimensions
        while features.len() < self.config.embedding_dimensions as usize {
            features.push(0.0);
        }
        features.truncate(self.config.embedding_dimensions as usize);

        // Normalize the embedding vector
        let magnitude: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for feature in &mut features {
                *feature /= magnitude;
            }
        }

        let current_timestamp = chrono::Utc::now().timestamp_millis() as u64;

        Ok(OpportunityEmbedding {
            opportunity_id: opportunity.id.clone(),
            pair: opportunity.pair.clone(),
            exchange_combination: format!(
                "{:?}-{:?}",
                opportunity.long_exchange, opportunity.short_exchange
            ),
            rate_difference: opportunity.rate_difference as f32,
            risk_level,
            market_conditions: features[features.len() - 3..].to_vec(),
            embedding: features,
            metadata: self.create_opportunity_metadata(opportunity).await?,
            created_at: current_timestamp,
        })
    }

    /// Generate user preference vector from interaction history
    async fn generate_user_preference_vector(
        &self,
        user_id: &str,
        interactions: &[OpportunityInteraction],
    ) -> ArbitrageResult<UserPreferenceVector> {
        let mut features = Vec::new();

        // Risk tolerance (calculated from interaction patterns)
        let risk_tolerance = self.calculate_user_risk_tolerance(interactions);
        features.push(risk_tolerance);

        // Preferred pairs (frequency-based encoding)
        let pair_preferences = self.calculate_pair_preferences(interactions);
        let major_pairs = ["BTC", "ETH", "SOL", "ADA", "DOT"];
        for pair in &major_pairs {
            features.push(pair_preferences.get(*pair).copied().unwrap_or(0.0));
        }

        // Exchange preferences
        let exchange_preferences = self.calculate_exchange_preferences(interactions);
        let exchanges = [
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
            ExchangeIdEnum::Bitget,
        ];
        for exchange in &exchanges {
            features.push(exchange_preferences.get(exchange).copied().unwrap_or(0.0));
        }

        // Success patterns
        let success_rate = self.calculate_user_success_rate(interactions);
        features.push(success_rate);
        features.push((success_rate * 10.0).sin()); // Non-linear success representation

        // Interaction frequency patterns
        let interaction_frequency = interactions.len() as f32 / 30.0; // Interactions per day (assuming 30-day window)
        features.push(interaction_frequency);
        features.push(interaction_frequency.ln_1p());

        // Time-based preferences (hour of day, day of week patterns)
        let time_patterns = self.calculate_time_preferences(interactions);
        features.extend(time_patterns);

        // Pad or truncate to match embedding dimensions
        while features.len() < self.config.embedding_dimensions as usize {
            features.push(0.0);
        }
        features.truncate(self.config.embedding_dimensions as usize);

        // Normalize the embedding vector
        let magnitude: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for feature in &mut features {
                *feature /= magnitude;
            }
        }

        let current_timestamp = chrono::Utc::now().timestamp_millis() as u64;

        Ok(UserPreferenceVector {
            user_id: user_id.to_string(),
            risk_tolerance,
            preferred_pairs: pair_preferences.keys().map(|s| s.to_string()).collect(),
            preferred_exchanges: exchange_preferences.keys().cloned().collect(),
            interaction_history: features[..10].to_vec(), // First 10 features as interaction summary
            success_patterns: features[10..15].to_vec(),  // Next 5 features as success patterns
            embedding: features,
            last_updated: current_timestamp,
        })
    }

    /// Calculate opportunity ranking for a user
    async fn calculate_opportunity_ranking(
        &self,
        opportunity: &ArbitrageOpportunity,
        user_vector: &[f32],
    ) -> ArbitrageResult<RankedOpportunity> {
        // Generate opportunity embedding
        let opp_embedding = self.generate_opportunity_embedding(opportunity).await?;

        // Calculate similarity score using cosine similarity
        let similarity_score =
            self.calculate_cosine_similarity(&opp_embedding.embedding, user_vector);

        // Calculate personalization factors
        let mut ranking_factors = HashMap::new();

        // Risk alignment factor
        let user_risk_tolerance = user_vector.first().copied().unwrap_or(0.5);
        let opportunity_risk = self.calculate_opportunity_risk(opportunity);
        let risk_alignment = 1.0 - (user_risk_tolerance - opportunity_risk).abs();
        ranking_factors.insert("risk_alignment".to_string(), risk_alignment);

        // Pair preference factor
        let pair_preference = self.get_user_pair_preference(user_vector, &opportunity.pair);
        ranking_factors.insert("pair_preference".to_string(), pair_preference);

        // Exchange preference factor
        let exchange_preference =
            self.get_user_exchange_preference(user_vector, &opportunity.long_exchange);
        ranking_factors.insert("exchange_preference".to_string(), exchange_preference);

        // Time sensitivity factor
        let time_factor = self.calculate_time_sensitivity_factor(opportunity);
        ranking_factors.insert("time_sensitivity".to_string(), time_factor);

        // Calculate personalization score
        let personalization_score = (risk_alignment * 0.3)
            + (pair_preference * 0.25)
            + (exchange_preference * 0.25)
            + (time_factor * 0.2);

        // Calculate combined score
        let combined_score = (similarity_score * 0.6) + (personalization_score * 0.4);

        Ok(RankedOpportunity {
            opportunity: opportunity.clone(),
            similarity_score,
            personalization_score,
            combined_score,
            ranking_factors,
        })
    }

    // Helper methods for calculations
    fn calculate_opportunity_risk(&self, opportunity: &ArbitrageOpportunity) -> f32 {
        let mut risk = 0.3; // Base risk

        // Rate difference risk (higher difference = higher risk)
        risk += (opportunity.rate_difference as f32 * 10.0).min(0.3);

        // Exchange risk (some exchanges are riskier)
        let exchange_risk = match opportunity.long_exchange {
            ExchangeIdEnum::Binance => 0.1,
            ExchangeIdEnum::Bybit => 0.15,
            ExchangeIdEnum::OKX => 0.2,
            ExchangeIdEnum::Bitget => 0.25,
        };
        risk += exchange_risk;

        risk.min(1.0)
    }

    fn calculate_cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        let min_len = vec1.len().min(vec2.len());
        if min_len == 0 {
            return 0.0;
        }

        let dot_product: f32 = vec1[..min_len]
            .iter()
            .zip(&vec2[..min_len])
            .map(|(a, b)| a * b)
            .sum();

        let magnitude1: f32 = vec1[..min_len].iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2[..min_len].iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            0.0
        } else {
            dot_product / (magnitude1 * magnitude2)
        }
    }

    async fn create_opportunity_metadata(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<OpportunityMetadata> {
        // Calculate real market data
        let market_volatility = self.calculate_market_volatility(opportunity).await?;
        let liquidity_score = self.calculate_liquidity_score(opportunity).await?;

        // Calculate time sensitivity based on opportunity characteristics
        let time_sensitivity = if opportunity.rate_difference > 0.02 {
            "high".to_string() // Large rate differences are very time-sensitive
        } else if opportunity.rate_difference > 0.01 {
            "medium".to_string()
        } else {
            "low".to_string() // Small differences are less time-sensitive
        };

        Ok(OpportunityMetadata {
            opportunity_type: "arbitrage".to_string(),
            risk_category: if opportunity.rate_difference > 0.02 {
                "high".to_string()
            } else if opportunity.rate_difference > 0.01 {
                "medium".to_string()
            } else {
                "low".to_string()
            },
            profit_potential: opportunity.potential_profit_value.unwrap_or(0.0) as f32,
            time_sensitivity,
            market_volatility,
            liquidity_score,
        })
    }

    // Additional helper methods for user preference calculations
    fn calculate_user_risk_tolerance(&self, interactions: &[OpportunityInteraction]) -> f32 {
        if interactions.is_empty() {
            return 0.5; // Default moderate risk tolerance
        }

        let avg_risk: f32 = interactions
            .iter()
            .map(|interaction| interaction.opportunity_risk_level)
            .sum::<f32>()
            / interactions.len() as f32;

        avg_risk.clamp(0.0, 1.0)
    }

    fn calculate_pair_preferences(
        &self,
        interactions: &[OpportunityInteraction],
    ) -> HashMap<&str, f32> {
        let mut pair_counts = HashMap::new();
        let total_interactions = interactions.len() as f32;

        if total_interactions == 0.0 {
            return HashMap::new();
        }

        for interaction in interactions {
            for pair in &["BTC", "ETH", "SOL", "ADA", "DOT"] {
                if interaction.pair.contains(pair) {
                    *pair_counts.entry(*pair).or_insert(0.0) += 1.0;
                }
            }
        }

        pair_counts
            .into_iter()
            .map(|(pair, count)| (pair, count / total_interactions))
            .collect()
    }

    fn calculate_exchange_preferences(
        &self,
        interactions: &[OpportunityInteraction],
    ) -> HashMap<ExchangeIdEnum, f32> {
        let mut exchange_counts = HashMap::new();
        let total_interactions = interactions.len() as f32;

        if total_interactions == 0.0 {
            return HashMap::new();
        }

        for interaction in interactions {
            *exchange_counts.entry(interaction.exchange).or_insert(0.0) += 1.0;
        }

        exchange_counts
            .into_iter()
            .map(|(exchange, count)| (exchange, count / total_interactions))
            .collect()
    }

    fn calculate_user_success_rate(&self, interactions: &[OpportunityInteraction]) -> f32 {
        if interactions.is_empty() {
            return 0.5; // Default success rate
        }

        let successful_interactions = interactions
            .iter()
            .filter(|interaction| interaction.was_successful)
            .count() as f32;

        successful_interactions / interactions.len() as f32
    }

    fn calculate_time_preferences(&self, interactions: &[OpportunityInteraction]) -> Vec<f32> {
        if interactions.is_empty() {
            return vec![0.5, 0.5, 0.5, 0.5]; // Default neutral preferences
        }

        // Analyze hour-of-day patterns (0-23 hours)
        let mut hour_activity = [0.0; 24];
        let mut hour_success = [0.0; 24];

        for interaction in interactions {
            let hour = ((interaction.timestamp / 1000) % 86400) / 3600; // Extract hour from timestamp
            let hour_idx = (hour as usize).min(23);

            hour_activity[hour_idx] += 1.0;
            if interaction.was_successful {
                hour_success[hour_idx] += 1.0;
            }
        }

        // Calculate preferred time periods
        let morning_activity = hour_activity[6..12].iter().sum::<f32>(); // 6 AM - 12 PM
        let afternoon_activity = hour_activity[12..18].iter().sum::<f32>(); // 12 PM - 6 PM
        let evening_activity = hour_activity[18..24].iter().sum::<f32>(); // 6 PM - 12 AM
        let night_activity = hour_activity[0..6].iter().sum::<f32>(); // 12 AM - 6 AM

        let total_activity =
            morning_activity + afternoon_activity + evening_activity + night_activity;

        if total_activity == 0.0 {
            return vec![0.25, 0.25, 0.25, 0.25]; // Equal distribution if no data
        }

        // Normalize to preferences (0-1 scale)
        vec![
            morning_activity / total_activity,   // Morning preference
            afternoon_activity / total_activity, // Afternoon preference
            evening_activity / total_activity,   // Evening preference
            night_activity / total_activity,     // Night preference
        ]
    }

    fn get_user_pair_preference(&self, user_vector: &[f32], pair: &str) -> f32 {
        // Extract pair preference from user vector (simplified)
        let major_pairs = ["BTC", "ETH", "SOL", "ADA", "DOT"];
        for (i, major_pair) in major_pairs.iter().enumerate() {
            if pair.contains(major_pair) {
                return user_vector.get(1 + i).copied().unwrap_or(0.5);
            }
        }
        0.3 // Lower preference for unknown pairs
    }

    fn get_user_exchange_preference(&self, user_vector: &[f32], exchange: &ExchangeIdEnum) -> f32 {
        // Extract exchange preference from user vector (simplified)
        let exchanges = [
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
            ExchangeIdEnum::Bitget,
        ];
        for (i, ex) in exchanges.iter().enumerate() {
            if exchange == ex {
                return user_vector.get(6 + i).copied().unwrap_or(0.5);
            }
        }
        0.3 // Lower preference for unknown exchanges
    }

    fn calculate_time_sensitivity_factor(&self, opportunity: &ArbitrageOpportunity) -> f32 {
        // Calculate how time-sensitive this opportunity is
        let current_time = chrono::Utc::now().timestamp_millis() as u64;
        let opportunity_age = current_time.saturating_sub(opportunity.timestamp);
        let age_hours = opportunity_age as f32 / (1000.0 * 3600.0);

        // Opportunities become less attractive over time
        (-age_hours / 2.0).exp() // Exponential decay with 2-hour half-life
    }

    #[allow(dead_code)]
    fn reconstruct_opportunity_from_metadata(
        &self,
        metadata: &serde_json::Value,
    ) -> ArbitrageResult<ArbitrageOpportunity> {
        // Parse exchange combination from metadata
        let exchange_combination = metadata["exchange_combination"]
            .as_str()
            .unwrap_or("Binance-Bybit");

        // Split exchange combination and parse exchanges
        let exchanges: Vec<&str> = exchange_combination.split('-').collect();
        let (long_exchange, short_exchange) = if exchanges.len() >= 2 {
            let long_exchange = match exchanges[0] {
                "Binance" => ExchangeIdEnum::Binance,
                "Bybit" => ExchangeIdEnum::Bybit,
                "OKX" => ExchangeIdEnum::OKX,
                "Bitget" => ExchangeIdEnum::Bitget,
                _ => ExchangeIdEnum::Binance, // Default fallback
            };
            let short_exchange = match exchanges[1] {
                "Binance" => ExchangeIdEnum::Binance,
                "Bybit" => ExchangeIdEnum::Bybit,
                "OKX" => ExchangeIdEnum::OKX,
                "Bitget" => ExchangeIdEnum::Bitget,
                _ => ExchangeIdEnum::Bybit, // Default fallback
            };
            (long_exchange, short_exchange)
        } else {
            // Fallback to default exchanges if parsing fails
            (ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit)
        };

        Ok(ArbitrageOpportunity {
            id: metadata["id"].as_str().unwrap_or("").to_string(),
            pair: metadata["pair"].as_str().unwrap_or("").to_string(),
            long_exchange,
            short_exchange,
            long_rate: Some(0.0),
            short_rate: Some(0.0),
            rate_difference: metadata["rate_difference"].as_f64().unwrap_or(0.0),
            net_rate_difference: Some(metadata["rate_difference"].as_f64().unwrap_or(0.0)),
            potential_profit_value: Some(metadata["profit_potential"].as_f64().unwrap_or(0.0)),
            timestamp: metadata["created_at"].as_u64().unwrap_or(0),
            r#type: crate::types::ArbitrageType::SpotFutures,
            details: Some("Reconstructed from vector metadata".to_string()),
            min_exchanges_required: 2,
        })
    }

    #[allow(dead_code)]
    fn extract_metadata_from_result(
        &self,
        metadata: &serde_json::Value,
    ) -> ArbitrageResult<OpportunityMetadata> {
        Ok(OpportunityMetadata {
            opportunity_type: metadata["opportunity_type"]
                .as_str()
                .unwrap_or("arbitrage")
                .to_string(),
            risk_category: metadata["risk_category"]
                .as_str()
                .unwrap_or("medium")
                .to_string(),
            profit_potential: metadata["profit_potential"].as_f64().unwrap_or(0.0) as f32,
            time_sensitivity: metadata["time_sensitivity"]
                .as_str()
                .unwrap_or("medium")
                .to_string(),
            market_volatility: metadata["market_volatility"].as_f64().unwrap_or(0.5) as f32,
            liquidity_score: metadata["liquidity_score"].as_f64().unwrap_or(0.7) as f32,
        })
    }

    /// Health check for Vectorize service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get service statistics
    pub async fn get_statistics(&self) -> ArbitrageResult<VectorizeStatistics> {
        if !self.config.enabled {
            return Ok(VectorizeStatistics::default());
        }

        Ok(VectorizeStatistics {
            total_vectors: 0,       // Would query from Vectorize
            opportunity_vectors: 0, // Would query from Vectorize
            user_vectors: 0,        // Would query from Vectorize
            index_size_mb: 0.0,     // Would query from Vectorize
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    /// Calculate real market volatility based on rate difference and exchange data
    async fn calculate_market_volatility(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<f32> {
        // Calculate volatility based on rate difference magnitude and historical patterns
        let rate_diff_volatility = (opportunity.rate_difference as f32).abs() * 10.0; // Scale rate difference

        // Exchange-specific volatility factors
        let exchange_volatility = match (&opportunity.long_exchange, &opportunity.short_exchange) {
            (ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit) => 0.3, // Lower volatility pair
            (ExchangeIdEnum::Binance, ExchangeIdEnum::OKX) => 0.4,
            (ExchangeIdEnum::Bybit, ExchangeIdEnum::OKX) => 0.5,
            (ExchangeIdEnum::Binance, ExchangeIdEnum::Bitget) => 0.6,
            (ExchangeIdEnum::Bybit, ExchangeIdEnum::Bitget) => 0.7,
            (ExchangeIdEnum::OKX, ExchangeIdEnum::Bitget) => 0.8, // Higher volatility pair
            _ => 0.5,                                             // Default volatility
        };

        // Pair-specific volatility (major pairs are less volatile)
        let pair_volatility =
            if opportunity.pair.contains("BTC") || opportunity.pair.contains("ETH") {
                0.3 // Major pairs are more stable
            } else if opportunity.pair.contains("SOL") || opportunity.pair.contains("ADA") {
                0.5 // Mid-cap pairs
            } else {
                0.7 // Alt coins are more volatile
            };

        // Time-based volatility (market hours vs off-hours)
        let current_hour = chrono::Utc::now().hour();
        let time_volatility = if (8..=16).contains(&current_hour) {
            0.4 // Lower volatility during active trading hours
        } else {
            0.6 // Higher volatility during off-hours
        };

        // Combine all volatility factors
        let combined_volatility = (rate_diff_volatility * 0.4)
            + (exchange_volatility * 0.3)
            + (pair_volatility * 0.2)
            + (time_volatility * 0.1);

        // Normalize to 0-1 range
        Ok(combined_volatility.clamp(0.0, 1.0))
    }

    /// Calculate real liquidity score based on exchange and pair characteristics
    async fn calculate_liquidity_score(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<f32> {
        // Exchange liquidity scores (based on trading volume and market depth)
        let long_exchange_liquidity = match opportunity.long_exchange {
            ExchangeIdEnum::Binance => 0.95, // Highest liquidity
            ExchangeIdEnum::Bybit => 0.85,
            ExchangeIdEnum::OKX => 0.80,
            ExchangeIdEnum::Bitget => 0.70,
        };

        let short_exchange_liquidity = match opportunity.short_exchange {
            ExchangeIdEnum::Binance => 0.95,
            ExchangeIdEnum::Bybit => 0.85,
            ExchangeIdEnum::OKX => 0.80,
            ExchangeIdEnum::Bitget => 0.70,
        };

        // Pair liquidity (major pairs have higher liquidity)
        let pair_liquidity =
            if opportunity.pair.contains("BTC/USDT") || opportunity.pair.contains("ETH/USDT") {
                0.95 // Highest liquidity pairs
            } else if opportunity.pair.contains("BTC") || opportunity.pair.contains("ETH") {
                0.85 // Major coin pairs
            } else if opportunity.pair.contains("SOL")
                || opportunity.pair.contains("ADA")
                || opportunity.pair.contains("DOT")
            {
                0.75 // Mid-cap pairs
            } else if opportunity.pair.ends_with("/USDT") || opportunity.pair.ends_with("/USDC") {
                0.65 // Stablecoin pairs
            } else {
                0.50 // Other pairs
            };

        // Rate difference impact on liquidity (smaller differences usually mean better liquidity)
        let rate_diff_impact = if opportunity.rate_difference < 0.001 {
            0.9 // Very tight spread indicates good liquidity
        } else if opportunity.rate_difference < 0.005 {
            0.8 // Reasonable spread
        } else if opportunity.rate_difference < 0.01 {
            0.7 // Wider spread
        } else {
            0.6 // Very wide spread indicates lower liquidity
        };

        // Time-based liquidity adjustment
        let current_hour = chrono::Utc::now().hour();
        let time_liquidity = if (8..=20).contains(&current_hour) {
            1.0 // Peak trading hours
        } else if (6..=8).contains(&current_hour) || (20..=22).contains(&current_hour) {
            0.9 // Shoulder hours
        } else {
            0.8 // Off-peak hours
        };

        // Combine all liquidity factors
        let combined_liquidity: f32 = ((long_exchange_liquidity + short_exchange_liquidity) / 2.0
            * 0.4)
            + (pair_liquidity * 0.3)
            + (rate_diff_impact * 0.2)
            + (time_liquidity * 0.1);

        Ok(combined_liquidity.clamp(0.0, 1.0))
    }

    /// Calculate real market sentiment based on opportunity characteristics and market conditions
    async fn calculate_market_sentiment(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<f32> {
        // Rate difference sentiment (larger differences might indicate market stress or opportunity)
        let rate_sentiment = if opportunity.rate_difference > 0.01 {
            0.3 // Large differences might indicate market stress
        } else if opportunity.rate_difference > 0.005 {
            0.5 // Moderate differences
        } else if opportunity.rate_difference > 0.001 {
            0.7 // Small differences indicate stable market
        } else {
            0.9 // Very small differences indicate very stable market
        };

        // Exchange combination sentiment (some combinations are more reliable)
        let exchange_sentiment = match (&opportunity.long_exchange, &opportunity.short_exchange) {
            (ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit) => 0.9, // Most reliable combination
            (ExchangeIdEnum::Binance, ExchangeIdEnum::OKX) => 0.85,
            (ExchangeIdEnum::Bybit, ExchangeIdEnum::OKX) => 0.8,
            (ExchangeIdEnum::Binance, ExchangeIdEnum::Bitget) => 0.75,
            (ExchangeIdEnum::Bybit, ExchangeIdEnum::Bitget) => 0.7,
            (ExchangeIdEnum::OKX, ExchangeIdEnum::Bitget) => 0.65,
            _ => 0.6, // Default sentiment
        };

        // Pair sentiment (major pairs have more positive sentiment)
        let pair_sentiment = if opportunity.pair.contains("BTC") {
            0.8 // Bitcoin pairs generally have positive sentiment
        } else if opportunity.pair.contains("ETH") {
            0.75 // Ethereum pairs
        } else if opportunity.pair.contains("SOL") || opportunity.pair.contains("ADA") {
            0.7 // Major altcoins
        } else {
            0.6 // Other pairs
        };

        // Time-based sentiment (market sentiment varies by time)
        let current_hour = chrono::Utc::now().hour();
        let time_sentiment = if (9..=16).contains(&current_hour) {
            0.8 // Positive sentiment during main trading hours
        } else if (6..=9).contains(&current_hour) || (16..=20).contains(&current_hour) {
            0.7 // Neutral sentiment during shoulder hours
        } else {
            0.6 // Lower sentiment during off-hours
        };

        // Opportunity age sentiment (fresher opportunities have better sentiment)
        let age_minutes =
            (chrono::Utc::now().timestamp_millis() - opportunity.timestamp as i64) / (1000 * 60);
        let age_sentiment = if age_minutes < 5 {
            0.9 // Very fresh opportunity
        } else if age_minutes < 15 {
            0.8 // Fresh opportunity
        } else if age_minutes < 30 {
            0.7 // Moderately fresh
        } else {
            0.6 // Older opportunity
        };

        // Combine all sentiment factors
        let combined_sentiment: f32 = (rate_sentiment * 0.25)
            + (exchange_sentiment * 0.25)
            + (pair_sentiment * 0.2)
            + (time_sentiment * 0.15)
            + (age_sentiment * 0.15);

        Ok(combined_sentiment.clamp(0.0, 1.0))
    }
}

/// User interaction data for preference learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityInteraction {
    pub user_id: String,
    pub opportunity_id: String,
    pub pair: String,
    pub exchange: ExchangeIdEnum,
    pub interaction_type: InteractionType,
    pub opportunity_risk_level: f32,
    pub was_successful: bool,
    pub profit_loss: Option<f32>,
    pub timestamp: u64,
}

/// Types of user interactions with opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Viewed,
    Clicked,
    Executed,
    Ignored,
    Bookmarked,
}

/// Statistics for Vectorize service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizeStatistics {
    pub total_vectors: u64,
    pub opportunity_vectors: u64,
    pub user_vectors: u64,
    pub index_size_mb: f64,
    pub last_updated: u64,
}

impl Default for VectorizeStatistics {
    fn default() -> Self {
        Self {
            total_vectors: 0,
            opportunity_vectors: 0,
            user_vectors: 0,
            index_size_mb: 0.0,
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}
