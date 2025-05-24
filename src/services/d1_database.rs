use worker::{Env, Result, D1Database};
use serde_json::{Value, json};
use crate::types::{UserProfile, UserInvitation, TradingAnalytics, InvitationCode, UserApiKey};
use crate::services::user_trading_preferences::UserTradingPreferences;
use crate::utils::{ArbitrageError, ArbitrageResult};
use std::collections::HashMap;
use uuid;
use chrono;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// D1Service provides database operations using Cloudflare D1 SQL database
/// This service handles persistent storage for user profiles, invitations, and analytics
pub struct D1Service {
    db: D1Database,
}

impl D1Service {
    /// Create a new D1Service instance
    pub fn new(env: &Env) -> Result<Self> {
        let db = env.d1("ArbEdgeDB")?;
        Ok(D1Service { db })
    }

    // ============= USER PROFILE OPERATIONS =============

    /// Create a new user profile in D1 database
    pub async fn create_user_profile(&self, profile: &UserProfile) -> ArbitrageResult<()> {
        self.store_user_profile(profile).await
    }

    /// Update a user profile in D1 database
    pub async fn update_user_profile(&self, profile: &UserProfile) -> ArbitrageResult<()> {
        self.store_user_profile(profile).await
    }

    /// Store a user profile in D1 database
    pub async fn store_user_profile(&self, profile: &UserProfile) -> ArbitrageResult<()> {
        // Serialize complex fields to JSON
        let api_keys_json = serde_json::to_string(&profile.api_keys)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize API keys: {}", e)))?;
        
        let subscription_json = serde_json::to_string(&profile.subscription)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize subscription: {}", e)))?;
        
        let configuration_json = serde_json::to_string(&profile.configuration)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize configuration: {}", e)))?;

        // Prepare INSERT OR REPLACE statement
        let stmt = self.db.prepare("
            INSERT OR REPLACE INTO user_profiles (
                user_id, telegram_user_id, telegram_username, api_keys, 
                subscription, configuration, invitation_code,
                created_at, updated_at, last_active, is_active, 
                total_trades, total_pnl_usdt
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        // Bind parameters and execute
        stmt.bind(&[
            profile.user_id.clone().into(),
            profile.telegram_user_id.into(),
            profile.telegram_username.clone().unwrap_or_default().into(),
            api_keys_json.into(),
            subscription_json.into(),
            configuration_json.into(),
            profile.invitation_code.clone().unwrap_or_default().into(),
            (profile.created_at as i64).into(),
            (profile.updated_at as i64).into(),
            (profile.last_active as i64).into(),
            profile.is_active.into(),
            (profile.total_trades as i64).into(),
            profile.total_pnl_usdt.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Retrieve a user profile by user ID
    pub async fn get_user_profile(&self, user_id: &str) -> ArbitrageResult<Option<UserProfile>> {
        let stmt = self.db.prepare("SELECT * FROM user_profiles WHERE user_id = ?");
        
        let result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        match result {
            Some(row) => {
                let profile = self.row_to_user_profile(row)?;
                Ok(Some(profile))
            }
            None => Ok(None)
        }
    }

    /// Retrieve a user profile by Telegram ID
    pub async fn get_user_by_telegram_id(&self, telegram_user_id: i64) -> ArbitrageResult<Option<UserProfile>> {
        let stmt = self.db.prepare("SELECT * FROM user_profiles WHERE telegram_user_id = ?");
        
        let result = stmt.bind(&[telegram_user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        match result {
            Some(row) => {
                let profile = self.row_to_user_profile(row)?;
                Ok(Some(profile))
            }
            None => Ok(None)
        }
    }

    /// List user profiles with pagination
    pub async fn list_user_profiles(&self, limit: Option<i32>, offset: Option<i32>) -> ArbitrageResult<Vec<UserProfile>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let stmt = self.db.prepare("
            SELECT * FROM user_profiles 
            ORDER BY created_at DESC 
            LIMIT ? OFFSET ?
        ");
        
        let result = stmt.bind(&[limit.into(), offset.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        let mut profiles = Vec::new();
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
        
        for row in results {
            profiles.push(self.row_to_user_profile(row)?);
        }

        Ok(profiles)
    }

    /// Delete a user profile
    pub async fn delete_user_profile(&self, user_id: &str) -> ArbitrageResult<bool> {
        let stmt = self.db.prepare("DELETE FROM user_profiles WHERE user_id = ?");
        
        let _result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        // For now, just return true if no error occurred
        // In a production system, we'd need to check if rows were actually affected
        Ok(true)
    }

    // ============= USER TRADING PREFERENCES OPERATIONS =============

    /// Store user trading preferences in D1 database
    pub async fn store_trading_preferences(&self, preferences: &UserTradingPreferences) -> ArbitrageResult<()> {
        let trading_focus_str = serde_json::to_string(&preferences.trading_focus)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize trading focus: {}", e)))?;
        
        let experience_level_str = serde_json::to_string(&preferences.experience_level)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize experience level: {}", e)))?;
        
        let risk_tolerance_str = serde_json::to_string(&preferences.risk_tolerance)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize risk tolerance: {}", e)))?;
        
        let automation_level_str = serde_json::to_string(&preferences.automation_level)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize automation level: {}", e)))?;
        
        let automation_scope_str = serde_json::to_string(&preferences.automation_scope)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize automation scope: {}", e)))?;
        
        let notification_channels_json = serde_json::to_string(&preferences.preferred_notification_channels)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize notification channels: {}", e)))?;
        
        let tutorial_steps_json = serde_json::to_string(&preferences.tutorial_steps_completed)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize tutorial steps: {}", e)))?;

        let stmt = self.db.prepare("
            INSERT OR REPLACE INTO user_trading_preferences (
                preference_id, user_id, trading_focus, experience_level, risk_tolerance,
                automation_level, automation_scope, arbitrage_enabled, technical_enabled,
                advanced_analytics_enabled, preferred_notification_channels,
                trading_hours_timezone, trading_hours_start, trading_hours_end,
                onboarding_completed, tutorial_steps_completed, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            preferences.preference_id.clone().into(),
            preferences.user_id.clone().into(),
            trading_focus_str.into(),
            experience_level_str.into(),
            risk_tolerance_str.into(),
            automation_level_str.into(),
            automation_scope_str.into(),
            preferences.arbitrage_enabled.into(),
            preferences.technical_enabled.into(),
            preferences.advanced_analytics_enabled.into(),
            notification_channels_json.into(),
            preferences.trading_hours_timezone.clone().into(),
            preferences.trading_hours_start.clone().into(),
            preferences.trading_hours_end.clone().into(),
            preferences.onboarding_completed.into(),
            tutorial_steps_json.into(),
            (preferences.created_at as i64).into(),
            (preferences.updated_at as i64).into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get user trading preferences by user ID
    pub async fn get_trading_preferences(&self, user_id: &str) -> ArbitrageResult<Option<UserTradingPreferences>> {
        let stmt = self.db.prepare("SELECT * FROM user_trading_preferences WHERE user_id = ?");
        
        let result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        match result {
            Some(row) => {
                let preferences = self.row_to_trading_preferences(row)?;
                Ok(Some(preferences))
            }
            None => Ok(None)
        }
    }

    /// Update user trading preferences
    pub async fn update_trading_preferences(&self, preferences: &UserTradingPreferences) -> ArbitrageResult<()> {
        self.store_trading_preferences(preferences).await
    }

    /// Delete user trading preferences
    pub async fn delete_trading_preferences(&self, user_id: &str) -> ArbitrageResult<bool> {
        let stmt = self.db.prepare("DELETE FROM user_trading_preferences WHERE user_id = ?");
        
        let _result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(true)
    }

    // ============= API KEY OPERATIONS =============

    /// Store a user API key 
    pub async fn store_user_api_key(&self, user_id: &str, api_key: &UserApiKey) -> ArbitrageResult<()> {
        let provider_json = serde_json::to_string(&api_key.provider)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize provider: {}", e)))?;
        
        let permissions_json = serde_json::to_string(&api_key.permissions)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize permissions: {}", e)))?;

        let stmt = self.db.prepare("
            INSERT OR REPLACE INTO user_api_keys (
                id, user_id, provider, encrypted_key, encrypted_secret,
                metadata, is_active, created_at, last_used, permissions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            api_key.id.clone().into(),
            user_id.into(),
            provider_json.into(),
            api_key.encrypted_key.clone().into(),
            api_key.encrypted_secret.clone().unwrap_or_default().into(),
            api_key.metadata.to_string().into(),
            api_key.is_active.into(),
            (api_key.created_at as i64).into(),
            api_key.last_used.map(|t| t as i64).unwrap_or(0).into(),
            permissions_json.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    // ============= INVITATION CODE OPERATIONS =============

    /// Create invitation code
    pub async fn create_invitation_code(&self, invitation: &InvitationCode) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT INTO invitation_codes (
                code, created_by, created_at, expires_at, max_uses,
                current_uses, is_active, purpose
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            invitation.code.clone().into(),
            invitation.created_by.clone().unwrap_or_default().into(),
            (invitation.created_at as i64).into(),
            invitation.expires_at.map(|t| t as i64).unwrap_or(0).into(),
            invitation.max_uses.map(|u| u as i64).unwrap_or(0).into(),
            (invitation.current_uses as i64).into(),
            invitation.is_active.into(),
            invitation.purpose.clone().into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get invitation code
    pub async fn get_invitation_code(&self, code: &str) -> ArbitrageResult<Option<InvitationCode>> {
        let stmt = self.db.prepare("SELECT * FROM invitation_codes WHERE code = ?");
        
        let result = stmt.bind(&[code.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        match result {
            Some(row) => {
                let invitation = self.row_to_invitation_code(row)?;
                Ok(Some(invitation))
            }
            None => Ok(None)
        }
    }

    /// Update invitation code
    pub async fn update_invitation_code(&self, invitation: &InvitationCode) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            UPDATE invitation_codes SET 
                current_uses = ?, is_active = ?
            WHERE code = ?
        ");

        stmt.bind(&[
            (invitation.current_uses as i64).into(),
            invitation.is_active.into(),
            invitation.code.clone().into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    // ============= USER INVITATION OPERATIONS =============

    /// Store a user invitation
    pub async fn store_user_invitation(&self, invitation: &UserInvitation) -> ArbitrageResult<()> {
        let invitation_data_json = serde_json::to_string(&invitation.invitation_data)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize invitation data: {}", e)))?;

        let stmt = self.db.prepare("
            INSERT OR REPLACE INTO user_invitations (
                invitation_id, inviter_user_id, invitee_identifier,
                invitation_type, status, message, invitation_data,
                created_at, expires_at, accepted_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        let expires_at = invitation.expires_at.map(|dt| dt.timestamp_millis()).unwrap_or(0);
        let accepted_at = invitation.accepted_at.map(|dt| dt.timestamp_millis()).unwrap_or(0);
        let created_at = invitation.created_at.timestamp_millis();

        stmt.bind(&[
            invitation.invitation_id.clone().into(),
            invitation.inviter_user_id.clone().into(),
            invitation.invitee_identifier.clone().into(),
            invitation.invitation_type.to_string().into(),
            invitation.status.to_string().into(),
            invitation.message.clone().unwrap_or_default().into(),
            invitation_data_json.into(),
            created_at.into(),
            expires_at.into(),
            accepted_at.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Retrieve user invitations by user ID
    pub async fn get_user_invitations(&self, user_id: &str) -> ArbitrageResult<Vec<UserInvitation>> {
        let stmt = self.db.prepare("
            SELECT * FROM user_invitations 
            WHERE inviter_user_id = ? 
            ORDER BY created_at DESC
        ");
        
        let result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        let mut invitations = Vec::new();
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
        
        for row in results {
            invitations.push(self.row_to_user_invitation(row)?);
        }

        Ok(invitations)
    }

    // ============= TRADING ANALYTICS OPERATIONS =============

    /// Store trading analytics data
    pub async fn store_trading_analytics(&self, analytics: &TradingAnalytics) -> ArbitrageResult<()> {
        let metric_data_json = serde_json::to_string(&analytics.metric_data)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize metric data: {}", e)))?;
        
        let analytics_metadata_json = serde_json::to_string(&analytics.analytics_metadata)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize analytics metadata: {}", e)))?;

        let stmt = self.db.prepare("
            INSERT INTO trading_analytics (
                analytics_id, user_id, metric_type, metric_value,
                metric_data, exchange_id, trading_pair, opportunity_type,
                timestamp, session_id, analytics_metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        let timestamp = analytics.timestamp.timestamp_millis();

        stmt.bind(&[
            analytics.analytics_id.clone().into(),
            analytics.user_id.clone().into(),
            analytics.metric_type.clone().into(),
            analytics.metric_value.into(),
            metric_data_json.into(),
            analytics.exchange_id.clone().unwrap_or_default().into(),
            analytics.trading_pair.clone().unwrap_or_default().into(),
            analytics.opportunity_type.clone().unwrap_or_default().into(),
            timestamp.into(),
            analytics.session_id.clone().unwrap_or_default().into(),
            analytics_metadata_json.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Retrieve trading analytics for a user
    pub async fn get_trading_analytics(&self, user_id: &str, limit: Option<i32>) -> ArbitrageResult<Vec<TradingAnalytics>> {
        let limit = limit.unwrap_or(100);
        
        let stmt = self.db.prepare("
            SELECT * FROM trading_analytics 
            WHERE user_id = ? 
            ORDER BY timestamp DESC 
            LIMIT ?
        ");
        
        let result = stmt.bind(&[user_id.into(), limit.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        let mut analytics = Vec::new();
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
        
        for row in results {
            analytics.push(self.row_to_trading_analytics(row)?);
        }

        Ok(analytics)
    }

    // ============= HELPER METHODS =============

    /// Convert database row to UserProfile
    fn row_to_user_profile(&self, row: HashMap<String, Value>) -> ArbitrageResult<UserProfile> {
        // Parse required fields
        let user_id = row.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing user_id".to_string()))?
            .to_string();

        let telegram_user_id = row.get("telegram_user_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ArbitrageError::parse_error("Missing telegram_user_id".to_string()))?;

        let telegram_username = row.get("telegram_username")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // Parse JSON fields
        let api_keys: Vec<UserApiKey> = row.get("api_keys")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let subscription = row.get("subscription")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let configuration = row.get("configuration")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let invitation_code = row.get("invitation_code")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // Parse timestamp fields
        let created_at = row.get("created_at")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u64;

        let updated_at = row.get("updated_at")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u64;

        let last_active = row.get("last_active")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u64;

        let is_active = row.get("is_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let total_trades = row.get("total_trades")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u32;

        let total_pnl_usdt = row.get("total_pnl_usdt")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        Ok(UserProfile {
            user_id,
            telegram_user_id,
            telegram_username,
            subscription,
            configuration,
            api_keys,
            invitation_code,
            created_at,
            updated_at,
            last_active,
            is_active,
            total_trades,
            total_pnl_usdt,
        })
    }

    /// Convert database row to InvitationCode
    fn row_to_invitation_code(&self, row: HashMap<String, Value>) -> ArbitrageResult<InvitationCode> {
        let code = row.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing code".to_string()))?
            .to_string();

        let created_by = row.get("created_by")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let created_at = row.get("created_at")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u64;

        let expires_at = row.get("expires_at")
            .and_then(|v| v.as_i64())
            .filter(|&t| t > 0)
            .map(|t| t as u64);

        let max_uses = row.get("max_uses")
            .and_then(|v| v.as_i64())
            .filter(|&u| u > 0)
            .map(|u| u as u32);

        let current_uses = row.get("current_uses")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u32;

        let is_active = row.get("is_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let purpose = row.get("purpose")
            .and_then(|v| v.as_str())
            .unwrap_or("general")
            .to_string();

        Ok(InvitationCode {
            code,
            created_by,
            created_at,
            expires_at,
            max_uses,
            current_uses,
            is_active,
            purpose,
        })
    }

    /// Convert database row to UserInvitation
    fn row_to_user_invitation(&self, row: HashMap<String, Value>) -> ArbitrageResult<UserInvitation> {
        // Parse required fields
        let invitation_id = row.get("invitation_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing invitation_id".to_string()))?
            .to_string();

        let inviter_user_id = row.get("inviter_user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing inviter_user_id".to_string()))?
            .to_string();

        let invitee_identifier = row.get("invitee_identifier")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing invitee_identifier".to_string()))?
            .to_string();

        let invitation_type_str = row.get("invitation_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing invitation_type".to_string()))?;

        let invitation_type = invitation_type_str.parse()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid invitation_type: {}", e)))?;

        let status_str = row.get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing status".to_string()))?;

        let status = status_str.parse()
            .map_err(|e| ArbitrageError::parse_error(format!("Invalid status: {}", e)))?;

        let message = row.get("message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let invitation_data: serde_json::Value = row.get("invitation_data")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(json!({}));

        // Parse timestamps
        let created_at = row.get("created_at")
            .and_then(|v| v.as_i64())
            .map(|ts| chrono::DateTime::from_timestamp_millis(ts))
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|| chrono::Utc::now());

        let expires_at = row.get("expires_at")
            .and_then(|v| v.as_i64())
            .filter(|&ts| ts > 0)
            .map(|ts| chrono::DateTime::from_timestamp_millis(ts))
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let accepted_at = row.get("accepted_at")
            .and_then(|v| v.as_i64())
            .filter(|&ts| ts > 0)
            .map(|ts| chrono::DateTime::from_timestamp_millis(ts))
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        Ok(UserInvitation {
            invitation_id,
            inviter_user_id,
            invitee_identifier,
            invitation_type,
            status,
            message,
            invitation_data,
            created_at,
            expires_at,
            accepted_at,
        })
    }

    /// Convert database row to TradingAnalytics
    fn row_to_trading_analytics(&self, row: HashMap<String, Value>) -> ArbitrageResult<TradingAnalytics> {
        // Parse required fields
        let analytics_id = row.get("analytics_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing analytics_id".to_string()))?
            .to_string();

        let user_id = row.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing user_id".to_string()))?
            .to_string();

        let metric_type = row.get("metric_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing metric_type".to_string()))?
            .to_string();

        let metric_value = row.get("metric_value")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::parse_error("Missing metric_value".to_string()))?;

        let metric_data: serde_json::Value = row.get("metric_data")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(json!({}));

        let analytics_metadata: serde_json::Value = row.get("analytics_metadata")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(json!({}));

        let exchange_id = row.get("exchange_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let trading_pair = row.get("trading_pair")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let opportunity_type = row.get("opportunity_type")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let session_id = row.get("session_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let timestamp = row.get("timestamp")
            .and_then(|v| v.as_i64())
            .map(|ts| chrono::DateTime::from_timestamp_millis(ts))
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|| chrono::Utc::now());

        Ok(TradingAnalytics {
            analytics_id,
            user_id,
            metric_type,
            metric_value,
            metric_data,
            exchange_id,
            trading_pair,
            opportunity_type,
            timestamp,
            session_id,
            analytics_metadata,
        })
    }

    /// Store AI analysis audit trail in D1 database
    pub async fn store_ai_analysis_audit(
        &self,
        user_id: &str,
        ai_provider: &str,
        request_data: &serde_json::Value,
        response_data: &serde_json::Value,
        processing_time_ms: u64,
    ) -> ArbitrageResult<()> {
        let audit_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp_millis();

        let stmt = self.db.prepare(
            "INSERT INTO ai_analysis_audit (
                audit_id, user_id, ai_provider, 
                request_data, response_data, processing_time_ms,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?)"
        );

        stmt.bind(&[
            audit_id.into(),
            user_id.into(),
            ai_provider.into(),
            serde_json::to_string(request_data)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize request_data: {}", e)))?
                .into(),
            serde_json::to_string(response_data)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize response_data: {}", e)))?
                .into(),
            (processing_time_ms as i64).into(),
            timestamp.into(),
        ])?.run().await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to store AI analysis audit: {}", e)))?;

        Ok(())
    }

    /// Store opportunity analysis in D1 database
    pub async fn store_opportunity_analysis(
        &self,
        analysis: &crate::services::ai_exchange_router::AiOpportunityAnalysis,
    ) -> ArbitrageResult<()> {
        let stmt = self.db.prepare(
            "INSERT INTO ai_opportunity_analysis (
                opportunity_id, user_id, ai_score, viability_assessment,
                risk_factors, recommended_position_size, confidence_level,
                analysis_timestamp, ai_provider_used, custom_recommendations
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        );

        stmt.bind(&[
            analysis.opportunity_id.as_str().into(),
            analysis.user_id.as_str().into(),
            analysis.ai_score.into(),
            analysis.viability_assessment.as_str().into(),
            serde_json::to_string(&analysis.risk_factors)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize risk_factors: {}", e)))?
                .into(),
            analysis.recommended_position_size.into(),
            analysis.confidence_level.into(),
            (analysis.analysis_timestamp as i64).into(),
            analysis.ai_provider_used.as_str().into(),
            serde_json::to_string(&analysis.custom_recommendations)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize custom_recommendations: {}", e)))?
                .into(),
        ])?.run().await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to store opportunity analysis: {}", e)))?;

        Ok(())
    }

    /// Health check for D1 database connection
    pub async fn health_check(&self) -> ArbitrageResult<bool> {
        let stmt = self.db.prepare("SELECT 1 as test");
        let result = stmt.first::<serde_json::Value>(None).await;
        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    // ============= GENERIC QUERY HELPERS =============
    
    #[cfg(target_arch = "wasm32")]
    /// ⚠️ **SECURITY WARNING**: Execute a prepared statement with parameters (for INSERT, UPDATE, DELETE)
    /// 
    /// **SQL INJECTION RISK**: This method accepts raw SQL strings. Only use with:
    /// - Hardcoded SQL statements 
    /// - Properly parameterized queries using the `params` array
    /// - Never pass user input directly in the `sql` parameter
    /// 
    /// Consider using specific typed methods instead of this generic query executor.
    pub async fn execute_query(&self, sql: &str, params: &[JsValue]) -> ArbitrageResult<()> {
        let stmt = self.db.prepare(sql);
        stmt.bind(params)
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;
        Ok(())
    }
    
    #[cfg(target_arch = "wasm32")]
    /// ⚠️ **SECURITY WARNING**: Execute a query that returns results (for SELECT)
    /// 
    /// **SQL INJECTION RISK**: This method accepts raw SQL strings. Only use with:
    /// - Hardcoded SQL statements
    /// - Properly parameterized queries using the `params` array  
    /// - Never pass user input directly in the `sql` parameter
    /// 
    /// Consider using specific typed methods instead of this generic query executor.
    pub async fn query(&self, sql: &str, params: &[JsValue]) -> ArbitrageResult<QueryResult> {
        let stmt = self.db.prepare(sql);
        let result = stmt.bind(params)
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;
            
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
            
        Ok(QueryResult { results })
    }
    
    #[cfg(target_arch = "wasm32")]
    /// ⚠️ **SECURITY WARNING**: Execute a query that returns a single result (for SELECT with LIMIT 1)
    /// 
    /// **SQL INJECTION RISK**: This method accepts raw SQL strings. Only use with:
    /// - Hardcoded SQL statements
    /// - Properly parameterized queries using the `params` array
    /// - Never pass user input directly in the `sql` parameter
    /// 
    /// Consider using specific typed methods instead of this generic query executor.
    pub async fn query_first(&self, sql: &str, params: &[JsValue]) -> ArbitrageResult<Option<HashMap<String, Value>>> {
        let stmt = self.db.prepare(sql);
        let result = stmt.bind(params)
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;
        Ok(result)
    }

    // ============= FUND MONITORING OPERATIONS =============

    /// Store balance history entry
    pub async fn store_balance_history(&self, entry: &crate::services::fund_monitoring::BalanceHistoryEntry) -> ArbitrageResult<()> {
        let balance_json = serde_json::to_string(&entry.balance)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize balance: {}", e)))?;

        let stmt = self.db.prepare("
            INSERT INTO balance_history (
                id, user_id, exchange_id, asset, balance_data, 
                usd_value, timestamp, snapshot_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            entry.id.clone().into(),
            entry.user_id.clone().into(),
            entry.exchange_id.clone().into(),
            entry.asset.clone().into(),
            balance_json.into(),
            entry.usd_value.into(),
            (entry.timestamp as i64).into(),
            entry.snapshot_id.clone().into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get balance history for a user with optional filters
    pub async fn get_balance_history(
        &self,
        user_id: &str,
        exchange_id: Option<&str>,
        asset: Option<&str>,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
        limit: Option<u32>,
    ) -> ArbitrageResult<Vec<crate::services::fund_monitoring::BalanceHistoryEntry>> {
        let mut query = "SELECT * FROM balance_history WHERE user_id = ?".to_string();
        let mut params: Vec<serde_json::Value> = vec![user_id.into()];

        if let Some(exchange) = exchange_id {
            query.push_str(" AND exchange_id = ?");
            params.push(exchange.into());
        }

        if let Some(asset_filter) = asset {
            query.push_str(" AND asset = ?");
            params.push(asset_filter.into());
        }

        if let Some(from_ts) = from_timestamp {
            query.push_str(" AND timestamp >= ?");
            params.push((from_ts as i64).into());
        }

        if let Some(to_ts) = to_timestamp {
            query.push_str(" AND timestamp <= ?");
            params.push((to_ts as i64).into());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit_val) = limit {
            query.push_str(" LIMIT ?");
            params.push((limit_val as i64).into());
        }

        let stmt = self.db.prepare(&query);
        
        let result = stmt.bind(&params.iter().map(|v| match v {
            serde_json::Value::String(s) => s.as_str().into(),
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0).into(),
            _ => "".into(),
        }).collect::<Vec<_>>())
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        let mut entries = Vec::new();
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
        
        for row in results {
            entries.push(self.row_to_balance_history(row)?);
        }

        Ok(entries)
    }

    /// Convert database row to BalanceHistoryEntry
    fn row_to_balance_history(&self, row: HashMap<String, Value>) -> ArbitrageResult<crate::services::fund_monitoring::BalanceHistoryEntry> {
        let balance_data = row.get("balance_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing balance_data field".to_string()))?;

        let balance: crate::types::Balance = serde_json::from_str(balance_data)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to parse balance data: {}", e)))?;

        Ok(crate::services::fund_monitoring::BalanceHistoryEntry {
            id: row.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            user_id: row.get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            exchange_id: row.get("exchange_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            asset: row.get("asset")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            balance,
            usd_value: row.get("usd_value")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            timestamp: row.get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as u64,
            snapshot_id: row.get("snapshot_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        })
    }

    // ============= DYNAMIC CONFIGURATION OPERATIONS =============

    /// Store a dynamic configuration template
    pub async fn store_config_template(&self, template: &crate::services::dynamic_config::DynamicConfigTemplate) -> ArbitrageResult<()> {
        let template_json = serde_json::to_string(template)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize template: {}", e)))?;

        let stmt = self.db.prepare("
            INSERT INTO dynamic_config_templates (
                template_id, name, description, version, category, 
                parameters, created_at, created_by, is_system_template, 
                subscription_tier_required
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            template.template_id.clone().into(),
            template.name.clone().into(),
            template.description.clone().into(),
            template.version.clone().into(),
            serde_json::to_string(&template.category)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize category: {}", e)))?.into(),
            template_json.into(),
            (template.created_at as i64).into(),
            template.created_by.clone().into(),
            template.is_system_template.into(),
            serde_json::to_string(&template.subscription_tier_required)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize subscription_tier_required: {}", e)))?.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get a dynamic configuration template by ID
    pub async fn get_config_template(&self, template_id: &str) -> ArbitrageResult<Option<HashMap<String, Value>>> {
        let stmt = self.db.prepare("SELECT parameters FROM dynamic_config_templates WHERE template_id = ?");
        
        let result = stmt.bind(&[template_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(result)
    }

    /// Store a dynamic configuration preset
    pub async fn store_config_preset(&self, preset: &crate::services::dynamic_config::ConfigPreset) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT INTO dynamic_config_presets (
                preset_id, name, description, template_id, parameter_values, 
                risk_level, target_audience, created_at, is_system_preset
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            preset.preset_id.clone().into(),
            preset.name.clone().into(),
            preset.description.clone().into(),
            preset.template_id.clone().into(),
            serde_json::to_string(&preset.parameter_values)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize parameter_values: {}", e)))?.into(),
            serde_json::to_string(&preset.risk_level)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize risk_level: {}", e)))?.into(),
            preset.target_audience.clone().into(),
            (preset.created_at as i64).into(),
            preset.is_system_preset.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Deactivate user configuration instances
    pub async fn deactivate_user_config(&self, user_id: &str, template_id: &str) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            UPDATE user_config_instances 
            SET is_active = false 
            WHERE user_id = ? AND template_id = ? AND is_active = true
        ");

        stmt.bind(&[user_id.into(), template_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Store a user configuration instance
    pub async fn store_user_config_instance(&self, instance: &crate::services::dynamic_config::UserConfigInstance) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT INTO user_config_instances (
                instance_id, user_id, template_id, preset_id, parameter_values, 
                version, is_active, created_at, updated_at, rollback_data
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            instance.instance_id.clone().into(),
            instance.user_id.clone().into(),
            instance.template_id.clone().into(),
            instance.preset_id.as_deref().unwrap_or("").into(),
            serde_json::to_string(&instance.parameter_values)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize parameter_values: {}", e)))?.into(),
            (instance.version as i64).into(),
            instance.is_active.into(),
            (instance.created_at as i64).into(),
            (instance.updated_at as i64).into(),
            instance.rollback_data.as_deref().unwrap_or("").into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get user's active configuration instance
    pub async fn get_user_config_instance(&self, user_id: &str, template_id: &str) -> ArbitrageResult<Option<HashMap<String, Value>>> {
        let stmt = self.db.prepare("
            SELECT * FROM user_config_instances 
            WHERE user_id = ? AND template_id = ? AND is_active = true 
            ORDER BY version DESC LIMIT 1
        ");
        
        let result = stmt.bind(&[user_id.into(), template_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(result)
    }

    // ============= NOTIFICATION SYSTEM OPERATIONS =============

    /// Store a notification template
    pub async fn store_notification_template(&self, template: &crate::services::notifications::NotificationTemplate) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT OR REPLACE INTO notification_templates (
                template_id, name, description, category, title_template, 
                message_template, priority, channels, variables, 
                is_system_template, is_active, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            template.template_id.clone().into(),
            template.name.clone().into(),
            template.description.clone().unwrap_or_default().into(),
            template.category.clone().into(),
            template.title_template.clone().into(),
            template.message_template.clone().into(),
            template.priority.clone().into(),
            serde_json::to_string(&template.channels)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize channels: {}", e)))?.into(),
            serde_json::to_string(&template.variables)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize variables: {}", e)))?.into(),
            template.is_system_template.into(),
            template.is_active.into(),
            (template.created_at as i64).into(),
            (template.updated_at as i64).into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get notification template by ID
    pub async fn get_notification_template(&self, template_id: &str) -> ArbitrageResult<Option<crate::services::notifications::NotificationTemplate>> {
        let stmt = self.db.prepare("SELECT * FROM notification_templates WHERE template_id = ? AND is_active = TRUE");
        
        let result = stmt.bind(&[template_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .first::<HashMap<String, Value>>(None)
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        if let Some(row) = result {
            Ok(Some(self.row_to_notification_template(row)?))
        } else {
            Ok(None)
        }
    }

    /// Store an alert trigger
    pub async fn store_alert_trigger(&self, trigger: &crate::services::notifications::AlertTrigger) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT OR REPLACE INTO alert_triggers (
                trigger_id, user_id, name, description, trigger_type, 
                conditions, template_id, is_active, priority, channels, 
                cooldown_minutes, max_alerts_per_hour, created_at, 
                updated_at, last_triggered_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            trigger.trigger_id.clone().into(),
            trigger.user_id.clone().into(),
            trigger.name.clone().into(),
            trigger.description.clone().unwrap_or_default().into(),
            trigger.trigger_type.clone().into(),
            serde_json::to_string(&trigger.conditions)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize conditions: {}", e)))?.into(),
            trigger.template_id.clone().unwrap_or_default().into(),
            trigger.is_active.into(),
            trigger.priority.clone().into(),
            serde_json::to_string(&trigger.channels)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize channels: {}", e)))?.into(),
            (trigger.cooldown_minutes as i64).into(),
            (trigger.max_alerts_per_hour as i64).into(),
            (trigger.created_at as i64).into(),
            (trigger.updated_at as i64).into(),
            trigger.last_triggered_at.map(|t| t as i64).unwrap_or(0).into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get user's alert triggers
    pub async fn get_user_alert_triggers(&self, user_id: &str) -> ArbitrageResult<Vec<crate::services::notifications::AlertTrigger>> {
        let stmt = self.db.prepare("
            SELECT * FROM alert_triggers 
            WHERE user_id = ? AND is_active = TRUE 
            ORDER BY priority DESC, created_at ASC
        ");
        
        let result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        let mut triggers = Vec::new();
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
        
        for row in results {
            triggers.push(self.row_to_alert_trigger(row)?);
        }

        Ok(triggers)
    }

    /// Store a notification
    pub async fn store_notification(&self, notification: &crate::services::notifications::Notification) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT INTO notifications (
                notification_id, user_id, trigger_id, template_id, title, 
                message, category, priority, notification_data, channels, 
                status, created_at, scheduled_at, sent_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            notification.notification_id.clone().into(),
            notification.user_id.clone().into(),
            notification.trigger_id.clone().unwrap_or_default().into(),
            notification.template_id.clone().unwrap_or_default().into(),
            notification.title.clone().into(),
            notification.message.clone().into(),
            notification.category.clone().into(),
            notification.priority.clone().into(),
            serde_json::to_string(&notification.notification_data)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize notification_data: {}", e)))?.into(),
            serde_json::to_string(&notification.channels)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize channels: {}", e)))?.into(),
            notification.status.clone().into(),
            (notification.created_at as i64).into(),
            notification.scheduled_at.map(|t| t as i64).unwrap_or(0).into(),
            notification.sent_at.map(|t| t as i64).unwrap_or(0).into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Update notification status
    pub async fn update_notification_status(&self, notification_id: &str, status: &str, sent_at: Option<u64>) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            UPDATE notifications 
            SET status = ?, sent_at = ? 
            WHERE notification_id = ?
        ");

        stmt.bind(&[
            status.into(),
            sent_at.map(|t| t as i64).unwrap_or(0).into(),
            notification_id.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Store notification delivery history
    pub async fn store_notification_history(&self, history: &crate::services::notifications::NotificationHistory) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            INSERT INTO notification_history (
                history_id, notification_id, user_id, channel, delivery_status, 
                response_data, error_message, delivery_time_ms, retry_count, 
                attempted_at, delivered_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ");

        stmt.bind(&[
            history.history_id.clone().into(),
            history.notification_id.clone().into(),
            history.user_id.clone().into(),
            history.channel.clone().into(),
            history.delivery_status.clone().into(),
            serde_json::to_string(&history.response_data)
                .map_err(|e| ArbitrageError::parse_error(format!("Failed to serialize response_data: {}", e)))?.into(),
            history.error_message.clone().unwrap_or_default().into(),
            history.delivery_time_ms.map(|t| t as i64).unwrap_or(0).into(),
            (history.retry_count as i64).into(),
            (history.attempted_at as i64).into(),
            history.delivered_at.map(|t| t as i64).unwrap_or(0).into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Get user's notification history
    pub async fn get_user_notification_history(&self, user_id: &str, limit: Option<i32>) -> ArbitrageResult<Vec<crate::services::notifications::NotificationHistory>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let query = format!("
            SELECT nh.*, n.title, n.category 
            FROM notification_history nh
            LEFT JOIN notifications n ON nh.notification_id = n.notification_id
            WHERE nh.user_id = ? 
            ORDER BY nh.attempted_at DESC{}
        ", limit_clause);

        let stmt = self.db.prepare(&query);
        
        let result = stmt.bind(&[user_id.into()])
            .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
            .all()
            .await
            .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        let mut history = Vec::new();
        let results = result.results::<HashMap<String, Value>>()
            .map_err(|e| ArbitrageError::database_error(format!("Failed to parse results: {}", e)))?;
        
        for row in results {
            history.push(self.row_to_notification_history(row)?);
        }

        Ok(history)
    }

    /// Update alert trigger last triggered time
    pub async fn update_trigger_last_triggered(&self, trigger_id: &str, timestamp: u64) -> ArbitrageResult<()> {
        let stmt = self.db.prepare("
            UPDATE alert_triggers 
            SET last_triggered_at = ? 
            WHERE trigger_id = ?
        ");

        stmt.bind(&[
            (timestamp as i64).into(),
            trigger_id.into(),
        ]).map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    // Helper methods for converting database rows to structs
    fn row_to_trading_preferences(&self, row: HashMap<String, Value>) -> ArbitrageResult<UserTradingPreferences> {
        let trading_focus: crate::services::user_trading_preferences::TradingFocus = serde_json::from_str(
            &row.get("trading_focus").unwrap().to_string().trim_matches('"')
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse trading focus: {}", e)))?;
        
        let experience_level: crate::services::user_trading_preferences::ExperienceLevel = serde_json::from_str(
            &row.get("experience_level").unwrap().to_string().trim_matches('"')
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse experience level: {}", e)))?;
        
        let risk_tolerance: crate::services::user_trading_preferences::RiskTolerance = serde_json::from_str(
            &row.get("risk_tolerance").unwrap().to_string().trim_matches('"')
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse risk tolerance: {}", e)))?;
        
        let automation_level: crate::services::user_trading_preferences::AutomationLevel = serde_json::from_str(
            &row.get("automation_level").unwrap().to_string().trim_matches('"')
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse automation level: {}", e)))?;
        
        let automation_scope: crate::services::user_trading_preferences::AutomationScope = serde_json::from_str(
            &row.get("automation_scope").unwrap().to_string().trim_matches('"')
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse automation scope: {}", e)))?;
        
        let preferred_notification_channels: Vec<String> = serde_json::from_str(
            &row.get("preferred_notification_channels").unwrap().to_string()
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse notification channels: {}", e)))?;
        
        let tutorial_steps_completed: Vec<String> = serde_json::from_str(
            &row.get("tutorial_steps_completed").unwrap().to_string()
        ).map_err(|e| ArbitrageError::parse_error(format!("Failed to parse tutorial steps: {}", e)))?;

        Ok(UserTradingPreferences {
            preference_id: row.get("preference_id").unwrap().to_string(),
            user_id: row.get("user_id").unwrap().to_string(),
            trading_focus,
            experience_level,
            risk_tolerance,
            automation_level,
            automation_scope,
            arbitrage_enabled: row.get("arbitrage_enabled").unwrap().to_string().parse().unwrap_or(true),
            technical_enabled: row.get("technical_enabled").unwrap().to_string().parse().unwrap_or(false),
            advanced_analytics_enabled: row.get("advanced_analytics_enabled").unwrap().to_string().parse().unwrap_or(false),
            preferred_notification_channels,
            trading_hours_timezone: row.get("trading_hours_timezone").unwrap().to_string(),
            trading_hours_start: row.get("trading_hours_start").unwrap().to_string(),
            trading_hours_end: row.get("trading_hours_end").unwrap().to_string(),
            onboarding_completed: row.get("onboarding_completed").unwrap().to_string().parse().unwrap_or(false),
            tutorial_steps_completed,
            created_at: row.get("created_at").unwrap().to_string().parse().unwrap_or(0) as u64,
            updated_at: row.get("updated_at").unwrap().to_string().parse().unwrap_or(0) as u64,
        })
    }

    fn row_to_notification_template(&self, row: HashMap<String, Value>) -> ArbitrageResult<crate::services::notifications::NotificationTemplate> {
        Ok(crate::services::notifications::NotificationTemplate {
            template_id: row.get("template_id").unwrap().to_string(),
            name: row.get("name").unwrap().to_string(),
            description: if row.get("description").unwrap().to_string().is_empty() { None } else { Some(row.get("description").unwrap().to_string()) },
            category: row.get("category").unwrap().to_string(),
            title_template: row.get("title_template").unwrap().to_string(),
            message_template: row.get("message_template").unwrap().to_string(),
            priority: row.get("priority").unwrap().to_string(),
            channels: serde_json::from_str(&row.get("channels").unwrap().to_string())?,
            variables: serde_json::from_str(&row.get("variables").unwrap().to_string())?,
            is_system_template: row.get("is_system_template").unwrap().to_string().parse().unwrap_or(false),
            is_active: row.get("is_active").unwrap().to_string().parse().unwrap_or(true),
            created_at: row.get("created_at").unwrap().to_string().parse().unwrap_or(0) as u64,
            updated_at: row.get("updated_at").unwrap().to_string().parse().unwrap_or(0) as u64,
        })
    }

    fn row_to_alert_trigger(&self, row: HashMap<String, Value>) -> ArbitrageResult<crate::services::notifications::AlertTrigger> {
        Ok(crate::services::notifications::AlertTrigger {
            trigger_id: row.get("trigger_id").unwrap().to_string(),
            user_id: row.get("user_id").unwrap().to_string(),
            name: row.get("name").unwrap().to_string(),
            description: if row.get("description").unwrap().to_string().is_empty() { None } else { Some(row.get("description").unwrap().to_string()) },
            trigger_type: row.get("trigger_type").unwrap().to_string(),
            conditions: serde_json::from_str(&row.get("conditions").unwrap().to_string())?,
            template_id: if row.get("template_id").unwrap().to_string().is_empty() { None } else { Some(row.get("template_id").unwrap().to_string()) },
            is_active: row.get("is_active").unwrap().to_string().parse().unwrap_or(true),
            priority: row.get("priority").unwrap().to_string(),
            channels: serde_json::from_str(&row.get("channels").unwrap().to_string())?,
            cooldown_minutes: row.get("cooldown_minutes").unwrap().to_string().parse().unwrap_or(5) as u32,
            max_alerts_per_hour: row.get("max_alerts_per_hour").unwrap().to_string().parse().unwrap_or(10) as u32,
            created_at: row.get("created_at").unwrap().to_string().parse().unwrap_or(0) as u64,
            updated_at: row.get("updated_at").unwrap().to_string().parse().unwrap_or(0) as u64,
            last_triggered_at: {
                let val = row.get("last_triggered_at").unwrap().to_string().parse().unwrap_or(0i64);
                if val > 0 { Some(val as u64) } else { None }
            },
        })
    }

    fn row_to_notification_history(&self, row: HashMap<String, Value>) -> ArbitrageResult<crate::services::notifications::NotificationHistory> {
        Ok(crate::services::notifications::NotificationHistory {
            history_id: row.get("history_id").unwrap().to_string(),
            notification_id: row.get("notification_id").unwrap().to_string(),
            user_id: row.get("user_id").unwrap().to_string(),
            channel: row.get("channel").unwrap().to_string(),
            delivery_status: row.get("delivery_status").unwrap().to_string(),
            response_data: serde_json::from_str(&row.get("response_data").unwrap().to_string())?,
            error_message: if row.get("error_message").unwrap().to_string().is_empty() { None } else { Some(row.get("error_message").unwrap().to_string()) },
            delivery_time_ms: {
                let val = row.get("delivery_time_ms").unwrap().to_string().parse().unwrap_or(0i64);
                if val > 0 { Some(val as u64) } else { None }
            },
            retry_count: row.get("retry_count").unwrap().to_string().parse().unwrap_or(0) as u32,
            attempted_at: row.get("attempted_at").unwrap().to_string().parse().unwrap_or(0) as u64,
            delivered_at: {
                let val = row.get("delivered_at").unwrap().to_string().parse().unwrap_or(0i64);
                if val > 0 { Some(val as u64) } else { None }
            },
        })
    }
}

/// Query result wrapper for compatibility with dynamic config service
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub results: Vec<HashMap<String, Value>>,
} 