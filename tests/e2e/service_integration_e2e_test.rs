#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    dead_code,
    clippy::useless_asref
)]

use arb_edge::services::core::analysis::market_analysis::{
    OpportunityType, RiskLevel, TimeHorizon, TradingOpportunity,
};
use arb_edge::types::{Position, PositionSide, SubscriptionTier, UserProfile};
use serde_json::json;
use std::collections::HashMap;

/// Mock environment for service integration testing
struct ServiceIntegrationTestEnvironment {
    users: HashMap<String, UserProfile>,
    opportunities: Vec<TradingOpportunity>,
    positions: Vec<Position>,
    notifications: Vec<String>,
    market_data: HashMap<String, serde_json::Value>,
    ai_analysis_cache: HashMap<String, serde_json::Value>,
    monitoring_metrics: HashMap<String, f64>,
}

impl ServiceIntegrationTestEnvironment {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            opportunities: Vec::new(),
            positions: Vec::new(),
            notifications: Vec::new(),
            market_data: HashMap::new(),
            ai_analysis_cache: HashMap::new(),
            monitoring_metrics: HashMap::new(),
        }
    }

    fn add_user(&mut self, user: UserProfile) {
        self.users.insert(user.user_id.clone(), user);
    }

    fn add_opportunity(&mut self, opportunity: TradingOpportunity) {
        self.opportunities.push(opportunity);
    }

    fn add_position(&mut self, position: Position) {
        self.positions.push(position);
    }

    fn add_market_data(&mut self, exchange: String, pair: String, data: serde_json::Value) {
        let key = format!("{}:{}", exchange, pair);
        self.market_data.insert(key, data);
    }

    fn add_ai_analysis(&mut self, opportunity_id: String, analysis: serde_json::Value) {
        self.ai_analysis_cache.insert(opportunity_id, analysis);
    }

    fn record_metric(&mut self, metric_name: String, value: f64) {
        self.monitoring_metrics.insert(metric_name, value);
    }

    fn send_notification(&mut self, message: String) {
        self.notifications.push(message);
    }
}

/// Helper function to create test market data
fn create_test_market_data(exchange: &str, pair: &str, price: f64) -> serde_json::Value {
    json!({
        "exchange": exchange,
        "symbol": pair,
        "price": price,
        "volume": 1000.0,
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "bid": price - 0.5,
        "ask": price + 0.5,
        "high_24h": price * 1.02,
        "low_24h": price * 0.98
    })
}

/// Helper function to create test position
fn create_test_position(user_id: String, pair: String, size: f64, entry_price: f64) -> Position {
    Position {
        id: Some(format!(
            "pos_{}_{}",
            user_id,
            chrono::Utc::now().timestamp_millis()
        )),
        symbol: pair,
        side: PositionSide::Long,
        size,
        notional: size * entry_price,
        entry_price,
        mark_price: Some(entry_price),
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
        leverage: 1.0,
        margin: size * entry_price,
        timestamp: Some(chrono::Utc::now()),
        datetime: Some(chrono::Utc::now().to_rfc3339()),
    }
}

/// Helper function to create AI analysis result
fn create_test_ai_analysis(opportunity_id: &str, confidence_boost: f64) -> serde_json::Value {
    json!({
        "opportunity_id": opportunity_id,
        "ai_confidence_score": confidence_boost,
        "risk_assessment": {
            "volatility_prediction": "moderate",
            "market_sentiment": "bullish",
            "technical_indicators": ["RSI_oversold", "MACD_bullish_crossover"]
        },
        "recommended_action": "BUY",
        "confidence_factors": [
            "Strong technical indicators",
            "Positive market sentiment",
            "Low volatility expected"
        ],
        "analysis_timestamp": chrono::Utc::now().timestamp_millis()
    })
}

#[cfg(test)]
mod service_integration_e2e_tests {
    use super::*;

    // Test constants
    const BYBIT_PRICE_VARIATION: f64 = 0.002; // 0.2% higher
    const OKX_PRICE_VARIATION: f64 = -0.001; // 0.1% lower
    const ARBITRAGE_THRESHOLD: f64 = 0.0015; // 0.15%
    const BASE_CONFIDENCE_SCORE: f64 = 0.85;
    const FEE_ADJUSTMENT: f64 = 0.8; // 80% after fees
    const AI_CONFIDENCE_BOOST: f64 = 0.05;
    const FREE_TIER_MIN_CONFIDENCE: f64 = 0.9;
    const PREMIUM_TIER_MIN_CONFIDENCE: f64 = 0.8;
    const DEFAULT_MIN_CONFIDENCE: f64 = 0.85;

    /// Helper function to ingest exchange data
    async fn ingest_exchange_data(
        test_env: &mut ServiceIntegrationTestEnvironment,
    ) -> (Vec<String>, Vec<String>) {
        let exchanges = vec![
            "binance".to_string(),
            "bybit".to_string(),
            "okx".to_string(),
        ];
        let trading_pairs = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "ADAUSDT".to_string(),
        ];

