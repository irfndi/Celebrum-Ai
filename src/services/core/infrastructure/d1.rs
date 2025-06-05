//! D1 Database Service Module
//!
//! Provides a high-level interface for Cloudflare D1 database operations
//! with connection pooling, transaction management, and error handling.

use crate::utils::error::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Env};

/// D1 Database Service for high-level database operations
pub struct D1Service {
    database: D1Database,
    #[allow(dead_code)] // Will be used for connection management
    connection_pool_size: u32,
    #[allow(dead_code)] // Will be used for query timeout handling
    query_timeout_ms: u64,
}

/// D1 Service Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1ServiceConfig {
    pub database_name: String,
    pub connection_pool_size: u32,
    pub query_timeout_ms: u64,
    pub enable_query_logging: bool,
    pub max_retry_attempts: u32,
}

impl Default for D1ServiceConfig {
    fn default() -> Self {
        Self {
            database_name: "ArbEdgeDB".to_string(),
            connection_pool_size: 10,
            query_timeout_ms: 30000,
            enable_query_logging: true,
            max_retry_attempts: 3,
        }
    }
}

impl D1Service {
    /// Create a new D1 service instance
    pub async fn new(env: &Env, config: D1ServiceConfig) -> ArbitrageResult<Self> {
        let database = env.d1(&config.database_name).map_err(|e| {
            ArbitrageError::database_error(format!("Failed to get D1 database: {}", e))
        })?;

        Ok(Self {
            database,
            connection_pool_size: config.connection_pool_size,
            query_timeout_ms: config.query_timeout_ms,
        })
    }

    /// Execute a query and return the first result
    pub async fn query_first<T>(&self, query: &str, params: &[&str]) -> ArbitrageResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let statement = self.database.prepare(query);
        let mut bound_statement = statement;

        for param in params {
            bound_statement = bound_statement.bind(&[JsValue::from(*param)])?;
        }

        let result = bound_statement
            .first::<T>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("D1 query failed: {}", e)))?;

        Ok(result)
    }

    /// Execute a query and return all results
    pub async fn query_all<T>(&self, query: &str, params: &[&str]) -> ArbitrageResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let statement = self.database.prepare(query);
        let mut bound_statement = statement;

        for param in params {
            bound_statement = bound_statement.bind(&[JsValue::from(*param)])?;
        }

        let result = bound_statement
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("D1 query failed: {}", e)))?;

        let results = result.results::<T>().map_err(|e| {
            ArbitrageError::database_error(format!("D1 result parsing failed: {}", e))
        })?;

        Ok(results)
    }

    /// Execute a write operation (INSERT, UPDATE, DELETE)
    pub async fn execute(&self, query: &str, params: &[&str]) -> ArbitrageResult<u64> {
        let statement = self.database.prepare(query);
        let mut bound_statement = statement;

        for param in params {
            bound_statement = bound_statement.bind(&[JsValue::from(*param)])?;
        }

        let result = bound_statement
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("D1 execute failed: {}", e)))?;

        Ok(result.meta().map_or(0, |m| m.unwrap().changes.unwrap_or(0)) as u64)
    }

    /// Get the underlying D1 database instance
    pub fn get_database(&self) -> &D1Database {
        &self.database
    }

    /// Perform a health check on the D1 database
    pub async fn health_check(&self) -> bool {
        match self
            .query_first::<serde_json::Value>("SELECT 1 as health", &[])
            .await
        {
            Ok(Some(_)) => true,
            _ => false,
        }
    }
}
