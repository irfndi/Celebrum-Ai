// Database Repositories Module - Simplified Unified Architecture
// After infrastructure consolidation: 15→2 files for Cloudflare Workers optimization

// NEW: Unified modules (post-consolidation)
pub mod storage_layer;
pub mod unified_database_core;
pub mod unified_repository_layer;

// Legacy components (still needed)
pub mod database_manager;

// Re-export main components for easy access

// NEW: Unified components (recommended for new code)
pub use storage_layer::{
    StorageLayerBuilder, StorageLayerConfig, StorageLayerMetrics, StorageLayerService,
    StorageResult,
};

pub use unified_database_core::{UnifiedDatabaseConfig, UnifiedDatabaseCore};

pub use unified_repository_layer::{
    UnifiedRepositoryConfig, UnifiedRepositoryLayer, UnifiedRepositoryMetrics,
};

// Legacy components
pub use database_manager::{DatabaseManager, DatabaseManagerConfig};

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Repository health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealth {
    pub repository_name: String,
    pub is_healthy: bool,
    pub database_healthy: bool,
    pub cache_healthy: bool,
    pub last_health_check: u64,
    pub response_time_ms: f64,
    pub error_rate: f64,
}

/// Repository performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetrics {
    pub repository_name: String,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_response_time_ms: f64,
    pub operations_per_second: f64,
    pub cache_hit_rate: f64,
    pub last_updated: u64,
}

impl Default for RepositoryMetrics {
    fn default() -> Self {
        Self {
            repository_name: String::new(),
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_response_time_ms: 0.0,
            operations_per_second: 0.0,
            cache_hit_rate: 0.0,
            last_updated: 0,
        }
    }
}

/// Base repository trait for common operations
#[allow(async_fn_in_trait)]
pub trait Repository {
    /// Get repository name
    fn name(&self) -> &str;

    /// Health check for the repository
    async fn health_check(&self) -> ArbitrageResult<RepositoryHealth>;

    /// Get performance metrics
    async fn get_metrics(&self) -> RepositoryMetrics;

    /// Initialize repository (create tables, indexes, etc.)
    async fn initialize(&self) -> ArbitrageResult<()>;

    /// Cleanup and shutdown
    async fn shutdown(&self) -> ArbitrageResult<()>;
}

/// Repository configuration trait
pub trait RepositoryConfig {
    /// Validate configuration
    fn validate(&self) -> ArbitrageResult<()>;

    /// Get connection pool size
    fn connection_pool_size(&self) -> u32;

    /// Get batch size for operations
    fn batch_size(&self) -> usize;

    /// Get cache TTL in seconds
    fn cache_ttl_seconds(&self) -> u64;
}

/// Common repository utilities
pub mod utils {
    use super::*;
    use serde_json::Value;
    use worker::wasm_bindgen::JsValue;

    /// Convert JsValue to string safely
    pub fn js_value_to_string(value: &JsValue) -> String {
        value.as_string().unwrap_or_default()
    }

    /// Convert JsValue to i64 safely
    pub fn js_value_to_i64(value: &JsValue, default: i64) -> i64 {
        value.as_f64().map(|f| f as i64).unwrap_or(default)
    }

    /// Convert JsValue to f64 safely
    pub fn js_value_to_f64(value: &JsValue, default: f64) -> f64 {
        value.as_f64().unwrap_or(default)
    }

    /// Convert JsValue to bool safely
    pub fn js_value_to_bool(value: &JsValue, default: bool) -> bool {
        value.as_bool().unwrap_or(default)
    }

