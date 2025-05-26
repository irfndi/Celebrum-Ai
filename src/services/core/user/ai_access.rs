use crate::types::{
    AIAccessLevel, AITemplate, AITemplateType, AITemplateParameters, AIUsageTracker,
    TemplateAccess, UserProfile, ApiKeyProvider
};
use crate::services::core::infrastructure::d1::D1Service;
use crate::services::core::infrastructure::kv::KVService;
use std::collections::HashMap;
use log;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use serde_json::Value;

/// Service for managing AI access levels, usage tracking, and template management
pub struct AIAccessService {
    d1_service: D1Service,
    kv_service: KVService,
}

impl AIAccessService {
    pub fn new(d1_service: D1Service, kv_service: KVService) -> Self {
        Self {
            d1_service,
            kv_service,
        }
    }

    /// Helper function to extract f64 field from database row
    fn get_field_as_f64(row: &Value, field: &str, default: f64) -> f64 {
        row.get(field)
            .and_then(|v| v.as_f64())
            .unwrap_or(default)
    }

    /// Helper function to extract u32 field from database row
    fn get_field_as_u32(row: &Value, field: &str, default: u32) -> u32 {
        Self::get_field_as_f64(row, field, default as f64) as u32
    }

    /// Helper function to extract u64 field from database row
    fn get_field_as_u64(row: &Value, field: &str, default: u64) -> u64 {
        Self::get_field_as_f64(row, field, default as f64) as u64
    }

