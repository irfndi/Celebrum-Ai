use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvOperationError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Underlying KV store error: {0}")]
    SdkError(String), // To wrap errors from the actual KvStore or other sources
}

// Define a generic Result type for KV operations
pub type KvResult<T> = Result<T, KvOperationError>;

#[cfg(target_arch = "wasm32")]
#[async_trait]
pub trait KvOperations {
    async fn put<T: Serialize + Send + ?Sized>(&self, key: &str, value: &T) -> KvResult<()>;
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> KvResult<Option<T>>;
    async fn delete(&self, key: &str) -> KvResult<()>;
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait KvOperations: Send + Sync {
    async fn put<T: Serialize + Send + Sync + ?Sized>(&self, key: &str, value: &T) -> KvResult<()>;
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> KvResult<Option<T>>;
    async fn delete(&self, key: &str) -> KvResult<()>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl KvOperations for worker::kv::KvStore {
    async fn put<T: Serialize + Send + ?Sized>(&self, key: &str, value: &T) -> KvResult<()> {
        let serialized = serde_json::to_string(value)?;
        self.put(key, serialized)
            .await
            .map_err(|e| KvOperationError::Storage(e.to_string()))?
            .execute()
            .await
            .map_err(|e| KvOperationError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> KvResult<Option<T>> {
        let result: Option<String> = self
            .get(key)
            .json()
            .await
            .map_err(|e| KvOperationError::Storage(e.to_string()))?;

        match result {
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, key: &str) -> KvResult<()> {
        self.delete(key)
            .await
            .map_err(|e| KvOperationError::Storage(e.to_string()))?;
        Ok(())
    }
}

// Note: For non-WASM targets, we would need a different KV implementation
// that properly implements Send + Sync. The worker::kv::KvStore is designed
// specifically for WASM/Cloudflare Workers environment.
