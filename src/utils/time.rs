// src/utils/time.rs

use chrono::{DateTime, Utc};

/// Service for handling time-related operations.
#[derive(Debug, Clone)]
pub struct TimeService;

impl TimeService {
    /// Creates a new instance of `TimeService`.
    pub fn new() -> Self {
        TimeService
    }

    /// Gets the current UTC date and time.
    pub fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    /// Gets the current timestamp in seconds since Unix epoch.
    pub fn current_timestamp(&self) -> u64 {
        Utc::now().timestamp() as u64
    }

    /// Gets the current timestamp in milliseconds since Unix epoch.
    pub fn current_timestamp_ms(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}

/// Gets the current timestamp in seconds since Unix epoch (standalone function).
pub fn get_current_timestamp() -> u64 {
    Utc::now().timestamp() as u64
}

impl Default for TimeService {
    fn default() -> Self {
        Self::new()
    }
}
