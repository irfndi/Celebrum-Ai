-- Test minimal migration
CREATE TABLE IF NOT EXISTS invitation_codes (
    id TEXT PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    created_by_admin_id TEXT NOT NULL,
    used_by_user_id TEXT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    used_at TEXT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Record this migration
INSERT INTO schema_migrations (version, description) VALUES ('004', 'Test invitation codes table'); 