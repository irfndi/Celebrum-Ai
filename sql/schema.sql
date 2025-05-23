-- ArbEdge D1 Database Schema
-- User Profile Management, Analytics, and Notifications
-- Updated: 2025-01-26 - Consolidated with notification system

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

-- User Profiles Table
CREATE TABLE user_profiles (
    user_id TEXT PRIMARY KEY,
    telegram_id INTEGER UNIQUE,
    username TEXT,
    
    -- API Keys (encrypted, stored as JSON)
    api_keys TEXT, -- JSON array of encrypted API keys
    
    -- User Configuration
    risk_tolerance TEXT DEFAULT 'medium', -- low, medium, high, custom
    trading_preferences TEXT, -- JSON object with preferences
    notification_settings TEXT, -- JSON object with notification preferences
    
    -- Status and Metadata
    subscription_tier TEXT DEFAULT 'free', -- free, premium, pro
    account_status TEXT DEFAULT 'active', -- active, suspended, pending
    email_verification_status TEXT DEFAULT 'pending', -- pending, verified
    
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
    trading_focus TEXT DEFAULT 'arbitrage', -- arbitrage, technical, hybrid
    experience_level TEXT DEFAULT 'beginner', -- beginner, intermediate, advanced
    risk_tolerance TEXT DEFAULT 'conservative', -- conservative, balanced, aggressive
    
    -- Automation Preferences  
    automation_level TEXT DEFAULT 'manual', -- manual, semi_auto, full_auto
    automation_scope TEXT DEFAULT 'none', -- arbitrage_only, technical_only, both, none
    
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

-- Notifications Table
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
    
    -- Foreign key references
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id),
    FOREIGN KEY (trigger_id) REFERENCES alert_triggers(trigger_id),
    FOREIGN KEY (template_id) REFERENCES notification_templates(template_id)
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

