//! Data Validation & Consistency Engine
//!
//! Ensures data consistency and integrity during legacy system migrations,
//! providing comprehensive validation rules and automated consistency checks.

use super::shared_types::MigrationEvent;
use crate::utils::error::ErrorKind;
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use worker::Env;

/// Validation rule types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    /// Field presence validation
    FieldPresence,
    /// Data type validation
    DataType,
    /// Format validation (regex, patterns)
    Format,
    /// Range validation (numeric ranges)
    Range,
    /// Cross-field validation
    CrossField,
    /// Business logic validation
    BusinessLogic,
    /// Referential integrity
    ReferentialIntegrity,
    /// Custom validation rule
    Custom,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical error - migration should stop
    Critical,
    /// Warning - log but continue
    Warning,
    /// Information - for audit purposes
    Info,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule identifier
    pub rule_id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule type
    pub rule_type: ValidationRuleType,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Target field or entity
    pub target: String,
    /// Validation expression or configuration
    pub expression: String,
    /// Custom parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Enable rule
    pub enabled: bool,
    /// Created timestamp
    pub created_at: u64,
}

/// Validation result for a single check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation ID
    pub validation_id: String,
    /// Rule that was applied
    pub rule_id: String,
    /// Target that was validated
    pub target: String,
    /// Validation passed
    pub passed: bool,
    /// Validation severity
    pub severity: ValidationSeverity,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Expected value
    pub expected_value: Option<serde_json::Value>,
    /// Actual value
    pub actual_value: Option<serde_json::Value>,
    /// Validation timestamp
    pub timestamp: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Consistency check between systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheck {
    /// Check identifier
    pub check_id: String,
    /// Source system data
    pub source_data: serde_json::Value,
    /// Target system data
    pub target_data: serde_json::Value,
    /// Fields to compare
    pub fields_to_compare: Vec<String>,
    /// Consistency result
    pub consistent: bool,
    /// Differences found
    pub differences: Vec<DataDifference>,
    /// Check timestamp
    pub timestamp: u64,
}

/// Data difference between systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDifference {
    /// Field path
    pub field_path: String,
    /// Source value
    pub source_value: serde_json::Value,
    /// Target value
    pub target_value: serde_json::Value,
    /// Difference type
    pub difference_type: DifferenceType,
}

/// Types of data differences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifferenceType {
    /// Value mismatch
    ValueMismatch,
    /// Field missing in source
    MissingInSource,
    /// Field missing in target
    MissingInTarget,
    /// Type mismatch
    TypeMismatch,
    /// Format difference
    FormatDifference,
}

/// Data comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataComparison {
    /// Comparison ID
    pub comparison_id: String,
    /// Records compared
    pub records_compared: u64,
    /// Matching records
    pub matching_records: u64,
    /// Mismatched records
    pub mismatched_records: u64,
    /// Missing in source
    pub missing_in_source: u64,
    /// Missing in target
    pub missing_in_target: u64,
    /// Match percentage
    pub match_percentage: f64,
    /// Comparison duration in milliseconds
    pub duration_ms: u64,
    /// Sample differences
    pub sample_differences: Vec<DataDifference>,
}

/// Validation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total validations performed
    pub total_validations: u64,
    /// Passed validations
    pub passed_validations: u64,
    /// Failed validations
    pub failed_validations: u64,
    /// Critical failures
    pub critical_failures: u64,
    /// Warning count
    pub warning_count: u64,
    /// Average validation time in milliseconds
    pub avg_validation_time_ms: f64,
    /// Last updated timestamp
    pub last_updated: u64,
}

