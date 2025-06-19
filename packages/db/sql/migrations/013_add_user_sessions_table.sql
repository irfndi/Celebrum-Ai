-- Migration 013: Add User Sessions Table
-- Purpose: Implement session management for Telegram bot users
-- Date: 2025-01-28
-- Related: Session Management & Opportunity Distribution System

-- User Sessions Table
-- Tracks active user sessions with lifecycle management
CREATE TABLE IF NOT EXISTS user_sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    telegram_id INTEGER NOT NULL,
    session_state TEXT NOT NULL CHECK (session_state IN ('active', 'expired', 'terminated')),
    started_at INTEGER NOT NULL,
    last_activity_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    onboarding_completed BOOLEAN DEFAULT FALSE,
    preferences_set BOOLEAN DEFAULT FALSE,
    metadata TEXT, -- JSON for additional session data
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Indexes for efficient session lookups
CREATE INDEX IF NOT EXISTS idx_user_sessions_telegram_id ON user_sessions(telegram_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_state ON user_sessions(session_state);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_last_activity ON user_sessions(last_activity_at);

-- Composite index for active session lookups
CREATE INDEX IF NOT EXISTS idx_user_sessions_active_lookup ON user_sessions(telegram_id, session_state, expires_at);

-- Record migration
INSERT INTO d1_migrations (name, applied_at) 
VALUES ('013_add_user_sessions_table', strftime('%s', 'now') * 1000); 