-- Opportunity Distribution Tracking
CREATE TABLE IF NOT EXISTS opportunity_distributions (
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
CREATE TABLE IF NOT EXISTS user_activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    activity_type TEXT NOT NULL, -- 'opportunity_received', 'opportunity_executed', 'login', 'api_call'
    timestamp INTEGER NOT NULL,
    metadata_json TEXT, -- Additional context
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- Invitation Codes System
CREATE TABLE IF NOT EXISTS invitation_codes (
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
CREATE TABLE IF NOT EXISTS user_api_keys (
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
CREATE TABLE IF NOT EXISTS opportunities (
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
CREATE TABLE IF NOT EXISTS positions (
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
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    description TEXT,
    updated_at INTEGER NOT NULL,
    updated_by TEXT -- User ID or 'system'
);

-- Audit Trail
CREATE TABLE IF NOT EXISTS audit_log (
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
CREATE INDEX IF NOT EXISTS idx_users_telegram_user_id ON user_profiles(telegram_id);
CREATE INDEX IF NOT EXISTS idx_users_subscription_tier ON user_profiles(subscription_tier);
CREATE INDEX IF NOT EXISTS idx_users_last_active ON user_profiles(last_login_at);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON user_profiles(account_status);

CREATE INDEX IF NOT EXISTS idx_user_api_keys_user_id ON user_api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_exchange ON user_api_keys(exchange);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_is_active ON user_api_keys(is_active);

CREATE INDEX IF NOT EXISTS idx_opportunities_timestamp ON opportunities(timestamp);
CREATE INDEX IF NOT EXISTS idx_opportunities_detection_timestamp ON opportunities(detection_timestamp);
CREATE INDEX IF NOT EXISTS idx_opportunities_expiry_timestamp ON opportunities(expiry_timestamp);
CREATE INDEX IF NOT EXISTS idx_opportunities_priority_score ON opportunities(priority_score);
CREATE INDEX IF NOT EXISTS idx_opportunities_pair ON opportunities(pair);
CREATE INDEX IF NOT EXISTS idx_opportunities_type ON opportunities(type);
CREATE INDEX IF NOT EXISTS idx_opportunities_source ON opportunities(source);

CREATE INDEX IF NOT EXISTS idx_opportunity_distributions_opportunity_id ON opportunity_distributions(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_opportunity_distributions_user_id ON opportunity_distributions(user_id);
CREATE INDEX IF NOT EXISTS idx_opportunity_distributions_distributed_at ON opportunity_distributions(distributed_at);

CREATE INDEX IF NOT EXISTS idx_user_activity_user_id ON user_activity(user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_activity_type ON user_activity(activity_type);
CREATE INDEX IF NOT EXISTS idx_user_activity_timestamp ON user_activity(timestamp);

CREATE INDEX IF NOT EXISTS idx_invitation_codes_created_by ON invitation_codes(created_by);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_is_active ON invitation_codes(is_active);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_expires_at ON invitation_codes(expires_at);

CREATE INDEX IF NOT EXISTS idx_positions_user_id ON positions(user_id);
CREATE INDEX IF NOT EXISTS idx_positions_opportunity_id ON positions(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_positions_exchange ON positions(exchange);
CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
CREATE INDEX IF NOT EXISTS idx_positions_created_at ON positions(created_at);

CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource_type ON audit_log(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);

-- User Trading Preferences indexes (Task 1.5)
CREATE INDEX IF NOT EXISTS idx_user_trading_preferences_user_id ON user_trading_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_user_trading_preferences_trading_focus ON user_trading_preferences(trading_focus);
CREATE INDEX IF NOT EXISTS idx_user_trading_preferences_automation_level ON user_trading_preferences(automation_level);
CREATE INDEX IF NOT EXISTS idx_user_trading_preferences_experience_level ON user_trading_preferences(experience_level);
CREATE INDEX IF NOT EXISTS idx_user_trading_preferences_arbitrage_enabled ON user_trading_preferences(arbitrage_enabled);
CREATE INDEX IF NOT EXISTS idx_user_trading_preferences_technical_enabled ON user_trading_preferences(technical_enabled);

-- Insert default system configuration
INSERT OR IGNORE INTO system_config (key, value_json, description, updated_at, updated_by) VALUES
('global_opportunity_config', 
 '{"detection_interval_seconds":30,"min_threshold":0.0005,"max_threshold":0.02,"max_queue_size":100,"opportunity_ttl_minutes":10,"distribution_strategy":"RoundRobin","fairness_config":{"rotation_interval_minutes":15,"max_opportunities_per_user_per_hour":10,"max_opportunities_per_user_per_day":50,"tier_multipliers":{"Free":1.0,"Basic":1.5,"Premium":2.0,"Enterprise":3.0},"activity_boost_factor":1.2,"cooldown_period_minutes":5},"monitored_exchanges":["binance","bybit"],"monitored_pairs":["BTCUSDT","ETHUSDT"]}',
 'Global opportunity detection and distribution configuration',
 unixepoch('now') * 1000,
 'system'),
('feature_flags',
 '{"enable_ai_integration":false,"enable_auto_trading":false,"enable_reporting":true,"maintenance_mode":false}',
 'System-wide feature flags',
 unixepoch('now') * 1000,
 'system');

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

CREATE VIEW IF NOT EXISTS user_statistics AS
SELECT 
    u.user_id,
    u.username,
    u.subscription_tier,
    COUNT(DISTINCT od.opportunity_id) as opportunities_received,
    COUNT(CASE WHEN od.user_response = 'executed' THEN 1 END) as opportunities_executed,
    MAX(ua.timestamp) as last_activity_timestamp
FROM user_profiles u
LEFT JOIN opportunity_distributions od ON u.user_id = od.user_id
LEFT JOIN user_activity ua ON u.user_id = ua.user_id
WHERE account_status = 'active'
GROUP BY u.user_id; 