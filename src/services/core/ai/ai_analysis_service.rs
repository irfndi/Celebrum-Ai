use crate::utils::{ArbitrageError, ArbitrageResult};
use serde_json::json;
use std::sync::Arc;
use worker::kv::KvStore;

/// Simplified AI Analysis Service for API endpoints
/// Provides basic AI analysis functionality without complex dependencies
pub struct AiAnalysisService {
    kv_store: KvStore,
    d1_service: Arc<worker::D1Database>,
}

impl AiAnalysisService {
    /// Create new AiAnalysisService
    pub fn new(kv_store: KvStore, d1_service: Arc<worker::D1Database>) -> Self {
        Self {
            kv_store,
            d1_service,
        }
    }

    /// Perform market analysis for a user
    pub async fn analyze_market(&self, user_id: &str) -> ArbitrageResult<serde_json::Value> {
        // Check cache first
        let cache_key = format!("ai_market_analysis:{}", user_id);
        if let Some(cached) = self.kv_store.get(&cache_key).text().await? {
            if let Ok(analysis) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(analysis);
            }
        }

        // Generate fresh analysis
        let analysis = self.generate_market_analysis(user_id).await?;

        // Cache for 30 minutes
        let _ = self
            .kv_store
            .put(&cache_key, serde_json::to_string(&analysis)?)
            .map_err(|e| ArbitrageError::cache_error(e.to_string()))?
            .expiration_ttl(1800)
            .execute()
            .await;

