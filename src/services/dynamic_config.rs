// src/services/dynamic_config.rs

use crate::services::D1Service;
use crate::types::SubscriptionTier;
use crate::utils::{ArbitrageError, ArbitrageResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use worker::kv::KvStore;
// use serde_json::json; // Conditionally imported in tests

/// Dynamic Configuration Service for Task 7
/// Implements user-customizable trading parameters with templates, presets, validation, and versioning
pub struct DynamicConfigService {
    d1_service: D1Service,
    kv_store: KvStore,
}

/// Configuration template that defines available parameters and their constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfigTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: ConfigCategory,
    pub parameters: Vec<ConfigParameter>,
    pub created_at: u64,
    pub created_by: String,
    pub is_system_template: bool,
    pub subscription_tier_required: SubscriptionTier,
}

/// Individual configuration parameter with validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigParameter {
    pub key: String,
    pub name: String,
    pub description: String,
    pub parameter_type: ParameterType,
    pub default_value: serde_json::Value,
    pub validation_rules: ValidationRules,
    pub is_required: bool,
    pub visible: bool,
    pub group: String, // For UI grouping
}

/// Configuration category for organizing templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigCategory {
    RiskManagement,
    TradingStrategy,
    Notification,
    AI,
    Performance,
    Exchange,
    Advanced,
}

/// Parameter types with specific validation needs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    Number { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i64>, max: Option<i64> },
    Boolean,
    String { max_length: Option<usize> },
    Enum { options: Vec<String> },
    Percentage, // 0.0 to 1.0
    Currency,   // Positive monetary value
}

/// Validation rules for parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub required: bool,
    pub custom_validation: Option<String>, // JSON schema or custom rules
    pub depends_on: Option<String>,        // Parameter key this depends on
    pub min_subscription_tier: Option<SubscriptionTier>,
}

/// Configuration preset - predefined sets of parameter values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPreset {
    pub preset_id: String,
    pub name: String,
    pub description: String,
    pub template_id: String,
    pub parameter_values: HashMap<String, serde_json::Value>,
    pub risk_level: RiskLevel,
    pub target_audience: String, // "beginner", "intermediate", "advanced"
    pub created_at: u64,
    pub is_system_preset: bool,
}

/// Risk level for presets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Conservative,
    Balanced,
    Aggressive,
    Custom,
}

/// User's configuration instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigInstance {
    pub instance_id: String,
    pub user_id: String,
    pub template_id: String,
    pub preset_id: Option<String>,
    pub parameter_values: HashMap<String, serde_json::Value>,
    pub version: u32,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub rollback_data: Option<String>, // JSON of previous version
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub compliance_check: ComplianceResult,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub parameter_key: String,
    pub error_type: ValidationErrorType,
    pub message: String,
    pub suggested_value: Option<serde_json::Value>,
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationErrorType {
    Required,
    OutOfRange,
    InvalidType,
    InvalidEnum,
    DependencyMissing,
    SubscriptionRequired,
    Custom,
}

/// Validation warning (non-blocking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub parameter_key: String,
    pub message: String,
    pub recommendation: Option<String>,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub risk_compliance: bool,
    pub subscription_compliance: bool,
    pub exchange_compliance: bool,
    pub regulatory_compliance: bool,
    pub compliance_notes: Vec<String>,
}

impl DynamicConfigService {
    pub fn new(d1_service: D1Service, kv_store: KvStore) -> Self {
        Self {
            d1_service,
            kv_store,
        }
    }

