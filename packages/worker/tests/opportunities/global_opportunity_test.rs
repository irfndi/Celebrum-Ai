#![allow(unused_imports, unused_variables, unused_mut, dead_code)]

// GlobalOpportunityService Unit Tests
// Comprehensive testing of opportunity generation, distribution, user eligibility, and queue management

use cerebrum_ai::services::core::analysis::market_analysis::OpportunityType;
use cerebrum_ai::types::{ExchangeIdEnum, RiskLevel, UserAccessLevel, UserOpportunityLimits};
use cerebrum_ai::utils::{ArbitrageError, ArbitrageResult};
use std::collections::HashMap;
use uuid::Uuid;

// Mock configuration structures for testing
#[derive(Debug, Clone)]
struct MockGlobalOpportunityConfig {
    pub max_queue_size: usize,
    pub opportunity_ttl_minutes: u32,
    pub distribution_delay_seconds: u64,
    pub max_opportunities_per_user_per_day: u32,
    pub fairness_enabled: bool,
    pub activity_boost_enabled: bool,
    pub priority_scoring_enabled: bool,
}

#[derive(Debug, Clone)]
struct MockFairnessConfig {
    pub base_fairness_score: f64,
    pub max_fairness_boost: f64,
    pub fairness_decay_rate: f64,
    pub consecutive_opportunity_penalty: f64,
    pub time_since_last_opportunity_boost: f64,
}

#[derive(Debug, Clone)]
struct MockActivityBoostConfig {
    pub base_activity_multiplier: f64,
    pub max_activity_boost: f64,
    pub consecutive_days_threshold: u32,
    pub daily_activity_boost: f64,
    pub weekly_activity_bonus: f64,
}

#[derive(Debug, Clone)]
struct MockOpportunityQueue {
    pub opportunities: Vec<MockOpportunity>,
    pub max_size: usize,
    pub priority_sorted: bool,
}

#[derive(Debug, Clone)]
struct MockUserDistributionTracking {
    pub user_id: String,
    pub opportunities_received_today: u32,
    pub last_opportunity_at: Option<u64>,
    pub activity_boost_multiplier: f64,
    pub fairness_score: f64,
    pub consecutive_days_active: u32,
    pub total_opportunities_received: u32,
    pub last_reset_date: String,
}

// Mock D1Service for testing
struct MockD1Service {
    user_limits: HashMap<String, UserOpportunityLimits>,
    user_distribution_tracking: HashMap<String, MockUserDistributionTracking>,
    opportunity_queue: Vec<MockOpportunity>,
    error_simulation: Option<String>,
}

#[derive(Debug, Clone)]
struct MockOpportunity {
    pub opportunity_id: String,
    pub opportunity_type: OpportunityType,
    pub risk_level: RiskLevel,
    pub confidence_score: f64,
    pub expected_return: f64,
    pub trading_pair: String,
    pub exchanges: Vec<ExchangeIdEnum>,
    pub created_at: u64,
    pub expires_at: u64,
    pub priority_score: f64,
    pub user_eligibility: Vec<String>, // User IDs eligible for this opportunity
}

impl MockD1Service {
    fn new() -> Self {
        Self {
            user_limits: HashMap::new(),
            user_distribution_tracking: HashMap::new(),
            opportunity_queue: Vec::new(),
            error_simulation: None,
        }
    }

    fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    fn reset_error_simulation(&mut self) {
        self.error_simulation = None;
    }

    async fn mock_get_user_limits(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<UserOpportunityLimits>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "database_error" => {
                    Err(ArbitrageError::database_error("Database connection failed"))
                }
                "user_not_found" => Ok(None),
                _ => Err(ArbitrageError::validation_error("Unknown database error")),
            };
        }

        Ok(self.user_limits.get(user_id).cloned())
    }

    async fn mock_update_user_distribution(
        &mut self,
        user_id: &str,
        opportunity_id: &str,
    ) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "update_failed" => Err(ArbitrageError::database_error(
                    "Failed to update distribution tracking",
                )),
                _ => Err(ArbitrageError::validation_error("Unknown update error")),
            };
        }

        let tracking = self
            .user_distribution_tracking
            .entry(user_id.to_string())
            .or_insert_with(|| MockUserDistributionTracking {
                user_id: user_id.to_string(),
                opportunities_received_today: 0,
                last_opportunity_at: None,
                activity_boost_multiplier: 1.0,
                fairness_score: 1.0,
                consecutive_days_active: 0,
                total_opportunities_received: 0,
                last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            });

        tracking.opportunities_received_today += 1;
        tracking.total_opportunities_received += 1;
        tracking.last_opportunity_at = Some(chrono::Utc::now().timestamp_millis() as u64);

        Ok(())
    }

    fn add_mock_user_limits(&mut self, user_id: &str, limits: UserOpportunityLimits) {
        self.user_limits.insert(user_id.to_string(), limits);
    }

    fn add_mock_opportunity(&mut self, opportunity: MockOpportunity) {
        self.opportunity_queue.push(opportunity);
    }

    fn get_opportunity_count(&self) -> usize {
        self.opportunity_queue.len()
    }

    fn get_opportunities_for_user(&self, user_id: &str) -> Vec<&MockOpportunity> {
        self.opportunity_queue
            .iter()
            .filter(|opp| opp.user_eligibility.contains(&user_id.to_string()))
            .collect()
    }
}

