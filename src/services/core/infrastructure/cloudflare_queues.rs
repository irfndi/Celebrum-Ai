// Cloudflare Queues Service for Robust Message Queuing
// Leverages Cloudflare Queues for reliable opportunity distribution, retry logic, and dead letter queues

use crate::types::ArbitrageOpportunity;
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::Env;

// Mock Queue and QueueEvent types for compilation when not available in worker crate
#[cfg(not(feature = "cloudflare_queues"))]
pub struct Queue;

#[cfg(not(feature = "cloudflare_queues"))]
impl Queue {
    pub async fn send(&self, _message: &QueueMessage, _delay: Option<u64>) -> Result<(), String> {
        // Mock implementation - in real Cloudflare environment this would send to queue
        Ok(())
    }
}

#[cfg(not(feature = "cloudflare_queues"))]
pub struct QueueEvent;

#[cfg(feature = "cloudflare_queues")]
use worker::{Queue, QueueEvent};

/// Configuration for Cloudflare Queues service
#[derive(Debug, Clone)]
pub struct CloudflareQueuesConfig {
    pub enabled: bool,
    pub opportunity_queue_name: String,
    pub notification_queue_name: String,
    pub analytics_queue_name: String,
    pub dead_letter_queue_name: String,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub batch_size: u32,
    pub visibility_timeout_seconds: u64,
}

impl Default for CloudflareQueuesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            opportunity_queue_name: "opportunity-distribution".to_string(),
            notification_queue_name: "user-notifications".to_string(),
            analytics_queue_name: "analytics-events".to_string(),
            dead_letter_queue_name: "dead-letter-queue".to_string(),
            max_retries: 3,
            retry_delay_seconds: 60,
            batch_size: 10,
            visibility_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Opportunity distribution message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityDistributionMessage {
    pub message_id: String,
    pub opportunity: ArbitrageOpportunity,
    pub target_users: Vec<String>,
    pub priority: MessagePriority,
    pub created_at: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub distribution_strategy: DistributionStrategy,
}

/// Distribution strategy options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionStrategy {
    Broadcast,       // Send to all eligible users
    RoundRobin,      // Distribute fairly among users
    PriorityBased,   // Send to highest priority users first
    GeographicBased, // Send based on user location/timezone
}

/// User notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationMessage {
    pub message_id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub content: String,
    pub priority: MessagePriority,
    pub delivery_method: DeliveryMethod,
    pub scheduled_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub retry_count: u32,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    OpportunityAlert,
    TradingSignal,
    SystemNotification,
    MarketingMessage,
    SecurityAlert,
}

/// Delivery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryMethod {
    Telegram,
    Email,
    WebPush,
    SMS,
}

/// Analytics event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEventMessage {
    pub event_id: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub priority: MessagePriority,
}

/// Queue message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: String,
    pub queue_name: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub priority: MessagePriority,
    pub created_at: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub visibility_timeout: u64,
}

/// Message processing result
#[derive(Debug, Clone)]
pub enum MessageProcessingResult {
    Success,
    Retry(String),      // Reason for retry
    DeadLetter(String), // Reason for dead letter
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatistics {
    pub queue_name: String,
    pub total_messages: u64,
    pub pending_messages: u64,
    pub processing_messages: u64,
    pub dead_letter_messages: u64,
    pub success_rate_percent: f32,
    pub average_processing_time_ms: f32,
    pub last_updated: u64,
}

/// Cloudflare Queues Service
pub struct CloudflareQueuesService {
    config: CloudflareQueuesConfig,
    queues: HashMap<String, Queue>,
    statistics: HashMap<String, QueueStatistics>,
    logger: crate::utils::logger::Logger,
}

impl CloudflareQueuesService {
    /// Create new CloudflareQueuesService instance
    pub fn new(_env: &Env, config: CloudflareQueuesConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);
        let mut queues = HashMap::new();

        // Initialize queues
        let queue_names = vec![
            &config.opportunity_queue_name,
            &config.notification_queue_name,
            &config.analytics_queue_name,
            &config.dead_letter_queue_name,
        ];

