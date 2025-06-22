use arbedge_worker::opportunities::opportunity_engine::*;
use arbedge_worker::types::{ChatContext, SubscriptionTier};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_profile(user_id: &str) -> UserProfile {
        UserProfile {
            user_id: user_id.to_string(),
            subscription: Subscription {
                tier: SubscriptionTier::Free,
                expires_at: None,
                features: Vec::new(),
            },
            preferences: UserPreferences {
                notification_settings: NotificationSettings {
                    telegram_enabled: true,
                    discord_enabled: false,
                    email_enabled: false,
                    push_enabled: false,
                },
                trading_preferences: TradingPreferences {
                    preferred_exchanges: Vec::new(),
                    preferred_pairs: Vec::new(),
                    risk_tolerance: RiskTolerance::Medium,
                    max_investment_amount: None,
                },
            },
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            account_balance_usdt: 0.0,
            group_admin_roles: Vec::new(),
            is_beta_active: false,
        }
    }

    #[test]
    fn test_opportunity_context_mapping() {
        // Test that different contexts are properly handled
        let personal_context = OpportunityContext::Personal {
            user_id: "test_user".to_string(),
        };
        let group_context = OpportunityContext::Group {
            admin_id: "admin_user".to_string(),
            chat_context: ChatContext {
                chat_id: -123456789,
                chat_type: "group".to_string(),
                user_id: Some("test_user".to_string()),
                username: Some("testuser".to_string()),
                is_group: true,
                group_title: Some("Test Group".to_string()),
                message_id: Some(1),
                reply_to_message_id: None,
            },
        };
        let global_context = OpportunityContext::Global { system_level: true };

        assert!(matches!(
            personal_context,
            OpportunityContext::Personal { .. }
        ));
        assert!(matches!(group_context, OpportunityContext::Group { .. }));
        assert!(matches!(global_context, OpportunityContext::Global { .. }));
    }

    #[test]
    fn test_user_profile_structure() {
        let user_profile = create_test_user_profile("test_user");

        assert_eq!(user_profile.user_id, "test_user");
        assert_eq!(user_profile.subscription.tier, SubscriptionTier::Free);
        assert!(user_profile.is_active);
        assert_eq!(user_profile.account_balance_usdt, 0.0);
    }

    #[test]
    fn test_opportunity_config_defaults() {
        let config = OpportunityConfig::default();

        assert!(config.min_rate_difference > 0.0);
        assert!(config.min_rate_difference > 0.0);
        assert!(!config.default_pairs.is_empty());
        assert!(!config.monitored_exchanges.is_empty());
        assert!(config.opportunity_ttl_minutes > 0);
        assert!(config.max_participants_per_opportunity > 0);
    }
}