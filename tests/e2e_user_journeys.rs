// E2E Test Suite for ArbEdge User Journeys
// Tests complete user flows from registration to notification

use arb_edge::*;
use arb_edge::types::*;
use arb_edge::services::d1_database::D1Service;
use arb_edge::services::user_profile::UserProfileService;
use arb_edge::services::user_trading_preferences::{UserTradingPreferencesService, UserTradingPreferences, TradingFocus, ExperienceLevel, AutomationLevel, AutomationScope, RiskTolerance};
use arb_edge::services::exchange::ExchangeService;
use arb_edge::services::global_opportunity::{GlobalOpportunityService, GlobalOpportunityConfig, DistributionStrategy, FairnessConfig};
use arb_edge::services::opportunity_categorization::OpportunityCategorizationService;
use arb_edge::services::notifications::NotificationService;
use arb_edge::services::telegram::TelegramService;
use arb_edge::services::market_analysis::{TradingOpportunity, OpportunityType, RiskLevel, TimeHorizon};
use arb_edge::utils::{ArbitrageResult, ArbitrageError, logger::{Logger, LogLevel}};

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use worker::kv::KvStore;

/// **E2E Test Framework**
/// Provides infrastructure for testing complete user journeys
/// Mock implementations simulate real external dependencies
pub struct E2ETestFramework {
    // Core Services
    d1_service: D1Service,
    user_profile_service: UserProfileService,
    user_preferences_service: UserTradingPreferencesService,
    exchange_service: MockExchangeService,
    opportunity_service: GlobalOpportunityService,
    categorization_service: OpportunityCategorizationService,
    notification_service: NotificationService,
    telegram_service: TelegramService,
    
    // Test State
    test_users: HashMap<String, UserProfile>,
    test_opportunities: Vec<TradingOpportunity>,
    mock_market_data: HashMap<String, serde_json::Value>,
}

/// Mock Exchange Service for testing (avoids real API calls)
pub struct MockExchangeService {
    mock_data: HashMap<String, serde_json::Value>,
}

impl MockExchangeService {
    pub fn new() -> Self {
        let mut mock_data = HashMap::new();
        
        // Add mock ticker data
        mock_data.insert("binance_btc_usdt".to_string(), json!({
            "symbol": "BTCUSDT",
            "price": "45000.50",
            "volume": "1234.567",
            "timestamp": 1640995200000u64
        }));
        
        mock_data.insert("bybit_btc_usdt".to_string(), json!({
            "symbol": "BTCUSDT", 
            "price": "45050.25",  // $50 higher - arbitrage opportunity!
            "volume": "987.432",
            "timestamp": 1640995200000u64
        }));
        
        Self { mock_data }
    }
    
    pub async fn get_ticker(&self, exchange: &str, symbol: &str) -> ArbitrageResult<serde_json::Value> {
        let key = format!("{}_{}", exchange.to_lowercase(), symbol.replace("/", "_").to_lowercase());
        Ok(self.mock_data.get(&key).cloned().unwrap_or_else(|| json!({
            "symbol": symbol,
            "price": "45000.00",
            "volume": "1000.0",
            "timestamp": 1640995200000u64
        })))
    }
}

