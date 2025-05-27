use crate::types::{ArbitrageResult, ArbitrageError};
use crate::services::core::infrastructure::cloudflare_pipelines::CloudflarePipelinesService;
use crate::services::core::logging::Logger;
use worker::kv::KvStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinMarketCapConfig {
    pub api_key: String,
    pub base_url: String,
    pub monthly_credit_limit: u32,     // 10,000 credits/month (hard cap)
    pub daily_credit_target: u32,      // ~333 credits/day
    pub cache_ttl_seconds: u64,        // Cache TTL for aggressive caching
    pub priority_symbols: Vec<String>, // High-priority symbols to track
    pub batch_size: u32,               // Max symbols per request
    pub rate_limit_per_minute: u32,    // 30 requests per minute
    pub endpoints_enabled: u32,        // 14 endpoints enabled
    pub currency_conversions_limit: u32, // 1 conversion per request
    pub has_historical_data: bool,     // No historical data access
}

impl Default for CoinMarketCapConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://pro-api.coinmarketcap.com/v1".to_string(),
            monthly_credit_limit: 10000,        // Hard cap from CMC Basic plan
            daily_credit_target: 333,           // ~10k/30 days
            cache_ttl_seconds: 180,              // 3 minutes (aggressive caching due to limits)
            priority_symbols: vec![
                "BTC".to_string(), "ETH".to_string(), "BNB".to_string(),
                "SOL".to_string(), "XRP".to_string(), "ADA".to_string(),
                "DOGE".to_string(), "AVAX".to_string(), "DOT".to_string(),
                "MATIC".to_string()
            ],
            batch_size: 100,                    // Max symbols per request (conservative)
            rate_limit_per_minute: 30,          // CMC Basic plan: 30 requests/minute
            endpoints_enabled: 14,              // CMC Basic plan: 14 endpoints
            currency_conversions_limit: 1,      // CMC Basic plan: 1 conversion per request
            has_historical_data: false,         // CMC Basic plan: No historical data
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmcQuoteData {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub percent_change_1h: f64,
    pub percent_change_24h: f64,
    pub percent_change_7d: f64,
    pub market_cap: f64,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmcGlobalMetrics {
    pub total_market_cap: f64,
    pub total_volume_24h: f64,
    pub bitcoin_dominance: f64,
    pub active_cryptocurrencies: u32,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsage {
    pub daily_credits_used: u32,
    pub monthly_credits_used: u32,
    pub last_reset_date: String,
    pub last_monthly_reset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_this_minute: u32,
    pub minute_window_start: u64,
    pub can_make_request: bool,
    pub seconds_until_reset: u64,
}

pub struct CoinMarketCapService {
    config: CoinMarketCapConfig,
    kv_store: KvStore,
    pipelines_service: Option<CloudflarePipelinesService>,
    logger: Logger,
}

impl CoinMarketCapService {
    pub fn new(
        config: CoinMarketCapConfig,
        kv_store: KvStore,
        pipelines_service: Option<CloudflarePipelinesService>,
        logger: Logger,
    ) -> Self {
        Self {
            config,
            kv_store,
            pipelines_service,
            logger,
        }
    }

    /// Get latest quotes for priority symbols with smart quota management
    pub async fn get_priority_quotes(&self) -> ArbitrageResult<Vec<CmcQuoteData>> {
        // Check rate limit first
        if !self.check_rate_limit().await? {
            self.logger.warn("CMC rate limit exceeded, using cached data only");
            return self.get_cached_priority_quotes().await;
        }

        // Check quota before making API call
        if !self.check_quota_available(1).await? {
            self.logger.warn("CMC quota exhausted, using cached data only");
            return self.get_cached_priority_quotes().await;
        }

        // Try cache first
        if let Ok(cached) = self.get_cached_priority_quotes().await {
            if !cached.is_empty() {
                self.logger.info("Using cached CMC priority quotes");
                return Ok(cached);
            }
        }

        // Fetch fresh data
        let symbols = self.config.priority_symbols.join(",");
        let quotes = self.fetch_quotes_by_symbol(&symbols).await?;
        
        // Cache the results
        self.cache_priority_quotes(&quotes).await?;
        
        // Store to pipelines for analytics
        self.store_quotes_to_pipeline(&quotes).await?;
        
        // Update quota usage and rate limit
        self.increment_quota_usage(1).await?;
        self.increment_rate_limit().await?;
        
        Ok(quotes)
    }

    /// Get global market metrics with quota management
    pub async fn get_global_metrics(&self) -> ArbitrageResult<CmcGlobalMetrics> {
        // Check rate limit first
        if !self.check_rate_limit().await? {
            self.logger.warn("CMC rate limit exceeded, using cached global metrics");
            return self.get_cached_global_metrics().await;
        }

        // Check quota
        if !self.check_quota_available(1).await? {
            return self.get_cached_global_metrics().await;
        }

        // Try cache first
        if let Ok(cached) = self.get_cached_global_metrics().await {
            self.logger.info("Using cached CMC global metrics");
            return Ok(cached);
        }

        // Fetch fresh data
        let metrics = self.fetch_global_metrics().await?;
        
        // Cache the results
        self.cache_global_metrics(&metrics).await?;
        
        // Store to pipelines
        self.store_global_metrics_to_pipeline(&metrics).await?;
        
        // Update quota usage and rate limit
        self.increment_quota_usage(1).await?;
        self.increment_rate_limit().await?;
        
        Ok(metrics)
    }

    /// Get specific symbol quote with caching
    pub async fn get_symbol_quote(&self, symbol: &str) -> ArbitrageResult<CmcQuoteData> {
        // Try cache first
        let cache_key = format!("cmc_quote:{}", symbol);
        if let Ok(cached_data) = self.kv_store.get(&cache_key).text().await {
            if let Ok(quote) = serde_json::from_str::<CmcQuoteData>(&cached_data) {
                self.logger.info(&format!("Using cached CMC quote for {}", symbol));
                return Ok(quote);
            }
        }

        // Check rate limit
        if !self.check_rate_limit().await? {
            return Err(ArbitrageError::rate_limit_exceeded("CMC rate limit exceeded"));
        }

        // Check quota
        if !self.check_quota_available(1).await? {
            return Err(ArbitrageError::quota_exceeded("CMC quota exhausted"));
        }

        // Fetch fresh data
        let quotes = self.fetch_quotes_by_symbol(symbol).await?;
        if let Some(quote) = quotes.first() {
            // Cache individual quote
            let cache_data = serde_json::to_string(quote)?;
            if let Ok(put_builder) = self.kv_store.put(&cache_key, cache_data) {
                let _ = put_builder.expiration_ttl(self.config.cache_ttl_seconds).execute().await;
            }
            
            // Update quota and rate limit
            self.increment_quota_usage(1).await?;
            self.increment_rate_limit().await?;
            
            Ok(quote.clone())
        } else {
            Err(ArbitrageError::not_found(&format!("Symbol {} not found", symbol)))
        }
    }

    /// Fetch quotes from CMC API
    async fn fetch_quotes_by_symbol(&self, symbols: &str) -> ArbitrageResult<Vec<CmcQuoteData>> {
        let url = format!("{}/cryptocurrency/quotes/latest", self.config.base_url);
        
        let mut headers = Headers::new();
        headers.set("X-CMC_PRO_API_KEY", &self.config.api_key)?;
        headers.set("Accept", "application/json")?;
        
        let request_url = format!("{}?symbol={}&convert=USD", url, symbols);
        
        let request = Request::new_with_init(
            &request_url,
            RequestInit::new()
                .with_method(Method::Get)
                .with_headers(headers),
        )?;

        let mut response = Fetch::Request(request).send().await?;
        
        if response.status_code() != 200 {
            return Err(ArbitrageError::api_error(&format!(
                "CMC API error: {}", response.status_code()
            )));
        }

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        
        self.logger.info(&format!("CMC API response received for symbols: {}", symbols));
        
        // Parse response
        self.parse_quotes_response(&response_json)
    }

    /// Fetch global metrics from CMC API
    async fn fetch_global_metrics(&self) -> ArbitrageResult<CmcGlobalMetrics> {
        let url = format!("{}/global-metrics/quotes/latest", self.config.base_url);
        
        let mut headers = Headers::new();
        headers.set("X-CMC_PRO_API_KEY", &self.config.api_key)?;
        headers.set("Accept", "application/json")?;
        
        let request = Request::new_with_init(
            &url,
            RequestInit::new()
                .with_method(Method::Get)
                .with_headers(headers),
        )?;

        let mut response = Fetch::Request(request).send().await?;
        
        if response.status_code() != 200 {
            return Err(ArbitrageError::api_error(&format!(
                "CMC Global Metrics API error: {}", response.status_code()
            )));
        }

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        
        self.parse_global_metrics_response(&response_json)
    }

    /// Parse CMC quotes API response
    fn parse_quotes_response(&self, response: &serde_json::Value) -> ArbitrageResult<Vec<CmcQuoteData>> {
        let mut quotes = Vec::new();
        
        if let Some(data) = response.get("data") {
            if let Some(data_obj) = data.as_object() {
                for (_symbol, quote_data) in data_obj {
                    if let Some(quote) = quote_data.as_object() {
                        if let Some(usd_quote) = quote.get("quote").and_then(|q| q.get("USD")) {
                            let quote_obj = CmcQuoteData {
                                symbol: quote.get("symbol")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("UNKNOWN")
                                    .to_string(),
                                price: usd_quote.get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0),
                                volume_24h: usd_quote.get("volume_24h")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0),
                                percent_change_1h: usd_quote.get("percent_change_1h")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0),
                                percent_change_24h: usd_quote.get("percent_change_24h")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0),
                                percent_change_7d: usd_quote.get("percent_change_7d")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0),
                                market_cap: usd_quote.get("market_cap")
                                    .and_then(|m| m.as_f64())
                                    .unwrap_or(0.0),
                                last_updated: usd_quote.get("last_updated")
                                    .and_then(|u| u.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            };
                            quotes.push(quote_obj);
                        }
                    }
                }
            }
        }
        
        Ok(quotes)
    }

    /// Parse global metrics response
    fn parse_global_metrics_response(&self, response: &serde_json::Value) -> ArbitrageResult<CmcGlobalMetrics> {
        if let Some(data) = response.get("data") {
            if let Some(quote) = data.get("quote").and_then(|q| q.get("USD")) {
                return Ok(CmcGlobalMetrics {
                    total_market_cap: quote.get("total_market_cap")
                        .and_then(|m| m.as_f64())
                        .unwrap_or(0.0),
                    total_volume_24h: quote.get("total_volume_24h")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    bitcoin_dominance: data.get("btc_dominance")
                        .and_then(|b| b.as_f64())
                        .unwrap_or(0.0),
                    active_cryptocurrencies: data.get("active_cryptocurrencies")
                        .and_then(|a| a.as_u64())
                        .unwrap_or(0) as u32,
                    last_updated: quote.get("last_updated")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
        
        Err(ArbitrageError::parsing_error("Failed to parse CMC global metrics"))
    }

    /// Check if quota is available for API calls
    async fn check_quota_available(&self, credits_needed: u32) -> ArbitrageResult<bool> {
        let usage = self.get_quota_usage().await?;
        
        // Check daily limit
        if usage.daily_credits_used + credits_needed > self.config.daily_credit_target {
            self.logger.warn(&format!(
                "Daily CMC quota would be exceeded: {} + {} > {}",
                usage.daily_credits_used, credits_needed, self.config.daily_credit_target
            ));
            return Ok(false);
        }
        
        // Check monthly limit
        if usage.monthly_credits_used + credits_needed > self.config.monthly_credit_limit {
            self.logger.warn(&format!(
                "Monthly CMC quota would be exceeded: {} + {} > {}",
                usage.monthly_credits_used, credits_needed, self.config.monthly_credit_limit
            ));
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Get current quota usage
    async fn get_quota_usage(&self) -> ArbitrageResult<QuotaUsage> {
        let cache_key = "cmc_quota_usage";
        
        if let Ok(cached_data) = self.kv_store.get(cache_key).text().await {
            if let Ok(usage) = serde_json::from_str::<QuotaUsage>(&cached_data) {
                // Check if we need to reset daily counter
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                if usage.last_reset_date != today {
                    return Ok(QuotaUsage {
                        daily_credits_used: 0,
                        monthly_credits_used: usage.monthly_credits_used,
                        last_reset_date: today,
                        last_monthly_reset: usage.last_monthly_reset,
                    });
                }
                
                // Check if we need to reset monthly counter
                let this_month = chrono::Utc::now().format("%Y-%m").to_string();
                if usage.last_monthly_reset != this_month {
                    return Ok(QuotaUsage {
                        daily_credits_used: 0,
                        monthly_credits_used: 0,
                        last_reset_date: today,
                        last_monthly_reset: this_month,
                    });
                }
                
                return Ok(usage);
            }
        }
        
        // Initialize quota tracking
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let this_month = chrono::Utc::now().format("%Y-%m").to_string();
        
        Ok(QuotaUsage {
            daily_credits_used: 0,
            monthly_credits_used: 0,
            last_reset_date: today,
            last_monthly_reset: this_month,
        })
    }

    /// Increment quota usage
    async fn increment_quota_usage(&self, credits_used: u32) -> ArbitrageResult<()> {
        let mut usage = self.get_quota_usage().await?;
        usage.daily_credits_used += credits_used;
        usage.monthly_credits_used += credits_used;
        
        let cache_key = "cmc_quota_usage";
        let usage_data = serde_json::to_string(&usage)?;
        
        if let Ok(put_builder) = self.kv_store.put(cache_key, usage_data) {
            let _ = put_builder.expiration_ttl(86400).execute().await; // 24 hour TTL
        }
        
        self.logger.info(&format!(
            "CMC quota updated: daily={}/{}, monthly={}/{}",
            usage.daily_credits_used, self.config.daily_credit_target,
            usage.monthly_credits_used, self.config.monthly_credit_limit
        ));
        
        Ok(())
    }

    /// Cache priority quotes
    async fn cache_priority_quotes(&self, quotes: &[CmcQuoteData]) -> ArbitrageResult<()> {
        let cache_key = "cmc_priority_quotes";
        let cache_data = serde_json::to_string(quotes)?;
        
        if let Ok(put_builder) = self.kv_store.put(cache_key, cache_data) {
            let _ = put_builder.expiration_ttl(self.config.cache_ttl_seconds).execute().await;
        }
        
        Ok(())
    }

    /// Get cached priority quotes
    async fn get_cached_priority_quotes(&self) -> ArbitrageResult<Vec<CmcQuoteData>> {
        let cache_key = "cmc_priority_quotes";
        
        if let Ok(cached_data) = self.kv_store.get(cache_key).text().await {
            if let Ok(quotes) = serde_json::from_str::<Vec<CmcQuoteData>>(&cached_data) {
                return Ok(quotes);
            }
        }
        
        Ok(Vec::new())
    }

    /// Cache global metrics
    async fn cache_global_metrics(&self, metrics: &CmcGlobalMetrics) -> ArbitrageResult<()> {
        let cache_key = "cmc_global_metrics";
        let cache_data = serde_json::to_string(metrics)?;
        
        if let Ok(put_builder) = self.kv_store.put(cache_key, cache_data) {
            let _ = put_builder.expiration_ttl(self.config.cache_ttl_seconds).execute().await;
        }
        
        Ok(())
    }

    /// Get cached global metrics
    async fn get_cached_global_metrics(&self) -> ArbitrageResult<CmcGlobalMetrics> {
        let cache_key = "cmc_global_metrics";
        
        if let Ok(cached_data) = self.kv_store.get(cache_key).text().await {
            if let Ok(metrics) = serde_json::from_str::<CmcGlobalMetrics>(&cached_data) {
                return Ok(metrics);
            }
        }
        
        Err(ArbitrageError::not_found("No cached global metrics"))
    }

    /// Store quotes to pipeline for analytics
    async fn store_quotes_to_pipeline(&self, quotes: &[CmcQuoteData]) -> ArbitrageResult<()> {
        if let Some(pipelines) = &self.pipelines_service {
            for quote in quotes {
                let event = serde_json::json!({
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                    "source": "coinmarketcap",
                    "data_type": "quote",
                    "symbol": quote.symbol,
                    "price": quote.price,
                    "volume_24h": quote.volume_24h,
                    "percent_change_1h": quote.percent_change_1h,
                    "percent_change_24h": quote.percent_change_24h,
                    "percent_change_7d": quote.percent_change_7d,
                    "market_cap": quote.market_cap,
                    "last_updated": quote.last_updated
                });
                
                let _ = pipelines.send_event(event).await;
            }
        }
        Ok(())
    }

    /// Store global metrics to pipeline
    async fn store_global_metrics_to_pipeline(&self, metrics: &CmcGlobalMetrics) -> ArbitrageResult<()> {
        if let Some(pipelines) = &self.pipelines_service {
            let event = serde_json::json!({
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "source": "coinmarketcap",
                "data_type": "global_metrics",
                "total_market_cap": metrics.total_market_cap,
                "total_volume_24h": metrics.total_volume_24h,
                "bitcoin_dominance": metrics.bitcoin_dominance,
                "active_cryptocurrencies": metrics.active_cryptocurrencies,
                "last_updated": metrics.last_updated
            });
            
            let _ = pipelines.send_event(event).await;
        }
        Ok(())
    }

    /// Get quota status for monitoring
    pub async fn get_quota_status(&self) -> ArbitrageResult<QuotaUsage> {
        self.get_quota_usage().await
    }

    /// Force refresh priority data (admin function)
    pub async fn force_refresh_priority_data(&self) -> ArbitrageResult<Vec<CmcQuoteData>> {
        if !self.check_quota_available(1).await? {
            return Err(ArbitrageError::quota_exceeded("Insufficient quota for force refresh"));
        }
        
        let symbols = self.config.priority_symbols.join(",");
        let quotes = self.fetch_quotes_by_symbol(&symbols).await?;
        
        self.cache_priority_quotes(&quotes).await?;
        self.store_quotes_to_pipeline(&quotes).await?;
        self.increment_quota_usage(1).await?;
        
        self.logger.info("Force refreshed CMC priority data");
        Ok(quotes)
    }

    /// Check if we can make a request within rate limits (30 requests/minute)
    async fn check_rate_limit(&self) -> ArbitrageResult<bool> {
        let rate_limit_key = "cmc_rate_limit";
        let now = Utc::now().timestamp() as u64;
        let minute_window = now / 60; // Current minute window
        
        if let Ok(rate_data) = self.kv_store.get(rate_limit_key).text().await {
            if let Ok(status) = serde_json::from_str::<RateLimitStatus>(&rate_data) {
                // Check if we're in the same minute window
                if status.minute_window_start == minute_window {
                    return Ok(status.requests_this_minute < self.config.rate_limit_per_minute);
                }
            }
        }
        
        // New minute window or no data, we can make a request
        Ok(true)
    }

    /// Increment rate limit counter
    async fn increment_rate_limit(&self) -> ArbitrageResult<()> {
        let rate_limit_key = "cmc_rate_limit";
        let now = Utc::now().timestamp() as u64;
        let minute_window = now / 60;
        
        let mut requests_this_minute = 1;
        
        // Get current rate limit status
        if let Ok(rate_data) = self.kv_store.get(rate_limit_key).text().await {
            if let Ok(status) = serde_json::from_str::<RateLimitStatus>(&rate_data) {
                if status.minute_window_start == minute_window {
                    requests_this_minute = status.requests_this_minute + 1;
                }
            }
        }
        
        let new_status = RateLimitStatus {
            requests_this_minute,
            minute_window_start: minute_window,
            can_make_request: requests_this_minute < self.config.rate_limit_per_minute,
            seconds_until_reset: 60 - (now % 60),
        };
        
        let status_data = serde_json::to_string(&new_status)?;
        if let Ok(put_builder) = self.kv_store.put(rate_limit_key, status_data) {
            let _ = put_builder.expiration_ttl(120).execute().await; // 2 minutes TTL
        }
        
        Ok(())
    }

    /// Get current rate limit status
    pub async fn get_rate_limit_status(&self) -> ArbitrageResult<RateLimitStatus> {
        let rate_limit_key = "cmc_rate_limit";
        let now = Utc::now().timestamp() as u64;
        let minute_window = now / 60;
        
        if let Ok(rate_data) = self.kv_store.get(rate_limit_key).text().await {
            if let Ok(mut status) = serde_json::from_str::<RateLimitStatus>(&rate_data) {
                // Update timing info
                status.seconds_until_reset = 60 - (now % 60);
                status.can_make_request = status.requests_this_minute < self.config.rate_limit_per_minute;
                
                if status.minute_window_start == minute_window {
                    return Ok(status);
                }
            }
        }
        
        // No data or new minute window
        Ok(RateLimitStatus {
            requests_this_minute: 0,
            minute_window_start: minute_window,
            can_make_request: true,
            seconds_until_reset: 60 - (now % 60),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmc_config_creation() {
        let config = CoinMarketCapConfig::default();
        assert_eq!(config.monthly_credit_limit, 10000);
        assert_eq!(config.daily_credit_target, 333);
        assert!(config.priority_symbols.contains(&"BTC".to_string()));
    }

    #[test]
    fn test_quota_usage_structure() {
        let usage = QuotaUsage {
            daily_credits_used: 100,
            monthly_credits_used: 1500,
            last_reset_date: "2025-01-28".to_string(),
            last_monthly_reset: "2025-01".to_string(),
        };
        
        assert_eq!(usage.daily_credits_used, 100);
        assert_eq!(usage.monthly_credits_used, 1500);
    }

    #[test]
    fn test_cmc_quote_data_structure() {
        let quote = CmcQuoteData {
            symbol: "BTC".to_string(),
            price: 45000.0,
            volume_24h: 1000000000.0,
            percent_change_1h: 0.5,
            percent_change_24h: 2.1,
            percent_change_7d: -1.2,
            market_cap: 850000000000.0,
            last_updated: "2025-01-28T10:00:00Z".to_string(),
        };
        
        assert_eq!(quote.symbol, "BTC");
        assert_eq!(quote.price, 45000.0);
    }
} 