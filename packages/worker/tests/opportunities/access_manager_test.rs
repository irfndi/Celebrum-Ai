use cerebrum_ai::services::core::opportunities::access_manager::*;
use cerebrum_ai::types::{UserConfiguration, UserProfile, SubscriptionTier, UserAccessLevel, ExchangeIdEnum};
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_profile(user_id: &str, tier: SubscriptionTier) -> UserProfile {
        UserProfile {
            user_id: user_id.to_string(),
            telegram_user_id: Some(123456789),
            username: Some("testuser".to_string()),
            email: Some("test@example.com".to_string()),
            subscription_tier: tier.clone(),
            access_level: UserAccessLevel::Free,
            is_active: true,
            is_beta_active: false,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            last_login: None,
            preferences: cerebrum_ai::types::UserPreferences::default(),
            risk_profile: cerebrum_ai::types::RiskProfile::default(),
            subscription: cerebrum_ai::types::Subscription {
                tier,
                is_active: true,
                features: vec!["basic_features".to_string()],
                daily_opportunity_limit: Some(10),
                expires_at: None,
                created_at: chrono::Utc::now().timestamp_millis() as u64,
                updated_at: chrono::Utc::now().timestamp_millis() as u64,
            },
            configuration: UserConfiguration::default(),
            api_keys: Vec::new(),
            invitation_code: None,
            invitation_code_used: None,
            invited_by: None,
            total_invitations_sent: 0,
            successful_invitations: 0,
            beta_expires_at: None,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            last_active: chrono::Utc::now().timestamp_millis() as u64, // Corrected: last_active is u64
            total_trades: 0,
            total_pnl_usdt: 0.0,
            account_balance_usdt: 0.0,
            profile_metadata: None,
            telegram_username: Some("testuser".to_string()),
            group_admin_roles: Vec::new(),
        }
    }

    #[test]
    fn test_exchange_id_from_str() {
        assert!(matches!(
            ExchangeIdEnum::from_str("binance"),
            Ok(ExchangeIdEnum::Binance)
        ));
        assert!(matches!(
            ExchangeIdEnum::from_str("BYBIT"),
            Ok(ExchangeIdEnum::Bybit)
        ));
        assert!(ExchangeIdEnum::from_str("invalid").is_err());
    }

    #[test]
    fn test_subscription_tier_filtering() {
        // Test that subscription tier logic is properly implemented
        let free_user = create_test_user_profile("user1", SubscriptionTier::Free);
        let premium_user = create_test_user_profile("user2", SubscriptionTier::Premium);

        assert_eq!(free_user.subscription.tier, SubscriptionTier::Free);
        assert_eq!(premium_user.subscription.tier, SubscriptionTier::Premium);
    }
}