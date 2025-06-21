//! Tests for kv_standards module
//! Extracted from src/utils/kv_standards.rs

#[cfg(test)]
mod tests {
    use cerebrum_ai::utils::kv_standards::*;

    #[test]
    fn test_key_builder() {
        let key = KvKeyBuilder::new(KeyPrefix::UserProfile)
            .add_component("user123")
            .add_component("preferences")
            .build();

        assert_eq!(key, "user_profile:user123:preferences");
    }

    #[test]
    fn test_cache_metadata_expiration() {
        let metadata = CacheMetadata::new(CacheTTL::Short, 100, "test_service".to_string());
        assert!(!metadata.is_expired());

        // Test with expired metadata
        let mut expired_metadata = metadata.clone();
        expired_metadata.expires_at = 0; // Set to past
        assert!(expired_metadata.is_expired());
    }

    #[test]
    fn test_cached_data_validity() {
        let data = "test_data".to_string();
        let cached = CachedData::new(data, CacheTTL::Short, "test_service".to_string());
        assert!(cached.is_valid());
    }

    #[test]
    fn test_ttl_values() {
        let config = TtlConfig::default();
        assert_eq!(CacheTTL::RealTime.as_seconds(&config), 30);
        assert_eq!(CacheTTL::Short.as_seconds(&config), 300);
        assert_eq!(CacheTTL::Medium.as_seconds(&config), 3600);
        assert_eq!(CacheTTL::Long.as_seconds(&config), 86400);
        assert_eq!(CacheTTL::VeryLong.as_seconds(&config), 604800);
    }

    #[test]
    fn test_key_prefixes() {
        assert_eq!(KeyPrefix::UserProfile.as_str(), "user_profile");
        assert_eq!(KeyPrefix::Position.as_str(), "positions");
        assert_eq!(KeyPrefix::MarketData.as_str(), "market_data");
    }
}