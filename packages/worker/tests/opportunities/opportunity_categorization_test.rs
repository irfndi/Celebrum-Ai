// Tests for opportunity categorization service

use cerebrum_ai::services::core::opportunities::opportunity_categorization::*;

#[test]
fn test_opportunity_category_display_names() {
    assert_eq!(
        OpportunityCategory::LowRiskArbitrage.display_name(),
        "Low Risk Arbitrage"
    );
    assert_eq!(
        OpportunityCategory::TechnicalSignals.display_name(),
        "Technical Signals"
    );
    assert_eq!(
        OpportunityCategory::BeginnerFriendly.display_name(),
        "Beginner Friendly"
    );
}

#[test]
fn test_category_risk_assessment() {
    assert_eq!(
        OpportunityCategory::LowRiskArbitrage.risk_assessment(),
        RiskLevel::Low
    );
    assert_eq!(
        OpportunityCategory::MomentumTrading.risk_assessment(),
        RiskLevel::High
    );
    assert_eq!(
        OpportunityCategory::TechnicalSignals.risk_assessment(),
        RiskLevel::Medium
    );
}

#[test]
fn test_experience_level_suitability() {
    let beginner_suitable = OpportunityCategory::BeginnerFriendly;
    let advanced_only = OpportunityCategory::AdvancedStrategies;

    assert!(beginner_suitable.is_suitable_for_experience(&ExperienceLevel::Beginner));
    assert!(!advanced_only.is_suitable_for_experience(&ExperienceLevel::Beginner));
    assert!(advanced_only.is_suitable_for_experience(&ExperienceLevel::Advanced));
}

#[test]
fn test_risk_indicator_creation() {
    let risk_indicator = RiskIndicator::new(RiskLevel::Low, 0.9);
    assert_eq!(risk_indicator.risk_level, RiskLevel::Low);
    assert!(risk_indicator.risk_score < 50.0); // Low risk should have low score
    assert_eq!(risk_indicator.volatility_assessment, "Low");
}

#[test]
fn test_alert_priority_levels() {
    let low_priority = AlertPriority::Low;
    let critical_priority = AlertPriority::Critical;
    assert_ne!(low_priority, critical_priority);
}

#[test]
fn test_default_alert_config() {
    let default_config = CategoryAlertConfig::default();
    assert!(default_config.enabled);
    assert!(default_config.min_confidence_threshold > 0.0);
    assert!(default_config.cooldown_minutes > 0);
}

#[test]
fn test_global_alert_settings_default() {
    let settings = GlobalAlertSettings::default();
    assert!(settings.alerts_enabled);
    assert_eq!(settings.quiet_hours_start, "22:00");
    assert_eq!(settings.quiet_hours_end, "08:00");
    assert!(settings.max_total_alerts_per_hour > 0);
}

#[test]
fn test_personalization_settings_default() {
    let settings = PersonalizationSettings::default();
    assert!(settings.learn_from_interactions);
    assert!(!settings.preferred_trading_pairs.is_empty());
    assert!(settings.max_simultaneous_opportunities > 0);
    assert!(settings.diversity_preference >= 0.0 && settings.diversity_preference <= 1.0);
}

// Note: For unit testing, we focus on testing individual methods that don't require DB
// Integration tests would use actual D1Service and UserTradingPreferencesService instances
// In real tests, we'd use actual D1Service and UserTradingPreferencesService
// This test focuses on the categorization logic