        Ok(analysis)
    }

    /// Generate price predictions
    pub async fn predict_prices(&self, user_id: &str) -> ArbitrageResult<serde_json::Value> {
        let cache_key = format!("ai_price_predictions:{}", user_id);
        if let Some(cached) = self.kv_store.get(&cache_key).text().await? {
            if let Ok(predictions) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(predictions);
            }
        }

        let predictions = self.generate_price_predictions(user_id).await?;

        // Cache for 15 minutes
        let _ = self
            .kv_store
            .put(&cache_key, serde_json::to_string(&predictions)?)
            .map_err(|e| ArbitrageError::cache_error(e.to_string()))?
            .expiration_ttl(900)
            .execute()
            .await;

        Ok(predictions)
    }

    /// Analyze market sentiment
    pub async fn analyze_sentiment(&self, user_id: &str) -> ArbitrageResult<serde_json::Value> {
        let cache_key = format!("ai_sentiment_analysis:{}", user_id);
        if let Some(cached) = self.kv_store.get(&cache_key).text().await? {
            if let Ok(sentiment) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(sentiment);
            }
        }

        let sentiment = self.generate_sentiment_analysis(user_id).await?;

        // Cache for 20 minutes
        let _ = self
            .kv_store
            .put(&cache_key, serde_json::to_string(&sentiment)?)
            .map_err(|e| ArbitrageError::cache_error(e.to_string()))?
            .expiration_ttl(1200)
            .execute()
            .await;

        Ok(sentiment)
    }

    /// Generate market analysis based on current data
    async fn generate_market_analysis(&self, _user_id: &str) -> ArbitrageResult<serde_json::Value> {
        // Get current market data from opportunities
        let query = "SELECT pair, rate_difference, confidence_score, timestamp FROM opportunities WHERE timestamp > ? ORDER BY timestamp DESC LIMIT 20";
        let one_hour_ago = chrono::Utc::now().timestamp_millis() as f64 - 3600000.0;

        let stmt = self.d1_service.prepare(query);
        let result = stmt
            .bind(&[worker::wasm_bindgen::JsValue::from_f64(one_hour_ago)])
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        let rows = result
            .results::<std::collections::HashMap<String, serde_json::Value>>()
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        // Analyze the data
        let total_opportunities = rows.len();
        let mut avg_rate_diff = 0.0;
        let mut avg_confidence = 0.0;
        let mut pair_counts = std::collections::HashMap::new();

        for row in &rows {
            if let (Some(rate_diff), Some(confidence), Some(pair)) = (
                row.get("rate_difference").and_then(|v| v.as_f64()),
                row.get("confidence_score").and_then(|v| v.as_f64()),
                row.get("pair").and_then(|v| v.as_str()),
            ) {
                avg_rate_diff += rate_diff;
                avg_confidence += confidence;
                *pair_counts.entry(pair.to_string()).or_insert(0) += 1;
            }
        }

        if total_opportunities > 0 {
            avg_rate_diff /= total_opportunities as f64;
            avg_confidence /= total_opportunities as f64;
        }

        // Generate insights
        let market_trend = if avg_rate_diff > 0.5 {
            "Bullish - High arbitrage opportunities detected"
        } else if avg_rate_diff > 0.2 {
            "Neutral - Moderate arbitrage activity"
        } else {
            "Bearish - Limited arbitrage opportunities"
        };

        let volatility = if avg_rate_diff > 0.8 {
            "High"
        } else if avg_rate_diff > 0.3 {
            "Medium"
        } else {
            "Low"
        };

        Ok(json!({
            "analysis_type": "market_analysis",
            "timestamp": chrono::Utc::now().timestamp(),
            "market_trend": market_trend,
            "volatility": volatility,
            "total_opportunities": total_opportunities,
            "average_rate_difference": format!("{:.4}%", avg_rate_diff * 100.0),
            "average_confidence": format!("{:.1}%", avg_confidence * 100.0),
            "top_pairs": pair_counts.into_iter()
                .collect::<Vec<_>>()
                .into_iter()
                .take(5)
                .map(|(pair, count)| json!({"pair": pair, "count": count}))
                .collect::<Vec<_>>(),
            "insights": [
                format!("Market showing {} volatility with {} opportunities in the last hour", volatility.to_lowercase(), total_opportunities),
                format!("Average arbitrage spread: {:.4}%", avg_rate_diff * 100.0),
                format!("AI confidence level: {:.1}%", avg_confidence * 100.0)
            ],
            "recommendations": [
                if avg_rate_diff > 0.5 { "Consider increasing position sizes due to high spreads" } else { "Monitor for better entry opportunities" },
                if avg_confidence > 0.7 { "High confidence signals - good time for automated trading" } else { "Lower confidence - consider manual review" },
                "Diversify across multiple pairs to reduce risk"
            ]
        }))
    }

    /// Generate price predictions for major cryptocurrencies
    async fn generate_price_predictions(
        &self,
        _user_id: &str,
    ) -> ArbitrageResult<serde_json::Value> {
        // Get recent price data from opportunities
        let query = "SELECT pair, rate_difference, timestamp FROM opportunities WHERE pair IN ('BTCUSDT', 'ETHUSDT', 'BNBUSDT') AND timestamp > ? ORDER BY timestamp DESC LIMIT 50";
        let six_hours_ago = chrono::Utc::now().timestamp_millis() as f64 - 21600000.0;

        let stmt = self.d1_service.prepare(query);
        let result = stmt
            .bind(&[worker::wasm_bindgen::JsValue::from_f64(six_hours_ago)])
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        let rows = result
            .results::<std::collections::HashMap<String, serde_json::Value>>()
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        // Analyze trends for each pair
        let mut pair_trends = std::collections::HashMap::new();
        for row in &rows {
            if let (Some(pair), Some(rate_diff)) = (
                row.get("pair").and_then(|v| v.as_str()),
                row.get("rate_difference").and_then(|v| v.as_f64()),
            ) {
                pair_trends
                    .entry(pair.to_string())
                    .or_insert(Vec::new())
                    .push(rate_diff);
            }
        }

        let mut predictions = Vec::new();
        for (pair, diffs) in pair_trends {
            let avg_diff = diffs.iter().sum::<f64>() / diffs.len() as f64;
            let trend = if avg_diff > 0.3 {
                "Upward"
            } else if avg_diff < -0.1 {
                "Downward"
            } else {
                "Sideways"
            };

            // Generate confidence based on data consistency
            let variance =
                diffs.iter().map(|x| (x - avg_diff).powi(2)).sum::<f64>() / diffs.len() as f64;
            let confidence = (1.0 - variance.min(1.0)) * 100.0;

            predictions.push(json!({
                "pair": pair,
                "prediction": trend,
                "confidence": format!("{:.1}%", confidence),
                "timeframe": "4-6 hours",
                "data_points": diffs.len(),
                "average_spread": format!("{:.4}%", avg_diff * 100.0)
            }));
        }

        Ok(json!({
            "analysis_type": "price_predictions",
            "timestamp": chrono::Utc::now().timestamp(),
            "predictions": predictions,
            "disclaimer": "Predictions based on arbitrage spread analysis and should not be considered financial advice",
            "model_info": {
                "type": "Arbitrage Spread Analysis",
                "data_window": "6 hours",
                "update_frequency": "15 minutes"
            }
        }))
    }

    /// Generate sentiment analysis based on market activity
    async fn generate_sentiment_analysis(
        &self,
        _user_id: &str,
    ) -> ArbitrageResult<serde_json::Value> {
        // Get recent opportunity data
        let query = "SELECT rate_difference, confidence_score, timestamp FROM opportunities WHERE timestamp > ? ORDER BY timestamp DESC LIMIT 100";
        let two_hours_ago = chrono::Utc::now().timestamp_millis() as f64 - 7200000.0;

        let stmt = self.d1_service.prepare(query);
        let result = stmt
            .bind(&[worker::wasm_bindgen::JsValue::from_f64(two_hours_ago)])
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        let rows = result
            .results::<std::collections::HashMap<String, serde_json::Value>>()
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        // Calculate sentiment metrics
        let total_ops = rows.len() as f64;
        let high_confidence_ops = rows
            .iter()
            .filter(|row| {
                row.get("confidence_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
                    > 0.7
            })
            .count() as f64;

        let high_spread_ops = rows
            .iter()
            .filter(|row| {
                row.get("rate_difference")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
                    > 0.5
            })
            .count() as f64;

        let avg_confidence = rows
            .iter()
            .map(|row| {
                row.get("confidence_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            })
            .sum::<f64>()
            / total_ops.max(1.0);

        let avg_spread = rows
            .iter()
            .map(|row| {
                row.get("rate_difference")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            })
            .sum::<f64>()
            / total_ops.max(1.0);

        // Calculate sentiment score (0-100)
        let sentiment_score = ((avg_confidence * 0.4
            + avg_spread * 0.3
            + (high_confidence_ops / total_ops.max(1.0)) * 0.3)
            * 100.0)
            .min(100.0);

        let sentiment_label = if sentiment_score > 75.0 {
            "Very Bullish"
        } else if sentiment_score > 60.0 {
            "Bullish"
        } else if sentiment_score > 40.0 {
            "Neutral"
        } else if sentiment_score > 25.0 {
            "Bearish"
        } else {
            "Very Bearish"
        };

        Ok(json!({
            "analysis_type": "sentiment_analysis",
            "timestamp": chrono::Utc::now().timestamp(),
            "sentiment_score": format!("{:.1}", sentiment_score),
            "sentiment_label": sentiment_label,
            "metrics": {
                "total_opportunities": total_ops as u32,
                "high_confidence_ratio": format!("{:.1}%", (high_confidence_ops / total_ops.max(1.0)) * 100.0),
                "high_spread_ratio": format!("{:.1}%", (high_spread_ops / total_ops.max(1.0)) * 100.0),
                "average_confidence": format!("{:.1}%", avg_confidence * 100.0),
                "average_spread": format!("{:.4}%", avg_spread * 100.0)
            },
            "key_factors": [
                format!("Market activity: {} opportunities in 2 hours", total_ops as u32),
                format!("Confidence level: {:.1}% average", avg_confidence * 100.0),
                format!("Spread quality: {:.4}% average", avg_spread * 100.0)
            ],
            "interpretation": match sentiment_label {
                "Very Bullish" => "Excellent market conditions with high-quality opportunities",
                "Bullish" => "Good market conditions, favorable for trading",
                "Neutral" => "Mixed signals, proceed with caution",
                "Bearish" => "Challenging conditions, consider reducing exposure",
                "Very Bearish" => "Poor market conditions, avoid high-risk positions",
                _ => "Market analysis inconclusive"
            }
        }))
    }
}
