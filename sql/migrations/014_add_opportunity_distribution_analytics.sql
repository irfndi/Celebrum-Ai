-- Migration 014: Add Opportunity Distribution Analytics Table
-- This migration adds the table for tracking opportunity distribution analytics
-- Created: 2025-01-28

-- Create opportunity distribution analytics table
CREATE TABLE IF NOT EXISTS opportunity_distribution_analytics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opportunity_id TEXT NOT NULL,
    pair TEXT NOT NULL,
    rate_difference REAL NOT NULL,
    priority_score REAL NOT NULL,
    distributed_count INTEGER NOT NULL DEFAULT 0,
    distribution_strategy TEXT NOT NULL,
    detection_timestamp INTEGER NOT NULL,
    distribution_timestamp INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for performance
    INDEX idx_opportunity_distribution_analytics_opportunity_id (opportunity_id),
    INDEX idx_opportunity_distribution_analytics_pair (pair),
    INDEX idx_opportunity_distribution_analytics_distribution_timestamp (distribution_timestamp),
    INDEX idx_opportunity_distribution_analytics_detection_timestamp (detection_timestamp)
);

-- Create index for analytics queries
CREATE INDEX IF NOT EXISTS idx_opportunity_distribution_analytics_composite 
ON opportunity_distribution_analytics (distribution_timestamp, pair, distributed_count);

-- Create index for performance analytics
CREATE INDEX IF NOT EXISTS idx_opportunity_distribution_analytics_performance 
ON opportunity_distribution_analytics (detection_timestamp, distribution_timestamp);

-- Add comments for documentation
PRAGMA table_info(opportunity_distribution_analytics);

-- Verify table creation
SELECT name FROM sqlite_master WHERE type='table' AND name='opportunity_distribution_analytics'; 