    /// Helper function to extract string field from database row
    fn get_field_as_string(row: &Value, field: &str) -> Option<String> {
        row.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Helper function to extract JSON field as HashMap from database row
    fn get_field_as_json_map(row: &Value, field: &str) -> HashMap<String, f64> {
        row.get(field)
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// Helper function to extract boolean field from database row
    fn get_field_as_bool(row: &Value, field: &str, default: bool) -> bool {
        row.get(field)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Get user's AI access level with caching
    pub async fn get_user_ai_access_level(&self, user_profile: &UserProfile) -> Result<AIAccessLevel, String> {
        let cache_key = format!("ai_access_level:{}", user_profile.user_id);
        
        // Try to get from cache first
        if let Ok(Some(cached_level)) = self.kv_service.get(&cache_key).await {
            if let Ok(access_level) = serde_json::from_str::<AIAccessLevel>(&cached_level) {
                return Ok(access_level);
            }
        }

        // Calculate access level from user profile
        let access_level = user_profile.get_ai_access_level();

        // Cache the result for 1 hour
        let cache_value = serde_json::to_string(&access_level)
            .map_err(|e| format!("Failed to serialize AI access level: {}", e))?;
        
        let _ = self.kv_service.put(&cache_key, &cache_value, Some(3600)).await;

        Ok(access_level)
    }

    /// Get or create AI usage tracker for a user
    pub async fn get_ai_usage_tracker(&self, user_id: &str, access_level: AIAccessLevel) -> Result<AIUsageTracker, String> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        
        #[cfg(target_arch = "wasm32")]
        {
            let query = "SELECT * FROM ai_usage_tracking WHERE user_id = ? AND date = ?";
            let params = vec![
                JsValue::from_str(user_id),
                JsValue::from_str(&today),
            ];

            match self.d1_service.query_first(query, params).await {
                Ok(Some(row)) => {
                    // Parse existing tracker using helper functions
                    let ai_calls_used = Self::get_field_as_u32(&row, "ai_calls_used", 0);
                    let ai_calls_limit = Self::get_field_as_u32(&row, "ai_calls_limit", 0);
                    let last_reset = Self::get_field_as_u64(&row, "last_reset", 0);
                    let total_cost_usd = Self::get_field_as_f64(&row, "total_cost_usd", 0.0);
                    let cost_breakdown_by_provider = Self::get_field_as_json_map(&row, "cost_breakdown_by_provider");
                    let cost_breakdown_by_feature = Self::get_field_as_json_map(&row, "cost_breakdown_by_feature");

                    Ok(AIUsageTracker {
                        user_id: user_id.to_string(),
                        date: today,
                        ai_calls_used,
                        ai_calls_limit,
                        last_reset,
                        access_level,
                        total_cost_usd,
                        cost_breakdown_by_provider,
                        cost_breakdown_by_feature,
                    })
                },
                Ok(None) => {
                    // Create new tracker
                    let tracker = AIUsageTracker::new(user_id.to_string(), access_level);
                    self.save_ai_usage_tracker(&tracker).await?;
                    Ok(tracker)
                },
                Err(e) => Err(format!("Failed to query AI usage tracker: {}", e)),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM implementation for development/testing
            Ok(AIUsageTracker::new(user_id.to_string(), access_level))
        }
    }

    /// Save AI usage tracker to database
    pub async fn save_ai_usage_tracker(&self, tracker: &AIUsageTracker) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            let cost_breakdown_by_provider = serde_json::to_string(&tracker.cost_breakdown_by_provider)
                .map_err(|e| format!("Failed to serialize provider cost breakdown: {}", e))?;
            
            let cost_breakdown_by_feature = serde_json::to_string(&tracker.cost_breakdown_by_feature)
                .map_err(|e| format!("Failed to serialize feature cost breakdown: {}", e))?;

            let query = r#"
                INSERT OR REPLACE INTO ai_usage_tracking 
                (user_id, date, ai_calls_used, ai_calls_limit, last_reset, total_cost_usd, 
                 cost_breakdown_by_provider, cost_breakdown_by_feature)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            let params = vec![
                JsValue::from_str(&tracker.user_id),
                JsValue::from_str(&tracker.date),
                JsValue::from_f64(tracker.ai_calls_used as f64),
                JsValue::from_f64(tracker.ai_calls_limit as f64),
                JsValue::from_f64(tracker.last_reset as f64),
                JsValue::from_f64(tracker.total_cost_usd),
                JsValue::from_str(&cost_breakdown_by_provider),
                JsValue::from_str(&cost_breakdown_by_feature),
            ];

            self.d1_service.execute_query(query, params).await
                .map_err(|e| format!("Failed to save AI usage tracker: {}", e))?;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM implementation for development/testing
            // In a real implementation, this would save to a local database or file
        }

        Ok(())
    }

    /// Record an AI call and update usage tracking
    pub async fn record_ai_call(
        &self,
        user_id: &str,
        access_level: AIAccessLevel,
        cost_usd: f64,
        provider: String,
        feature: String,
    ) -> Result<bool, String> {
        let mut tracker = self.get_ai_usage_tracker(user_id, access_level).await?;
        
        // Check if daily reset is needed
        if tracker.needs_daily_reset() {
            tracker.reset_daily_counters();
        }

        // Record the AI call
        let success = tracker.record_ai_call(cost_usd, provider, feature);
        
        // Save updated tracker
        self.save_ai_usage_tracker(&tracker).await?;

        // Invalidate cache
        let cache_key = format!("ai_usage_tracker:{}", user_id);
        let _ = self.kv_service.delete(&cache_key).await;

        Ok(success)
    }

    /// Check if user can make an AI call
    pub async fn can_make_ai_call(&self, user_id: &str, access_level: AIAccessLevel) -> Result<bool, String> {
        let tracker = self.get_ai_usage_tracker(user_id, access_level).await?;
        Ok(tracker.can_make_ai_call())
    }

    /// Get remaining AI calls for a user
    pub async fn get_remaining_ai_calls(&self, user_id: &str, access_level: AIAccessLevel) -> Result<u32, String> {
        let tracker = self.get_ai_usage_tracker(user_id, access_level).await?;
        Ok(tracker.get_remaining_calls())
    }

    /// Create a new AI template
    pub async fn create_ai_template(
        &self,
        template_name: String,
        template_type: AITemplateType,
        prompt_template: String,
        parameters: AITemplateParameters,
        created_by: Option<String>,
    ) -> Result<AITemplate, String> {
        let template = if let Some(user_id) = created_by {
            AITemplate::new_user_template(
                template_name,
                template_type,
                prompt_template,
                parameters,
                user_id,
            )
        } else {
            AITemplate::new_system_template(
                template_name,
                template_type,
                prompt_template,
                parameters,
            )
        };

        self.save_ai_template(&template).await?;
        Ok(template)
    }

    /// Save AI template to database
    pub async fn save_ai_template(&self, template: &AITemplate) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            let parameters_json = serde_json::to_string(&template.parameters)
                .map_err(|e| format!("Failed to serialize template parameters: {}", e))?;

            let query = r#"
                INSERT OR REPLACE INTO ai_templates 
                (template_id, template_name, template_type, access_level, prompt_template, 
                 parameters, created_by, is_system_default, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            let params = vec![
                JsValue::from_str(&template.template_id),
                JsValue::from_str(&template.template_name),
                JsValue::from_str(&template.template_type.to_string()),
                JsValue::from_str(&template.access_level.to_string()),
                JsValue::from_str(&template.prompt_template),
                JsValue::from_str(&parameters_json),
                template.created_by.as_ref().map(|s| JsValue::from_str(s)).unwrap_or(JsValue::NULL),
                JsValue::from_bool(template.is_system_default),
                JsValue::from_f64(template.created_at as f64),
                JsValue::from_f64(template.updated_at as f64),
            ];

            self.d1_service.execute_query(query, params).await
                .map_err(|e| format!("Failed to save AI template: {}", e))?;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM implementation for development/testing
        }

        Ok(())
    }

