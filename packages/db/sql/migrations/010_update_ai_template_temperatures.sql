-- Migration 010: Update AI Template Temperatures for Optimal Performance
-- This migration updates the temperature parameters for AI templates based on their specific use cases

-- Update Global Opportunity Analysis - Keep at 0.7 for balanced analysis
UPDATE ai_templates 
SET parameters = '{"max_tokens": 1000, "temperature": 0.7, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}'
WHERE template_id = 'global-opportunity-analysis-default';

-- Update Personal Opportunity Generation - Increase to 0.8 for more creative opportunity discovery
UPDATE ai_templates 
SET parameters = '{"max_tokens": 1000, "temperature": 0.8, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}'
WHERE template_id = 'personal-opportunity-generation-default';

-- Update Trading Decision Support - Reduce to 0.6 for more consistent decision recommendations
UPDATE ai_templates 
SET parameters = '{"max_tokens": 1000, "temperature": 0.6, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}'
WHERE template_id = 'trading-decision-support-default';

-- Update Risk Assessment - Reduce to 0.4 for more deterministic risk analysis
UPDATE ai_templates 
SET parameters = '{"max_tokens": 1000, "temperature": 0.4, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}'
WHERE template_id = 'risk-assessment-default';

-- Update Position Sizing - Reduce to 0.3 for highly consistent position calculations
UPDATE ai_templates 
SET parameters = '{"max_tokens": 1000, "temperature": 0.3, "top_p": 1.0, "frequency_penalty": 0.0, "presence_penalty": 0.0, "custom_parameters": {}}'
WHERE template_id = 'position-sizing-default';

-- Update the updated_at timestamp for all modified templates
UPDATE ai_templates 
SET updated_at = strftime('%s', 'now') * 1000 
WHERE template_id IN (
    'global-opportunity-analysis-default',
    'personal-opportunity-generation-default', 
    'trading-decision-support-default',
    'risk-assessment-default',
    'position-sizing-default'
); 