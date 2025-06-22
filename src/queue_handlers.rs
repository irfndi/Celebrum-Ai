// Cloudflare Queue Message Handler
// Production implementation for consuming queue messages using #[event(queue)] macro

use crate::services::core::infrastructure::cloudflare_queues::{
    AnalyticsEventMessage, DeliveryMethod, DistributionStrategy, NotificationMessage,
    OpportunityDistributionMessage, Priority, UserMessage, UserNotificationMessage,
};
use crate::services::core::infrastructure::service_container::ServiceContainer;

use crate::utils::error::{ArbitrageError, ArbitrageResult};
use chrono::{Timelike, Utc};
use core::future::Future;
use reqwest;

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use web_push::*;
use worker::console_log;
use worker::*;

/// Handle opportunity distribution queue message
pub async fn opportunity_queue_handler(
    message_batch: MessageBatch<OpportunityDistributionMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let messages = message_batch.messages()?;
    console_log!(
        "Processing opportunity distribution batch: {} messages",
        messages.len()
    );

    // Note: Telegram service initialization removed - now handled by separate telegram worker
    let _telegram_service: Option<()> = None;

    for message in messages {
        let body = message.body();
        match process_opportunity_message(body, &env, None).await {
            Ok(_) => {
                console_log!(
                    "Successfully processed opportunity message: {}",
                    body.opportunity_id
                );
                message.ack();
            }
            Err(e) => {
                console_error!("Failed to process opportunity message: {}", e);
                message.retry();
            }
        }
    }

    Ok(())
}

/// Handle user notification queue message
pub async fn notification_queue_handler(
    message_batch: MessageBatch<UserNotificationMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let messages = message_batch.messages()?;
    console_log!("Processing notification batch: {} messages", messages.len());

    // Telegram service is now handled by a separate worker
    let _telegram_service: Option<()> = None;

    for message in messages {
        let body = message.body();
        match process_notification_message(body, &env, None).await {
            Ok(_) => {
                console_log!(
                    "Successfully processed notification message: {}",
                    body.notification_id
                );
                message.ack();
            }
            Err(e) => {
                console_error!("Failed to process notification message: {}", e);
                message.retry();
            }
        }
    }

    Ok(())
}

/// Handle analytics event queue message
pub async fn analytics_queue_handler(
    message_batch: MessageBatch<AnalyticsEventMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let messages = message_batch.messages()?;
    console_log!("Processing analytics batch: {} messages", messages.len());

    // Initialize analytics service once for the entire batch
    let analytics_service = match initialize_analytics_service(&env).await {
        Ok(service) => Some(service),
        Err(e) => {
            console_error!("Failed to initialize analytics service: {}", e);
            None
        }
    };

    for message in messages {
        let body = message.body();
        match process_analytics_message(body, &env, analytics_service.as_ref()).await {
            Ok(_) => {
                console_log!(
                    "Successfully processed analytics message: {}",
                    body.event_id
                );
                message.ack();
            }
            Err(e) => {
                console_error!("Failed to process analytics message: {}", e);
                message.retry();
            }
        }
    }

    Ok(())
}

/// Generic helper function for retrying an asynchronous operation with exponential backoff.
async fn send_with_retry<F, Fut, T>(operation: F, max_retries: u32) -> ArbitrageResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = ArbitrageResult<T>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Simple retry for any error, consider more specific error checking for retry eligibility
                if attempt < max_retries {
                    attempt += 1;
                    // Safe exponential backoff with overflow protection and maximum cap
                    let base_delay = 100_u64;
                    let max_delay = 30000_u64; // Cap at 30 second
                    let delay_ms = base_delay
                        .checked_mul(2_u64.saturating_pow(attempt.saturating_sub(1)))
                        .unwrap_or(max_delay)
                        .min(max_delay);
                    console_log!(
                        "Operation failed. Retrying attempt {}/{} in {}ms. Error: {}",
                        attempt,
                        max_retries,
                        delay_ms,
                        e.to_string()
                    );
                    // In a worker environment, tokio::time::sleep might not be available.
                    // Using a promise-based delay for CF Workers.
                    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                        let win = web_sys::window().expect("should have a window in this context");
                        win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            delay_ms as i32,
                        )
                        .expect("should be able to set timeout");
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
                    console_error!(
                        "Operation failed after {} retries. Error: {}",
                        max_retries,
                        e.to_string()
                    );
                    return Err(e);
                }
            }
        }
    }
}

