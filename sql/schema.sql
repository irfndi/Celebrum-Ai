-- ArbEdge D1 Database Schema
-- User Profile Management, Analytics, Notifications, and AI Intelligence
-- Updated: 2025-01-26 - Consolidated with notification system and AI intelligence tables

-- Drop existing tables if they exist (for development/migration purposes)
DROP TABLE IF EXISTS user_profiles;
DROP TABLE IF EXISTS user_invitations;
DROP TABLE IF EXISTS trading_analytics;
DROP TABLE IF EXISTS balance_history;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS notification_templates;
DROP TABLE IF EXISTS alert_triggers;
DROP TABLE IF EXISTS notification_history;
DROP TABLE IF EXISTS opportunity_distributions;
DROP TABLE IF EXISTS user_activity;
DROP TABLE IF EXISTS invitation_codes;
DROP TABLE IF EXISTS user_api_keys;
DROP TABLE IF EXISTS opportunities;
DROP TABLE IF EXISTS positions;
DROP TABLE IF EXISTS system_config;
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS user_trading_preferences;
-- AI Intelligence Tables
DROP TABLE IF EXISTS ai_opportunity_enhancements;
DROP TABLE IF EXISTS ai_portfolio_analysis;
DROP TABLE IF EXISTS ai_performance_insights;
DROP TABLE IF EXISTS ai_parameter_suggestions;
DROP TABLE IF EXISTS user_opportunity_preferences;

-- Telegram Group/Channel Registrations Table
CREATE TABLE telegram_group_registrations (
    group_id TEXT PRIMARY KEY,
    group_type TEXT NOT NULL CHECK (group_type IN ('group', 'supergroup', 'channel')),
    group_title TEXT,
    group_username TEXT,
    member_count INTEGER,
    admin_user_ids TEXT, -- JSON array of Telegram user IDs
    bot_permissions TEXT, -- JSON array of bot permissions
    enabled_features TEXT, -- JSON array of enabled features
    global_opportunities_enabled BOOLEAN DEFAULT TRUE,
    technical_analysis_enabled BOOLEAN DEFAULT FALSE,
    
    -- Rate limiting configuration (JSON object)
    rate_limit_config TEXT,
    
    -- Activity tracking
    registered_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,
    total_messages_sent INTEGER DEFAULT 0,
    last_member_count_update INTEGER,
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- User Profiles Table
CREATE TABLE user_profiles (
    user_id TEXT PRIMARY KEY NOT NULL,
    telegram_id INTEGER NOT NULL UNIQUE CHECK (telegram_id > 0),
    username TEXT,
    
    -- API Keys (encrypted, stored as JSON)
    api_keys TEXT, -- JSON array of encrypted API keys
    
    -- User Configuration
    risk_tolerance TEXT DEFAULT 'medium' CHECK (risk_tolerance IN ('low', 'medium', 'high', 'custom')),
    trading_preferences TEXT, -- JSON object with preferences
    notification_settings TEXT, -- JSON object with notification preferences
    
    -- Status and Metadata
    subscription_tier TEXT DEFAULT 'free' CHECK (subscription_tier IN ('free', 'basic', 'premium', 'pro')),
    account_status TEXT DEFAULT 'active' CHECK (account_status IN ('active', 'suspended', 'pending', 'deactivated')),
    email_verification_status TEXT DEFAULT 'pending' CHECK (email_verification_status IN ('pending', 'verified', 'failed')),
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    last_login_at TEXT,
    
    -- Additional metadata
    profile_metadata TEXT -- JSON object for additional profile data
);

-- User Trading Preferences Table (Task 1.5)
CREATE TABLE user_trading_preferences (
    preference_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE,
    
    -- Trading Focus Selection
    trading_focus TEXT DEFAULT 'arbitrage' CHECK (trading_focus IN ('arbitrage', 'technical', 'hybrid')),
    experience_level TEXT DEFAULT 'beginner' CHECK (experience_level IN ('beginner', 'intermediate', 'advanced')),
    risk_tolerance TEXT DEFAULT 'conservative' CHECK (risk_tolerance IN ('conservative', 'balanced', 'aggressive')),
    
    -- Automation Preferences  
    automation_level TEXT DEFAULT 'manual' CHECK (automation_level IN ('manual', 'semi_auto', 'full_auto')),
    automation_scope TEXT DEFAULT 'none' CHECK (automation_scope IN ('none', 'arbitrage_only', 'technical_only', 'both')),
    
    -- Feature Access Control
    arbitrage_enabled BOOLEAN DEFAULT TRUE,
    technical_enabled BOOLEAN DEFAULT FALSE,
    advanced_analytics_enabled BOOLEAN DEFAULT FALSE,
    
    -- User Preferences
    preferred_notification_channels TEXT, -- JSON array: ["telegram", "email", "push"]
    trading_hours_timezone TEXT DEFAULT 'UTC',
    trading_hours_start TEXT DEFAULT '00:00',
    trading_hours_end TEXT DEFAULT '23:59',
    
    -- Onboarding Progress
    onboarding_completed BOOLEAN DEFAULT FALSE,
    tutorial_steps_completed TEXT, -- JSON array of completed tutorial steps
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    
    -- Foreign key reference
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- User Opportunity Preferences Table (for Comment 23-24)
CREATE TABLE user_opportunity_preferences (
    user_id TEXT PRIMARY KEY,
    preferences_json TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- User Invitations Table
CREATE TABLE user_invitations (
    invitation_id TEXT PRIMARY KEY,
    inviter_user_id TEXT NOT NULL,
    invitee_identifier TEXT NOT NULL, -- email, telegram username, or phone
    invitation_type TEXT NOT NULL, -- email, telegram, referral
    
    status TEXT DEFAULT 'pending', -- pending, accepted, expired, cancelled
    
    -- Invitation Details
    message TEXT,
    invitation_data TEXT, -- JSON object with invitation-specific data
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    expires_at TEXT,
    accepted_at TEXT,
    
    -- Foreign key reference
    FOREIGN KEY (inviter_user_id) REFERENCES user_profiles(user_id)
);

-- Trading Analytics Table
CREATE TABLE trading_analytics (
    analytics_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    
    -- Analytics Data
    metric_type TEXT NOT NULL, -- opportunity_found, trade_executed, profit_loss, etc.
    metric_value REAL,
    metric_data TEXT, -- JSON object with detailed metric data
    
    -- Exchange and Trading Context
    exchange_id TEXT,
    trading_pair TEXT,
    opportunity_type TEXT, -- arbitrage, momentum, pattern, etc.
    
    -- Timestamps
    timestamp TEXT DEFAULT (datetime('now')),
    date_bucket TEXT, -- For aggregation: YYYY-MM-DD format
    
    -- Additional context
    session_id TEXT,
    analytics_metadata TEXT, -- JSON object for additional analytics data
    
    -- Foreign key reference
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id)
);

-- Balance History Table for Fund Monitoring
CREATE TABLE balance_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    exchange_id TEXT NOT NULL,
    asset TEXT NOT NULL,
    balance_data TEXT NOT NULL, -- JSON object with free, used, total balance
    usd_value REAL NOT NULL DEFAULT 0.0,
    timestamp INTEGER NOT NULL,
    snapshot_id TEXT NOT NULL,
    
    -- Foreign key reference
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id)
);

