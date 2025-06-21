// Helper utility unit tests
use crate::utils::helpers::*;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_safe_parse_float() {
        assert_eq!(safe_parse_float(&json!(42.5), 0.0), 42.5);
        assert_eq!(safe_parse_float(&json!("123.45"), 0.0), 123.45);
        assert_eq!(safe_parse_float(&json!(null), 10.0), 10.0);
        assert_eq!(safe_parse_float(&json!(""), 5.0), 5.0);
        assert_eq!(safe_parse_float(&json!("invalid"), 7.0), 7.0);
        assert_eq!(safe_parse_float(&json!(true), 0.0), 1.0);
        assert_eq!(safe_parse_float(&json!(false), 0.0), 0.0);
    }

    #[test]
    fn test_safe_parse_float_str() {
        assert_eq!(safe_parse_float_str("123.45", 0.0), 123.45);
        assert_eq!(safe_parse_float_str("", 10.0), 10.0);
        assert_eq!(safe_parse_float_str("  ", 5.0), 5.0);
        assert_eq!(safe_parse_float_str("invalid", 7.0), 7.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 1, 10), 5);
        assert_eq!(clamp(0, 1, 10), 1);
        assert_eq!(clamp(15, 1, 10), 10);
    }

    #[test]
    fn test_round_to_decimal_places() {
        let pi_2_decimal = round_to_decimal_places(std::f64::consts::PI, 2);
        let pi_4_decimal = round_to_decimal_places(std::f64::consts::PI, 4);

        // Test that the function works correctly by checking the rounded values
        // Compute expected values to avoid hardcoded PI approximations
        let expected_2_decimal = (std::f64::consts::PI * 100.0).round() / 100.0;
        let expected_4_decimal = (std::f64::consts::PI * 10000.0).round() / 10000.0;

        assert_eq!(pi_2_decimal, expected_2_decimal);
        assert_eq!(pi_4_decimal, expected_4_decimal);

        // Also test with a simple known value
        assert_eq!(round_to_decimal_places(2.56789, 2), 2.57);
        assert_eq!(round_to_decimal_places(2.56789, 3), 2.568);
    }

    #[test]
    fn test_percentage_to_decimal() {
        assert_eq!(percentage_to_decimal("50%").unwrap(), 0.5);
        assert_eq!(percentage_to_decimal("1.5%").unwrap(), 0.015);
        assert_eq!(percentage_to_decimal("100").unwrap(), 1.0);
        assert!(percentage_to_decimal("invalid%").is_err());
    }

    #[test]
    fn test_decimal_to_percentage() {
        assert_eq!(decimal_to_percentage(0.5, 1), "50.0%");
        assert_eq!(decimal_to_percentage(0.015, 3), "1.500%");
    }

    #[test]
    fn test_approximately_equal() {
        assert!(approximately_equal(1.0, 1.001, 0.01));
        assert!(!approximately_equal(1.0, 1.1, 0.01));
    }

    #[test]
    fn test_percentage_difference() {
        let expected = 200.0 * 10.0 / 190.0; // ~10.526315789473685
        let actual = percentage_difference(100.0, 90.0).unwrap();
        assert!(approximately_equal(actual, expected, 1e-10));
        assert_eq!(percentage_difference(0.0, 0.0), Some(0.0));
        assert_eq!(percentage_difference(100.0, 0.0), None);
        assert_eq!(percentage_difference(0.0, 100.0), None);
    }

    #[test]
    fn test_validate_range() {
        assert_eq!(validate_range(5, 1, 10, "test").unwrap(), 5);
        assert!(validate_range(0, 1, 10, "test").is_err());
        assert!(validate_range(15, 1, 10, "test").is_err());
    }

    #[test]
    fn test_moving_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = moving_average(&values, 3);
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], 1.0); // [1] avg = 1
        assert_eq!(result[1], 1.5); // [1,2] avg = 1.5
        assert_eq!(result[2], 2.0); // [1,2,3] avg = 2
        assert_eq!(result[3], 3.0); // [2,3,4] avg = 3
        assert_eq!(result[4], 4.0); // [3,4,5] avg = 4
    }

    #[test]
    fn test_validate_api_key_valid_keys() {
        // Test valid keys of different lengths
        assert!(validate_api_key("abcdef1234567890")); // 16 chars (minimum)
        assert!(validate_api_key("ABCDEFabcdef1234567890123456")); // 26 chars
        assert!(validate_api_key("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6A7B8C9D0E1F2G3H4I5J6K7L8M9N0O1P2Q3R4S5T6U7V8W9X0Y1Z2")); // 128 chars (maximum)

        // Test mixed alphanumeric
        assert!(validate_api_key("ABC123def456GHI789"));
        assert!(validate_api_key("1234567890abcdef"));
        assert!(validate_api_key("ABCDEFGHIJKLMNOP"));
    }

    #[test]
    fn test_validate_api_key_invalid_characters() {
        // Test keys with invalid characters
        assert!(!validate_api_key("abc-def-123")); // Contains hyphens
        assert!(!validate_api_key("abc_def_123")); // Contains underscores
        assert!(!validate_api_key("abc def 123")); // Contains spaces
        assert!(!validate_api_key("abc@def#123")); // Contains special characters
        assert!(!validate_api_key("abc.def.123")); // Contains dots
        assert!(!validate_api_key("abc+def=123")); // Contains plus and equals
        assert!(!validate_api_key("abc/def\\123")); // Contains slashes
        assert!(!validate_api_key("abc!def?123")); // Contains punctuation
    }

    #[test]
    fn test_validate_api_key_empty_and_boundary_cases() {
        // Test empty string
        assert!(!validate_api_key(""));

        // Test too short (less than 16 characters)
        assert!(!validate_api_key("a")); // 1 char
        assert!(!validate_api_key("abc123")); // 6 chars
        assert!(!validate_api_key("abcdef123456789")); // 15 chars (just under minimum)

        // Test too long (more than 128 characters)
        let too_long = "a".repeat(129);
        assert!(!validate_api_key(&too_long));

        let way_too_long = "a".repeat(256);
        assert!(!validate_api_key(&way_too_long));
    }

    #[test]
    fn test_validate_api_key_boundary_lengths() {
        // Test exact boundary lengths
        let min_length = "a".repeat(16); // Exactly 16 chars
        assert!(validate_api_key(&min_length));

        let max_length = "a".repeat(128); // Exactly 128 chars
        assert!(validate_api_key(&max_length));

        // Test just outside boundaries
        let under_min = "a".repeat(15); // 15 chars
        assert!(!validate_api_key(&under_min));

        let over_max = "a".repeat(129); // 129 chars
        assert!(!validate_api_key(&over_max));
    }

    #[test]
    fn test_validate_api_key_security_properties() {
        // Test that function maintains security properties

        // Should reject common weak patterns
        assert!(!validate_api_key("1111111111111111")); // All same digit (but still valid format)
                                                        // Note: The function only validates format, not strength

        // Should accept properly formatted keys regardless of content
        assert!(validate_api_key("0000000000000000")); // All zeros but valid format
        assert!(validate_api_key("aaaaaaaaaaaaaaaa")); // All same letter but valid format

        // Test with realistic API key formats
        assert!(validate_api_key("sk1234567890abcdef1234567890abcd")); // 32 chars
        assert!(validate_api_key(
            "pk1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd"
        )); // 64 chars
    }

    #[test]
    fn test_validate_api_key_unicode_and_edge_cases() {
        // Test Unicode characters (should be rejected)
        assert!(!validate_api_key("abc123αβγ456")); // Greek letters
        assert!(!validate_api_key("abc123中文456")); // Chinese characters
        assert!(!validate_api_key("abc123🔑456")); // Emoji

        // Test whitespace variations
        assert!(!validate_api_key(" abcdef1234567890")); // Leading space
        assert!(!validate_api_key("abcdef1234567890 ")); // Trailing space
        assert!(!validate_api_key("abcd ef1234567890")); // Internal space
        assert!(!validate_api_key("\tabcdef1234567890")); // Tab character
        assert!(!validate_api_key("abcdef1234567890\n")); // Newline character
    }
}