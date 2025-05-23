-- ArbEdge D1 Database Schema
-- User Profile Management and Analytics

-- Drop existing tables if they exist (for development/migration purposes)
DROP TABLE IF EXISTS user_profiles;
DROP TABLE IF EXISTS user_invitations;
DROP TABLE IF EXISTS trading_analytics;
DROP TABLE IF EXISTS balance_history;

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