-- Notification Templates Table
CREATE TABLE notification_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL, -- opportunity, risk, balance, system, custom
    
    -- Template Content
    title_template TEXT NOT NULL,
    message_template TEXT NOT NULL,
    priority TEXT DEFAULT 'medium', -- low, medium, high, critical
    
    -- Channel Configuration
    channels TEXT NOT NULL, -- JSON array of supported channels: ["telegram", "email", "push"]
    
    -- Template Metadata
    variables TEXT, -- JSON array of template variables
    is_system_template BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Alert Triggers Table
CREATE TABLE alert_triggers (
    trigger_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    
    -- Trigger Configuration
    trigger_type TEXT NOT NULL, -- opportunity_threshold, balance_change, price_alert, profit_loss, custom
    conditions TEXT NOT NULL, -- JSON object with trigger conditions
    template_id TEXT,
    
    -- Trigger Settings
    is_active BOOLEAN DEFAULT TRUE,
    priority TEXT DEFAULT 'medium',
    channels TEXT NOT NULL, -- JSON array of notification channels
    
    -- Rate Limiting
    cooldown_minutes INTEGER DEFAULT 5,
    max_alerts_per_hour INTEGER DEFAULT 10,
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    last_triggered_at TEXT,
    
    -- Foreign key reference
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id),
    FOREIGN KEY (template_id) REFERENCES notification_templates(template_id)
);

