-- ArbEdge D1 Database Schema
-- User Profile Management and Analytics

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

-- Create indexes for better query performance
CREATE INDEX idx_user_profiles_telegram_id ON user_profiles(telegram_id);
CREATE INDEX idx_user_profiles_subscription_tier ON user_profiles(subscription_tier);
CREATE INDEX idx_user_profiles_account_status ON user_profiles(account_status);
CREATE INDEX idx_user_profiles_created_at ON user_profiles(created_at);

CREATE INDEX idx_user_invitations_inviter ON user_invitations(inviter_user_id);
CREATE INDEX idx_user_invitations_status ON user_invitations(status);
CREATE INDEX idx_user_invitations_type ON user_invitations(invitation_type);
CREATE INDEX idx_user_invitations_created_at ON user_invitations(created_at);

CREATE INDEX idx_trading_analytics_user_id ON trading_analytics(user_id);
CREATE INDEX idx_trading_analytics_metric_type ON trading_analytics(metric_type);
CREATE INDEX idx_trading_analytics_timestamp ON trading_analytics(timestamp);
CREATE INDEX idx_trading_analytics_date_bucket ON trading_analytics(date_bucket);
CREATE INDEX idx_trading_analytics_exchange ON trading_analytics(exchange_id);

-- Create indexes for balance history performance
CREATE INDEX idx_balance_history_user_id ON balance_history(user_id);
CREATE INDEX idx_balance_history_exchange_id ON balance_history(exchange_id);
CREATE INDEX idx_balance_history_asset ON balance_history(asset);
CREATE INDEX idx_balance_history_timestamp ON balance_history(timestamp);
CREATE INDEX idx_balance_history_snapshot_id ON balance_history(snapshot_id);
CREATE INDEX idx_balance_history_user_exchange ON balance_history(user_id, exchange_id);
CREATE INDEX idx_balance_history_user_asset ON balance_history(user_id, asset);

-- Create indexes for notification system performance
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

CREATE INDEX idx_notification_history_notification_id ON notification_history(notification_id);
CREATE INDEX idx_notification_history_user_id ON notification_history(user_id);
CREATE INDEX idx_notification_history_channel ON notification_history(channel);
CREATE INDEX idx_notification_history_delivery_status ON notification_history(delivery_status);
CREATE INDEX idx_notification_history_attempted_at ON notification_history(attempted_at);

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

INSERT INTO user_invitations (
    invitation_id,
    inviter_user_id,
    invitee_identifier,
    invitation_type,
    status,
    message
) VALUES (
    'inv_sample_001',
    'user_123456789',
    'friend@example.com',
    'email',
    'pending',
    'Join me on ArbEdge for crypto arbitrage opportunities!'
);

INSERT INTO trading_analytics (
    analytics_id,
    user_id,
    metric_type,
    metric_value,
    metric_data,
    exchange_id,
    trading_pair,
    opportunity_type
) VALUES (
    'analytics_sample_001',
    'user_123456789',
    'opportunity_found',
    2.5, -- 2.5% arbitrage opportunity
    '{"spread_percentage": 2.5, "volume": 1000, "confidence": 0.85}',
    'binance',
    'BTC/USDT',
    'arbitrage'
);

-- Insert sample balance history data for testing
INSERT INTO balance_history (
    id,
    user_id,
    exchange_id,
    asset,
    balance_data,
    usd_value,
    timestamp,
    snapshot_id
) VALUES (
    'bal_hist_001',
    'user_123456789',
    'binance',
    'BTC',
    '{"free": 0.5, "used": 0.0, "total": 0.5}',
    21500.0, -- 0.5 BTC at $43,000
    1672531200000, -- Sample timestamp
    'snapshot_001'
),
(
    'bal_hist_002',
    'user_123456789',
    'binance',
    'ETH',
    '{"free": 10.0, "used": 0.0, "total": 10.0}',
    26000.0, -- 10 ETH at $2,600
    1672531200000,
    'snapshot_001'
),
(
    'bal_hist_003',
    'user_123456789',
    'coinbase',
    'USDT',
    '{"free": 5000.0, "used": 0.0, "total": 5000.0}',
    5000.0, -- 5000 USDT at $1
    1672531200000,
    'snapshot_002'
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
),
(
    'tmpl_balance_alert',
    'Balance Change Alert',
    'Notification for significant balance changes',
    'balance',
    '⚠️ Balance Alert: {{asset}}',
    '💼 Your {{asset}} balance changed by {{change_amount}} ({{change_percentage}}%)\n🏦 Exchange: {{exchange}}\n💰 New Balance: {{new_balance}}',
    'medium',
    '["telegram"]',
    '["asset", "change_amount", "change_percentage", "exchange", "new_balance"]',
    TRUE
),
(
    'tmpl_risk_alert',
    'Risk Management Alert',
    'Notification for risk-related events',
    'risk',
    '🛡️ Risk Alert: {{risk_type}}',
    '⚠️ {{message}}\n📊 Current Risk Level: {{risk_level}}\n💡 Recommendation: {{recommendation}}',
    'critical',
    '["telegram"]',
    '["risk_type", "message", "risk_level", "recommendation"]',
    TRUE
);

