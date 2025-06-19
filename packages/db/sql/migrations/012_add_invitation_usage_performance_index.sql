-- Migration 012: Add performance index for invitation usage queries
-- Created: 2025-05-26
-- Description: Add composite index on invitation_usage table for better performance

-- Add composite index for user_id and beta_expires_at columns
-- This significantly improves query performance for the has_active_beta_access method
-- which uses complex conditions with typeof() checks
CREATE INDEX IF NOT EXISTS idx_invitation_usage_user_beta 
ON invitation_usage(user_id, beta_expires_at);

-- Record migration
INSERT INTO schema_migrations (version, description) 
VALUES ('012', 'Add composite index on invitation_usage for performance optimization'); 