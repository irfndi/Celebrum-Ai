use crate::services::core::infrastructure::persistence_layer::utils::*;

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