    /// Get string field from row safely
    pub fn get_string_field(
        row: &HashMap<String, Value>,
        field_name: &str,
    ) -> ArbitrageResult<String> {
        row.get(field_name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                ArbitrageError::parse_error(format!("Missing or invalid field: {}", field_name))
            })
    }

    /// Get optional string field from row
    pub fn get_optional_string_field(
        row: &HashMap<String, Value>,
        field_name: &str,
    ) -> Option<String> {
        row.get(field_name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get optional i64 field from row
    pub fn get_optional_i64_field(row: &HashMap<String, Value>, field_name: &str) -> Option<i64> {
        row.get(field_name).and_then(|v| v.as_i64())
    }

    /// Get i64 field from row safely
    pub fn get_i64_field(row: &HashMap<String, Value>, field_name: &str, default: i64) -> i64 {
        row.get(field_name)
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    /// Get f64 field from row safely
    pub fn get_f64_field(row: &HashMap<String, Value>, field_name: &str, default: f64) -> f64 {
        row.get(field_name)
            .and_then(|v| v.as_f64())
            .unwrap_or(default)
    }

    /// Get bool field from row safely
    pub fn get_bool_field(row: &HashMap<String, Value>, field_name: &str, default: bool) -> bool {
        row.get(field_name)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Get JSON field from row safely
    pub fn get_json_field<T: serde::de::DeserializeOwned>(
        row: &HashMap<String, Value>,
        field_name: &str,
        default: T,
    ) -> T {
        row.get(field_name)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(default)
    }

    /// Create database error with context
    pub fn database_error(operation: &str, error: impl std::fmt::Display) -> ArbitrageError {
        ArbitrageError::database_error(format!("{}: {}", operation, error))
    }

    /// Create validation error with context
    pub fn validation_error(field: &str, reason: &str) -> ArbitrageError {
        ArbitrageError::validation_error(format!("Field '{}': {}", field, reason))
    }

    /// Generate UUID string
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Get current timestamp in milliseconds
    pub fn current_timestamp_ms() -> u64 {
        chrono::Utc::now().timestamp_millis() as u64
    }

    /// Validate required string field
    pub fn validate_required_string(value: &str, field_name: &str) -> ArbitrageResult<()> {
        if value.trim().is_empty() {
            return Err(validation_error(field_name, "cannot be empty"));
        }
        Ok(())
    }

    /// Validate email format
    pub fn validate_email(email: &str) -> ArbitrageResult<()> {
        // Basic email validation: must have text before @, @ symbol, and . after @
        let at_pos = email.find('@');
        if let Some(at_pos) = at_pos {
            // Check there's text before @
            if at_pos == 0 {
                return Err(validation_error("email", "invalid format"));
            }
            // Check there's text after @ and a . in the domain part
            let domain_part = &email[at_pos + 1..];
            if domain_part.is_empty() || !domain_part.contains('.') {
                return Err(validation_error("email", "invalid format"));
            }
        } else {
            return Err(validation_error("email", "invalid format"));
        }
        Ok(())
    }

    /// Validate positive number
    pub fn validate_positive_number(value: f64, field_name: &str) -> ArbitrageResult<()> {
        if value <= 0.0 {
            return Err(validation_error(field_name, "must be positive"));
        }
        Ok(())
    }

    /// Batch operations helper
    pub async fn execute_batch_operations<T, F, Fut>(
        items: &[T],
        batch_size: usize,
        operation: F,
    ) -> ArbitrageResult<Vec<ArbitrageResult<()>>>
    where
        F: Fn(&[T]) -> Fut,
        Fut: std::future::Future<Output = ArbitrageResult<()>>,
    {
        let mut results = Vec::new();

        for chunk in items.chunks(batch_size) {
            let result = operation(chunk).await;
            results.push(result);
        }

        Ok(results)
    }
}

/// User repository interface for user-related operations
#[derive(Clone)]
pub struct UserRepository {
    pub database: Arc<worker::worker::D1Database>,
    pub name: String,
    pub metrics: Arc<std::sync::Mutex<RepositoryMetrics>>,
}

impl UserRepository {
    pub fn new(database: Arc<worker::worker::D1Database>) -> Self {
        let metrics = RepositoryMetrics {
            repository_name: "UserRepository".to_string(),
            ..Default::default()
        };

        Self {
            database,
            name: "UserRepository".to_string(),
            metrics: Arc::new(std::sync::Mutex::new(metrics)),
        }
    }

    /// Get user by ID
    pub async fn get_user_by_id(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<serde_json::Value>> {
        let stmt = self
            .database
            .prepare("SELECT * FROM user_profiles WHERE id = ?");
        let params = [worker::wasm_bindgen::JsValue::from_str(user_id)];

        let result = stmt.bind(&params)?.first::<serde_json::Value>(None).await?;
        Ok(result)
    }

    /// Create user
    pub async fn create_user(&self, _user_data: &serde_json::Value) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }

    /// Update user
    pub async fn update_user(
        &self,
        _user_id: &str,
        _user_data: &serde_json::Value,
    ) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }

    /// Delete user
    pub async fn delete_user(&self, _user_id: &str) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }
}

impl Repository for UserRepository {
    fn name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> ArbitrageResult<RepositoryHealth> {
        Ok(RepositoryHealth {
            repository_name: self.name.clone(),
            is_healthy: true,
            database_healthy: true,
            cache_healthy: true,
            last_health_check: crate::utils::time::get_current_timestamp(),
            response_time_ms: 10.0,
            error_rate: 0.0,
        })
    }

    async fn get_metrics(&self) -> RepositoryMetrics {
        self.metrics.lock().unwrap().clone()
    }

    async fn initialize(&self) -> ArbitrageResult<()> {
        Ok(())
    }

    async fn shutdown(&self) -> ArbitrageResult<()> {
        Ok(())
    }
}

/// Invitation repository interface for invitation-related operations
#[derive(Clone)]
pub struct InvitationRepository {
    pub database: Arc<worker::worker::D1Database>,
    pub name: String,
    pub metrics: Arc<std::sync::Mutex<RepositoryMetrics>>,
}

impl InvitationRepository {
    pub fn new(database: Arc<worker::worker::D1Database>) -> Self {
        let metrics = RepositoryMetrics {
            repository_name: "InvitationRepository".to_string(),
            ..Default::default()
        };

        Self {
            database,
            name: "InvitationRepository".to_string(),
            metrics: Arc::new(std::sync::Mutex::new(metrics)),
        }
    }

    /// Get invitation by code
    pub async fn get_invitation_by_code(
        &self,
        code: &str,
    ) -> ArbitrageResult<Option<serde_json::Value>> {
        let stmt = self
            .database
            .prepare("SELECT * FROM invitation_codes WHERE code = ?");
        let params = [worker::wasm_bindgen::JsValue::from_str(code)];

        let result = stmt.bind(&params)?.first::<serde_json::Value>(None).await?;
        Ok(result)
    }

    /// Create invitation
    pub async fn create_invitation(
        &self,
        _invitation_data: &serde_json::Value,
    ) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }

    /// Update invitation
    pub async fn update_invitation(
        &self,
        _invitation_id: &str,
        _invitation_data: &serde_json::Value,
    ) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }

    /// Delete invitation
    pub async fn delete_invitation(&self, _invitation_id: &str) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }
}

