-- Migration 003: Add Superadmin User
-- Created: 2025-01-27  
-- Description: Add superadmin user with full permissions from environment variables

-- Insert superadmin user profile
INSERT OR REPLACE INTO user_profiles (
    user_id,
    telegram_id,
    username,
    api_keys,
    risk_tolerance,
    trading_preferences,
    notification_settings,
    subscription_tier,
    account_status,
    email_verification_status,
    created_at,
    updated_at,
    last_login_at,
    profile_metadata
) VALUES (
    'superadmin_' || COALESCE(${SUPERADMIN_TELEGRAM_ID}, '0'),
    COALESCE(${SUPERADMIN_TELEGRAM_ID}, 0),
    COALESCE('${SUPERADMIN_USERNAME}', 'admin'),
    '[]',
    'high',
    '{"max_position_size":10000.0,"preferred_exchanges":["binance","bybit"],"auto_execution_enabled":true,"max_daily_trades":100,"profit_threshold":0.001}',
    '{"telegram_enabled":true,"email_enabled":true,"push_enabled":true,"priority_filter":"all","quiet_hours":false}',
    'pro',
    'active',
    'verified',
    datetime('now'),
    datetime('now'),
    datetime('now'),
    '{"role":"superadmin","permissions":["all"],"access_level":"unlimited","created_by":"system","notes":"System superadmin with full access from environment configuration"}'
);

-- Insert superadmin trading preferences with all features enabled
INSERT OR REPLACE INTO user_trading_preferences (
    preference_id,
    user_id,
    trading_focus,
    experience_level,
    risk_tolerance,
    automation_level,
    automation_scope,
    arbitrage_enabled,
    technical_enabled,
    advanced_analytics_enabled,
    preferred_notification_channels,
    trading_hours_timezone,
    trading_hours_start,
    trading_hours_end,
    onboarding_completed,
    tutorial_steps_completed,
    created_at,
    updated_at
) VALUES (
    'pref_superadmin_' || COALESCE(${SUPERADMIN_TELEGRAM_ID}, '0'),
    'superadmin_' || COALESCE(${SUPERADMIN_TELEGRAM_ID}, '0'),
    'hybrid',
    'advanced',
    'aggressive',
    'full_auto',
    'both',
    TRUE,
    TRUE,
    TRUE,
    '["telegram","email","push"]',
    'UTC',
    '00:00',
    '23:59',
    TRUE,
    '["profile_setup","api_keys","risk_settings","automation_config","advanced_features","admin_tools"]',
    datetime('now'),
    datetime('now')
);

-- Insert comprehensive opportunity preferences for superadmin
INSERT OR REPLACE INTO user_opportunity_preferences (
    user_id,
    preferences_json,
    created_at,
    updated_at
) VALUES (
    'superadmin_' || COALESCE(${SUPERADMIN_TELEGRAM_ID}, '0'),
    '{"risk_tolerance":"high","max_position_size_usd":10000.0,"min_profit_threshold":0.001,"max_profit_threshold":1.0,"preferred_exchanges":["binance","bybit"],"excluded_exchanges":[],"preferred_pairs":["BTCUSDT","ETHUSDT","BNBUSDT","ADAUSDT","XRPUSDT"],"excluded_pairs":[],"max_exposure_per_pair":5000.0,"max_daily_trades":100,"auto_execution":true,"requires_confirmation":false,"profit_taking_strategy":"aggressive","stop_loss_strategy":"tight","position_sizing_method":"fixed","diversification_enabled":true,"correlation_analysis_enabled":true,"sentiment_analysis_enabled":true,"technical_analysis_enabled":true,"ai_enhancement_enabled":true,"notification_preferences":{"instant_alerts":true,"daily_summary":true,"weekly_report":true,"performance_updates":true},"admin_features":{"access_all_users":true,"modify_system_config":true,"view_audit_logs":true,"manage_user_permissions":true,"system_maintenance":true}}',
    datetime('now'),
    datetime('now')
);

-- Log audit entry for superadmin creation (using NULL for system actions)
INSERT INTO audit_log (
    user_id,
    action,
    resource_type,
    resource_id,
    new_value_json,
    timestamp,
    ip_address,
    user_agent
) VALUES (
    NULL,
    'create_superadmin',
    'user_profile',
    'superadmin_' || COALESCE(${SUPERADMIN_TELEGRAM_ID}, '0'),
    '{"telegram_id":"' || COALESCE(${SUPERADMIN_TELEGRAM_ID}, '0') || '","username":"' || COALESCE('${SUPERADMIN_USERNAME}', 'admin') || '","role":"superadmin","created_by":"system","notes":"Initial superadmin user setup from environment variables"}',
    unixepoch('now') * 1000,
    'system',
    'migration_script'
);

-- Record this migration
INSERT INTO schema_migrations (version, description) VALUES ('003', 'Add superadmin user from environment variables'); 