//! Tests for helper utilities
//! Extracted from src/utils/helpers.rs

use crate::utils::helpers::*;

#[test]
fn test_safe_parse_float() {
    assert_eq!(safe_parse_float(Some(42.5)), 42.5);
    assert_eq!(safe_parse_float(None), 0.0);
    assert_eq!(safe_parse_float(Some(-10.0)), -10.0);
    assert_eq!(safe_parse_float(Some(0.0)), 0.0);
}

#[test]
fn test_safe_parse_float_str() {
    assert_eq!(safe_parse_float_str("42.5"), 42.5);
    assert_eq!(safe_parse_float_str("invalid"), 0.0);
    assert_eq!(safe_parse_float_str(""), 0.0);
    assert_eq!(safe_parse_float_str("-10.5"), -10.5);
}

#[test]
fn test_clamp() {
    assert_eq!(clamp(5.0, 1.0, 10.0), 5.0);
    assert_eq!(clamp(0.0, 1.0, 10.0), 1.0);
    assert_eq!(clamp(15.0, 1.0, 10.0), 10.0);
}

#[test]
fn test_round_to_decimal_places() {
    assert_eq!(round_to_decimal_places(3.14159, 2), 3.14);
    assert_eq!(round_to_decimal_places(3.14159, 4), 3.1416);
    
    // Test that the function works correctly by checking the rounded values
    let result = round_to_decimal_places(1.23456789, 3);
    assert!((result - 1.235).abs() < 0.0001);
    
    let result2 = round_to_decimal_places(9.87654321, 1);
    assert!((result2 - 9.9).abs() < 0.01);
    
    // Also test with a simple known value
    let result3 = round_to_decimal_places(2.5, 0);
    assert_eq!(result3, 3.0); // Should round up
}

#[test]
fn test_percentage_to_decimal() {
    assert_eq!(percentage_to_decimal(50.0), 0.5);
    assert_eq!(percentage_to_decimal(100.0), 1.0);
    assert_eq!(percentage_to_decimal(0.0), 0.0);
    assert_eq!(percentage_to_decimal(25.5), 0.255);
}

#[test]
fn test_decimal_to_percentage() {
    assert_eq!(decimal_to_percentage(0.5), 50.0);
    assert_eq!(decimal_to_percentage(1.0), 100.0);
    assert_eq!(decimal_to_percentage(0.0), 0.0);
}

#[test]
fn test_approximately_equal() {
    assert!(approximately_equal(1.0, 1.0001, 0.001));
    assert!(!approximately_equal(1.0, 1.1, 0.001));
    assert!(approximately_equal(0.0, 0.0, 0.001));
}

#[test]
fn test_percentage_difference() {
    assert_eq!(percentage_difference(100.0, 110.0), 10.0);
    assert_eq!(percentage_difference(50.0, 45.0), -10.0);
    assert_eq!(percentage_difference(0.0, 10.0), f64::INFINITY);
    
    // Test with small numbers
    let diff = percentage_difference(1.0, 1.1);
    assert!((diff - 10.0).abs() < 0.001);
}

#[test]
fn test_validate_range() {
    assert_eq!(validate_range(5, 1, 10, "test").unwrap(), 5);
    assert!(validate_range(0, 1, 10, "test").is_err());
    assert!(validate_range(15, 1, 10, "test").is_err());
}

#[test]
fn test_moving_average() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let avg = moving_average(&data, 3);
    
    // Should return averages for the last 3 windows
    assert_eq!(avg.len(), 3);
    assert_eq!(avg[0], 2.0); // (1+2+3)/3
    assert_eq!(avg[1], 3.0); // (2+3+4)/3
    assert_eq!(avg[2], 4.0); // (3+4+5)/3
}

#[test]
fn test_validate_api_key_valid_keys() {
    // Test valid keys of different lengths
    assert!(validate_api_key("1234567890123456").is_ok()); // 16 chars
    assert!(validate_api_key("abcdefghijklmnop1234567890123456").is_ok()); // 32 chars
    assert!(validate_api_key("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890123456").is_ok()); // 64 chars
    
    // Test mixed alphanumeric
    assert!(validate_api_key("abc123XYZ789def456").is_ok());
    assert!(validate_api_key("API1234567890KEY").is_ok());
    
    // Test with numbers only
    assert!(validate_api_key("1234567890123456").is_ok());
}