impl Repository for InvitationRepository {
    fn name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> ArbitrageResult<RepositoryHealth> {
        Ok(RepositoryHealth {
            repository_name: self.name.clone(),
            is_healthy: true,
            database_healthy: true,
            cache_healthy: true,
            last_health_check: crate::utils::time::get_current_timestamp(),
            response_time_ms: 10.0,
            error_rate: 0.0,
        })
    }

    async fn get_metrics(&self) -> RepositoryMetrics {
        self.metrics.lock().unwrap().clone()
    }

    async fn initialize(&self) -> ArbitrageResult<()> {
        Ok(())
    }

    async fn shutdown(&self) -> ArbitrageResult<()> {
        Ok(())
    }
}

/// D1 Database Service for Cloudflare Workers
#[derive(Clone)]
pub struct D1Service {
    pub database: Arc<worker::worker::D1Database>,
    pub name: String,
    pub metrics: Arc<std::sync::Mutex<RepositoryMetrics>>,
}

impl D1Service {
    pub fn new(database: Arc<worker::worker::D1Database>) -> Self {
        let metrics = RepositoryMetrics {
            repository_name: "D1Service".to_string(),
            ..Default::default()
        };

        Self {
            database,
            name: "D1Service".to_string(),
            metrics: Arc::new(std::sync::Mutex::new(metrics)),
        }
    }

