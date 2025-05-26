// Task 1.5: Trading Focus & Automation Preferences - Unit Tests
// Tests for user trading preferences service and validation logic

use arb_edge::services::core::user::user_trading_preferences::*;

#[tokio::test]
async fn test_default_user_trading_preferences() {
    let preferences = UserTradingPreferences::new_default("user123".to_string());

    // Test safe defaults for new users
    assert_eq!(preferences.user_id, "user123");
    assert_eq!(preferences.preference_id, "pref_user123");
    assert_eq!(preferences.trading_focus, TradingFocus::Arbitrage);
    assert_eq!(preferences.automation_level, AutomationLevel::Manual);
    assert_eq!(preferences.automation_scope, AutomationScope::None);
    assert_eq!(preferences.experience_level, ExperienceLevel::Beginner);
    assert_eq!(preferences.risk_tolerance, RiskTolerance::Conservative);

    // Feature access defaults
    assert!(preferences.arbitrage_enabled);
    assert!(!preferences.technical_enabled);
    assert!(!preferences.advanced_analytics_enabled);
    assert!(!preferences.onboarding_completed);

    // Notification defaults
    assert_eq!(
        preferences.preferred_notification_channels,
        vec!["telegram"]
    );
    assert_eq!(preferences.trading_hours_timezone, "UTC");
    assert_eq!(preferences.trading_hours_start, "00:00");
    assert_eq!(preferences.trading_hours_end, "23:59");
}

#[tokio::test]
async fn test_trading_focus_validation() {
    let mut preferences = UserTradingPreferences::new_default("user123".to_string());

    // Beginners should not be able to enable technical trading
    assert!(preferences.enable_technical_trading().is_err());

    // Upgrade to intermediate should allow technical trading
    preferences.experience_level = ExperienceLevel::Intermediate;
    assert!(preferences.enable_technical_trading().is_ok());
    assert!(preferences.technical_enabled);

    // Advanced users should be able to enable technical trading
    preferences.experience_level = ExperienceLevel::Advanced;
    preferences.technical_enabled = false; // Reset
    assert!(preferences.enable_technical_trading().is_ok());
    assert!(preferences.technical_enabled);
}

#[tokio::test]
async fn test_automation_level_validation() {
    let mut preferences = UserTradingPreferences::new_default("user123".to_string());

    // Manual should always work
    assert!(preferences
        .set_automation_level(AutomationLevel::Manual, AutomationScope::None)
        .is_ok());

    // Semi-auto should fail for beginners
    assert!(preferences
        .set_automation_level(AutomationLevel::SemiAuto, AutomationScope::ArbitrageOnly)
        .is_err());

    // Should work for intermediate users
    preferences.experience_level = ExperienceLevel::Intermediate;
    assert!(preferences
        .set_automation_level(AutomationLevel::SemiAuto, AutomationScope::ArbitrageOnly)
        .is_ok());
    assert_eq!(preferences.automation_level, AutomationLevel::SemiAuto);
    assert_eq!(preferences.automation_scope, AutomationScope::ArbitrageOnly);

    // Full auto should fail for intermediate
    assert!(preferences
        .set_automation_level(AutomationLevel::FullAuto, AutomationScope::Both)
        .is_err());

    // Should work for advanced users
    preferences.experience_level = ExperienceLevel::Advanced;
    assert!(preferences
        .set_automation_level(AutomationLevel::FullAuto, AutomationScope::Both)
        .is_ok());
    assert_eq!(preferences.automation_level, AutomationLevel::FullAuto);
    assert_eq!(preferences.automation_scope, AutomationScope::Both);
}

#[tokio::test]
async fn test_feature_access_calculation() {
    let mut preferences = UserTradingPreferences::new_default("user123".to_string());
    preferences.experience_level = ExperienceLevel::Intermediate;
    preferences.technical_enabled = true;
    preferences.automation_level = AutomationLevel::SemiAuto;
    preferences.automation_scope = AutomationScope::ArbitrageOnly;
    preferences.advanced_analytics_enabled = true;

    let access = FeatureAccess::from_preferences(&preferences);

    // Basic access
    assert!(access.arbitrage_alerts);
    assert!(access.technical_alerts); // Technical enabled + not arbitrage-only focus

    // Automation access
    assert!(access.arbitrage_automation); // Semi-auto + arbitrage scope
    assert!(!access.technical_automation); // Technical not in automation scope

    // Experience-based access
    assert!(access.ai_integration); // Intermediate level
    assert!(access.priority_notifications); // Not beginner
    assert!(access.custom_indicators); // Technical enabled + not beginner

    // Explicit feature access
    assert!(access.advanced_analytics); // Explicitly enabled
}