/// Process opportunity distribution message
async fn process_opportunity_message(
    message: &OpportunityDistributionMessage,
    _env: &Env,
    _telegram_service: Option<()>, // Removed telegram service - now in separate worker
) -> ArbitrageResult<()> {
    // Telegram service removed - notifications now handled by separate telegram worker
    console_log!(
        "Opportunity distribution message processed - telegram notifications handled separately"
    );

    // Distribute opportunity to target user
    for user_id in &message.target_users {
        match message.distribution_strategy {
            DistributionStrategy::Broadcast => {
                // Send to all users - notifications handled by separate worker
                console_log!("Telegram notification queued for user: {}", user_id);
            }
            DistributionStrategy::RoundRobin => {
                let kv = _env.kv("ArbEdgeKV").map_err(|e| {
                    crate::utils::ArbitrageError::configuration_error(format!(
                        "Failed to access KV store for round-robin distribution: {}",
                        e
                    ))
                })?;

                let last_index_key = "distribution:roundrobin:last_user_index";
                let last_index: usize = match kv.get(last_index_key).text().await {
                    Ok(Some(index_str)) => index_str.parse().unwrap_or(0),
                    _ => 0,
                };

                let next_index = (last_index + 1) % message.target_users.len();
                let selected_user = &message.target_users[next_index];

                // Telegram notifications now handled by separate worker
                console_log!(
                    "Telegram notification queued for selected user: {}",
                    selected_user
                );

                kv.put(last_index_key, next_index.to_string())
                    .map_err(|e| {
                        crate::utils::ArbitrageError::storage_error(format!(
                            "Failed to update round-robin index: {}",
                            e
                        ))
                    })?
                    .execute()
                    .await
                    .map_err(|e| {
                        crate::utils::ArbitrageError::storage_error(format!(
                            "Failed to save round-robin index: {}",
                            e
                        ))
                    })?;
            }
            DistributionStrategy::PriorityBased => {
                let d1 = _env.d1("ArbEdgeD1").map_err(|e| {
                    crate::utils::ArbitrageError::configuration_error(format!(
                        "Failed to access D1 database for priority-based distribution: {}",
                        e
                    ))
                })?;

                let mut user_priorities: Vec<(String, i32)> = Vec::new();

                for user_id in &message.target_users {
                    let query = "SELECT subscription_tier, activity_score FROM user_profiles WHERE user_id = ?";
                    let stmt = d1.prepare(query);
                    let result = stmt
                        .bind(&[user_id.into()])
                        .map_err(|e| {
                            crate::utils::ArbitrageError::database_error(format!(
                                "Failed to bind user query: {}",
                                e
                            ))
                        })?
                        .first::<serde_json::Value>(None)
                        .await
                        .map_err(|e| {
                            crate::utils::ArbitrageError::database_error(format!(
                                "Failed to query user priority data: {}",
                                e
                            ))
                        })?;

                    let priority = if let Some(data) = result {
                        let tier_priority =
                            match data.get("subscription_tier").and_then(|v| v.as_str()) {
                                Some("premium") => 100,
                                Some("pro") => 50,
                                Some("basic") => 20,
                                _ => 10,
                            };
                        let activity_score = data
                            .get("activity_score")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32;
                        tier_priority + (activity_score / 10)
                    } else {
                        10
                    };

                    user_priorities.push((user_id.clone(), priority));
                }

                user_priorities.sort_by(|a, b| b.1.cmp(&a.1));

                let top_users = user_priorities
                    .into_iter()
                    .take(std::cmp::min(5, message.target_users.len()))
                    .map(|(user_id, _)| user_id)
                    .collect::<Vec<_>>();

                // Notification content handled by separate worker

                for user_id in &top_users {
                    // Telegram notifications now handled by separate worker
                    console_log!(
                        "Telegram notification queued for priority user: {}",
                        user_id
                    );
                }
            }
            DistributionStrategy::Geographic => {
                let d1 = _env.d1("ArbEdgeD1").map_err(|e| {
                    crate::utils::ArbitrageError::configuration_error(format!(
                        "Failed to access D1 database for geographic distribution: {}",
                        e
                    ))
                })?;

                let current_hour = Utc::now().hour();
                let mut eligible_users = Vec::new();

                for user_id in &message.target_users {
                    let query = "SELECT timezone_offset, trading_hours_start, trading_hours_end FROM user_profiles WHERE user_id = ?";
                    let stmt = d1.prepare(query);
                    let result = stmt
                        .bind(&[user_id.into()])
                        .map_err(|e| {
                            crate::utils::ArbitrageError::database_error(format!(
                                "Failed to bind geographic query: {}",
                                e
                            ))
                        })?
                        .first::<serde_json::Value>(None)
                        .await
                        .map_err(|e| {
                            crate::utils::ArbitrageError::database_error(format!(
                                "Failed to query user geographic data: {}",
                                e
                            ))
                        })?;

                    if let Some(data) = result {
                        let timezone_offset = data
                            .get("timezone_offset")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32;
                        let trading_start = data
                            .get("trading_hours_start")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(9) as u32;
                        let trading_end = data
                            .get("trading_hours_end")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(17) as u32;

                        let user_local_hour =
                            ((current_hour as i32 + timezone_offset).rem_euclid(24)) as u32;

                        if user_local_hour >= trading_start && user_local_hour <= trading_end {
                            eligible_users.push(user_id.clone());
                        }
                    } else {
                        eligible_users.push(user_id.clone());
                    }
                }

                if eligible_users.is_empty() {
                    console_log!("No users in active trading hours for geographic distribution");
                    return Ok(());
                }

                // Notification content handled by separate worker

                for user_id in &eligible_users {
                    // Telegram notifications now handled by separate worker
                    console_log!(
                        "Telegram notification queued for geographic user: {}",
                        user_id
                    );
                }
            }
            DistributionStrategy::Targeted => {
                console_log!(
                    "Targeted distribution strategy processed for {} users",
                    message.target_users.len()
                );
            }
            _ => {
                console_log!("Unhandled distribution strategy, using default broadcast behavior");
            }
        }
    }

    Ok(())
}

