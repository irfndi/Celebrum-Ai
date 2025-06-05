use crate::services::core::trading::{KvOperationError, KvOperations, KvResult};
use crate::utils::{ArbitrageError, ArbitrageResult};
use async_trait::async_trait;
use parking_lot::Mutex;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Mock KV Store for testing
pub struct MockKvStore {
    pub data: Arc<Mutex<HashMap<String, String>>>,
    pub error_simulation: Option<String>,
    pub operation_count: Arc<Mutex<u32>>,
}

impl MockKvStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            error_simulation: None,
            operation_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    pub fn reset_error_simulation(&mut self) {
        self.error_simulation = None;
    }

    pub async fn mock_put(&mut self, key: &str, value: &str) -> ArbitrageResult<()> {
        *self.operation_count.lock() += 1;

        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_put_failed" => Err(ArbitrageError::database_error("KV put operation failed")),
                "rate_limit" => Err(ArbitrageError::rate_limit_error("Rate limit exceeded")),
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }
        self.data.lock().insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub async fn mock_get(&mut self, key: &str) -> ArbitrageResult<Option<String>> {
        *self.operation_count.lock() += 1;
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_get_failed" => Err(ArbitrageError::database_error("KV get operation failed")),
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }
        Ok(self.data.lock().get(key).cloned())
    }
}

#[async_trait]
impl KvOperations for MockKvStore {
    async fn put<T: Serialize + Send + Sync + ?Sized>(&self, key: &str, value: &T) -> KvResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_put_failed" => Err(KvOperationError::Storage(
                    "KV put operation failed".to_string(),
                )),
                "rate_limit" => Err(KvOperationError::Storage("Rate limit exceeded".to_string())),
                _ => Err(KvOperationError::Storage("Unknown KV error".to_string())),
            };
        }
        let mut data_guard = self.data.lock();
        let s_value = serde_json::to_string(value).map_err(KvOperationError::Serialization)?;
        data_guard.insert(key.to_string(), s_value);
        *self.operation_count.lock() += 1;
        Ok(())
    }

    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> KvResult<Option<T>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_get_failed" => Err(KvOperationError::Storage(
                    "KV get operation failed".to_string(),
                )),
                _ => Err(KvOperationError::Storage("Unknown KV error".to_string())),
            };
        }
        let data_guard = self.data.lock();
        *self.operation_count.lock() += 1;
        match data_guard.get(key) {
            Some(s_val) => {
                let val: T =
                    serde_json::from_str(s_val).map_err(KvOperationError::Serialization)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, key: &str) -> KvResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_delete_failed" => Err(KvOperationError::Storage(
                    "KV delete operation failed".to_string(),
                )),
                _ => Err(KvOperationError::Storage("Unknown KV error".to_string())),
            };
        }
        let mut data_guard = self.data.lock();
        data_guard.remove(key);
        *self.operation_count.lock() += 1;
        Ok(())
    }
}

impl MockKvStore {
    pub async fn mock_delete(&mut self, key: &str) -> ArbitrageResult<()> {
        *self.operation_count.lock() += 1;
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_delete_failed" => {
                    Err(ArbitrageError::database_error("KV delete operation failed"))
                }
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }
        self.data.lock().remove(key);
        Ok(())
    }
}