-- Insert sample alert triggers
INSERT INTO alert_triggers (
    trigger_id,
    user_id,
    name,
    description,
    trigger_type,
    conditions,
    template_id,
    is_active,
    priority,
    channels,
    cooldown_minutes,
    max_alerts_per_hour
) VALUES (
    'trigger_high_opportunity',
    'user_123456789',
    'High Opportunity Alert',
    'Alert for arbitrage opportunities above 2%',
    'opportunity_threshold',
    '{"min_rate_difference": 0.02, "min_profit": 50.0, "pairs": ["BTC/USDT", "ETH/USDT"]}',
    'tmpl_opportunity_alert',
    TRUE,
    'high',
    '["telegram"]',
    5,
    6
),
(
    'trigger_balance_change',
    'user_123456789',
    'Significant Balance Change',
    'Alert for balance changes over 10%',
    'balance_change',
    '{"min_change_percentage": 0.10, "assets": ["BTC", "ETH", "USDT"], "exchanges": ["binance", "coinbase"]}',
    'tmpl_balance_alert',
    TRUE,
    'medium',
    '["telegram"]',
    10,
    4
);

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

-- Views for Common Queries
CREATE VIEW IF NOT EXISTS active_users AS
SELECT 
    user_id,
    telegram_id as telegram_user_id,
    username as telegram_username,
    subscription_tier,
    last_login_at as last_active,
    profile_metadata
FROM user_profiles 
WHERE account_status = 'active';

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
    u.username as telegram_username,
    u.subscription_tier,
    COUNT(DISTINCT od.opportunity_id) as opportunities_received,
    COUNT(CASE WHEN od.user_response = 'executed' THEN 1 END) as opportunities_executed,
    MAX(ua.timestamp) as last_activity_timestamp,
    COUNT(DISTINCT p.id) as total_positions,
    SUM(CASE WHEN p.status = 'Closed' AND p.pnl > 0 THEN p.pnl ELSE 0 END) as total_profit,
    SUM(CASE WHEN p.status = 'Closed' AND p.pnl < 0 THEN p.pnl ELSE 0 END) as total_loss
FROM user_profiles u
LEFT JOIN opportunity_distributions od ON u.user_id = od.user_id
LEFT JOIN user_activity ua ON u.user_id = ua.user_id
LEFT JOIN positions p ON u.user_id = p.user_id
WHERE u.account_status = 'active'
GROUP BY u.user_id;

CREATE VIEW IF NOT EXISTS notification_analytics AS
SELECT 
    n.user_id,
    n.category,
    n.priority,
    COUNT(*) as total_notifications,
    COUNT(CASE WHEN n.status = 'sent' THEN 1 END) as sent_count,
    COUNT(CASE WHEN n.status = 'failed' THEN 1 END) as failed_count,
    AVG(nh.delivery_time_ms) as avg_delivery_time_ms
FROM notifications n
LEFT JOIN notification_history nh ON n.notification_id = nh.notification_id
GROUP BY n.user_id, n.category, n.priority;

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

-- Create indexes for additional tables
CREATE INDEX IF NOT EXISTS idx_opportunity_distributions_opportunity_id ON opportunity_distributions(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_opportunity_distributions_user_id ON opportunity_distributions(user_id);
CREATE INDEX IF NOT EXISTS idx_opportunity_distributions_distributed_at ON opportunity_distributions(distributed_at);

CREATE INDEX IF NOT EXISTS idx_user_activity_user_id ON user_activity(user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_activity_type ON user_activity(activity_type);
CREATE INDEX IF NOT EXISTS idx_user_activity_timestamp ON user_activity(timestamp);

CREATE INDEX IF NOT EXISTS idx_invitation_codes_created_by ON invitation_codes(created_by);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_is_active ON invitation_codes(is_active);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_expires_at ON invitation_codes(expires_at);

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

CREATE INDEX IF NOT EXISTS idx_positions_user_id ON positions(user_id);
CREATE INDEX IF NOT EXISTS idx_positions_opportunity_id ON positions(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_positions_exchange ON positions(exchange);
CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
CREATE INDEX IF NOT EXISTS idx_positions_created_at ON positions(created_at);

CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource_type ON audit_log(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp); 