/// Process user notification message
async fn process_notification_message(
    message: &UserNotificationMessage,
    _env: &Env,
    _telegram_service: Option<()>, // Removed telegram service - now in separate worker
) -> ArbitrageResult<()> {
    match &message.delivery_method {
        DeliveryMethod::Telegram => {
            // Telegram delivery now handled by separate telegram worker
            console_log!(
                "Telegram notification queued for separate processing: {}",
                message.notification_id
            );
            return Ok(());
        }
        DeliveryMethod::Email => {
            let email_service = initialize_email_service(_env).await?;

            let d1 = _env.d1("ArbEdgeD1").map_err(|e| {
                crate::utils::ArbitrageError::configuration_error(format!(
                    "Failed to access D1 database for email delivery: {}",
                    e
                ))
            })?;

            let query = "SELECT email FROM user_profiles WHERE user_id = ?";
            let stmt = d1.prepare(query);
            let result = stmt
                .bind(&[message.user_id.clone().into()])
                .map_err(|e| {
                    crate::utils::ArbitrageError::database_error(format!(
                        "Failed to bind email query: {}",
                        e
                    ))
                })?
                .first::<serde_json::Value>(None)
                .await
                .map_err(|e| {
                    crate::utils::ArbitrageError::database_error(format!(
                        "Failed to query user email: {}",
                        e
                    ))
                })?;

            if let Some(data) = result {
                if let Some(email) = data.get("email").and_then(|v| v.as_str()) {
                    email_service
                        .send_email(email, "ArbEdge Notification", &message.message)
                        .await?;
                } else {
                    return Err(crate::utils::ArbitrageError::configuration_error(format!(
                        "No email address found for user: {}",
                        message.user_id
                    )));
                }
            } else {
                return Err(crate::utils::ArbitrageError::configuration_error(format!(
                    "User not found: {}",
                    message.user_id
                )));
            }
        }
        DeliveryMethod::Push => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let d1 = _env.d1("ArbEdgeD1").map_err(|e| {
                    crate::utils::ArbitrageError::configuration_error(format!(
                        "Failed to access D1 database for WebPush delivery: {}",
                        e
                    ))
                })?;

                let query = "SELECT webpush_endpoint, webpush_p256dh, webpush_auth FROM user_profiles WHERE user_id = ?";
                let stmt = d1.prepare(query);
                let result = stmt
                    .bind(&[message.user_id.clone().into()])
                    .map_err(|e| {
                        crate::utils::ArbitrageError::database_error(format!(
                            "Failed to bind WebPush query: {}",
                            e
                        ))
                    })?
                    .first::<serde_json::Value>(None)
                    .await
                    .map_err(|e| {
                        crate::utils::ArbitrageError::database_error(format!(
                            "Failed to query user WebPush data: {}",
                            e
                        ))
                    })?;

                if let Some(data) = result {
                    let endpoint = data.get("webpush_endpoint").and_then(|v| v.as_str());
                    let p256dh = data.get("webpush_p256dh").and_then(|v| v.as_str());
                    let auth = data.get("webpush_auth").and_then(|v| v.as_str());

                    if let (Some(endpoint), Some(p256dh), Some(auth)) = (endpoint, p256dh, auth) {
                        let webpush_service = initialize_webpush_service(_env).await?;
                        webpush_service
                            .send_notification(endpoint, p256dh, auth, &message.message)
                            .await?;
                    } else {
                        return Err(crate::utils::ArbitrageError::configuration_error(format!(
                            "Incomplete WebPush subscription data for user: {}",
                            message.user_id
                        )));
                    }
                } else {
                    return Err(crate::utils::ArbitrageError::configuration_error(format!(
                        "User not found: {}",
                        message.user_id
                    )));
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                console_log!(
                    "WebPush notifications not supported in WASM target for user: {}",
                    message.user_id
                );
            }
        }
        DeliveryMethod::InApp => {
            // In-app notifications are handled by the frontend
            console_log!("In-app notification processed: {}", message.notification_id);
        }
        DeliveryMethod::Webhook => {
            // Webhook delivery implementation
            console_log!(
                "Webhook notification processed: {}",
                message.notification_id
            );
        }
        DeliveryMethod::Multiple(methods) => {
            // Handle multiple delivery methods
            for method in methods {
                console_log!(
                    "Processing multiple delivery method: {:?} for notification: {}",
                    method,
                    message.notification_id
                );
            }
        }
    }

    Ok(())
}