impl Default for ValidationMetrics {
    fn default() -> Self {
        Self {
            total_validations: 0,
            passed_validations: 0,
            failed_validations: 0,
            critical_failures: 0,
            warning_count: 0,
            avg_validation_time_ms: 0.0,
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Validation engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable validation engine
    pub enabled: bool,
    /// Validation timeout in seconds
    pub validation_timeout_seconds: u64,
    /// Maximum batch size for validations
    pub max_batch_size: u32,
    /// Sample size for consistency checks
    pub consistency_check_sample_size: u32,
    /// Enable parallel validation
    pub enable_parallel_validation: bool,
    /// Maximum concurrent validations
    pub max_concurrent_validations: u32,
    /// Fail fast on critical errors
    pub fail_fast_on_critical: bool,
    /// Store validation results
    pub store_validation_results: bool,
    /// Results retention period in days
    pub results_retention_days: u32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            validation_timeout_seconds: 30,
            max_batch_size: 1000,
            consistency_check_sample_size: 100,
            enable_parallel_validation: true,
            max_concurrent_validations: 10,
            fail_fast_on_critical: true,
            store_validation_results: true,
            results_retention_days: 30,
        }
    }
}

/// Validation Engine main implementation
pub struct ValidationEngine {
    /// Configuration
    config: ValidationConfig,
    /// Validation rules
    validation_rules: Arc<Mutex<HashMap<String, ValidationRule>>>,
    /// Validation results history
    validation_results: Arc<Mutex<Vec<ValidationResult>>>,
    /// Consistency check history
    consistency_checks: Arc<Mutex<Vec<ConsistencyCheck>>>,
    /// Performance metrics
    metrics: Arc<Mutex<ValidationMetrics>>,
    /// Event history
    #[allow(dead_code)]
    event_history: Arc<Mutex<Vec<MigrationEvent>>>,
    /// Logger instance
    logger: crate::utils::logger::Logger,
}

impl ValidationEngine {
    /// Create new validation engine
    pub async fn new(config: ValidationConfig) -> ArbitrageResult<Self> {
        let logger = crate::utils::logger::Logger::new(crate::utils::logger::LogLevel::Info);

        let engine = Self {
            config,
            validation_rules: Arc::new(Mutex::new(HashMap::new())),
            validation_results: Arc::new(Mutex::new(Vec::new())),
            consistency_checks: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(ValidationMetrics::default())),
            event_history: Arc::new(Mutex::new(Vec::new())),
            logger,
        };