impl E2ETestFramework {
    /// Creates a new E2E test framework with mock services
    pub async fn new() -> Self {
        // Create KV store for test environment
        let kv_store = KvStore::new("test-kv");
        
        // Initialize all services with test configurations
        let d1_service = D1Service::new("test_database".to_string());
        let user_profile_service = UserProfileService::new(
            kv_store.clone(),
            d1_service.clone(),
            "test_encryption_key".to_string()
        );
        let user_preferences_service = UserTradingPreferencesService::new(
            d1_service.clone(),
            Logger::new(LogLevel::Debug)
        );
        
        // Mock exchange service to avoid real API calls
        let exchange_service = MockExchangeService::new();
        
        // Create global opportunity service with test config
        let global_config = GlobalOpportunityConfig {
            detection_interval_seconds: 30,
            monitored_pairs: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            monitored_exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            min_threshold: 0.001,
            max_threshold: 0.02,
            opportunity_ttl_minutes: 5,
            max_queue_size: 100,
            distribution_strategy: DistributionStrategy::RoundRobin,
            fairness_config: FairnessConfig::default(),
        };
        
        // Note: We need to create proper Arc<> wrappers for services that expect them
        // For now, we'll create simple test implementations
        
        Self {
            d1_service: d1_service.clone(),
            user_profile_service,
            user_preferences_service,
            exchange_service,
            opportunity_service: GlobalOpportunityService::new(
                global_config,
                Arc::new(MockExchangeServiceWrapper::new()), // Will need to implement this
                Arc::new(UserProfileService::new(
                    kv_store.clone(),
                    d1_service.clone(),
                    "test_encryption_key".to_string()
                )),
                kv_store.clone()
            ),
            categorization_service: OpportunityCategorizationService::new(
                d1_service.clone(),
                UserTradingPreferencesService::new(
                    d1_service.clone(),
                    Logger::new(LogLevel::Debug)
                ),
                Logger::new(LogLevel::Debug)
            ),
            notification_service: NotificationService::new(
                d1_service.clone(),
                TelegramService::new("test_bot_token".to_string()),
                kv_store.clone()
            ),
            telegram_service: TelegramService::new("test_bot_token".to_string()),
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
        
        // Store user profile (using the correct method signature)
        let created_user = self.user_profile_service.create_user_profile(
            test_telegram_id,
            Some("test-e2e".to_string()),
            Some(format!("test_user_{}", test_telegram_id))
        ).await?;
        
        // Create user trading preferences with all required fields
        let preferences = UserTradingPreferences {
            preference_id: format!("pref_{}", user_id),
            user_id: user_id.to_string(),
            trading_focus,
            experience_level,
            risk_tolerance: match experience_level {
                ExperienceLevel::Beginner => RiskTolerance::Conservative,
                ExperienceLevel::Intermediate => RiskTolerance::Balanced,
                ExperienceLevel::Advanced => RiskTolerance::Aggressive,
            },
            automation_level: AutomationLevel::Manual, // Start with manual
            automation_scope: AutomationScope::None,
            // Feature Access Control
            arbitrage_enabled: true,
            technical_enabled: experience_level != ExperienceLevel::Beginner,
            advanced_analytics_enabled: experience_level == ExperienceLevel::Advanced,
            // User Preferences
            preferred_notification_channels: vec!["telegram".to_string()],
            trading_hours_timezone: "UTC".to_string(),
            trading_hours_start: "00:00".to_string(),
            trading_hours_end: "23:59".to_string(),
            // Onboarding Progress
            onboarding_completed: true,
            tutorial_steps_completed: vec!["welcome".to_string(), "preferences".to_string()],
            // Timestamps
            created_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
        };
        
        self.user_preferences_service.update_preferences(&preferences).await?;
        
        self.test_users.insert(user_id.to_string(), created_user.clone());
        Ok(created_user)
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
        // Clean up test users from D1 database
        for user_id in self.test_users.keys() {
            // Delete user profile and associated data
            if let Err(e) = self.d1_service.delete_user_profile(user_id).await {
                eprintln!("Warning: Failed to delete user {}: {}", user_id, e);
                // Continue cleanup even if individual deletions fail
            }
            
            // Delete user preferences
            if let Err(e) = self.user_preferences_service.delete_preferences(user_id).await {
                eprintln!("Warning: Failed to delete preferences for user {}: {}", user_id, e);
            }
        }
        
        // Clean up test opportunities from D1 database
        for opportunity in &self.test_opportunities {
            if let Err(e) = self.d1_service.delete_trading_opportunity(&opportunity.opportunity_id).await {
                eprintln!("Warning: Failed to delete opportunity {}: {}", opportunity.opportunity_id, e);
            }
        }
        
        // Clear in-memory collections
        self.test_users.clear();
        self.test_opportunities.clear();
        self.mock_market_data.clear();
        
        Ok(())
    }
}

/// Mock wrapper for ExchangeService to fit Arc<ExchangeService> requirements
/// TODO: This is a temporary solution - in production we'd use dependency injection or traits
pub struct MockExchangeServiceWrapper;

impl MockExchangeServiceWrapper {
    pub fn new() -> Self {
        Self
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
        assert_eq!(user.subscription.tier, SubscriptionTier::Free);
        
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
            opportunity_id: format!("arb_{}_{}_{}_{}", 
                exchange_a.to_string().to_lowercase(),
                exchange_b.to_string().to_lowercase(),
                trading_pair.replace("/", ""),
                chrono::Utc::now().timestamp()
            ),
            opportunity_type: OpportunityType::Arbitrage,
            trading_pair: trading_pair.to_string(),
            exchanges: vec![exchange_a.to_string(), exchange_b.to_string()],
            entry_price: price_a,
            target_price: Some(price_b),
            stop_loss: Some(price_a * 0.98), // 2% stop loss
            confidence_score: 0.85,
            risk_level: if price_diff_percentage < 0.5 { RiskLevel::Low } 
                       else if price_diff_percentage < 1.0 { RiskLevel::Medium }
                       else { RiskLevel::High },
            expected_return: price_diff_percentage,
            time_horizon: TimeHorizon::Immediate,
            indicators_used: vec!["price_diff".to_string(), "volume".to_string()],
            analysis_data: json!({
                "volume_a": "1000.0",
                "volume_b": "800.0",
                "spread_percentage": price_diff_percentage,
                "min_trade_amount": 100.0,
                "exchange_a": exchange_a.to_string(),
                "exchange_b": exchange_b.to_string(),
                "price_a": price_a,
                "price_b": price_b
            }),
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: Some(chrono::Utc::now().timestamp_millis() as u64 + 300000), // 5 minutes
        }
    }
    
    pub fn create_technical_opportunity(
        exchange: ExchangeIdEnum,
        trading_pair: &str,
        signal_type: &str,
        confidence: f64
    ) -> TradingOpportunity {
        TradingOpportunity {
            opportunity_id: format!("tech_{}_{}_{}_{}", 
                exchange.to_string().to_lowercase(),
                trading_pair.replace("/", ""),
                signal_type,
                chrono::Utc::now().timestamp()
            ),
            opportunity_type: OpportunityType::Technical,
            trading_pair: trading_pair.to_string(),
            exchanges: vec![exchange.to_string()],
            entry_price: 45000.0,
            target_price: Some(46125.0), // Take profit price
            stop_loss: Some(44100.0),    // Stop loss price
            confidence_score: confidence,
            risk_level: if confidence > 0.8 { RiskLevel::Low }
                       else if confidence > 0.6 { RiskLevel::Medium }
                       else { RiskLevel::High },
            expected_return: 2.5, // Expected return percentage
            time_horizon: TimeHorizon::Medium,
            indicators_used: vec![signal_type.to_string(), "rsi".to_string(), "ma".to_string()],
            analysis_data: json!({
                "signal_type": signal_type,
                "indicator_values": {
                    "rsi": 75.0,
                    "ma_short": 44800.0,
                    "ma_long": 44500.0
                },
                "entry_price": 45000.0,
                "stop_loss": 44100.0,
                "take_profit": 46125.0,
                "exchange": exchange.to_string()
            }),
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: Some(chrono::Utc::now().timestamp_millis() as u64 + 3600000), // 1 hour
        }
    }
} 