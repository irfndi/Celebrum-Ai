-- Migration 011: Add beta_expires_at column to user_profiles
-- Created: 2025-05-26
-- Description: Add beta_expires_at column to track beta access expiration for invited users

-- Add beta_expires_at column to user_profiles table
ALTER TABLE user_profiles 
ADD COLUMN beta_expires_at INTEGER NOT NULL DEFAULT 0;

-- Add comment for documentation
-- beta_expires_at: Unix timestamp (milliseconds) when beta access expires
-- Default 0 means no beta access or expired
-- Used by invitation system to track 90-day beta periods

-- Record migration
INSERT INTO schema_migrations (version, description) 
VALUES ('011', 'Add beta_expires_at column to track beta access expiration for invited users'); 