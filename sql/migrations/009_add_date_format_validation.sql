-- Migration 009: Add Date Format Validation
-- This migration adds CHECK constraint for YYYY-MM-DD format validation to ai_usage_tracking table

-- Add CHECK constraint for date format validation
-- Note: This requires recreating the table since SQLite doesn't support adding CHECK constraints to existing tables

-- Drop views that reference ai_usage_tracking to avoid conflicts
DROP VIEW IF EXISTS active_users;

-- Create new table with CHECK constraint
CREATE TABLE IF NOT EXISTS ai_usage_tracking_new (
    user_id TEXT NOT NULL,
    date TEXT NOT NULL CHECK (date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'), -- YYYY-MM-DD format validation
    ai_calls_used INTEGER NOT NULL DEFAULT 0,
    ai_calls_limit INTEGER NOT NULL DEFAULT 0,
    last_reset INTEGER NOT NULL,                 -- Unix timestamp
    total_cost_usd REAL NOT NULL DEFAULT 0.0,
    cost_breakdown_by_provider TEXT NOT NULL DEFAULT '{}', -- JSON object
    cost_breakdown_by_feature TEXT NOT NULL DEFAULT '{}',  -- JSON object
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000),
    PRIMARY KEY (user_id, date),
    FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- Copy existing data if any
INSERT INTO ai_usage_tracking_new 
SELECT * FROM ai_usage_tracking WHERE EXISTS (SELECT 1 FROM ai_usage_tracking LIMIT 1);

-- Drop old table
DROP TABLE IF EXISTS ai_usage_tracking;

-- Rename new table
ALTER TABLE ai_usage_tracking_new RENAME TO ai_usage_tracking;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_ai_usage_tracking_user_id ON ai_usage_tracking(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_usage_tracking_date ON ai_usage_tracking(date);
CREATE INDEX IF NOT EXISTS idx_ai_usage_tracking_user_date ON ai_usage_tracking(user_id, date);

-- Recreate triggers
CREATE TRIGGER IF NOT EXISTS validate_ai_usage_limits
BEFORE INSERT ON ai_usage_tracking
FOR EACH ROW
WHEN NEW.ai_calls_used > NEW.ai_calls_limit
BEGIN
    SELECT RAISE(ABORT, 'ai_calls_used cannot exceed ai_calls_limit');
END;

CREATE TRIGGER IF NOT EXISTS validate_ai_usage_limits_update
BEFORE UPDATE ON ai_usage_tracking
FOR EACH ROW
WHEN NEW.ai_calls_used > NEW.ai_calls_limit
BEGIN
    SELECT RAISE(ABORT, 'ai_calls_used cannot exceed ai_calls_limit');
END;

CREATE TRIGGER IF NOT EXISTS update_ai_usage_tracking_timestamp
AFTER UPDATE ON ai_usage_tracking
FOR EACH ROW
BEGIN
    UPDATE ai_usage_tracking 
    SET updated_at = strftime('%s', 'now') * 1000 
    WHERE user_id = NEW.user_id AND date = NEW.date;
END;

-- Recreate the active_users view with proper table prefixes
CREATE VIEW active_users AS
SELECT
    u.user_id,
    u.telegram_id,
    u.username,
    u.subscription_tier,
    u.account_status,
    u.last_login_at,
    COUNT(DISTINCT od.opportunity_id) as opportunities_received,
    COUNT(CASE WHEN od.user_response = 'executed' THEN 1 END) as opportunities_executed,
    MAX(ua.timestamp) as last_activity_timestamp
FROM user_profiles u
LEFT JOIN opportunity_distributions od ON u.user_id = od.user_id
LEFT JOIN user_activity ua ON u.user_id = ua.user_id
WHERE u.account_status = 'active'
GROUP BY u.user_id;

-- Record this migration
INSERT INTO schema_migrations (version, description) VALUES ('009', 'Add date format validation to ai_usage_tracking table'); 