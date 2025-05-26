use crate::utils::error::ArbitrageError;
use crate::utils::ArbitrageResult;
use worker::kv::KvStore;

/// Simple KV service wrapper for consistent KV operations
pub struct KVService {
    kv_store: KvStore,
}

impl KVService {
    pub fn new(kv_store: KvStore) -> Self {
        Self { kv_store }
    }

    /// Get a value from KV store
    pub async fn get(&self, key: &str) -> ArbitrageResult<Option<String>> {
        match self.kv_store.get(key).text().await {
            Ok(value) => Ok(value),
            Err(e) => Err(ArbitrageError::storage_error(format!(
                "KV get error: {:?}",
                e
            ))),
        }
    }

    /// Put a value into KV store with optional TTL
    pub async fn put(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> ArbitrageResult<()> {
        let mut put_request = self.kv_store.put(key, value)?;

        if let Some(ttl) = ttl_seconds {
            put_request = put_request.expiration_ttl(ttl);
        }

        match put_request.execute().await {
            Ok(_) => Ok(()),
            Err(e) => Err(ArbitrageError::storage_error(format!(
                "KV put error: {:?}",
                e
            ))),
        }
    }

    /// Delete a value from KV store
    pub async fn delete(&self, key: &str) -> ArbitrageResult<()> {
        match self.kv_store.delete(key).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ArbitrageError::storage_error(format!(
                "KV delete error: {:?}",
                e
            ))),
        }
    }

    /// List keys with a prefix
    pub async fn list_keys(
        &self,
        prefix: &str,
        limit: Option<u64>,
    ) -> ArbitrageResult<Vec<String>> {
        let mut list_request = self.kv_store.list().prefix(prefix.to_string());

        if let Some(limit) = limit {
            list_request = list_request.limit(limit);
        }

        match list_request.execute().await {
            Ok(list_result) => {
                let keys = list_result.keys.into_iter().map(|key| key.name).collect();
                Ok(keys)
            }
            Err(e) => Err(ArbitrageError::storage_error(format!(
                "KV list error: {:?}",
                e
            ))),
        }
    }
}
