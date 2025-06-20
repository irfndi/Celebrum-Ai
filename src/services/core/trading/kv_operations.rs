use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub enum KvOperationError {
    NotFound,
    SerializationError(String),
    NetworkError(String),
    Unauthorized,
    RateLimited,
    ServiceUnavailable,
    Storage(String),
}

impl std::fmt::Display for KvOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvOperationError::NotFound => write!(f, "Key not found"),
            KvOperationError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            KvOperationError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            KvOperationError::Unauthorized => write!(f, "Unauthorized access"),
            KvOperationError::RateLimited => write!(f, "Rate limited"),
            KvOperationError::ServiceUnavailable => write!(f, "Service unavailable"),
            KvOperationError::Storage(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for KvOperationError {}

impl From<serde_json::Error> for KvOperationError {
    fn from(err: serde_json::Error) -> Self {
        KvOperationError::SerializationError(err.to_string())
    }
}

// Define a generic Result type for KV operations
pub type KvResult<T> = Result<T, KvOperationError>;

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
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
