// src/test_utils/mod.rs

#[cfg(test)]
pub mod mock_kv_store;

// Add test utility functions
#[cfg(test)]
pub fn create_test_user_profile() -> crate::types::UserProfile {
    use crate::types::{
        RiskProfile, Subscription, SubscriptionTier, UserAccessLevel, UserConfiguration,
        UserPreferences, UserProfile,
    };

    UserProfile {
        user_id: "test_user_123".to_string(),
        telegram_user_id: Some(123456789),
        telegram_username: Some("test_user".to_string()),
        username: Some("test_user".to_string()),
        email: Some("test@example.com".to_string()),
        access_level: UserAccessLevel::Free,
        subscription_tier: SubscriptionTier::Free,
        api_keys: Vec::new(),
        preferences: UserPreferences::default(),
        risk_profile: RiskProfile::default(),
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        updated_at: chrono::Utc::now().timestamp_millis() as u64,
        last_active: chrono::Utc::now().timestamp_millis() as u64,
        last_login: Some(chrono::Utc::now().timestamp_millis() as u64),
        is_active: true,
        is_beta_active: false,
        invitation_code_used: None,
        invitation_code: None,
        invited_by: None,
        total_invitations_sent: 0,
        successful_invitations: 0,
        beta_expires_at: None,
        total_trades: 0,
        total_pnl_usdt: 0.0,
        account_balance_usdt: 1000.0,
        profile_metadata: None,
        subscription: Subscription::default(),
        group_admin_roles: Vec::new(),
        configuration: UserConfiguration::default(),
    }
}

#[cfg(test)]
pub fn create_mock_session() -> crate::types::EnhancedUserSession {
    use crate::types::{
        EnhancedSessionState, EnhancedUserSession, SessionAnalytics, SessionConfig,
    };
    use std::collections::HashMap;

    EnhancedUserSession {
        session_id: "test_session_123".to_string(),
        user_id: "test_user_123".to_string(),
        telegram_chat_id: 123456789,
        telegram_id: 123456789,
        last_command: None,
        current_state: EnhancedSessionState::Active,
        session_state: EnhancedSessionState::Active,
        temporary_data: HashMap::new(),
        started_at: chrono::Utc::now().timestamp_millis() as u64,
        last_activity_at: chrono::Utc::now().timestamp_millis() as u64,
        expires_at: chrono::Utc::now().timestamp_millis() as u64 + 3600000, // 1 hour
        onboarding_completed: true,
        preferences_set: true,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        updated_at: chrono::Utc::now().timestamp_millis() as u64,
        session_analytics: SessionAnalytics::default(),
        config: SessionConfig::default(),
    }
}

#[cfg(test)]
pub fn create_mock_arbitrage_opportunity() -> crate::types::ArbitrageOpportunity {
    use crate::types::{ArbitrageOpportunity, ArbitrageType, ExchangeIdEnum};

    ArbitrageOpportunity {
        id: "test_arb_123".to_string(),
        trading_pair: "BTCUSDT".to_string(),
        exchanges: vec!["Binance".to_string(), "Bybit".to_string()],
        profit_percentage: 2.5,
        confidence_score: 85.0,
        risk_level: "Medium".to_string(),
        buy_exchange: "Binance".to_string(),
        sell_exchange: "Bybit".to_string(),
        buy_price: 50000.0,
        sell_price: 51250.0,
        volume: 1.0,
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        expires_at: Some(chrono::Utc::now().timestamp_millis() as u64 + 300000), // 5 minutes
        pair: "BTCUSDT".to_string(),
        long_exchange: ExchangeIdEnum::Binance,
        short_exchange: ExchangeIdEnum::Bybit,
        long_rate: Some(0.01),
        short_rate: Some(0.02),
        rate_difference: 2.5,
        net_rate_difference: Some(2.3),
        potential_profit_value: Some(1250.0),
        confidence: 85.0,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        detected_at: chrono::Utc::now().timestamp_millis() as u64,
        r#type: ArbitrageType::CrossExchange,
        details: Some("Cross-exchange arbitrage between Binance and Bybit".to_string()),
        min_exchanges_required: 2,
    }
}
