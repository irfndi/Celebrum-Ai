// src/utils/error.rs

use crate::services::core::trading::kv_operations::KvOperationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub type ArbitrageResult<T> = Result<T, ArbitrageError>;

/// Custom error details for additional context
pub type ErrorDetails = HashMap<String, serde_json::Value>;

/// Main error type for the arbitrage application
/// Optimized for size by boxing large fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageError {
    pub message: String,
    pub details: Option<Box<ErrorDetails>>, // Boxed to reduce enum size
    pub status: Option<u16>,
    pub error_code: Option<String>,
    pub method: Option<String>,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    #[default]
    UnknownError,
    ApiError,
    NetworkError,
    DatabaseError,
    ValidationError,
    AuthenticationError,
    AuthorizationError,
    ConfigurationError,
    SerializationError,
    DeserializationError,
    RateLimitError,
    TimeoutError,
    NotFoundError,
    ConflictError,
    InternalServerError,
    ServiceUnavailable,
    BadRequest,
    ExternalServiceError,
    CacheError,
    StorageError,
    ProcessingError,
    InfrastructureError,
    Cache,
    Storage,
    Internal,
    Service,
}

impl fmt::Display for ArbitrageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ArbitrageError {}

impl ArbitrageError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: None,
            status: None,
            error_code: None,
            method: None,
            kind,
        }
    }

    pub fn with_details(mut self, details: ErrorDetails) -> Self {
        self.details = Some(Box::new(details));
        self
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_code(mut self, error_code: impl Into<String>) -> Self {
        self.error_code = Some(error_code.into());
        self
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    // Convenience constructors for common error types
    pub fn network_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::NetworkError, message)
            .with_status(503)
            .with_code("NETWORK_ERROR")
    }

    pub fn api_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ApiError, message)
            .with_status(500)
            .with_code("API_ERROR")
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ValidationError, message)
            .with_status(400)
            .with_code("VALIDATION_ERROR")
    }

    pub fn not_found<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::NotFoundError, message)
            .with_status(404)
            .with_code("NOT_FOUND")
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthenticationError, message)
            .with_status(401)
            .with_code("AUTH_ERROR")
    }

    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthorizationError, message)
            .with_status(403)
            .with_code("AUTH_Z_ERROR")
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthorizationError, message)
            .with_status(401)
            .with_code("UNAUTHORIZED")
    }

    pub fn rate_limit_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::RateLimitError, message)
            .with_status(429)
            .with_code("RATE_LIMIT")
    }

    pub fn exchange_error(exchange: &str, message: impl Into<String>) -> Self {
        let mut details = ErrorDetails::new();
        details.insert(
            "exchange".to_string(),
            serde_json::Value::String(exchange.to_string()),
        );

        Self::new(ErrorKind::ExternalServiceError, message)
            .with_details(details)
            .with_status(502)
            .with_code("EXCHANGE_ERROR")
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DeserializationError, message)
            .with_status(400)
            .with_code("PARSE_ERROR")
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
            .with_status(500)
            .with_code("CONFIG_ERROR")
    }

    pub fn database_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DatabaseError, message)
            .with_status(500)
            .with_code("DATABASE_ERROR")
    }

    pub fn telegram_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ExternalServiceError, message)
            .with_status(502)
            .with_code("TELEGRAM_ERROR")
    }

    pub fn internal_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::Internal, message)
            .with_status(500)
            .with_code("INTERNAL_ERROR")
    }

    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InternalServerError, message)
            .with_status(501)
            .with_code("NOT_IMPLEMENTED")
    }

    pub fn serialization_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::SerializationError, message)
            .with_status(400)
            .with_code("SERIALIZATION_ERROR")
    }

    pub fn storage_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::StorageError, message)
    }

    pub fn kv_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::StorageError, message)
            .with_status(500)
            .with_code("KV_ERROR")
    }

    pub fn infrastructure_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::InfrastructureError, message)
            .with_status(500)
            .with_code("INFRASTRUCTURE_ERROR")
    }

    pub fn service_unavailable<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::ServiceUnavailable, message)
    }

    pub fn parsing_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DeserializationError, message)
            .with_status(400)
            .with_code("PARSING_ERROR")
    }

    pub fn configuration_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
    }

    pub fn data_unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFoundError, message)
            .with_status(503)
            .with_code("DATA_UNAVAILABLE")
    }

    pub fn session_not_found(identifier: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::NotFoundError,
            format!("Session not found: {}", identifier.into()),
        )
        .with_status(404)
        .with_code("SESSION_NOT_FOUND")
    }

    pub fn rate_limit_exceeded(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RateLimitError, message)
            .with_status(429)
            .with_code("RATE_LIMIT_EXCEEDED")
    }

    pub fn quota_exceeded(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RateLimitError, message)
            .with_status(429)
            .with_code("QUOTA_EXCEEDED")
    }

    pub fn cache_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::CacheError, message)
    }

    pub fn feature_disabled(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
            .with_status(400)
            .with_code("FEATURE_DISABLED")
    }

    pub fn processing_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ProcessingError, message)
    }

    pub fn access_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthorizationError, message)
            .with_status(403)
            .with_code("ACCESS_DENIED")
    }

    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TimeoutError, message)
            .with_status(408)
            .with_code("TIMEOUT_ERROR")
    }

    pub fn operation_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::ProcessingError, message)
    }

    pub fn initialization_error<T: Into<String>>(message: T) -> Self {
        Self::new(ErrorKind::InfrastructureError, message)
    }

    pub fn service_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Service, message)
    }
}

