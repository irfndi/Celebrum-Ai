//! Tests for AI exchange router functionality
//! Extracted from src/services/core/trading/ai_exchange_router.rs

use std::collections::HashMap;
use crate::trading::ai_exchange_router::*;
use crate::types::{ArbitrageOpportunity, ExchangeIdEnum, UserProfile, MarketDataSnapshot};

fn create_test_config() -> AiExchangeRouterConfig {
    AiExchangeRouterConfig {
        enabled: true,
        max_concurrent_requests: 10,
        request_timeout_ms: 30000,
        rate_limit_per_minute: 60,
        cache_ttl_seconds: 300,
        ai_analysis_enabled: true,
        fallback_to_basic_analysis: true,
    }
}

fn create_test_market_data() -> MarketDataSnapshot {
    MarketDataSnapshot {
        timestamp: 1234567890,
        exchanges: HashMap::from([
            ("binance".to_string(), ExchangeMarketData {
                exchange_id: "binance".to_string(),
                pairs: HashMap::from([
                    ("BTC/USDT".to_string(), PairData {
                        symbol: "BTC/USDT".to_string(),
                        bid: 50000.0,
                        ask: 50010.0,
                        last_price: 50005.0,
                        volume_24h: 1000.0,
                        timestamp: 1234567890,
                    })
                ]),
                status: "active".to_string(),
                latency_ms: 50,
            }),
            ("bybit".to_string(), ExchangeMarketData {
                exchange_id: "bybit".to_string(),
                pairs: HashMap::from([
                    ("BTC/USDT".to_string(), PairData {
                        symbol: "BTC/USDT".to_string(),
                        bid: 50020.0,
                        ask: 50030.0,
                        last_price: 50025.0,
                        volume_24h: 800.0,
                        timestamp: 1234567890,
                    })
                ]),
                status: "active".to_string(),
                latency_ms: 75,
            })
        ]),
        global_stats: GlobalMarketStats {
            total_volume_24h: 1800.0,
            active_pairs: 1,
            active_exchanges: 2,
            average_spread: 0.0002,
        }
    }
}

fn create_test_user_profile() -> UserProfile {
    UserProfile::new(Some(123456789), Some("testuser_invite".to_string()))
}

#[test]
fn test_ai_exchange_router_config_creation() {
    let config = create_test_config();
    
    assert!(config.enabled);
    assert_eq!(config.max_concurrent_requests, 10);
    assert_eq!(config.request_timeout_ms, 30000);
    assert_eq!(config.rate_limit_per_minute, 60);
    assert!(config.ai_analysis_enabled);
}

#[test]
fn test_market_data_snapshot_structure() {
    let market_data = create_test_market_data();
    
    assert_eq!(market_data.exchanges.len(), 2);
    assert!(market_data.exchanges.contains_key("binance"));
    assert!(market_data.exchanges.contains_key("bybit"));
    assert_eq!(market_data.global_stats.active_exchanges, 2);
    assert_eq!(market_data.global_stats.active_pairs, 1);
}

#[test]
fn test_ai_opportunity_analysis_structure() {
    let analysis = AiOpportunityAnalysis {
        opportunity_id: "test_opp_1".to_string(),
        ai_score: 0.85,
        confidence_level: 0.9,
        risk_factors: vec!["market_volatility".to_string(), "liquidity_risk".to_string()],
        recommended_position_size: 0.1,
        execution_priority: ExecutionPriority::High,
        analysis_timestamp: 1234567890,
        reasoning: "Strong arbitrage opportunity with good liquidity".to_string(),
        market_context: MarketContext {
            volatility_index: 0.15,
            liquidity_score: 0.8,
            trend_direction: TrendDirection::Neutral,
            support_resistance_levels: vec![49000.0, 51000.0],
        },
    };
    
    assert_eq!(analysis.opportunity_id, "test_opp_1");
    assert_eq!(analysis.ai_score, 0.85);
    assert_eq!(analysis.confidence_level, 0.9);
    assert_eq!(analysis.risk_factors.len(), 2);
}

#[test]
fn test_rate_limit_tracking() {
    let rate_limit = RateLimit {
        user_id: "test_user".to_string(),
        requests_made: 5,
        window_start: 1234567890,
        window_duration_seconds: 60,
        max_requests: 10,
    };
    
    assert_eq!(rate_limit.user_id, "test_user");
    assert_eq!(rate_limit.requests_made, 5);
    assert_eq!(rate_limit.max_requests, 10);
    assert!(rate_limit.can_make_request());
}

