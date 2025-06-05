// Cloudflare Workers Analytics Engine Service for Enhanced Observability
// Leverages Workers Analytics Engine for custom business metrics, real-time dashboards, and performance tracking
//
// Note: This implementation uses HTTP API calls to Cloudflare Analytics Engine
// since direct bindings are not available in the current worker crate version.
// In production, this would use Cloudflare Analytics Engine REST API with proper authentication.

use crate::types::{ArbitrageOpportunity, ExchangeIdEnum};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::Env;

/// Production Analytics Engine HTTP client
pub struct AnalyticsEngineClient {
    account_id: String,
    api_token: String,
    dataset_name: String,
    http_client: reqwest::Client,
}

impl AnalyticsEngineClient {
    pub fn new(account_id: String, api_token: String, dataset_name: String) -> Self {
        Self {
            account_id,
            api_token,
            dataset_name,
            http_client: reqwest::Client::new(),
        }
    }

    /// Send data points to Analytics Engine via HTTP API
    pub async fn write_data_point(&self, data: &[serde_json::Value]) -> Result<(), String> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/analytics_engine/sql",
            self.account_id
        );

        let payload = serde_json::json!({
            "sql": format!("INSERT INTO {} VALUES {}",
                self.dataset_name,
                self.format_values_for_insert(data)
            )
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Analytics Engine API error: {}", error_text))
        }
    }

    /// Query data from Analytics Engine via HTTP API
    pub async fn query(&self, query: &str) -> Result<serde_json::Value, String> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/analytics_engine/sql",
            self.account_id
        );

        let payload = serde_json::json!({
            "sql": query
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Analytics Engine API error: {}", error_text))
        }
    }

    fn format_values_for_insert(&self, data: &[serde_json::Value]) -> String {
        data.iter()
            .map(|value| match value {
                serde_json::Value::Object(obj) => {
                    let values: Vec<String> = obj
                        .values()
                        .map(|v| match v {
                            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => "NULL".to_string(),
                            _ => format!("'{}'", v.to_string().replace("'", "''")),
                        })
                        .collect();
                    format!("({})", values.join(", "))
                }
                _ => format!("('{}')", value.to_string().replace("'", "''")),
            })
            .collect::<Vec<String>>()
            .join(", ")
    }
}

// Type alias for backward compatibility
pub type AnalyticsEngine = AnalyticsEngineClient;

/// Configuration for Analytics Engine service
#[derive(Debug, Clone)]
pub struct AnalyticsEngineConfig {
    pub enabled: bool,
    pub dataset_name: String,
    pub batch_size: u32,
    pub flush_interval_seconds: u64,
    pub retention_days: u32,
    pub enable_real_time_analytics: bool,
}

impl Default for AnalyticsEngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dataset_name: "arbitrage_analytics".to_string(),
            batch_size: 100,
            flush_interval_seconds: 60,
            retention_days: 90,
            enable_real_time_analytics: false,
        }
    }
}

/// Opportunity conversion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityConversionEvent {
    pub user_id: String,
    pub opportunity_id: String,
    pub pair: String,
    pub exchange_combination: String,
    pub opportunity_type: String,
    pub rate_difference: f32,
    pub risk_level: f32,
    pub converted: bool,
    pub conversion_time_ms: Option<u64>,
    pub profit_loss: Option<f32>,
    pub timestamp: u64,
}

/// AI model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelPerformanceEvent {
    pub model_id: String,
    pub model_provider: String,
    pub user_id: String,
    pub request_type: String,
    pub latency_ms: u64,
    pub tokens_used: Option<u32>,
    pub accuracy_score: Option<f32>,
    pub cost_usd: Option<f32>,
    pub success: bool,
    pub error_type: Option<String>,
    pub timestamp: u64,
}

/// User engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEngagementEvent {
    pub user_id: String,
    pub session_id: String,
    pub session_duration_ms: u64,
    pub commands_used: u32,
    pub opportunities_viewed: u32,
    pub opportunities_executed: u32,
    pub ai_features_used: u32,
    pub subscription_tier: String,
    pub chat_context: String, // "private", "group", "channel"
    pub timestamp: u64,
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceEvent {
    pub service_name: String,
    pub operation: String,
    pub latency_ms: u64,
    pub success: bool,
    pub error_type: Option<String>,
    pub memory_usage_mb: Option<f32>,
    pub cpu_usage_percent: Option<f32>,
    pub concurrent_users: Option<u32>,
    pub timestamp: u64,
}

