use crate::user::user_access::*;

#[test]
fn test_user_access_level_limits() {
    let free_tier = UserAccessLevel::Free;
    let pro_tier = UserAccessLevel::Pro;
    let ultra_tier = UserAccessLevel::Ultra;

    // Test daily limits
    assert_eq!(free_tier.get_daily_opportunity_limits(), (0, 0));
    assert_eq!(
        pro_tier.get_daily_opportunity_limits(),
        (u32::MAX, u32::MAX)
    );
    assert_eq!(
        ultra_tier.get_daily_opportunity_limits(),
        (u32::MAX, u32::MAX)
    );

    // Test trading capability
    assert!(!free_tier.can_trade());
    assert!(pro_tier.can_trade());
    assert!(ultra_tier.can_trade());

    // Test real-time opportunities
    assert!(!free_tier.gets_realtime_opportunities());
    assert!(pro_tier.gets_realtime_opportunities());
    assert!(ultra_tier.gets_realtime_opportunities());

    // Test delay
    assert_eq!(free_tier.get_opportunity_delay_seconds(), 300);
    assert_eq!(pro_tier.get_opportunity_delay_seconds(), 5);
    assert_eq!(ultra_tier.get_opportunity_delay_seconds(), 0);
}

#[test]
fn test_user_opportunity_limits() {
    let user_id = "test_user".to_string();
    let access_level = UserAccessLevel::Pro;

    // Test private context
    let mut limits = UserOpportunityLimits::new(user_id.clone(), &access_level, false);
    assert_eq!(limits.daily_global_opportunities, 500); // Pro tier gets high limits
    assert_eq!(limits.daily_technical_opportunities, 250); // Pro tier gets 250 technical
    assert!(limits.can_receive_realtime); // Pro tier gets realtime

    // Test group context (reduced limits for groups)
    let group_limits = UserOpportunityLimits::new(user_id, &access_level, true);
    assert_eq!(group_limits.daily_global_opportunities, 250); // Reduced for groups (500/2)
    assert_eq!(group_limits.daily_technical_opportunities, 125); // Reduced for groups (250/2)
    assert!(group_limits.can_receive_realtime);

    // Test receiving opportunities
    assert!(limits.can_receive_arbitrage());
    assert!(limits.record_arbitrage_received());
    assert_eq!(limits.current_arbitrage_count, 1);

    assert!(limits.can_receive_technical());
    assert!(limits.record_technical_received());
    assert_eq!(limits.current_technical_count, 1);

    // Test remaining opportunities
    let (remaining_arb, remaining_tech) = limits.get_remaining_opportunities();
    assert_eq!(remaining_arb, 499);
    assert_eq!(remaining_tech, 249);
}

#[test]
fn test_chat_context() {
    let private = ChatContext {
        chat_id: 123,
        chat_type: ChatContext::PRIVATE.to_string(),
        user_id: Some("user123".to_string()),
        username: None,
        is_group: false,
        group_title: None,
        message_id: None,
        reply_to_message_id: None,
    };
    let group = ChatContext {
        chat_id: 456,
        chat_type: ChatContext::GROUP.to_string(),
        user_id: Some("user123".to_string()),
        username: None,
        is_group: true,
        group_title: Some("Test Group".to_string()),
        message_id: None,
        reply_to_message_id: None,
    };
    let channel = ChatContext {
        chat_id: 789,
        chat_type: ChatContext::CHANNEL.to_string(),
        user_id: Some("user123".to_string()),
        username: None,
        is_group: false,
        group_title: Some("Test Channel".to_string()),
        message_id: None,
        reply_to_message_id: None,
    };

    assert!(!private.is_group_context());
    assert!(group.is_group_context());
    assert!(channel.is_group_context());

    assert_eq!(private.get_context_id(), "private");
    assert_eq!(group.get_context_id(), "group");
    assert_eq!(channel.get_context_id(), "channel");
}