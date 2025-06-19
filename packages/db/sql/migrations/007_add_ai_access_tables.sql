-- Migration 007: Add AI Access Level Architecture Tables
-- This migration creates tables for AI usage tracking and AI template management

-- AI Usage Tracking Table
-- Tracks daily AI usage limits and cost monitoring per user
CREATE TABLE IF NOT EXISTS ai_usage_tracking (
    user_id TEXT NOT NULL,
    date TEXT NOT NULL,                          -- YYYY-MM-DD format
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

-- AI Templates Table
-- Stores AI analysis templates (system defaults and user-created)
CREATE TABLE IF NOT EXISTS ai_templates (
    template_id TEXT PRIMARY KEY,
    template_name TEXT NOT NULL,
    template_type TEXT NOT NULL,                 -- global_opportunity_analysis, personal_opportunity_generation, etc.
    access_level TEXT NOT NULL,                  -- none, default_only, full
    prompt_template TEXT NOT NULL,
    parameters TEXT NOT NULL DEFAULT '{}',       -- JSON object with AI parameters
    created_by TEXT,                             -- NULL for system templates, user_id for user templates
    is_system_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000),
    FOREIGN KEY (created_by) REFERENCES user_profiles(user_id) ON DELETE CASCADE
);

-- Indexes for AI Usage Tracking
CREATE INDEX IF NOT EXISTS idx_ai_usage_tracking_user_id ON ai_usage_tracking(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_usage_tracking_date ON ai_usage_tracking(date);
CREATE INDEX IF NOT EXISTS idx_ai_usage_tracking_user_date ON ai_usage_tracking(user_id, date);

-- Indexes for AI Templates
CREATE INDEX IF NOT EXISTS idx_ai_templates_template_type ON ai_templates(template_type);
CREATE INDEX IF NOT EXISTS idx_ai_templates_access_level ON ai_templates(access_level);
CREATE INDEX IF NOT EXISTS idx_ai_templates_created_by ON ai_templates(created_by);
CREATE INDEX IF NOT EXISTS idx_ai_templates_system_default ON ai_templates(is_system_default);
CREATE INDEX IF NOT EXISTS idx_ai_templates_type_access ON ai_templates(template_type, access_level);

-- Insert default AI templates
INSERT OR IGNORE INTO ai_templates (
    template_id, template_name, template_type, access_level, prompt_template, 
    parameters, created_by, is_system_default, created_at, updated_at
) VALUES 
(
    'global-opportunity-analysis-default',
    'Global Opportunity Analysis',
    'global_opportunity_analysis',
    'default_only',
    'Analyze this arbitrage opportunity and provide insights on market conditions, risk factors, and execution recommendations. Opportunity details: {opportunity_data}',
    '{"max_tokens": 1000, "temperature": 0.7, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}',
    NULL,
    TRUE,
    strftime('%s', 'now') * 1000,
    strftime('%s', 'now') * 1000
),
(
    'personal-opportunity-generation-default',
    'Personal Opportunity Generation',
    'personal_opportunity_generation',
    'default_only',
    'Based on the user''s trading preferences and market data, generate personalized trading opportunities. User preferences: {user_preferences}, Market data: {market_data}',
    '{"max_tokens": 1000, "temperature": 0.7, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}',
    NULL,
    TRUE,
    strftime('%s', 'now') * 1000,
    strftime('%s', 'now') * 1000
),
(
    'trading-decision-support-default',
    'Trading Decision Support',
    'trading_decision_support',
    'default_only',
    'Provide trading decision support for this opportunity. Consider risk management, position sizing, and market timing. Opportunity: {opportunity}, User profile: {user_profile}',
    '{"max_tokens": 1000, "temperature": 0.7, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}',
    NULL,
    TRUE,
    strftime('%s', 'now') * 1000,
    strftime('%s', 'now') * 1000
),
(
    'risk-assessment-default',
    'Risk Assessment',
    'risk_assessment',
    'default_only',
    'Assess the risk level of this trading opportunity. Consider market volatility, liquidity, correlation risks, and position concentration. Data: {risk_data}',
    '{"max_tokens": 1000, "temperature": 0.7, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}',
    NULL,
    TRUE,
    strftime('%s', 'now') * 1000,
    strftime('%s', 'now') * 1000
),
(
    'position-sizing-default',
    'Position Sizing',
    'position_sizing',
    'default_only',
    'Calculate optimal position size for this opportunity based on user''s risk tolerance and account balance. User data: {user_data}, Opportunity: {opportunity}',
    '{"max_tokens": 1000, "temperature": 0.7, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}',
    NULL,
    TRUE,
    strftime('%s', 'now') * 1000,
    strftime('%s', 'now') * 1000
);

-- Check constraints for data integrity
-- Ensure template_type is valid
CREATE TRIGGER IF NOT EXISTS validate_ai_template_type
BEFORE INSERT ON ai_templates
FOR EACH ROW
WHEN NEW.template_type NOT IN (
    'global_opportunity_analysis',
    'personal_opportunity_generation', 
    'trading_decision_support',
    'risk_assessment',
    'position_sizing'
)
BEGIN
    SELECT RAISE(ABORT, 'Invalid template_type. Must be one of: global_opportunity_analysis, personal_opportunity_generation, trading_decision_support, risk_assessment, position_sizing');
END;

-- Ensure access_level is valid
CREATE TRIGGER IF NOT EXISTS validate_ai_template_access_level
BEFORE INSERT ON ai_templates
FOR EACH ROW
WHEN NEW.access_level NOT IN ('none', 'default_only', 'full')
BEGIN
    SELECT RAISE(ABORT, 'Invalid access_level. Must be one of: none, default_only, full');
END;

-- Ensure ai_calls_used doesn't exceed ai_calls_limit
CREATE TRIGGER IF NOT EXISTS validate_ai_usage_limits
BEFORE INSERT ON ai_usage_tracking
FOR EACH ROW
WHEN NEW.ai_calls_used > NEW.ai_calls_limit
BEGIN
    SELECT RAISE(ABORT, 'ai_calls_used cannot exceed ai_calls_limit');
END;

-- Update trigger for ai_usage_tracking
CREATE TRIGGER IF NOT EXISTS validate_ai_usage_limits_update
BEFORE UPDATE ON ai_usage_tracking
FOR EACH ROW
WHEN NEW.ai_calls_used > NEW.ai_calls_limit
BEGIN
    SELECT RAISE(ABORT, 'ai_calls_used cannot exceed ai_calls_limit');
END;

-- Auto-update updated_at timestamp for ai_usage_tracking
CREATE TRIGGER IF NOT EXISTS update_ai_usage_tracking_timestamp
AFTER UPDATE ON ai_usage_tracking
FOR EACH ROW
BEGIN
    UPDATE ai_usage_tracking 
    SET updated_at = strftime('%s', 'now') * 1000 
    WHERE user_id = NEW.user_id AND date = NEW.date;
END;

-- Auto-update updated_at timestamp for ai_templates
CREATE TRIGGER IF NOT EXISTS update_ai_templates_timestamp
AFTER UPDATE ON ai_templates
FOR EACH ROW
BEGIN
    UPDATE ai_templates 
    SET updated_at = strftime('%s', 'now') * 1000 
    WHERE template_id = NEW.template_id;
END; 