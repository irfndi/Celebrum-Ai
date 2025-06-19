-- Migration 002: Add Indexes, Views, and Initial Data
-- Created: 2025-01-27  
-- Description: Add performance indexes, views, and insert default system data

-- Performance Indexes for RBAC Tables
CREATE INDEX idx_user_profiles_telegram_id ON user_profiles(telegram_id);
CREATE INDEX idx_user_profiles_subscription_tier ON user_profiles(subscription_tier);
CREATE INDEX idx_user_profiles_account_status ON user_profiles(account_status);
CREATE INDEX idx_user_profiles_created_at ON user_profiles(created_at);

-- User Trading Preferences indexes
CREATE INDEX idx_user_trading_preferences_user_id ON user_trading_preferences(user_id);
CREATE INDEX idx_user_trading_preferences_trading_focus ON user_trading_preferences(trading_focus);
CREATE INDEX idx_user_trading_preferences_automation_level ON user_trading_preferences(automation_level);
CREATE INDEX idx_user_trading_preferences_experience_level ON user_trading_preferences(experience_level);
CREATE INDEX idx_user_trading_preferences_arbitrage_enabled ON user_trading_preferences(arbitrage_enabled);
CREATE INDEX idx_user_trading_preferences_technical_enabled ON user_trading_preferences(technical_enabled);

-- User Opportunity Preferences indexes
CREATE INDEX idx_user_opportunity_preferences_user_id ON user_opportunity_preferences(user_id);

CREATE INDEX idx_user_invitations_inviter ON user_invitations(inviter_user_id);
CREATE INDEX idx_user_invitations_status ON user_invitations(status);
CREATE INDEX idx_user_invitations_type ON user_invitations(invitation_type);
CREATE INDEX idx_user_invitations_created_at ON user_invitations(created_at);

CREATE INDEX idx_trading_analytics_user_id ON trading_analytics(user_id);
CREATE INDEX idx_trading_analytics_metric_type ON trading_analytics(metric_type);
CREATE INDEX idx_trading_analytics_timestamp ON trading_analytics(timestamp);
CREATE INDEX idx_trading_analytics_date_bucket ON trading_analytics(date_bucket);
CREATE INDEX idx_trading_analytics_exchange ON trading_analytics(exchange_id);

-- Balance history indexes
CREATE INDEX idx_balance_history_user_id ON balance_history(user_id);
CREATE INDEX idx_balance_history_exchange_id ON balance_history(exchange_id);
CREATE INDEX idx_balance_history_asset ON balance_history(asset);
CREATE INDEX idx_balance_history_timestamp ON balance_history(timestamp);
CREATE INDEX idx_balance_history_snapshot_id ON balance_history(snapshot_id);
CREATE INDEX idx_balance_history_user_exchange ON balance_history(user_id, exchange_id);
CREATE INDEX idx_balance_history_user_asset ON balance_history(user_id, asset);

-- Notification system indexes
CREATE INDEX idx_notification_templates_category ON notification_templates(category);
CREATE INDEX idx_notification_templates_is_active ON notification_templates(is_active);
CREATE INDEX idx_notification_templates_is_system ON notification_templates(is_system_template);

CREATE INDEX idx_alert_triggers_user_id ON alert_triggers(user_id);
CREATE INDEX idx_alert_triggers_trigger_type ON alert_triggers(trigger_type);
CREATE INDEX idx_alert_triggers_is_active ON alert_triggers(is_active);
CREATE INDEX idx_alert_triggers_priority ON alert_triggers(priority);
CREATE INDEX idx_alert_triggers_last_triggered ON alert_triggers(last_triggered_at);

CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_category ON notifications(category);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);
CREATE INDEX idx_notifications_scheduled_at ON notifications(scheduled_at);
CREATE INDEX idx_notifications_date_partition ON notifications(date_partition);

CREATE INDEX idx_notification_history_notification_id ON notification_history(notification_id);
CREATE INDEX idx_notification_history_user_id ON notification_history(user_id);
CREATE INDEX idx_notification_history_channel ON notification_history(channel);
CREATE INDEX idx_notification_history_delivery_status ON notification_history(delivery_status);
CREATE INDEX idx_notification_history_attempted_at ON notification_history(attempted_at);

-- AI Intelligence indexes
CREATE INDEX idx_ai_opportunity_enhancements_user_id ON ai_opportunity_enhancements(user_id);
CREATE INDEX idx_ai_opportunity_enhancements_opportunity_id ON ai_opportunity_enhancements(opportunity_id);
CREATE INDEX idx_ai_opportunity_enhancements_timestamp ON ai_opportunity_enhancements(analysis_timestamp);

CREATE INDEX idx_ai_portfolio_analysis_user_id ON ai_portfolio_analysis(user_id);
CREATE INDEX idx_ai_portfolio_analysis_timestamp ON ai_portfolio_analysis(analysis_timestamp);

CREATE INDEX idx_ai_performance_insights_user_id ON ai_performance_insights(user_id);
CREATE INDEX idx_ai_performance_insights_generated_at ON ai_performance_insights(generated_at);

CREATE INDEX idx_ai_parameter_suggestions_user_id ON ai_parameter_suggestions(user_id);
CREATE INDEX idx_ai_parameter_suggestions_parameter_name ON ai_parameter_suggestions(parameter_name);

CREATE INDEX idx_opportunity_distributions_opportunity_id ON opportunity_distributions(opportunity_id);
CREATE INDEX idx_opportunity_distributions_user_id ON opportunity_distributions(user_id);
CREATE INDEX idx_opportunity_distributions_distributed_at ON opportunity_distributions(distributed_at);