-- Notifications Table (with partitioning strategy for large-scale data management)
CREATE TABLE notifications (
    notification_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    trigger_id TEXT,
    template_id TEXT,
    
    -- Notification Content
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    category TEXT NOT NULL,
    priority TEXT DEFAULT 'medium',
    
    -- Notification Data
    notification_data TEXT, -- JSON object with contextual data
    
    -- Delivery Configuration
    channels TEXT NOT NULL, -- JSON array of channels to send to
    
    -- Status Tracking
    status TEXT DEFAULT 'pending', -- pending, sent, failed, cancelled
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    scheduled_at TEXT,
    sent_at TEXT,
    
    -- Partitioning Strategy: Add date partition key for time-based partitioning
    date_partition TEXT GENERATED ALWAYS AS (date(created_at)) STORED,
    
    -- Foreign key references
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id),
    FOREIGN KEY (trigger_id) REFERENCES alert_triggers(trigger_id),
    FOREIGN KEY (template_id) REFERENCES notification_templates(template_id),
    
    -- Retention Policy Note: Consider implementing automated cleanup for notifications older than 90 days
    -- This should be implemented as a scheduled job: DELETE FROM notifications WHERE date(created_at) < date('now', '-90 days')
);

-- Notification History Table
CREATE TABLE notification_history (
    history_id TEXT PRIMARY KEY,
    notification_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    
    -- Delivery Details
    channel TEXT NOT NULL, -- telegram, email, push
    delivery_status TEXT NOT NULL, -- success, failed, retrying
    
    -- Response Data
    response_data TEXT, -- JSON object with delivery response
    error_message TEXT,
    
    -- Performance Metrics
    delivery_time_ms INTEGER,
    retry_count INTEGER DEFAULT 0,
    
    -- Timestamps
    attempted_at TEXT DEFAULT (datetime('now')),
    delivered_at TEXT,
    
    -- Foreign key references
    FOREIGN KEY (notification_id) REFERENCES notifications(notification_id),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id)
);

-- AI Intelligence Tables (Comment 28)
CREATE TABLE ai_opportunity_enhancements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opportunity_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    ai_confidence_score REAL NOT NULL,
    ai_risk_assessment TEXT NOT NULL, -- JSON object
    ai_recommendations TEXT NOT NULL, -- JSON array
    position_sizing_suggestion REAL NOT NULL,
    timing_score REAL NOT NULL,
    technical_confirmation REAL NOT NULL,
    portfolio_impact_score REAL NOT NULL,
    ai_provider_used TEXT NOT NULL,
    analysis_timestamp INTEGER NOT NULL,
    enhancement_data TEXT NOT NULL, -- Full JSON serialization of enhancement
    created_at INTEGER NOT NULL DEFAULT (unixepoch('now') * 1000),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

CREATE TABLE ai_portfolio_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    correlation_risk_score REAL NOT NULL,
    concentration_risk_score REAL NOT NULL,
    diversification_score REAL NOT NULL,
    recommended_adjustments TEXT NOT NULL, -- JSON array
    overexposure_warnings TEXT NOT NULL, -- JSON array
    optimal_allocation_suggestions TEXT NOT NULL, -- JSON object
    analysis_timestamp INTEGER NOT NULL,
    analysis_data TEXT NOT NULL, -- Full JSON serialization
    created_at INTEGER NOT NULL DEFAULT (unixepoch('now') * 1000),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

CREATE TABLE ai_performance_insights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    performance_score REAL NOT NULL,
    strengths TEXT NOT NULL, -- JSON array
    weaknesses TEXT NOT NULL, -- JSON array
    suggested_focus_adjustment TEXT, -- Trading focus suggestion
    parameter_optimization_suggestions TEXT NOT NULL, -- JSON array
    learning_recommendations TEXT NOT NULL, -- JSON array
    automation_readiness_score REAL NOT NULL,
    generated_at INTEGER NOT NULL,
    insights_data TEXT NOT NULL, -- Full JSON serialization
    created_at INTEGER NOT NULL DEFAULT (unixepoch('now') * 1000),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

CREATE TABLE ai_parameter_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    parameter_name TEXT NOT NULL,
    current_value TEXT NOT NULL,
    suggested_value TEXT NOT NULL,
    rationale TEXT NOT NULL,
    impact_assessment REAL NOT NULL,
    confidence REAL NOT NULL,
    suggestion_data TEXT NOT NULL, -- Full JSON serialization
    created_at INTEGER NOT NULL DEFAULT (unixepoch('now') * 1000),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- Opportunity Distribution Tracking
