// Market Data Ingestion Service
// Centralized data collection for all market data to address infrastructure gaps
// Implements pipeline-first, cache-fallback, API-last pattern

use crate::types::{ArbitrageResult, ExchangeIdEnum, FundingRateInfo, Ticker};
use crate::utils::{ArbitrageError, logger::Logger};
use crate::services::core::infrastructure::{
    CloudflarePipelinesService, KVService, HybridDataAccessService
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::Env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataIngestionConfig {
    pub enabled: bool,
    pub ingestion_interval_seconds: u64,
    pub pipeline_batch_size: u32,
    pub cache_ttl_seconds: u64,
    pub super_admin_fallback: bool,
}

impl Default for MarketDataIngestionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ingestion_interval_seconds: 30,
            pipeline_batch_size: 100,
            cache_ttl_seconds: 60,
            super_admin_fallback: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataSnapshot {
    pub exchange: ExchangeIdEnum,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: u64,
    pub funding_rate: Option<f64>,
    pub orderbook_depth: Option<OrderbookDepth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookDepth {
    pub bids: Vec<(f64, f64)>, // price, volume
    pub asks: Vec<(f64, f64)>, // price, volume
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionStats {
    pub total_ingested: u64,
    pub pipeline_ingestions: u64,
    pub cache_hits: u64,
    pub api_fallbacks: u64,
    pub last_ingestion: u64,
    pub ingestion_rate_per_minute: f64,
}

/// Centralized Market Data Ingestion Service
/// Addresses the critical gap where services bypass pipelines and make direct API calls
pub struct MarketDataIngestionService {
    config: MarketDataIngestionConfig,
    pipelines_service: Option<CloudflarePipelinesService>,
    kv_service: KVService,
    hybrid_access: HybridDataAccessService,
    logger: Logger,
    stats: IngestionStats,
}

impl MarketDataIngestionService {
    pub fn new(
        env: &Env,
        config: MarketDataIngestionConfig,
        kv_service: KVService,
    ) -> ArbitrageResult<Self> {
        let logger = Logger::new("MarketDataIngestionService");
        
        // Initialize hybrid data access service
        let hybrid_access = HybridDataAccessService::new(env)?;
        
        Ok(Self {
            config,
            pipelines_service: None,
            kv_service,
            hybrid_access,
            logger,
            stats: IngestionStats {
                total_ingested: 0,
                pipeline_ingestions: 0,
                cache_hits: 0,
                api_fallbacks: 0,
                last_ingestion: 0,
                ingestion_rate_per_minute: 0.0,
            },
        })
    }

    pub fn set_pipelines_service(&mut self, pipelines_service: CloudflarePipelinesService) {
        self.pipelines_service = Some(pipelines_service);
    }

    /// Ingest market data from all exchanges using pipeline-first approach
    pub async fn ingest_all_market_data(&mut self) -> ArbitrageResult<u32> {
        if !self.config.enabled {
            return Ok(0);
        }

        let exchanges = vec![
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
        ];

        let symbols = vec![
            "BTC/USDT", "ETH/USDT", "BNB/USDT", "ADA/USDT", "SOL/USDT",
            "XRP/USDT", "DOT/USDT", "AVAX/USDT", "MATIC/USDT", "LINK/USDT"
        ];

        let mut ingested_count = 0;

        for exchange in exchanges {
            for symbol in &symbols {
                match self.ingest_market_data_for_pair(&exchange, symbol).await {
                    Ok(_) => {
                        ingested_count += 1;
                        self.stats.total_ingested += 1;
                    }
                    Err(e) => {
                        self.logger.error(&format!(
                            "Failed to ingest data for {}:{} - {}",
                            exchange.as_str(), symbol, e.message
                        ));
                    }
                }
            }
        }

        self.stats.last_ingestion = chrono::Utc::now().timestamp() as u64;
        self.update_ingestion_rate().await?;

        Ok(ingested_count)
    }

    /// Ingest market data for a specific exchange-symbol pair
    pub async fn ingest_market_data_for_pair(
        &mut self,
        exchange: &ExchangeIdEnum,
        symbol: &str,
    ) -> ArbitrageResult<MarketDataSnapshot> {
        // 1. Try to get fresh data using hybrid access pattern
        let market_data = self.hybrid_access.get_market_data(exchange.as_str(), symbol).await?;

        // 2. Create market data snapshot
        let snapshot = MarketDataSnapshot {
            exchange: exchange.clone(),
            symbol: symbol.to_string(),
            price: market_data.price,
            volume: market_data.volume_24h.unwrap_or(0.0),
            timestamp: chrono::Utc::now().timestamp() as u64,
            funding_rate: market_data.funding_rate,
            orderbook_depth: None, // TODO: Add orderbook data
        };

        // 3. Store to pipeline for historical tracking
        if let Some(ref pipelines) = self.pipelines_service {
            match self.store_to_pipeline(pipelines, &snapshot).await {
                Ok(_) => {
                    self.stats.pipeline_ingestions += 1;
                    self.logger.info(&format!(
                        "Stored market data to pipeline: {}:{}",
                        exchange.as_str(), symbol
                    ));
                }
                Err(e) => {
                    self.logger.warn(&format!(
                        "Failed to store to pipeline: {} - falling back to cache only",
                        e.message
                    ));
                }
            }
        }

        // 4. Cache for fast access
        self.cache_market_data(&snapshot).await?;

        Ok(snapshot)
    }

    /// Store market data to Cloudflare Pipelines for historical tracking
    async fn store_to_pipeline(
        &self,
        pipelines: &CloudflarePipelinesService,
        snapshot: &MarketDataSnapshot,
    ) -> ArbitrageResult<()> {
        let pipeline_data = serde_json::to_string(&serde_json::json!({
            "event_type": "market_data_ingestion",
            "timestamp": snapshot.timestamp,
            "exchange": snapshot.exchange.as_str(),
            "symbol": snapshot.symbol,
            "price": snapshot.price,
            "volume": snapshot.volume,
            "funding_rate": snapshot.funding_rate,
            "data_source": "market_data_ingestion_service"
        }))?;

        pipelines.store_analysis_results("market_data", &pipeline_data).await?;
        Ok(())
    }

    /// Cache market data in KV for fast access
    async fn cache_market_data(&mut self, snapshot: &MarketDataSnapshot) -> ArbitrageResult<()> {
        let cache_key = format!("market_data:{}:{}", snapshot.exchange.as_str(), snapshot.symbol);
        let cache_data = serde_json::to_string(snapshot)?;

        self.kv_service.put(&cache_key, &cache_data, Some(self.config.cache_ttl_seconds)).await?;
        self.stats.cache_hits += 1;

        Ok(())
    }

    /// Get cached market data (used by other services)
    pub async fn get_cached_market_data(
        &self,
        exchange: &ExchangeIdEnum,
        symbol: &str,
    ) -> ArbitrageResult<Option<MarketDataSnapshot>> {
        let cache_key = format!("market_data:{}:{}", exchange.as_str(), symbol);
        
        match self.kv_service.get(&cache_key).await? {
            Some(cached_data) => {
                match serde_json::from_str::<MarketDataSnapshot>(&cached_data) {
                    Ok(snapshot) => Ok(Some(snapshot)),
                    Err(e) => {
                        self.logger.warn(&format!("Failed to parse cached data: {}", e));
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Get funding rates for all exchanges (used by GlobalOpportunityService)
    pub async fn get_all_funding_rates(&mut self) -> ArbitrageResult<HashMap<String, FundingRateInfo>> {
        let mut funding_rates = HashMap::new();

        let exchanges = vec![
            ExchangeIdEnum::Binance,
            ExchangeIdEnum::Bybit,
            ExchangeIdEnum::OKX,
        ];

        for exchange in exchanges {
            match self.hybrid_access.get_funding_rates(exchange.as_str()).await {
                Ok(rates) => {
                    for rate in rates {
                        let key = format!("{}:{}", exchange.as_str(), rate.symbol);
                        funding_rates.insert(key, rate);
                    }
                }
                Err(e) => {
                    self.logger.warn(&format!(
                        "Failed to get funding rates for {}: {}",
                        exchange.as_str(), e.message
                    ));
                }
            }
        }

        Ok(funding_rates)
    }

    /// Update ingestion rate statistics
    async fn update_ingestion_rate(&mut self) -> ArbitrageResult<()> {
        let now = chrono::Utc::now().timestamp() as u64;
        let time_window = 60; // 1 minute

        if self.stats.last_ingestion > 0 {
            let time_diff = now - self.stats.last_ingestion;
            if time_diff > 0 {
                self.stats.ingestion_rate_per_minute = 
                    (self.stats.total_ingested as f64) / (time_diff as f64 / 60.0);
            }
        }

        Ok(())
    }

    /// Get ingestion statistics
    pub fn get_stats(&self) -> &IngestionStats {
        &self.stats
    }

    /// Health check for the ingestion service
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        // Check if we can access KV
        let kv_health = self.kv_service.health_check().await.unwrap_or(false);
        
        // Check if pipelines are available
        let pipelines_health = self.pipelines_service.is_some();
        
        // Check if hybrid access is working
        let hybrid_health = self.hybrid_access.health_check().await.unwrap_or(false);

        Ok(kv_health && hybrid_health)
    }

    /// Start automated ingestion (called by scheduler)
    pub async fn start_automated_ingestion(&mut self) -> ArbitrageResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.logger.info("Starting automated market data ingestion");

        loop {
            match self.ingest_all_market_data().await {
                Ok(count) => {
                    self.logger.info(&format!("Ingested {} market data points", count));
                }
                Err(e) => {
                    self.logger.error(&format!("Ingestion failed: {}", e.message));
                }
            }

            // Wait for next ingestion cycle
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.config.ingestion_interval_seconds
            )).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_ingestion() {
        // Test market data ingestion functionality
        // This would be implemented with mock services
    }

    #[tokio::test]
    async fn test_pipeline_storage() {
        // Test pipeline storage functionality
    }

    #[tokio::test]
    async fn test_cache_operations() {
        // Test KV cache operations
    }
} 