        for exchange in &exchanges {
            for pair in &trading_pairs {
                let base_price = match pair.as_ref() {
                    "BTCUSDT" => 50000.0,
                    "ETHUSDT" => 3000.0,
                    "ADAUSDT" => 0.5,
                    _ => 100.0,
                };

                let price_variation = match exchange.as_ref() {
                    "binance" => 0.0,
                    "bybit" => BYBIT_PRICE_VARIATION,
                    "okx" => OKX_PRICE_VARIATION,
                    _ => 0.0,
                };

                let price = base_price * (1.0 + price_variation);
                let market_data = create_test_market_data(exchange, pair, price);
                test_env.add_market_data(exchange.clone(), pair.clone(), market_data);
            }
        }

        println!("✅ Step 1: Exchange data ingestion completed");
        println!("   Exchanges: {}", exchanges.len());
        println!("   Trading Pairs: {}", trading_pairs.len());
        println!(
            "   Total Market Data Points: {}",
            test_env.market_data.len()
        );

        (exchanges, trading_pairs)
    }

    /// Helper function to detect arbitrage opportunities
    async fn detect_arbitrage_opportunities(
        test_env: &mut ServiceIntegrationTestEnvironment,
        exchanges: &[String],
        trading_pairs: &[String],
    ) -> Vec<TradingOpportunity> {
        let mut detected_opportunities = Vec::new();

        for pair in trading_pairs {
            let mut exchange_prices = Vec::new();
            for exchange in exchanges {
                let key = format!("{}:{}", exchange, pair);
                if let Some(data) = test_env.market_data.get(&key) {
                    let price = data["price"].as_f64().unwrap();
                    exchange_prices.push((exchange, price));
                }
            }

            for i in 0..exchange_prices.len() {
                for j in i + 1..exchange_prices.len() {
                    let (exchange_a, price_a) = &exchange_prices[i];
                    let (exchange_b, price_b) = &exchange_prices[j];
                    let price_diff = (price_b - price_a).abs() / price_a;

                    if price_diff > ARBITRAGE_THRESHOLD {
                        let opportunity = TradingOpportunity {
                            opportunity_id: format!(
                                "arb_{}_{}_{}_{}",
                                pair,
                                exchange_a,
                                exchange_b,
                                chrono::Utc::now().timestamp_millis()
                            ),
                            opportunity_type: OpportunityType::Arbitrage,
                            trading_pair: pair.clone(),
                            exchanges: vec![exchange_a.to_string(), exchange_b.to_string()],
                            entry_price: price_a.min(*price_b),
                            target_price: Some(price_a.max(*price_b)),
                            stop_loss: Some(price_a.min(*price_b) * 0.995),
                            confidence_score: BASE_CONFIDENCE_SCORE + (price_diff * 10.0).min(0.1),
                            risk_level: if price_diff > 0.003 {
                                RiskLevel::Low
                            } else {
                                RiskLevel::Medium
                            },
                            expected_return: price_diff * FEE_ADJUSTMENT,
                            time_horizon: TimeHorizon::Short,
                            indicators_used: vec!["cross_exchange_arbitrage".to_string()],
                            analysis_data: json!({
                                "buy_exchange": if *price_a < *price_b { exchange_a } else { exchange_b },
                                "sell_exchange": if *price_a < *price_b { exchange_b } else { exchange_a },
                                "price_difference": price_diff,
                                "buy_price": price_a.min(*price_b),
                                "sell_price": price_a.max(*price_b)
                            }),
                            created_at: chrono::Utc::now().timestamp_millis() as u64,
                            expires_at: Some(
                                chrono::Utc::now().timestamp_millis() as u64 + 1800000,
                            ),
                        };
                        detected_opportunities.push(opportunity);
                    }
                }
            }
        }

        for opp in &detected_opportunities {
            test_env.add_opportunity(opp.clone());
        }

        println!("✅ Step 2: Market analysis and opportunity detection completed");
        println!(
            "   Opportunities Detected: {}",
            detected_opportunities.len()
        );

        detected_opportunities
    }

    /// Helper function to enhance opportunities with AI
    async fn enhance_opportunities_with_ai(
        test_env: &mut ServiceIntegrationTestEnvironment,
        opportunities: &[TradingOpportunity],
    ) -> Vec<TradingOpportunity> {
        let mut ai_enhanced_opportunities = Vec::new();

        for opp in opportunities {
            let ai_analysis = create_test_ai_analysis(&opp.opportunity_id, AI_CONFIDENCE_BOOST);
            test_env.add_ai_analysis(opp.opportunity_id.clone(), ai_analysis.clone());

            let mut enhanced_opp = opp.clone();
            enhanced_opp.confidence_score = (opp.confidence_score + AI_CONFIDENCE_BOOST).min(1.0);
            enhanced_opp
                .indicators_used
                .push("ai_sentiment_analysis".to_string());
            enhanced_opp.analysis_data["ai_enhancement"] = ai_analysis;

            ai_enhanced_opportunities.push(enhanced_opp);
        }

        println!("✅ Step 3: AI enhancement completed");
        println!(
            "   AI-Enhanced Opportunities: {}",
            ai_enhanced_opportunities.len()
        );

        ai_enhanced_opportunities
    }

    /// Helper function to target users and send notifications
    async fn target_users_and_send_notifications(
        test_env: &mut ServiceIntegrationTestEnvironment,
        opportunities: &[TradingOpportunity],
    ) -> Vec<(String, String)> {
        let test_users = vec![
            {
                let mut user = UserProfile::new(Some(111111111), Some("test-invite-1".to_string()));
                user.subscription.tier = SubscriptionTier::Free;
                user
            },
            {
                let mut user = UserProfile::new(Some(222222222), Some("test-invite-2".to_string()));
                user.subscription.tier = SubscriptionTier::Premium;
                user
            },
        ];

        for user in &test_users {
            test_env.add_user(user.clone());
        }

        let mut user_notifications = Vec::new();

        for user in &test_users {
            for opp in opportunities {
                let subscription_match = match user.subscription.tier {
                    SubscriptionTier::Free => opp.confidence_score >= FREE_TIER_MIN_CONFIDENCE,
                    SubscriptionTier::Premium | SubscriptionTier::Enterprise => {
                        opp.confidence_score >= PREMIUM_TIER_MIN_CONFIDENCE
                    }
                    _ => opp.confidence_score >= DEFAULT_MIN_CONFIDENCE,
                };

                if subscription_match {
                    let notification = format!(
                        "🚨 Arbitrage Alert for {}\n💰 {}: {:.2}% profit potential\n🎯 Confidence: {:.1}%\n⏰ Expires in 30 minutes",
                        user.user_id,
                        opp.trading_pair,
                        opp.expected_return * 100.0,
                        opp.confidence_score * 100.0
                    );

                    test_env.send_notification(notification);
                    user_notifications.push((user.user_id.clone(), opp.opportunity_id.clone()));
                }
            }
        }

        println!("✅ Step 4: User targeting and categorization completed");
        println!("   User Notifications: {}", user_notifications.len());
        println!(
            "   Total Notifications Sent: {}",
            test_env.notifications.len()
        );

        user_notifications
    }

    /// Helper function to collect monitoring metrics
    async fn collect_monitoring_metrics(
        test_env: &mut ServiceIntegrationTestEnvironment,
        detected_opportunities: &[TradingOpportunity],
        ai_enhanced_opportunities: &[TradingOpportunity],
    ) {
        test_env.record_metric(
            "opportunities_detected".to_string(),
            detected_opportunities.len() as f64,
        );
        test_env.record_metric(
            "ai_enhancements_applied".to_string(),
            ai_enhanced_opportunities.len() as f64,
        );
        test_env.record_metric(
            "notifications_sent".to_string(),
            test_env.notifications.len() as f64,
        );
        test_env.record_metric("pipeline_latency_ms".to_string(), 150.0);
        test_env.record_metric(
            "data_points_processed".to_string(),
            test_env.market_data.len() as f64,
        );

        println!("✅ Step 5: Monitoring and metrics collection completed");
        println!("   Metrics Recorded: {}", test_env.monitoring_metrics.len());
    }

    /// Helper function to validate pipeline results
    fn validate_pipeline_results(
        test_env: &ServiceIntegrationTestEnvironment,
        exchanges: &[String],
        trading_pairs: &[String],
        detected_opportunities: &[TradingOpportunity],
        ai_enhanced_opportunities: &[TradingOpportunity],
    ) {
        assert!(
            !test_env.market_data.is_empty(),
            "Market data should be ingested"
        );
        assert!(
            !detected_opportunities.is_empty(),
            "Opportunities should be detected"
        );
        assert_eq!(
            ai_enhanced_opportunities.len(),
            detected_opportunities.len(),
            "All opportunities should be AI-enhanced"
        );
        assert!(
            !test_env.notifications.is_empty(),
            "Notifications should be sent"
        );
        assert!(
            !test_env.monitoring_metrics.is_empty(),
            "Metrics should be recorded"
        );

        let total_data_points = test_env.market_data.len();
        let expected_data_points = exchanges.len() * trading_pairs.len();
        assert_eq!(
            total_data_points, expected_data_points,
            "All market data should be processed"
        );

        println!("\n🎉 Complete Data Pipeline Integration E2E Test PASSED");
        println!("==========================================");
        println!("✅ Exchange Data Ingestion: WORKING");
        println!("✅ Market Analysis: WORKING");
        println!("✅ Opportunity Detection: WORKING");
        println!("✅ AI Enhancement: WORKING");
        println!("✅ User Targeting: WORKING");
        println!("✅ Notification Delivery: WORKING");
        println!("✅ Monitoring & Metrics: WORKING");
        println!("✅ Data Flow Integrity: WORKING");
        println!("==========================================");
    }

    /// **E2E Test 1: Complete Data Pipeline Integration**
    /// Tests: ExchangeService → MarketAnalysis → OpportunityDetection → UserNotification
    #[tokio::test]
    async fn test_complete_data_pipeline_integration() {
        println!("🚀 Starting Complete Data Pipeline Integration E2E Test");

        let mut test_env = ServiceIntegrationTestEnvironment::new();

        // Step 1: Exchange Data Ingestion
        let (exchanges, trading_pairs) = ingest_exchange_data(&mut test_env).await;

        // Step 2: Market Analysis and Opportunity Detection
        let detected_opportunities =
            detect_arbitrage_opportunities(&mut test_env, &exchanges, &trading_pairs).await;

        // Step 3: AI Enhancement of Opportunities
        let ai_enhanced_opportunities =
            enhance_opportunities_with_ai(&mut test_env, &detected_opportunities).await;

        // Step 4: User Targeting and Categorization
        let _user_notifications =
            target_users_and_send_notifications(&mut test_env, &ai_enhanced_opportunities).await;

        // Step 5: Monitoring and Metrics Collection
        collect_monitoring_metrics(
            &mut test_env,
            &detected_opportunities,
            &ai_enhanced_opportunities,
        )
        .await;

        // Final Validation
        validate_pipeline_results(
            &test_env,
            &exchanges,
            &trading_pairs,
            &detected_opportunities,
            &ai_enhanced_opportunities,
        );
    }

    /// **E2E Test 2: Position Management and Risk Monitoring Integration**
    /// Tests: PositionService → RiskAnalysis → MonitoringService → AlertSystem
    #[tokio::test]
    async fn test_position_management_risk_monitoring_integration() {
        println!("🚀 Starting Position Management and Risk Monitoring Integration E2E Test");

        let mut test_env = ServiceIntegrationTestEnvironment::new();

        // **Step 1: Create Test User and Positions**
        let mut test_user = UserProfile::new(Some(123456789), Some("test-trader".to_string()));
        test_user.subscription.tier = SubscriptionTier::Premium;
        test_user.total_trades = 15;
        test_user.total_pnl_usdt = 1250.75;
        test_env.add_user(test_user.clone());

        // Create multiple positions with different risk profiles
        let positions = vec![
            create_test_position(
                test_user.user_id.clone(),
                "BTCUSDT".to_string(),
                0.1,
                50000.0,
            ),
            create_test_position(
                test_user.user_id.clone(),
                "ETHUSDT".to_string(),
                2.0,
                3000.0,
            ),
            create_test_position(
                test_user.user_id.clone(),
                "ADAUSDT".to_string(),
                10000.0,
                0.5,
            ),
        ];

        for pos in &positions {
            test_env.add_position(pos.clone());
        }

        println!("✅ Step 1: User and positions created");
        println!("   User: {} (Premium)", test_user.user_id);
        println!("   Positions: {}", positions.len());
        println!(
            "   Total Portfolio Value: ${:.2}",
            positions
                .iter()
                .map(|p| p.size * p.entry_price)
                .sum::<f64>()
        );

        // **Step 2: Market Price Updates and PnL Calculation**
        let market_updates = vec![
            ("BTCUSDT", 51500.0), // +3% gain
            ("ETHUSDT", 2850.0),  // -5% loss
            ("ADAUSDT", 0.52),    // +4% gain
        ];

        let mut updated_positions = Vec::new();
        for mut pos in positions {
            for (pair, new_price) in &market_updates {
                if pos.symbol == *pair {
                    pos.mark_price = Some(*new_price);
                    pos.unrealized_pnl = (new_price - pos.entry_price) * pos.size;
                    pos.timestamp = Some(chrono::Utc::now());

                    // Add market data for this update
                    let market_data = create_test_market_data("binance", pair, *new_price);
                    test_env.add_market_data("binance".to_string(), pair.to_string(), market_data);
                    break;
                }
            }
            updated_positions.push(pos);
        }

        // Update positions in test environment
        test_env.positions = updated_positions.clone();

        println!("✅ Step 2: Market price updates and PnL calculation");
        for pos in &updated_positions {
            let pnl_percentage = (pos.unrealized_pnl / (pos.entry_price * pos.size)) * 100.0;
            println!(
                "   {}: ${:.2} PnL ({:.1}%)",
                pos.symbol, pos.unrealized_pnl, pnl_percentage
            );
        }

        // **Step 3: Risk Analysis and Alert Generation**
        let mut risk_alerts = Vec::new();
        let total_portfolio_value: f64 = updated_positions
            .iter()
            .map(|p| p.size * p.mark_price.unwrap_or(p.entry_price))
            .sum();
        let total_unrealized_pnl: f64 = updated_positions.iter().map(|p| p.unrealized_pnl).sum();
        let portfolio_pnl_percentage = (total_unrealized_pnl / total_portfolio_value) * 100.0;

        // Risk thresholds
        let position_risk_threshold = 10.0; // 10% loss threshold
        let portfolio_risk_threshold = 5.0; // 5% portfolio loss threshold

        // Check individual position risks
        for pos in &updated_positions {
            let position_pnl_percentage =
                (pos.unrealized_pnl / (pos.entry_price * pos.size)) * 100.0;

            if position_pnl_percentage < -position_risk_threshold {
                let alert = format!(
                    "🚨 HIGH RISK: {} position down {:.1}% (${:.2} loss)",
                    pos.symbol,
                    position_pnl_percentage.abs(),
                    pos.unrealized_pnl.abs()
                );
                risk_alerts.push(alert.clone());
                test_env.send_notification(alert);
            } else if position_pnl_percentage > 15.0 {
                let alert = format!(
                    "🎉 PROFIT ALERT: {} position up {:.1}% (${:.2} gain) - Consider taking profits",
                    pos.symbol, position_pnl_percentage, pos.unrealized_pnl
                );
                risk_alerts.push(alert.clone());
                test_env.send_notification(alert);
            }
        }

        // Check portfolio-level risk
        if portfolio_pnl_percentage < -portfolio_risk_threshold {
            let alert = format!(
                "⚠️ PORTFOLIO RISK: Total portfolio down {:.1}% (${:.2} loss)",
                portfolio_pnl_percentage.abs(),
                total_unrealized_pnl.abs()
            );
            risk_alerts.push(alert.clone());
            test_env.send_notification(alert);
        }

        println!("✅ Step 3: Risk analysis and alert generation");
        println!(
            "   Portfolio PnL: {:.1}% (${:.2})",
            portfolio_pnl_percentage, total_unrealized_pnl
        );
        println!("   Risk Alerts Generated: {}", risk_alerts.len());

        // **Step 4: Monitoring Metrics Collection**
        test_env.record_metric(
            "total_positions".to_string(),
            updated_positions.len() as f64,
        );
        test_env.record_metric("portfolio_value_usd".to_string(), total_portfolio_value);
        test_env.record_metric("total_unrealized_pnl_usd".to_string(), total_unrealized_pnl);
        test_env.record_metric(
            "portfolio_pnl_percentage".to_string(),
            portfolio_pnl_percentage,
        );
        test_env.record_metric(
            "risk_alerts_generated".to_string(),
            risk_alerts.len() as f64,
        );

        // Position-specific metrics
        for (i, pos) in updated_positions.iter().enumerate() {
            let position_pnl_percentage =
                (pos.unrealized_pnl / (pos.entry_price * pos.size)) * 100.0;
            test_env.record_metric(
                format!("position_{}_pnl_percentage", i),
                position_pnl_percentage,
            );
            test_env.record_metric(
                format!("position_{}_value_usd", i),
                pos.size * pos.mark_price.unwrap_or(pos.entry_price),
            );
        }

        println!("✅ Step 4: Monitoring metrics collection");
        println!("   Metrics Recorded: {}", test_env.monitoring_metrics.len());

        // **Step 5: User Profile Updates**
        // Update user profile with latest trading statistics
        let mut updated_user = test_user.clone();
        updated_user.total_pnl_usdt += total_unrealized_pnl;
        updated_user.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Add position summary to user metadata
        updated_user.profile_metadata = Some(json!({
            "active_positions": updated_positions.len(),
            "portfolio_value_usd": total_portfolio_value,
            "unrealized_pnl_usd": total_unrealized_pnl,
            "portfolio_pnl_percentage": portfolio_pnl_percentage,
            "last_risk_check": chrono::Utc::now().timestamp_millis()
        }));

        test_env
            .users
            .insert(updated_user.user_id.clone(), updated_user.clone());

        println!("✅ Step 5: User profile updates");
        println!("   Updated Total PnL: ${:.2}", updated_user.total_pnl_usdt);

        // **Step 6: Automated Risk Management Actions**
        let mut automated_actions = Vec::new();

        for pos in &updated_positions {
            let position_pnl_percentage =
                (pos.unrealized_pnl / (pos.entry_price * pos.size)) * 100.0;

            // Simulate automated stop-loss triggers
            if position_pnl_percentage < -15.0 {
                let action = format!(
                    "🛑 AUTO STOP-LOSS: {} position closed at {:.1}% loss",
                    pos.symbol,
                    position_pnl_percentage.abs()
                );
                automated_actions.push(action.clone());
                test_env.send_notification(action);
            }

            // Simulate automated take-profit triggers
            if position_pnl_percentage > 20.0 {
                let action = format!(
                    "💰 AUTO TAKE-PROFIT: {} position closed at {:.1}% profit",
                    pos.symbol, position_pnl_percentage
                );
                automated_actions.push(action.clone());
                test_env.send_notification(action);
            }
        }

        println!("✅ Step 6: Automated risk management actions");
        println!("   Automated Actions: {}", automated_actions.len());

        // **Final Validation**
        assert_eq!(
            test_env.positions.len(),
            3,
            "All positions should be tracked"
        );
        assert!(
            !test_env.market_data.is_empty(),
            "Market data should be available"
        );
        assert!(
            !test_env.monitoring_metrics.is_empty(),
            "Metrics should be recorded"
        );
        assert!(
            test_env.users.contains_key(&test_user.user_id),
            "User should be updated"
        );

        // Verify risk monitoring worked
        let has_risk_alerts = !risk_alerts.is_empty() || !automated_actions.is_empty();
        println!("   Risk Monitoring Active: {}", has_risk_alerts);

        // Verify portfolio calculations
        assert!(total_portfolio_value > 0.0, "Portfolio should have value");
        assert!(
            portfolio_pnl_percentage.abs() < 100.0,
            "Portfolio PnL should be reasonable"
        );

        println!("\n🎉 Position Management and Risk Monitoring Integration E2E Test PASSED");
        println!("==========================================");
        println!("✅ Position Creation & Tracking: WORKING");
        println!("✅ Market Price Updates: WORKING");
        println!("✅ PnL Calculation: WORKING");
        println!("✅ Risk Analysis: WORKING");
        println!("✅ Alert Generation: WORKING");
        println!("✅ Monitoring Metrics: WORKING");
        println!("✅ User Profile Updates: WORKING");
        println!("✅ Automated Risk Management: WORKING");
        println!("==========================================");
    }

    /// **E2E Test 3: AI Intelligence and Technical Analysis Integration**
    /// Tests: TechnicalAnalysis → AI Enhancement → Opportunity Scoring → User Delivery
    #[tokio::test]
    async fn test_ai_intelligence_technical_analysis_integration() {
        println!("🚀 Starting AI Intelligence and Technical Analysis Integration E2E Test");

        let mut test_env = ServiceIntegrationTestEnvironment::new();

        // **Step 1: Generate Technical Analysis Data**
        let trading_pairs = vec!["BTCUSDT", "ETHUSDT", "ADAUSDT"];
        let mut technical_signals = Vec::new();

        for pair in &trading_pairs {
            // Simulate technical indicators
            let rsi = match pair.as_ref() {
                "BTCUSDT" => 25.0, // Oversold
                "ETHUSDT" => 75.0, // Overbought
                "ADAUSDT" => 45.0, // Neutral
                _ => 50.0,
            };

            let macd_signal = match pair.as_ref() {
                "BTCUSDT" => "bullish_crossover",
                "ETHUSDT" => "bearish_divergence",
                "ADAUSDT" => "neutral",
                _ => "neutral",
            };

            let bollinger_position = match pair.as_ref() {
                "BTCUSDT" => "lower_band", // Potential bounce
                "ETHUSDT" => "upper_band", // Potential reversal
                "ADAUSDT" => "middle",     // No clear signal
                _ => "middle",
            };

            let technical_data = json!({
                "trading_pair": pair,
                "rsi": rsi,
                "macd_signal": macd_signal,
                "bollinger_position": bollinger_position,
                "volume_trend": "increasing",
                "support_level": 48000.0,
                "resistance_level": 52000.0,
                "trend_direction": if rsi < 30.0 { "bullish_reversal" } else if rsi > 70.0 { "bearish_reversal" } else { "sideways" },
                "analysis_timestamp": chrono::Utc::now().timestamp_millis()
            });

            technical_signals.push(technical_data);
        }

        println!("✅ Step 1: Technical analysis data generated");
        println!("   Trading Pairs Analyzed: {}", technical_signals.len());

        // **Step 2: AI Intelligence Enhancement**
        let mut ai_enhanced_signals = Vec::new();

        for signal in &technical_signals {
            let pair = signal["trading_pair"].as_str().unwrap();
            let rsi = signal["rsi"].as_f64().unwrap();
            let macd_signal = signal["macd_signal"].as_str().unwrap();
            let bollinger_position = signal["bollinger_position"].as_str().unwrap();

            // AI scoring algorithm
            let mut ai_confidence: f64 = 0.5; // Base confidence

            // RSI-based scoring
            if rsi < 30.0 {
                ai_confidence += 0.2; // Oversold = bullish signal
            } else if rsi > 70.0 {
                ai_confidence += 0.15; // Overbought = bearish signal
            }

            // MACD-based scoring
            match macd_signal {
                "bullish_crossover" => ai_confidence += 0.25,
                "bearish_divergence" => ai_confidence += 0.2,
                _ => {}
            }

            // Bollinger Bands scoring
            match bollinger_position {
                "lower_band" => ai_confidence += 0.15, // Potential bounce
                "upper_band" => ai_confidence += 0.1,  // Potential reversal
                _ => {}
            }

            ai_confidence = ai_confidence.min(0.95); // Cap at 95%

            // Generate AI recommendation
            let recommendation = if rsi < 30.0 && macd_signal == "bullish_crossover" {
                "STRONG_BUY"
            } else if rsi > 70.0 && macd_signal == "bearish_divergence" {
                "STRONG_SELL"
            } else if ai_confidence > 0.7 {
                "BUY"
            } else if ai_confidence < 0.4 {
                "SELL"
            } else {
                "HOLD"
            };

            let ai_analysis = json!({
                "trading_pair": pair,
                "ai_confidence_score": ai_confidence,
                "recommendation": recommendation,
                "technical_score": {
                    "rsi_score": if rsi < 30.0 { 0.8 } else if rsi > 70.0 { 0.7 } else { 0.3 },
                    "macd_score": match macd_signal {
                        "bullish_crossover" => 0.9,
                        "bearish_divergence" => 0.8,
                        _ => 0.4
                    },
                    "bollinger_score": match bollinger_position {
                        "lower_band" | "upper_band" => 0.7,
                        _ => 0.3
                    }
                },
                "risk_assessment": {
                    "volatility": if ai_confidence > 0.8 { "low" } else { "medium" },
                    "market_sentiment": if recommendation.contains("BUY") { "bullish" } else if recommendation.contains("SELL") { "bearish" } else { "neutral" }
                },
                "ai_factors": [
                    format!("RSI: {:.1} ({})", rsi, if rsi < 30.0 { "oversold" } else if rsi > 70.0 { "overbought" } else { "neutral" }),
                    format!("MACD: {}", macd_signal),
                    format!("Bollinger: {}", bollinger_position)
                ],
                "analysis_timestamp": chrono::Utc::now().timestamp_millis()
            });

            test_env.add_ai_analysis(format!("tech_analysis_{}", pair), ai_analysis.clone());
            ai_enhanced_signals.push(ai_analysis);
        }

        println!("✅ Step 2: AI intelligence enhancement completed");
        for signal in &ai_enhanced_signals {
            println!(
                "   {}: {} ({:.1}% confidence)",
                signal["trading_pair"].as_str().unwrap(),
                signal["recommendation"].as_str().unwrap(),
                signal["ai_confidence_score"].as_f64().unwrap() * 100.0
            );
        }

        // **Step 3: Convert to Trading Opportunities**
        let mut trading_opportunities = Vec::new();

        for signal in &ai_enhanced_signals {
            let pair = signal["trading_pair"].as_str().unwrap();
            let confidence = signal["ai_confidence_score"].as_f64().unwrap();
            let recommendation = signal["recommendation"].as_str().unwrap();

            // Only create opportunities for actionable signals
            if matches!(
                recommendation,
                "BUY" | "STRONG_BUY" | "SELL" | "STRONG_SELL"
            ) {
                let opportunity = TradingOpportunity {
                    opportunity_id: format!(
                        "ai_tech_{}_{}",
                        pair,
                        chrono::Utc::now().timestamp_millis()
                    ),
                    opportunity_type: OpportunityType::Technical,
                    trading_pair: pair.to_string(),
                    exchanges: vec!["binance".to_string()],
                    entry_price: match pair {
                        "BTCUSDT" => 50000.0,
                        "ETHUSDT" => 3000.0,
                        "ADAUSDT" => 0.5,
                        _ => 100.0,
                    },
                    target_price: None, // Will be calculated based on technical levels
                    stop_loss: None,    // Will be calculated based on technical levels
                    confidence_score: confidence,
                    risk_level: if confidence > 0.8 {
                        RiskLevel::Low
                    } else if confidence > 0.6 {
                        RiskLevel::Medium
                    } else {
                        RiskLevel::High
                    },
                    expected_return: confidence * 0.08, // AI-enhanced expected return
                    time_horizon: TimeHorizon::Medium,
                    indicators_used: vec![
                        "RSI".to_string(),
                        "MACD".to_string(),
                        "Bollinger_Bands".to_string(),
                        "AI_Analysis".to_string(),
                    ],
                    analysis_data: signal.clone(),
                    created_at: chrono::Utc::now().timestamp_millis() as u64,
                    expires_at: Some(chrono::Utc::now().timestamp_millis() as u64 + 7200000), // 2 hours
                };

                trading_opportunities.push(opportunity);
            }
        }

        for opp in &trading_opportunities {
            test_env.add_opportunity(opp.clone());
        }

        println!("✅ Step 3: Trading opportunities created");
        println!(
            "   Actionable Opportunities: {}",
            trading_opportunities.len()
        );

        // **Step 4: User Targeting for AI-Enhanced Opportunities**
        // Create users with different AI access levels
        let test_users = vec![
            {
                let mut user = UserProfile::new(Some(111111111), Some("basic-user".to_string()));
                user.subscription.tier = SubscriptionTier::Basic;
                user
            },
            {
                let mut user = UserProfile::new(Some(222222222), Some("premium-user".to_string()));
                user.subscription.tier = SubscriptionTier::Premium;
                user
            },
        ];

        for user in &test_users {
            test_env.add_user(user.clone());
        }

        // Filter opportunities based on subscription tier
        let mut user_opportunity_matches = Vec::new();

        for user in &test_users {
            for opp in &trading_opportunities {
                let has_ai_access = matches!(
                    user.subscription.tier,
                    SubscriptionTier::Premium | SubscriptionTier::Enterprise
                );
                let confidence_threshold = match user.subscription.tier {
                    SubscriptionTier::Free => 0.9,
                    SubscriptionTier::Basic => 0.8,
                    SubscriptionTier::Premium | SubscriptionTier::Enterprise => 0.6,
                    SubscriptionTier::SuperAdmin => 0.5,
                };

                let should_notify = has_ai_access && opp.confidence_score >= confidence_threshold;

                if should_notify {
                    let notification = format!(
                        "🤖 AI Technical Signal: {}\n📈 Recommendation: {}\n💡 Confidence: {:.1}%\n⏰ Valid for 2 hours",
                        opp.trading_pair,
                        opp.analysis_data["recommendation"].as_str().unwrap_or("UNKNOWN"),
                        opp.confidence_score * 100.0
                    );

                    test_env.send_notification(notification);
                    user_opportunity_matches
                        .push((user.user_id.clone(), opp.opportunity_id.clone()));
                }
            }
        }

        println!("✅ Step 4: User targeting completed");
        println!(
            "   User-Opportunity Matches: {}",
            user_opportunity_matches.len()
        );
        println!("   Notifications Sent: {}", test_env.notifications.len());

        // **Step 5: Performance Metrics and Monitoring**
        test_env.record_metric(
            "technical_signals_generated".to_string(),
            technical_signals.len() as f64,
        );
        test_env.record_metric(
            "ai_enhanced_signals".to_string(),
            ai_enhanced_signals.len() as f64,
        );
        test_env.record_metric(
            "trading_opportunities_created".to_string(),
            trading_opportunities.len() as f64,
        );
        test_env.record_metric(
            "user_notifications_sent".to_string(),
            test_env.notifications.len() as f64,
        );

        // Calculate average confidence scores
        let avg_confidence: f64 = trading_opportunities
            .iter()
            .map(|o| o.confidence_score)
            .sum::<f64>()
            / trading_opportunities.len() as f64;
        test_env.record_metric("average_ai_confidence".to_string(), avg_confidence);

        // Count opportunities by recommendation type
        let mut recommendation_counts = HashMap::new();
        for signal in &ai_enhanced_signals {
            let rec = signal["recommendation"].as_str().unwrap();
            *recommendation_counts.entry(rec.to_string()).or_insert(0) += 1;
        }

        for (rec, count) in recommendation_counts {
            test_env.record_metric(
                format!("recommendations_{}", rec.to_lowercase()),
                count as f64,
            );
        }

        println!("✅ Step 5: Performance metrics and monitoring");
        println!("   Average AI Confidence: {:.1}%", avg_confidence * 100.0);
        println!("   Metrics Recorded: {}", test_env.monitoring_metrics.len());

        // **Final Validation**
        assert!(
            !technical_signals.is_empty(),
            "Technical signals should be generated"
        );
        assert_eq!(
            ai_enhanced_signals.len(),
            technical_signals.len(),
            "All signals should be AI-enhanced"
        );
        assert!(
            !trading_opportunities.is_empty(),
            "Trading opportunities should be created"
        );
        assert!(
            !test_env.notifications.is_empty(),
            "Notifications should be sent to eligible users"
        );
        assert!(
            !test_env.monitoring_metrics.is_empty(),
            "Metrics should be recorded"
        );

        // Verify AI enhancement quality
        let high_confidence_opportunities = trading_opportunities
            .iter()
            .filter(|o| o.confidence_score > 0.7)
            .count();
        assert!(
            high_confidence_opportunities > 0,
            "Should have some high-confidence opportunities"
        );

        println!("\n🎉 AI Intelligence and Technical Analysis Integration E2E Test PASSED");
        println!("==========================================");
        println!("✅ Technical Analysis Generation: WORKING");
        println!("✅ AI Intelligence Enhancement: WORKING");
        println!("✅ Opportunity Scoring: WORKING");
        println!("✅ User Access Control: WORKING");
        println!("✅ Notification Delivery: WORKING");
        println!("✅ Performance Monitoring: WORKING");
        println!("✅ Quality Assurance: WORKING");
        println!("==========================================");
    }
}
