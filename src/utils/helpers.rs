use serde_json::Value;
use uuid::Uuid;

/// Safely parses a value to a floating-point number.
/// If parsing fails or results in NaN, returns a default value.
pub fn safe_parse_float(value: &Value, default_value: f64) -> f64 {
    match value {
        Value::Null => default_value,
        Value::Number(n) => n.as_f64().unwrap_or(default_value),
        Value::String(s) => {
            if s.trim().is_empty() {
                default_value
            } else {
                s.parse::<f64>().unwrap_or(default_value)
            }
        }
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => default_value,
    }
}

/// Safely parses a string to a floating-point number.
pub fn safe_parse_float_str(value: &str, default_value: f64) -> f64 {
    if value.trim().is_empty() {
        return default_value;
    }

    value.parse::<f64>().unwrap_or(default_value)
}

/// Safely parses an optional string to a floating-point number.
pub fn safe_parse_float_opt(value: Option<&str>, default_value: f64) -> f64 {
    match value {
        Some(s) => safe_parse_float_str(s, default_value),
        None => default_value,
    }
}

/// Performs a deep clone of a JSON-serializable value.
/// This is equivalent to JSON.parse(JSON.stringify()) in JavaScript.
pub fn deep_clone<T>(value: &T) -> Result<T, serde_json::Error>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    let json_str = serde_json::to_string(value)?;
    serde_json::from_str(&json_str)
}

/// Clamps a value between a minimum and maximum
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Rounds a float to a specified number of decimal places
pub fn round_to_decimal_places(value: f64, decimal_places: u32) -> f64 {
    let multiplier = 10_f64.powi(decimal_places as i32);
    (value * multiplier).round() / multiplier
}

/// Converts a percentage string (e.g., "1.5%") to a decimal
pub fn percentage_to_decimal(percentage_str: &str) -> Result<f64, String> {
    let cleaned = percentage_str.trim().trim_end_matches('%');
    cleaned
        .parse::<f64>()
        .map(|p| p / 100.0)
        .map_err(|_| format!("Invalid percentage format: {}", percentage_str))
}

/// Converts a decimal to a percentage string
pub fn decimal_to_percentage(decimal: f64, decimal_places: u32) -> String {
    format!(
        "{:.prec$}%",
        decimal * 100.0,
        prec = decimal_places as usize
    )
}

/// Checks if a float is approximately equal to another within a tolerance
pub fn approximately_equal(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// Calculates the absolute percentage difference between two values
/// Returns None when one value is zero and the other is not (undefined percentage)
pub fn percentage_difference(value1: f64, value2: f64) -> Option<f64> {
    if value1 == 0.0 && value2 == 0.0 {
        Some(0.0)
    } else if value1 == 0.0 || value2 == 0.0 {
        None // Undefined percentage difference when one value is zero
    } else {
        Some(((value1 - value2).abs() / ((value1 + value2) / 2.0)) * 100.0)
    }
}

/// Validates that a value is within a specific range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<T, String> {
    if value < min || value > max {
        Err(format!(
            "{} must be between {} and {}, got {}",
            field_name, min, max, value
        ))
    } else {
        Ok(value)
    }
}

/// Calculates the moving average of a slice of values
pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    if window_size == 0 || values.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for i in 0..values.len() {
        let start = (i + 1).saturating_sub(window_size);
        let end = i + 1;
        let window = &values[start..end];
        let avg = window.iter().sum::<f64>() / window.len() as f64;
        result.push(avg);
    }
    result
}

/// Generate a new UUID string
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a new API key (32 character random string)
pub fn generate_api_key() -> String {
    use rand::rngs::OsRng;
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = OsRng;

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a new secret key (64 character random string)
pub fn generate_secret_key() -> String {
    use rand::rngs::OsRng;
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = OsRng;

    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Validate an API key format (basic validation)
pub fn validate_api_key(api_key: &str) -> bool {
    // Basic format validation
    if api_key.is_empty()
        || api_key.len() < 16
        || api_key.len() > 128
        || !api_key.chars().all(|c| c.is_alphanumeric())
    {
        return false;
    }

    // Security checks - reject specific weak patterns
    if api_key.chars().all(|c| c == '1') {
        return false; // Reject keys with all 1s like "1111111111111111"
    }

    true
}

// Tests have been moved to packages/worker/tests/utils/helpers_test.rs
