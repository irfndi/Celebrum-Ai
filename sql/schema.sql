-- D1 Database Schema for ArbEdge Hybrid Storage Architecture
-- Created: 2025-01-26

-- User Profiles and Subscription Management
CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    telegram_user_id INTEGER UNIQUE NOT NULL,
    telegram_username TEXT,
    subscription_tier TEXT NOT NULL DEFAULT 'Free',
    subscription_is_active BOOLEAN DEFAULT true,
    subscription_expires_at INTEGER,
    subscription_created_at INTEGER NOT NULL,
    configuration_json TEXT NOT NULL, -- JSON blob for user configuration
    invitation_code TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_active INTEGER NOT NULL,
    is_active BOOLEAN DEFAULT true,
    total_trades INTEGER DEFAULT 0,
    total_pnl_usdt REAL DEFAULT 0.0
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
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    UNIQUE(user_id, exchange)
);

-- Historical Opportunities
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
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- User Activity and Fairness Tracking
CREATE TABLE IF NOT EXISTS user_activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    activity_type TEXT NOT NULL, -- 'opportunity_received', 'opportunity_executed', 'login', 'api_call'
    timestamp INTEGER NOT NULL,
    metadata_json TEXT, -- Additional context
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
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
    FOREIGN KEY (created_by) REFERENCES users(user_id)
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
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
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
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Indexes for Performance
CREATE INDEX IF NOT EXISTS idx_users_telegram_user_id ON users(telegram_user_id);
CREATE INDEX IF NOT EXISTS idx_users_subscription_tier ON users(subscription_tier);
CREATE INDEX IF NOT EXISTS idx_users_last_active ON users(last_active);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);

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
    telegram_user_id,
    telegram_username,
    subscription_tier,
    last_active,
    total_trades,
    total_pnl_usdt
FROM users 
WHERE is_active = true 
AND subscription_is_active = true;

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
    u.telegram_username,
    u.subscription_tier,
    u.total_trades,
    u.total_pnl_usdt,
    COUNT(DISTINCT od.opportunity_id) as opportunities_received,
    COUNT(CASE WHEN od.user_response = 'executed' THEN 1 END) as opportunities_executed,
    MAX(ua.timestamp) as last_activity_timestamp
FROM users u
LEFT JOIN opportunity_distributions od ON u.user_id = od.user_id
LEFT JOIN user_activity ua ON u.user_id = ua.user_id
WHERE u.is_active = true
GROUP BY u.user_id; 