// Mock GlobalOpportunityService for testing
struct MockGlobalOpportunityService {
    d1_service: MockD1Service,
    config: MockGlobalOpportunityConfig,
    fairness_config: MockFairnessConfig,
    activity_boost_config: MockActivityBoostConfig,
    opportunity_queue: MockOpportunityQueue,
    user_access_service: MockUserAccessService,
}

struct MockUserAccessService {
    user_access_levels: HashMap<String, UserAccessLevel>,
}

impl MockUserAccessService {
    fn new() -> Self {
        Self {
            user_access_levels: HashMap::new(),
        }
    }

    fn set_user_access_level(&mut self, user_id: &str, access_level: UserAccessLevel) {
        self.user_access_levels
            .insert(user_id.to_string(), access_level);
    }

    async fn mock_get_user_access_level(&self, user_id: &str) -> ArbitrageResult<UserAccessLevel> {
        Ok(self
            .user_access_levels
            .get(user_id)
            .cloned()
            .unwrap_or(UserAccessLevel::FreeWithoutAPI))
    }

    async fn mock_check_user_eligibility(
        &self,
        user_id: &str,
        opportunity: &MockOpportunity,
    ) -> ArbitrageResult<bool> {
        let access_level = self.mock_get_user_access_level(user_id).await?;

        // Basic eligibility logic
        Ok(match access_level {
            UserAccessLevel::Guest
            | UserAccessLevel::Free
            | UserAccessLevel::Registered
            | UserAccessLevel::Verified
            | UserAccessLevel::FreeWithoutAPI
            | UserAccessLevel::Basic
            | UserAccessLevel::User => {
                // Basic access: Low risk, high confidence
                opportunity.risk_level == RiskLevel::Low && opportunity.confidence_score >= 70.0
            }
            UserAccessLevel::FreeWithAPI => {
                // Free with API: Medium risk, moderate confidence
                opportunity.risk_level != RiskLevel::High && opportunity.confidence_score >= 60.0
            }
            UserAccessLevel::Paid
            | UserAccessLevel::Premium
            | UserAccessLevel::Admin
            | UserAccessLevel::SuperAdmin
            | UserAccessLevel::BetaUser
            | UserAccessLevel::SubscriptionWithAPI => {
                // Higher tiers & special access: All opportunities
                true
            }
        })
    }
}

impl MockGlobalOpportunityService {
    fn new() -> Self {
        Self {
            d1_service: MockD1Service::new(),
            config: MockGlobalOpportunityConfig {
                max_queue_size: 100,
                opportunity_ttl_minutes: 30,
                distribution_delay_seconds: 300, // 5 minutes
                max_opportunities_per_user_per_day: 20,
                fairness_enabled: true,
                activity_boost_enabled: true,
                priority_scoring_enabled: true,
            },
            fairness_config: MockFairnessConfig {
                base_fairness_score: 1.0,
                max_fairness_boost: 2.0,
                fairness_decay_rate: 0.1,
                consecutive_opportunity_penalty: 0.2,
                time_since_last_opportunity_boost: 0.1,
            },
            activity_boost_config: MockActivityBoostConfig {
                base_activity_multiplier: 1.0,
                max_activity_boost: 3.0,
                consecutive_days_threshold: 7,
                daily_activity_boost: 0.2,
                weekly_activity_bonus: 0.5,
            },
            opportunity_queue: MockOpportunityQueue {
                opportunities: Vec::new(),
                max_size: 100,
                priority_sorted: true,
            },
            user_access_service: MockUserAccessService::new(),
        }
    }