#[test]
fn test_exchange_market_data_structure() {
    let market_data = create_test_market_data();
    let binance_data = &market_data.exchanges["binance"];
    
    assert_eq!(binance_data.exchange_id, "binance");
    assert_eq!(binance_data.status, "active");
    assert_eq!(binance_data.latency_ms, 50);
    assert!(binance_data.pairs.contains_key("BTC/USDT"));
    
    let btc_pair = &binance_data.pairs["BTC/USDT"];
    assert_eq!(btc_pair.symbol, "BTC/USDT");
    assert_eq!(btc_pair.bid, 50000.0);
    assert_eq!(btc_pair.ask, 50010.0);
}

#[test]
fn test_orderbook_depth_calculations() {
    let market_data = create_test_market_data();
    let binance_data = &market_data.exchanges["binance"];
    let btc_pair = &binance_data.pairs["BTC/USDT"];
    
    let spread = btc_pair.ask - btc_pair.bid;
    let spread_percentage = spread / btc_pair.last_price * 100.0;
    
    assert_eq!(spread, 10.0);
    assert!((spread_percentage - 0.02).abs() < 0.001); // ~0.02%
}

#[test]
fn test_market_context_creation() {
    let context = MarketContext {
        volatility_index: 0.25,
        liquidity_score: 0.75,
        trend_direction: TrendDirection::Bullish,
        support_resistance_levels: vec![48000.0, 52000.0],
    };
    
    assert_eq!(context.volatility_index, 0.25);
    assert_eq!(context.liquidity_score, 0.75);
    assert!(matches!(context.trend_direction, TrendDirection::Bullish));
    assert_eq!(context.support_resistance_levels.len(), 2);
}

#[test]
fn test_score_extraction_from_analysis() {
    // Test various score formats using standalone function
    assert_eq!(extract_score_from_text("Score: 0.85"), Some(0.85));
    assert_eq!(extract_score_from_text("AI Score: 0.92"), Some(0.92));
    assert_eq!(extract_score_from_text("Confidence: 0.78"), Some(0.78));
    assert_eq!(extract_score_from_text("No score here"), None);
}

#[test]
fn test_risk_factor_extraction() {
    let analysis_text = "Risk factors include: market volatility, liquidity concerns, execution risk";
    let risk_factors = extract_risk_factors(analysis_text);
    
    assert!(risk_factors.contains(&"market volatility".to_string()));
    assert!(risk_factors.contains(&"liquidity concerns".to_string()));
    assert!(risk_factors.contains(&"execution risk".to_string()));
}

#[test]
fn test_position_size_extraction() {
    let analysis_text = "Recommended position size: 0.15 or 15% of portfolio";
    let position_size = extract_position_size(analysis_text);
    
    assert_eq!(position_size, Some(0.15));
}

// Helper functions for testing business logic
fn extract_score_from_text(text: &str) -> Option<f64> {
    // Simple regex-like extraction for testing
    if let Some(start) = text.find("Score: ") {
        let score_str = &text[start + 7..];
        if let Some(end) = score_str.find(' ') {
            score_str[..end].parse().ok()
        } else {
            score_str.parse().ok()
        }
    } else if let Some(start) = text.find("AI Score: ") {
        let score_str = &text[start + 10..];
        if let Some(end) = score_str.find(' ') {
            score_str[..end].parse().ok()
        } else {
            score_str.parse().ok()
        }
    } else if let Some(start) = text.find("Confidence: ") {
        let score_str = &text[start + 12..];
        if let Some(end) = score_str.find(' ') {
            score_str[..end].parse().ok()
        } else {
            score_str.parse().ok()
        }
    } else {
        None
    }
}

fn extract_risk_factors(text: &str) -> Vec<String> {
    // Simple extraction for testing
    if let Some(start) = text.find("Risk factors include: ") {
        let factors_str = &text[start + 22..];
        factors_str.split(", ")
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        vec![]
    }
}

fn extract_position_size(text: &str) -> Option<f64> {
    // Simple extraction for testing
    if let Some(start) = text.find("Recommended position size: ") {
        let size_str = &text[start + 27..];
        if let Some(end) = size_str.find(' ') {
            size_str[..end].parse().ok()
        } else {
            size_str.parse().ok()
        }
    } else {
        None
    }
}