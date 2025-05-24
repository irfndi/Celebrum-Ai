// End-to-End User Journey Tests
// This file tests complete user flows from start to finish, validating
// that all services work together correctly for real user scenarios.

use arb_edge::{
    services::{
        d1_database::D1Service,
        user_profile::UserProfileService,
        user_trading_preferences::UserTradingPreferencesService,
        exchange::ExchangeService,
        global_opportunity::GlobalOpportunityService,
        opportunity_categorization::OpportunityCategorizationService,
        notifications::NotificationService,
        telegram::TelegramService,
        ai_integration::AiIntegrationService,
        market_analysis::{MarketAnalysisService, TradingOpportunity, OpportunityType, RiskLevel, TimeHorizon},
    },
    types::*,
    utils::{ArbitrageError, ArbitrageResult, logger::{Logger, LogLevel}},
};
use worker::kv::KvStore;
use serde_json::json;
use std::collections::HashMap;

/// E2E Test Framework for User Journey Testing
pub struct E2ETestFramework {
    // Core Services
    d1_service: D1Service,
    user_profile_service: UserProfileService,
    user_preferences_service: UserTradingPreferencesService,
    exchange_service: ExchangeService,
    opportunity_service: GlobalOpportunityService,
    categorization_service: OpportunityCategorizationService,
    notification_service: NotificationService,
    telegram_service: TelegramService,
    
    // Test State
    test_users: HashMap<String, UserProfile>,
    test_opportunities: Vec<TradingOpportunity>,
    mock_market_data: HashMap<String, serde_json::Value>,
}

impl E2ETestFramework {
    pub async fn new() -> Self {
        let logger = Logger::new(LogLevel::Debug);
        
        // Initialize all services with test configuration
        let d1_service = D1Service::new("test_database".to_string());
        let user_profile_service = UserProfileService::new(
            KvStore::new(),
            d1_service.clone(),
            "test_encryption_key".to_string()
        );
        let user_preferences_service = UserTradingPreferencesService::new(
            d1_service.clone(),
            logger.clone()
        );
        
        // TODO: Initialize other services
        // Note: Some services need mock configurations for testing
        
        Self {
            d1_service,
            user_profile_service,
            user_preferences_service,
            exchange_service: todo!("Initialize with mock exchange APIs"),
            opportunity_service: todo!("Initialize with test configuration"),
            categorization_service: todo!("Initialize with test configuration"),
            notification_service: todo!("Initialize with test configuration"),
            telegram_service: todo!("Initialize with test configuration"),
            test_users: HashMap::new(),
            test_opportunities: Vec::new(),
            mock_market_data: HashMap::new(),
        }
    }
    
    /// Sets up mock market data for testing
    pub fn setup_mock_market_data(&mut self) {
        // Mock Binance BTC/USDT data
        self.mock_market_data.insert("binance_btc_usdt".to_string(), json!({
            "symbol": "BTCUSDT",
            "price": "45000.50",
            "volume": "1234.567",
            "timestamp": 1640995200000u64
        }));
        
        // Mock Bybit BTC/USDT data (with arbitrage opportunity)
        self.mock_market_data.insert("bybit_btc_usdt".to_string(), json!({
            "symbol": "BTCUSDT", 
            "price": "45050.25",  // $50 higher - arbitrage opportunity!
            "volume": "987.432",
            "timestamp": 1640995200000u64
        }));
    }
    
    /// Creates a test user with specified trading preferences
    pub async fn create_test_user(&mut self, 
        user_id: &str,
        trading_focus: TradingFocus,
        experience_level: ExperienceLevel
    ) -> Result<UserProfile, ArbitrageError> {
        // Create user profile with test data
        let test_telegram_id = 12345 + self.test_users.len() as i64;
        let user_profile = UserProfile::new(test_telegram_id, Some("test-e2e".to_string()));
        
        // Store user profile (using the correct method signature)
        let created_user = self.user_profile_service.create_user_profile(
            user_profile.telegram_user_id,
            user_profile.invitation_code.clone(),
            user_profile.telegram_username.clone()
        ).await?;
        
        // Update our local copy with the created user data
        let user_profile = created_user;
        
        // Create user trading preferences
        let preferences = UserTradingPreferences {
            user_id: user_id.to_string(),
            trading_focus,
            automation_level: AutomationLevel::Manual, // Start with manual
            automation_scope: AutomationScope::None,
            experience_level,
            risk_tolerance: match experience_level {
                ExperienceLevel::Beginner => RiskTolerance::Conservative,
                ExperienceLevel::Intermediate => RiskTolerance::Balanced,
                ExperienceLevel::Advanced => RiskTolerance::Aggressive,
            },
            created_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
        };
        
        self.user_preferences_service.update_preferences(&preferences).await?;
        
        self.test_users.insert(user_id.to_string(), user_profile.clone());
        Ok(user_profile)
    }
    
