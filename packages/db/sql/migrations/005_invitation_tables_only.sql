-- Migration 005: Invitation System Tables Only
-- Created: 2025-01-28
-- Description: Add invitation system tables without indexes or views

-- Table for tracking invitation usage and beta periods
CREATE TABLE IF NOT EXISTS invitation_usage (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    invitation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    telegram_id INTEGER NOT NULL,
    used_at TEXT NOT NULL, -- ISO 8601 datetime
    beta_expires_at TEXT NOT NULL, -- ISO 8601 datetime (90 days from usage)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id) -- One invitation usage per user
);

-- Table for user referral codes (every user gets one)
CREATE TABLE IF NOT EXISTS user_referral_codes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    referral_code TEXT UNIQUE NOT NULL, -- User's personal referral code
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL, -- ISO 8601 datetime
    updated_at TEXT NOT NULL, -- ISO 8601 datetime
    total_uses INTEGER NOT NULL DEFAULT 0,
    total_bonuses_earned REAL NOT NULL DEFAULT 0.0,
    last_used_at TEXT NULL, -- ISO 8601 datetime
    UNIQUE(user_id) -- One referral code per user
);

-- Table for tracking referral usage and bonuses
CREATE TABLE IF NOT EXISTS referral_usage (
    id TEXT PRIMARY KEY,
    referrer_user_id TEXT NOT NULL,
    referred_user_id TEXT NOT NULL,
    referral_code TEXT NOT NULL,
    used_at TEXT NOT NULL, -- ISO 8601 datetime
    bonus_awarded REAL NOT NULL DEFAULT 0.0,
    bonus_type TEXT NOT NULL, -- 'FeatureAccess', 'RevenueKickback', 'Points', 'SubscriptionDiscount'
    conversion_status TEXT NOT NULL, -- 'Registered', 'FirstTrade', 'Subscribed', 'ActiveUser'
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(referred_user_id) -- One referral per user
);

-- Table for affiliation program applications
CREATE TABLE IF NOT EXISTS affiliation_applications (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    application_type TEXT NOT NULL, -- 'influencer', 'educator', 'community_leader'
    follower_count INTEGER NOT NULL DEFAULT 0,
    platform_details TEXT NOT NULL, -- JSON with social media details
    verification_documents TEXT NULL, -- JSON with document URLs/hashes
    application_status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'under_review'
    reviewed_by_admin_id TEXT NULL,
    reviewed_at TEXT NULL, -- ISO 8601 datetime
    review_notes TEXT NULL,
    applied_at TEXT NOT NULL, -- ISO 8601 datetime
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Table for approved affiliation programs
CREATE TABLE IF NOT EXISTS affiliation_programs (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    program_type TEXT NOT NULL, -- 'influencer', 'educator', 'community_leader'
    tier_level TEXT NOT NULL, -- 'bronze', 'silver', 'gold', 'platinum'
    kickback_rate REAL NOT NULL, -- Percentage rate (e.g., 0.15 for 15%)
    revenue_share_rate REAL NOT NULL, -- Revenue sharing percentage
    special_features TEXT NOT NULL, -- JSON array of special features
    performance_metrics TEXT NOT NULL, -- JSON with performance tracking
    is_active BOOLEAN NOT NULL DEFAULT true,
    activated_at TEXT NOT NULL, -- ISO 8601 datetime
    last_performance_review TEXT NULL, -- ISO 8601 datetime
    total_revenue_generated REAL NOT NULL DEFAULT 0.0,
    total_referrals INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id) -- One active program per user
);

-- Insert default referral bonus configuration
INSERT OR IGNORE INTO system_config (key, value_json, description, updated_at, updated_by) VALUES 
('referral_bonus_feature_access', '"0.0"', 'Monetary value for feature access bonus', unixepoch('now') * 1000, 'migration_005'),
('referral_bonus_revenue_kickback', '"5.0"', 'Revenue kickback amount in USD', unixepoch('now') * 1000, 'migration_005'),
('referral_bonus_points', '"100.0"', 'Points awarded for referrals', unixepoch('now') * 1000, 'migration_005'),
('referral_bonus_subscription_discount', '"10.0"', 'Subscription discount value in USD', unixepoch('now') * 1000, 'migration_005'),
('invitation_code_expiry_days', '"30"', 'Default expiry period for invitation codes in days', unixepoch('now') * 1000, 'migration_005'),
('beta_access_period_days', '"90"', 'Beta access period in days after invitation usage', unixepoch('now') * 1000, 'migration_005'),
('max_invitation_codes_per_admin', '"100"', 'Maximum invitation codes an admin can generate at once', unixepoch('now') * 1000, 'migration_005');

-- Insert affiliation program tier configuration
INSERT OR IGNORE INTO system_config (key, value_json, description, updated_at, updated_by) VALUES 
('affiliation_tier_bronze_kickback', '"0.05"', 'Bronze tier kickback rate (5%)', unixepoch('now') * 1000, 'migration_005'),
('affiliation_tier_silver_kickback', '"0.10"', 'Silver tier kickback rate (10%)', unixepoch('now') * 1000, 'migration_005'),
('affiliation_tier_gold_kickback', '"0.15"', 'Gold tier kickback rate (15%)', unixepoch('now') * 1000, 'migration_005'),
('affiliation_tier_platinum_kickback', '"0.20"', 'Platinum tier kickback rate (20%)', unixepoch('now') * 1000, 'migration_005'),
('affiliation_min_followers_bronze', '"1000"', 'Minimum followers for bronze tier', unixepoch('now') * 1000, 'migration_005'),
('affiliation_min_followers_silver', '"5000"', 'Minimum followers for silver tier', unixepoch('now') * 1000, 'migration_005'),
('affiliation_min_followers_gold', '"25000"', 'Minimum followers for gold tier', unixepoch('now') * 1000, 'migration_005'),
('affiliation_min_followers_platinum', '"100000"', 'Minimum followers for platinum tier', unixepoch('now') * 1000, 'migration_005');

-- Record this migration
INSERT INTO schema_migrations (version, description) VALUES ('005', 'Invitation system tables and configuration'); 