// Data Validator - Data Quality and Freshness Validation Component
// Provides comprehensive data validation, quality checks, and freshness verification

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data validation rules and criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field_name: String,
    pub rule_type: ValidationRuleType,
    pub required: bool,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<String>>,
    pub custom_validator: Option<String>,
}

/// Types of validation rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRuleType {
    Required,
    Numeric,
    String,
    Email,
    Url,
    DateTime,
    Boolean,
    Array,
    Object,
    Enum,
    Pattern,
    Custom,
    Range,
    Length,
    Positive,
    NonZero,
    Currency,
    Percentage,
    TradingPair,
    Price,
    Volume,
    Timestamp,
}

/// Data freshness requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessRule {
    pub data_type: String,
    pub max_age_seconds: u64,
    pub critical_threshold_seconds: u64,
    pub warning_threshold_seconds: u64,
    pub enable_staleness_check: bool,
    pub enable_drift_detection: bool,
    pub max_drift_percent: f64,
}

impl Default for FreshnessRule {
    fn default() -> Self {
        Self {
            data_type: "generic".to_string(),
            max_age_seconds: 300,            // 5 minutes
            critical_threshold_seconds: 600, // 10 minutes
            warning_threshold_seconds: 180,  // 3 minutes
            enable_staleness_check: true,
            enable_drift_detection: false,
            max_drift_percent: 5.0,
        }
    }
}

/// Validation result for individual fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationResult {
    pub field_name: String,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub value: Option<serde_json::Value>,
    pub rule_type: ValidationRuleType,
}

/// Overall validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub is_fresh: bool,
    pub overall_score: f32, // 0.0 to 1.0
    pub field_results: Vec<FieldValidationResult>,
    pub freshness_score: f32,
    pub quality_score: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub data_age_seconds: u64,
    pub validation_timestamp: u64,
    pub data_source: String,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: false,
            is_fresh: false,
            overall_score: 0.0,
            field_results: Vec::new(),
            freshness_score: 0.0,
            quality_score: 0.0,
            errors: Vec::new(),
            warnings: Vec::new(),
            data_age_seconds: 0,
            validation_timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data_source: "unknown".to_string(),
        }
    }
}

/// Data quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetrics {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub average_quality_score: f32,
    pub average_freshness_score: f32,
    pub stale_data_count: u64,
    pub invalid_data_count: u64,
    pub validation_errors_by_type: HashMap<String, u64>,
    pub data_sources_quality: HashMap<String, f32>,
    pub last_updated: u64,
}