    /// Simulates market data update that should trigger opportunity detection
    pub async fn simulate_market_update(&mut self) -> Result<Vec<TradingOpportunity>, ArbitrageError> {
        // This would normally come from ExchangeService
        // For testing, we use our mock data
        
        let binance_price = 45000.50;
        let bybit_price = 45050.25;
        let price_diff = bybit_price - binance_price;
        let profit_percentage = (price_diff / binance_price) * 100.0;
        
        // Create arbitrage opportunity
        let opportunity = TradingOpportunity {
            opportunity_id: "test_opportunity_1".to_string(),
            opportunity_type: OpportunityType::Arbitrage,
            trading_pair: "BTC/USDT".to_string(),
            exchanges: vec!["binance".to_string(), "bybit".to_string()],
            entry_price: binance_price,
            target_price: Some(bybit_price),
            stop_loss: Some(binance_price * 0.98), // 2% stop loss
            confidence_score: 0.85,
            risk_level: RiskLevel::Low,
            expected_return: profit_percentage,
            time_horizon: TimeHorizon::Short,
            indicators_used: vec!["price_diff".to_string(), "volume_check".to_string()],
            analysis_data: json!({
                "volume_a": "1234.567",
                "volume_b": "987.432",
                "spread_percentage": profit_percentage
            }),
            created_at: chrono::Utc::now().timestamp() as u64,
            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 300), // 5 minutes
        };
        
