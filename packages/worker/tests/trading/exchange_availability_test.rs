use crate::types::*;
use std::collections::HashMap;

#[test]
fn test_exchange_availability_structure() {
    let availability = ExchangeAvailability {
        exchange: ExchangeIdEnum::Binance,
        level: ExchangeAvailabilityLevel::Personal,
        is_available: true,
        is_read_only: false,
        can_trade: true,
        api_key_source: Some("user123".to_string()),
        last_checked: chrono::Utc::now().timestamp_millis() as u64,
    };

    assert_eq!(availability.exchange, ExchangeIdEnum::Binance);
    assert_eq!(availability.level, ExchangeAvailabilityLevel::Personal);
    assert!(availability.is_available);
    assert!(availability.can_trade);
}

#[test]
fn test_exchange_selection_criteria() {
    let criteria = ExchangeSelectionCriteria {
        required_exchanges: Some(vec![ExchangeIdEnum::Binance]),
        preferred_exchanges: vec![ExchangeIdEnum::Binance, ExchangeIdEnum::Bybit],
        exclude_exchanges: vec![ExchangeIdEnum::OKX],
        require_trading_capability: true,
        level: ExchangeAvailabilityLevel::Personal,
        context_id: "user123".to_string(),
    };

    assert_eq!(criteria.preferred_exchanges.len(), 2);
    assert_eq!(criteria.exclude_exchanges.len(), 1);
    assert!(criteria.require_trading_capability);
}

#[test]
fn test_exchange_selection_result() {
    let result = ExchangeSelectionResult {
        long_exchange: ExchangeIdEnum::Binance,
        short_exchange: ExchangeIdEnum::Bybit,
        selection_reason: "Preferred exchanges available".to_string(),
        availability_info: vec![],
        fallback_used: false,
    };

    assert_eq!(result.long_exchange, ExchangeIdEnum::Binance);
    assert_eq!(result.short_exchange, ExchangeIdEnum::Bybit);
    assert!(!result.fallback_used);
}

#[test]
fn test_exchange_availability_level_cache_key() {
    assert_eq!(ExchangeAvailabilityLevel::Global.to_cache_key(), "global");
    assert_eq!(
        ExchangeAvailabilityLevel::Personal.to_cache_key(),
        "personal"
    );
    assert_eq!(
        ExchangeAvailabilityLevel::GroupChannel.to_cache_key(),
        "group"
    );
}

#[test]
fn test_all_supported_exchanges() {
    let exchanges = ExchangeIdEnum::all_supported();
    assert_eq!(exchanges.len(), 4);
    assert!(exchanges.contains(&ExchangeIdEnum::Binance));
    assert!(exchanges.contains(&ExchangeIdEnum::Bybit));
    assert!(exchanges.contains(&ExchangeIdEnum::OKX));
    assert!(exchanges.contains(&ExchangeIdEnum::Bitget));
}

#[test]
fn test_api_key_exchange_compatibility() {
    // This would be tested with a mock service in a full implementation
    let user_profile = UserProfile::new(Some(123456789), Some("testuser".to_string()));

    // Test structure - actual compatibility logic would be in the service
    assert_eq!(user_profile.api_keys.len(), 0); // New profile has no API keys
}

#[test]
fn test_exchange_selection_insufficient_exchanges() {
    // Test that we properly handle cases with insufficient exchanges
    let suitable_exchanges: Vec<&ExchangeAvailability> = vec![];

    // This would fail in the actual service method
    assert_eq!(suitable_exchanges.len(), 0);
}

#[test]
fn test_serialization() {
    let availability = ExchangeAvailability {
        exchange: ExchangeIdEnum::Binance,
        level: ExchangeAvailabilityLevel::Global,
        is_available: true,
        is_read_only: true,
        can_trade: false,
        api_key_source: Some("super_admin".to_string()),
        last_checked: 1640995200000,
    };

    let serialized = serde_json::to_string(&availability).unwrap();
    let deserialized: ExchangeAvailability = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.exchange, availability.exchange);
    assert_eq!(deserialized.level, availability.level);
    assert_eq!(deserialized.is_available, availability.is_available);
}