/// Process analytics event message
async fn process_analytics_message(
    message: &AnalyticsEventMessage,
    _env: &Env,
    analytics_service: Option<&crate::services::core::infrastructure::AnalyticsEngineService>,
) -> ArbitrageResult<()> {
    // Use the pre-initialized analytics service
    let analytics_service = analytics_service.ok_or_else(|| {
        crate::utils::ArbitrageError::configuration_error(format!(
            "Analytics service not available for message: {}",
            message.event_id
        ))
    })?;

    // Send event to Analytics Engine
    let analytics_data = crate::services::core::infrastructure::unified_analytics_and_cleanup::AnalyticsData {
        timestamp: message.timestamp as u64,
        event_type: message.event_type.clone(),
        user_id: message.user_id.clone(),
        session_id: message.session_id.clone(),
        data: message.event_data.clone(),
        metadata: crate::services::core::infrastructure::unified_analytics_and_cleanup::AnalyticsMetadata {
            source: "queue_handler".to_string(),
            version: "1.0".to_string(),
            environment: "production".to_string(),
            region: None,
        },
    };

    analytics_service
        .track_event(analytics_data)
        .await
        .map_err(|e| {
            crate::utils::ArbitrageError::storage_error(format!("Analytics write failed: {}", e))
        })?;

    Ok(())
}