    /// Get AI templates accessible to a user
    pub async fn get_user_ai_templates(
        &self,
        user_id: &str,
        access_level: &AIAccessLevel,
    ) -> Result<Vec<AITemplate>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            let template_access = access_level.get_template_access();
            
            let query = match template_access {
                TemplateAccess::None => {
                    return Ok(vec![]); // No templates for users without access
                },
                TemplateAccess::DefaultOnly => {
                    "SELECT * FROM ai_templates WHERE is_system_default = true"
                },
                TemplateAccess::Full => {
                    "SELECT * FROM ai_templates WHERE is_system_default = true OR created_by = ?"
                },
            };

            let params = if matches!(template_access, TemplateAccess::Full) {
                vec![JsValue::from_str(user_id)]
            } else {
                vec![]
            };

            match self.d1_service.query_all(query, params).await {
                Ok(rows) => {
                    let mut templates = Vec::new();
                    for row in rows {
                        if let Ok(template) = self.parse_ai_template_from_row(&row) {
                            templates.push(template);
                        }
                    }
                    Ok(templates)
                },
                Err(e) => Err(format!("Failed to query AI templates: {}", e)),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM implementation for development/testing
            Ok(vec![])
        }
    }

    /// Parse AI template from database row
    #[cfg(target_arch = "wasm32")]
    fn parse_ai_template_from_row(&self, row: &Value) -> Result<AITemplate, String> {
        let template_id = row.get("template_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing template_id")?
            .to_string();

        let template_name = row.get("template_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing template_name")?
            .to_string();

        let template_type_str = row.get("template_type")
            .and_then(|v| v.as_str())
            .ok_or("Missing template_type")?;

        let template_type = match template_type_str {
            "global_opportunity_analysis" => AITemplateType::GlobalOpportunityAnalysis,
            "personal_opportunity_generation" => AITemplateType::PersonalOpportunityGeneration,
            "trading_decision_support" => AITemplateType::TradingDecisionSupport,
            "risk_assessment" => AITemplateType::RiskAssessment,
            "position_sizing" => AITemplateType::PositionSizing,
            _ => return Err(format!(
                "Invalid template type: '{}'. Valid types are: global_opportunity_analysis, personal_opportunity_generation, trading_decision_support, risk_assessment, position_sizing", 
                template_type_str
            )),
        };

        let access_level_str = row.get("access_level")
            .and_then(|v| v.as_str())
            .ok_or("Missing access_level")?;

        let access_level = match access_level_str {
            "none" => TemplateAccess::None,
            "default_only" => TemplateAccess::DefaultOnly,
            "full" => TemplateAccess::Full,
            _ => return Err(format!("Invalid access level: {}", access_level_str)),
        };

        let prompt_template = row.get("prompt_template")
            .and_then(|v| v.as_str())
            .ok_or("Missing prompt_template")?
            .to_string();

        let parameters_str = row.get("parameters")
            .and_then(|v| v.as_str())
            .ok_or("Missing parameters")?;

        let parameters: AITemplateParameters = serde_json::from_str(parameters_str)
            .map_err(|e| format!("Failed to parse template parameters: {}", e))?;

        let created_by = Self::get_field_as_string(&row, "created_by");
        let is_system_default = Self::get_field_as_bool(&row, "is_system_default", false);

        let created_at = Self::get_field_as_u64(&row, "created_at", 0);
        let updated_at = Self::get_field_as_u64(&row, "updated_at", 0);

        Ok(AITemplate {
            template_id,
            template_name,
            template_type,
            access_level,
            prompt_template,
            parameters,
            created_by,
            is_system_default,
            created_at,
            updated_at,
        })
    }

    /// Initialize default AI templates
    pub async fn initialize_default_templates(&self) -> Result<(), String> {
        let default_templates = vec![
            (
                "Global Opportunity Analysis".to_string(),
                AITemplateType::GlobalOpportunityAnalysis,
                "Analyze this arbitrage opportunity and provide insights on market conditions, risk factors, and execution recommendations. Opportunity details: {opportunity_data}".to_string(),
            ),
            (
                "Personal Opportunity Generation".to_string(),
                AITemplateType::PersonalOpportunityGeneration,
                "Based on the user's trading preferences and market data, generate personalized trading opportunities. User preferences: {user_preferences}, Market data: {market_data}".to_string(),
            ),
            (
                "Trading Decision Support".to_string(),
                AITemplateType::TradingDecisionSupport,
                "Provide trading decision support for this opportunity. Consider risk management, position sizing, and market timing. Opportunity: {opportunity}, User profile: {user_profile}".to_string(),
            ),
            (
                "Risk Assessment".to_string(),
                AITemplateType::RiskAssessment,
                "Assess the risk level of this trading opportunity. Consider market volatility, liquidity, correlation risks, and position concentration. Data: {risk_data}".to_string(),
            ),
            (
                "Position Sizing".to_string(),
                AITemplateType::PositionSizing,
                "Calculate optimal position size for this opportunity based on user's risk tolerance and account balance. User data: {user_data}, Opportunity: {opportunity}".to_string(),
            ),
        ];

        for (name, template_type, prompt) in default_templates {
            let template = AITemplate::new_system_template(
                name,
                template_type,
                prompt,
                AITemplateParameters::default(),
            );
            
            self.save_ai_template(&template).await?;
        }

        Ok(())
    }

    /// Validate AI key for a specific provider with enhanced security
    pub async fn validate_ai_key(
        &self,
        provider: &ApiKeyProvider,
        api_key: &str,
        metadata: &serde_json::Value,
        user_id: &str,
        perform_live_validation: bool,
    ) -> Result<bool, String> {
        // First, perform format validation
        let format_valid = match provider {
            ApiKeyProvider::OpenAI => {
                // OpenAI keys are typically 51 characters starting with sk-
                api_key.starts_with("sk-") && api_key.len() == 51 && api_key.chars().all(|c| c.is_alphanumeric() || c == '-')
            },
            ApiKeyProvider::Anthropic => {
                // Anthropic keys have a specific format
                api_key.starts_with("sk-ant-") && api_key.len() > 20 && api_key.chars().all(|c| c.is_alphanumeric() || c == '-')
            },
            ApiKeyProvider::Custom => {
                // For custom providers, check if base_url is provided in metadata
                metadata.get("base_url").and_then(|v| v.as_str()).is_some()
            },
            ApiKeyProvider::Exchange(_) => {
                return Err("Exchange API keys are not valid for AI services".to_string());
            },
        };

        if !format_valid {
            return match provider {
                ApiKeyProvider::OpenAI => Err("Invalid OpenAI API key format. Expected format: sk-<48 alphanumeric characters>".to_string()),
                ApiKeyProvider::Anthropic => Err("Invalid Anthropic API key format. Expected format: sk-ant-<alphanumeric characters>".to_string()),
                ApiKeyProvider::Custom => Err("Custom AI provider requires base_url in metadata".to_string()),
                _ => Err("Invalid API key format".to_string()),
            };
        }

        // If live validation is not requested, return format validation result
        if !perform_live_validation {
            return Ok(true);
        }

        // Check rate limiting for validation attempts
        if !self.check_validation_rate_limit(user_id).await? {
            return Err("Rate limit exceeded for API key validation. Please try again later.".to_string());
        }

        // Perform live API validation
        let validation_result = match provider {
            ApiKeyProvider::OpenAI => self.validate_openai_key_live(api_key).await,
            ApiKeyProvider::Anthropic => self.validate_anthropic_key_live(api_key).await,
            ApiKeyProvider::Custom => {
                let base_url = metadata.get("base_url").and_then(|v| v.as_str()).unwrap();
                self.validate_custom_key_live(api_key, base_url).await
            },
            _ => Ok(false),
        };

        // Record validation attempt for rate limiting
        self.record_validation_attempt(user_id).await?;

        validation_result
    }

    /// Check rate limiting for API key validation attempts
    async fn check_validation_rate_limit(&self, user_id: &str) -> Result<bool, String> {
        let cache_key = format!("ai_key_validation_rate_limit:{}", user_id);
        
        // Allow 5 validation attempts per hour
        match self.kv_service.get(&cache_key).await {
            Ok(Some(count_str)) => {
                let count: u32 = count_str.parse().unwrap_or(0);
                Ok(count < 5)
            },
            Ok(None) => Ok(true), // No previous attempts
            Err(_) => Ok(true), // Allow on cache errors
        }
    }

    /// Record a validation attempt for rate limiting
    async fn record_validation_attempt(&self, user_id: &str) -> Result<(), String> {
        let cache_key = format!("ai_key_validation_rate_limit:{}", user_id);
        
        match self.kv_service.get(&cache_key).await {
            Ok(Some(count_str)) => {
                let count: u32 = count_str.parse().unwrap_or(0);
                let new_count = count + 1;
                self.kv_service.set(&cache_key, &new_count.to_string(), Some(3600)).await
                    .map_err(|e| format!("Failed to update validation rate limit: {}", e))?;
            },
            Ok(None) => {
                self.kv_service.set(&cache_key, "1", Some(3600)).await
                    .map_err(|e| format!("Failed to set validation rate limit: {}", e))?;
            },
            Err(e) => {
                log::warn!("Failed to check validation rate limit: {}", e);
                // Continue without rate limiting on cache errors
            }
        }
        Ok(())
    }

    /// Validate OpenAI API key by making a test API call
    async fn validate_openai_key_live(&self, api_key: &str) -> Result<bool, String> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(true)
                } else if resp.status() == 401 {
                    Err("Invalid OpenAI API key - authentication failed".to_string())
                } else {
                    Err(format!("OpenAI API validation failed with status: {}", resp.status()))
                }
            },
            Err(e) => {
                log::warn!("OpenAI API validation request failed: {}", e);
                Err("Failed to validate OpenAI API key - network error".to_string())
            }
        }
    }

    /// Validate Anthropic API key by making a test API call
    async fn validate_anthropic_key_live(&self, api_key: &str) -> Result<bool, String> {
        use reqwest::Client;
        
        let client = Client::new();
        let test_payload = serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "test"}]
        });

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&test_payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(true)
                } else if resp.status() == 401 {
                    Err("Invalid Anthropic API key - authentication failed".to_string())
                } else {
                    Err(format!("Anthropic API validation failed with status: {}", resp.status()))
                }
            },
            Err(e) => {
                log::warn!("Anthropic API validation request failed: {}", e);
                Err("Failed to validate Anthropic API key - network error".to_string())
            }
        }
    }

    /// Validate custom API key by making a test API call
    async fn validate_custom_key_live(&self, api_key: &str, base_url: &str) -> Result<bool, String> {
        use reqwest::Client;
        
        let client = Client::new();
        let health_url = format!("{}/health", base_url.trim_end_matches('/'));
        
        let response = client
            .get(&health_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(true)
                } else if resp.status() == 401 {
                    Err("Invalid custom API key - authentication failed".to_string())
                } else {
                    Err(format!("Custom API validation failed with status: {}", resp.status()))
                }
            },
            Err(e) => {
                log::warn!("Custom API validation request failed: {}", e);
                Err("Failed to validate custom API key - network error".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SubscriptionTier, SubscriptionInfo, UserConfiguration, NotificationPreferences};

    #[test]
    fn test_ai_access_level_determination() {
        let mut user_profile = UserProfile::new(Some(123456), None);
        
        // Test free user without AI keys
        let access_level = user_profile.get_ai_access_level();
        assert!(matches!(access_level, AIAccessLevel::FreeWithoutAI { .. }));
        assert_eq!(access_level.get_daily_ai_limits(), 0);
        assert!(!access_level.can_use_ai_analysis());

        // Test free user with AI keys
        user_profile.subscription.tier = SubscriptionTier::Free;
        // Simulate having AI keys by checking the logic
        let access_level_with_ai = AIAccessLevel::FreeWithAI {
            ai_analysis: true,
            custom_templates: false,
            daily_ai_limit: 5,
            global_ai_enhancement: true,
            personal_ai_generation: false,
            template_access: TemplateAccess::DefaultOnly,
        };
        assert_eq!(access_level_with_ai.get_daily_ai_limits(), 5);
        assert!(access_level_with_ai.can_use_ai_analysis());
        assert!(!access_level_with_ai.can_create_custom_templates());

        // Test subscription user with AI keys
        let subscription_access = AIAccessLevel::SubscriptionWithAI {
            ai_analysis: true,
            custom_templates: true,
            daily_ai_limit: 100,
            global_ai_enhancement: true,
            personal_ai_generation: true,
            ai_marketplace: true,
            template_access: TemplateAccess::Full,
        };
        assert_eq!(subscription_access.get_daily_ai_limits(), 100);
        assert!(subscription_access.can_use_ai_analysis());
        assert!(subscription_access.can_create_custom_templates());
        assert!(subscription_access.can_generate_personal_ai_opportunities());
    }

    #[test]
    fn test_ai_usage_tracker() {
        let access_level = AIAccessLevel::FreeWithAI {
            ai_analysis: true,
            custom_templates: false,
            daily_ai_limit: 5,
            global_ai_enhancement: true,
            personal_ai_generation: false,
            template_access: TemplateAccess::DefaultOnly,
        };

        let mut tracker = AIUsageTracker::new("test_user".to_string(), access_level);
        
        // Test initial state
        assert_eq!(tracker.get_remaining_calls(), 5);
        assert!(tracker.can_make_ai_call());

        // Test recording AI calls
        assert!(tracker.record_ai_call(0.01, "openai".to_string(), "analysis".to_string()));
        assert_eq!(tracker.get_remaining_calls(), 4);
        assert_eq!(tracker.total_cost_usd, 0.01);

        // Test reaching limit
        for _ in 0..4 {
            tracker.record_ai_call(0.01, "openai".to_string(), "analysis".to_string());
        }
        assert_eq!(tracker.get_remaining_calls(), 0);
        assert!(!tracker.can_make_ai_call());
        assert!(!tracker.record_ai_call(0.01, "openai".to_string(), "analysis".to_string()));
    }

    #[test]
    fn test_ai_template_creation() {
        let template = AITemplate::new_system_template(
            "Test Template".to_string(),
            AITemplateType::GlobalOpportunityAnalysis,
            "Test prompt: {data}".to_string(),
            AITemplateParameters::default(),
        );

        assert_eq!(template.template_name, "Test Template");
        assert!(template.is_system_default);
        assert!(template.created_by.is_none());
        assert_eq!(template.access_level, TemplateAccess::DefaultOnly);

        let user_template = AITemplate::new_user_template(
            "User Template".to_string(),
            AITemplateType::PersonalOpportunityGeneration,
            "User prompt: {data}".to_string(),
            AITemplateParameters::default(),
            "user123".to_string(),
        );

        assert_eq!(user_template.template_name, "User Template");
        assert!(!user_template.is_system_default);
        assert_eq!(user_template.created_by, Some("user123".to_string()));
        assert_eq!(user_template.access_level, TemplateAccess::Full);
    }
} 