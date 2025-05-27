// Cloudflare AI Gateway Service for Centralized AI Model Management
// Leverages AI Gateway for model routing, caching, performance optimization, and cost tracking

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::{Fetch, Method, Request, RequestInit};

/// Configuration for AI Gateway service
#[derive(Debug, Clone)]
pub struct AIGatewayConfig {
    pub enabled: bool,
    pub gateway_id: String,
    pub base_url: String,
    pub cache_ttl_seconds: u64,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub cost_tracking_enabled: bool,
}

impl Default for AIGatewayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gateway_id: "arbitrage-ai-gateway".to_string(),
            base_url: "https://gateway.ai.cloudflare.com".to_string(),
            cache_ttl_seconds: 300, // 5 minutes
            timeout_seconds: 30,
            max_retries: 3,
            cost_tracking_enabled: true,
        }
    }
}

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelConfig {
    pub model_id: String,
    pub provider: String, // "openai", "anthropic", "workers-ai"
    pub endpoint: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub cost_per_token: Option<f32>,
}

/// AI request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub model_config: AIModelConfig,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub user_id: String,
    pub request_type: String, // "opportunity_analysis", "market_prediction", "risk_assessment"
    pub priority: RequestPriority,
    pub cache_enabled: bool,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// AI response from gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model_used: String,
    pub tokens_used: Option<u32>,
    pub latency_ms: u64,
    pub cost_usd: Option<f32>,
    pub cached: bool,
    pub request_id: String,
    pub timestamp: u64,
}

/// Model performance analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAnalytics {
    pub model_id: String,
    pub total_requests: u64,
    pub average_latency_ms: f32,
    pub success_rate_percent: f32,
    pub total_cost_usd: f32,
    pub cache_hit_rate_percent: f32,
    pub error_rate_percent: f32,
    pub last_24h_requests: u64,
    pub last_updated: u64,
}

/// Model requirements for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub task_type: String,
    pub max_latency_ms: Option<u64>,
    pub max_cost_per_request: Option<f32>,
    pub min_accuracy_score: Option<f32>,
    pub preferred_providers: Vec<String>,
    pub fallback_enabled: bool,
}

/// AI Gateway routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_model: AIModelConfig,
    pub routing_reason: String,
    pub estimated_cost: Option<f32>,
    pub estimated_latency_ms: u64,
    pub confidence_score: f32,
}

/// Cloudflare AI Gateway Service
pub struct AIGatewayService {
    config: AIGatewayConfig,
    model_configs: HashMap<String, AIModelConfig>,
    model_analytics: HashMap<String, ModelAnalytics>,
    logger: crate::utils::logger::Logger,
}

impl AIGatewayService {
    /// Create new AIGatewayService instance
    pub fn new(config: AIGatewayConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        let mut service = Self {
            config,
            model_configs: HashMap::new(),
            model_analytics: HashMap::new(),
            logger,
        };

        // Initialize default model configurations
        service.initialize_default_models()?;

        Ok(service)
    }

    /// Call AI model through gateway
    pub async fn call_ai_model(&mut self, request: &AIRequest) -> ArbitrageResult<AIResponse> {
        if !self.config.enabled {
            return Err(ArbitrageError::service_unavailable("AI Gateway disabled"));
        }

        let start_time = std::time::Instant::now();

        // Build gateway URL
        let gateway_url = format!(
            "{}/v1/{}/{}",
            self.config.base_url, self.config.gateway_id, request.model_config.provider
        );

        // Prepare request payload
        let payload = self.build_request_payload(request)?;

        // Make request through gateway
        let response = self.make_gateway_request(&gateway_url, &payload).await?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Parse response
        let ai_response = self
            .parse_gateway_response(response, &request.model_config, latency_ms)
            .await?;

        // Update analytics
        self.update_model_analytics(&request.model_config.model_id, &ai_response, true)
            .await?;

        self.logger.info(&format!(
            "AI Gateway request completed: model={}, latency={}ms, tokens={}",
            request.model_config.model_id,
            ai_response.latency_ms,
            ai_response.tokens_used.unwrap_or(0)
        ));

        Ok(ai_response)
    }

    /// Get model performance analytics
    pub async fn get_model_analytics(&self, model_id: &str) -> ArbitrageResult<ModelAnalytics> {
        self.model_analytics.get(model_id).cloned().ok_or_else(|| {
            ArbitrageError::not_found(format!("Model analytics not found: {}", model_id))
        })
    }

    /// Intelligent model routing based on requirements
    pub async fn route_to_best_model(
        &self,
        task_type: &str,
        requirements: &ModelRequirements,
    ) -> ArbitrageResult<RoutingDecision> {
        let available_models = self.get_models_for_task(task_type);

        if available_models.is_empty() {
            return Err(ArbitrageError::not_found(
                "No models available for task type",
            ));
        }

        // Score models based on requirements
        let mut scored_models = Vec::new();
        for model in available_models {
            let score = self.calculate_model_score(&model, requirements).await?;
            scored_models.push((model, score));
        }

        // Sort by score (highest first)
        scored_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_model, score) = scored_models
            .first()
            .ok_or_else(|| ArbitrageError::not_found("No suitable model found"))?;