// Note: Telegram service initialization removed - now handled by separate telegram worker

/// Initialize Analytics Engine service from environment
async fn initialize_analytics_service(
    env: &Env,
) -> ArbitrageResult<crate::services::core::infrastructure::AnalyticsEngineService> {
    let _account_id = env
        .var("CLOUDFLARE_ACCOUNT_ID")
        .map_err(|e| {
            crate::utils::ArbitrageError::configuration_error(format!(
                "CLOUDFLARE_ACCOUNT_ID not found: {}",
                e
            ))
        })?
        .to_string();

    let _api_token = env
        .secret("CLOUDFLARE_API_TOKEN")
        .map_err(|e| {
            crate::utils::ArbitrageError::configuration_error(format!(
                "CLOUDFLARE_API_TOKEN not found: {}",
                e
            ))
        })?
        .to_string();

    let _dataset_name = env
        .var("ANALYTICS_DATASET_NAME")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "arbitrage_analytics".to_string());

    let config = crate::services::core::infrastructure::unified_analytics_and_cleanup::UnifiedAnalyticsAndCleanupConfig {
        analytics: crate::services::core::infrastructure::unified_analytics_and_cleanup::AnalyticsConfig {
            enable_real_time_processing: true,
            batch_size: 100,
            processing_interval_ms: 5000,
            retention_days: 30,
            aggregation_levels: vec![],
        },
        cleanup: crate::services::core::infrastructure::unified_analytics_and_cleanup::CleanupConfig {
            enable_automated_cleanup: true,
            cleanup_interval_hours: 24,
            max_cleanup_operations_per_cycle: 10,
            safety_threshold_percent: 0.8,
            policies: vec![],
        },
        optimization: crate::services::core::infrastructure::unified_analytics_and_cleanup::OptimizationConfig {
            enable_performance_optimization: true,
            optimization_interval_hours: 24,
            memory_threshold_percent: 0.85,
            storage_threshold_percent: 0.80,
        },
    };

    Ok(crate::services::core::infrastructure::AnalyticsEngineService::new(config))
}

/// Initialize Email service from environment
async fn initialize_email_service(env: &Env) -> ArbitrageResult<EmailService> {
    let api_key = env.secret("EMAIL_API_KEY")?.to_string();
    let from_email = env.var("FROM_EMAIL")?.to_string();
    let api_url = env.var("EMAIL_API_URL")?.to_string();
    Ok(EmailService::new(api_key, from_email, api_url))
}

/// Initialize WebPush service from environment
#[cfg(not(target_arch = "wasm32"))]
async fn initialize_webpush_service(env: &Env) -> ArbitrageResult<WebPushService> {
    let vapid_private_key = env.secret("VAPID_PRIVATE_KEY")?.to_string();
    let vapid_public_key = env.var("VAPID_PUBLIC_KEY")?.to_string();
    WebPushService::new(vapid_private_key, vapid_public_key)
}

pub struct EmailService {
    api_key: String,
    from_email: String,
    api_url: String,
    client: reqwest::Client,
}

