//! Cloudflare Queues Module
//!
//! This module defines all queue message types and related enums for Cloudflare Workers Queue functionality.
//! Designed for high efficiency, type safety, and production-ready queue message processing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export MessageBatch for convenience
pub use worker::MessageBatch;

// ============= QUEUE MESSAGE TYPES =============

/// Analytics event message for tracking user actions and system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEventMessage {
    pub event_id: String,
    pub user_id: Option<String>,
    pub event_type: String,
    pub event_data: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
    pub session_id: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Opportunity distribution message for sharing arbitrage opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityDistributionMessage {
    pub opportunity_id: String,
    pub user_id: String,
    pub target_users: Vec<String>,
    pub opportunity: OpportunityData,
    pub opportunity_data: serde_json::Value,
    pub priority: Priority,
    pub distribution_strategy: DistributionStrategy,
    pub expires_at: Option<i64>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Opportunity data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityData {
    pub pair: String,
    pub long_exchange: String,
    pub short_exchange: String,
    pub rate_difference: f64,
    pub confidence: f64,
}

/// General notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub notification_id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub notification_type: String,
    pub priority: Priority,
    pub delivery_method: DeliveryMethod,
    pub scheduled_at: Option<i64>,
    pub metadata: Option<HashMap<String, String>>,
}

/// User-specific notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationMessage {
    pub notification_id: String,
    pub user_id: String,
    pub message: String,
    pub notification_type: String,
    pub priority: Priority,
    pub delivery_method: DeliveryMethod,
    pub read_status: bool,
    pub created_at: i64,
    pub metadata: Option<HashMap<String, String>>,
}

/// General user message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub message_id: String,
    pub user_id: String,
    pub content: String,
    pub message_type: String,
    pub priority: Priority,
    pub created_at: i64,
    pub metadata: Option<HashMap<String, String>>,
}

/// Generic queue event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEvent<T> {
    pub event_id: String,
    pub event_type: String,
    pub payload: T,
    pub timestamp: i64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: Option<HashMap<String, String>>,
}

// ============= ENUMS =============

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Distribution strategies for opportunity messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DistributionStrategy {
    Broadcast, // Send to all eligible users
    #[default]
    Targeted, // Send to specific users based on criteria
    RoundRobin, // Distribute evenly among users
    PriorityBased, // Send to highest priority users first
    Geographic, // Send based on geographic location
    Tiered,    // Send based on subscription tier
    Personalized, // AI-personalized distribution
    HighestBidder, // Premium users first
    Immediate, // Immediate distribution
    Batched,   // Batched distribution
    Prioritized, // Prioritized distribution
    RateLimited, // Rate-limited distribution
    FirstComeFirstServe, // First-come, first-serve distribution
    Priority,  // Priority distribution
}

impl DistributionStrategy {
    pub fn to_stable_string(&self) -> String {
        match self {
            DistributionStrategy::Broadcast => "broadcast".to_string(),
            DistributionStrategy::Targeted => "targeted".to_string(),
            DistributionStrategy::RoundRobin => "round_robin".to_string(),
            DistributionStrategy::PriorityBased => "priority_based".to_string(),
            DistributionStrategy::Geographic => "geographic".to_string(),
            DistributionStrategy::Tiered => "tiered".to_string(),
            DistributionStrategy::Personalized => "personalized".to_string(),
            DistributionStrategy::HighestBidder => "highest_bidder".to_string(),
            DistributionStrategy::Immediate => "immediate".to_string(),
            DistributionStrategy::Batched => "batched".to_string(),
            DistributionStrategy::Prioritized => "prioritized".to_string(),
            DistributionStrategy::RateLimited => "rate_limited".to_string(),
            DistributionStrategy::FirstComeFirstServe => "first_come_first_serve".to_string(),
            DistributionStrategy::Priority => "priority".to_string(),
        }
    }
}

/// Delivery methods for notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DeliveryMethod {
    #[default]
    InApp, // In-application notification
    Telegram,                      // Telegram bot message
    Email,                         // Email notification
    Push,                          // Push notification
    Webhook,                       // Webhook delivery
    Multiple(Vec<DeliveryMethod>), // Multiple delivery methods
}

// ============= HELPER FUNCTIONS =============

impl AnalyticsEventMessage {
    pub fn new(event_type: String, user_id: Option<String>) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            event_type,
            event_data: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
            session_id: None,
            metadata: None,
        }
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.event_data.insert(key, value);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

impl OpportunityDistributionMessage {
    pub fn new(
        user_id: String,
        target_users: Vec<String>,
        opportunity: OpportunityData,
        opportunity_data: serde_json::Value,
        priority: Priority,
    ) -> Self {
        Self {
            opportunity_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            target_users,
            opportunity,
            opportunity_data,
            priority,
            distribution_strategy: DistributionStrategy::default(),
            expires_at: None,
            metadata: None,
        }
    }

    pub fn with_strategy(mut self, strategy: DistributionStrategy) -> Self {
        self.distribution_strategy = strategy;
        self
    }

    pub fn with_expiry(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

impl NotificationMessage {
    pub fn new(user_id: String, title: String, content: String, notification_type: String) -> Self {
        Self {
            notification_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            title,
            content,
            notification_type,
            priority: Priority::default(),
            delivery_method: DeliveryMethod::default(),
            scheduled_at: None,
            metadata: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_delivery_method(mut self, method: DeliveryMethod) -> Self {
        self.delivery_method = method;
        self
    }

    pub fn with_schedule(mut self, scheduled_at: i64) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }
}

impl UserNotificationMessage {
    pub fn new(user_id: String, message: String, notification_type: String) -> Self {
        Self {
            notification_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            message,
            notification_type,
            priority: Priority::default(),
            delivery_method: DeliveryMethod::default(),
            read_status: false,
            created_at: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_delivery_method(mut self, method: DeliveryMethod) -> Self {
        self.delivery_method = method;
        self
    }

    pub fn mark_as_read(mut self) -> Self {
        self.read_status = true;
        self
    }
}

impl UserMessage {
    pub fn new(user_id: String, content: String, message_type: String) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            content,
            message_type,
            priority: Priority::default(),
            created_at: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

impl<T> QueueEvent<T> {
    pub fn new(event_type: String, payload: T) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            payload,
            timestamp: chrono::Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 3,
            metadata: None,
        }
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn increment_retry(mut self) -> Self {
        self.retry_count += 1;
        self
    }

    pub fn should_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}