CREATE TABLE opportunity_distributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opportunity_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    distributed_at INTEGER NOT NULL,
    user_response TEXT, -- 'viewed', 'executed', 'ignored'
    response_at INTEGER,
    execution_result_json TEXT, -- JSON blob for execution details
    FOREIGN KEY (opportunity_id) REFERENCES opportunities(id),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- User Activity and Fairness Tracking
CREATE TABLE user_activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    activity_type TEXT NOT NULL, -- 'opportunity_received', 'opportunity_executed', 'login', 'api_call'
    timestamp INTEGER NOT NULL,
    metadata_json TEXT, -- Additional context
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- Invitation Codes System
CREATE TABLE invitation_codes (
    code TEXT PRIMARY KEY,
    created_by TEXT, -- User ID who created this code
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    max_uses INTEGER,
    current_uses INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    purpose TEXT NOT NULL, -- 'beta_testing', 'referral', 'admin'
    FOREIGN KEY (created_by) REFERENCES user_profiles(user_id)
);

-- User API Keys (encrypted)
CREATE TABLE user_api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    exchange TEXT NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    secret_encrypted TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at INTEGER NOT NULL,
    last_validated INTEGER,
    permissions_json TEXT, -- JSON array of permissions
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE,
    UNIQUE(user_id, exchange)
);

-- Historical Opportunities (extended)
CREATE TABLE opportunities (
    id TEXT PRIMARY KEY,
    pair TEXT NOT NULL,
    long_exchange TEXT,
    short_exchange TEXT,
    long_rate REAL,
    short_rate REAL,
    rate_difference REAL NOT NULL,
    net_rate_difference REAL,
    potential_profit_value REAL,
    timestamp INTEGER NOT NULL,
    type TEXT NOT NULL, -- 'FundingRate', 'SpotFutures', 'CrossExchange'
    details TEXT,
    -- Global opportunity fields
    detection_timestamp INTEGER NOT NULL,
    expiry_timestamp INTEGER NOT NULL,
    priority_score REAL NOT NULL,
    max_participants INTEGER,
    current_participants INTEGER DEFAULT 0,
    distribution_strategy TEXT NOT NULL,
    source TEXT NOT NULL, -- 'SystemGenerated', 'UserAI', 'External'
    source_user_id TEXT, -- If source is UserAI
    created_at INTEGER NOT NULL DEFAULT (unixepoch('now') * 1000)
);

-- Trading Positions
CREATE TABLE positions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    opportunity_id TEXT, -- Link to the opportunity that created this position
    exchange TEXT NOT NULL,
    pair TEXT NOT NULL,
    side TEXT NOT NULL, -- 'Long', 'Short'
    size REAL NOT NULL,
    entry_price REAL NOT NULL,
    current_price REAL,
    pnl REAL,
    status TEXT NOT NULL DEFAULT 'Open', -- 'Open', 'Closed', 'Pending'
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    closed_at INTEGER,
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE,
    FOREIGN KEY (opportunity_id) REFERENCES opportunities(id)
);

-- System Configuration
CREATE TABLE system_config (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    description TEXT,
    updated_at INTEGER NOT NULL,
    updated_by TEXT -- User ID or 'system'
);

-- Audit Trail
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL, -- 'user', 'opportunity', 'position', 'config'
    resource_id TEXT,
    old_value_json TEXT,
    new_value_json TEXT,
    timestamp INTEGER NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id)
);

-- Indexes for Performance
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

-- Insert sample data for testing
INSERT INTO user_profiles (
    user_id, 
    telegram_id, 
    username, 
    api_keys, 
    risk_tolerance, 
    trading_preferences,
    subscription_tier,
    account_status
) VALUES (
    'user_123456789',
    123456789,
    'test_user',
    '[]', -- Empty API keys array
    'medium',
    '{"max_position_size": 1000, "auto_trading": false}',
    'free',
    'active'
);

-- Insert sample user trading preferences
INSERT INTO user_trading_preferences (
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
    onboarding_completed
) VALUES (
    'pref_123456789',
    'user_123456789',
    'arbitrage',
    'beginner',
    'conservative',
    'manual',
    'none',
    TRUE,
    FALSE,
    FALSE,
    '["telegram"]',
    FALSE
);

-- Insert sample notification templates
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

-- Views for Common Queries
CREATE VIEW IF NOT EXISTS active_users AS
SELECT 
    user_id,
    telegram_id,
    username,
    subscription_tier,
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