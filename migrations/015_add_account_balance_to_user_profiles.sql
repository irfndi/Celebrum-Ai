-- Migration: Add account_balance_usdt field to user_profiles table
-- This separates actual account balance from PnL tracking

ALTER TABLE user_profiles 
ADD COLUMN account_balance_usdt REAL DEFAULT 0.0;

-- Update existing records to have a default balance
UPDATE user_profiles 
SET account_balance_usdt = 0.0 
WHERE account_balance_usdt IS NULL;

-- Add index for balance queries
CREATE INDEX IF NOT EXISTS idx_user_profiles_account_balance 
ON user_profiles(account_balance_usdt); 