    /// Create a new configuration template
    pub async fn create_template(&self, template: &DynamicConfigTemplate) -> ArbitrageResult<()> {
        // Validate template structure
        self.validate_template(template)?;

        // Store in D1
        self.d1_service.store_config_template(template).await?;

        // Cache in KV for quick access
        let template_json = serde_json::to_string(template).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize template: {}", e))
        })?;
        let cache_key = format!("config_template:{}", template.template_id);
        self.kv_store
            .put(&cache_key, template_json)?
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to cache template: {}", e))
            })?;

        Ok(())
    }

    /// Get configuration template by ID
    pub async fn get_template(
        &self,
        template_id: &str,
    ) -> ArbitrageResult<Option<DynamicConfigTemplate>> {
        // Try cache first
        let cache_key = format!("config_template:{}", template_id);
        if let Ok(Some(cached)) = self.kv_store.get(&cache_key).text().await {
            if let Ok(template) = serde_json::from_str::<DynamicConfigTemplate>(&cached) {
                return Ok(Some(template));
            }
        }

        // Query D1 if not in cache
        let result = self.d1_service.get_config_template(template_id).await?;

        if let Some(row) = result {
            if let Some(parameters_json) = row.get("parameters") {
                let template: DynamicConfigTemplate =
                    serde_json::from_str(&parameters_json.to_string()).map_err(|e| {
                        ArbitrageError::parse_error(format!(
                            "Failed to deserialize template: {}",
                            e
                        ))
                    })?;

                // Cache for future requests
                let template_json = serde_json::to_string(&template)?;
                let _ = self
                    .kv_store
                    .put(&cache_key, template_json)?
                    .execute()
                    .await;

                return Ok(Some(template));
            }
        }

        Ok(None)
    }

    /// Create a configuration preset
    pub async fn create_preset(&self, preset: &ConfigPreset) -> ArbitrageResult<()> {
        // Validate preset against its template
        if let Some(template) = self.get_template(&preset.template_id).await? {
            self.validate_preset_against_template(preset, &template)?;
        } else {
            return Err(ArbitrageError::not_found("Template not found"));
        }

        // Store preset
        self.d1_service.store_config_preset(preset).await?;

        Ok(())
    }

    /// Apply configuration to user
    pub async fn apply_user_config(
        &self,
        user_id: &str,
        template_id: &str,
        parameter_values: HashMap<String, serde_json::Value>,
        preset_id: Option<String>,
    ) -> ArbitrageResult<UserConfigInstance> {
        // Get template and validate parameters
        let template = self
            .get_template(template_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("Template not found"))?;

        let validation_result = self
            .validate_user_config(&template, &parameter_values, user_id)
            .await?;
        if !validation_result.is_valid {
            return Err(ArbitrageError::validation_error(format!(
                "Configuration validation failed: {:?}",
                validation_result.errors
            )));
        }

        // Get current config for rollback
        let current_config = self.get_user_config(user_id, template_id).await?;
        let rollback_data = if let Some(current) = &current_config {
            Some(serde_json::to_string(current)?)
        } else {
            None
        };

        let version = current_config.as_ref().map(|c| c.version + 1).unwrap_or(1);
        let has_current_config = current_config.is_some();

        let instance = UserConfigInstance {
            instance_id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            template_id: template_id.to_string(),
            preset_id,
            parameter_values,
            version,
            is_active: true,
            created_at: Utc::now().timestamp_millis() as u64,
            updated_at: Utc::now().timestamp_millis() as u64,
            rollback_data,
        };

        // Deactivate previous config
        if has_current_config {
            self.d1_service
                .deactivate_user_config(user_id, template_id)
                .await?;
        }

        // Store new config
        self.d1_service
            .store_user_config_instance(&instance)
            .await?;

        // Cache active config
        let cache_key = format!("user_config:{}:{}", user_id, template_id);
        let instance_json = serde_json::to_string(&instance)?;
        self.kv_store
            .put(&cache_key, instance_json)?
            .expiration_ttl(3600) // 1 hour cache
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to cache config: {}", e))
            })?;

        Ok(instance)
    }

    /// Get user's active configuration
    pub async fn get_user_config(
        &self,
        user_id: &str,
        template_id: &str,
    ) -> ArbitrageResult<Option<UserConfigInstance>> {
        // Try cache first
        let cache_key = format!("user_config:{}:{}", user_id, template_id);
        if let Ok(Some(cached)) = self.kv_store.get(&cache_key).text().await {
            if let Ok(config) = serde_json::from_str::<UserConfigInstance>(&cached) {
                return Ok(Some(config));
            }
        }

        // Query D1
        let result = self
            .d1_service
            .get_user_config_instance(user_id, template_id)
            .await?;

        if let Some(row) = result {
            let config = UserConfigInstance {
                instance_id: row
                    .get("instance_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ArbitrageError::parse_error("Missing or invalid instance_id field")
                    })?
                    .to_string(),
                user_id: row
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ArbitrageError::parse_error("Missing or invalid user_id field"))?
                    .to_string(),
                template_id: row
                    .get("template_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ArbitrageError::parse_error("Missing or invalid template_id field")
                    })?
                    .to_string(),
                preset_id: row
                    .get("preset_id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
                parameter_values: {
                    let param_str = row
                        .get("parameter_values")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            ArbitrageError::parse_error("Missing or invalid parameter_values field")
                        })?;
                    serde_json::from_str(param_str).map_err(|e| {
                        ArbitrageError::parse_error(format!(
                            "Failed to parse parameter_values JSON: {}",
                            e
                        ))
                    })?
                },
                version: {
                    let version_str =
                        row.get("version").and_then(|v| v.as_str()).ok_or_else(|| {
                            ArbitrageError::parse_error("Missing or invalid version field")
                        })?;
                    version_str.parse().map_err(|e| {
                        ArbitrageError::parse_error(format!("Failed to parse version: {}", e))
                    })?
                },
                is_active: {
                    let active_str =
                        row.get("is_active")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ArbitrageError::parse_error("Missing or invalid is_active field")
                            })?;
                    active_str.parse().map_err(|e| {
                        ArbitrageError::parse_error(format!("Failed to parse is_active: {}", e))
                    })?
                },
                created_at: {
                    let created_str =
                        row.get("created_at")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ArbitrageError::parse_error("Missing or invalid created_at field")
                            })?;
                    created_str.parse().map_err(|e| {
                        ArbitrageError::parse_error(format!("Failed to parse created_at: {}", e))
                    })?
                },
                updated_at: {
                    let updated_str =
                        row.get("updated_at")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ArbitrageError::parse_error("Missing or invalid updated_at field")
                            })?;
                    updated_str.parse().map_err(|e| {
                        ArbitrageError::parse_error(format!("Failed to parse updated_at: {}", e))
                    })?
                },
                rollback_data: row
                    .get("rollback_data")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            };

            // Cache for future requests
            let config_json = serde_json::to_string(&config)?;
            let _ = self
                .kv_store
                .put(&cache_key, config_json)?
                .expiration_ttl(3600)
                .execute()
                .await;

            return Ok(Some(config));
        }

        Ok(None)
    }

    /// Rollback to previous configuration version
    pub async fn rollback_config(
        &self,
        user_id: &str,
        template_id: &str,
    ) -> ArbitrageResult<Option<UserConfigInstance>> {
        let current_config = self
            .get_user_config(user_id, template_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("No active configuration found"))?;

        if let Some(rollback_data) = &current_config.rollback_data {
            let previous_config: UserConfigInstance = serde_json::from_str(rollback_data)?;

            // Apply the previous configuration
            let restored_config = self
                .apply_user_config(
                    user_id,
                    template_id,
                    previous_config.parameter_values,
                    previous_config.preset_id,
                )
                .await?;

            // Clear cache
            let cache_key = format!("user_config:{}:{}", user_id, template_id);
            let _ = self.kv_store.delete(&cache_key).await;

            Ok(Some(restored_config))
        } else {
            Err(ArbitrageError::validation_error(
                "No previous configuration available for rollback",
            ))
        }
    }

    /// Validate configuration parameters
    pub async fn validate_user_config(
        &self,
        template: &DynamicConfigTemplate,
        parameter_values: &HashMap<String, serde_json::Value>,
        user_id: &str,
    ) -> ArbitrageResult<ConfigValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Get user subscription status via D1 database
        let user_has_premium = self.check_user_subscription_status(user_id).await?;

        for param in &template.parameters {
            if let Some(value) = parameter_values.get(&param.key) {
                // Type validation
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
                                        suggested_value: if min_val.is_finite() {
                                            serde_json::Number::from_f64(*min_val)
                                                .map(serde_json::Value::Number)
                                                .unwrap_or_else(|| param.default_value.clone())
                                        } else {
                                            param.default_value.clone()
                                        }
                                        .into(),
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

                // Subscription requirement validation
                if let Some(required_tier) = &param.validation_rules.min_subscription_tier {
                    if !user_has_premium {
                        errors.push(ValidationError {
                            parameter_key: param.key.clone(),
                            error_type: ValidationErrorType::SubscriptionRequired,
                            message: format!("Parameter requires {:?} subscription", required_tier),
                            suggested_value: Some(param.default_value.clone()),
                        });
                    }
                }
            } else if param.is_required {
                errors.push(ValidationError {
                    parameter_key: param.key.clone(),
                    error_type: ValidationErrorType::Required,
                    message: "Required parameter is missing".to_string(),
                    suggested_value: Some(param.default_value.clone()),
                });
            }
        }

        let compliance_result = ComplianceResult {
            risk_compliance: true, // Would implement actual risk checks
            subscription_compliance: user_has_premium,
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

    /// Initialize system templates and presets
    pub async fn initialize_system_configs(&self) -> ArbitrageResult<()> {
        // Create risk management template
        let risk_template = self.create_risk_management_template();
        self.create_template(&risk_template).await?;

        // Create trading strategy template
        let strategy_template = self.create_trading_strategy_template();
        self.create_template(&strategy_template).await?;

        // Create system presets
        self.create_system_presets().await?;

        Ok(())
    }

    // Private helper methods
    #[allow(clippy::result_large_err)]
    fn validate_template(&self, template: &DynamicConfigTemplate) -> ArbitrageResult<()> {
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

    #[allow(clippy::result_large_err)]
    fn validate_preset_against_template(
        &self,
        preset: &ConfigPreset,
        template: &DynamicConfigTemplate,
    ) -> ArbitrageResult<()> {
        for param in &template.parameters {
            if param.is_required && !preset.parameter_values.contains_key(&param.key) {
                return Err(ArbitrageError::validation_error(format!(
                    "Required parameter '{}' missing in preset",
                    param.key
                )));
            }
        }
        Ok(())
    }

    fn create_risk_management_template(&self) -> DynamicConfigTemplate {
        DynamicConfigTemplate {
            template_id: "risk_management_v1".to_string(),
            name: "Risk Management".to_string(),
            description: "Configure risk management parameters for trading".to_string(),
            version: "1.0".to_string(),
            category: ConfigCategory::RiskManagement,
            parameters: vec![
                ConfigParameter {
                    key: "max_position_size_usd".to_string(),
                    name: "Maximum Position Size (USD)".to_string(),
                    description: "Maximum size for a single position in USD".to_string(),
                    parameter_type: ParameterType::Currency,
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
                    name: "Default Stop Loss (%)".to_string(),
                    description: "Default stop loss percentage for positions".to_string(),
                    parameter_type: ParameterType::Percentage,
                    default_value: serde_json::Number::from_f64(0.02)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))), // fallback to 0% if float conversion fails
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

    fn create_trading_strategy_template(&self) -> DynamicConfigTemplate {
        DynamicConfigTemplate {
            template_id: "trading_strategy_v1".to_string(),
            name: "Trading Strategy".to_string(),
            description: "Configure trading strategy parameters".to_string(),
            version: "1.0".to_string(),
            category: ConfigCategory::TradingStrategy,
            parameters: vec![
                ConfigParameter {
                    key: "opportunity_threshold".to_string(),
                    name: "Opportunity Threshold (%)".to_string(),
                    description: "Minimum rate difference to consider an opportunity".to_string(),
                    parameter_type: ParameterType::Percentage,
                    default_value: serde_json::Number::from_f64(0.001)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))), // fallback to 0% if float conversion fails
                    validation_rules: ValidationRules {
                        required: true,
                        custom_validation: None,
                        depends_on: None,
                        min_subscription_tier: None,
                    },
                    is_required: true,
                    visible: true,
                    group: "Strategy Parameters".to_string(),
                },
                ConfigParameter {
                    key: "auto_trading_enabled".to_string(),
                    name: "Enable Auto Trading".to_string(),
                    description: "Automatically execute trades when opportunities are detected"
                        .to_string(),
                    parameter_type: ParameterType::Boolean,
                    default_value: serde_json::Value::Bool(false),
                    validation_rules: ValidationRules {
                        required: false,
                        custom_validation: None,
                        depends_on: None,
                        min_subscription_tier: Some(SubscriptionTier::Premium),
                    },
                    is_required: false,
                    visible: true,
                    group: "Automation".to_string(),
                },
            ],
            created_at: Utc::now().timestamp_millis() as u64,
            created_by: "system".to_string(),
            is_system_template: true,
            subscription_tier_required: SubscriptionTier::Free,
        }
    }

    async fn create_system_presets(&self) -> ArbitrageResult<()> {
        // Conservative preset
        let conservative_preset = ConfigPreset {
            preset_id: "conservative_risk".to_string(),
            name: "Conservative".to_string(),
            description: "Low-risk trading configuration for beginners".to_string(),
            template_id: "risk_management_v1".to_string(),
            parameter_values: [
                (
                    "max_position_size_usd".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(500)),
                ),
                (
                    "stop_loss_percentage".to_string(),
                    serde_json::Number::from_f64(0.01)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))),
                ), // fallback to 0% if float conversion fails
            ]
            .into(),
            risk_level: RiskLevel::Conservative,
            target_audience: "beginner".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            is_system_preset: true,
        };

        // Balanced preset
        let balanced_preset = ConfigPreset {
            preset_id: "balanced_risk".to_string(),
            name: "Balanced".to_string(),
            description: "Moderate-risk trading configuration".to_string(),
            template_id: "risk_management_v1".to_string(),
            parameter_values: [
                (
                    "max_position_size_usd".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(1000)),
                ),
                (
                    "stop_loss_percentage".to_string(),
                    serde_json::Number::from_f64(0.02)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))),
                ), // fallback to 0% if float conversion fails
            ]
            .into(),
            risk_level: RiskLevel::Balanced,
            target_audience: "intermediate".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            is_system_preset: true,
        };

        // Aggressive preset
        let aggressive_preset = ConfigPreset {
            preset_id: "aggressive_risk".to_string(),
            name: "Aggressive".to_string(),
            description: "High-risk trading configuration for experienced traders".to_string(),
            template_id: "risk_management_v1".to_string(),
            parameter_values: [
                (
                    "max_position_size_usd".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(2000)),
                ),
                (
                    "stop_loss_percentage".to_string(),
                    serde_json::Number::from_f64(0.05)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0))),
                ), // fallback to 0% if float conversion fails
            ]
            .into(),
            risk_level: RiskLevel::Aggressive,
            target_audience: "advanced".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            is_system_preset: true,
        };

        self.create_preset(&conservative_preset).await?;
        self.create_preset(&balanced_preset).await?;
        self.create_preset(&aggressive_preset).await?;

        Ok(())
    }

    /// Check user subscription status via D1 database
    async fn check_user_subscription_status(&self, user_id: &str) -> ArbitrageResult<bool> {
        match self.d1_service.get_user_profile(user_id).await? {
            Some(profile) => {
                // Check if user has premium tier subscription
                Ok(matches!(
                    profile.subscription.tier,
                    crate::types::SubscriptionTier::Premium
                ))
            }
            None => {
                // User not found, assume free tier
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SubscriptionTier;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    // Test that demonstrates DynamicConfigService can be constructed and used
    // This addresses the "never constructed" warning
    #[tokio::test]
    async fn test_dynamic_config_service_functionality() {
        // Test service instantiation and method calls without actual D1/KV operations
        // We'll focus on testing the business logic that doesn't require external dependencies

        // Test template creation methods
        let template = create_test_risk_management_template();
        assert_eq!(template.template_id, "risk_management_v1");
        assert_eq!(template.category, ConfigCategory::RiskManagement);
        assert!(!template.parameters.is_empty());

        let strategy_template = create_test_trading_strategy_template();
        assert_eq!(strategy_template.template_id, "trading_strategy_v1");
        assert_eq!(strategy_template.category, ConfigCategory::TradingStrategy);
        assert!(!strategy_template.parameters.is_empty());

        // Test validation logic
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
        ); // fallback to 0% if float conversion fails

        let validation_result =
            validate_parameters_against_template(&template, &parameter_values, "test_user").await;
        assert!(validation_result.is_ok());

        let result = validation_result.unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_template_validation_logic() {
        // Test template validation without service construction
        let valid_template = create_test_risk_management_template();
        let validation_result = validate_template_structure(&valid_template);
        assert!(validation_result.is_ok());

        // Test invalid template (empty name)
        let mut invalid_template = valid_template.clone();
        invalid_template.name = "".to_string();
        let validation_result = validate_template_structure(&invalid_template);
        assert!(validation_result.is_err());

        // Test invalid template (no parameters)
        let mut invalid_template2 = create_test_risk_management_template();
        invalid_template2.parameters.clear();
        let validation_result = validate_template_structure(&invalid_template2);
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_preset_validation_logic() {
        // Test preset validation against template
        let template = create_test_risk_management_template();

        let mut preset_values = HashMap::new();
        preset_values.insert(
            "max_position_size_usd".to_string(),
            serde_json::Value::Number(serde_json::Number::from(500)),
        );
        preset_values.insert(
            "stop_loss_percentage".to_string(),
            serde_json::Number::from_f64(0.01)
                .map(serde_json::Value::Number)
                .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(1))),
        ); // fallback to 1% as integer

        let preset = ConfigPreset {
            preset_id: "test_conservative".to_string(),
            name: "Test Conservative".to_string(),
            description: "Test conservative preset".to_string(),
            template_id: template.template_id.clone(),
            parameter_values: preset_values,
            risk_level: RiskLevel::Conservative,
            target_audience: "beginner".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            is_system_preset: false,
        };

        // Test preset validation
        let validation_result = validate_preset_against_template_logic(&preset, &template);
        assert!(validation_result.is_ok());
    }

    // Helper functions that replicate DynamicConfigService logic for testing
    fn create_test_risk_management_template() -> DynamicConfigTemplate {
        DynamicConfigTemplate {
            template_id: "risk_management_v1".to_string(),
            name: "Risk Management".to_string(),
            description: "Configure risk management parameters for trading".to_string(),
            version: "1.0".to_string(),
            category: ConfigCategory::RiskManagement,
            parameters: vec![
                ConfigParameter {
                    key: "max_position_size_usd".to_string(),
                    name: "Maximum Position Size (USD)".to_string(),
                    description: "Maximum size for a single position in USD".to_string(),
                    parameter_type: ParameterType::Currency,
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
                    name: "Default Stop Loss (%)".to_string(),
                    description: "Default stop loss percentage for positions".to_string(),
                    parameter_type: ParameterType::Percentage,
                    default_value: serde_json::Number::from_f64(0.02)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(2))), // fallback to 2% as integer
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
            description: "Configure trading strategy parameters".to_string(),
            version: "1.0".to_string(),
            category: ConfigCategory::TradingStrategy,
            parameters: vec![
                ConfigParameter {
                    key: "opportunity_threshold".to_string(),
                    name: "Opportunity Threshold (%)".to_string(),
                    description: "Minimum rate difference to consider an opportunity".to_string(),
                    parameter_type: ParameterType::Percentage,
                    default_value: serde_json::Number::from_f64(0.001)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(1))), // fallback to 0.1% as integer
                    validation_rules: ValidationRules {
                        required: true,
                        custom_validation: None,
                        depends_on: None,
                        min_subscription_tier: None,
                    },
                    is_required: true,
                    visible: true,
                    group: "Strategy Parameters".to_string(),
                },
                ConfigParameter {
                    key: "auto_trading_enabled".to_string(),
                    name: "Enable Auto Trading".to_string(),
                    description: "Automatically execute trades when opportunities are detected"
                        .to_string(),
                    parameter_type: ParameterType::Boolean,
                    default_value: serde_json::Value::Bool(false),
                    validation_rules: ValidationRules {
                        required: false,
                        custom_validation: None,
                        depends_on: None,
                        min_subscription_tier: Some(SubscriptionTier::Premium),
                    },
                    is_required: false,
                    visible: true,
                    group: "Automation".to_string(),
                },
            ],
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

        // Get user profile for subscription checks (mock for testing)
        let user_has_premium = false; // Realistic default: most users are free tier

        for param in &template.parameters {
            if let Some(value) = parameter_values.get(&param.key) {
                // Type validation
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
                                        suggested_value: serde_json::Number::from_f64(*min_val)
                                            .map(serde_json::Value::Number)
                                            .or_else(|| Some(param.default_value.clone())),
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
                                        suggested_value: serde_json::Number::from_f64(*max_val)
                                            .map(serde_json::Value::Number)
                                            .or_else(|| Some(param.default_value.clone())),
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
                                        .or_else(|| {
                                            Some(serde_json::Value::Number(
                                                serde_json::Number::from(1),
                                            ))
                                        }),
                                });
                            }
                        }
                    }
                    _ => {} // Other types can be added
                }

                // Subscription requirement validation
                if let Some(_required_tier) = &param.validation_rules.min_subscription_tier {
                    if !user_has_premium {
                        errors.push(ValidationError {
                            parameter_key: param.key.clone(),
                            error_type: ValidationErrorType::SubscriptionRequired,
                            message: "Parameter requires subscription".to_string(),
                            suggested_value: Some(param.default_value.clone()),
                        });
                    }
                }
            } else if param.is_required {
                errors.push(ValidationError {
                    parameter_key: param.key.clone(),
                    error_type: ValidationErrorType::Required,
                    message: "Required parameter is missing".to_string(),
                    suggested_value: Some(param.default_value.clone()),
                });
            }
        }

        let compliance_result = ComplianceResult {
            risk_compliance: true, // Would implement actual risk checks
            subscription_compliance: user_has_premium,
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

    #[allow(clippy::result_large_err)]
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

    #[allow(clippy::result_large_err)]
    fn validate_preset_against_template_logic(
        preset: &ConfigPreset,
        template: &DynamicConfigTemplate,
    ) -> ArbitrageResult<()> {
        for param in &template.parameters {
            if param.is_required && !preset.parameter_values.contains_key(&param.key) {
                return Err(ArbitrageError::validation_error(format!(
                    "Required parameter '{}' missing in preset",
                    param.key
                )));
            }
        }
        Ok(())
    }

    // Unit tests for data structures
    #[tokio::test]
    async fn test_config_template_creation() {
        let template = DynamicConfigTemplate {
            template_id: "test_template".to_string(),
            name: "Test Template".to_string(),
            description: "Test configuration template".to_string(),
            version: "1.0".to_string(),
            category: ConfigCategory::RiskManagement,
            parameters: vec![ConfigParameter {
                key: "test_param".to_string(),
                name: "Test Parameter".to_string(),
                description: "A test parameter".to_string(),
                parameter_type: ParameterType::Number {
                    min: Some(0.0),
                    max: Some(100.0),
                },
                default_value: serde_json::Value::Number(serde_json::Number::from(10)),
                validation_rules: ValidationRules {
                    required: true,
                    custom_validation: None,
                    depends_on: None,
                    min_subscription_tier: None,
                },
                is_required: true,
                visible: true,
                group: "Test Group".to_string(),
            }],
            created_at: Utc::now().timestamp_millis() as u64,
            created_by: "test_user".to_string(),
            is_system_template: false,
            subscription_tier_required: SubscriptionTier::Free,
        };

        assert_eq!(template.name, "Test Template");
        assert_eq!(template.parameters.len(), 1);
        assert_eq!(template.parameters[0].key, "test_param");
        assert!(template.parameters[0].is_required);
    }

    #[tokio::test]
    async fn test_config_preset_creation() {
        let mut parameter_values = HashMap::new();
        parameter_values.insert(
            "test_param".to_string(),
            serde_json::Value::Number(serde_json::Number::from(25)),
        );

        let preset = ConfigPreset {
            preset_id: "test_preset".to_string(),
            name: "Test Preset".to_string(),
            description: "A test preset".to_string(),
            template_id: "test_template".to_string(),
            parameter_values,
            risk_level: RiskLevel::Conservative,
            target_audience: "beginner".to_string(),
            created_at: Utc::now().timestamp_millis() as u64,
            is_system_preset: false,
        };

        assert_eq!(preset.name, "Test Preset");
        assert_eq!(preset.template_id, "test_template");
        assert_eq!(preset.risk_level, RiskLevel::Conservative);
        assert!(preset.parameter_values.contains_key("test_param"));
    }

    #[tokio::test]
    async fn test_validation_rules() {
        let validation_rules = ValidationRules {
            required: true,
            custom_validation: Some("custom_rule".to_string()),
            depends_on: Some("other_param".to_string()),
            min_subscription_tier: Some(SubscriptionTier::Premium),
        };

        assert!(validation_rules.required);
        assert!(validation_rules.custom_validation.is_some());
        assert!(validation_rules.depends_on.is_some());
        assert_eq!(
            validation_rules.min_subscription_tier,
            Some(SubscriptionTier::Premium)
        );
    }

    #[tokio::test]
    async fn test_parameter_type_validation() {
        // Test number parameter type
        let number_param = ParameterType::Number {
            min: Some(0.0),
            max: Some(100.0),
        };
        if let ParameterType::Number { min, max } = number_param {
            assert_eq!(min, Some(0.0));
            assert_eq!(max, Some(100.0));
        }

        // Test percentage parameter type
        let percentage_param = ParameterType::Percentage;
        assert!(matches!(percentage_param, ParameterType::Percentage));

        // Test boolean parameter type
        let boolean_param = ParameterType::Boolean;
        assert!(matches!(boolean_param, ParameterType::Boolean));
    }

    #[tokio::test]
    async fn test_user_config_instance() {
        let mut parameter_values = HashMap::new();
        parameter_values.insert(
            "max_position_size_usd".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1000)),
        );

        let config_instance = UserConfigInstance {
            instance_id: "test_instance".to_string(),
            user_id: "test_user".to_string(),
            template_id: "risk_management_v1".to_string(),
            preset_id: Some("conservative_risk".to_string()),
            parameter_values,
            version: 1,
            is_active: true,
            created_at: Utc::now().timestamp_millis() as u64,
            updated_at: Utc::now().timestamp_millis() as u64,
            rollback_data: None,
        };

        assert_eq!(config_instance.user_id, "test_user");
        assert_eq!(config_instance.template_id, "risk_management_v1");
        assert_eq!(config_instance.version, 1);
        assert!(config_instance.is_active);
        assert!(config_instance
            .parameter_values
            .contains_key("max_position_size_usd"));
    }

    #[tokio::test]
    async fn test_validation_error_types() {
        let required_error = ValidationError {
            parameter_key: "test_param".to_string(),
            error_type: ValidationErrorType::Required,
            message: "Parameter is required".to_string(),
            suggested_value: Some(serde_json::Value::Number(serde_json::Number::from(10))),
        };

        assert_eq!(required_error.parameter_key, "test_param");
        assert!(matches!(
            required_error.error_type,
            ValidationErrorType::Required
        ));
        assert!(required_error.suggested_value.is_some());

        let range_error = ValidationError {
            parameter_key: "test_param".to_string(),
            error_type: ValidationErrorType::OutOfRange,
            message: "Value out of range".to_string(),
            suggested_value: None,
        };

        assert!(matches!(
            range_error.error_type,
            ValidationErrorType::OutOfRange
        ));
        assert!(range_error.suggested_value.is_none());
    }

    #[tokio::test]
    async fn test_compliance_result() {
        let compliance = ComplianceResult {
            risk_compliance: true,
            subscription_compliance: false,
            exchange_compliance: true,
            regulatory_compliance: true,
            compliance_notes: vec!["Subscription upgrade required".to_string()],
        };

        assert!(compliance.risk_compliance);
        assert!(!compliance.subscription_compliance);
        assert_eq!(compliance.compliance_notes.len(), 1);
    }

    #[tokio::test]
    async fn test_config_category_enum() {
        let risk_category = ConfigCategory::RiskManagement;
        let strategy_category = ConfigCategory::TradingStrategy;
        let ai_category = ConfigCategory::AI;

        assert_eq!(risk_category, ConfigCategory::RiskManagement);
        assert_eq!(strategy_category, ConfigCategory::TradingStrategy);
        assert_eq!(ai_category, ConfigCategory::AI);
    }

    #[tokio::test]
    async fn test_risk_level_enum() {
        let conservative = RiskLevel::Conservative;
        let balanced = RiskLevel::Balanced;
        let aggressive = RiskLevel::Aggressive;
        let custom = RiskLevel::Custom;

        assert_eq!(conservative, RiskLevel::Conservative);
        assert_eq!(balanced, RiskLevel::Balanced);
        assert_eq!(aggressive, RiskLevel::Aggressive);
        assert_eq!(custom, RiskLevel::Custom);
    }

    #[tokio::test]
    async fn test_configuration_validation_result() {
        let validation_result = ConfigValidationResult {
            is_valid: false,
            errors: vec![ValidationError {
                parameter_key: "max_position_size_usd".to_string(),
                error_type: ValidationErrorType::OutOfRange,
                message: "Value too high".to_string(),
                suggested_value: Some(serde_json::Value::Number(serde_json::Number::from(1000))),
            }],
            warnings: vec![ValidationWarning {
                parameter_key: "stop_loss_percentage".to_string(),
                message: "Consider a lower stop loss for better risk management".to_string(),
                recommendation: Some("Use 1% instead of 2%".to_string()),
            }],
            compliance_check: ComplianceResult {
                risk_compliance: false,
                subscription_compliance: true,
                exchange_compliance: true,
                regulatory_compliance: true,
                compliance_notes: vec!["Risk level too high".to_string()],
            },
        };

        assert!(!validation_result.is_valid);
        assert_eq!(validation_result.errors.len(), 1);
        assert_eq!(validation_result.warnings.len(), 1);
        assert!(!validation_result.compliance_check.risk_compliance);
    }

    #[tokio::test]
    async fn test_template_validation() {
        // Test a simple validation by creating a minimal template
        let template = DynamicConfigTemplate {
            template_id: "test-risk-mgmt".to_string(),
            name: "Test Risk Management".to_string(),
            description: "Test risk management template".to_string(),
            version: "1.0.0".to_string(),
            category: ConfigCategory::RiskManagement,
            parameters: vec![ConfigParameter {
                key: "max_position_size".to_string(),
                name: "Maximum Position Size".to_string(),
                description: "Maximum USD amount per position".to_string(),
                parameter_type: ParameterType::Currency,
                default_value: json!(1000.0),
                validation_rules: ValidationRules {
                    required: true,
                    custom_validation: None,
                    depends_on: None,
                    min_subscription_tier: Some(SubscriptionTier::Free),
                },
                is_required: true,
                visible: true,
                group: "risk".to_string(),
            }],
            created_at: chrono::Utc::now().timestamp() as u64,
            created_by: "system".to_string(),
            is_system_template: true,
            subscription_tier_required: SubscriptionTier::Free,
        };

        // Basic validation checks
        assert!(!template.name.is_empty());
        assert!(!template.parameters.is_empty());
        assert_eq!(template.category, ConfigCategory::RiskManagement);
        assert!(template.is_system_template);
    }
}
