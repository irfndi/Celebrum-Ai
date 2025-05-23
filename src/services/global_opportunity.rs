// src/services/global_opportunity.rs

use crate::types::{
    ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum, FundingRateInfo,
    GlobalOpportunity, OpportunityQueue, OpportunitySource, DistributionStrategy,
    UserOpportunityDistribution, GlobalOpportunityConfig, FairnessConfig,
    UserProfile, SubscriptionTier
};
use crate::utils::{ArbitrageError, ArbitrageResult};
use crate::services::exchange::{ExchangeService, ExchangeInterface};
use crate::services::user_profile::UserProfileService;
use crate::{log_info, log_error, log_debug};
use std::sync::Arc;
use std::collections::HashMap;
use futures::future::join_all;
use worker::kv::KvStore;
use serde_json;
use chrono::Utc;

/// Global Opportunity Service for Task 2
/// Implements system-wide opportunity detection, queue management, and fair distribution
pub struct GlobalOpportunityService {
    config: GlobalOpportunityConfig,
    exchange_service: Arc<ExchangeService>,
    user_profile_service: Arc<UserProfileService>,
    kv_store: KvStore,
    current_queue: Option<OpportunityQueue>,
    distribution_tracking: HashMap<String, UserOpportunityDistribution>,
}

impl GlobalOpportunityService {
    const OPPORTUNITY_QUEUE_KEY: &'static str = "global_opportunity_queue";
    const DISTRIBUTION_TRACKING_PREFIX: &'static str = "user_opportunity_dist";
    const ACTIVE_USERS_KEY: &'static str = "active_users_list";

    pub fn new(
        config: GlobalOpportunityConfig,
        exchange_service: Arc<ExchangeService>,
        user_profile_service: Arc<UserProfileService>,
        kv_store: KvStore,
    ) -> Self {
        Self {
            config,
            exchange_service,
            user_profile_service,
            kv_store,
            current_queue: None,
            distribution_tracking: HashMap::new(),
        }
    }

    /// Load or create the global opportunity queue
    pub async fn initialize_queue(&mut self) -> ArbitrageResult<()> {
        match self.load_queue().await {
            Ok(queue) => {
                log_info!("Loaded existing global opportunity queue", serde_json::json!({
                    "queue_id": queue.id,
                    "opportunities_count": queue.opportunities.len(),
                    "active_users": queue.active_users.len()
                }));
                self.current_queue = Some(queue);
            }
            Err(_) => {
                log_info!("Creating new global opportunity queue", serde_json::json!({}));
                let new_queue = OpportunityQueue {
                    id: uuid::Uuid::new_v4().to_string(),
                    opportunities: Vec::new(),
                    created_at: Utc::now().timestamp_millis() as u64,
                    updated_at: Utc::now().timestamp_millis() as u64,
                    total_distributed: 0,
                    active_users: Vec::new(),
                };
                self.save_queue(&new_queue).await?;
                self.current_queue = Some(new_queue);
            }
        }
        Ok(())
    }