CREATE INDEX idx_user_activity_user_id ON user_activity(user_id);
CREATE INDEX idx_user_activity_activity_type ON user_activity(activity_type);
CREATE INDEX idx_user_activity_timestamp ON user_activity(timestamp);

CREATE INDEX idx_invitation_codes_created_by ON invitation_codes(created_by);
CREATE INDEX idx_invitation_codes_is_active ON invitation_codes(is_active);
CREATE INDEX idx_invitation_codes_expires_at ON invitation_codes(expires_at);

CREATE INDEX idx_user_api_keys_user_id ON user_api_keys(user_id);
CREATE INDEX idx_user_api_keys_exchange ON user_api_keys(exchange);
CREATE INDEX idx_user_api_keys_is_active ON user_api_keys(is_active);

CREATE INDEX idx_opportunities_timestamp ON opportunities(timestamp);
CREATE INDEX idx_opportunities_detection_timestamp ON opportunities(detection_timestamp);
CREATE INDEX idx_opportunities_expiry_timestamp ON opportunities(expiry_timestamp);
CREATE INDEX idx_opportunities_priority_score ON opportunities(priority_score);
CREATE INDEX idx_opportunities_pair ON opportunities(pair);
CREATE INDEX idx_opportunities_type ON opportunities(type);
CREATE INDEX idx_opportunities_source ON opportunities(source);

CREATE INDEX idx_positions_user_id ON positions(user_id);
CREATE INDEX idx_positions_opportunity_id ON positions(opportunity_id);
CREATE INDEX idx_positions_exchange ON positions(exchange);
CREATE INDEX idx_positions_status ON positions(status);
CREATE INDEX idx_positions_created_at ON positions(created_at);

CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_resource_type ON audit_log(resource_type);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);

-- Telegram Group Registrations indexes
CREATE INDEX idx_telegram_group_registrations_group_type ON telegram_group_registrations(group_type);
CREATE INDEX idx_telegram_group_registrations_opportunities_enabled ON telegram_group_registrations(global_opportunities_enabled);
CREATE INDEX idx_telegram_group_registrations_registered_at ON telegram_group_registrations(registered_at);
CREATE INDEX idx_telegram_group_registrations_last_activity ON telegram_group_registrations(last_activity);

-- Insert default system configuration
INSERT OR IGNORE INTO system_config (key, value_json, description, updated_at, updated_by) VALUES
('global_opportunity_config', 
 '{"detection_interval_seconds":30,"min_threshold":0.0005,"max_threshold":0.02,"max_queue_size":100,"opportunity_ttl_minutes":10,"distribution_strategy":"RoundRobin","fairness_config":{"rotation_interval_minutes":15,"max_opportunities_per_user_per_hour":10,"max_opportunities_per_user_per_day":50,"tier_multipliers":{"Free":1.0,"Basic":1.5,"Premium":2.0,"Enterprise":3.0},"activity_boost_factor":1.2,"cooldown_period_minutes":5},"monitored_exchanges":["binance","bybit"],"monitored_pairs":["BTCUSDT","ETHUSDT"]}',
 'Global opportunity detection and distribution configuration',
 unixepoch('now') * 1000,
 'system'),
('feature_flags',
 '{"enable_ai_integration":true,"enable_auto_trading":false,"enable_reporting":true,"enable_notifications":true,"maintenance_mode":false}',
 'System-wide feature flags',
 unixepoch('now') * 1000,
 'system');

-- Insert sample notification templates for RBAC
INSERT INTO notification_templates (
    template_id,
    name,
    description,
    category,
    title_template,
    message_template,
    priority,
    channels,
    variables,
    is_system_template
) VALUES (
    'tmpl_opportunity_alert',
    'Arbitrage Opportunity Alert',
    'Notification for new arbitrage opportunities',
    'opportunity',
    '🚀 Arbitrage Opportunity: {{pair}}',
    '💰 Found {{rate_difference}}% opportunity on {{pair}}\n📈 Long: {{long_exchange}} ({{long_rate}}%)\n📉 Short: {{short_exchange}} ({{short_rate}}%)\n💵 Potential Profit: ${{potential_profit}}',
    'high',
    '["telegram"]',
    '["pair", "rate_difference", "long_exchange", "short_exchange", "long_rate", "short_rate", "potential_profit"]',
    TRUE
);

-- Views for Common RBAC Queries
CREATE VIEW IF NOT EXISTS active_users AS
SELECT 
    user_id,
    telegram_id,
    username,
    subscription_tier,
    account_status,
    last_login_at,
    COUNT(DISTINCT od.opportunity_id) as opportunities_received,
    COUNT(CASE WHEN od.user_response = 'executed' THEN 1 END) as opportunities_executed,
    MAX(ua.timestamp) as last_activity_timestamp
FROM user_profiles u
LEFT JOIN opportunity_distributions od ON u.user_id = od.user_id
LEFT JOIN user_activity ua ON u.user_id = ua.user_id
WHERE account_status = 'active'
GROUP BY u.user_id;

CREATE VIEW IF NOT EXISTS recent_opportunities AS
SELECT 
    o.id,
    o.pair,
    o.rate_difference,
    o.priority_score,
    o.detection_timestamp,
    o.expiry_timestamp,
    o.current_participants,
    o.max_participants,
    COUNT(od.id) as distribution_count,
    COUNT(CASE WHEN od.user_response = 'executed' THEN 1 END) as execution_count
FROM opportunities o
LEFT JOIN opportunity_distributions od ON o.id = od.opportunity_id
WHERE o.detection_timestamp > (unixepoch('now') * 1000) - (24 * 60 * 60 * 1000) -- Last 24 hours
GROUP BY o.id
ORDER BY o.detection_timestamp DESC;

-- Record this migration
INSERT INTO schema_migrations (version, description) VALUES ('002', 'Add indexes, views, and initial system data'); 