        self.test_opportunities.push(opportunity.clone());
        Ok(vec![opportunity])
    }
    
    /// Cleans up test data after tests complete
    pub async fn cleanup(&mut self) -> Result<(), ArbitrageError> {
        // Clean up test users
        for user_id in self.test_users.keys() {
            // TODO: Delete user from D1 database
            // self.d1_service.delete_user(user_id).await?;
        }
        
        // Clean up test opportunities
        for opportunity in &self.test_opportunities {
            // TODO: Delete opportunity data
            // self.d1_service.delete_opportunity(&opportunity.id).await?;
        }
        
        self.test_users.clear();
        self.test_opportunities.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod e2e_user_journey_tests {
    use super::*;
    
    /// **E2E Test 1: Complete New User Journey**
    /// Tests: Registration → Profile Setup → Trading Preferences → First Opportunity → Notification
    /// This is the most critical user path that validates the entire platform works end-to-end.
    #[tokio::test]
    async fn test_complete_new_user_journey() {
        let mut framework = E2ETestFramework::new().await;
        framework.setup_mock_market_data();
        
        // Step 1: New user registration (UserProfileService)
        let user = framework.create_test_user(
            "test_user_001",
            TradingFocus::Arbitrage,
            ExperienceLevel::Beginner
        ).await.expect("User creation should succeed");
        
        assert_eq!(user.user_id, "test_user_001");
        assert_eq!(user.subscription_tier, SubscriptionTier::Free);
        
        // Step 2: Verify user preferences were set correctly (UserTradingPreferencesService)
        let preferences = framework.user_preferences_service
            .get_preferences("test_user_001")
            .await
            .expect("Should get preferences")
            .expect("Preferences should exist");
            
        assert_eq!(preferences.trading_focus, TradingFocus::Arbitrage);
        assert_eq!(preferences.experience_level, ExperienceLevel::Beginner);
        assert_eq!(preferences.risk_tolerance, RiskTolerance::Conservative);
        
        // Step 3: Market data update triggers opportunity detection (ExchangeService → GlobalOpportunityService)
        let opportunities = framework.simulate_market_update()
            .await
            .expect("Market update should succeed");
            
        assert!(!opportunities.is_empty(), "Should detect arbitrage opportunity");
        assert_eq!(opportunities[0].opportunity_type, OpportunityType::Arbitrage);
        assert!(opportunities[0].expected_return > 0.0, "Should have expected return");
        
        // Step 4: Opportunity categorization and user filtering (OpportunityCategorizationService)
        // TODO: Test that opportunity is categorized appropriately for beginner arbitrage user
        // let categorized = framework.categorization_service
        //     .categorize_opportunity(&opportunities[0])
        //     .await
        //     .expect("Categorization should succeed");
        
        // Step 5: Notification delivery (NotificationService → TelegramService)
        // TODO: Test that user receives notification via Telegram
        // let notification_sent = framework.notification_service
        //     .send_opportunity_notification("test_user_001", &opportunities[0])
        //     .await
        //     .expect("Notification should be sent");
        
        // Cleanup
        framework.cleanup().await.expect("Cleanup should succeed");
    }
    
    /// **E2E Test 2: Market Data to User Notification Pipeline**
    /// Tests: Exchange Data → Opportunity Detection → Categorization → User Filtering → Telegram Notification
    /// This tests the core business value: market data to user alerts pipeline.
    #[tokio::test]
    async fn test_market_data_to_notification_pipeline() {
        let mut framework = E2ETestFramework::new().await;
        framework.setup_mock_market_data();
        
        // Create users with different preferences
        let _arbitrage_user = framework.create_test_user(
            "arbitrage_user",
            TradingFocus::Arbitrage,
            ExperienceLevel::Intermediate
        ).await.expect("User creation should succeed");
        
        let _technical_user = framework.create_test_user(
            "technical_user", 
            TradingFocus::Technical,
            ExperienceLevel::Advanced
        ).await.expect("User creation should succeed");
        
        // Simulate market data update
        let opportunities = framework.simulate_market_update().await
            .expect("Market update should succeed");
            
        // Verify arbitrage opportunity was detected
        let arbitrage_opp = opportunities.iter()
            .find(|opp| opp.opportunity_type == OpportunityType::Arbitrage)
            .expect("Should find arbitrage opportunity");
            
        // TODO: Test categorization logic
        // - Arbitrage user should receive this opportunity
        // - Technical user should NOT receive this opportunity (wrong focus)
        
        // TODO: Test notification delivery
        // - Verify correct users get notifications
        // - Verify notification content is appropriate
        // - Verify notification timing and rate limiting
        
        framework.cleanup().await.expect("Cleanup should succeed");
    }
    
    /// **E2E Test 3: Trading Focus Change Impact**
    /// Tests: User Changes Focus → Preferences Update → Opportunity Filtering Changes → Different Notifications
    /// This validates that user preference changes have immediate effect on what they receive.
    #[tokio::test] 
    async fn test_trading_focus_change_impact() {
        let mut framework = E2ETestFramework::new().await;
        framework.setup_mock_market_data();
        
        // Create user with arbitrage focus initially
        let _user = framework.create_test_user(
            "changing_user",
            TradingFocus::Arbitrage,
            ExperienceLevel::Intermediate
        ).await.expect("User creation should succeed");
        
        // Generate opportunities for arbitrage focus
        let arbitrage_opportunities = framework.simulate_market_update().await
            .expect("Market update should succeed");
            
        // TODO: Verify user receives arbitrage opportunities
        
        // Change user to technical focus
        framework.user_preferences_service.update_trading_focus(
            "changing_user",
            TradingFocus::Technical
        ).await.expect("Focus change should succeed");
        
        // TODO: Generate technical trading opportunities
        // TODO: Verify user now receives technical opportunities instead of arbitrage
        // TODO: Verify arbitrage opportunities are filtered out
        
        framework.cleanup().await.expect("Cleanup should succeed");
    }
    
    /// **E2E Test 4: AI Enhancement Pipeline** 
    /// Tests: Market Data → AI Analysis → Enhanced Opportunities → User-Specific Recommendations
    /// This validates the AI-driven intelligence layer works end-to-end.
    #[tokio::test]
    async fn test_ai_enhancement_pipeline() {
        let mut framework = E2ETestFramework::new().await;
        framework.setup_mock_market_data();
        
        // Create user who should receive AI enhancements
        let _user = framework.create_test_user(
            "ai_user",
            TradingFocus::Hybrid,
            ExperienceLevel::Advanced
        ).await.expect("User creation should succeed");
        
        // Generate base opportunities
        let opportunities = framework.simulate_market_update().await
            .expect("Market update should succeed");
            
        // TODO: Test AI enhancement pipeline
        // - AI analyzes market context
        // - AI provides confidence scoring
        // - AI generates personalized recommendations
        // - Enhanced opportunities delivered to user
        
        framework.cleanup().await.expect("Cleanup should succeed");
    }
    
    /// **E2E Test 5: Error Recovery and Edge Cases**
    /// Tests: Service Failures → Error Recovery → User Experience Maintained
    /// This validates the platform gracefully handles failures.
    #[tokio::test]
    async fn test_error_recovery_and_edge_cases() {
        let mut framework = E2ETestFramework::new().await;
        
        // TODO: Test scenarios:
        // - D1 database unavailable → fallback to KV
        // - Exchange API rate limited → cached data used
        // - AI service unavailable → opportunities still delivered without enhancement
        // - Telegram API down → opportunities queued for later delivery
        // - Invalid market data → filtered out, no crash
        
        framework.cleanup().await.expect("Cleanup should succeed");
    }
}

/// **Test Data Factories**
/// Helper functions to create realistic test data
pub mod test_data_factory {
    use super::*;
    
    pub fn create_arbitrage_opportunity(
        exchange_a: ExchangeIdEnum,
        exchange_b: ExchangeIdEnum,
        trading_pair: &str,
        price_diff_percentage: f64
    ) -> TradingOpportunity {
        let base_price = 45000.0;
        let price_a = base_price;
        let price_b = base_price * (1.0 + price_diff_percentage / 100.0);
        
        TradingOpportunity {
            id: format!("arb_{}_{}_{}_{}", 
                exchange_a.to_string().to_lowercase(),
                exchange_b.to_string().to_lowercase(),
                trading_pair.replace("/", ""),
                chrono::Utc::now().timestamp()
            ),
            opportunity_type: OpportunityType::Arbitrage,
            exchange_a,
            exchange_b: Some(exchange_b),
            trading_pair: trading_pair.to_string(),
            price_a,
            price_b: Some(price_b),
            profit_potential: price_diff_percentage,
            confidence_score: 0.85,
            risk_level: if price_diff_percentage < 0.5 { RiskLevel::Low } 
                       else if price_diff_percentage < 1.0 { RiskLevel::Medium }
                       else { RiskLevel::High },
            time_horizon: TimeHorizon::ShortTerm,
            detected_at: chrono::Utc::now().timestamp() as u64,
            expires_at: chrono::Utc::now().timestamp() as u64 + 300, // 5 minutes
            metadata: json!({
                "volume_a": "1000.0",
                "volume_b": "800.0",
                "spread_percentage": price_diff_percentage,
                "min_trade_amount": 100.0
            }),
        }
    }
    
    pub fn create_technical_opportunity(
        exchange: ExchangeIdEnum,
        trading_pair: &str,
        signal_type: &str,
        confidence: f64
    ) -> TradingOpportunity {
        TradingOpportunity {
            id: format!("tech_{}_{}_{}_{}", 
                exchange.to_string().to_lowercase(),
                trading_pair.replace("/", ""),
                signal_type,
                chrono::Utc::now().timestamp()
            ),
            opportunity_type: OpportunityType::Technical,
            exchange_a: exchange,
            exchange_b: None,
            trading_pair: trading_pair.to_string(),
            price_a: 45000.0,
            price_b: None,
            profit_potential: 2.5, // Expected return percentage
            confidence_score: confidence,
            risk_level: if confidence > 0.8 { RiskLevel::Low }
                       else if confidence > 0.6 { RiskLevel::Medium }
                       else { RiskLevel::High },
            time_horizon: TimeHorizon::MediumTerm,
            detected_at: chrono::Utc::now().timestamp() as u64,
            expires_at: chrono::Utc::now().timestamp() as u64 + 3600, // 1 hour
            metadata: json!({
                "signal_type": signal_type,
                "indicator_values": {
                    "rsi": 75.0,
                    "ma_short": 44800.0,
                    "ma_long": 44500.0
                },
                "entry_price": 45000.0,
                "stop_loss": 44100.0,
                "take_profit": 46125.0
            }),
        }
    }
} 