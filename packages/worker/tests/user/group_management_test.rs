use crate::services::core::ai::types::*;
use crate::services::core::user::group_management::*;
use crate::services::core::user::types::*;

#[tokio::test]
async fn test_group_registration() {
    let group_id = "test_group_123";
    let group_name = "Test Group";
    let admin_user_id = "admin_123";

    // Test group registration data structure
    let now = chrono::Utc::now().timestamp_millis() as u64;
    let registration = GroupRegistration {
        group_id: group_id.to_string(),
        group_title: group_name.to_string(),
        group_type: "group".to_string(),
        registered_by: admin_user_id.to_string(),
        registered_at: now,
        is_active: true,
        settings: GroupSettings::default(),
        rate_limit_config: GroupRateLimitConfig::default(),
        group_name: group_name.to_string(),
        registration_date: now,
        subscription_tier: SubscriptionTier::Free,
        registration_id: format!("reg_{}", group_id),
        registered_by_user_id: admin_user_id.to_string(),
        group_username: None,
        member_count: None,
        admin_user_ids: vec![admin_user_id.to_string()],
        bot_permissions: serde_json::Value::Object(serde_json::Map::new()),
        enabled_features: Vec::new(),
        last_activity: Some(now),
        total_messages_sent: 0,
        last_member_count_update: None,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(registration.group_id, group_id);
    assert_eq!(registration.subscription_tier, SubscriptionTier::Free);
    assert!(registration.is_active);
}

#[tokio::test]
async fn test_group_config() {
    let group_id = "test_group_123";
    let admin_user_id = "admin_123";

    let config = GroupChannelConfig::new_group(group_id.to_string(), admin_user_id.to_string());

    assert_eq!(config.group_id, group_id);
    assert_eq!(config.group_type, "group");
    assert!(config.opportunities_enabled);
    assert!(!config.manual_requests_enabled);
    assert!(!config.trading_enabled);
    assert!(config.take_action_buttons);
    assert!(config.is_admin(admin_user_id));
}

#[tokio::test]
async fn test_ai_settings() {
    let group_id = "test_group_123";
    let admin_user_id = "admin_123";

    let mut settings = GroupAISettings::new(group_id.to_string(), admin_user_id.to_string());

    assert_eq!(settings.group_id, group_id);
    assert!(!settings.ai_enabled);
    assert!(!settings.byok_enabled); // Fixed: GroupAISettings::new sets byok_enabled to false
    assert_eq!(
        settings.get_ai_enhancement_mode(),
        AIEnhancementMode::Disabled
    );

    // Test enabling AI
    settings.enable_ai(ApiKeyProvider::OpenAI, Some("gpt-4".to_string()));
    assert!(settings.ai_enabled);
    assert_eq!(
        settings.get_ai_enhancement_mode(),
        AIEnhancementMode::BYOKOnly
    );
}
