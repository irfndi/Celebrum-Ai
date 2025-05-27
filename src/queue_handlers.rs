// Cloudflare Queue Message Handlers
// Production implementation for consuming queue messages using #[event(queue)] macro

use worker::*;
use crate::services::core::infrastructure::cloudflare_queues::{
    OpportunityDistributionMessage, UserNotificationMessage, AnalyticsEventMessage, QueueEvent
};
use crate::utils::ArbitrageResult;

/// Handle opportunity distribution queue messages
#[event(queue)]
pub async fn opportunity_queue_handler(
    message_batch: MessageBatch<OpportunityDistributionMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    console_log!("Processing opportunity distribution batch: {} messages", message_batch.messages().len());

    // Initialize telegram service once for the entire batch
    let telegram_service = match initialize_telegram_service(&env).await {
        Ok(service) => Some(service),
        Err(e) => {
            console_error!("Failed to initialize telegram service: {}", e);
            None
        }
    };

    for message in message_batch.messages() {
        match process_opportunity_message(message.body(), &env, telegram_service.as_ref()).await {
            Ok(_) => {
                console_log!("Successfully processed opportunity message: {}", message.body().message_id);
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

/// Handle user notification queue messages
#[event(queue)]
pub async fn notification_queue_handler(
    message_batch: MessageBatch<UserNotificationMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    console_log!("Processing notification batch: {} messages", message_batch.messages().len());

    // Initialize telegram service once for the entire batch
    let telegram_service = match initialize_telegram_service(&env).await {
        Ok(service) => Some(service),
        Err(e) => {
            console_error!("Failed to initialize telegram service: {}", e);
            None
        }
    };

    for message in message_batch.messages() {
        match process_notification_message(message.body(), &env, telegram_service.as_ref()).await {
            Ok(_) => {
                console_log!("Successfully processed notification message: {}", message.body().message_id);
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

/// Handle analytics event queue messages
#[event(queue)]
pub async fn analytics_queue_handler(
    message_batch: MessageBatch<AnalyticsEventMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    console_log!("Processing analytics batch: {} messages", message_batch.messages().len());

    // Initialize analytics service once for the entire batch
    let analytics_service = match initialize_analytics_service(&env).await {
        Ok(service) => Some(service),
        Err(e) => {
            console_error!("Failed to initialize analytics service: {}", e);
            None
        }
    };

    for message in message_batch.messages() {
        match process_analytics_message(message.body(), &env, analytics_service.as_ref()).await {
            Ok(_) => {
                console_log!("Successfully processed analytics message: {}", message.body().event_id);
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

/// Process opportunity distribution message
async fn process_opportunity_message(
    message: &OpportunityDistributionMessage,
    _env: &Env,
    telegram_service: Option<&crate::services::interfaces::telegram::telegram::TelegramService>,
) -> ArbitrageResult<()> {
    // Use the pre-initialized telegram service
    let telegram_service = telegram_service.ok_or_else(|| {
        crate::utils::ArbitrageError::configuration_error("Telegram service not available")
    })?;
    
    // Distribute opportunity to target users
    for user_id in &message.target_users {
        match message.distribution_strategy {
            crate::services::core::infrastructure::cloudflare_queues::DistributionStrategy::Broadcast => {
                // Send to all users
                let notification_text = format!(
                    "🚀 New Arbitrage Opportunity!\n\n\
                    Pair: {}\n\
                    Exchanges: {} ↔ {}\n\
                    Profit: {:.2}%\n\
                    Confidence: {:.1}%",
                    message.opportunity.pair,
                    message.opportunity.long_exchange,
                    message.opportunity.short_exchange,
                    message.opportunity.rate_difference * 100.0,
                    message.opportunity.confidence * 100.0
                );
                
                telegram_service.send_private_message(&notification_text, user_id).await?;
            }
            crate::services::core::infrastructure::cloudflare_queues::DistributionStrategy::RoundRobin => {
                // TODO: Implement round-robin distribution logic - tracked in issue #124
                // This should implement fair rotation among users based on last distribution index
                todo!("Round-robin distribution not yet implemented - tracked in issue #124");
            }
            crate::services::core::infrastructure::cloudflare_queues::DistributionStrategy::PriorityBased => {
                // TODO: Implement priority-based distribution logic - tracked in issue #125
                // This should prioritize users based on subscription tier or activity level
                todo!("Priority-based distribution not yet implemented - tracked in issue #125");
            }
            crate::services::core::infrastructure::cloudflare_queues::DistributionStrategy::GeographicBased => {
                // TODO: Implement geographic-based distribution logic - tracked in issue #126
                // This should filter users by location or timezone before sending messages
                todo!("Geographic-based distribution not yet implemented - tracked in issue #126");
            }
        }
    }

    Ok(())
}

/// Process user notification message
async fn process_notification_message(
    message: &UserNotificationMessage,
    _env: &Env,
    telegram_service: Option<&crate::services::interfaces::telegram::telegram::TelegramService>,
) -> ArbitrageResult<()> {
    match message.delivery_method {
        crate::services::core::infrastructure::cloudflare_queues::DeliveryMethod::Telegram => {
            let telegram_service = telegram_service.ok_or_else(|| {
                crate::utils::ArbitrageError::configuration_error("Telegram service not available")
            })?;
            telegram_service.send_private_message(&message.content, &message.user_id).await?;
        }
        crate::services::core::infrastructure::cloudflare_queues::DeliveryMethod::Email => {
            // TODO: Implement email delivery - tracked in issue #123
            // See: https://github.com/irfndi/ArbEdge/issues/123
            console_log!("Email delivery not yet implemented for message: {} - tracked in issue #123", message.message_id);
            return Err(crate::utils::ArbitrageError::configuration_error(
                "Email delivery not yet implemented - tracked in issue #123"
            ));
        }
        crate::services::core::infrastructure::cloudflare_queues::DeliveryMethod::WebPush => {
            console_log!("WebPush notification delivery not yet implemented for message: {}", message.message_id);
            return Err(crate::utils::ArbitrageError::configuration_error(
                "WebPush notification delivery not yet implemented"
            ));
        }
        crate::services::core::infrastructure::cloudflare_queues::DeliveryMethod::SMS => {
            console_log!("SMS delivery not yet implemented for message: {}", message.message_id);
            return Err(crate::utils::ArbitrageError::configuration_error(
                "SMS delivery not yet implemented"
            ));
        }
    }

    Ok(())
}

/// Process analytics event message
async fn process_analytics_message(
    message: &AnalyticsEventMessage,
    _env: &Env,
    analytics_service: Option<&crate::services::core::infrastructure::analytics_engine::AnalyticsEngineClient>,
) -> ArbitrageResult<()> {
    // Use the pre-initialized analytics service
    let analytics_service = analytics_service.ok_or_else(|| {
        crate::utils::ArbitrageError::configuration_error("Analytics service not available")
    })?;
    
    // Send event to Analytics Engine
    let event_data = serde_json::json!({
        "event_id": message.event_id,
        "event_type": message.event_type,
        "user_id": message.user_id,
        "timestamp": message.timestamp,
        "data": message.data
    });

    analytics_service.write_data_point(&[event_data]).await
        .map_err(|e| crate::utils::ArbitrageError::storage_error(format!("Analytics write failed: {}", e)))?;

    Ok(())
}

/// Initialize Telegram service from environment
async fn initialize_telegram_service(env: &Env) -> ArbitrageResult<crate::services::interfaces::telegram::telegram::TelegramService> {
    let bot_token = env.secret("TELEGRAM_BOT_TOKEN")
        .map_err(|_| crate::utils::ArbitrageError::configuration_error("TELEGRAM_BOT_TOKEN not found"))?
        .to_string();
    
    let chat_id = env.var("TELEGRAM_CHAT_ID")
        .map_err(|_| crate::utils::ArbitrageError::configuration_error("TELEGRAM_CHAT_ID not found"))?
        .to_string();

    let config = crate::services::interfaces::telegram::telegram::TelegramConfig {
        bot_token,
        chat_id,
        is_test_mode: env.var("TELEGRAM_TEST_MODE")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false),
    };

    Ok(crate::services::interfaces::telegram::telegram::TelegramService::new(config))
}

/// Initialize Analytics Engine service from environment
async fn initialize_analytics_service(env: &Env) -> ArbitrageResult<crate::services::core::infrastructure::analytics_engine::AnalyticsEngineClient> {
    let account_id = env.var("CLOUDFLARE_ACCOUNT_ID")
        .map_err(|_| crate::utils::ArbitrageError::configuration_error("CLOUDFLARE_ACCOUNT_ID not found"))?
        .to_string();
    
    let api_token = env.secret("CLOUDFLARE_API_TOKEN")
        .map_err(|_| crate::utils::ArbitrageError::configuration_error("CLOUDFLARE_API_TOKEN not found"))?
        .to_string();

    let dataset_name = env.var("ANALYTICS_DATASET_NAME")
        .unwrap_or_else(|_| "arbitrage_analytics".to_string())
        .to_string();

    Ok(crate::services::core::infrastructure::analytics_engine::AnalyticsEngineClient::new(
        account_id,
        api_token,
        dataset_name,
    ))
} 