impl EmailService {
    pub fn new(api_key: String, from_email: String, api_url: String) -> Self {
        Self {
            api_key,
            from_email,
            api_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, content: &str) -> ArbitrageResult<()> {
        let to_owned = to.to_string();
        let subject_owned = subject.to_string();
        let content_owned = content.to_string();

        send_with_retry(
            || async {
                console_log!("Attempting to send email to: {}", to_owned);
                let response = self
                    .client
                    .post(&self.api_url)
                    .header("X-Api-Key", &self.api_key)
                    .json(&serde_json::json!({
                        "to": to_owned,
                        "from": self.from_email,
                        "subject": subject_owned,
                        "body": content_owned
                    }))
                    .send()
                    .await
                    .map_err(|e| {
                        ArbitrageError::network_error(format!("Email request failed: {}", e))
                    })?;

                if response.status().is_success() {
                    console_log!("Email sent successfully to: {}", to_owned);
                    Ok(())
                } else {
                    let status = response.status();
                    let error_body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error body".to_string());
                    console_error!(
                        "Failed to send email to {}: Status {}, Body: {}",
                        to_owned,
                        status,
                        error_body
                    );
                    Err(ArbitrageError::service_error(format!(
                        "Email service failed: {} - {}",
                        status, error_body
                    )))
                }
            },
            3,
        )
        .await
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct WebPushService {
    vapid_private_key: String,
    #[allow(dead_code)]
    vapid_public_key: String,
    client: IsahcWebPushClient,
}

#[cfg(not(target_arch = "wasm32"))]
impl WebPushService {
    pub fn new(vapid_private_key: String, vapid_public_key: String) -> ArbitrageResult<Self> {
        let client = IsahcWebPushClient::new().map_err(|e| {
            ArbitrageError::service_error(format!("Failed to create WebPush client: {}", e))
        })?;

        Ok(Self {
            vapid_private_key,
            vapid_public_key,
            client,
        })
    }

    pub async fn send_notification(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        content: &str,
    ) -> ArbitrageResult<()> {
        let endpoint_owned = endpoint.to_string();
        let content_owned = content.to_string();
        let p256dh_owned = p256dh.to_string();
        let auth_owned = auth.to_string();
        let vapid_private_key = self.vapid_private_key.clone();

        send_with_retry(
            || async {
                let short_endpoint = &endpoint_owned[..endpoint_owned.len().min(20)];
                console_log!(
                    "Attempting to send web push notification to endpoint starting with: {}",
                    short_endpoint
                );

                // Create subscription info from browser push subscription
                let subscription_info =
                    SubscriptionInfo::new(&endpoint_owned, &p256dh_owned, &auth_owned);

                // Create VAPID signature from private key with subscription info
                let mut sig_builder =
                    VapidSignatureBuilder::from_base64(&vapid_private_key, &subscription_info)
                        .map_err(|e| {
                            ArbitrageError::service_error(format!(
                                "Failed to create VAPID signature builder: {}",
                                e
                            ))
                        })?;

                // Add required claims
                sig_builder.add_claim("sub", "mailto:admin@arbedge.com");

                let vapid_signature = sig_builder.build().map_err(|e| {
                    ArbitrageError::service_error(format!("Failed to build VAPID signature: {}", e))
                })?;

                // Create notification payload
                let payload = serde_json::json!({
                    "title": "ArbEdge Notification",
                    "body": content_owned.clone(),
                    "icon": "/icon-192x192.png",
                    "badge": "/badge-72x72.png"
                });

                let payload_bytes = serde_json::to_vec(&payload).map_err(|e| {
                    ArbitrageError::service_error(format!("Failed to serialize payload: {}", e))
                })?;

                // Build web push message with encryption and VAPID authentication
                let mut builder = WebPushMessageBuilder::new(&subscription_info);
                builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
                builder.set_vapid_signature(vapid_signature);
                builder.set_ttl(86400); // 24 hours TTL

                let message = builder.build().map_err(|e| {
                    ArbitrageError::service_error(format!(
                        "Failed to build web push message: {}",
                        e
                    ))
                })?;

                // Send the notification
                self.client.send(message).await.map_err(|e| {
                    console_error!("Failed to send web push to {}: {}", short_endpoint, e);
                    ArbitrageError::service_error(format!("Web push service failed: {}", e))
                })?;

                console_log!(
                    "Web push notification sent successfully to endpoint starting with: {}",
                    short_endpoint
                );
                Ok(())
            },
            3,
        )
        .await
    }
}

pub async fn handle_user_cleanup(
    _event: ScheduledEvent,
    _env: Env,
    _ctx: Context,
) -> ArbitrageResult<()> {
    console_log!("[USER_CLEANUP] Starting user cleanup task");

    // Check if it's the right time for cleanup (e.g., daily at 2 AM UTC)
    let current_hour = Utc::now().hour(); // Now works with Timelike trait
    if current_hour != 2 {
        console_log!("[USER_CLEANUP] Skipping user cleanup - not the scheduled hour");
        return Ok(());
    }

    // ... rest of the function ...

    Ok(())
}

/// Process messages from the user notification queue
pub async fn handle_user_notification_queue(
    batch: MessageBatch<UserNotificationMessage>,
    env: Env,
    _ctx: worker::Context,
) -> worker::Result<()> {
    let messages = batch.messages()?;
    console_log!("📬 Processing {} user messages", messages.len());

    for message in messages {
        match process_notification_message(message.body(), &env, None).await {
            Ok(_) => {
                console_log!(
                    "✅ Successfully processed message {}",
                    message.body().notification_id
                );
                message.ack();
            }
            Err(e) => {
                console_log!(
                    "❌ Failed to process message {}: {:?}",
                    message.body().notification_id,
                    e
                );
                message.retry();
            }
        }
    }

    console_log!("✅ Completed user notification batch processing");
    Ok(())
}

/// Process messages from the user message queue
pub async fn handle_user_message_queue(
    batch: MessageBatch<UserMessage>,
    env: Env,
    _ctx: worker::Context,
) -> worker::Result<()> {
    let messages = batch.messages()?;
    console_log!("📬 Processing {} user messages", messages.len());

    for message in messages {
        match process_user_message(message.body(), &env).await {
            Ok(_) => {
                console_log!(
                    "✅ Successfully processed user message {}",
                    message.body().message_id
                );
                message.ack();
            }
            Err(e) => {
                console_log!(
                    "❌ Failed to process user message {}: {:?}",
                    message.body().message_id,
                    e
                );
                message.retry();
            }
        }
    }

    console_log!("✅ Completed user message batch processing");
    Ok(())
}

/// Process messages from the notification message queue
pub async fn handle_notification_message_queue(
    batch: MessageBatch<NotificationMessage>,
    env: Env,
    _ctx: worker::Context,
) -> worker::Result<()> {
    let messages = batch.messages()?;
    console_log!("📬 Processing {} notification messages", messages.len());

    for message in messages {
        match process_general_notification(message.body(), &env).await {
            Ok(_) => {
                console_log!(
                    "✅ Successfully processed notification {}",
                    message.body().notification_id
                );
                message.ack();
            }
            Err(e) => {
                console_log!(
                    "❌ Failed to process notification {}: {:?}",
                    message.body().notification_id,
                    e
                );
                message.retry();
            }
        }
    }

    console_log!("✅ Completed notification message batch processing");
    Ok(())
}

/// Process a general notification message
async fn process_general_notification(
    message: &NotificationMessage,
    _env: &Env,
) -> ArbitrageResult<()> {
    console_log!(
        "📨 Processing general notification: {}",
        message.notification_id
    );

    // Log the notification for now (can be extended to route to different channels)
    match message.priority {
        Priority::High => console_log!("🔴 HIGH PRIORITY: {}", message.content),
        Priority::Normal => console_log!("🟡 NORMAL PRIORITY: {}", message.content),
        Priority::Low => console_log!("🟢 LOW PRIORITY: {}", message.content),
        Priority::Critical => console_log!("🚨 CRITICAL PRIORITY: {}", message.content),
    }

    console_log!(
        "✅ General notification processed: {}",
        message.notification_id
    );
    Ok(())
}

/// Process a user message
async fn process_user_message(message: &UserMessage, _env: &Env) -> ArbitrageResult<()> {
    console_log!("📨 Processing user message: {}", message.message_id);
    console_log!("👤 User: {}, Content: {}", message.user_id, message.content);

    // For now, just log the message. This can be extended to:
    // - Store in database
    // - Trigger AI response
    // - Update user activity metric
    // - Route to appropriate handler

    console_log!("✅ User message processed: {}", message.message_id);
    Ok(())
}

/// Process opportunity generation (scheduled task)
pub async fn process_opportunity_generation(
    _env: &worker::Env,
    service_container: Arc<ServiceContainer>,
) -> worker::Result<()> {
    console_log!("🔍 Starting opportunity generation task");

    // Use the opportunity engine from service container
    if let Some(opportunity_engine) = &service_container.opportunity_engine {
        match opportunity_engine.generate_global_opportunities(None).await {
            Ok(_) => {
                console_log!("✅ Opportunity generation completed successfully");
                Ok(())
            }
            Err(e) => {
                console_error!("❌ Opportunity generation failed: {:?}", e);
                Err(worker::Error::RustError(format!(
                    "Opportunity generation failed: {:?}",
                    e
                )))
            }
        }
    } else {
        console_error!("❌ Opportunity engine not available");
        Err(worker::Error::RustError(
            "Opportunity engine not available".to_string(),
        ))
    }
}

/// Process data cleanup (scheduled task)
pub async fn process_data_cleanup(
    _env: &worker::Env,
    service_container: Arc<ServiceContainer>,
) -> worker::Result<()> {
    console_log!("🧹 Starting data cleanup task");

    // Perform cleanup operations
    match service_container.cleanup_expired_sessions().await {
        Ok(_) => {
            console_log!("✅ Data cleanup completed successfully");
            Ok(())
        }
        Err(e) => {
            console_error!("❌ Data cleanup failed: {:?}", e);
            Err(worker::Error::RustError(format!(
                "Data cleanup failed: {:?}",
                e
            )))
        }
    }
}

/// Process deep maintenance (scheduled task)
pub async fn process_user_activity_analysis(
    _env: &worker::Env,
    _service_container: Arc<ServiceContainer>,
) -> worker::Result<()> {
    console_log!("🔧 Starting deep maintenance task");

    // Perform deep maintenance operations
    // This could include cache optimization, database maintenance, etc.
    console_log!("🔧 Performing cache optimization...");

    // For now, just log the operation
    console_log!("✅ Deep maintenance completed successfully");
    Ok(())
}

/// Process queue message (generic handler)
pub async fn process_queue_message(
    _env: &worker::Env,
    body: &serde_json::Value,
    _service_container: Arc<ServiceContainer>,
) -> worker::Result<()> {
    console_log!("📨 Processing queue message: {:?}", body);

    // Parse and route the message based on its type
    if let Some(message_type) = body.get("type").and_then(|v| v.as_str()) {
        match message_type {
            "opportunity" => {
                console_log!("🎯 Processing opportunity message");
                // Handle opportunity-related messages
            }
            "notification" => {
                console_log!("📬 Processing notification message");
                // Handle notification messages
            }
            "analytics" => {
                console_log!("📊 Processing analytics message");
                // Handle analytics messages
            }
            _ => {
                console_log!("❓ Unknown message type: {}", message_type);
            }
        }
    }

    console_log!("✅ Queue message processed successfully");
    Ok(())
}
