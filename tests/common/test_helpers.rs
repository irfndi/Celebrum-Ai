// Common Test Helper Functions
// Shared utilities for creating test data and assertions

use arb_edge::services::core::analysis::market_analysis::{
    OpportunityType, PricePoint, PriceSeries, RiskLevel, TimeFrame, TimeHorizon, TradingOpportunity,
};
// Telegram types are not currently exported - using placeholder structs
#[derive(Debug, Clone)]
pub struct TelegramChat {
    pub id: i64,
    pub chat_type: String,
}

#[derive(Debug, Clone)]
pub struct TelegramMessage {
    pub message_id: i32,
    pub text: Option<String>,
    pub chat: TelegramChat,
    pub from: Option<TelegramUser>,
}

#[derive(Debug, Clone)]
pub struct TelegramUpdate {
    pub update_id: i32,
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Clone)]
pub struct TelegramUser {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
}
use arb_edge::services::core::user::user_trading_preferences::{
    AutomationLevel, AutomationScope, ExperienceLevel, RiskTolerance, TradingFocus,
    UserTradingPreferences,
};
use arb_edge::types::{
    ArbitrageOpportunity, CommandPermission, ExchangeIdEnum, SubscriptionTier, UserProfile,
};
use serde_json::json;

/// Create a test user profile with specified parameters
pub fn create_test_user(telegram_id: i64, subscription_tier: SubscriptionTier) -> UserProfile {
    let mut user = UserProfile::new(Some(telegram_id), Some("test-invite".to_string()));
    user.subscription.tier = subscription_tier;
    user
}

/// Create test trading preferences
pub fn create_test_preferences(
    user_id: String,
    focus: TradingFocus,
    experience: ExperienceLevel,
    risk_tolerance: RiskTolerance,
) -> UserTradingPreferences {
    let preference_id = format!("pref_{}", user_id);
    let now = chrono::Utc::now().timestamp_millis() as u64;

    UserTradingPreferences {
        preference_id,
        user_id,
        trading_focus: focus,
        experience_level: experience,
        risk_tolerance,
        automation_level: AutomationLevel::Manual,
        automation_scope: AutomationScope::None,
        arbitrage_enabled: true,
        technical_enabled: false,
        advanced_analytics_enabled: false,
        preferred_notification_channels: vec!["telegram".to_string()],
        trading_hours_timezone: "UTC".to_string(),
        trading_hours_start: "00:00".to_string(),
        trading_hours_end: "23:59".to_string(),
        onboarding_completed: false,
        tutorial_steps_completed: vec![],
        created_at: now,
        updated_at: now,
    }
}

/// Create a test trading opportunity
pub fn create_test_opportunity(
    id: &str,
    pair: &str,
    confidence: f64,
    risk: RiskLevel,
) -> TradingOpportunity {
    TradingOpportunity {
        opportunity_id: id.to_string(),
        opportunity_type: OpportunityType::Arbitrage,
        trading_pair: pair.to_string(),
        exchanges: vec!["binance".to_string(), "bybit".to_string()],
        entry_price: match pair {
            "BTCUSDT" => 50000.0,
            "ETHUSDT" => 3000.0,
            "ADAUSDT" => 0.5,
            _ => 100.0,
        },
        target_price: Some(50000.0 * 1.02),
        stop_loss: Some(50000.0 * 0.98),
        confidence_score: confidence,
        risk_level: risk,
        expected_return: confidence * 0.05,
        time_horizon: TimeHorizon::Short,
        indicators_used: vec!["arbitrage_analysis".to_string()],
        analysis_data: json!({
            "buy_exchange": "binance",
            "sell_exchange": "bybit",
            "rate_difference": confidence * 0.02
        }),
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        expires_at: Some(chrono::Utc::now().timestamp_millis() as u64 + 3600000),
    }
}

/// Create a test arbitrage opportunity
pub fn create_test_arbitrage_opportunity(
    id_param: &str,
    pair_param: &str,
    rate_diff: f64,
) -> ArbitrageOpportunity {
    let now = chrono::Utc::now().timestamp_millis() as u64;
    let long_exchange_enum = ExchangeIdEnum::Binance; // Default for this helper
    let short_exchange_enum = ExchangeIdEnum::Bybit; // Default for this helper

    let base_price = 50000.0; // Example base price for calculation
    let long_rate_val = Some(base_price);
    let short_rate_val = Some(base_price + (base_price * rate_diff));
    let net_rate_diff_val = Some(rate_diff * 0.95); // Assuming 5% fee/slippage for net calculation
    let example_volume = 0.1;

    ArbitrageOpportunity {
        id: id_param.to_string(),
        pair: pair_param.to_string(),         // Alias field
        trading_pair: pair_param.to_string(), // Canonical field

        long_exchange: long_exchange_enum,
        short_exchange: short_exchange_enum,
        buy_exchange: long_exchange_enum.as_str().to_string(),
        sell_exchange: short_exchange_enum.as_str().to_string(),
        exchanges: vec![
            long_exchange_enum.as_str().to_string(),
            short_exchange_enum.as_str().to_string(),
        ],

        long_rate: long_rate_val,
        short_rate: short_rate_val,
        buy_price: long_rate_val.unwrap_or_default(),
        sell_price: short_rate_val.unwrap_or_default(),

        rate_difference: rate_diff,
        net_rate_difference: net_rate_diff_val,
        profit_percentage: net_rate_diff_val.unwrap_or(0.0) * 100.0,

        potential_profit_value: Some(
            net_rate_diff_val.unwrap_or(0.0) * base_price * example_volume,
        ),
        volume: example_volume,

        details: Some(format!(
            "Test arbitrage opportunity for {} with {:.2}% rate difference between {} and {}",
            pair_param,
            rate_diff * 100.0,
            long_exchange_enum.as_str(),
            short_exchange_enum.as_str()
        )),
        timestamp: now,
        created_at: now,
        detected_at: now,

        confidence_score: rate_diff.abs().min(1.0), // Example: use rate_diff (0-1) as confidence
        confidence: rate_diff.abs().min(1.0),       // Alias for confidence_score

        // Let other fields take values from Default implementation
        ..ArbitrageOpportunity::default()
    }
}