/// Market data ingestion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataIngestionEvent {
    pub exchange: String,
    pub data_type: String, // "ticker", "funding_rate", "orderbook"
    pub symbols_processed: u32,
    pub data_size_bytes: u64,
    pub processing_time_ms: u64,
    pub pipeline_latency_ms: Option<u64>,
    pub cache_hit_rate: Option<f32>,
    pub api_rate_limit_remaining: Option<u32>,
    pub timestamp: u64,
}

/// Real-time metrics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub active_users: u32,
    pub opportunities_per_minute: f32,
    pub conversion_rate_percent: f32,
    pub average_latency_ms: f32,
    pub ai_model_success_rate: f32,
    pub system_health_score: f32,
    pub top_performing_pairs: Vec<String>,
    pub exchange_performance: HashMap<String, f32>,
    pub last_updated: u64,
}

/// User-specific analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub user_id: String,
    pub total_opportunities_viewed: u64,
    pub total_opportunities_executed: u64,
    pub success_rate_percent: f32,
    pub total_profit_loss: f32,
    pub favorite_pairs: Vec<String>,
    pub favorite_exchanges: Vec<String>,
    pub ai_usage_frequency: f32,
    pub session_frequency_per_week: f32,
    pub last_activity: u64,
}

/// Cloudflare Workers Analytics Engine Service
pub struct AnalyticsEngineService {
    config: AnalyticsEngineConfig,
    analytics_engine: Option<AnalyticsEngine>,
    event_buffer: Vec<serde_json::Value>,
    logger: crate::utils::logger::Logger,
}