impl Default for DataQualityMetrics {
    fn default() -> Self {
        Self {
            total_validations: 0,
            successful_validations: 0,
            failed_validations: 0,
            average_quality_score: 0.0,
            average_freshness_score: 0.0,
            stale_data_count: 0,
            invalid_data_count: 0,
            validation_errors_by_type: HashMap::new(),
            data_sources_quality: HashMap::new(),
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Configuration for DataValidator
#[derive(Debug, Clone)]
pub struct DataValidatorConfig {
    pub enable_strict_validation: bool,
    pub enable_freshness_check: bool,
    pub enable_quality_scoring: bool,
    pub enable_drift_detection: bool,
    pub default_freshness_threshold_seconds: u64,
    pub quality_score_threshold: f32,
    pub freshness_score_threshold: f32,
    pub enable_performance_tracking: bool,
    pub enable_detailed_logging: bool,
    pub max_validation_time_ms: u64,
}

impl Default for DataValidatorConfig {
    fn default() -> Self {
        Self {
            enable_strict_validation: true,
            enable_freshness_check: true,
            enable_quality_scoring: true,
            enable_drift_detection: false,
            default_freshness_threshold_seconds: 300, // 5 minutes
            quality_score_threshold: 0.8,             // 80%
            freshness_score_threshold: 0.7,           // 70%
            enable_performance_tracking: true,
            enable_detailed_logging: true,
            max_validation_time_ms: 5000, // 5 seconds
        }
    }
}

impl DataValidatorConfig {
    /// Create configuration optimized for high performance
    pub fn high_performance() -> Self {
        Self {
            enable_strict_validation: false,
            enable_drift_detection: false,
            enable_detailed_logging: false,
            max_validation_time_ms: 1000,   // 1 second
            quality_score_threshold: 0.6,   // 60%
            freshness_score_threshold: 0.5, // 50%
            ..Default::default()
        }
    }

    /// Create configuration optimized for data quality
    pub fn high_quality() -> Self {
        Self {
            enable_strict_validation: true,
            enable_drift_detection: true,
            enable_detailed_logging: true,
            max_validation_time_ms: 10000,           // 10 seconds
            quality_score_threshold: 0.95,           // 95%
            freshness_score_threshold: 0.9,          // 90%
            default_freshness_threshold_seconds: 60, // 1 minute
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> ArbitrageResult<()> {
        if !(0.0..=1.0).contains(&self.quality_score_threshold) {
            return Err(ArbitrageError::validation_error(
                "quality_score_threshold must be between 0.0 and 1.0",
            ));
        }
        if !(0.0..=1.0).contains(&self.freshness_score_threshold) {
            return Err(ArbitrageError::validation_error(
                "freshness_score_threshold must be between 0.0 and 1.0",
            ));
        }
        if self.max_validation_time_ms == 0 {
            return Err(ArbitrageError::validation_error(
                "max_validation_time_ms must be greater than 0",
            ));
        }
        Ok(())
    }
}

/// Data validator for quality and freshness validation
pub struct DataValidator {
    config: DataValidatorConfig,
    logger: crate::utils::logger::Logger,

    // Validation rules by data type
    validation_rules: std::sync::Arc<std::sync::Mutex<HashMap<String, Vec<ValidationRule>>>>,

    // Freshness rules by data type
    freshness_rules: std::sync::Arc<std::sync::Mutex<HashMap<String, FreshnessRule>>>,

    // Quality metrics tracking
    metrics: std::sync::Arc<std::sync::Mutex<DataQualityMetrics>>,

    // Performance tracking
    startup_time: u64,
}

impl DataValidator {
    /// Create new DataValidator instance
    pub fn new(mut config: DataValidatorConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        // Validate configuration
        config.validate()?;

        let validator = Self {
            config,
            logger,
            validation_rules: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            freshness_rules: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            metrics: std::sync::Arc::new(std::sync::Mutex::new(DataQualityMetrics::default())),
            startup_time: chrono::Utc::now().timestamp_millis() as u64,
        };

        // Initialize default rules
        validator.initialize_default_rules()?;

        validator.logger.info(&format!(
            "DataValidator initialized: strict_validation={}, freshness_check={}, quality_threshold={}",
            validator.config.enable_strict_validation,
            validator.config.enable_freshness_check,
            validator.config.quality_score_threshold
        ));

        Ok(validator)
    }

    /// Initialize default validation rules for common data types
    fn initialize_default_rules(&self) -> ArbitrageResult<()> {
        // Market data validation rules
        let market_data_rules = vec![
            ValidationRule {
                field_name: "symbol".to_string(),
                rule_type: ValidationRuleType::TradingPair,
                required: true,
                min_length: Some(3),
                max_length: Some(20),
                pattern: Some(r"^[A-Z]+[/-][A-Z]+$".to_string()),
                ..Default::default()
            },
            ValidationRule {
                field_name: "price".to_string(),
                rule_type: ValidationRuleType::Price,
                required: true,
                min_value: Some(0.0),
                ..Default::default()
            },
            ValidationRule {
                field_name: "volume".to_string(),
                rule_type: ValidationRuleType::Volume,
                required: true,
                min_value: Some(0.0),
                ..Default::default()
            },
            ValidationRule {
                field_name: "timestamp".to_string(),
                rule_type: ValidationRuleType::Timestamp,
                required: true,
                ..Default::default()
            },
        ];

        // Funding rates validation rules
        let funding_rates_rules = vec![
            ValidationRule {
                field_name: "symbol".to_string(),
                rule_type: ValidationRuleType::TradingPair,
                required: true,
                min_length: Some(3),
                max_length: Some(20),
                ..Default::default()
            },
            ValidationRule {
                field_name: "funding_rate".to_string(),
                rule_type: ValidationRuleType::Percentage,
                required: true,
                min_value: Some(-1.0),
                max_value: Some(1.0),
                ..Default::default()
            },
            ValidationRule {
                field_name: "next_funding_time".to_string(),
                rule_type: ValidationRuleType::Timestamp,
                required: true,
                ..Default::default()
            },
        ];

        // User data validation rules
        let user_data_rules = vec![
            ValidationRule {
                field_name: "user_id".to_string(),
                rule_type: ValidationRuleType::String,
                required: true,
                min_length: Some(1),
                max_length: Some(100),
                ..Default::default()
            },
            ValidationRule {
                field_name: "email".to_string(),
                rule_type: ValidationRuleType::Email,
                required: false,
                max_length: Some(255),
                ..Default::default()
            },
            ValidationRule {
                field_name: "balance".to_string(),
                rule_type: ValidationRuleType::Currency,
                required: false,
                min_value: Some(0.0),
                ..Default::default()
            },
        ];

        // Add rules to the validator
        if let Ok(mut rules) = self.validation_rules.lock() {
            rules.insert("market_data".to_string(), market_data_rules);
            rules.insert("funding_rates".to_string(), funding_rates_rules);
            rules.insert("user_data".to_string(), user_data_rules);
        }

        // Initialize default freshness rules
        let freshness_rules = vec![
            (
                "market_data",
                FreshnessRule {
                    data_type: "market_data".to_string(),
                    max_age_seconds: 60,             // 1 minute
                    critical_threshold_seconds: 300, // 5 minutes
                    warning_threshold_seconds: 30,   // 30 seconds
                    enable_staleness_check: true,
                    enable_drift_detection: true,
                    max_drift_percent: 2.0,
                },
            ),
            (
                "funding_rates",
                FreshnessRule {
                    data_type: "funding_rates".to_string(),
                    max_age_seconds: 900,             // 15 minutes
                    critical_threshold_seconds: 1800, // 30 minutes
                    warning_threshold_seconds: 600,   // 10 minutes
                    enable_staleness_check: true,
                    enable_drift_detection: false,
                    max_drift_percent: 5.0,
                },
            ),
            (
                "user_data",
                FreshnessRule {
                    data_type: "user_data".to_string(),
                    max_age_seconds: 3600,            // 1 hour
                    critical_threshold_seconds: 7200, // 2 hours
                    warning_threshold_seconds: 1800,  // 30 minutes
                    enable_staleness_check: true,
                    enable_drift_detection: false,
                    max_drift_percent: 10.0,
                },
            ),
        ];

        if let Ok(mut rules) = self.freshness_rules.lock() {
            for (data_type, rule) in freshness_rules {
                rules.insert(data_type.to_string(), rule);
            }
        }

        Ok(())
    }

    /// Add custom validation rule for a data type
    pub fn add_validation_rule(
        &self,
        data_type: String,
        rule: ValidationRule,
    ) -> ArbitrageResult<()> {
        if let Ok(mut rules) = self.validation_rules.lock() {
            rules.entry(data_type).or_insert_with(Vec::new).push(rule);
        }
        Ok(())
    }

    /// Add freshness rule for a data type
    pub fn add_freshness_rule(
        &self,
        data_type: String,
        rule: FreshnessRule,
    ) -> ArbitrageResult<()> {
        if let Ok(mut rules) = self.freshness_rules.lock() {
            rules.insert(data_type, rule);
        }
        Ok(())
    }

    /// Validate data with quality and freshness checks
    pub async fn validate_data(
        &self,
        data: &serde_json::Value,
        data_type: &str,
        data_source: &str,
        data_timestamp: Option<u64>,
    ) -> ArbitrageResult<ValidationResult> {
        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        let mut result = ValidationResult {
            data_source: data_source.to_string(),
            validation_timestamp: start_time,
            ..Default::default()
        };

        // Calculate data age
        if let Some(timestamp) = data_timestamp {
            result.data_age_seconds = (start_time - timestamp) / 1000;
        }

        // Perform field validation
        if let Ok(rules) = self.validation_rules.lock() {
            if let Some(validation_rules) = rules.get(data_type) {
                result.field_results = self.validate_fields(data, validation_rules).await?;
            }
        }

        // Calculate quality score
        result.quality_score = self.calculate_quality_score(&result.field_results);
        result.is_valid = result.quality_score >= self.config.quality_score_threshold;

        // Perform freshness validation
        if self.config.enable_freshness_check {
            result.freshness_score = self
                .calculate_freshness_score(data_type, result.data_age_seconds)
                .await;
            result.is_fresh = result.freshness_score >= self.config.freshness_score_threshold;
        } else {
            result.is_fresh = true;
            result.freshness_score = 1.0;
        }

        // Calculate overall score
        result.overall_score = if self.config.enable_freshness_check {
            (result.quality_score + result.freshness_score) / 2.0
        } else {
            result.quality_score
        };

        // Collect errors and warnings
        for field_result in &result.field_results {
            result.errors.extend(field_result.errors.clone());
            result.warnings.extend(field_result.warnings.clone());
        }

        // Add freshness warnings/errors
        if self.config.enable_freshness_check {
            if let Ok(freshness_rules) = self.freshness_rules.lock() {
                if let Some(rule) = freshness_rules.get(data_type) {
                    if result.data_age_seconds > rule.critical_threshold_seconds {
                        result.errors.push(format!(
                            "Data is critically stale: {} seconds old",
                            result.data_age_seconds
                        ));
                    } else if result.data_age_seconds > rule.warning_threshold_seconds {
                        result.warnings.push(format!(
                            "Data is stale: {} seconds old",
                            result.data_age_seconds
                        ));
                    }
                }
            }
        }

        // Record metrics
        self.record_validation_metrics(&result, data_type, start_time)
            .await;

        // Check validation timeout
        let validation_time = chrono::Utc::now().timestamp_millis() as u64 - start_time;
        if validation_time > self.config.max_validation_time_ms {
            self.logger.warn(&format!(
                "Validation took {} ms, exceeding limit of {} ms",
                validation_time, self.config.max_validation_time_ms
            ));
        }

        Ok(result)
    }

    /// Validate individual fields against rules
    async fn validate_fields(
        &self,
        data: &serde_json::Value,
        rules: &[ValidationRule],
    ) -> ArbitrageResult<Vec<FieldValidationResult>> {
        let mut results = Vec::new();

        for rule in rules {
            let mut field_result = FieldValidationResult {
                field_name: rule.field_name.clone(),
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                value: None,
                rule_type: rule.rule_type.clone(),
            };

            // Get field value
            let field_value = data.get(&rule.field_name);

            // Check if required field is present
            if rule.required && field_value.is_none() {
                field_result.is_valid = false;
                field_result
                    .errors
                    .push(format!("Required field '{}' is missing", rule.field_name));
                results.push(field_result);
                continue;
            }

            if let Some(value) = field_value {
                field_result.value = Some(value.clone());

                // Validate based on rule type
                match rule.rule_type {
                    ValidationRuleType::Numeric => {
                        if let Some(num) = value.as_f64() {
                            if let Some(min) = rule.min_value {
                                if num < min {
                                    field_result.is_valid = false;
                                    field_result
                                        .errors
                                        .push(format!("Value {} is below minimum {}", num, min));
                                }
                            }
                            if let Some(max) = rule.max_value {
                                if num > max {
                                    field_result.is_valid = false;
                                    field_result
                                        .errors
                                        .push(format!("Value {} is above maximum {}", num, max));
                                }
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Value is not a valid number".to_string());
                        }
                    }
                    ValidationRuleType::String => {
                        if let Some(str_val) = value.as_str() {
                            if let Some(min_len) = rule.min_length {
                                if str_val.len() < min_len {
                                    field_result.is_valid = false;
                                    field_result.errors.push(format!(
                                        "String length {} is below minimum {}",
                                        str_val.len(),
                                        min_len
                                    ));
                                }
                            }
                            if let Some(max_len) = rule.max_length {
                                if str_val.len() > max_len {
                                    field_result.is_valid = false;
                                    field_result.errors.push(format!(
                                        "String length {} is above maximum {}",
                                        str_val.len(),
                                        max_len
                                    ));
                                }
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Value is not a valid string".to_string());
                        }
                    }
                    ValidationRuleType::Email => {
                        if let Some(email) = value.as_str() {
                            if !self.is_valid_email(email) {
                                field_result.is_valid = false;
                                field_result.errors.push("Invalid email format".to_string());
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Email must be a string".to_string());
                        }
                    }
                    ValidationRuleType::Price => {
                        if let Some(price) = value.as_f64() {
                            if price <= 0.0 {
                                field_result.is_valid = false;
                                field_result
                                    .errors
                                    .push("Price must be positive".to_string());
                            }
                            if price > 1_000_000.0 {
                                field_result
                                    .warnings
                                    .push("Price seems unusually high".to_string());
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Price must be a number".to_string());
                        }
                    }
                    ValidationRuleType::Volume => {
                        if let Some(volume) = value.as_f64() {
                            if volume < 0.0 {
                                field_result.is_valid = false;
                                field_result
                                    .errors
                                    .push("Volume cannot be negative".to_string());
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Volume must be a number".to_string());
                        }
                    }
                    ValidationRuleType::TradingPair => {
                        if let Some(pair) = value.as_str() {
                            if !self.is_valid_trading_pair(pair) {
                                field_result.is_valid = false;
                                field_result
                                    .errors
                                    .push("Invalid trading pair format".to_string());
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Trading pair must be a string".to_string());
                        }
                    }
                    ValidationRuleType::Timestamp => {
                        if let Some(timestamp) = value.as_u64() {
                            let now = chrono::Utc::now().timestamp_millis() as u64;
                            if timestamp > now + 300000 {
                                // 5 minutes in future
                                field_result
                                    .warnings
                                    .push("Timestamp is in the future".to_string());
                            }
                            if timestamp < now - 86400000 {
                                // 24 hours ago
                                field_result
                                    .warnings
                                    .push("Timestamp is very old".to_string());
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Timestamp must be a number".to_string());
                        }
                    }
                    ValidationRuleType::Percentage => {
                        if let Some(pct) = value.as_f64() {
                            if let Some(min) = rule.min_value {
                                if pct < min {
                                    field_result.is_valid = false;
                                    field_result.errors.push(format!(
                                        "Percentage {} is below minimum {}",
                                        pct, min
                                    ));
                                }
                            }
                            if let Some(max) = rule.max_value {
                                if pct > max {
                                    field_result.is_valid = false;
                                    field_result.errors.push(format!(
                                        "Percentage {} is above maximum {}",
                                        pct, max
                                    ));
                                }
                            }
                        } else {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Percentage must be a number".to_string());
                        }
                    }
                    _ => {
                        // For other types, perform basic validation
                        if rule.required && value.is_null() {
                            field_result.is_valid = false;
                            field_result
                                .errors
                                .push("Required field cannot be null".to_string());
                        }
                    }
                }

                // Check pattern if specified
                if let Some(pattern) = &rule.pattern {
                    if let Some(str_val) = value.as_str() {
                        if !self.matches_pattern(str_val, pattern) {
                            field_result.is_valid = false;
                            field_result.errors.push(format!(
                                "Value does not match required pattern: {}",
                                pattern
                            ));
                        }
                    }
                }

                // Check allowed values if specified
                if let Some(allowed) = &rule.allowed_values {
                    if let Some(str_val) = value.as_str() {
                        if !allowed.contains(&str_val.to_string()) {
                            field_result.is_valid = false;
                            field_result.errors.push(format!(
                                "Value '{}' is not in allowed values: {:?}",
                                str_val, allowed
                            ));
                        }
                    }
                }
            }

            results.push(field_result);
        }

        Ok(results)
    }

    /// Calculate quality score based on field validation results
    fn calculate_quality_score(&self, field_results: &[FieldValidationResult]) -> f32 {
        if field_results.is_empty() {
            return 1.0;
        }

        let valid_fields = field_results.iter().filter(|r| r.is_valid).count();
        let total_fields = field_results.len();

        let base_score = valid_fields as f32 / total_fields as f32;

        // Reduce score for warnings
        let warning_count = field_results
            .iter()
            .map(|r| r.warnings.len())
            .sum::<usize>();
        let warning_penalty = (warning_count as f32 * 0.05).min(0.2); // Max 20% penalty

        (base_score - warning_penalty).max(0.0)
    }

    /// Calculate freshness score based on data age
    async fn calculate_freshness_score(&self, data_type: &str, data_age_seconds: u64) -> f32 {
        if let Ok(rules) = self.freshness_rules.lock() {
            if let Some(rule) = rules.get(data_type) {
                if data_age_seconds <= rule.warning_threshold_seconds {
                    return 1.0; // Perfect freshness
                } else if data_age_seconds <= rule.max_age_seconds {
                    // Linear decay from warning to max age
                    let decay_range = rule.max_age_seconds - rule.warning_threshold_seconds;
                    let age_in_range = data_age_seconds - rule.warning_threshold_seconds;
                    return 1.0 - (age_in_range as f32 / decay_range as f32) * 0.5;
                // 50% to 100%
                } else if data_age_seconds <= rule.critical_threshold_seconds {
                    // Further decay from max age to critical
                    let decay_range = rule.critical_threshold_seconds - rule.max_age_seconds;
                    let age_in_range = data_age_seconds - rule.max_age_seconds;
                    return 0.5 - (age_in_range as f32 / decay_range as f32) * 0.5;
                // 0% to 50%
                } else {
                    return 0.0; // Critically stale
                }
            }
        }

        // Default freshness calculation
        let default_max_age = self.config.default_freshness_threshold_seconds;
        if data_age_seconds <= default_max_age {
            1.0 - (data_age_seconds as f32 / default_max_age as f32) * 0.5
        } else {
            0.0
        }
    }

    /// Record validation metrics
    async fn record_validation_metrics(
        &self,
        result: &ValidationResult,
        _data_type: &str,
        _start_time: u64,
    ) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_validations += 1;

            if result.is_valid {
                metrics.successful_validations += 1;
            } else {
                metrics.failed_validations += 1;
                metrics.invalid_data_count += 1;
            }

            if !result.is_fresh {
                metrics.stale_data_count += 1;
            }

            // Update average scores
            let total = metrics.total_validations as f32;
            metrics.average_quality_score =
                (metrics.average_quality_score * (total - 1.0) + result.quality_score) / total;
            metrics.average_freshness_score =
                (metrics.average_freshness_score * (total - 1.0) + result.freshness_score) / total;

            // Update data source quality
            metrics
                .data_sources_quality
                .insert(result.data_source.clone(), result.overall_score);

            // Count validation errors by type
            for error in &result.errors {
                let error_type = self.categorize_error(error);
                *metrics
                    .validation_errors_by_type
                    .entry(error_type)
                    .or_insert(0) += 1;
            }

            metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
        }
    }

    /// Categorize error for metrics
    fn categorize_error(&self, error: &str) -> String {
        if error.contains("missing") || error.contains("required") {
            "missing_field".to_string()
        } else if error.contains("format") || error.contains("pattern") {
            "format_error".to_string()
        } else if error.contains("range") || error.contains("minimum") || error.contains("maximum")
        {
            "range_error".to_string()
        } else if error.contains("stale") || error.contains("old") {
            "freshness_error".to_string()
        } else {
            "other".to_string()
        }
    }

    /// Validate email format
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    /// Validate trading pair format
    fn is_valid_trading_pair(&self, pair: &str) -> bool {
        pair.contains('/') || pair.contains('-') || pair.contains('_')
    }

    /// Check if string matches pattern (simplified regex)
    fn matches_pattern(&self, value: &str, pattern: &str) -> bool {
        // Simplified pattern matching - in a real implementation, use regex crate
        if pattern == r"^[A-Z]+[/-][A-Z]+$" {
            return self.is_valid_trading_pair(value)
                && value
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '/' || c == '-');
        }
        true // Default to true for other patterns
    }

    /// Get validation metrics
    pub async fn get_metrics(&self) -> DataQualityMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            DataQualityMetrics::default()
        }
    }

    /// Health check for data validator
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        let metrics = self.get_metrics().await;

        // Consider healthy if success rate is above 80%
        if metrics.total_validations > 0 {
            let success_rate =
                metrics.successful_validations as f32 / metrics.total_validations as f32;
            Ok(success_rate >= 0.8)
        } else {
            Ok(true) // No validations yet, assume healthy
        }
    }

    /// Get health summary
    pub async fn get_health_summary(&self) -> ArbitrageResult<serde_json::Value> {
        let metrics = self.get_metrics().await;

        let success_rate = if metrics.total_validations > 0 {
            metrics.successful_validations as f32 / metrics.total_validations as f32 * 100.0
        } else {
            100.0
        };

        Ok(serde_json::json!({
            "is_healthy": success_rate >= 80.0,
            "success_rate_percent": success_rate,
            "total_validations": metrics.total_validations,
            "average_quality_score": metrics.average_quality_score,
            "average_freshness_score": metrics.average_freshness_score,
            "stale_data_count": metrics.stale_data_count,
            "invalid_data_count": metrics.invalid_data_count,
            "data_sources_quality": metrics.data_sources_quality,
            "validation_errors_by_type": metrics.validation_errors_by_type,
            "last_updated": metrics.last_updated
        }))
    }
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            field_name: String::new(),
            rule_type: ValidationRuleType::String,
            required: false,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
            pattern: None,
            allowed_values: None,
            custom_validator: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rule_creation() {
        let rule = ValidationRule {
            field_name: "price".to_string(),
            rule_type: ValidationRuleType::Price,
            required: true,
            min_value: Some(0.0),
            ..Default::default()
        };

        assert_eq!(rule.field_name, "price");
        assert_eq!(rule.rule_type, ValidationRuleType::Price);
        assert!(rule.required);
    }

    #[test]
    fn test_freshness_rule_default() {
        let rule = FreshnessRule::default();
        assert_eq!(rule.max_age_seconds, 300);
        assert!(rule.enable_staleness_check);
    }

    #[test]
    fn test_data_validator_config_validation() {
        let config = DataValidatorConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config;
        invalid_config.quality_score_threshold = 1.5; // Invalid
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(!result.is_valid);
        assert!(!result.is_fresh);
        assert_eq!(result.overall_score, 0.0);
    }
}