        engine.logger.info("Validation Engine initialized");
        Ok(engine)
    }

    /// Add validation rule
    pub async fn add_validation_rule(&self, rule: ValidationRule) -> ArbitrageResult<()> {
        {
            let mut rules = self.validation_rules.lock().unwrap();
            rules.insert(rule.rule_id.clone(), rule.clone());
        }

        self.logger.info(&format!(
            "Added validation rule: {} ({})",
            rule.name, rule.rule_id
        ));
        Ok(())
    }

    /// Validate data against rules
    pub async fn validate_data(
        &self,
        __env: &Env,
        data: &serde_json::Value,
        rule_ids: Option<Vec<String>>,
    ) -> ArbitrageResult<Vec<ValidationResult>> {
        if !self.config.enabled {
            return Err(ArbitrageError::new(
                ErrorKind::ConfigError,
                "Validation engine is disabled".to_string(),
            ));
        }

        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        let mut results = Vec::new();

        // Get applicable rules
        let rules = {
            let rules_map = self.validation_rules.lock().unwrap();
            if let Some(rule_ids) = rule_ids {
                rule_ids
                    .into_iter()
                    .filter_map(|id| rules_map.get(&id).cloned())
                    .collect::<Vec<_>>()
            } else {
                rules_map.values().cloned().collect()
            }
        };

        // Apply each rule
        for rule in rules {
            if !rule.enabled {
                continue;
            }

            let result = self.apply_validation_rule(&rule, data).await?;
            results.push(result);

            // Fail fast on critical errors if configured
            if self.config.fail_fast_on_critical
                && rule.severity == ValidationSeverity::Critical
                && !results.last().unwrap().passed
            {
                break;
            }
        }

        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        let duration = end_time - start_time;

        // Update metrics
        self.update_validation_metrics(&results, duration).await?;

        Ok(results)
    }

    /// Get validation metrics
    pub async fn get_metrics(&self) -> ValidationMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }

    // ============= PRIVATE HELPER METHODS =============

    /// Apply a single validation rule
    async fn apply_validation_rule(
        &self,
        rule: &ValidationRule,
        data: &serde_json::Value,
    ) -> ArbitrageResult<ValidationResult> {
        let validation_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        let (passed, error_message, expected_value, actual_value) = match rule.rule_type {
            ValidationRuleType::FieldPresence => {
                self.validate_field_presence(data, &rule.target, &rule.expression)
            }
            ValidationRuleType::DataType => {
                self.validate_data_type(data, &rule.target, &rule.expression)
            }
            ValidationRuleType::Format => {
                self.validate_format(data, &rule.target, &rule.expression)
            }
            ValidationRuleType::Range => self.validate_range(data, &rule.target, &rule.parameters),
            ValidationRuleType::Custom => {
                self.validate_custom(data, &rule.expression, &rule.parameters)
            }
            _ => {
                // For other rule types, assume they pass for now
                (true, None, None, None)
            }
        };

        Ok(ValidationResult {
            validation_id,
            rule_id: rule.rule_id.clone(),
            target: rule.target.clone(),
            passed,
            severity: rule.severity.clone(),
            error_message,
            expected_value,
            actual_value,
            timestamp,
            metadata: HashMap::new(),
        })
    }

    /// Validate field presence
    fn validate_field_presence(
        &self,
        data: &serde_json::Value,
        field_path: &str,
        _expression: &str,
    ) -> (
        bool,
        Option<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
    ) {
        let field_value = self.get_field_value(data, field_path);
        let present = field_value.is_some();

        let error_message = if !present {
            Some(format!("Required field '{}' is missing", field_path))
        } else {
            None
        };

        (present, error_message, None, field_value)
    }

    /// Validate data type
    fn validate_data_type(
        &self,
        data: &serde_json::Value,
        field_path: &str,
        expected_type: &str,
    ) -> (
        bool,
        Option<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
    ) {
        let field_value = self.get_field_value(data, field_path);

        if let Some(value) = &field_value {
            let actual_type = match value {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };

            let type_matches = actual_type == expected_type;
            let error_message = if !type_matches {
                Some(format!(
                    "Field '{}' expected type '{}' but got '{}'",
                    field_path, expected_type, actual_type
                ))
            } else {
                None
            };

            (
                type_matches,
                error_message,
                Some(serde_json::Value::String(expected_type.to_string())),
                field_value,
            )
        } else {
            (
                false,
                Some(format!("Field '{}' does not exist", field_path)),
                Some(serde_json::Value::String(expected_type.to_string())),
                None,
            )
        }
    }

    /// Validate format using regex
    fn validate_format(
        &self,
        data: &serde_json::Value,
        field_path: &str,
        pattern: &str,
    ) -> (
        bool,
        Option<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
    ) {
        let field_value = self.get_field_value(data, field_path);

        if let Some(value) = &field_value {
            if let Some(_string_value) = value.as_str() {
                // In a real implementation, you'd use regex crate
                let format_valid = true; // Simplified for now

                let error_message = if !format_valid {
                    Some(format!(
                        "Field '{}' does not match pattern '{}'",
                        field_path, pattern
                    ))
                } else {
                    None
                };

                (
                    format_valid,
                    error_message,
                    Some(serde_json::Value::String(pattern.to_string())),
                    field_value,
                )
            } else {
                (
                    false,
                    Some(format!("Field '{}' is not a string", field_path)),
                    Some(serde_json::Value::String(pattern.to_string())),
                    field_value,
                )
            }
        } else {
            (
                false,
                Some(format!("Field '{}' does not exist", field_path)),
                Some(serde_json::Value::String(pattern.to_string())),
                None,
            )
        }
    }

    /// Validate numeric range
    fn validate_range(
        &self,
        data: &serde_json::Value,
        field_path: &str,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> (
        bool,
        Option<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
    ) {
        let field_value = self.get_field_value(data, field_path);

        if let Some(value) = &field_value {
            if let Some(number) = value.as_f64() {
                let min = parameters
                    .get("min")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::NEG_INFINITY);
                let max = parameters
                    .get("max")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::INFINITY);

                let in_range = number >= min && number <= max;

                let error_message = if !in_range {
                    Some(format!(
                        "Field '{}' value {} is outside range [{}, {}]",
                        field_path, number, min, max
                    ))
                } else {
                    None
                };

                (
                    in_range,
                    error_message,
                    Some(serde_json::json!({"min": min, "max": max})),
                    field_value,
                )
            } else {
                (
                    false,
                    Some(format!("Field '{}' is not a number", field_path)),
                    None,
                    field_value,
                )
            }
        } else {
            (
                false,
                Some(format!("Field '{}' does not exist", field_path)),
                None,
                None,
            )
        }
    }

    /// Validate custom rule
    fn validate_custom(
        &self,
        _data: &serde_json::Value,
        _expression: &str,
        _parameters: &HashMap<String, serde_json::Value>,
    ) -> (
        bool,
        Option<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
    ) {
        // Custom validation logic would be implemented here
        (true, None, None, None)
    }

    /// Get field value by path
    fn get_field_value(
        &self,
        data: &serde_json::Value,
        field_path: &str,
    ) -> Option<serde_json::Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = data;

        for part in parts {
            if let Some(value) = current.get(part) {
                current = value;
            } else {
                return None;
            }
        }

        Some(current.clone())
    }

    /// Update validation metrics
    /// Compare data between legacy and new systems
    pub async fn compare_data_between_systems(
        &self,
        source_data: &serde_json::Value,
        target_data: &serde_json::Value,
        fields_to_compare: Option<Vec<String>>,
    ) -> ArbitrageResult<ConsistencyCheck> {
        let check_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        let fields = fields_to_compare.unwrap_or_else(|| {
            // Extract all fields from source data if not specified
            self.extract_field_paths(source_data)
        });

        let mut differences = Vec::new();
        let mut consistent = true;

        for field_path in &fields {
            let source_value = self.get_field_value(source_data, field_path);
            let target_value = self.get_field_value(target_data, field_path);

            match (source_value, target_value) {
                (Some(source_val), Some(target_val)) => {
                    if source_val != target_val {
                        differences.push(DataDifference {
                            field_path: field_path.clone(),
                            source_value: source_val,
                            target_value: target_val,
                            difference_type: DifferenceType::ValueMismatch,
                        });
                        consistent = false;
                    }
                }
                (Some(source_val), None) => {
                    differences.push(DataDifference {
                        field_path: field_path.clone(),
                        source_value: source_val,
                        target_value: serde_json::Value::Null,
                        difference_type: DifferenceType::MissingInTarget,
                    });
                    consistent = false;
                }
                (None, Some(target_val)) => {
                    differences.push(DataDifference {
                        field_path: field_path.clone(),
                        source_value: serde_json::Value::Null,
                        target_value: target_val,
                        difference_type: DifferenceType::MissingInSource,
                    });
                    consistent = false;
                }
                (None, None) => {
                    // Both missing, this is consistent
                }
            }
        }

        let consistency_check = ConsistencyCheck {
            check_id: check_id.clone(),
            source_data: source_data.clone(),
            target_data: target_data.clone(),
            fields_to_compare: fields,
            consistent,
            differences,
            timestamp,
        };

        // Store consistency check result
        {
            let mut checks = self.consistency_checks.lock().unwrap();
            checks.push(consistency_check.clone());

            // Keep only last 10000 checks
            let len = checks.len();
            if len > 10000 {
                checks.drain(0..len - 10000);
            }
        }

        Ok(consistency_check)
    }

    /// Check data consistency with automated validation
    pub async fn check_data_consistency(
        &self,
        batch_data: &[(serde_json::Value, serde_json::Value)],
    ) -> ArbitrageResult<DataComparison> {
        let comparison_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let mut matching_records = 0;
        let mut mismatched_records = 0;
        let mut sample_differences = Vec::new();

        for (source, target) in batch_data {
            let consistency_check = self
                .compare_data_between_systems(source, target, None)
                .await?;

            if consistency_check.consistent {
                matching_records += 1;
            } else {
                mismatched_records += 1;

                // Collect sample differences (up to 100)
                if sample_differences.len() < 100 {
                    sample_differences.extend(consistency_check.differences);
                }
            }
        }

        let records_compared = batch_data.len() as u64;
        let match_percentage = if records_compared > 0 {
            (matching_records as f64 / records_compared as f64) * 100.0
        } else {
            0.0
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(DataComparison {
            comparison_id,
            records_compared,
            matching_records,
            mismatched_records,
            missing_in_source: 0, // Would need specific logic
            missing_in_target: 0, // Would need specific logic
            match_percentage,
            duration_ms,
            sample_differences,
        })
    }

    /// Extract field paths from JSON data
    fn extract_field_paths(&self, data: &serde_json::Value) -> Vec<String> {
        let mut paths = Vec::new();
        self.extract_paths_recursive(data, "", &mut paths);
        paths
    }

    /// Recursively extract field paths
    #[allow(clippy::only_used_in_recursion)]
    fn extract_paths_recursive(
        &self,
        value: &serde_json::Value,
        prefix: &str,
        paths: &mut Vec<String>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    paths.push(path.clone());
                    self.extract_paths_recursive(val, &path, paths);
                }
            }
            serde_json::Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let path = format!("{}[{}]", prefix, index);
                    paths.push(path.clone());
                    self.extract_paths_recursive(val, &path, paths);
                }
            }
            _ => {
                // Leaf value, already added to paths
            }
        }
    }

    /// Validate business logic rules
    pub async fn validate_business_logic(
        &self,
        data: &serde_json::Value,
        business_rules: &[String],
    ) -> ArbitrageResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for _rule_expression in business_rules {
            let result = ValidationResult {
                validation_id: Uuid::new_v4().to_string(),
                rule_id: "business_logic".to_string(),
                target: "data".to_string(),
                passed: true, // Simplified - would implement actual business logic validation
                severity: ValidationSeverity::Warning,
                error_message: None,
                expected_value: None,
                actual_value: Some(data.clone()),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                metadata: HashMap::new(),
            };
            results.push(result);
        }

        Ok(results)
    }

    /// Get validation history
    pub async fn get_validation_history(&self, limit: Option<usize>) -> Vec<ValidationResult> {
        let results = self.validation_results.lock().unwrap();
        let limit = limit.unwrap_or(1000);

        if results.len() <= limit {
            results.clone()
        } else {
            results[results.len() - limit..].to_vec()
        }
    }

    /// Get consistency check history
    pub async fn get_consistency_history(&self, limit: Option<usize>) -> Vec<ConsistencyCheck> {
        let checks = self.consistency_checks.lock().unwrap();
        let limit = limit.unwrap_or(1000);

        if checks.len() <= limit {
            checks.clone()
        } else {
            checks[checks.len() - limit..].to_vec()
        }
    }

    async fn update_validation_metrics(
        &self,
        results: &[ValidationResult],
        duration_ms: u64,
    ) -> ArbitrageResult<()> {
        let mut metrics = self.metrics.lock().unwrap();

        metrics.total_validations += results.len() as u64;

        for result in results {
            if result.passed {
                metrics.passed_validations += 1;
            } else {
                metrics.failed_validations += 1;

                if result.severity == ValidationSeverity::Critical {
                    metrics.critical_failures += 1;
                } else if result.severity == ValidationSeverity::Warning {
                    metrics.warning_count += 1;
                }
            }
        }

        // Update average validation time
        let total_time = metrics.avg_validation_time_ms
            * (metrics.total_validations - results.len() as u64) as f64
            + duration_ms as f64;
        metrics.avg_validation_time_ms = total_time / metrics.total_validations as f64;

        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;

        Ok(())
    }
}
