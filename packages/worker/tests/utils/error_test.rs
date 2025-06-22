use cerebrum_ai::utils::error::*;
use serde_json::{json, Value};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrage_error_creation() {
        let error = ArbitrageError::new(ErrorKind::NetworkError, "Network issue");
        assert_eq!(error.kind, ErrorKind::NetworkError);
        assert_eq!(error.message, "Network issue");
        assert!(error.details.is_none());
        assert!(error.status.is_none()); // Status is not set by default by `new`
    }

    #[test]
    fn test_error_with_details() {
        let mut details = HashMap::new();
        details.insert("info".to_string(), json!("extra data"));
        let error = ArbitrageError::new(ErrorKind::ValidationError, "Validation failed")
            .with_details(details.clone());
        assert_eq!(error.kind, ErrorKind::ValidationError);
        assert_eq!(*error.details.unwrap(), details);
    }

    #[test]
    fn test_error_with_status() {
        let error =
            ArbitrageError::new(ErrorKind::NotFoundError, "Item not found").with_status(404);
        assert_eq!(error.kind, ErrorKind::NotFoundError);
        assert_eq!(error.status, Some(404));
    }

    #[test]
    fn test_error_with_code() {
        let error = ArbitrageError::new(ErrorKind::ApiError, "API problem").with_code("API_001");
        assert_eq!(error.kind, ErrorKind::ApiError);
        assert_eq!(error.error_code, Some("API_001".to_string()));
    }

    #[test]
    fn test_convenience_constructors() {
        let net_err = ArbitrageError::network_error("Timeout");
        assert_eq!(net_err.kind, ErrorKind::NetworkError);
        assert_eq!(net_err.status, Some(503));
        assert_eq!(net_err.error_code, Some("NETWORK_ERROR".to_string()));

        let val_err = ArbitrageError::validation_error("Bad input");
        assert_eq!(val_err.kind, ErrorKind::ValidationError);
        assert_eq!(val_err.status, Some(400));

        let nf_err = ArbitrageError::not_found("Resource missing");
        assert_eq!(nf_err.kind, ErrorKind::NotFoundError);
        assert_eq!(nf_err.status, Some(404));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err_str = "{\"key\": invalid_json}"; // Malformed JSON
        let serde_error = serde_json::from_str::<Value>(json_err_str).unwrap_err();
        let arbitrage_error = ArbitrageError::from(serde_error);
        assert_eq!(arbitrage_error.kind, ErrorKind::DeserializationError);
        assert!(arbitrage_error.message.contains("JSON parsing error"));
    }

    #[test]
    fn test_from_worker_error() {
        let worker_err = worker::Error::RustError("Worker failed".to_string());
        let arbitrage_error = ArbitrageError::from(worker_err);
        assert_eq!(arbitrage_error.kind, ErrorKind::Internal);
        assert!(arbitrage_error.message.contains("Worker error"));
    }

    #[test]
    fn test_from_kv_operation_error_not_found() {
        let kv_err = KvOperationError::NotFound;
        let arb_err = ArbitrageError::from(kv_err);
        assert_eq!(arb_err.kind, ErrorKind::NotFoundError);
        assert_eq!(arb_err.status, Some(404));
        assert!(arb_err.message.contains("KV item not found"));
    }

    #[test]
    fn test_from_kv_operation_error_serialization() {
        // Create a real serde_json::Error by trying to parse malformed JSON
        let malformed_json = "{\"key\": invalid_value}"; // Missing quotes around invalid_value
        let json_error = serde_json::from_str::<Value>(malformed_json).unwrap_err();
        let kv_err = KvOperationError::SerializationError(json_error.to_string());
        let arb_err = ArbitrageError::from(kv_err);
        assert_eq!(arb_err.kind, ErrorKind::SerializationError);
        assert_eq!(arb_err.status, Some(400)); // Assuming serialization error maps to 400
        assert!(arb_err
            .message
            .contains("KV serialization/deserialization error"));
    }

    #[test]
    fn test_into_worker_error() {
        let arbitrage_error =
            ArbitrageError::internal_error("Something went wrong").with_status(500);
        let worker_error: worker::Error = arbitrage_error.into();

        match worker_error {
            worker::Error::RustError(msg) => {
                assert!(msg.contains("[Status: 500]"));
                assert!(msg.contains("ArbitrageError (Kind: Internal): Something went wrong"));
            }
            _ => panic!("Expected RustError variant"),
        }

        let arbitrage_error_no_status = ArbitrageError::network_error("Network down");
        // Clear status to test the other branch, network_error sets a status by default
        let arbitrage_error_no_status = ArbitrageError {
            status: None,
            ..arbitrage_error_no_status
        };
        let worker_error_no_status: worker::Error = arbitrage_error_no_status.into();
        match worker_error_no_status {
            worker::Error::RustError(msg) => {
                assert!(msg.contains("ArbitrageError (Kind: NetworkError): Network down"));
            }
            _ => panic!("Expected RustError variant for error with no status code"),
        }
    }

    #[test]
    fn test_arbitrage_error_macro() {
        let error_simple = arbitrage_error!(ErrorKind::ConfigurationError, "Bad config file");
        assert_eq!(error_simple.kind, ErrorKind::ConfigurationError);
        assert_eq!(error_simple.message, "Bad config file");

        let error_with_details = arbitrage_error!(
            ErrorKind::DatabaseError,
            "Query failed",
            "query" => "SELECT * FROM users",
            "params" => vec!["id1", "id2"]
        );
        assert_eq!(error_with_details.kind, ErrorKind::DatabaseError);
        assert_eq!(error_with_details.message, "Query failed");
        let details = error_with_details.details.unwrap();
        assert_eq!(details.get("query").unwrap(), &json!("SELECT * FROM users"));
        assert_eq!(details.get("params").unwrap(), &json!(vec!["id1", "id2"]));
    }
}