        let decision = RoutingDecision {
            selected_model: best_model.clone(),
            routing_reason: format!("Best score: {:.2} for task: {}", score, task_type),
            estimated_cost: self.estimate_cost(best_model, requirements).await?,
            estimated_latency_ms: self.estimate_latency(best_model).await?,
            confidence_score: *score,
        };

        self.logger.info(&format!(
            "Model routing decision: model={}, score={:.2}, task={}",
            best_model.model_id, score, task_type
        ));

        Ok(decision)
    }

    /// Get all available models
    pub fn get_available_models(&self) -> Vec<&AIModelConfig> {
        self.model_configs.values().collect()
    }

    /// Add or update model configuration
    pub fn add_model_config(&mut self, config: AIModelConfig) {
        self.model_configs.insert(config.model_id.clone(), config);
    }

    /// Remove model configuration
    pub fn remove_model_config(&mut self, model_id: &str) -> Option<AIModelConfig> {
        self.model_configs.remove(model_id)
    }

    /// Initialize default model configurations
    fn initialize_default_models(&mut self) -> ArbitrageResult<()> {
        // OpenAI GPT-4
        self.add_model_config(AIModelConfig {
            model_id: "gpt-4".to_string(),
            provider: "openai".to_string(),
            endpoint: "/chat/completions".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            cost_per_token: Some(0.00003), // $0.03 per 1K tokens
        });

        // OpenAI GPT-3.5 Turbo
        self.add_model_config(AIModelConfig {
            model_id: "gpt-3.5-turbo".to_string(),
            provider: "openai".to_string(),
            endpoint: "/chat/completions".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            cost_per_token: Some(0.000002), // $0.002 per 1K tokens
        });

        // Anthropic Claude
        self.add_model_config(AIModelConfig {
            model_id: "claude-3-sonnet".to_string(),
            provider: "anthropic".to_string(),
            endpoint: "/messages".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            cost_per_token: Some(0.000015), // $0.015 per 1K tokens
        });

        // Cloudflare Workers AI
        self.add_model_config(AIModelConfig {
            model_id: "llama-2-7b-chat".to_string(),
            provider: "workers-ai".to_string(),
            endpoint: "/llama-2-7b-chat-fp16".to_string(),
            max_tokens: Some(2048),
            temperature: Some(0.7),
            top_p: Some(1.0),
            frequency_penalty: None,
            presence_penalty: None,
            cost_per_token: Some(0.000001), // Very low cost for Workers AI
        });

        Ok(())
    }

    /// Build request payload for gateway
    fn build_request_payload(&self, request: &AIRequest) -> ArbitrageResult<serde_json::Value> {
        let mut payload = serde_json::json!({
            "model": request.model_config.model_id,
            "messages": [
                {
                    "role": "user",
                    "content": request.prompt
                }
            ]
        });

        // Add system prompt if provided
        if let Some(system_prompt) = &request.system_prompt {
            payload["messages"] = serde_json::json!([
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": request.prompt
                }
            ]);
        }

        // Add model-specific parameters
        if let Some(max_tokens) = request.model_config.max_tokens {
            payload["max_tokens"] = serde_json::Value::Number(max_tokens.into());
        }
        if let Some(temperature) = request.model_config.temperature {
            payload["temperature"] = serde_json::Value::Number(
                serde_json::Number::from_f64(temperature as f64).unwrap(),
            );
        }
        if let Some(top_p) = request.model_config.top_p {
            payload["top_p"] =
                serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap());
        }

        Ok(payload)
    }

    /// Make request to AI Gateway
    async fn make_gateway_request(
        &self,
        url: &str,
        payload: &serde_json::Value,
    ) -> ArbitrageResult<worker::Response> {
        let mut headers = worker::Headers::new();
        headers.set("Content-Type", "application/json")?;
        headers.set("User-Agent", "ArbEdge-AI-Gateway/1.0")?;

        let request = Request::new_with_init(
            url,
            RequestInit::new()
                .with_method(Method::Post)
                .with_headers(headers)
                .with_body(Some(payload.to_string().into())),
        )?;

        Fetch::Request(request)
            .send()
            .await
            .map_err(|e| ArbitrageError::network_error(format!("AI Gateway request failed: {}", e)))
    }

    /// Parse gateway response
    async fn parse_gateway_response(
        &self,
        mut response: worker::Response,
        model_config: &AIModelConfig,
        latency_ms: u64,
    ) -> ArbitrageResult<AIResponse> {
        let response_text = response.text().await.map_err(|e| {
            ArbitrageError::parsing_error(format!("Failed to read response: {}", e))
        })?;

        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| ArbitrageError::parsing_error(format!("Invalid JSON response: {}", e)))?;

        // Extract content based on provider
        let content = match model_config.provider.as_str() {
            "openai" => response_json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            "anthropic" => response_json["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            "workers-ai" => response_json["result"]["response"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            _ => response_text,
        };

        // Extract usage information
        let tokens_used = response_json["usage"]["total_tokens"]
            .as_u64()
            .map(|t| t as u32);

        // Calculate cost
        let cost_usd = if let (Some(tokens), Some(cost_per_token)) =
            (tokens_used, model_config.cost_per_token)
        {
            Some((tokens as f32) * cost_per_token / 1000.0) // Cost per 1K tokens
        } else {
            None
        };

        Ok(AIResponse {
            content,
            model_used: model_config.model_id.clone(),
            tokens_used,
            latency_ms,
            cost_usd,
            cached: false, // Gateway would indicate if cached
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Update model analytics
    async fn update_model_analytics(
        &mut self,
        model_id: &str,
        response: &AIResponse,
        success: bool,
    ) -> ArbitrageResult<()> {
        let analytics = self
            .model_analytics
            .entry(model_id.to_string())
            .or_insert_with(|| ModelAnalytics {
                model_id: model_id.to_string(),
                total_requests: 0,
                average_latency_ms: 0.0,
                success_rate_percent: 100.0,
                total_cost_usd: 0.0,
                cache_hit_rate_percent: 0.0,
                error_rate_percent: 0.0,
                last_24h_requests: 0,
                last_updated: chrono::Utc::now().timestamp() as u64,
            });

        // Update metrics
        analytics.total_requests += 1;
        analytics.last_24h_requests += 1;

        // Update average latency
        analytics.average_latency_ms = (analytics.average_latency_ms
            * (analytics.total_requests - 1) as f32
            + response.latency_ms as f32)
            / analytics.total_requests as f32;

        // Update cost
        if let Some(cost) = response.cost_usd {
            analytics.total_cost_usd += cost;
        }

        // Update success rate
        if success {
            analytics.success_rate_percent =
                (analytics.success_rate_percent * (analytics.total_requests - 1) as f32 + 100.0)
                    / analytics.total_requests as f32;
        } else {
            analytics.success_rate_percent = (analytics.success_rate_percent
                * (analytics.total_requests - 1) as f32)
                / analytics.total_requests as f32;
        }

        analytics.error_rate_percent = 100.0 - analytics.success_rate_percent;
        analytics.last_updated = chrono::Utc::now().timestamp() as u64;

        Ok(())
    }

    /// Get models suitable for a specific task
    fn get_models_for_task(&self, _task_type: &str) -> Vec<AIModelConfig> {
        // For now, return all models. In a real implementation,
        // this would filter based on task-specific capabilities
        self.model_configs.values().cloned().collect()
    }

    /// Calculate model score based on requirements
    async fn calculate_model_score(
        &self,
        model: &AIModelConfig,
        requirements: &ModelRequirements,
    ) -> ArbitrageResult<f32> {
        let mut score = 0.0;
        let mut factors = 0;

        // Cost factor
        if let Some(max_cost) = requirements.max_cost_per_request {
            if let Some(cost_per_token) = model.cost_per_token {
                let estimated_cost = cost_per_token * 1000.0; // Assume 1K tokens
                if estimated_cost <= max_cost {
                    score += 1.0 - (estimated_cost / max_cost);
                }
                factors += 1;
            }
        }

        // Latency factor (based on historical data)
        if let Some(max_latency) = requirements.max_latency_ms {
            if let Some(analytics) = self.model_analytics.get(&model.model_id) {
                if analytics.average_latency_ms <= max_latency as f32 {
                    score += 1.0 - (analytics.average_latency_ms / max_latency as f32);
                }
                factors += 1;
            }
        }

        // Provider preference
        if !requirements.preferred_providers.is_empty() {
            if requirements.preferred_providers.contains(&model.provider) {
                score += 1.0;
            }
            factors += 1;
        }

        // Success rate factor
        if let Some(analytics) = self.model_analytics.get(&model.model_id) {
            score += analytics.success_rate_percent / 100.0;
            factors += 1;
        }

        // Return average score
        if factors > 0 {
            Ok(score / factors as f32)
        } else {
            Ok(0.5) // Default score if no factors
        }
    }

    /// Estimate cost for a model and requirements
    async fn estimate_cost(
        &self,
        model: &AIModelConfig,
        _requirements: &ModelRequirements,
    ) -> ArbitrageResult<Option<f32>> {
        // Simple estimation based on average token usage
        if let Some(cost_per_token) = model.cost_per_token {
            Ok(Some(cost_per_token * 1000.0)) // Assume 1K tokens
        } else {
            Ok(None)
        }
    }

    /// Estimate latency for a model
    async fn estimate_latency(&self, model: &AIModelConfig) -> ArbitrageResult<u64> {
        if let Some(analytics) = self.model_analytics.get(&model.model_id) {
            Ok(analytics.average_latency_ms as u64)
        } else {
            // Default estimates by provider
            match model.provider.as_str() {
                "workers-ai" => Ok(500), // Fast local processing
                "openai" => Ok(2000),    // Typical OpenAI latency
                "anthropic" => Ok(3000), // Typical Anthropic latency
                _ => Ok(5000),           // Conservative default
            }
        }
    }

    /// Health check for AI Gateway
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        Ok(self.config.enabled && !self.model_configs.is_empty())
    }
}
