//! Tests for formatter utilities
//! Extracted from src/utils/formatter.rs

use crate::types::{ArbitrageOpportunity, ExchangeIdEnum, OpportunityCategory, RiskLevel};
use crate::utils::formatter::*;

#[test]
fn test_escape_markdown_v2() {
    assert_eq!(escape_markdown_v2("test_string"), "test\\_string");
    assert_eq!(escape_markdown_v2("test*bold*"), "test\\*bold\\*");
    assert_eq!(escape_markdown_v2("test-dash"), "test\\-dash");
}

#[test]
fn test_format_percentage() {
    assert_eq!(format_percentage(0.1234), "12.3400");
    assert_eq!(format_percentage(0.0001), "0.0100");
}

#[test]
fn test_category_emoji_mapping() {
    assert_eq!(
        get_category_emoji(&OpportunityCategory::LowRiskArbitrage),
        "🛡️"
    );
    assert_eq!(
        get_category_emoji(&OpportunityCategory::AiRecommended),
        "🤖"
    );
    assert_eq!(
        get_category_emoji(&OpportunityCategory::TechnicalSignals),
        "📊"
    );
}

#[test]
fn test_risk_emoji_mapping() {
    assert_eq!(get_risk_emoji(&RiskLevel::Low), "🟢");
    assert_eq!(get_risk_emoji(&RiskLevel::Medium), "🟡");
    assert_eq!(get_risk_emoji(&RiskLevel::High), "🔴");
}

#[test]
fn test_confidence_emoji() {
    assert_eq!(get_confidence_emoji(0.9), "🌟");
    assert_eq!(get_confidence_emoji(0.7), "⭐");
    assert_eq!(get_confidence_emoji(0.5), "✨");
    assert_eq!(get_confidence_emoji(0.2), "❓");
}

#[test]
#[ignore] // Skip this test for now due to WASM binding issues in test environment
fn test_format_opportunity_message() {
    let mut opportunity = ArbitrageOpportunity::new(
        "BTC/USDT".to_string(),
        ExchangeIdEnum::Binance,
        ExchangeIdEnum::Bybit,
        0.0001,  // rate_difference as f64
    );
    
    // Set a fixed timestamp to avoid WASM binding issues in tests
    opportunity.timestamp = 1234567890;
    
    let message = format_opportunity_message(&opportunity);
    assert!(message.contains("BTC/USDT"));
    assert!(message.contains("Binance"));
    assert!(message.contains("Bybit"));
}

#[test]
fn test_format_currency() {
    assert_eq!(format_currency(1234.56), "$1,234.56");
    assert_eq!(format_currency(0.01), "$0.01");
    assert_eq!(format_currency(1000000.0), "$1,000,000.00");
}

#[test]
fn test_format_decimal_places() {
    assert_eq!(format_decimal_places(1.23456, 2), "1.23");
    assert_eq!(format_decimal_places(1.23456, 4), "1.2346");
    assert_eq!(format_decimal_places(1.0, 2), "1.00");
}

#[test]
fn test_truncate_string() {
    assert_eq!(truncate_string("Hello World", 5), "Hello...");
    assert_eq!(truncate_string("Short", 10), "Short");
    assert_eq!(truncate_string("", 5), "");
}

#[test]
fn test_format_time_ago() {
    // Test relative time formatting
    let now = 1234567890;
    let minute_ago = now - 60;
    let hour_ago = now - 3600;
    let day_ago = now - 86400;
    
    assert!(format_time_ago(minute_ago, now).contains("minute"));
    assert!(format_time_ago(hour_ago, now).contains("hour"));
    assert!(format_time_ago(day_ago, now).contains("day"));
}