        for queue_name in queue_names {
            #[cfg(feature = "cloudflare_queues")]
            match env.queue(queue_name) {
                Ok(queue) => {
                    logger.info(&format!("Queue '{}' connected successfully", queue_name));
                    queues.insert(queue_name.clone(), queue);
                }
                Err(e) => {
                    logger.warn(&format!(
                        "Failed to connect to queue '{}': {}",
                        queue_name, e
                    ));
                }
            }

            #[cfg(not(feature = "cloudflare_queues"))]
            {
                logger.warn(&format!(
                    "Queue '{}' not available - using mock implementation",
                    queue_name
                ));
                queues.insert(queue_name.clone(), Queue);
            }
        }

        Ok(Self {
            config,
            queues,
            statistics: HashMap::new(),
            logger,
        })
    }

    /// Send opportunity distribution message
    pub async fn send_opportunity_distribution(
        &self,
        opportunity: &ArbitrageOpportunity,
        target_users: Vec<String>,
        priority: MessagePriority,
        strategy: DistributionStrategy,
    ) -> ArbitrageResult<String> {
        if !self.config.enabled {
            return Err(ArbitrageError::service_unavailable(
                "Queues service disabled",
            ));
        }

        let message = OpportunityDistributionMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            opportunity: opportunity.clone(),
            target_users,
            priority: priority.clone(),
            created_at: chrono::Utc::now().timestamp() as u64,
            retry_count: 0,
            max_retries: self.config.max_retries,
            distribution_strategy: strategy,
        };

