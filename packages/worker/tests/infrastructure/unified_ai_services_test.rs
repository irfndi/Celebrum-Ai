use crate::services::core::infrastructure::unified_ai_services::*;

#[test]
fn test_unified_ai_config_default() {
    let config = UnifiedAIConfig::default();
    assert!(config.cache_enabled);
    assert!(config.personalization_enabled);
    assert_eq!(config.default_model, "gpt-3.5-turbo");
}

#[test]
fn test_ai_request_creation() {
    let request = create_simple_ai_request(
        AIRequestType::TextGeneration,
        "Test prompt".to_string(),
        Some("user123".to_string()),
    );

    assert_eq!(request.request_type, AIRequestType::TextGeneration);
    assert_eq!(request.prompt, "Test prompt");
    assert_eq!(request.user_id, Some("user123".to_string()));
    assert!(request.personalize);
}

#[test]
fn test_cached_ai_request_creation() {
    let request = create_cached_ai_request(
        AIRequestType::Embedding,
        "Text to embed".to_string(),
        "cache_key_123".to_string(),
    );

    assert_eq!(request.request_type, AIRequestType::Embedding);
    assert_eq!(request.cache_key, Some("cache_key_123".to_string()));
    assert!(!request.personalize);
}

#[test]
fn test_ai_parameters_default() {
    let params = AIParameters::default();
    assert!(params.model.is_none());
    assert!(params.max_tokens.is_none());
    assert!(params.temperature.is_none());
}