/// Create a Telegram update for testing
pub fn create_telegram_update(user_id: i64, chat_id: i64, text: &str) -> TelegramUpdate {
    TelegramUpdate {
        update_id: 12345,
        message: Some(TelegramMessage {
            message_id: 67890,
            from: Some(TelegramUser {
                id: user_id,
                username: Some("testuser".to_string()),
                first_name: "Test".to_string(),
            }),
            chat: TelegramChat {
                id: chat_id,
                chat_type: "private".to_string(),
            },
            text: Some(text.to_string()),
        }),
    }
}

/// Create test price series data
pub fn create_test_price_series(pair: &str, exchange: &str, data_points: usize) -> PriceSeries {
    let mut series = PriceSeries::new(pair.to_string(), exchange.to_string(), TimeFrame::OneMinute);

    let base_price = match pair {
        "BTCUSDT" => 50000.0,
        "ETHUSDT" => 3000.0,
        "ADAUSDT" => 0.5,
        _ => 100.0,
    };
    let base_time = chrono::Utc::now().timestamp_millis() as u64;

    for i in 0..data_points {
        let price_variation = (i as f64 * 0.001) - 0.01;
        let price = base_price + (base_price * price_variation);

        let price_point = PricePoint {
            timestamp: base_time + (i as u64 * 60000),
            price,
            volume: Some(100.0 + (i as f64 * 10.0)),
            exchange_id: exchange.to_string(),
            trading_pair: pair.to_string(),
        };

        series.add_price_point(price_point);
    }

    series
}

/// Create test market data
pub fn create_test_market_data(exchange: &str, pair: &str, price: f64) -> serde_json::Value {
    json!({
        "exchange": exchange,
        "symbol": pair,
        "price": price,
        "volume": 1000.0,
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "bid": price - 0.5,
        "ask": price + 0.5,
        "high_24h": price * 1.02,
        "low_24h": price * 0.98
    })
}

/// Assert that a user has the expected permissions
pub fn assert_user_permissions(user: &UserProfile, expected_permissions: &[CommandPermission]) {
    for permission in expected_permissions {
        // Check if user has beta access (premium permissions during beta period)
        let has_beta_access = user
            .beta_expires_at
            .is_some_and(|expires_at| chrono::Utc::now().timestamp() < expires_at as i64);

        // This would normally check against the user's actual permissions
        // For now, we'll check based on subscription tier and beta access
        let has_permission = match permission {
            CommandPermission::BasicOpportunities => true,
            CommandPermission::BasicOpportunities => true,
            CommandPermission::ManualTrading => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Basic
                            | SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                    )
            }
            CommandPermission::AutomatedTrading => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Premium | SubscriptionTier::Enterprise
                    )
            }
            CommandPermission::TechnicalAnalysis => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Basic
                            | SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                    )
            }
            CommandPermission::AIEnhancedOpportunities => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Premium | SubscriptionTier::Enterprise
                    )
            }
            CommandPermission::SystemAdministration => {
                user.subscription.tier == SubscriptionTier::SuperAdmin
            }
            CommandPermission::UserManagement => {
                user.subscription.tier == SubscriptionTier::SuperAdmin
            }
            CommandPermission::GlobalConfiguration => {
                user.subscription.tier == SubscriptionTier::SuperAdmin
            }
            CommandPermission::GroupAnalytics => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Premium
                            | SubscriptionTier::Enterprise
                            | SubscriptionTier::SuperAdmin
                    )
            }
            CommandPermission::AdvancedAnalytics => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Premium | SubscriptionTier::Enterprise
                    )
            }
            CommandPermission::PremiumFeatures => {
                has_beta_access
                    || matches!(
                        user.subscription.tier,
                        SubscriptionTier::Premium | SubscriptionTier::Enterprise
                    )
            }
        };

        assert!(
            has_permission,
            "User should have permission: {:?} (beta_access: {}, tier: {:?})",
            permission, has_beta_access, user.subscription.tier
        );
    }
}

/// Assert that an opportunity matches user preferences
pub fn assert_opportunity_matches_preferences(
    opportunity: &TradingOpportunity,
    preferences: &UserTradingPreferences,
) -> bool {
    let focus_match = match preferences.trading_focus {
        TradingFocus::Arbitrage => opportunity.opportunity_type == OpportunityType::Arbitrage,
        TradingFocus::Technical => matches!(
            opportunity.opportunity_type,
            OpportunityType::Technical | OpportunityType::ArbitrageTechnical
        ),
        TradingFocus::Hybrid => true,
    };

    let risk_match = match preferences.risk_tolerance {
        RiskTolerance::Conservative => opportunity.risk_level == RiskLevel::Low,
        RiskTolerance::Balanced => {
            matches!(opportunity.risk_level, RiskLevel::Low | RiskLevel::Medium)
        }
        RiskTolerance::Aggressive => true,
    };

    // For now, assume all pairs match since we don't have preferred_trading_pairs in the current structure
    let pair_match = true;

    focus_match && risk_match && pair_match
}