        let queue_name = self.config.opportunity_queue_name.clone();
        self.send_message(&queue_name, "opportunity_distribution", &message, priority)
            .await
    }

    /// Send user notification message
    #[allow(clippy::too_many_arguments)]
    pub async fn send_user_notification(
        &mut self,
        user_id: &str,
        notification_type: NotificationType,
        content: &str,
        priority: MessagePriority,
        delivery_method: DeliveryMethod,
        scheduled_at: Option<u64>,
        expires_at: Option<u64>,
    ) -> ArbitrageResult<String> {
        if !self.config.enabled {
            return Err(ArbitrageError::service_unavailable(
                "Queues service disabled",
            ));
        }

        let message = UserNotificationMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            notification_type,
            content: content.to_string(),
            priority: priority.clone(),
            delivery_method,
            scheduled_at,
            expires_at,
            retry_count: 0,
        };

        let queue_name = self.config.notification_queue_name.clone();
        self.send_message(&queue_name, "user_notification", &message, priority)
            .await
    }

    /// Send analytics event message
    pub async fn send_analytics_event(
        &mut self,
        event_type: &str,
        user_id: Option<&str>,
        data: serde_json::Value,
        priority: MessagePriority,
    ) -> ArbitrageResult<String> {
        if !self.config.enabled {
            return Err(ArbitrageError::service_unavailable(
                "Queues service disabled",
            ));
        }

        let message = AnalyticsEventMessage {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            data,
            timestamp: chrono::Utc::now().timestamp() as u64,
            priority: priority.clone(),
        };

        let queue_name = self.config.analytics_queue_name.clone();
        self.send_message(&queue_name, "analytics_event", &message, priority)
            .await
    }

    /// Process opportunity distribution message
    pub async fn process_opportunity_distribution(
        &mut self,
        message: &OpportunityDistributionMessage,
    ) -> ArbitrageResult<MessageProcessingResult> {
        self.logger.info(&format!(
            "Processing opportunity distribution: id={}, users={}, strategy={:?}",
            message.message_id,
            message.target_users.len(),
            message.distribution_strategy
        ));

        // Simulate processing logic
        match message.distribution_strategy {
            DistributionStrategy::Broadcast => {
                // Send to all users simultaneously
                for user_id in &message.target_users {
                    // In real implementation, this would send notifications
                    self.logger
                        .info(&format!("Broadcasting opportunity to user: {}", user_id));
                }
            }
            DistributionStrategy::RoundRobin => {
                // Distribute fairly among users
                for (index, user_id) in message.target_users.iter().enumerate() {
                    self.logger.info(&format!(
                        "Round-robin distribution to user {}: {}",
                        index, user_id
                    ));
                }
            }
            DistributionStrategy::PriorityBased => {
                // Send to highest priority users first
                self.logger.info("Priority-based distribution");
            }
            DistributionStrategy::GeographicBased => {
                // Send based on user timezone
                self.logger.info("Geographic-based distribution");
            }
        }

        Ok(MessageProcessingResult::Success)
    }

    /// Process user notification message
    pub async fn process_user_notification(
        &mut self,
        message: &UserNotificationMessage,
    ) -> ArbitrageResult<MessageProcessingResult> {
        self.logger.info(&format!(
            "Processing user notification: id={}, user={}, type={:?}",
            message.message_id, message.user_id, message.notification_type
        ));

        // Check if message has expired
        if let Some(expires_at) = message.expires_at {
            let now = chrono::Utc::now().timestamp() as u64;
            if now > expires_at {
                return Ok(MessageProcessingResult::DeadLetter(
                    "Message expired".to_string(),
                ));
            }
        }

        // Check if message is scheduled for future delivery
        if let Some(scheduled_at) = message.scheduled_at {
            let now = chrono::Utc::now().timestamp() as u64;
            if now < scheduled_at {
                return Ok(MessageProcessingResult::Retry(
                    "Message scheduled for future".to_string(),
                ));
            }
        }

        // Process based on delivery method
        match message.delivery_method {
            DeliveryMethod::Telegram => {
                self.logger.info("Sending Telegram notification");
                // In real implementation, this would call TelegramService
            }
            DeliveryMethod::Email => {
                self.logger.info("Sending email notification");
                // In real implementation, this would call EmailService
            }
            DeliveryMethod::WebPush => {
                self.logger.info("Sending web push notification");
                // In real implementation, this would call WebPushService
            }
            DeliveryMethod::SMS => {
                self.logger.info("Sending SMS notification");
                // In real implementation, this would call SMSService
            }
        }

        Ok(MessageProcessingResult::Success)
    }

    /// Process analytics event message
    pub async fn process_analytics_event(
        &mut self,
        message: &AnalyticsEventMessage,
    ) -> ArbitrageResult<MessageProcessingResult> {
        self.logger.info(&format!(
            "Processing analytics event: id={}, type={}, user={:?}",
            message.event_id, message.event_type, message.user_id
        ));

        // In real implementation, this would send to Analytics Engine
        Ok(MessageProcessingResult::Success)
    }

    /// Send message to queue
    async fn send_message(
        &self,
        queue_name: &str,
        message_type: &str,
        payload: &impl Serialize,
        priority: MessagePriority,
    ) -> ArbitrageResult<String> {
        let queue = self.queues.get(queue_name).ok_or_else(|| {
            ArbitrageError::service_unavailable(format!("Queue '{}' not available", queue_name))
        })?;

        let message_id = uuid::Uuid::new_v4().to_string();
        let queue_message = QueueMessage {
            id: message_id.clone(),
            queue_name: queue_name.to_string(),
            message_type: message_type.to_string(),
            payload: serde_json::to_value(payload)?,
            priority: priority.clone(),
            created_at: chrono::Utc::now().timestamp() as u64,
            retry_count: 0,
            max_retries: self.config.max_retries,
            visibility_timeout: self.config.visibility_timeout_seconds,
        };

        // Send to queue with priority-based delay
        let delay_seconds = match priority {
            MessagePriority::Critical => 0,
            MessagePriority::High => 1,
            MessagePriority::Normal => 5,
            MessagePriority::Low => 30,
        };

        queue
            .send(&queue_message, Some(delay_seconds))
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to send message to queue '{}': {}",
                    queue_name, e
                ))
            })?;

        self.logger.info(&format!(
            "Message sent to queue '{}': id={}, type={}, priority={:?}",
            queue_name, message_id, message_type, priority
        ));

        // Update statistics (commented out for immutable self)
        // self.update_queue_statistics(queue_name, true).await?;

        Ok(message_id)
    }

    /// Handle message retry
    pub async fn retry_message(
        &mut self,
        queue_name: &str,
        message: &QueueMessage,
        reason: &str,
    ) -> ArbitrageResult<()> {
        if message.retry_count >= message.max_retries {
            return self.send_to_dead_letter_queue(message, reason).await;
        }

        let mut retry_message = message.clone();
        retry_message.retry_count += 1;

        // Exponential backoff for retry delay with cap to prevent overflow
        let max_delay_seconds = 3600; // Cap at 1 hour
        let exponent = std::cmp::min(retry_message.retry_count, 10); // Cap exponent to prevent overflow
        let retry_delay = std::cmp::min(
            self.config.retry_delay_seconds * (2_u64.pow(exponent)),
            max_delay_seconds
        );

        let queue = self.queues.get(queue_name).ok_or_else(|| {
            ArbitrageError::service_unavailable(format!("Queue '{}' not available", queue_name))
        })?;

        queue
            .send(&retry_message, Some(retry_delay))
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to retry message: {}", e))
            })?;

        self.logger.info(&format!(
            "Message retried: id={}, attempt={}/{}, delay={}s, reason={}",
            message.id, retry_message.retry_count, message.max_retries, retry_delay, reason
        ));

        Ok(())
    }

    /// Send message to dead letter queue
    pub async fn send_to_dead_letter_queue(
        &mut self,
        message: &QueueMessage,
        reason: &str,
    ) -> ArbitrageResult<()> {
        let dead_letter_queue = self
            .queues
            .get(&self.config.dead_letter_queue_name)
            .ok_or_else(|| {
                ArbitrageError::service_unavailable("Dead letter queue not available")
            })?;

        let dead_letter_payload = serde_json::json!({
            "original_message": message,
            "dead_letter_reason": reason,
            "dead_letter_timestamp": chrono::Utc::now().timestamp(),
            "original_queue": message.queue_name
        });

        let dead_letter_message = QueueMessage {
            id: uuid::Uuid::new_v4().to_string(),
            queue_name: self.config.dead_letter_queue_name.clone(),
            message_type: "dead_letter".to_string(),
            payload: dead_letter_payload,
            priority: MessagePriority::High,
            created_at: chrono::Utc::now().timestamp() as u64,
            retry_count: 0,
            max_retries: 0, // Dead letter messages don't retry
            visibility_timeout: self.config.visibility_timeout_seconds,
        };

        dead_letter_queue
            .send(&dead_letter_message, None)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to send to dead letter queue: {}", e))
            })?;

        self.logger.warn(&format!(
            "Message sent to dead letter queue: id={}, reason={}",
            message.id, reason
        ));

        Ok(())
    }

    /// Get queue statistics
    pub async fn get_queue_statistics(&self, queue_name: &str) -> ArbitrageResult<QueueStatistics> {
        self.statistics.get(queue_name).cloned().ok_or_else(|| {
            ArbitrageError::not_found(format!("Statistics not found for queue: {}", queue_name))
        })
    }

    /// Get all queue statistics
    pub async fn get_all_statistics(&self) -> ArbitrageResult<Vec<QueueStatistics>> {
        Ok(self.statistics.values().cloned().collect())
    }

    /// Update queue statistics
    #[allow(dead_code)]
    async fn update_queue_statistics(
        &mut self,
        queue_name: &str,
        success: bool,
    ) -> ArbitrageResult<()> {
        let stats = self
            .statistics
            .entry(queue_name.to_string())
            .or_insert_with(|| QueueStatistics {
                queue_name: queue_name.to_string(),
                total_messages: 0,
                pending_messages: 0,
                processing_messages: 0,
                dead_letter_messages: 0,
                success_rate_percent: 100.0,
                average_processing_time_ms: 0.0,
                last_updated: chrono::Utc::now().timestamp() as u64,
            });

        stats.total_messages += 1;

        if success {
            stats.success_rate_percent =
                (stats.success_rate_percent * (stats.total_messages - 1) as f32 + 100.0)
                    / stats.total_messages as f32;
        } else {
            stats.success_rate_percent = (stats.success_rate_percent
                * (stats.total_messages - 1) as f32)
                / stats.total_messages as f32;
        }

        stats.last_updated = chrono::Utc::now().timestamp() as u64;

        Ok(())
    }

    /// Health check for queues
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        Ok(self.config.enabled && !self.queues.is_empty())
    }

    /// Purge queue (for testing/maintenance)
    pub async fn purge_queue(&mut self, queue_name: &str) -> ArbitrageResult<()> {
        let _queue = self.queues.get(queue_name).ok_or_else(|| {
            ArbitrageError::service_unavailable(format!("Queue '{}' not available", queue_name))
        })?;

        // Note: Cloudflare Queues doesn't have a direct purge method
        // This would need to be implemented by consuming all messages
        self.logger
            .warn(&format!("Purge requested for queue: {}", queue_name));

        Ok(())
    }
}
