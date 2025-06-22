use crate::utils::feature_flags::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_manager() {
        let manager = FeatureFlagManager::default();

        // Test enabled flag
        assert!(manager.is_enabled("opportunity_engine.enhanced_logging"));

        // Test disabled flag (non-existent)
        assert!(!manager.is_enabled("non_existent_flag"));

        // Test value retrieval
        let threshold: Option<f64> = manager.get_value("opportunity_engine.min_rate_threshold");
        assert_eq!(threshold, Some(0.05));
    }

    #[test]
    fn test_global_feature_flag_access() {
        assert!(is_feature_enabled("opportunity_engine.enhanced_logging").unwrap_or(false));

        let threshold = get_numeric_feature_value("opportunity_engine.min_rate_threshold", 0.1);
        assert_eq!(threshold, 0.05);
    }
}
