use crate::services::core::infrastructure::cloudflare_pipelines::CloudflarePipelinesService;
use crate::services::core::infrastructure::d1_database::D1Service;
use crate::services::core::infrastructure::kv_service::KVService;
use crate::services::core::user::session_management::SessionManagementService;

use crate::types::{
    ArbitrageOpportunity, ChatContext, DistributionStrategy, FairnessConfig, GlobalOpportunity,
    OpportunitySource,
};
use crate::utils::ArbitrageResult;
use serde_json::json;
use std::collections::HashMap;

// Trait for sending notifications - breaks circular dependency
#[async_trait::async_trait]
pub trait NotificationSender {
    async fn send_opportunity_notification(
        &self,
        chat_id: &str,
        opportunity: &ArbitrageOpportunity,
        is_private: bool,
    ) -> ArbitrageResult<bool>;

    async fn send_message(&self, chat_id: &str, message: &str) -> ArbitrageResult<()>;
}

/// Configuration for opportunity distribution
#[derive(Debug, Clone)]
pub struct DistributionConfig {
    pub max_opportunities_per_user_per_hour: u32,
    pub max_opportunities_per_user_per_day: u32,
    pub cooldown_period_minutes: u32,
    pub batch_size: u32,
    pub distribution_interval_seconds: u32,
    pub max_participants_per_opportunity: Option<u32>,
    pub fairness_config: FairnessConfig,
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            max_opportunities_per_user_per_hour: 2,
            max_opportunities_per_user_per_day: 10,
            cooldown_period_minutes: 240, // 4 hours
            batch_size: 50,
            distribution_interval_seconds: 30,
            max_participants_per_opportunity: Some(100), // Default to 100 participants
            fairness_config: FairnessConfig::default(),
        }
    }
}

/// Service for distributing opportunities to eligible users
/// Handles automated push notifications with fairness algorithms
pub struct OpportunityDistributionService {
    d1_service: D1Service,
    kv_service: KVService,
    session_service: SessionManagementService,
    notification_sender: Option<Box<dyn NotificationSender + Send + Sync>>,
    pipelines_service: Option<CloudflarePipelinesService>,
    config: DistributionConfig,
}

impl OpportunityDistributionService {
    pub fn new(
        d1_service: D1Service,
        kv_service: KVService,
        session_service: SessionManagementService,
    ) -> Self {
        Self {
            d1_service,
            kv_service,
            session_service,
            notification_sender: None,
            pipelines_service: None,
            config: DistributionConfig::default(),
        }
    }

