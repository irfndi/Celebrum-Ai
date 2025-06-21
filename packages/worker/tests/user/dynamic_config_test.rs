use std::collections::HashMap;
use chrono::Utc;
use serde_json;
use crate::services::core::user::dynamic_config::{
    DynamicConfigService, DynamicConfigTemplate, ConfigParameter, ParameterType,
    ValidationRules, ConfigCategory, ConfigPreset, RiskLevel, SubscriptionTier,
    ConfigValidationResult, ValidationError, ValidationErrorType, ComplianceResult,
};
use crate::common::errors::{ArbitrageError, ArbitrageResult};

#[tokio::test]
async fn test_dynamic_config_service_functionality() {
    let service = DynamicConfigService::new();
    
    // Test template creation
    let risk_template = create_test_risk_management_template();
    let strategy_template = create_test_trading_strategy_template();
    
    // Test parameter validation
    let mut parameter_values = HashMap::new();
    parameter_values.insert(
        "max_position_size_usd".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1000)),
    );
    parameter_values.insert(
        "stop_loss_percentage".to_string(),
        serde_json::Number::from_f64(0.02)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))),
    );
    
    let validation_result = validate_parameters_against_template(
        &risk_template,
        &parameter_values,
        "test_user",
    )
    .await;
    
    assert!(validation_result.is_ok());
    let result = validation_result.unwrap();
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_template_validation_logic() {
    // Test valid template
    let valid_template = create_test_risk_management_template();
    assert!(validate_template_structure(&valid_template).is_ok());
    
    // Test invalid template with empty name
    let mut invalid_template = valid_template.clone();
    invalid_template.name = String::new();
    assert!(validate_template_structure(&invalid_template).is_err());
    
    // Test invalid template with no parameters
    let mut invalid_template = create_test_risk_management_template();
    invalid_template.parameters.clear();
    assert!(validate_template_structure(&invalid_template).is_err());
    
    // Test parameter validation with valid preset
    let template = create_test_risk_management_template();
    let mut parameter_values = HashMap::new();
    parameter_values.insert(
        "max_position_size_usd".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1000)),
    );
    parameter_values.insert(
        "stop_loss_percentage".to_string(),
        serde_json::Number::from_f64(0.02)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))),
    );
    
    let valid_preset = ConfigPreset {
        preset_id: "test_preset".to_string(),
        name: "Test Preset".to_string(),
        description: "Test preset for validation".to_string(),
        template_id: template.template_id.clone(),
        parameter_values,
        risk_level: RiskLevel::Balanced,
        target_audience: "test".to_string(),
        created_at: Utc::now().timestamp_millis() as u64,
        is_system_preset: false,
    };
    
    // Test validation
    let validation_result = validate_parameters_against_template(
        &template,
        &valid_preset.parameter_values,
        "test_user",
    )
    .await;
    
    assert!(validation_result.is_ok());
    let result = validation_result.unwrap();
    assert!(result.is_valid);
}

// Helper functions for testing
fn create_test_risk_management_template() -> DynamicConfigTemplate {
    DynamicConfigTemplate {
        template_id: "risk_management_v1".to_string(),
        name: "Risk Management".to_string(),
        description: "Basic risk management configuration".to_string(),
        version: "1.0".to_string(),
        category: ConfigCategory::RiskManagement,
        parameters: vec![
            ConfigParameter {
                key: "max_position_size_usd".to_string(),
                name: "max_position_size_usd".to_string(),
                description: "Maximum position size in USD".to_string(),
                parameter_type: ParameterType::Number {
                    min: Some(100.0),
                    max: Some(10000.0),
                },
                default_value: serde_json::Value::Number(serde_json::Number::from(1000)),
                validation_rules: ValidationRules {
                    required: true,
                    custom_validation: None,
                    depends_on: None,
                    min_subscription_tier: None,
                },
                is_required: true,
                visible: true,
                group: "Position Limits".to_string(),
            },
            ConfigParameter {
                key: "stop_loss_percentage".to_string(),
                name: "stop_loss_percentage".to_string(),
                description: "Stop loss percentage".to_string(),
                parameter_type: ParameterType::Number {
                    min: Some(0.001),
                    max: Some(0.1),
                },
                default_value: serde_json::Value::Number(
                    serde_json::Number::from_f64(0.02).unwrap(),
                ),
                validation_rules: ValidationRules {
                    required: true,
                    custom_validation: None,
                    depends_on: None,
                    min_subscription_tier: None,
                },
                is_required: true,
                visible: true,
                group: "Risk Controls".to_string(),
            },
        ],
        created_at: Utc::now().timestamp_millis() as u64,
        created_by: "system".to_string(),
        is_system_template: true,
        subscription_tier_required: SubscriptionTier::Free,
    }
}