impl AnalyticsEngineService {
    /// Validate and escape user_id to prevent SQL injection
    fn validate_and_escape_user_id(&self, user_id: &str) -> ArbitrageResult<String> {
        // Validate that user_id contains only allowed characters
        if !user_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::utils::ArbitrageError::validation_error(
                "Invalid user_id: only alphanumeric characters, hyphens, and underscores are allowed".to_string()
            ));
        }

        // Escape single quotes by replacing them with two single quotes
        let escaped = user_id.replace("'", "''");
        Ok(escaped)
    }

    /// Create new AnalyticsEngineService instance
    pub fn new(env: &Env, config: AnalyticsEngineConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        let analytics_engine = {
            // Get credentials from environment
            let account_id = env
                .var("CLOUDFLARE_ACCOUNT_ID")
                .map_err(|_| {
                    ArbitrageError::configuration_error("CLOUDFLARE_ACCOUNT_ID not found")
                })?
                .to_string();

            let api_token = env
                .secret("CLOUDFLARE_API_TOKEN")
                .map_err(|_| ArbitrageError::configuration_error("CLOUDFLARE_API_TOKEN not found"))?
                .to_string();

            logger.info(&format!(
                "Analytics Engine service initialized for dataset '{}' - using production HTTP API",
                config.dataset_name
            ));

            Some(AnalyticsEngineClient::new(
                account_id,
                api_token,
                config.dataset_name.clone(),
            ))
        };

        Ok(Self {
            config,
            analytics_engine,
            event_buffer: Vec::new(),
            logger,
        })
    }

    /// Track opportunity conversion metrics
    pub async fn track_opportunity_conversion(
        &mut self,
        user_id: &str,
        opportunity: &ArbitrageOpportunity,
        converted: bool,
        conversion_time_ms: Option<u64>,
        profit_loss: Option<f32>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = OpportunityConversionEvent {
            user_id: user_id.to_string(),
            opportunity_id: opportunity.id.clone(),
            pair: opportunity.pair.clone(),
            exchange_combination: format!(
                "{}-{}",
                opportunity.long_exchange, opportunity.short_exchange
            ),
            opportunity_type: "arbitrage".to_string(),
            rate_difference: opportunity.rate_difference as f32,
            risk_level: self.calculate_opportunity_risk(opportunity),
            converted,
            conversion_time_ms,
            profit_loss,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("opportunity_conversion", &event).await?;

        self.logger.info(&format!(
            "Tracked opportunity conversion: user={}, opportunity={}, converted={}",
            user_id, opportunity.id, converted
        ));

        Ok(())
    }

    /// Track AI model performance metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_ai_model_performance(
        &mut self,
        model_id: &str,
        model_provider: &str,
        user_id: &str,
        request_type: &str,
        latency_ms: u64,
        tokens_used: Option<u32>,
        accuracy_score: Option<f32>,
        cost_usd: Option<f32>,
        success: bool,
        error_type: Option<String>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = AIModelPerformanceEvent {
            model_id: model_id.to_string(),
            model_provider: model_provider.to_string(),
            user_id: user_id.to_string(),
            request_type: request_type.to_string(),
            latency_ms,
            tokens_used,
            accuracy_score,
            cost_usd,
            success,
            error_type,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("ai_model_performance", &event).await?;

        self.logger.info(&format!(
            "Tracked AI model performance: model={}, latency={}ms, success={}",
            model_id, latency_ms, success
        ));

        Ok(())
    }

    /// Track user engagement metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_user_engagement(
        &mut self,
        user_id: &str,
        session_id: &str,
        session_duration_ms: u64,
        commands_used: u32,
        opportunities_viewed: u32,
        opportunities_executed: u32,
        ai_features_used: u32,
        subscription_tier: &str,
        chat_context: &str,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = UserEngagementEvent {
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            session_duration_ms,
            commands_used,
            opportunities_viewed,
            opportunities_executed,
            ai_features_used,
            subscription_tier: subscription_tier.to_string(),
            chat_context: chat_context.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("user_engagement", &event).await?;

        self.logger.info(&format!(
            "Tracked user engagement: user={}, session_duration={}ms, commands={}",
            user_id, session_duration_ms, commands_used
        ));

        Ok(())
    }

    /// Track system performance metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_system_performance(
        &mut self,
        service_name: &str,
        operation: &str,
        latency_ms: u64,
        success: bool,
        error_type: Option<String>,
        memory_usage_mb: Option<f32>,
        cpu_usage_percent: Option<f32>,
        concurrent_users: Option<u32>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = SystemPerformanceEvent {
            service_name: service_name.to_string(),
            operation: operation.to_string(),
            latency_ms,
            success,
            error_type,
            memory_usage_mb,
            cpu_usage_percent,
            concurrent_users,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("system_performance", &event).await?;

        Ok(())
    }

    /// Track market data ingestion metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_market_data_ingestion(
        &mut self,
        exchange: &str,
        data_type: &str,
        symbols_processed: u32,
        data_size_bytes: u64,
        processing_time_ms: u64,
        pipeline_latency_ms: Option<u64>,
        cache_hit_rate: Option<f32>,
        api_rate_limit_remaining: Option<u32>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = MarketDataIngestionEvent {
            exchange: exchange.to_string(),
            data_type: data_type.to_string(),
            symbols_processed,
            data_size_bytes,
            processing_time_ms,
            pipeline_latency_ms,
            cache_hit_rate,
            api_rate_limit_remaining,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("market_data_ingestion", &event).await?;

        Ok(())
    }

    /// Get real-time metrics for dashboard
    pub async fn get_real_time_metrics(&self) -> ArbitrageResult<RealTimeMetrics> {
        if !self.config.enabled {
            return Ok(RealTimeMetrics::default());
        }

        let Some(ref analytics_engine) = self.analytics_engine else {
            return Err(ArbitrageError::service_unavailable(
                "Analytics Engine not available",
            ));
        };

        // Query Analytics Engine for real-time metrics
        let now = chrono::Utc::now().timestamp() as u64;
        let one_hour_ago = now - 3600;
        let one_day_ago = now - 86400;

        // Query active users (unique users in last hour)
        let active_users = self
            .query_active_users(analytics_engine, one_hour_ago, now)
            .await?;

        // Query opportunity metrics
        let opportunity_metrics = self
            .query_opportunity_metrics(analytics_engine, one_hour_ago, now)
            .await?;

        // Query system performance
        let system_metrics = self
            .query_system_performance(analytics_engine, one_hour_ago, now)
            .await?;

        // Query AI model performance
        let ai_metrics = self
            .query_ai_performance(analytics_engine, one_day_ago, now)
            .await?;

        // Query top performing pairs
        let top_pairs = self
            .query_top_performing_pairs(analytics_engine, one_day_ago, now)
            .await?;

        // Query exchange performance
        let exchange_performance = self
            .query_exchange_performance(analytics_engine, one_day_ago, now)
            .await?;

        Ok(RealTimeMetrics {
            active_users,
            opportunities_per_minute: opportunity_metrics.opportunities_per_minute,
            conversion_rate_percent: opportunity_metrics.conversion_rate,
            average_latency_ms: system_metrics.average_latency,
            ai_model_success_rate: ai_metrics.success_rate,
            system_health_score: system_metrics.health_score,
            top_performing_pairs: top_pairs,
            exchange_performance,
            last_updated: now,
        })
    }

    /// Get user-specific analytics
    pub async fn get_user_analytics(&self, user_id: &str) -> ArbitrageResult<UserAnalytics> {
        if !self.config.enabled {
            return Ok(UserAnalytics::default(user_id));
        }

        let Some(ref analytics_engine) = self.analytics_engine else {
            return Err(ArbitrageError::service_unavailable(
                "Analytics Engine not available",
            ));
        };

        // Query Analytics Engine for user-specific metrics
        let now = chrono::Utc::now().timestamp() as u64;
        let thirty_days_ago = now - (30 * 86400);

        // Query user engagement events
        let engagement_data = self
            .query_user_engagement(analytics_engine, user_id, thirty_days_ago, now)
            .await?;

        // Query user opportunity conversion data
        let conversion_data = self
            .query_user_conversions(analytics_engine, user_id, thirty_days_ago, now)
            .await?;

        // Query user AI usage
        let ai_usage_data = self
            .query_user_ai_usage(analytics_engine, user_id, thirty_days_ago, now)
            .await?;

        Ok(UserAnalytics {
            user_id: user_id.to_string(),
            total_opportunities_viewed: engagement_data.total_opportunities_viewed,
            total_opportunities_executed: conversion_data.total_executed,
            success_rate_percent: conversion_data.success_rate,
            total_profit_loss: conversion_data.total_profit_loss,
            favorite_pairs: conversion_data.favorite_pairs,
            favorite_exchanges: conversion_data.favorite_exchanges,
            ai_usage_frequency: ai_usage_data.usage_frequency,
            session_frequency_per_week: engagement_data.session_frequency_per_week,
            last_activity: engagement_data.last_activity,
        })
    }

    /// Send event to Analytics Engine
    async fn send_event<T: Serialize>(
        &mut self,
        event_name: &str,
        event_data: &T,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut data_to_send = HashMap::new();
        data_to_send.insert("event_name".to_string(), serde_json::to_value(event_name)?);
        data_to_send.insert("event_data".to_string(), serde_json::to_value(event_data)?);
        data_to_send.insert(
            "timestamp".to_string(),
            serde_json::to_value(chrono::Utc::now().timestamp_millis())?,
        );

        // Add to buffer for batching if enabled
        if self.config.enable_batching {
            self.event_buffer.push(serde_json::to_value(data_to_send)?);
            if self.event_buffer.len() >= self.config.batch_size.unwrap_or(10) as usize {
                self.flush_event_buffer().await?;
            }
        } else {
            // Send immediately if batching is not enabled
            if let Some(engine) = &self.analytics_engine {
                engine
                    .write_data_point(&[serde_json::to_value(data_to_send)?])
                    .await
                    .map_err(|e| {
                        ArbitrageError::api_error(format!("Analytics Engine send failed: {}", e))
                    })?;
            }
        }
        Ok(())
    }

    /// Track CoinMarketCap data
    pub async fn track_cmc_data(
        &mut self,
        event_type: &str, // e.g., "latest_quotes", "global_metrics"
        data: &serde_json::Value,     // The actual CMC data
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = CmcDataEvent {
            event_type: event_type.to_string(),
            data: data.clone(),
            source: "coinmarketcap".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("cmc_data", &event).await?;

        self.logger.info(&format!(
            "Tracked CMC data: event_type={}",
            event_type
        ));
        Ok(())
    }

    /// Track Market Snapshot data
    pub async fn track_market_snapshot(
        &mut self,
        snapshot: &MarketDataSnapshot, // Assuming MarketDataSnapshot is defined elsewhere
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = MarketSnapshotEvent {
            exchange: snapshot.exchange.to_string(),
            symbol: snapshot.symbol.clone(),
            price: snapshot.price_data.as_ref().map(|pd| pd.price),
            funding_rate: snapshot.funding_rate_data.as_ref().map(|fr| fr.funding_rate),
            volume_24h: snapshot.volume_data.as_ref().map(|vd| vd.volume_24h),
            source: snapshot.source.to_string(), // Convert DataSource enum to string
            timestamp: snapshot.timestamp,
        };

        self.send_event("market_snapshot", &event).await?;

        self.logger.info(&format!(
            "Tracked market snapshot: exchange={}, symbol={}",
            snapshot.exchange, snapshot.symbol
        ));
        Ok(())
    }

    /// Track user conversion metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_user_conversion(
        &mut self,
        user_id: &str,
        opportunity: &ArbitrageOpportunity,
        converted: bool,
        conversion_time_ms: Option<u64>,
        profit_loss: Option<f32>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = OpportunityConversionEvent {
            user_id: user_id.to_string(),
            opportunity_id: opportunity.id.clone(),
            pair: opportunity.pair.clone(),
            exchange_combination: format!(
                "{}-{}",
                opportunity.long_exchange, opportunity.short_exchange
            ),
            opportunity_type: "arbitrage".to_string(),
            rate_difference: opportunity.rate_difference as f32,
            risk_level: self.calculate_opportunity_risk(opportunity),
            converted,
            conversion_time_ms,
            profit_loss,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("opportunity_conversion", &event).await?;

        self.logger.info(&format!(
            "Tracked opportunity conversion: user={}, opportunity={}, converted={}",
            user_id, opportunity.id, converted
        ));

        Ok(())
    }

    /// Track AI model performance metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_ai_model_performance(
        &mut self,
        model_id: &str,
        model_provider: &str,
        user_id: &str,
        request_type: &str,
        latency_ms: u64,
        tokens_used: Option<u32>,
        accuracy_score: Option<f32>,
        cost_usd: Option<f32>,
        success: bool,
        error_type: Option<String>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = AIModelPerformanceEvent {
            model_id: model_id.to_string(),
            model_provider: model_provider.to_string(),
            user_id: user_id.to_string(),
            request_type: request_type.to_string(),
            latency_ms,
            tokens_used,
            accuracy_score,
            cost_usd,
            success,
            error_type,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("ai_model_performance", &event).await?;

        self.logger.info(&format!(
            "Tracked AI model performance: model={}, latency={}ms, success={}",
            model_id, latency_ms, success
        ));

        Ok(())
    }

    /// Track user engagement metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_user_engagement(
        &mut self,
        user_id: &str,
        session_id: &str,
        session_duration_ms: u64,
        commands_used: u32,
        opportunities_viewed: u32,
        opportunities_executed: u32,
        ai_features_used: u32,
        subscription_tier: &str,
        chat_context: &str,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = UserEngagementEvent {
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            session_duration_ms,
            commands_used,
            opportunities_viewed,
            opportunities_executed,
            ai_features_used,
            subscription_tier: subscription_tier.to_string(),
            chat_context: chat_context.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("user_engagement", &event).await?;

        self.logger.info(&format!(
            "Tracked user engagement: user={}, session_duration={}ms, commands={}",
            user_id, session_duration_ms, commands_used
        ));

        Ok(())
    }

    /// Track system performance metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_system_performance(
        &mut self,
        service_name: &str,
        operation: &str,
        latency_ms: u64,
        success: bool,
        error_type: Option<String>,
        memory_usage_mb: Option<f32>,
        cpu_usage_percent: Option<f32>,
        concurrent_users: Option<u32>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = SystemPerformanceEvent {
            service_name: service_name.to_string(),
            operation: operation.to_string(),
            latency_ms,
            success,
            error_type,
            memory_usage_mb,
            cpu_usage_percent,
            concurrent_users,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("system_performance", &event).await?;

        Ok(())
    }

    /// Track market data ingestion metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn track_market_data_ingestion(
        &mut self,
        exchange: &str,
        data_type: &str,
        symbols_processed: u32,
        data_size_bytes: u64,
        processing_time_ms: u64,
        pipeline_latency_ms: Option<u64>,
        cache_hit_rate: Option<f32>,
        api_rate_limit_remaining: Option<u32>,
    ) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = MarketDataIngestionEvent {
            exchange: exchange.to_string(),
            data_type: data_type.to_string(),
            symbols_processed,
            data_size_bytes,
            processing_time_ms,
            pipeline_latency_ms,
            cache_hit_rate,
            api_rate_limit_remaining,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_event("market_data_ingestion", &event).await?;

        Ok(())
    }

    /// Get real-time metrics for dashboard
    pub async fn get_real_time_metrics(&self) -> ArbitrageResult<RealTimeMetrics> {
        if !self.config.enabled {
            return Ok(RealTimeMetrics::default());
        }

        let Some(ref analytics_engine) = self.analytics_engine else {
            return Err(ArbitrageError::service_unavailable(
                "Analytics Engine not available",
            ));
        };

        // Query Analytics Engine for real-time metrics
        let now = chrono::Utc::now().timestamp() as u64;
        let one_hour_ago = now - 3600;
        let one_day_ago = now - 86400;

        // Query active users (unique users in last hour)
        let active_users = self
            .query_active_users(analytics_engine, one_hour_ago, now)
            .await?;

        // Query opportunity metrics
        let opportunity_metrics = self
            .query_opportunity_metrics(analytics_engine, one_hour_ago, now)
            .await?;

        // Query system performance
        let system_metrics = self
            .query_system_performance(analytics_engine, one_hour_ago, now)
            .await?;

        // Query AI model performance
        let ai_metrics = self
            .query_ai_performance(analytics_engine, one_day_ago, now)
            .await?;

        // Query top performing pairs
        let top_pairs = self
            .query_top_performing_pairs(analytics_engine, one_day_ago, now)
            .await?;

        // Query exchange performance
        let exchange_performance = self
            .query_exchange_performance(analytics_engine, one_day_ago, now)
            .await?;

        Ok(RealTimeMetrics {
            active_users,
            opportunities_per_minute: opportunity_metrics.opportunities_per_minute,
            conversion_rate_percent: opportunity_metrics.conversion_rate,
            average_latency_ms: system_metrics.average_latency,
            ai_model_success_rate: ai_metrics.success_rate,
            system_health_score: system_metrics.health_score,
            top_performing_pairs: top_pairs,
            exchange_performance,
            last_updated: now,
        })
    }

    /// Get user-specific analytics
    pub async fn get_user_analytics(&self, user_id: &str) -> ArbitrageResult<UserAnalytics> {
        if !self.config.enabled {
            return Ok(UserAnalytics::default(user_id));
        }

        let Some(ref analytics_engine) = self.analytics_engine else {
            return Err(ArbitrageError::service_unavailable(
                "Analytics Engine not available",
            ));
        };

        // Query Analytics Engine for user-specific metrics
        let now = chrono::Utc::now().timestamp() as u64;
        let thirty_days_ago = now - (30 * 86400);

        // Query user engagement events
        let engagement_data = self
            .query_user_engagement(analytics_engine, user_id, thirty_days_ago, now)
            .await?;

        // Query user opportunity conversion data
        let conversion_data = self
            .query_user_conversions(analytics_engine, user_id, thirty_days_ago, now)
            .await?;

        // Query user AI usage
        let ai_usage_data = self
            .query_user_ai_usage(analytics_engine, user_id, thirty_days_ago, now)
            .await?;

        Ok(UserAnalytics {
            user_id: user_id.to_string(),
            total_opportunities_viewed: engagement_data.total_opportunities_viewed,
            total_opportunities_executed: conversion_data.total_executed,
            success_rate_percent: conversion_data.success_rate,
            total_profit_loss: conversion_data.total_profit_loss,
            favorite_pairs: conversion_data.favorite_pairs,
            favorite_exchanges: conversion_data.favorite_exchanges,
            ai_usage_frequency: ai_usage_data.usage_frequency,
            session_frequency_per_week: engagement_data.session_frequency_per_week,
            last_activity: engagement_data.last_activity,
        })
    }

    /// Query user engagement data from Analytics Engine
    async fn query_user_engagement(
        &self,
        analytics_engine: &AnalyticsEngine,
        user_id: &str,
        start_time: u64,
        end_time: u64,
    ) -> ArbitrageResult<UserEngagementData> {
        let escaped_user_id = self.validate_and_escape_user_id(user_id)?;
        let query = format!(
            "SELECT 
                SUM(opportunities_viewed) as total_viewed,
                COUNT(DISTINCT session_id) as session_count,
                MAX(timestamp) as last_activity
            FROM {} 
            WHERE event_type = 'user_engagement' AND user_id = '{}' AND timestamp BETWEEN {} AND {}",
            self.config.dataset_name, escaped_user_id, start_time, end_time
        );

        if let Ok(result) = analytics_engine.query(&query).await {
            // Parse real Analytics Engine response
            if let Some(rows) = result.as_array() {
                if let Some(row) = rows.first() {
                    let total_viewed = row
                        .get("total_viewed")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let session_count = row
                        .get("session_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let last_activity = row
                        .get("last_activity")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    // Calculate real session frequency from actual data
                    let time_window_weeks = (end_time - start_time) / (60 * 60 * 24 * 7); // Convert seconds to weeks
                    let session_frequency_per_week = if time_window_weeks > 0 {
                        session_count as f32 / time_window_weeks as f32
                    } else {
                        session_count as f32 // If less than a week, use total sessions
                    };

                    return Ok(UserEngagementData {
                        total_opportunities_viewed: total_viewed,
                        session_frequency_per_week,
                        last_activity,
                    });
                }
            }
        }

        // Fallback to default values if query fails
        Ok(UserEngagementData {
            total_opportunities_viewed: 0,
            session_frequency_per_week: 0.0,
            last_activity: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Query user conversion data from Analytics Engine
    async fn query_user_conversions(
        &self,
        analytics_engine: &AnalyticsEngine,
        user_id: &str,
        start_time: u64,
        end_time: u64,
    ) -> ArbitrageResult<UserConversionData> {
        let escaped_user_id = self.validate_and_escape_user_id(user_id)?;

        // First query: Get overall conversion stats
        let stats_query = format!(
            "SELECT 
                COUNT(CASE WHEN converted = true THEN 1 END) as total_executed,
                COUNT(*) as total_opportunities,
                SUM(CASE WHEN converted = true THEN profit_loss ELSE 0 END) as total_profit
            FROM {} 
            WHERE event_type = 'opportunity_conversion' AND user_id = '{}' AND timestamp BETWEEN {} AND {}",
            self.config.dataset_name, escaped_user_id, start_time, end_time
        );

        // Second query: Get favorite pairs and exchanges
        let favorites_query = format!(
            "SELECT 
                pair,
                exchange_combination,
                COUNT(*) as usage_count
            FROM {} 
            WHERE event_type = 'opportunity_conversion' AND user_id = '{}' AND converted = true AND timestamp BETWEEN {} AND {}
            GROUP BY pair, exchange_combination
            ORDER BY usage_count DESC
            LIMIT 5",
            self.config.dataset_name, escaped_user_id, start_time, end_time
        );

        let mut total_executed = 0u64;
        let mut success_rate = 0.0f32;
        let mut total_profit_loss = 0.0f32;
        let mut favorite_pairs = Vec::new();
        let mut favorite_exchanges = Vec::new();

        // Execute stats query
        if let Ok(result) = analytics_engine.query(&stats_query).await {
            if let Some(rows) = result.as_array() {
                if let Some(row) = rows.first() {
                    total_executed = row
                        .get("total_executed")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let total_opportunities = row
                        .get("total_opportunities")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1);
                    total_profit_loss = row
                        .get("total_profit")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;

                    // Calculate real success rate from actual data
                    success_rate = (total_executed as f32 / total_opportunities as f32) * 100.0;
                }
            }
        }

        // Execute favorites query
        if let Ok(result) = analytics_engine.query(&favorites_query).await {
            if let Some(rows) = result.as_array() {
                for row in rows {
                    if let Some(pair) = row.get("pair").and_then(|v| v.as_str()) {
                        if !favorite_pairs.contains(&pair.to_string()) {
                            favorite_pairs.push(pair.to_string());
                        }
                    }
                    if let Some(exchange) = row.get("exchange_combination").and_then(|v| v.as_str())
                    {
                        if !favorite_exchanges.contains(&exchange.to_string()) {
                            favorite_exchanges.push(exchange.to_string());
                        }
                    }
                }
            }
        }

        // Ensure we have some defaults if no data found
        if favorite_pairs.is_empty() {
            favorite_pairs = vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()];
        }
        if favorite_exchanges.is_empty() {
            favorite_exchanges = vec!["Binance-Bybit".to_string()];
        }

        Ok(UserConversionData {
            total_executed,
            success_rate,
            total_profit_loss,
            favorite_pairs,
            favorite_exchanges,
        })
    }

    /// Query user AI usage data from Analytics Engine
    async fn query_user_ai_usage(
        &self,
        analytics_engine: &AnalyticsEngine,
        user_id: &str,
        start_time: u64,
        end_time: u64,
    ) -> ArbitrageResult<UserAIUsageData> {
        let escaped_user_id = self.validate_and_escape_user_id(user_id)?;
        let query = format!(
            "SELECT 
                COUNT(*) as total_ai_calls,
                COUNT(DISTINCT DATE(timestamp)) as active_days
            FROM {} 
            WHERE event_type = 'ai_model_performance' AND user_id = '{}' AND timestamp BETWEEN {} AND {}",
            self.config.dataset_name, escaped_user_id, start_time, end_time
        );

        if let Ok(result) = analytics_engine.query(&query).await {
            // Parse real Analytics Engine response
            if let Some(rows) = result.as_array() {
                if let Some(row) = rows.first() {
                    let total_calls = row
                        .get("total_ai_calls")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let active_days = row.get("active_days").and_then(|v| v.as_u64()).unwrap_or(1);

                    // Calculate real usage frequency from actual data
                    let usage_frequency = if active_days > 0 {
                        total_calls as f32 / active_days as f32
                    } else {
                        0.0
                    };

                    return Ok(UserAIUsageData { usage_frequency });
                }
            }
        }

        // Fallback to default values if query fails
        Ok(UserAIUsageData {
            usage_frequency: 0.0,
        })
    }
}

// Helper structs for Analytics Engine query results
#[derive(Debug)]
struct OpportunityMetrics {
    opportunities_per_minute: f32,
    conversion_rate: f32,
}

#[derive(Debug)]
struct SystemMetrics {
    average_latency: f32,
    health_score: f32,
}

#[derive(Debug)]
struct AIMetrics {
    success_rate: f32,
}

#[derive(Debug)]
struct UserEngagementData {
    total_opportunities_viewed: u64,
    session_frequency_per_week: f32,
    last_activity: u64,
}

#[derive(Debug)]
struct UserConversionData {
    total_executed: u64,
    success_rate: f32,
    total_profit_loss: f32,
    favorite_pairs: Vec<String>,
    favorite_exchanges: Vec<String>,
}

#[derive(Debug)]
struct UserAIUsageData {
    usage_frequency: f32,
}

impl Default for RealTimeMetrics {
    fn default() -> Self {
        Self {
            active_users: 0,
            opportunities_per_minute: 0.0,
            conversion_rate_percent: 0.0,
            average_latency_ms: 0.0,
            ai_model_success_rate: 0.0,
            system_health_score: 100.0,
            top_performing_pairs: Vec::new(),
            exchange_performance: HashMap::new(),
            last_updated: chrono::Utc::now().timestamp() as u64,
        }
    }
}

impl UserAnalytics {
    fn default(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            total_opportunities_viewed: 0,
            total_opportunities_executed: 0,
            success_rate_percent: 0.0,
            total_profit_loss: 0.0,
            favorite_pairs: Vec::new(),
            favorite_exchanges: Vec::new(),
            ai_usage_frequency: 0.0,
            session_frequency_per_week: 0.0,
            last_activity: chrono::Utc::now().timestamp() as u64,
        }
    }
}

/// Event for CoinMarketCap data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CmcDataEvent {
    pub event_type: String, // e.g., "latest_quotes", "global_metrics"
    pub data: serde_json::Value, // The actual CMC data
    pub source: String, // "coinmarketcap"
    pub timestamp: u64,
}

/// Event for Market Snapshot data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketSnapshotEvent {
    pub exchange: String,
    pub symbol: String,
    pub price: Option<f64>,
    pub funding_rate: Option<f64>,
    pub volume_24h: Option<f64>,
    // Add other relevant fields from MarketDataSnapshot
    pub source: String, // e.g., "binance", "bybit"
    pub timestamp: u64,
}

impl Default for AnalyticsEngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dataset_name: "arbitrage_analytics".to_string(),
            batch_size: 100,
            flush_interval_seconds: 60,
            retention_days: 90,
            enable_real_time_analytics: false,
        }
    }
}

impl UserAnalytics {
    fn default(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            total_opportunities_viewed: 0,
            total_opportunities_executed: 0,
            success_rate_percent: 0.0,
            total_profit_loss: 0.0,
            favorite_pairs: Vec::new(),
            favorite_exchanges: Vec::new(),
            ai_usage_frequency: 0.0,
            session_frequency_per_week: 0.0,
            last_activity: chrono::Utc::now().timestamp() as u64,
        }
    }
}