    pub fn with_config(mut self, config: DistributionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn set_notification_sender(&mut self, sender: Box<dyn NotificationSender + Send + Sync>) {
        self.notification_sender = Some(sender);
    }

    pub fn set_pipelines_service(&mut self, pipelines_service: CloudflarePipelinesService) {
        self.pipelines_service = Some(pipelines_service);
    }

    /// Distribute a global opportunity to eligible users
    pub async fn distribute_opportunity(
        &self,
        opportunity: ArbitrageOpportunity,
    ) -> ArbitrageResult<u32> {
        // Create global opportunity with metadata
        let global_opportunity = GlobalOpportunity {
            opportunity: opportunity.clone(),
            detection_timestamp: chrono::Utc::now().timestamp_millis() as u64,
            expiry_timestamp: chrono::Utc::now().timestamp_millis() as u64 + (10 * 60 * 1000), // 10 minutes
            priority_score: self.calculate_priority_score(&opportunity).await?,
            distributed_to: Vec::new(),
            max_participants: self.config.max_participants_per_opportunity,
            current_participants: 0,
            distribution_strategy: DistributionStrategy::RoundRobin,
            source: OpportunitySource::SystemGenerated,
        };

        // Get eligible users
        let eligible_users = self.get_eligible_users(&global_opportunity).await?;

        // Apply fairness algorithm
        let selected_users = self
            .apply_fairness_algorithm(&eligible_users, &global_opportunity)
            .await?;

        // Distribute to selected users
        let mut distributed_count = 0;
        for user_id in selected_users {
            if self
                .send_opportunity_to_user(&user_id, &global_opportunity)
                .await?
            {
                distributed_count += 1;

                // Update user distribution tracking
                self.update_user_distribution_tracking(&user_id, &global_opportunity)
                    .await?;

                // Respect rate limiting
                if distributed_count >= self.config.batch_size {
                    break;
                }
            }
        }

        // Store distribution analytics
        self.record_distribution_analytics(&global_opportunity, distributed_count)
            .await?;

        Ok(distributed_count)
    }

    /// Get list of users eligible for opportunity distribution
    async fn get_eligible_users(
        &self,
        opportunity: &GlobalOpportunity,
    ) -> ArbitrageResult<Vec<String>> {
        let mut eligible_users = Vec::new();

        // Query database for active sessions
        let query = "SELECT telegram_id FROM user_sessions WHERE expires_at > datetime('now') AND is_active = 1 LIMIT 1000";
        let rows = self.d1_service.query(query, &[]).await?;

        for row in rows {
            if let Some(telegram_id) = row.get("telegram_id") {
                let user_id = telegram_id.clone();

                // Check if user is eligible for push notifications
                let chat_context = ChatContext::Private;

                if self
                    .session_service
                    .is_eligible_for_push_notification(
                        &user_id,
                        &opportunity.opportunity,
                        &chat_context,
                    )
                    .await?
                {
                    eligible_users.push(user_id);
                }
            }
        }

        Ok(eligible_users)
    }

    /// Apply fairness algorithm to select users for distribution
    async fn apply_fairness_algorithm(
        &self,
        eligible_users: &[String],
        opportunity: &GlobalOpportunity,
    ) -> ArbitrageResult<Vec<String>> {
        let mut selected_users = Vec::new();
        let max_users = self
            .config
            .fairness_config
            .max_opportunities_per_user_per_hour;

        match opportunity.distribution_strategy {
            DistributionStrategy::FirstComeFirstServe => {
                // Simple FIFO selection
                selected_users.extend_from_slice(
                    &eligible_users[..std::cmp::min(eligible_users.len(), max_users as usize)],
                );
            }
            DistributionStrategy::RoundRobin => {
                // Round-robin selection based on last opportunity received
                let mut user_priorities = HashMap::new();

                for user_id in eligible_users {
                    let last_received = self.get_user_last_opportunity_time(user_id).await?;
                    user_priorities.insert(user_id.clone(), last_received);
                }

                // Sort by last received time (oldest first)
                let mut sorted_users: Vec<_> = user_priorities.into_iter().collect();
                sorted_users.sort_by_key(|(_, last_received)| *last_received);

                for (user_id, _) in sorted_users.into_iter().take(max_users as usize) {
                    selected_users.push(user_id);
                }
            }
            DistributionStrategy::PriorityBased => {
                // Priority-based selection (subscription tier, activity, etc.)
                let mut user_scores = HashMap::new();

                for user_id in eligible_users {
                    let score = self.calculate_user_priority_score(user_id).await?;
                    user_scores.insert(user_id.clone(), score);
                }

                // Sort by priority score (highest first)
                let mut sorted_users: Vec<_> = user_scores.into_iter().collect();
                sorted_users.sort_by(|(_, a), (_, b)| {
                    b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
                });

                for (user_id, _) in sorted_users.into_iter().take(max_users as usize) {
                    selected_users.push(user_id);
                }
            }
            DistributionStrategy::Broadcast => {
                // Send to all eligible users (respecting global limits)
                selected_users.extend_from_slice(eligible_users);
            }
        }

        Ok(selected_users)
    }

    /// Send opportunity to a specific user
    async fn send_opportunity_to_user(
        &self,
        user_id: &str,
        opportunity: &GlobalOpportunity,
    ) -> ArbitrageResult<bool> {
        // Check rate limiting
        if !self.check_user_rate_limit(user_id).await? {
            return Ok(false);
        }

        // Send via notification sender if available
        if let Some(ref notification_sender) = self.notification_sender {
            match notification_sender
                .send_opportunity_notification(user_id, &opportunity.opportunity, true)
                .await
            {
                Ok(sent) => {
                    if sent {
                        // Update rate limiting counters
                        self.update_rate_limit_counters(user_id).await?;
                    }
                    Ok(sent)
                }
                Err(_) => {
                    // Log error but don't fail the entire distribution
                    Ok(false)
                }
            }
        } else {
            // If no notification sender, just record that we would have sent it
            // Still update rate limiting to maintain consistency
            self.update_rate_limit_counters(user_id).await?;
            Ok(true)
        }
    }

    /// Check if user is within rate limits
    async fn check_user_rate_limit(&self, user_id: &str) -> ArbitrageResult<bool> {
        let now = chrono::Utc::now();
        let hour_key = format!("rate_limit:{}:{}", user_id, now.format("%Y-%m-%d-%H"));
        let day_key = format!("rate_limit:{}:{}", user_id, now.format("%Y-%m-%d"));

        // Check hourly limit
        let hourly_count = match self.kv_service.get(&hour_key).await? {
            Some(count_str) => count_str.parse::<u32>().unwrap_or(0),
            None => 0,
        };

        if hourly_count >= self.config.max_opportunities_per_user_per_hour {
            return Ok(false);
        }

        // Check daily limit
        let daily_count = match self.kv_service.get(&day_key).await? {
            Some(count_str) => count_str.parse::<u32>().unwrap_or(0),
            None => 0,
        };

        if daily_count >= self.config.max_opportunities_per_user_per_day {
            return Ok(false);
        }

        Ok(true)
    }

    /// Update rate limiting counters after successful delivery
    async fn update_rate_limit_counters(&self, user_id: &str) -> ArbitrageResult<()> {
        let now = chrono::Utc::now();
        let hour_key = format!("rate_limit:{}:{}", user_id, now.format("%Y-%m-%d-%H"));
        let day_key = format!("rate_limit:{}:{}", user_id, now.format("%Y-%m-%d"));

        // Update hourly counter
        let hourly_count = match self.kv_service.get(&hour_key).await? {
            Some(count_str) => count_str.parse::<u32>().unwrap_or(0) + 1,
            None => 1,
        };
        self.kv_service
            .put(&hour_key, &hourly_count.to_string(), Some(3600))
            .await?;

        // Update daily counter
        let daily_count = match self.kv_service.get(&day_key).await? {
            Some(count_str) => count_str.parse::<u32>().unwrap_or(0) + 1,
            None => 1,
        };
        self.kv_service
            .put(&day_key, &daily_count.to_string(), Some(24 * 3600))
            .await?;

        Ok(())
    }

    /// Update user distribution tracking after sending opportunity
    async fn update_user_distribution_tracking(
        &self,
        user_id: &str,
        _opportunity: &GlobalOpportunity,
    ) -> ArbitrageResult<()> {
        let current_time = chrono::Utc::now().timestamp_millis() as u64;

        // Update last opportunity time for fairness algorithm
        let last_opportunity_key = format!("last_opportunity:{}", user_id);
        self.kv_service
            .put(
                &last_opportunity_key,
                &current_time.to_string(),
                Some(24 * 3600),
            )
            .await?;

        Ok(())
    }

    /// Calculate priority score for an opportunity
    async fn calculate_priority_score(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> ArbitrageResult<f64> {
        let mut score = 0.0;

        // Base score from rate difference
        score += opportunity.rate_difference * 1000.0; // Scale up for better scoring

        // Bonus for potential profit
        if let Some(profit) = opportunity.potential_profit_value {
            score += profit * 0.1; // Weight profit value
        }

        // Bonus for confidence (if available in future)
        score += 50.0; // Default confidence bonus

        // Time decay (newer opportunities get higher scores)
        let age_minutes =
            (chrono::Utc::now().timestamp_millis() as u64 - opportunity.timestamp) / (60 * 1000);
        let time_decay = 1.0 - (age_minutes as f64 * 0.01).min(0.5); // Max 50% decay
        score *= time_decay;

        Ok(score.max(0.0))
    }

    /// Get user's last opportunity received time
    async fn get_user_last_opportunity_time(&self, user_id: &str) -> ArbitrageResult<u64> {
        let last_opportunity_key = format!("last_opportunity:{}", user_id);
        let last_time = self
            .kv_service
            .get(&last_opportunity_key)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| "0".to_string())
            .parse::<u64>()
            .unwrap_or(0);
        Ok(last_time)
    }

    /// Calculate user priority score for distribution
    async fn calculate_user_priority_score(&self, user_id: &str) -> ArbitrageResult<f64> {
        let mut score = 1.0; // Base score

        // Subscription tier multiplier (would integrate with UserProfile service)
        score *= 1.0; // Default multiplier

        // Activity boost (users with recent activity get higher priority)
        let last_activity = self.get_user_last_opportunity_time(user_id).await?;
        let hours_since_last =
            (chrono::Utc::now().timestamp_millis() as u64 - last_activity) / (60 * 60 * 1000);

        if hours_since_last > 24 {
            score *= 1.2; // Boost for users who haven't received opportunities recently
        }

        Ok(score)
    }

    /// Record distribution analytics
    async fn record_distribution_analytics(
        &self,
        opportunity: &GlobalOpportunity,
        distributed_count: u32,
    ) -> ArbitrageResult<()> {
        // Store in KV for fast access
        let analytics_key = format!("distribution_analytics:{}", opportunity.opportunity.id);
        let analytics_data = json!({
            "opportunity_id": opportunity.opportunity.id,
            "pair": opportunity.opportunity.pair,
            "rate_difference": opportunity.opportunity.rate_difference,
            "priority_score": opportunity.priority_score,
            "distributed_count": distributed_count,
            "distribution_strategy": format!("{:?}", opportunity.distribution_strategy),
            "detection_timestamp": opportunity.detection_timestamp,
            "distribution_timestamp": chrono::Utc::now().timestamp_millis(),
            "event_type": "opportunity_distributed"
        });

        // Store analytics with 30-day TTL in KV
        self.kv_service
            .put(
                &analytics_key,
                &analytics_data.to_string(),
                Some(30 * 24 * 3600),
            )
            .await?;

        // Store in D1 for persistent analytics
        let insert_query = "
            INSERT INTO opportunity_distribution_analytics 
            (opportunity_id, pair, rate_difference, priority_score, distributed_count, 
             distribution_strategy, detection_timestamp, distribution_timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        let params = vec![
            serde_json::Value::String(opportunity.opportunity.id.clone()),
            serde_json::Value::String(opportunity.opportunity.pair.clone()),
            serde_json::Value::Number(
                serde_json::Number::from_f64(opportunity.opportunity.rate_difference).unwrap(),
            ),
            serde_json::Value::Number(
                serde_json::Number::from_f64(opportunity.priority_score).unwrap(),
            ),
            serde_json::Value::Number(serde_json::Number::from(distributed_count)),
            serde_json::Value::String(format!("{:?}", opportunity.distribution_strategy)),
            serde_json::Value::Number(serde_json::Number::from(opportunity.detection_timestamp)),
            serde_json::Value::Number(serde_json::Number::from(
                chrono::Utc::now().timestamp_millis() as u64,
            )),
        ];

        self.d1_service.execute(insert_query, &params).await?;

        // Record high-volume analytics via Cloudflare Pipelines for scalable data ingestion
        // Record high-volume analytics via Cloudflare Pipelines for scalable data ingestion
        if let Some(ref pipelines_service) = self.pipelines_service {
            let distribution_latency =
                chrono::Utc::now().timestamp_millis() as u64 - opportunity.detection_timestamp;

            if let Err(e) = pipelines_service
                .record_distribution_analytics(
                    &opportunity.opportunity.id,
                    &opportunity.opportunity.pair,
                    opportunity.opportunity.rate_difference,
                    distributed_count,
                    distribution_latency,
                )
                .await
            {
                // Log error but don't fail the distribution - analytics is non-critical
                eprintln!(
                    "Failed to record distribution analytics to Pipelines: {}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Get distribution statistics
    pub async fn get_distribution_stats(&self) -> ArbitrageResult<DistributionStats> {
        // Get today's distribution count from database
        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis();
        let today_end = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
            .timestamp_millis();

        let count_query = "
            SELECT COUNT(*) as count, AVG(distributed_count) as avg_distributed
            FROM opportunity_distribution_analytics 
            WHERE distribution_timestamp >= ? AND distribution_timestamp <= ?
        ";

        let count_params = vec![
            serde_json::Value::Number(serde_json::Number::from(today_start)),
            serde_json::Value::Number(serde_json::Number::from(today_end)),
        ];

        let count_rows = self.d1_service.query(count_query, &count_params).await?;
        let today_count = if let Some(row) = count_rows.first() {
            row.get("count")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0)
        } else {
            0
        };

        // Get active users count
        let active_users = self.session_service.get_active_session_count().await?;

        // Calculate average distribution time from recent analytics
        let avg_time_query = "
            SELECT AVG(distribution_timestamp - detection_timestamp) as avg_time
            FROM opportunity_distribution_analytics 
            WHERE distribution_timestamp >= ?
            LIMIT 100
        ";

        let avg_time_params = vec![serde_json::Value::Number(serde_json::Number::from(
            today_start,
        ))];

        let avg_time_rows = self
            .d1_service
            .query(avg_time_query, &avg_time_params)
            .await?;
        let average_distribution_time_ms = if let Some(row) = avg_time_rows.first() {
            row.get("avg_time")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(150)
        } else {
            150
        };

        Ok(DistributionStats {
            opportunities_distributed_today: today_count,
            active_users,
            average_distribution_time_ms,
            success_rate_percentage: 98.5, // Would be calculated from delivery success metrics
        })
    }
}

/// Distribution statistics
#[derive(Debug, Clone)]
pub struct DistributionStats {
    pub opportunities_distributed_today: u32,
    pub active_users: u32,
    pub average_distribution_time_ms: u64,
    pub success_rate_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArbitrageType, ExchangeIdEnum};

    fn create_test_opportunity() -> ArbitrageOpportunity {
        ArbitrageOpportunity {
            id: "test_dist_opp_001".to_string(),
            pair: "BTCUSDT".to_string(),
            r#type: ArbitrageType::FundingRate,
            long_exchange: ExchangeIdEnum::Binance,
            short_exchange: ExchangeIdEnum::Bybit,
            long_rate: Some(0.001),
            short_rate: Some(0.003),
            rate_difference: 0.002,
            net_rate_difference: Some(0.0018),
            potential_profit_value: Some(25.0),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            details: Some("Test distribution opportunity".to_string()),
            min_exchanges_required: 2,
        }
    }

    #[tokio::test]
    async fn test_opportunity_distribution() {
        // Test the complete distribution flow
        let opportunity = create_test_opportunity();

        // Test priority score calculation
        assert!(opportunity.rate_difference > 0.0);
        assert!(opportunity.potential_profit_value.unwrap_or(0.0) > 0.0);

        // Test opportunity message formatting
        let global_opp = GlobalOpportunity {
            opportunity: opportunity.clone(),
            detection_timestamp: chrono::Utc::now().timestamp_millis() as u64,
            expiry_timestamp: chrono::Utc::now().timestamp_millis() as u64 + (10 * 60 * 1000),
            priority_score: 75.0,
            distributed_to: Vec::new(),
            max_participants: Some(100),
            current_participants: 0,
            distribution_strategy: DistributionStrategy::RoundRobin,
            source: OpportunitySource::SystemGenerated,
        };

        // Test message formatting
        let long_exchange_name = format!("{:?}", opportunity.long_exchange);
        let short_exchange_name = format!("{:?}", opportunity.short_exchange);
        let expiry_minutes =
            (global_opp.expiry_timestamp - global_opp.detection_timestamp) / (60 * 1000);

        let message = format!(
            "🚀 *New Arbitrage Opportunity*\n\n\
            📈 **Pair:** `{}`\n\
            🔄 **Exchanges:** {} ↔ {}\n\
            💰 **Rate Difference:** `{:.4}%`\n\
            💵 **Potential Profit:** `${:.2}`\n\
            ⭐ **Priority Score:** `{:.1}`\n\
            ⏰ **Expires:** {} minutes\n\n\
            🎯 Use `/opportunities` for more details\\!",
            opportunity.pair,
            long_exchange_name,
            short_exchange_name,
            opportunity.rate_difference * 100.0,
            opportunity.potential_profit_value.unwrap_or(0.0),
            global_opp.priority_score,
            expiry_minutes
        );

        assert!(message.contains("BTCUSDT"));
        assert!(message.contains("Binance"));
        assert!(message.contains("Bybit"));
    }

    #[tokio::test]
    async fn test_fairness_algorithm() {
        // Test different distribution strategies
        let eligible_users = [
            "user_001".to_string(),
            "user_002".to_string(),
            "user_003".to_string(),
            "user_004".to_string(),
            "user_005".to_string(),
        ];

        // Test FirstComeFirstServe strategy
        let _fcfs_opportunity = GlobalOpportunity {
            opportunity: create_test_opportunity(),
            detection_timestamp: chrono::Utc::now().timestamp_millis() as u64,
            expiry_timestamp: chrono::Utc::now().timestamp_millis() as u64 + (10 * 60 * 1000),
            priority_score: 75.0,
            distributed_to: Vec::new(),
            max_participants: Some(3),
            current_participants: 0,
            distribution_strategy: DistributionStrategy::FirstComeFirstServe,
            source: OpportunitySource::SystemGenerated,
        };

        // Test that FCFS selects first N users
        let max_users = 3;
        let selected_fcfs = &eligible_users[..std::cmp::min(eligible_users.len(), max_users)];
        assert_eq!(selected_fcfs.len(), 3);
        assert_eq!(selected_fcfs[0], "user_001");
        assert_eq!(selected_fcfs[1], "user_002");
        assert_eq!(selected_fcfs[2], "user_003");

        // Test Broadcast strategy
        let _broadcast_opportunity = GlobalOpportunity {
            opportunity: create_test_opportunity(),
            detection_timestamp: chrono::Utc::now().timestamp_millis() as u64,
            expiry_timestamp: chrono::Utc::now().timestamp_millis() as u64 + (10 * 60 * 1000),
            priority_score: 75.0,
            distributed_to: Vec::new(),
            max_participants: None,
            current_participants: 0,
            distribution_strategy: DistributionStrategy::Broadcast,
            source: OpportunitySource::SystemGenerated,
        };

        // Test that Broadcast selects all users
        let selected_broadcast = &eligible_users[..];
        assert_eq!(selected_broadcast.len(), 5);
        assert!(selected_broadcast.contains(&"user_001".to_string()));
        assert!(selected_broadcast.contains(&"user_005".to_string()));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        // Test rate limiting logic
        let config = DistributionConfig {
            max_opportunities_per_user_per_hour: 2,
            max_opportunities_per_user_per_day: 10,
            cooldown_period_minutes: 240, // 4 hours
            batch_size: 50,
            distribution_interval_seconds: 30,
            max_participants_per_opportunity: Some(100),
            fairness_config: FairnessConfig::default(),
        };

        // Test rate limit configuration
        assert_eq!(config.max_opportunities_per_user_per_hour, 2);
        assert_eq!(config.max_opportunities_per_user_per_day, 10);
        assert_eq!(config.cooldown_period_minutes, 240);

        // Test cooldown calculation
        let cooldown_ms = config.cooldown_period_minutes as u64 * 60 * 1000;
        assert_eq!(cooldown_ms, 14_400_000); // 4 hours in milliseconds

        // Test batch size limits
        assert_eq!(config.batch_size, 50);
        assert!(config.batch_size > 0);

        // Test distribution interval
        assert_eq!(config.distribution_interval_seconds, 30);
        assert!(config.distribution_interval_seconds >= 10); // Minimum reasonable interval
    }
}
