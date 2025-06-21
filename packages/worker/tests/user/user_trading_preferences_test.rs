use crate::services::core::user::user_trading_preferences::*;

#[test]
fn test_default_preferences() {
    let preferences = UserTradingPreferences::new_default("test_user".to_string());

    assert_eq!(preferences.trading_focus, TradingFocus::Arbitrage);
    assert_eq!(preferences.automation_level, AutomationLevel::Manual);
    assert_eq!(preferences.automation_scope, AutomationScope::None);
    assert_eq!(preferences.experience_level, ExperienceLevel::Beginner);
    assert_eq!(preferences.risk_tolerance, RiskTolerance::Conservative);
    assert!(preferences.arbitrage_enabled);
    assert!(!preferences.technical_enabled);
    assert!(!preferences.advanced_analytics_enabled);
    assert!(!preferences.onboarding_completed);
}

#[test]
fn test_enable_technical_trading() {
    let mut preferences = UserTradingPreferences::new_default("test_user".to_string());

    // Should fail for beginners
    assert!(preferences.enable_technical_trading().is_err());

    // Should work for intermediate users
    preferences.experience_level = ExperienceLevel::Intermediate;
    assert!(preferences.enable_technical_trading().is_ok());
    assert!(preferences.technical_enabled);
}

#[test]
fn test_automation_level_validation() {
    let mut preferences = UserTradingPreferences::new_default("test_user".to_string());

    // Manual should always work
    assert!(preferences
        .set_automation_level(AutomationLevel::Manual, AutomationScope::None)
        .is_ok());

    // Semi-auto should fail for beginners
    assert!(preferences
        .set_automation_level(AutomationLevel::SemiAuto, AutomationScope::ArbitrageOnly)
        .is_err());

    // Should work for intermediate
    preferences.experience_level = ExperienceLevel::Intermediate;
    assert!(preferences
        .set_automation_level(AutomationLevel::SemiAuto, AutomationScope::ArbitrageOnly)
        .is_ok());

    // Full auto should fail for intermediate
    assert!(preferences
        .set_automation_level(AutomationLevel::FullAuto, AutomationScope::Both)
        .is_err());

    // Should work for advanced
    preferences.experience_level = ExperienceLevel::Advanced;
    assert!(preferences
        .set_automation_level(AutomationLevel::FullAuto, AutomationScope::Both)
        .is_ok());
}

#[test]
fn test_feature_access() {
    let mut preferences = UserTradingPreferences::new_default("test_user".to_string());
    preferences.experience_level = ExperienceLevel::Intermediate;
    preferences.technical_enabled = true;
    preferences.automation_level = AutomationLevel::SemiAuto;
    preferences.automation_scope = AutomationScope::ArbitrageOnly;

    let access = FeatureAccess::from_preferences(&preferences);

    assert!(access.arbitrage_alerts);
    assert!(access.technical_alerts);
    assert!(access.arbitrage_automation);
    assert!(!access.technical_automation); // Technical not in scope
    assert!(access.ai_integration); // Intermediate level
    assert!(access.custom_indicators); // Technical enabled + not beginner
}

// TODO: Implement integration test for preference validation with proper test environment