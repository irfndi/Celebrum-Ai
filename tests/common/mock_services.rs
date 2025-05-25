// Mock Service Implementations
// Simplified service mocks for testing without external dependencies

use std::collections::HashMap;
use serde_json::json;
use arb_edge::types::{UserProfile, UserTradingPreferences, ArbitrageOpportunity};
use arb_edge::services::core::analysis::market_analysis::TradingOpportunity;

/// Mock environment for testing service interactions
pub struct MockTestEnvironment {
    pub users: HashMap<String, UserProfile>,
    pub preferences: HashMap<String, UserTradingPreferences>,
    pub opportunities: Vec<TradingOpportunity>,
    pub arbitrage_opportunities: Vec<ArbitrageOpportunity>,
    pub notifications_sent: Vec<String>,
    pub market_data: HashMap<String, serde_json::Value>,
    pub ai_analysis_cache: HashMap<String, serde_json::Value>,
    pub monitoring_metrics: HashMap<String, f64>,
}

impl MockTestEnvironment {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            preferences: HashMap::new(),
            opportunities: Vec::new(),
            arbitrage_opportunities: Vec::new(),
            notifications_sent: Vec::new(),
            market_data: HashMap::new(),
            ai_analysis_cache: HashMap::new(),
            monitoring_metrics: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: UserProfile) {
        self.users.insert(user.user_id.clone(), user);
    }

    pub fn add_preferences(&mut self, user_id: String, prefs: UserTradingPreferences) {
        self.preferences.insert(user_id, prefs);
    }

    pub fn add_opportunity(&mut self, opportunity: TradingOpportunity) {
        self.opportunities.push(opportunity);
    }

    pub fn add_arbitrage_opportunity(&mut self, opportunity: ArbitrageOpportunity) {
        self.arbitrage_opportunities.push(opportunity);
    }

    pub fn add_market_data(&mut self, exchange: String, pair: String, data: serde_json::Value) {
        let key = format!("{}:{}", exchange, pair);
        self.market_data.insert(key, data);
    }

    pub fn add_ai_analysis(&mut self, opportunity_id: String, analysis: serde_json::Value) {
        self.ai_analysis_cache.insert(opportunity_id, analysis);
    }

    pub fn record_metric(&mut self, metric_name: String, value: f64) {
        self.monitoring_metrics.insert(metric_name, value);
    }

    pub fn send_notification(&mut self, user_id: String, message: String) {
        self.notifications_sent.push(format!("{}:{}", user_id, message));
    }

    pub fn get_user(&self, user_id: &str) -> Option<&UserProfile> {
        self.users.get(user_id)
    }

    pub fn get_preferences(&self, user_id: &str) -> Option<&UserTradingPreferences> {
        self.preferences.get(user_id)
    }

    pub fn get_opportunities_for_user(&self, user_id: &str) -> Vec<&TradingOpportunity> {
        // Simple filtering logic for testing
        self.opportunities.iter().collect()
    }

    pub fn get_market_data(&self, exchange: &str, pair: &str) -> Option<&serde_json::Value> {
        let key = format!("{}:{}", exchange, pair);
        self.market_data.get(&key)
    }

    pub fn get_ai_analysis(&self, opportunity_id: &str) -> Option<&serde_json::Value> {
        self.ai_analysis_cache.get(opportunity_id)
    }

    pub fn get_metric(&self, metric_name: &str) -> Option<f64> {
        self.monitoring_metrics.get(metric_name).copied()
    }

    pub fn clear(&mut self) {
        self.users.clear();
        self.preferences.clear();
        self.opportunities.clear();
        self.arbitrage_opportunities.clear();
        self.notifications_sent.clear();
        self.market_data.clear();
        self.ai_analysis_cache.clear();
        self.monitoring_metrics.clear();
    }
}

impl Default for MockTestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock UserProfileService for testing
pub struct MockUserProfileService {
    pub users: HashMap<String, UserProfile>,
}

impl MockUserProfileService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: UserProfile) {
        self.users.insert(user.user_id.clone(), user);
    }

    pub fn get_user(&self, user_id: &str) -> Option<&UserProfile> {
        self.users.get(user_id)
    }

    pub fn update_user(&mut self, user: UserProfile) {
        self.users.insert(user.user_id.clone(), user);
    }
}

/// Mock ExchangeService for testing
pub struct MockExchangeService {
    pub market_data: HashMap<String, serde_json::Value>,
}

impl MockExchangeService {
    pub fn new() -> Self {
        Self {
            market_data: HashMap::new(),
        }
    }

    pub fn add_market_data(&mut self, exchange: &str, pair: &str, data: serde_json::Value) {
        let key = format!("{}:{}", exchange, pair);
        self.market_data.insert(key, data);
    }

    pub fn get_price(&self, exchange: &str, pair: &str) -> Option<f64> {
        let key = format!("{}:{}", exchange, pair);
        self.market_data.get(&key)?.get("price")?.as_f64()
    }
}

/// Mock NotificationService for testing
pub struct MockNotificationService {
    pub sent_notifications: Vec<(String, String)>,
}

impl MockNotificationService {
    pub fn new() -> Self {
        Self {
            sent_notifications: Vec::new(),
        }
    }

    pub fn send_notification(&mut self, user_id: String, message: String) {
        self.sent_notifications.push((user_id, message));
    }

    pub fn get_notifications_for_user(&self, user_id: &str) -> Vec<&String> {
        self.sent_notifications
            .iter()
            .filter(|(uid, _)| uid == user_id)
            .map(|(_, msg)| msg)
            .collect()
    }

    pub fn clear(&mut self) {
        self.sent_notifications.clear();
    }
} 