    async fn mock_generate_arbitrage_opportunity(
        &mut self,
        trading_pair: &str,
        exchanges: Vec<ExchangeIdEnum>,
    ) -> ArbitrageResult<MockOpportunity> {
        if exchanges.len() != 2 {
            return Err(ArbitrageError::validation_error(
                "Arbitrage opportunities require exactly 2 exchanges",
            ));
        }

        let opportunity = MockOpportunity {
            opportunity_id: format!("arb_{}", Uuid::new_v4()),
            opportunity_type: OpportunityType::Arbitrage,
            risk_level: RiskLevel::Medium,
            confidence_score: 75.0,
            expected_return: 2.5,
            trading_pair: trading_pair.to_string(),
            exchanges,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + (30 * 60 * 1000), // 30 minutes
            priority_score: 0.0,          // Will be calculated
            user_eligibility: Vec::new(), // Will be populated during distribution
        };

        Ok(opportunity)
    }

    async fn mock_generate_technical_opportunity(
        &mut self,
        trading_pair: &str,
        exchange: ExchangeIdEnum,
    ) -> ArbitrageResult<MockOpportunity> {
        let opportunity = MockOpportunity {
            opportunity_id: format!("tech_{}", Uuid::new_v4()),
            opportunity_type: OpportunityType::Technical,
            risk_level: RiskLevel::Low,
            confidence_score: 80.0,
            expected_return: 1.8,
            trading_pair: trading_pair.to_string(),
            exchanges: vec![exchange],
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + (15 * 60 * 1000), // 15 minutes
            priority_score: 0.0,          // Will be calculated
            user_eligibility: Vec::new(), // Will be populated during distribution
        };

        Ok(opportunity)
    }

    fn mock_calculate_priority_score(
        &self,
        opportunity: &MockOpportunity,
        user_tracking: &MockUserDistributionTracking,
    ) -> f64 {
        let mut score = 0.0;

        // Base score from opportunity characteristics
        score += opportunity.confidence_score * 0.4; // 40% weight on confidence
        score += opportunity.expected_return * 10.0; // 10x weight on expected return

        // Risk level adjustment
        score += match opportunity.risk_level {
            RiskLevel::Low => 20.0,
            RiskLevel::Medium => 15.0,
            RiskLevel::High => 10.0,
            RiskLevel::Critical => 5.0,
        };

        // Opportunity type adjustment
        score += match opportunity.opportunity_type {
            OpportunityType::Arbitrage => 25.0, // Higher priority for arbitrage
            OpportunityType::Technical => 20.0,
            OpportunityType::ArbitrageTechnical => 30.0, // Highest priority for combined
        };

        // User fairness adjustment
        score *= user_tracking.fairness_score;

        // Activity boost adjustment
        score *= user_tracking.activity_boost_multiplier;

        score
    }

    async fn mock_distribute_opportunity(
        &mut self,
        mut opportunity: MockOpportunity,
        eligible_users: Vec<String>,
    ) -> ArbitrageResult<Vec<String>> {
        let mut distributed_users = Vec::new();

        for user_id in eligible_users {
            // Check user limits
            if let Some(limits) = self.d1_service.mock_get_user_limits(&user_id).await? {
                let total_used = limits.arbitrage_received_today + limits.technical_received_today;
                let total_limit =
                    limits.daily_global_opportunities + limits.daily_technical_opportunities;
                if total_used >= total_limit {
                    continue; // Skip user who has reached daily limit
                }
            }

            // Check user eligibility based on access level
            if !self
                .user_access_service
                .mock_check_user_eligibility(&user_id, &opportunity)
                .await?
            {
                continue; // Skip ineligible user
            }

            // Add user to opportunity eligibility
            opportunity.user_eligibility.push(user_id.clone());
            distributed_users.push(user_id.clone());

            // Update user distribution tracking
            self.d1_service
                .mock_update_user_distribution(&user_id, &opportunity.opportunity_id)
                .await?;

            // Respect distribution limits
            if distributed_users.len() >= 10 {
                // Max 10 users per opportunity
                break;
            }
        }

        // Add opportunity to queue
        self.d1_service.add_mock_opportunity(opportunity);

        Ok(distributed_users)
    }

    fn mock_calculate_activity_boost(&self, user_tracking: &MockUserDistributionTracking) -> f64 {
        let mut boost = self.activity_boost_config.base_activity_multiplier;

        // Consecutive days boost
        if user_tracking.consecutive_days_active
            >= self.activity_boost_config.consecutive_days_threshold
        {
            boost += self.activity_boost_config.weekly_activity_bonus;
        }

        // Daily activity boost
        boost += user_tracking.consecutive_days_active as f64
            * self.activity_boost_config.daily_activity_boost;

        // Cap the boost
        boost.min(self.activity_boost_config.max_activity_boost)
    }