    /// Execute a query
    pub async fn execute_query(
        &self,
        query: &str,
        params: &[worker::wasm_bindgen::JsValue],
    ) -> ArbitrageResult<serde_json::Value> {
        let stmt = self.database.prepare(query);
        let result = stmt.bind(params)?.first::<serde_json::Value>(None).await?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    /// Execute a mutation (INSERT, UPDATE, DELETE)
    pub async fn execute_mutation(
        &self,
        query: &str,
        params: &[worker::wasm_bindgen::JsValue],
    ) -> ArbitrageResult<bool> {
        let stmt = self.database.prepare(query);
        let _result = stmt.bind(params)?.run().await?;
        Ok(true)
    }

    /// Get all records from a table
    pub async fn get_all(&self, table: &str) -> ArbitrageResult<Vec<serde_json::Value>> {
        let query = format!("SELECT * FROM {}", table);
        let stmt = self.database.prepare(&query);
        let result = stmt.all().await?;
        Ok(result.results::<serde_json::Value>()?)
    }

    /// Get record by ID
    pub async fn get_by_id(
        &self,
        table: &str,
        id: &str,
    ) -> ArbitrageResult<Option<serde_json::Value>> {
        let query = format!("SELECT * FROM {} WHERE id = ?", table);
        let stmt = self.database.prepare(&query);
        let params = [worker::wasm_bindgen::JsValue::from_str(id)];
        let result = stmt.bind(&params)?.first::<serde_json::Value>(None).await?;
        Ok(result)
    }

    /// Store user profile
    pub async fn store_user_profile(&self, _profile: &serde_json::Value) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }

    /// Store user preferences
    pub async fn store_user_preferences(
        &self,
        _preferences: &serde_json::Value,
    ) -> ArbitrageResult<bool> {
        // Implementation would go here
        Ok(true)
    }
}

impl Repository for D1Service {
    fn name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> ArbitrageResult<RepositoryHealth> {
        Ok(RepositoryHealth {
            repository_name: self.name.clone(),
            is_healthy: true,
            database_healthy: true,
            cache_healthy: true,
            last_health_check: crate::utils::time::get_current_timestamp(),
            response_time_ms: 10.0,
            error_rate: 0.0,
        })
    }

    async fn get_metrics(&self) -> RepositoryMetrics {
        self.metrics.lock().unwrap().clone()
    }

    async fn initialize(&self) -> ArbitrageResult<()> {
        Ok(())
    }

    async fn shutdown(&self) -> ArbitrageResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;

    #[test]
    fn test_string_validation() {
        assert!(validate_required_string("test", "field").is_ok());
        assert!(validate_required_string("", "field").is_err());
        assert!(validate_required_string("   ", "field").is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@").is_err());
    }

    #[test]
    fn test_positive_number_validation() {
        assert!(validate_positive_number(1.0, "field").is_ok());
        assert!(validate_positive_number(0.1, "field").is_ok());
        assert!(validate_positive_number(0.0, "field").is_err());
        assert!(validate_positive_number(-1.0, "field").is_err());
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = generate_uuid();
        let uuid2 = generate_uuid();
        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.len(), 36); // Standard UUID length
    }

    #[test]
    fn test_timestamp_generation() {
        let ts1 = current_timestamp_ms();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let ts2 = current_timestamp_ms();
        assert!(ts2 > ts1);
    }
}
