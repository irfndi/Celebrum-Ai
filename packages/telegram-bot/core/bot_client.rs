// src/services/interfaces/telegram/core/bot_client.rs

//! Telegram Bot API Client
//!
//! Handles all communication with the Telegram Bot API including:
//! - Sending messages
//! - API requests
//! - Error handling and retries
//! - Rate limiting

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use worker::console_log;

/// Result type for Telegram operations
pub type TelegramResult<T> = Result<T, TelegramError>;

/// Error types for Telegram operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum TelegramError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    Api(String),
    Timeout,
    RateLimit,
}

impl From<reqwest::Error> for TelegramError {
    fn from(err: reqwest::Error) -> Self {
        TelegramError::Http(err)
    }
}

impl From<serde_json::Error> for TelegramError {
    fn from(err: serde_json::Error) -> Self {
        TelegramError::Json(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
    pub is_test_mode: bool,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            chat_id: String::new(),
            is_test_mode: true,
        }
    }
}

// Simple retry configuration for Telegram API
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for retry attempt
    #[allow(dead_code)]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = Duration::from_millis(self.base_delay_ms);
        let backoff_multiplier = 2.0f64.powf(attempt as f64 - 1.0);
        let delay_ms = (base_delay.as_millis() as f64 * backoff_multiplier) as u64;
        Duration::from_millis(delay_ms.min(self.max_attempts as u64 * self.base_delay_ms))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RateLimitEntry {
    pub last_request: Instant,
    pub request_count: u32,
    pub window_start: Instant,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheEntry<T> {
    pub value: T,
    pub expires_at: Instant,
}

/// Telegram Bot API Client
#[allow(dead_code)]
pub struct TelegramBotClient {
    config: TelegramConfig,
    http_client: Client,
    retry_config: RetryConfig,
}

impl TelegramBotClient {
    #[allow(dead_code)]
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Send a message to Telegram
    #[allow(dead_code)]
    pub async fn send_message(
        &self,
        chat_id: &str,
        text: &str,
        parse_mode: Option<&str>,
        reply_markup: Option<Value>,
    ) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let mut payload = json!({
            "chat_id": chat_id,
            "text": text,
        });

        if let Some(mode) = parse_mode {
            payload["parse_mode"] = json!(mode);
        }

        if let Some(markup) = reply_markup {
            payload["reply_markup"] = markup;
        }

        self.make_api_request(&url, payload).await
    }

    /// Send a photo to Telegram
    #[allow(dead_code)]
    pub async fn send_photo(
        &self,
        chat_id: &str,
        photo_url: &str,
        caption: Option<&str>,
    ) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendPhoto",
            self.config.bot_token
        );

        let mut payload = json!({
            "chat_id": chat_id,
            "photo": photo_url,
        });

        if let Some(cap) = caption {
            payload["caption"] = json!(cap);
        }

        self.make_api_request(&url, payload).await
    }

    /// Edit an existing message
    #[allow(dead_code)]
    pub async fn edit_message(
        &self,
        chat_id: &str,
        message_id: i64,
        text: &str,
        parse_mode: Option<&str>,
        reply_markup: Option<Value>,
    ) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/editMessageText",
            self.config.bot_token
        );

        let mut payload = json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": text,
        });

        if let Some(mode) = parse_mode {
            payload["parse_mode"] = json!(mode);
        }

        if let Some(markup) = reply_markup {
            payload["reply_markup"] = markup;
        }

        self.make_api_request(&url, payload).await
    }

    /// Answer callback query
    #[allow(dead_code)]
    pub async fn answer_callback_query(
        &self,
        callback_query_id: &str,
        text: Option<&str>,
        show_alert: bool,
    ) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/answerCallbackQuery",
            self.config.bot_token
        );

        let mut payload = json!({
            "callback_query_id": callback_query_id,
            "show_alert": show_alert,
        });

        if let Some(text) = text {
            payload["text"] = json!(text);
        }

        self.make_api_request(&url, payload).await
    }

    /// Make API request with retry logic
    async fn make_api_request(&self, url: &str, payload: Value) -> TelegramResult<Value> {
        let mut last_error = None;

        for attempt in 0..=self.retry_config.max_attempts {
            match self.send_request(url, &payload).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.retry_config.max_attempts {
                        let delay = self.calculate_retry_delay(attempt);
                        console_log!(
                            "🔄 Telegram API request failed, retrying in {}ms (attempt {}/{})",
                            delay.as_millis(),
                            attempt + 1,
                            self.retry_config.max_attempts
                        );

                        // WASM-compatible sleep
                        #[cfg(target_arch = "wasm32")]
                        {
                            use worker::Delay;
                            Delay::from(Duration::from_millis(delay.as_millis() as u64)).await;
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TelegramError::Api("Failed to make Telegram API request after retries".to_string())
        }))
    }

    /// Send HTTP request to Telegram API
    async fn send_request(&self, url: &str, payload: &Value) -> TelegramResult<Value> {
        if self.config.is_test_mode {
            console_log!("🧪 Test mode: Would send to Telegram: {}", payload);
            return Ok(json!({"ok": true, "result": {"message_id": 12345}}));
        }

        let response = self
            .http_client
            .post(url)
            .json(payload)
            .send()
            .await
            .map_err(|e| TelegramError::Api(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| TelegramError::Api(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(TelegramError::Api(format!(
                "Telegram API error {}: {}",
                status, response_text
            )));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| TelegramError::Api(format!("Failed to parse response: {}", e)))
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        self.retry_config.calculate_delay(attempt)
    }

    /// Get bot info
    #[allow(dead_code)]
    pub async fn get_me(&self) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/getMe",
            self.config.bot_token
        );

        self.make_api_request(&url, json!({})).await
    }

    /// Set webhook URL
    #[allow(dead_code)]
    pub async fn set_webhook(&self, webhook_url: &str) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/setWebhook",
            self.config.bot_token
        );

        let payload = json!({
            "url": webhook_url,
            "allowed_updates": ["message", "callback_query"]
        });

        self.make_api_request(&url, payload).await
    }

    /// Delete webhook
    #[allow(dead_code)]
    pub async fn delete_webhook(&self) -> TelegramResult<Value> {
        let url = format!(
            "https://api.telegram.org/bot{}/deleteWebhook",
            self.config.bot_token
        );

        self.make_api_request(&url, json!({})).await
    }
}