    fn mock_calculate_fairness_score(&self, user_tracking: &MockUserDistributionTracking) -> f64 {
        let mut score = self.fairness_config.base_fairness_score;

        // Time since last opportunity boost
        if let Some(last_opportunity_at) = user_tracking.last_opportunity_at {
            let time_since_last =
                chrono::Utc::now().timestamp_millis() as u64 - last_opportunity_at;
            let hours_since_last = time_since_last as f64 / (1000.0 * 60.0 * 60.0);
            score += hours_since_last * self.fairness_config.time_since_last_opportunity_boost;
        } else {
            // New user gets maximum fairness boost
            score = self.fairness_config.max_fairness_boost;
        }

        // Cap the score
        score.min(self.fairness_config.max_fairness_boost)
    }

    async fn mock_cleanup_expired_opportunities(&mut self) -> ArbitrageResult<u32> {
        let current_time = chrono::Utc::now().timestamp_millis() as u64;
        let initial_count = self.d1_service.opportunity_queue.len();

        self.d1_service
            .opportunity_queue
            .retain(|opp| opp.expires_at > current_time);

        let removed_count = initial_count - self.d1_service.opportunity_queue.len();
        Ok(removed_count as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_opportunity_generation_and_validation() {
        let mut service = MockGlobalOpportunityService::new();

        // Test arbitrage opportunity generation
        let arbitrage_result = service
            .mock_generate_arbitrage_opportunity(
                "BTC/USDT",
                vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            )
            .await;

        assert!(arbitrage_result.is_ok());
        let arbitrage_opp = arbitrage_result.unwrap();
        assert_eq!(arbitrage_opp.opportunity_type, OpportunityType::Arbitrage);
        assert_eq!(arbitrage_opp.exchanges.len(), 2);
        assert!(arbitrage_opp.confidence_score > 0.0);
        assert!(arbitrage_opp.expected_return > 0.0);

        // Test technical opportunity generation
        let technical_result = service
            .mock_generate_technical_opportunity("ETH/USDT", ExchangeIdEnum::Binance)
            .await;

        assert!(technical_result.is_ok());
        let technical_opp = technical_result.unwrap();
        assert_eq!(technical_opp.opportunity_type, OpportunityType::Technical);
        assert_eq!(technical_opp.exchanges.len(), 1);
        assert!(technical_opp.confidence_score > 0.0);
        assert!(technical_opp.expected_return > 0.0);

        // Test invalid arbitrage opportunity (wrong number of exchanges)
        let invalid_arbitrage = service
            .mock_generate_arbitrage_opportunity(
                "BTC/USDT",
                vec![ExchangeIdEnum::Binance], // Only 1 exchange
            )
            .await;

        assert!(invalid_arbitrage.is_err());
        assert!(invalid_arbitrage
            .unwrap_err()
            .to_string()
            .contains("exactly 2 exchanges"));
    }

    #[tokio::test]
    async fn test_user_eligibility_and_access_levels() {
        let mut service = MockGlobalOpportunityService::new();

        // Set up test users with different access levels
        service
            .user_access_service
            .set_user_access_level("free_user", UserAccessLevel::FreeWithoutAPI);
        service
            .user_access_service
            .set_user_access_level("api_user", UserAccessLevel::FreeWithAPI);
        service
            .user_access_service
            .set_user_access_level("premium_user", UserAccessLevel::SubscriptionWithAPI);

        // Create opportunities with different risk levels
        let low_risk_opp = MockOpportunity {
            opportunity_id: "low_risk".to_string(),
            opportunity_type: OpportunityType::Technical,
            risk_level: RiskLevel::Low,
            confidence_score: 80.0,
            expected_return: 1.5,
            trading_pair: "BTC/USDT".to_string(),
            exchanges: vec![ExchangeIdEnum::Binance],
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + 1800000,
            priority_score: 0.0,
            user_eligibility: Vec::new(),
        };

        let high_risk_opp = MockOpportunity {
            opportunity_id: "high_risk".to_string(),
            opportunity_type: OpportunityType::Arbitrage,
            risk_level: RiskLevel::High,
            confidence_score: 60.0,
            expected_return: 5.0,
            trading_pair: "ETH/USDT".to_string(),
            exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + 1800000,
            priority_score: 0.0,
            user_eligibility: Vec::new(),
        };

        // Test eligibility for different user types
        let free_user_low_risk = service
            .user_access_service
            .mock_check_user_eligibility("free_user", &low_risk_opp)
            .await
            .unwrap();
        let free_user_high_risk = service
            .user_access_service
            .mock_check_user_eligibility("free_user", &high_risk_opp)
            .await
            .unwrap();
        let api_user_high_risk = service
            .user_access_service
            .mock_check_user_eligibility("api_user", &high_risk_opp)
            .await
            .unwrap();
        let premium_user_high_risk = service
            .user_access_service
            .mock_check_user_eligibility("premium_user", &high_risk_opp)
            .await
            .unwrap();

        // Free users should only get low risk opportunities
        assert!(free_user_low_risk);
        assert!(!free_user_high_risk);

        // API users should not get high risk opportunities
        assert!(!api_user_high_risk);

        // Premium users should get all opportunities
        assert!(premium_user_high_risk);
    }

    #[tokio::test]
    async fn test_opportunity_distribution_and_limits() {
        let mut service = MockGlobalOpportunityService::new();

        // Set up user limits
        service.d1_service.add_mock_user_limits(
            "user1",
            UserOpportunityLimits {
                daily_global_opportunities: 10,
                daily_technical_opportunities: 10,
                daily_ai_opportunities: 5,
                hourly_rate_limit: 100,
                can_receive_realtime: true,
                delay_seconds: 0,
                arbitrage_received_today: 3, // Mock previous usage
                technical_received_today: 2, // Mock previous usage
                current_arbitrage_count: 3,  // Mock current count
                current_technical_count: 2,  // Mock current count
            },
        );

        service.d1_service.add_mock_user_limits(
            "user2",
            UserOpportunityLimits {
                daily_global_opportunities: 10,
                daily_technical_opportunities: 10,
                daily_ai_opportunities: 5,
                hourly_rate_limit: 100,
                can_receive_realtime: true,
                delay_seconds: 0,
                arbitrage_received_today: 10, // Mock user at limit
                technical_received_today: 10, // Mock user at limit
                current_arbitrage_count: 10,  // Mock current count
                current_technical_count: 10,  // Mock current count
            },
        );

        // Set user access levels
        service
            .user_access_service
            .set_user_access_level("user1", UserAccessLevel::FreeWithAPI);
        service
            .user_access_service
            .set_user_access_level("user2", UserAccessLevel::FreeWithAPI);

        // Create a test opportunity
        let opportunity = service
            .mock_generate_technical_opportunity("BTC/USDT", ExchangeIdEnum::Binance)
            .await
            .unwrap();

        // Test distribution
        let eligible_users = vec!["user1".to_string(), "user2".to_string()];
        let distributed_users = service
            .mock_distribute_opportunity(opportunity, eligible_users)
            .await
            .unwrap();

        // Only user1 should receive the opportunity (user2 is at limit)
        assert_eq!(distributed_users.len(), 1);
        assert_eq!(distributed_users[0], "user1");

        // Verify opportunity was added to queue
        assert_eq!(service.d1_service.get_opportunity_count(), 1);
    }

    #[tokio::test]
    async fn test_priority_scoring_algorithm() {
        let service = MockGlobalOpportunityService::new();

        // Create test user tracking data
        let high_activity_user = MockUserDistributionTracking {
            user_id: "active_user".to_string(),
            opportunities_received_today: 2,
            last_opportunity_at: Some(chrono::Utc::now().timestamp_millis() as u64 - 3600000), // 1 hour ago
            activity_boost_multiplier: 2.0,
            fairness_score: 1.5,
            consecutive_days_active: 10,
            total_opportunities_received: 50,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        let new_user = MockUserDistributionTracking {
            user_id: "new_user".to_string(),
            opportunities_received_today: 0,
            last_opportunity_at: None,
            activity_boost_multiplier: 1.0,
            fairness_score: 2.0, // New users get fairness boost
            consecutive_days_active: 0,
            total_opportunities_received: 0,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Create test opportunities
        let high_confidence_opp = MockOpportunity {
            opportunity_id: "high_conf".to_string(),
            opportunity_type: OpportunityType::Arbitrage,
            risk_level: RiskLevel::Medium,
            confidence_score: 90.0,
            expected_return: 3.0,
            trading_pair: "BTC/USDT".to_string(),
            exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + 1800000,
            priority_score: 0.0,
            user_eligibility: Vec::new(),
        };

        let low_confidence_opp = MockOpportunity {
            opportunity_id: "low_conf".to_string(),
            opportunity_type: OpportunityType::Technical,
            risk_level: RiskLevel::Low,
            confidence_score: 60.0,
            expected_return: 1.0,
            trading_pair: "ETH/USDT".to_string(),
            exchanges: vec![ExchangeIdEnum::Binance],
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            expires_at: chrono::Utc::now().timestamp_millis() as u64 + 1800000,
            priority_score: 0.0,
            user_eligibility: Vec::new(),
        };

        // Calculate priority scores
        let active_user_high_conf_score =
            service.mock_calculate_priority_score(&high_confidence_opp, &high_activity_user);
        let active_user_low_conf_score =
            service.mock_calculate_priority_score(&low_confidence_opp, &high_activity_user);
        let new_user_high_conf_score =
            service.mock_calculate_priority_score(&high_confidence_opp, &new_user);
        let new_user_low_conf_score =
            service.mock_calculate_priority_score(&low_confidence_opp, &new_user);

        // High confidence opportunities should score higher than low confidence
        assert!(active_user_high_conf_score > active_user_low_conf_score);
        assert!(new_user_high_conf_score > new_user_low_conf_score);

        // New users should get fairness boost, but active users get activity boost
        // The exact comparison depends on the specific multipliers
        assert!(active_user_high_conf_score > 0.0);
        assert!(new_user_high_conf_score > 0.0);
    }

    #[tokio::test]
    async fn test_activity_boost_calculation() {
        let service = MockGlobalOpportunityService::new();

        // Test new user (no activity boost)
        let new_user_tracking = MockUserDistributionTracking {
            user_id: "new_user".to_string(),
            opportunities_received_today: 0,
            last_opportunity_at: None,
            activity_boost_multiplier: 1.0,
            fairness_score: 1.0,
            consecutive_days_active: 0,
            total_opportunities_received: 0,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Test moderately active user
        let moderate_user_tracking = MockUserDistributionTracking {
            user_id: "moderate_user".to_string(),
            opportunities_received_today: 3,
            last_opportunity_at: Some(chrono::Utc::now().timestamp_millis() as u64 - 1800000),
            activity_boost_multiplier: 1.0,
            fairness_score: 1.0,
            consecutive_days_active: 5,
            total_opportunities_received: 25,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Test highly active user
        let active_user_tracking = MockUserDistributionTracking {
            user_id: "active_user".to_string(),
            opportunities_received_today: 8,
            last_opportunity_at: Some(chrono::Utc::now().timestamp_millis() as u64 - 900000),
            activity_boost_multiplier: 1.0,
            fairness_score: 1.0,
            consecutive_days_active: 15,
            total_opportunities_received: 150,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Calculate activity boosts
        let new_user_boost = service.mock_calculate_activity_boost(&new_user_tracking);
        let moderate_user_boost = service.mock_calculate_activity_boost(&moderate_user_tracking);
        let active_user_boost = service.mock_calculate_activity_boost(&active_user_tracking);

        // Activity boost should increase with consecutive days
        assert_eq!(
            new_user_boost,
            service.activity_boost_config.base_activity_multiplier
        );
        assert!(moderate_user_boost > new_user_boost);
        assert!(active_user_boost > moderate_user_boost);

        // Active user should get weekly bonus
        assert!(
            active_user_boost
                >= service.activity_boost_config.base_activity_multiplier
                    + service.activity_boost_config.weekly_activity_bonus
        );

        // Boost should not exceed maximum
        assert!(active_user_boost <= service.activity_boost_config.max_activity_boost);
    }

    #[tokio::test]
    async fn test_fairness_score_calculation() {
        let service = MockGlobalOpportunityService::new();

        // Test new user (should get maximum fairness boost)
        let new_user_tracking = MockUserDistributionTracking {
            user_id: "new_user".to_string(),
            opportunities_received_today: 0,
            last_opportunity_at: None,
            activity_boost_multiplier: 1.0,
            fairness_score: 1.0,
            consecutive_days_active: 0,
            total_opportunities_received: 0,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Test user who received opportunity recently
        let recent_user_tracking = MockUserDistributionTracking {
            user_id: "recent_user".to_string(),
            opportunities_received_today: 2,
            last_opportunity_at: Some(chrono::Utc::now().timestamp_millis() as u64 - 1800000), // 30 minutes ago
            activity_boost_multiplier: 1.0,
            fairness_score: 1.0,
            consecutive_days_active: 3,
            total_opportunities_received: 15,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Test user who hasn't received opportunity in a while
        let waiting_user_tracking = MockUserDistributionTracking {
            user_id: "waiting_user".to_string(),
            opportunities_received_today: 1,
            last_opportunity_at: Some(chrono::Utc::now().timestamp_millis() as u64 - 7200000), // 2 hours ago
            activity_boost_multiplier: 1.0,
            fairness_score: 1.0,
            consecutive_days_active: 5,
            total_opportunities_received: 30,
            last_reset_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        // Calculate fairness scores
        let new_user_fairness = service.mock_calculate_fairness_score(&new_user_tracking);
        let recent_user_fairness = service.mock_calculate_fairness_score(&recent_user_tracking);
        let waiting_user_fairness = service.mock_calculate_fairness_score(&waiting_user_tracking);

        // New users should get maximum fairness boost
        assert_eq!(
            new_user_fairness,
            service.fairness_config.max_fairness_boost
        );

        // Users who haven't received opportunities recently should get higher fairness scores
        assert!(waiting_user_fairness > recent_user_fairness);

        // All scores should be within valid range
        assert!(new_user_fairness <= service.fairness_config.max_fairness_boost);
        assert!(recent_user_fairness >= service.fairness_config.base_fairness_score);
        assert!(waiting_user_fairness <= service.fairness_config.max_fairness_boost);
    }

    #[tokio::test]
    async fn test_opportunity_queue_management() {
        let mut service = MockGlobalOpportunityService::new();

        // Create multiple opportunities
        let opp1 = service
            .mock_generate_arbitrage_opportunity(
                "BTC/USDT",
                vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            )
            .await
            .unwrap();
        let opp2 = service
            .mock_generate_technical_opportunity("ETH/USDT", ExchangeIdEnum::Binance)
            .await
            .unwrap();
        let opp3 = service
            .mock_generate_arbitrage_opportunity(
                "ADA/USDT",
                vec![ExchangeIdEnum::Bybit, ExchangeIdEnum::OKX],
            )
            .await
            .unwrap();

        // Add opportunities to queue
        service.d1_service.add_mock_opportunity(opp1);
        service.d1_service.add_mock_opportunity(opp2);
        service.d1_service.add_mock_opportunity(opp3);

        assert_eq!(service.d1_service.get_opportunity_count(), 3);

        // Test queue size limit (config max_queue_size = 100)
        assert!(service.d1_service.get_opportunity_count() <= service.config.max_queue_size);

        // Test opportunity expiry cleanup
        // Create an expired opportunity
        let mut expired_opp = service
            .mock_generate_technical_opportunity("EXPIRED/USDT", ExchangeIdEnum::Binance)
            .await
            .unwrap();
        expired_opp.expires_at = chrono::Utc::now().timestamp_millis() as u64 - 1000; // Expired 1 second ago
        service.d1_service.add_mock_opportunity(expired_opp);

        assert_eq!(service.d1_service.get_opportunity_count(), 4);

        // Clean up expired opportunities
        let removed_count = service.mock_cleanup_expired_opportunities().await.unwrap();
        assert_eq!(removed_count, 1);
        assert_eq!(service.d1_service.get_opportunity_count(), 3);
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let mut service = MockGlobalOpportunityService::new();

        // Test database error simulation
        service.d1_service.simulate_error("database_error");

        let result = service.d1_service.mock_get_user_limits("test_user").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Database connection failed"));

        // Test recovery after error
        service.d1_service.reset_error_simulation();

        let recovery_result = service.d1_service.mock_get_user_limits("test_user").await;
        assert!(recovery_result.is_ok());

        // Test update failure simulation
        service.d1_service.simulate_error("update_failed");

        let update_result = service
            .d1_service
            .mock_update_user_distribution("test_user", "test_opp")
            .await;
        assert!(update_result.is_err());
        assert!(update_result
            .unwrap_err()
            .to_string()
            .contains("Failed to update distribution tracking"));

        // Test user not found scenario
        service.d1_service.reset_error_simulation();
        service.d1_service.simulate_error("user_not_found");

        let not_found_result = service
            .d1_service
            .mock_get_user_limits("nonexistent_user")
            .await;
        assert!(not_found_result.is_ok());
        assert!(not_found_result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_distribution_strategy_and_analytics() {
        let mut service = MockGlobalOpportunityService::new();

        // Set up multiple users with different profiles
        let users = vec![
            ("new_user", UserAccessLevel::FreeWithoutAPI, 0),
            ("api_user", UserAccessLevel::FreeWithAPI, 3),
            ("premium_user", UserAccessLevel::SubscriptionWithAPI, 8),
            ("active_user", UserAccessLevel::FreeWithAPI, 5),
        ];

        for (user_id, access_level, opportunities_used) in users {
            service
                .user_access_service
                .set_user_access_level(user_id, access_level.clone());
            service.d1_service.add_mock_user_limits(
                user_id,
                UserOpportunityLimits {
                    daily_global_opportunities: match &access_level {
                        UserAccessLevel::FreeWithoutAPI => 3,
                        UserAccessLevel::FreeWithAPI => 10,
                        UserAccessLevel::SubscriptionWithAPI => 50,
                        _ => 10,
                    },
                    daily_technical_opportunities: match &access_level {
                        UserAccessLevel::FreeWithoutAPI => 3,
                        UserAccessLevel::FreeWithAPI => 10,
                        UserAccessLevel::SubscriptionWithAPI => 50,
                        _ => 10,
                    },
                    daily_ai_opportunities: 5,
                    hourly_rate_limit: match &access_level {
                        UserAccessLevel::FreeWithoutAPI => 1,
                        UserAccessLevel::FreeWithAPI => 5,
                        UserAccessLevel::SubscriptionWithAPI => 20,
                        _ => 5,
                    },
                    can_receive_realtime: !matches!(access_level, UserAccessLevel::FreeWithoutAPI),
                    delay_seconds: match &access_level {
                        UserAccessLevel::FreeWithoutAPI => 600,
                        UserAccessLevel::FreeWithAPI => 120,
                        UserAccessLevel::SubscriptionWithAPI => 30,
                        _ => 300,
                    },
                    arbitrage_received_today: opportunities_used / 2,
                    technical_received_today: opportunities_used / 2,
                    current_arbitrage_count: opportunities_used / 2,
                    current_technical_count: opportunities_used / 2,
                },
            );
        }

        // Create and distribute a high-quality opportunity
        let opportunity = service
            .mock_generate_arbitrage_opportunity(
                "BTC/USDT",
                vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            )
            .await
            .unwrap();

        let eligible_users = vec![
            "new_user".to_string(),
            "api_user".to_string(),
            "premium_user".to_string(),
            "active_user".to_string(),
        ];

        let distributed_users = service
            .mock_distribute_opportunity(opportunity, eligible_users)
            .await
            .unwrap();

        // Verify distribution results
        assert!(!distributed_users.is_empty());

        // Premium user should always be included (if eligible)
        assert!(distributed_users.contains(&"premium_user".to_string()));

        // API users should be included if they haven't reached limits
        assert!(distributed_users.contains(&"api_user".to_string()));
        assert!(distributed_users.contains(&"active_user".to_string()));

        // New user might not be included due to access level restrictions
        // This depends on the opportunity risk level and eligibility logic

        // Verify opportunity was added to queue
        assert_eq!(service.d1_service.get_opportunity_count(), 1);

        // Verify users can retrieve their opportunities
        for user_id in &distributed_users {
            let user_opportunities = service.d1_service.get_opportunities_for_user(user_id);
            assert!(!user_opportunities.is_empty());
        }
    }

    #[test]
    fn test_configuration_validation() {
        let config = MockGlobalOpportunityConfig {
            max_queue_size: 100,
            opportunity_ttl_minutes: 30,
            distribution_delay_seconds: 300,
            max_opportunities_per_user_per_day: 20,
            fairness_enabled: true,
            activity_boost_enabled: true,
            priority_scoring_enabled: true,
        };

        // Validate configuration values
        assert!(config.max_queue_size > 0);
        assert!(config.opportunity_ttl_minutes > 0);
        assert!(config.distribution_delay_seconds > 0);
        assert!(config.max_opportunities_per_user_per_day > 0);

        let fairness_config = MockFairnessConfig {
            base_fairness_score: 1.0,
            max_fairness_boost: 2.0,
            fairness_decay_rate: 0.1,
            consecutive_opportunity_penalty: 0.2,
            time_since_last_opportunity_boost: 0.1,
        };

        // Validate fairness configuration
        assert!(fairness_config.base_fairness_score > 0.0);
        assert!(fairness_config.max_fairness_boost >= fairness_config.base_fairness_score);
        assert!(
            fairness_config.fairness_decay_rate > 0.0 && fairness_config.fairness_decay_rate < 1.0
        );
        assert!(fairness_config.consecutive_opportunity_penalty > 0.0);
        assert!(fairness_config.time_since_last_opportunity_boost > 0.0);

        let activity_config = MockActivityBoostConfig {
            base_activity_multiplier: 1.0,
            max_activity_boost: 3.0,
            consecutive_days_threshold: 7,
            daily_activity_boost: 0.2,
            weekly_activity_bonus: 0.5,
        };

        // Validate activity boost configuration
        assert!(activity_config.base_activity_multiplier > 0.0);
        assert!(activity_config.max_activity_boost >= activity_config.base_activity_multiplier);
        assert!(activity_config.consecutive_days_threshold > 0);
        assert!(activity_config.daily_activity_boost > 0.0);
        assert!(activity_config.weekly_activity_bonus > 0.0);
    }
}