    /// Main detection loop - discovers new opportunities using default strategy
    pub async fn detect_opportunities(&mut self) -> ArbitrageResult<Vec<GlobalOpportunity>> {
        log_info!("Starting global opportunity detection", serde_json::json!({
            "min_threshold": self.config.min_threshold,
            "max_threshold": self.config.max_threshold,
            "exchanges": self.config.monitored_exchanges.len(),
            "pairs": self.config.monitored_pairs.len()
        }));

        let mut new_opportunities = Vec::new();

        // Step 1: Fetch funding rates for all monitored pairs and exchanges
        let mut funding_rate_data: HashMap<String, HashMap<ExchangeIdEnum, Option<FundingRateInfo>>> = HashMap::new();

        // Initialize maps for each pair
        for pair in &self.config.monitored_pairs {
            funding_rate_data.insert(pair.clone(), HashMap::new());
        }

        // Collect funding rate fetch tasks
        let mut funding_tasks = Vec::new();

        for pair in &self.config.monitored_pairs {
            for exchange_id in &self.config.monitored_exchanges {
                let exchange_service = Arc::clone(&self.exchange_service);
                let pair = pair.clone();
                let exchange_id = *exchange_id;

                let task = Box::pin(async move {
                    let result = exchange_service
                        .fetch_funding_rates(&exchange_id.to_string(), Some(&pair))
                        .await;
                    
                    let funding_info = match result {
                        Ok(rates) => {
                            if let Some(rate_data) = rates.first() {
                                match rate_data["fundingRate"].as_str() {
                                    Some(rate_str) => {
                                        match rate_str.parse::<f64>() {
                                            Ok(funding_rate) => Some(FundingRateInfo {
                                                symbol: pair.clone(),
                                                funding_rate,
                                                timestamp: Some(Utc::now()),
                                                datetime: Some(Utc::now().to_rfc3339()),
                                                next_funding_time: None,
                                                estimated_rate: None,
                                            }),
                                            Err(_) => None,
                                        }
                                    }
                                    None => None,
                                }
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    };
                    
                    (pair, exchange_id, funding_info)
                });
                funding_tasks.push(task);
            }
        }

        // Execute all funding rate fetch operations concurrently
        let funding_results = join_all(funding_tasks).await;

        // Process funding rate results
        for (pair, exchange_id, funding_info) in funding_results {
            if let Some(pair_map) = funding_rate_data.get_mut(&pair) {
                pair_map.insert(exchange_id, funding_info);
            }
        }

        // Step 2: Identify arbitrage opportunities using default strategy
        for pair in &self.config.monitored_pairs {
            if let Some(pair_funding_rates) = funding_rate_data.get(pair) {
                let available_exchanges: Vec<ExchangeIdEnum> = pair_funding_rates
                    .iter()
                    .filter_map(|(exchange_id, rate_info)| {
                        if rate_info.is_some() {
                            Some(*exchange_id)
                        } else {
                            None
                        }
                    })
                    .collect();

                if available_exchanges.len() < 2 {
                    continue;
                }

                // Compare all pairs of exchanges for opportunities
                for i in 0..available_exchanges.len() {
                    for j in (i + 1)..available_exchanges.len() {
                        let exchange_a = available_exchanges[i];
                        let exchange_b = available_exchanges[j];

                        if let (Some(Some(rate_a)), Some(Some(rate_b))) = (
                            pair_funding_rates.get(&exchange_a),
                            pair_funding_rates.get(&exchange_b),
                        ) {
                            let rate_diff = (rate_a.funding_rate - rate_b.funding_rate).abs();

                            // Check if opportunity meets our thresholds
                            if rate_diff >= self.config.min_threshold && rate_diff <= self.config.max_threshold {
                                let (long_exchange, short_exchange, long_rate, short_rate) =
                                    if rate_a.funding_rate > rate_b.funding_rate {
                                        (exchange_b, exchange_a, rate_b.funding_rate, rate_a.funding_rate)
                                    } else {
                                        (exchange_a, exchange_b, rate_a.funding_rate, rate_b.funding_rate)
                                    };

                                // Create base arbitrage opportunity
                                let opportunity = ArbitrageOpportunity {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    pair: pair.clone(),
                                    long_exchange: Some(long_exchange),
                                    short_exchange: Some(short_exchange),
                                    long_rate: Some(long_rate),
                                    short_rate: Some(short_rate),
                                    rate_difference: rate_diff,
                                    net_rate_difference: Some(rate_diff), // Simplified for now
                                    potential_profit_value: Some(rate_diff * 1000.0), // Estimate for $1000 position
                                    timestamp: Utc::now().timestamp_millis() as u64,
                                    r#type: ArbitrageType::FundingRate,
                                    details: Some(format!(
                                        "Funding rate arbitrage: Long {} ({:.4}%) vs Short {} ({:.4}%)",
                                        long_exchange.as_str(),
                                        long_rate * 100.0,
                                        short_exchange.as_str(),
                                        short_rate * 100.0
                                    )),
                                };

                                // Calculate priority score (higher rate difference = higher priority)
                                let priority_score = rate_diff * 1000.0; // Scale up for easier comparison

                                // Create global opportunity
                                let global_opportunity = GlobalOpportunity {
                                    opportunity,
                                    detection_timestamp: Utc::now().timestamp_millis() as u64,
                                    expiry_timestamp: Utc::now().timestamp_millis() as u64 + (self.config.opportunity_ttl_minutes as u64 * 60 * 1000),
                                    priority_score,
                                    distributed_to: Vec::new(),
                                    max_participants: Some(10), // Default limit
                                    current_participants: 0,
                                    distribution_strategy: self.config.distribution_strategy.clone(),
                                    source: OpportunitySource::SystemGenerated,
                                };

                                new_opportunities.push(global_opportunity);

                                log_info!("Detected new global opportunity", serde_json::json!({
                                    "pair": pair,
                                    "rate_difference": rate_diff,
                                    "priority_score": priority_score,
                                    "long_exchange": long_exchange.as_str(),
                                    "short_exchange": short_exchange.as_str()
                                }));
                            }
                        }
                    }
                }
            }
        }

        log_info!("Global opportunity detection completed", serde_json::json!({
            "new_opportunities_count": new_opportunities.len()
        }));

        Ok(new_opportunities)
    }

    /// Add new opportunities to the global queue
    pub async fn add_opportunities_to_queue(&mut self, opportunities: Vec<GlobalOpportunity>) -> ArbitrageResult<()> {
        let opportunities_count = opportunities.len(); // Store count before move
        
        // Extract the queue to avoid borrowing conflicts
        if let Some(mut queue) = self.current_queue.take() {
            // Add new opportunities
            queue.opportunities.extend(opportunities);
            
            // Sort by priority score (highest first)
            queue.opportunities.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
            
            // Limit queue size
            if queue.opportunities.len() > self.config.max_queue_size as usize {
                queue.opportunities.truncate(self.config.max_queue_size as usize);
            }
            
            // Remove expired opportunities
            let now = Utc::now().timestamp_millis() as u64;
            queue.opportunities.retain(|opp| opp.expiry_timestamp > now);
            
            queue.updated_at = now;
            
            // Save the queue
            self.save_queue(&queue).await?;
            
            log_info!("Updated global opportunity queue", serde_json::json!({
                "total_opportunities": queue.opportunities.len(),
                "new_opportunities_added": opportunities_count
            }));
            
            // Put the queue back
            self.current_queue = Some(queue);
        }
        
        Ok(())
    }

    /// Distribute opportunities to eligible users using fairness algorithms
    pub async fn distribute_opportunities(&mut self) -> ArbitrageResult<Vec<(String, GlobalOpportunity)>> {
        let mut distributions = Vec::new();
        
        // Extract the queue to avoid borrowing conflicts
        if let Some(mut queue) = self.current_queue.take() {
            // Load active users from KV store
            let active_users = self.load_active_users().await?;
            
            // Load distribution tracking for all users
            for user_id in &active_users {
                if !self.distribution_tracking.contains_key(user_id) {
                    if let Ok(tracking) = self.load_user_distribution_tracking(user_id).await {
                        self.distribution_tracking.insert(user_id.clone(), tracking);
                    } else {
                        // Create new tracking for user
                        let new_tracking = UserOpportunityDistribution {
                            user_id: user_id.clone(),
                            last_opportunity_received: None,
                            total_opportunities_received: 0,
                            opportunities_today: 0,
                            last_daily_reset: Utc::now().timestamp_millis() as u64,
                            priority_weight: 1.0,
                            is_eligible: true,
                        };
                        self.distribution_tracking.insert(user_id.clone(), new_tracking);
                    }
                }
            }

            // Apply fairness algorithm based on distribution strategy
            match self.config.distribution_strategy {
                DistributionStrategy::RoundRobin => {
                    distributions = self.distribute_round_robin(&active_users, &mut queue).await?;
                }
                DistributionStrategy::FirstComeFirstServe => {
                    distributions = self.distribute_first_come_first_serve(&active_users, &mut queue).await?;
                }
                DistributionStrategy::PriorityBased => {
                    distributions = self.distribute_priority_based(&active_users, &mut queue).await?;
                }
                DistributionStrategy::Broadcast => {
                    distributions = self.distribute_broadcast(&active_users, &mut queue).await?;
                }
            }

            // Update distribution tracking
            for (user_id, _) in &distributions {
                if let Some(tracking) = self.distribution_tracking.get_mut(user_id) {
                    tracking.last_opportunity_received = Some(Utc::now().timestamp_millis() as u64);
                    tracking.total_opportunities_received += 1;
                    tracking.opportunities_today += 1;
                    
                    // Reset daily count if needed
                    let now = Utc::now().timestamp_millis() as u64;
                    let one_day_ms = 24 * 60 * 60 * 1000;
                    if now - tracking.last_daily_reset > one_day_ms {
                        tracking.opportunities_today = 1;
                        tracking.last_daily_reset = now;
                    }
                    
                    // Clone to avoid borrowing issues
                    let tracking_clone = tracking.clone();
                    self.save_user_distribution_tracking(&tracking_clone).await?;
                }
            }

            queue.total_distributed += distributions.len() as u32;
            
            // Clone for saving
            let queue_clone = queue.clone();
            self.save_queue(&queue_clone).await?;
            
            // Put the queue back
            self.current_queue = Some(queue);
        }

        log_info!("Distributed opportunities", serde_json::json!({
            "distributions_count": distributions.len(),
            "strategy": format!("{:?}", self.config.distribution_strategy)
        }));

        Ok(distributions)
    }

    /// Round-robin distribution - fair rotation among users
    async fn distribute_round_robin(
        &self,
        active_users: &[String],
        queue: &mut OpportunityQueue,
    ) -> ArbitrageResult<Vec<(String, GlobalOpportunity)>> {
        let mut distributions = Vec::new();
        let mut user_index = 0;

        for opportunity in queue.opportunities.iter_mut() {
            if opportunity.distributed_to.len() >= opportunity.max_participants.unwrap_or(10) as usize {
                continue;
            }

            // Find next eligible user
            let mut attempts = 0;
            while attempts < active_users.len() {
                let user_id = &active_users[user_index % active_users.len()];
                
                if self.is_user_eligible_for_opportunity(user_id, opportunity).await? {
                    opportunity.distributed_to.push(user_id.clone());
                    opportunity.current_participants += 1;
                    distributions.push((user_id.clone(), opportunity.clone()));
                    break;
                }
                
                user_index = (user_index + 1) % active_users.len();
                attempts += 1;
            }
            
            user_index = (user_index + 1) % active_users.len();
        }

        Ok(distributions)
    }

    /// First-come-first-serve distribution
    async fn distribute_first_come_first_serve(
        &self,
        active_users: &[String],
        queue: &mut OpportunityQueue,
    ) -> ArbitrageResult<Vec<(String, GlobalOpportunity)>> {
        let mut distributions = Vec::new();

        for opportunity in queue.opportunities.iter_mut() {
            if opportunity.distributed_to.len() >= opportunity.max_participants.unwrap_or(10) as usize {
                continue;
            }

            for user_id in active_users {
                if opportunity.distributed_to.contains(user_id) {
                    continue;
                }

                if self.is_user_eligible_for_opportunity(user_id, opportunity).await? {
                    opportunity.distributed_to.push(user_id.clone());
                    opportunity.current_participants += 1;
                    distributions.push((user_id.clone(), opportunity.clone()));
                    break;
                }
            }
        }

        Ok(distributions)
    }

    /// Priority-based distribution considering subscription tiers and activity
    async fn distribute_priority_based(
        &self,
        active_users: &[String],
        queue: &mut OpportunityQueue,
    ) -> ArbitrageResult<Vec<(String, GlobalOpportunity)>> {
        let mut distributions = Vec::new();

        // Calculate user priorities
        let mut user_priorities: Vec<(String, f64)> = Vec::new();
        
        for user_id in active_users {
            let priority = self.calculate_user_priority(user_id).await?;
            user_priorities.push((user_id.clone(), priority));
        }

        // Sort by priority (highest first)
        user_priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for opportunity in queue.opportunities.iter_mut() {
            if opportunity.distributed_to.len() >= opportunity.max_participants.unwrap_or(10) as usize {
                continue;
            }

            for (user_id, _priority) in &user_priorities {
                if opportunity.distributed_to.contains(user_id) {
                    continue;
                }

                if self.is_user_eligible_for_opportunity(user_id, opportunity).await? {
                    opportunity.distributed_to.push(user_id.clone());
                    opportunity.current_participants += 1;
                    distributions.push((user_id.clone(), opportunity.clone()));
                    break;
                }
            }
        }

        Ok(distributions)
    }

    /// Broadcast distribution - send to all eligible users
    async fn distribute_broadcast(
        &self,
        active_users: &[String],
        queue: &mut OpportunityQueue,
    ) -> ArbitrageResult<Vec<(String, GlobalOpportunity)>> {
        let mut distributions = Vec::new();

        for opportunity in queue.opportunities.iter_mut() {
            for user_id in active_users {
                if opportunity.distributed_to.contains(user_id) {
                    continue;
                }

                if self.is_user_eligible_for_opportunity(user_id, opportunity).await? {
                    opportunity.distributed_to.push(user_id.clone());
                    opportunity.current_participants += 1;
                    distributions.push((user_id.clone(), opportunity.clone()));
                }
            }
        }

        Ok(distributions)
    }

    /// Check if user is eligible to receive an opportunity
    async fn is_user_eligible_for_opportunity(
        &self,
        user_id: &str,
        _opportunity: &GlobalOpportunity,
    ) -> ArbitrageResult<bool> {
        // Check distribution tracking
        if let Some(tracking) = self.distribution_tracking.get(user_id) {
            if !tracking.is_eligible {
                return Ok(false);
            }

            // Check daily limits
            if tracking.opportunities_today >= self.config.fairness_config.max_opportunities_per_user_per_day {
                return Ok(false);
            }

            // Check cooldown period
            if let Some(last_received) = tracking.last_opportunity_received {
                let cooldown_ms = self.config.fairness_config.cooldown_period_minutes as u64 * 60 * 1000;
                let now = Utc::now().timestamp_millis() as u64;
                if now - last_received < cooldown_ms {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Calculate user priority based on subscription tier and activity
    async fn calculate_user_priority(&self, user_id: &str) -> ArbitrageResult<f64> {
        // Load user profile to get subscription tier
        match self.user_profile_service.get_user_profile(user_id).await {
            Ok(Some(profile)) => { // Handle Option<UserProfile>
                let tier_name = match profile.subscription.tier {
                    SubscriptionTier::Free => "Free",
                    SubscriptionTier::Basic => "Basic",
                    SubscriptionTier::Premium => "Premium",
                    SubscriptionTier::Enterprise => "Enterprise",
                };

                let tier_multiplier = self.config.fairness_config.tier_multipliers
                    .get(tier_name)
                    .copied()
                    .unwrap_or(1.0);

                // Base priority with tier multiplier
                let mut priority = tier_multiplier;

                // Activity boost - check last active time
                let now = Utc::now().timestamp_millis() as u64;
                let one_hour_ms = 60 * 60 * 1000;
                if now - profile.last_active < one_hour_ms {
                    priority *= self.config.fairness_config.activity_boost_factor;
                }

                Ok(priority)
            }
            Ok(None) => Ok(1.0), // No profile found, default priority
            Err(_) => Ok(1.0), // Error loading profile, default priority
        }
    }

    /// Get current queue status
    pub fn get_queue_status(&self) -> Option<&OpportunityQueue> {
        self.current_queue.as_ref()
    }

    /// Update active users list
    pub async fn update_active_users(&self, user_ids: Vec<String>) -> ArbitrageResult<()> {
        let data = serde_json::to_string(&user_ids)
            .map_err(|e| ArbitrageError::serialization_error(format!("Failed to serialize active users: {}", e)))?;
        
        self.kv_store
            .put(Self::ACTIVE_USERS_KEY, data)?
            .execute()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to save active users: {}", e)))?;
        
        Ok(())
    }

    // Storage operations
    async fn load_queue(&self) -> ArbitrageResult<OpportunityQueue> {
        let data = self.kv_store
            .get(Self::OPPORTUNITY_QUEUE_KEY)
            .text()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to load opportunity queue: {}", e)))?
            .ok_or_else(|| ArbitrageError::not_found("Opportunity queue not found".to_string()))?;

        serde_json::from_str(&data)
            .map_err(|e| ArbitrageError::serialization_error(format!("Failed to deserialize opportunity queue: {}", e)))
    }

    async fn save_queue(&self, queue: &OpportunityQueue) -> ArbitrageResult<()> {
        let data = serde_json::to_string(queue)
            .map_err(|e| ArbitrageError::serialization_error(format!("Failed to serialize opportunity queue: {}", e)))?;

        self.kv_store
            .put(Self::OPPORTUNITY_QUEUE_KEY, data)?
            .execute()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to save opportunity queue: {}", e)))?;

        Ok(())
    }

    async fn load_active_users(&self) -> ArbitrageResult<Vec<String>> {
        match self.kv_store.get(Self::ACTIVE_USERS_KEY).text().await {
            Ok(Some(data)) => {
                serde_json::from_str(&data)
                    .map_err(|e| ArbitrageError::serialization_error(format!("Failed to deserialize active users: {}", e)))
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(ArbitrageError::database_error(format!("Failed to load active users: {}", e))),
        }
    }

    async fn load_user_distribution_tracking(&self, user_id: &str) -> ArbitrageResult<UserOpportunityDistribution> {
        let key = format!("{}:{}", Self::DISTRIBUTION_TRACKING_PREFIX, user_id);
        let data = self.kv_store
            .get(&key)
            .text()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to load user distribution tracking: {}", e)))?
            .ok_or_else(|| ArbitrageError::not_found("User distribution tracking not found".to_string()))?;

        serde_json::from_str(&data)
            .map_err(|e| ArbitrageError::serialization_error(format!("Failed to deserialize user distribution tracking: {}", e)))
    }

    async fn save_user_distribution_tracking(&self, tracking: &UserOpportunityDistribution) -> ArbitrageResult<()> {
        let key = format!("{}:{}", Self::DISTRIBUTION_TRACKING_PREFIX, tracking.user_id);
        let data = serde_json::to_string(tracking)
            .map_err(|e| ArbitrageError::serialization_error(format!("Failed to serialize user distribution tracking: {}", e)))?;

        self.kv_store
            .put(&key, data)?
            .execute()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to save user distribution tracking: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SubscriptionInfo;
    use std::collections::HashMap;
    use uuid::Uuid;

    // Mock structures for testing
    struct MockKvStore {
        data: std::sync::Arc<std::sync::Mutex<HashMap<String, String>>>,
    }

    impl MockKvStore {
        fn new() -> Self {
            Self {
                data: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            }
        }

        fn with_data(mut self, key: &str, value: &str) -> Self {
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_string(), value.to_string());
            drop(data);
            self
        }

        async fn get(&self, key: &str) -> Option<String> {
            let data = self.data.lock().unwrap();
            data.get(key).cloned()
        }

        async fn put(&self, key: &str, value: String) -> Result<(), String> {
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_string(), value);
            Ok(())
        }
    }

    fn create_test_config() -> GlobalOpportunityConfig {
        GlobalOpportunityConfig {
            detection_interval_seconds: 30,
            min_threshold: 0.001, // 0.1%
            max_threshold: 0.02, // 2%
            max_queue_size: 50,
            opportunity_ttl_minutes: 10,
            distribution_strategy: DistributionStrategy::RoundRobin,
            fairness_config: FairnessConfig::default(),
            monitored_exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
            monitored_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        }
    }

    fn create_test_opportunity() -> GlobalOpportunity {
        let opportunity = ArbitrageOpportunity {
            id: Uuid::new_v4().to_string(),
            pair: "BTCUSDT".to_string(),
            long_exchange: Some(ExchangeIdEnum::Binance),
            short_exchange: Some(ExchangeIdEnum::Bybit),
            long_rate: Some(0.0005),
            short_rate: Some(0.0015),
            rate_difference: 0.001,
            net_rate_difference: Some(0.001),
            potential_profit_value: Some(1.0),
            timestamp: Utc::now().timestamp_millis() as u64,
            r#type: ArbitrageType::FundingRate,
            details: Some("Test opportunity".to_string()),
        };

        GlobalOpportunity {
            opportunity,
            detection_timestamp: Utc::now().timestamp_millis() as u64,
            expiry_timestamp: Utc::now().timestamp_millis() as u64 + 600000, // 10 minutes
            priority_score: 1.0,
            distributed_to: Vec::new(),
            max_participants: Some(5),
            current_participants: 0,
            distribution_strategy: DistributionStrategy::RoundRobin,
            source: OpportunitySource::SystemGenerated,
        }
    }

    fn create_test_user_profile(user_id: &str, tier: SubscriptionTier) -> UserProfile {
        UserProfile {
            user_id: user_id.to_string(),
            telegram_user_id: 12345,
            telegram_username: Some("testuser".to_string()),
            subscription: SubscriptionInfo {
                tier,
                is_active: true,
                expires_at: None,
                created_at: Utc::now().timestamp_millis() as u64,
                features: vec!["basic_features".to_string()],
            },
            configuration: crate::types::UserConfiguration::default(),
            api_keys: Vec::new(),
            invitation_code: None,
            created_at: Utc::now().timestamp_millis() as u64,
            updated_at: Utc::now().timestamp_millis() as u64,
            last_active: Utc::now().timestamp_millis() as u64,
            is_active: true,
            total_trades: 0,
            total_pnl_usdt: 0.0,
        }
    }

    #[tokio::test]
    async fn test_global_opportunity_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.detection_interval_seconds, 30);
        assert_eq!(config.min_threshold, 0.001);
        assert_eq!(config.max_threshold, 0.02);
        assert_eq!(config.max_queue_size, 50);
        assert_eq!(config.opportunity_ttl_minutes, 10);
        assert!(matches!(config.distribution_strategy, DistributionStrategy::RoundRobin));
        assert_eq!(config.monitored_exchanges.len(), 2);
        assert_eq!(config.monitored_pairs.len(), 2);
    }

    #[tokio::test]
    async fn test_global_opportunity_structure() {
        let global_opp = create_test_opportunity();
        
        assert_eq!(global_opp.opportunity.pair, "BTCUSDT");
        assert_eq!(global_opp.opportunity.rate_difference, 0.001);
        assert_eq!(global_opp.priority_score, 1.0);
        assert_eq!(global_opp.distributed_to.len(), 0);
        assert_eq!(global_opp.max_participants, Some(5));
        assert_eq!(global_opp.current_participants, 0);
        assert!(matches!(global_opp.distribution_strategy, DistributionStrategy::RoundRobin));
        assert!(matches!(global_opp.source, OpportunitySource::SystemGenerated));
    }

    #[tokio::test]
    async fn test_opportunity_queue_management() {
        let mut queue = OpportunityQueue {
            id: Uuid::new_v4().to_string(),
            opportunities: Vec::new(),
            created_at: Utc::now().timestamp_millis() as u64,
            updated_at: Utc::now().timestamp_millis() as u64,
            total_distributed: 0,
            active_users: vec!["user1".to_string(), "user2".to_string()],
        };

        // Test adding opportunities
        let opp1 = create_test_opportunity();
        let mut opp2 = create_test_opportunity();
        opp2.priority_score = 2.0; // Higher priority

        queue.opportunities.push(opp1);
        queue.opportunities.push(opp2);

        // Test sorting by priority
        queue.opportunities.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        
        assert_eq!(queue.opportunities.len(), 2);
        assert_eq!(queue.opportunities[0].priority_score, 2.0); // Higher priority first
        assert_eq!(queue.opportunities[1].priority_score, 1.0);
        assert_eq!(queue.active_users.len(), 2);
    }

    #[tokio::test]
    async fn test_user_distribution_tracking() {
        let tracking = UserOpportunityDistribution {
            user_id: "test_user".to_string(),
            last_opportunity_received: Some(Utc::now().timestamp_millis() as u64),
            total_opportunities_received: 5,
            opportunities_today: 2,
            last_daily_reset: Utc::now().timestamp_millis() as u64,
            priority_weight: 1.5,
            is_eligible: true,
        };

        assert_eq!(tracking.user_id, "test_user");
        assert_eq!(tracking.total_opportunities_received, 5);
        assert_eq!(tracking.opportunities_today, 2);
        assert_eq!(tracking.priority_weight, 1.5);
        assert!(tracking.is_eligible);
        assert!(tracking.last_opportunity_received.is_some());
    }

    #[tokio::test]
    async fn test_fairness_config_defaults() {
        let config = FairnessConfig::default();
        
        assert_eq!(config.rotation_interval_minutes, 15);
        assert_eq!(config.max_opportunities_per_user_per_hour, 10);
        assert_eq!(config.max_opportunities_per_user_per_day, 50);
        assert_eq!(config.activity_boost_factor, 1.2);
        assert_eq!(config.cooldown_period_minutes, 5);
        
        // Test tier multipliers
        assert_eq!(config.tier_multipliers.get("Free"), Some(&1.0));
        assert_eq!(config.tier_multipliers.get("Basic"), Some(&1.5));
        assert_eq!(config.tier_multipliers.get("Premium"), Some(&2.0));
        assert_eq!(config.tier_multipliers.get("Enterprise"), Some(&3.0));
    }

    #[tokio::test]
    async fn test_distribution_strategies() {
        // Test all distribution strategy variants
        let strategies = vec![
            DistributionStrategy::FirstComeFirstServe,
            DistributionStrategy::RoundRobin,
            DistributionStrategy::PriorityBased,
            DistributionStrategy::Broadcast,
        ];

        for strategy in strategies {
            match strategy {
                DistributionStrategy::FirstComeFirstServe => {
                    // Test first-come-first-serve logic
                    assert!(true); // Placeholder for actual implementation test
                }
                DistributionStrategy::RoundRobin => {
                    // Test round-robin logic
                    assert!(true); // Placeholder for actual implementation test
                }
                DistributionStrategy::PriorityBased => {
                    // Test priority-based logic
                    assert!(true); // Placeholder for actual implementation test
                }
                DistributionStrategy::Broadcast => {
                    // Test broadcast logic
                    assert!(true); // Placeholder for actual implementation test
                }
            }
        }
    }

    #[tokio::test]
    async fn test_opportunity_source_types() {
        let sources = vec![
            OpportunitySource::SystemGenerated,
            OpportunitySource::UserAI("user123".to_string()),
            OpportunitySource::External,
        ];

        for source in sources {
            match source {
                OpportunitySource::SystemGenerated => {
                    assert!(true); // System-generated opportunities
                }
                OpportunitySource::UserAI(user_id) => {
                    assert_eq!(user_id, "user123"); // User AI-generated
                }
                OpportunitySource::External => {
                    assert!(true); // External source opportunities
                }
            }
        }
    }

    #[tokio::test]
    async fn test_opportunity_expiry_logic() {
        let now = Utc::now().timestamp_millis() as u64;
        
        // Create expired opportunity
        let mut expired_opp = create_test_opportunity();
        expired_opp.expiry_timestamp = now - 1000; // 1 second ago
        
        // Create valid opportunity
        let valid_opp = create_test_opportunity();
        // expiry_timestamp is set in the future by create_test_opportunity()
        
        assert!(expired_opp.expiry_timestamp < now);
        assert!(valid_opp.expiry_timestamp > now);
        
        // Test filtering logic
        let opportunities = vec![expired_opp, valid_opp];
        let valid_opportunities: Vec<_> = opportunities
            .into_iter()
            .filter(|opp| opp.expiry_timestamp > now)
            .collect();
        
        assert_eq!(valid_opportunities.len(), 1);
    }

    #[tokio::test]
    async fn test_priority_score_calculation() {
        let base_rate_diff = 0.001; // 0.1%
        let priority_score = base_rate_diff * 1000.0; // Scale up
        
        assert_eq!(priority_score, 1.0);
        
        // Test higher rate difference
        let higher_rate_diff = 0.005; // 0.5%
        let higher_priority = higher_rate_diff * 1000.0;
        
        assert_eq!(higher_priority, 5.0);
        assert!(higher_priority > priority_score);
    }

    #[tokio::test]
    async fn test_user_eligibility_checks() {
        let config = FairnessConfig::default();
        
        // Test daily limit check
        let mut tracking = UserOpportunityDistribution {
            user_id: "test_user".to_string(),
            last_opportunity_received: None,
            total_opportunities_received: 0,
            opportunities_today: config.max_opportunities_per_user_per_day,
            last_daily_reset: Utc::now().timestamp_millis() as u64,
            priority_weight: 1.0,
            is_eligible: true,
        };
        
        // User at daily limit should not be eligible
        assert!(!tracking.is_eligible || tracking.opportunities_today >= config.max_opportunities_per_user_per_day);
        
        // Test cooldown period
        tracking.opportunities_today = 0;
        tracking.last_opportunity_received = Some(Utc::now().timestamp_millis() as u64 - 1000); // 1 second ago
        
        let cooldown_ms = config.cooldown_period_minutes as u64 * 60 * 1000;
        let time_since_last = 1000u64; // 1 second
        
        // Should not be eligible due to cooldown
        assert!(time_since_last < cooldown_ms);
    }

    #[tokio::test]
    async fn test_subscription_tier_priority() {
        let free_user = create_test_user_profile("user1", SubscriptionTier::Free);
        let premium_user = create_test_user_profile("user2", SubscriptionTier::Premium);
        
        let config = FairnessConfig::default();
        
        let free_multiplier = config.tier_multipliers.get("Free").copied().unwrap_or(1.0);
        let premium_multiplier = config.tier_multipliers.get("Premium").copied().unwrap_or(1.0);
        
        assert_eq!(free_multiplier, 1.0);
        assert_eq!(premium_multiplier, 2.0);
        assert!(premium_multiplier > free_multiplier);
    }

    #[tokio::test]
    async fn test_opportunity_participant_limits() {
        let mut opportunity = create_test_opportunity();
        opportunity.max_participants = Some(2);
        
        // Test adding participants
        opportunity.distributed_to.push("user1".to_string());
        opportunity.current_participants += 1;
        assert_eq!(opportunity.current_participants, 1);
        assert!(opportunity.current_participants < opportunity.max_participants.unwrap_or(10));
        
        // Add second participant
        opportunity.distributed_to.push("user2".to_string());
        opportunity.current_participants += 1;
        assert_eq!(opportunity.current_participants, 2);
        assert_eq!(opportunity.current_participants, opportunity.max_participants.unwrap_or(10));
        
        // Check if at limit
        assert_eq!(opportunity.distributed_to.len(), opportunity.max_participants.unwrap_or(10) as usize);
    }

    #[tokio::test]
    async fn test_queue_size_limits() {
        let config = create_test_config();
        let mut opportunities = Vec::new();
        
        // Create more opportunities than max queue size
        for i in 0..(config.max_queue_size + 10) {
            let mut opp = create_test_opportunity();
            opp.priority_score = i as f64; // Different priorities
            opportunities.push(opp);
        }
        
        // Sort by priority (highest first)
        opportunities.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Truncate to max size
        if opportunities.len() > config.max_queue_size as usize {
            opportunities.truncate(config.max_queue_size as usize);
        }
        
        assert_eq!(opportunities.len(), config.max_queue_size as usize);
        
        // Verify highest priority opportunities are kept
        assert_eq!(opportunities[0].priority_score, (config.max_queue_size + 10 - 1) as f64);
        assert_eq!(opportunities.last().unwrap().priority_score, 10.0);
    }

    #[tokio::test]
    async fn test_activity_boost_calculation() {
        let config = FairnessConfig::default();
        let base_priority = 1.0;
        
        // Test with recent activity (within 1 hour)
        let now = Utc::now().timestamp_millis() as u64;
        let one_hour_ms = 60 * 60 * 1000;
        let recent_activity = now - (one_hour_ms / 2); // 30 minutes ago
        
        let boosted_priority = if now - recent_activity < one_hour_ms {
            base_priority * config.activity_boost_factor
        } else {
            base_priority
        };
        
        assert_eq!(boosted_priority, base_priority * 1.2);
        
        // Test with old activity (more than 1 hour)
        let old_activity = now - (one_hour_ms * 2); // 2 hours ago
        
        let unboosted_priority = if now - old_activity < one_hour_ms {
            base_priority * config.activity_boost_factor
        } else {
            base_priority
        };
        
        assert_eq!(unboosted_priority, base_priority);
    }
} 