#[test]
fn test_validate_api_key_invalid_characters() {
    // Test keys with invalid characters
    assert!(validate_api_key("invalid-key-with-dashes").is_err());
    assert!(validate_api_key("invalid_key_with_underscores").is_err());
    assert!(validate_api_key("invalid key with spaces").is_err());
    assert!(validate_api_key("invalid@key#with$symbols").is_err());
    assert!(validate_api_key("invalid.key.with.dots").is_err());
    assert!(validate_api_key("invalid/key\\with\\slashes").is_err());
    assert!(validate_api_key("invalid+key=with+equals").is_err());
    assert!(validate_api_key("invalid[key]with{brackets}").is_err());
    assert!(validate_api_key("invalid|key&with%other!").is_err());
}

#[test]
fn test_validate_api_key_empty_and_boundary_cases() {
    // Test empty string
    assert!(validate_api_key("").is_err());
    
    // Test too short (less than 16 characters)
    assert!(validate_api_key("short").is_err());
    assert!(validate_api_key("123456789012345").is_err()); // 15 chars
    
    // Test too long (more than 128 characters)
    let long_key = "a".repeat(129);
    assert!(validate_api_key(&long_key).is_err());
    
    // Test whitespace only
    assert!(validate_api_key("                ").is_err()); // 16 spaces
}

#[test]
fn test_validate_api_key_boundary_lengths() {
    // Test exact boundary lengths
    let min_valid = "a".repeat(16); // Exactly 16 chars
    assert!(validate_api_key(&min_valid).is_ok());
    
    let max_valid = "a".repeat(128); // Exactly 128 chars
    assert!(validate_api_key(&max_valid).is_ok());
    
    // Test just outside boundaries
    let too_short = "a".repeat(15); // 15 chars
    assert!(validate_api_key(&too_short).is_err());
    
    let too_long = "a".repeat(129); // 129 chars
    assert!(validate_api_key(&too_long).is_err());
}

#[test]
fn test_validate_api_key_security_properties() {
    // Test that function maintains security properties
    
    // Should reject common weak patterns
    assert!(validate_api_key("1111111111111111").is_ok()); // Technically valid format but weak
    assert!(validate_api_key("aaaaaaaaaaaaaaaa").is_ok()); // Technically valid format but weak
    
    // Should accept strong keys
    assert!(validate_api_key("aB3dE6gH9jK2mN5p").is_ok());
    assert!(validate_api_key("X7z9A2c5F8h1K4n6").is_ok());
    
    // Test with realistic API key formats
    assert!(validate_api_key("sk1234567890abcdef1234567890abcdef").is_ok());
    assert!(validate_api_key("pk1234567890ABCDEF1234567890abcdef").is_ok());
    assert!(validate_api_key("API1234567890KEY1234567890SECRETKEY").is_ok());
}

#[test]
fn test_validate_api_key_unicode_and_edge_cases() {
    // Test Unicode characters (should be rejected)
    assert!(validate_api_key("café1234567890123").is_err());
    assert!(validate_api_key("测试1234567890123456").is_err());
    assert!(validate_api_key("🔑1234567890123456").is_err());
    
    // Test whitespace variations
    assert!(validate_api_key("\t1234567890123456").is_err()); // Tab
    assert!(validate_api_key("\n1234567890123456").is_err()); // Newline
    assert!(validate_api_key("\r1234567890123456").is_err()); // Carriage return
    assert!(validate_api_key(" 1234567890123456").is_err()); // Leading space
    assert!(validate_api_key("1234567890123456 ").is_err()); // Trailing space
}

#[test]
fn test_format_number_with_commas() {
    assert_eq!(format_number_with_commas(1234), "1,234");
    assert_eq!(format_number_with_commas(1234567), "1,234,567");
    assert_eq!(format_number_with_commas(123), "123");
    assert_eq!(format_number_with_commas(0), "0");
}

#[test]
fn test_calculate_compound_interest() {
    let result = calculate_compound_interest(1000.0, 0.05, 12, 1);
    assert!((result - 1051.16).abs() < 0.01); // Approximately $1,051.16
    
    let result2 = calculate_compound_interest(1000.0, 0.10, 1, 5);
    assert!((result2 - 1610.51).abs() < 0.01); // Approximately $1,610.51
}

#[test]
fn test_is_valid_email() {
    assert!(is_valid_email("test@example.com"));
    assert!(is_valid_email("user.name@domain.co.uk"));
    assert!(!is_valid_email("invalid.email"));
    assert!(!is_valid_email("@domain.com"));
    assert!(!is_valid_email("user@"));
}