// Implement From conversions for common error types
impl From<serde_json::Error> for ArbitrageError {
    fn from(err: serde_json::Error) -> Self {
        ArbitrageError::parse_error(format!("JSON parsing error: {}", err))
    }
}

impl From<worker::Error> for ArbitrageError {
    fn from(err: worker::Error) -> Self {
        ArbitrageError::internal_error(format!("Worker error: {:?}", err))
    }
}

impl From<worker::kv::KvError> for ArbitrageError {
    fn from(err: worker::kv::KvError) -> Self {
        Self::storage_error(format!("KV error: {:?}", err))
    }
}

impl From<String> for ArbitrageError {
    fn from(err: String) -> Self {
        Self::validation_error(err)
    }
}

impl From<&str> for ArbitrageError {
    fn from(err: &str) -> Self {
        Self::validation_error(err.to_string())
    }
}

impl From<url::ParseError> for ArbitrageError {
    fn from(err: url::ParseError) -> Self {
        ArbitrageError::validation_error(format!("URL parse error: {}", err))
    }
}

// Implement From<KvOperationError> for ArbitrageError
impl From<KvOperationError> for ArbitrageError {
    fn from(err: KvOperationError) -> Self {
        match err {
            KvOperationError::NotFound => {
                ArbitrageError::not_found("KV item not found".to_string())
            }
            KvOperationError::SerializationError(msg) => ArbitrageError::serialization_error(
                format!("KV serialization/deserialization error: {}", msg),
            ),
            KvOperationError::NetworkError(msg) => {
                ArbitrageError::network_error(format!("KV network error: {}", msg))
            }
            KvOperationError::Unauthorized => {
                ArbitrageError::unauthorized("KV unauthorized access".to_string())
            }
            KvOperationError::RateLimited => {
                ArbitrageError::rate_limit_exceeded("KV rate limited".to_string())
            }
            KvOperationError::ServiceUnavailable => {
                ArbitrageError::service_unavailable("KV service unavailable".to_string())
            }
            KvOperationError::Storage(msg) => ArbitrageError::database_error(msg),
        }
    }
}

// Implementation to convert ArbitrageError into worker::Error
impl From<ArbitrageError> for worker::Error {
    fn from(err: ArbitrageError) -> Self {
        // Log the original error for more detailed debugging if necessary
        // For example, using a logger if available: log::error!("Converting ArbitrageError to worker::Error: {:?}", err);

        // Convert our detailed ArbitrageError into a simpler worker::Error.
        // We'll try to preserve the status code if available, otherwise use a generic one.
        // The message will be the primary information carried over.
        let message = if let Some(status_code) = err.status {
            format!(
                "[Status: {}] ArbitrageError (Kind: {:?}): {}",
                status_code, err.kind, err.message
            )
        } else {
            format!("ArbitrageError (Kind: {:?}): {}", err.kind, err.message)
        };

        worker::Error::RustError(message)
    }
}

// Helper macro for creating errors with context
#[macro_export]
macro_rules! arbitrage_error {
    ($kind:expr, $msg:expr) => {
        ArbitrageError::new($kind, $msg)
    };
    ($kind:expr, $msg:expr, $($key:expr => $value:expr),+) => {{
        let mut details = std::collections::HashMap::new();
        $(
            details.insert($key.to_string(), serde_json::json!($value));
        )+
        ArbitrageError::new($kind, $msg).with_details(details)
    }};
}

// Tests have been moved to packages/worker/tests/utils/error_test.rs
