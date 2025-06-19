-- Add indexes for invitation codes table performance optimization
-- Migration 008: Add indexes on frequently queried fields

-- Add indexes on frequently queried fields for better lookup performance
CREATE INDEX IF NOT EXISTS idx_invitation_codes_code ON invitation_codes(code);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_expires_at ON invitation_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_is_active ON invitation_codes(is_active);
CREATE INDEX IF NOT EXISTS idx_invitation_codes_created_by ON invitation_codes(created_by);

-- Record this migration
INSERT INTO schema_migrations (version, description) VALUES ('008', 'Add indexes for invitation codes table performance'); 