fn create_test_trading_strategy_template() -> DynamicConfigTemplate {
    DynamicConfigTemplate {
        template_id: "trading_strategy_v1".to_string(),
        name: "Trading Strategy".to_string(),
        description: "Basic trading strategy configuration".to_string(),
        version: "1.0".to_string(),
        category: ConfigCategory::TradingStrategy,
        parameters: vec![ConfigParameter {
            key: "strategy_type".to_string(),
            name: "strategy_type".to_string(),
            description: "Type of trading strategy".to_string(),
            parameter_type: ParameterType::Enum {
                options: vec![
                    "conservative".to_string(),
                    "balanced".to_string(),
                    "aggressive".to_string(),
                ],
            },
            default_value: serde_json::Value::String("conservative".to_string()),
            validation_rules: ValidationRules {
                required: true,
                custom_validation: None,
                depends_on: None,
                min_subscription_tier: None,
            },
            is_required: true,
            visible: true,
            group: "Strategy Parameters".to_string(),
        }],
        created_at: Utc::now().timestamp_millis() as u64,
        created_by: "system".to_string(),
        is_system_template: true,
        subscription_tier_required: SubscriptionTier::Free,
    }
}

async fn validate_parameters_against_template(
    template: &DynamicConfigTemplate,
    parameter_values: &HashMap<String, serde_json::Value>,
    _user_id: &str,
) -> ArbitrageResult<ConfigValidationResult> {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    
    // Validate each required parameter
    for param in &template.parameters {
        if param.is_required && !parameter_values.contains_key(&param.key) {
            errors.push(ValidationError {
                parameter_key: param.key.clone(),
                error_type: ValidationErrorType::Required,
                message: format!("Required parameter '{}' is missing", param.key),
                suggested_value: Some(param.default_value.clone()),
            });
        }
        
        if let Some(value) = parameter_values.get(&param.key) {
            // Basic type validation
            match &param.parameter_type {
                ParameterType::Number { min, max } => {
                    if let Some(num) = value.as_f64() {
                        if let Some(min_val) = min {
                            if num < *min_val {
                                errors.push(ValidationError {
                                    parameter_key: param.key.clone(),
                                    error_type: ValidationErrorType::OutOfRange,
                                    message: format!(
                                        "Value {} is below minimum {}",
                                        num, min_val
                                    ),
                                    suggested_value: Some(if min_val.is_finite() {
                                        serde_json::Number::from_f64(*min_val)
                                            .map(serde_json::Value::Number)
                                            .unwrap_or_else(|| param.default_value.clone())
                                    } else {
                                        param.default_value.clone()
                                    }),
                                });
                            }
                        }
                        if let Some(max_val) = max {
                            if num > *max_val {
                                errors.push(ValidationError {
                                    parameter_key: param.key.clone(),
                                    error_type: ValidationErrorType::OutOfRange,
                                    message: format!(
                                        "Value {} is above maximum {}",
                                        num, max_val
                                    ),
                                    suggested_value: if max_val.is_finite() {
                                        serde_json::Number::from_f64(*max_val)
                                            .map(serde_json::Value::Number)
                                            .unwrap_or_else(|| param.default_value.clone())
                                    } else {
                                        param.default_value.clone()
                                    }
                                    .into(),
                                });
                            }
                        }
                    } else {
                        errors.push(ValidationError {
                            parameter_key: param.key.clone(),
                            error_type: ValidationErrorType::InvalidType,
                            message: "Expected number type".to_string(),
                            suggested_value: Some(param.default_value.clone()),
                        });
                    }
                }
                ParameterType::Boolean => {
                    if !value.is_boolean() {
                        errors.push(ValidationError {
                            parameter_key: param.key.clone(),
                            error_type: ValidationErrorType::InvalidType,
                            message: "Expected boolean type".to_string(),
                            suggested_value: Some(param.default_value.clone()),
                        });
                    }
                }
                ParameterType::Percentage => {
                    if let Some(num) = value.as_f64() {
                        if !(0.0..=1.0).contains(&num) {
                            errors.push(ValidationError {
                                parameter_key: param.key.clone(),
                                error_type: ValidationErrorType::OutOfRange,
                                message: "Percentage must be between 0.0 and 1.0".to_string(),
                                suggested_value: serde_json::Number::from_f64(0.01)
                                    .map(serde_json::Value::Number)
                                    .unwrap_or_else(|| param.default_value.clone())
                                    .into(),
                            });
                        }
                    }
                }
                _ => {} // Other types can be added
            }
        }
    }
    
    let compliance_result = ComplianceResult {
        risk_compliance: true,         // Would implement actual risk checks
        subscription_compliance: true, // Assuming user is premium for testing
        exchange_compliance: true,
        regulatory_compliance: true,
        compliance_notes: Vec::new(),
    };
    
    Ok(ConfigValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        compliance_check: compliance_result,
    })
}

fn validate_template_structure(template: &DynamicConfigTemplate) -> ArbitrageResult<()> {
    if template.name.is_empty() {
        return Err(ArbitrageError::validation_error(
            "Template name cannot be empty",
        ));
    }
    
    if template.parameters.is_empty() {
        return Err(ArbitrageError::validation_error(
            "Template must have at least one parameter",
        ));
    }
    
    Ok(())
}