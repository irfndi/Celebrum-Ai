-- Migration 006: Add user opportunity limits table
-- This table tracks daily opportunity limits for users based on their access level and context

CREATE TABLE IF NOT EXISTS user_opportunity_limits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    access_level TEXT NOT NULL CHECK (access_level IN ('free_without_api', 'free_with_api', 'subscription_with_api')),
    date TEXT NOT NULL, -- YYYY-MM-DD format
    context_id TEXT NOT NULL, -- 'private', 'group_123', 'channel_456', etc.
    arbitrage_opportunities_received INTEGER NOT NULL DEFAULT 0,
    technical_opportunities_received INTEGER NOT NULL DEFAULT 0,
    arbitrage_limit INTEGER NOT NULL,
    technical_limit INTEGER NOT NULL,
    last_reset INTEGER NOT NULL, -- Unix timestamp
    is_group_context INTEGER NOT NULL DEFAULT 0, -- Boolean: 0 = false, 1 = true
    group_multiplier_applied INTEGER NOT NULL DEFAULT 0, -- Boolean: 0 = false, 1 = true
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_user_opportunity_limits_user_date_context 
ON user_opportunity_limits(user_id, date, context_id);

CREATE INDEX IF NOT EXISTS idx_user_opportunity_limits_date 
ON user_opportunity_limits(date);

CREATE INDEX IF NOT EXISTS idx_user_opportunity_limits_access_level 
ON user_opportunity_limits(access_level);

-- Create unique constraint to prevent duplicate entries
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_opportunity_limits_unique 
ON user_opportunity_limits(user_id, date, context_id);

-- Add trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_user_opportunity_limits_updated_at
    AFTER UPDATE ON user_opportunity_limits
    FOR EACH ROW
BEGIN
    UPDATE user_opportunity_limits 
    SET updated_at = strftime('%s', 'now') * 1000 
    WHERE id = NEW.id;
END; 