#[tokio::test]
async fn test_feature_access_arbitrage_only() {
    let mut preferences = UserTradingPreferences::new_default("user123".to_string());
    preferences.trading_focus = TradingFocus::Arbitrage;
    preferences.experience_level = ExperienceLevel::Beginner;

    let access = FeatureAccess::from_preferences(&preferences);

    // Should have arbitrage access
    assert!(access.arbitrage_alerts);

    // Should not have technical features
    assert!(!access.technical_alerts);
    assert!(!access.technical_automation);
    assert!(!access.custom_indicators);

    // Should not have advanced features for beginners
    assert!(!access.ai_integration);
    assert!(!access.priority_notifications);
    assert!(!access.advanced_analytics);

    // Should not have automation features
    assert!(!access.arbitrage_automation);
}

#[tokio::test]
async fn test_hybrid_focus_feature_access() {
    let mut preferences = UserTradingPreferences::new_default("user123".to_string());
    preferences.trading_focus = TradingFocus::Hybrid;
    preferences.experience_level = ExperienceLevel::Advanced;
    preferences.technical_enabled = true;
    preferences.automation_level = AutomationLevel::FullAuto;
    preferences.automation_scope = AutomationScope::Both;
    preferences.advanced_analytics_enabled = true;

    let access = FeatureAccess::from_preferences(&preferences);

    // Should have full access to all features
    assert!(access.arbitrage_alerts);
    assert!(access.technical_alerts);
    assert!(access.arbitrage_automation);
    assert!(access.technical_automation);
    assert!(access.advanced_analytics);
    assert!(access.priority_notifications);
    assert!(access.ai_integration);
    assert!(access.custom_indicators);
}

#[tokio::test]
async fn test_trading_focus_serialization() {
    // Test JSON serialization/deserialization
    let focus = TradingFocus::Arbitrage;
    let json = serde_json::to_string(&focus).unwrap();
    assert_eq!(json, "\"arbitrage\"");

    let deserialized: TradingFocus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, TradingFocus::Arbitrage);

    // Test all variants
    let test_cases = vec![
        (TradingFocus::Arbitrage, "\"arbitrage\""),
        (TradingFocus::Technical, "\"technical\""),
        (TradingFocus::Hybrid, "\"hybrid\""),
    ];

    for (focus, expected_json) in test_cases {
        let json = serde_json::to_string(&focus).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: TradingFocus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, focus);
    }
}

#[tokio::test]
async fn test_automation_level_serialization() {
    let test_cases = vec![
        (AutomationLevel::Manual, "\"manual\""),
        (AutomationLevel::SemiAuto, "\"semi_auto\""),
        (AutomationLevel::FullAuto, "\"full_auto\""),
    ];

    for (level, expected_json) in test_cases {
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: AutomationLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, level);
    }
}

#[tokio::test]
async fn test_automation_scope_serialization() {
    let test_cases = vec![
        (AutomationScope::None, "\"none\""),
        (AutomationScope::ArbitrageOnly, "\"arbitrage_only\""),
        (AutomationScope::TechnicalOnly, "\"technical_only\""),
        (AutomationScope::Both, "\"both\""),
    ];

    for (scope, expected_json) in test_cases {
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: AutomationScope = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, scope);
    }
}

#[tokio::test]
async fn test_experience_level_serialization() {
    let test_cases = vec![
        (ExperienceLevel::Beginner, "\"beginner\""),
        (ExperienceLevel::Intermediate, "\"intermediate\""),
        (ExperienceLevel::Advanced, "\"advanced\""),
    ];

    for (level, expected_json) in test_cases {
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: ExperienceLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, level);
    }
}

#[tokio::test]
async fn test_risk_tolerance_serialization() {
    let test_cases = vec![
        (RiskTolerance::Conservative, "\"conservative\""),
        (RiskTolerance::Balanced, "\"balanced\""),
        (RiskTolerance::Aggressive, "\"aggressive\""),
    ];

    for (tolerance, expected_json) in test_cases {
        let json = serde_json::to_string(&tolerance).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: RiskTolerance = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tolerance);
    }
}

#[tokio::test]
async fn test_preferences_complete_serialization() {
    let preferences = UserTradingPreferences::new_default("user123".to_string());

    // Test full serialization
    let json = serde_json::to_string(&preferences).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: UserTradingPreferences = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, preferences.user_id);
    assert_eq!(deserialized.trading_focus, preferences.trading_focus);
    assert_eq!(deserialized.automation_level, preferences.automation_level);
    assert_eq!(deserialized.experience_level, preferences.experience_level);
    assert_eq!(deserialized.risk_tolerance, preferences.risk_tolerance);
}
