use crate::services::core::infrastructure::DatabaseManager;
use crate::types::{
    ArbitrageOpportunity, ChatContext, EnhancedSessionState, EnhancedUserSession, SessionAnalytics,
    SessionConfig, SessionOutcome,
};
use crate::utils::{ArbitrageError, ArbitrageResult};

use serde_json::{self};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use worker::console_log;
use worker::wasm_bindgen::JsValue;

/// Comprehensive session management service for user lifecycle tracking
/// and push notification eligibility management
#[derive(Clone)]
pub struct SessionManagementService {
    d1_service: Arc<DatabaseManager>,
    kv_service: Arc<worker::kv::KvStore>,
    config: SessionConfig,
}

impl SessionManagementService {
    pub fn new(d1_service: DatabaseManager, kv_service: worker::kv::KvStore) -> Self {
        Self {
            d1_service: Arc::new(d1_service),
            kv_service: Arc::new(kv_service),
            config: SessionConfig::default(),
        }
    }

    pub fn with_config(mut self, config: SessionConfig) -> Self {
        self.config = config;
        self
    }

    /// Start a new session for a user
    pub async fn start_session(
        &self,
        telegram_id: i64,
        user_id: String,
    ) -> ArbitrageResult<EnhancedUserSession> {
        // Check if user already has an active session
        if let Ok(existing_session) = self.get_session_by_telegram_id(telegram_id).await {
            if existing_session.is_active() {
                // Extend existing session
                let mut updated_session = existing_session;
                updated_session.update_activity();
                self.update_session(&updated_session).await?;
                return Ok(updated_session);
            }
        }

        // Create new session
        let session = EnhancedUserSession::new(user_id, telegram_id);

        // Store in database
        console_log!(
            "SessionManagementService: Storing session in database for user {}",
            session.user_id
        );
        self.store_session(&session).await?;

        // Cache in KV for fast lookups
        console_log!(
            "SessionManagementService: Caching session in KV for user {}",
            session.user_id
        );
        self.cache_session(&session).await?;

        // Record session analytics
        console_log!(
            "SessionManagementService: Recording session start for user {}",
            session.user_id
        );
        self.record_session_start(&session).await?;

        Ok(session)
    }

    /// Validate if a user has an active session and return it
    pub async fn validate_session(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<EnhancedUserSession>> {
        console_log!(
            "SessionManagementService: Validating session for user {}",
            user_id
        );
        match self.get_session_by_user_id(user_id).await {
            Ok(session) => {
                if session.is_active() {
                    console_log!(
                        "SessionManagementService: Session active for user {}",
                        user_id
                    );
                    Ok(Some(session))
                } else {
                    console_log!(
                        "SessionManagementService: Session inactive for user {}",
                        user_id
                    );
                    Ok(None)
                }
            }
            Err(e) if e.error_code.as_deref() == Some("SESSION_NOT_FOUND") => {
                console_log!(
                    "SessionManagementService: Session not found for user {}",
                    user_id
                );
                Ok(None)
            } // Not found is not an error here
            Err(e) => {
                console_log!(
                    "SessionManagementService: Error getting session for user {}: {:?}",
                    user_id,
                    e
                );
                Err(e)
            } // Other errors are propagated
        }
    }

    /// Validate session by telegram ID (faster lookup) and return it
    pub async fn validate_session_by_telegram_id(
        &self,
        telegram_id: i64,
    ) -> ArbitrageResult<Option<EnhancedUserSession>> {
        console_log!(
            "SessionManagementService: Validating session by Telegram ID {}",
            telegram_id
        );
        match self.get_session_by_telegram_id(telegram_id).await {
            Ok(session) => {
                if session.is_active() {
                    console_log!(
                        "SessionManagementService: Session active for Telegram ID {}",
                        telegram_id
                    );
                    Ok(Some(session))
                } else {
                    console_log!(
                        "SessionManagementService: Session inactive for Telegram ID {}",
                        telegram_id
                    );
                    Ok(None)
                }
            }
            Err(e) if e.error_code.as_deref() == Some("SESSION_NOT_FOUND") => {
                console_log!(
                    "SessionManagementService: Session not found for Telegram ID {}",
                    telegram_id
                );
                Ok(None)
            } // Not found is not an error here
            Err(e) => {
                console_log!(
                    "SessionManagementService: Error getting session for Telegram ID {}: {:?}",
                    telegram_id,
                    e
                );
                Err(e)
            } // Other errors are propagated
        }
    }

    /// Update user activity and extend session
    pub async fn update_activity(&self, user_id: &str) -> ArbitrageResult<()> {
        console_log!(
            "SessionManagementService: Updating activity for user {}",
            user_id
        );
        let mut session = self.get_session_by_user_id(user_id).await?;
        session.update_activity();

        console_log!(
            "SessionManagementService: Updating session in DB for user {}",
            user_id
        );
        self.update_session(&session).await?;
        console_log!(
            "SessionManagementService: Caching updated session in KV for user {}",
            user_id
        );
        self.cache_session(&session).await?;

        Ok(())
    }

    /// Update activity by telegram ID (faster)
    pub async fn update_activity_by_telegram_id(&self, telegram_id: i64) -> ArbitrageResult<()> {
        console_log!(
            "SessionManagementService: Updating activity for Telegram ID {}",
            telegram_id
        );
        let mut session = self.get_session_by_telegram_id(telegram_id).await?;
        session.update_activity();

        console_log!(
            "SessionManagementService: Updating session in DB for Telegram ID {}",
            telegram_id
        );
        self.update_session(&session).await?;
        console_log!(
            "SessionManagementService: Caching updated session in KV for Telegram ID {}",
            telegram_id
        );
        self.cache_session(&session).await?;

        Ok(())
    }

    /// End a session manually
    pub async fn end_session(&self, user_id: &str) -> ArbitrageResult<()> {
        console_log!(
            "SessionManagementService: Ending session for user {}",
            user_id
        );
        let mut session = self.get_session_by_user_id(user_id).await?;
        session.terminate();

        console_log!(
            "SessionManagementService: Updating session in DB (terminated) for user {}",
            user_id
        );
        self.update_session(&session).await?;
        console_log!(
            "SessionManagementService: Invalidating session cache for user {}",
            user_id
        );
        self.invalidate_session_cache(session.telegram_id).await?;

        // Record session analytics
        console_log!(
            "SessionManagementService: Recording session end for user {}",
            user_id
        );
        self.record_session_end(&session, SessionOutcome::Terminated)
            .await?;

        Ok(())
    }

    /// Get session by session ID
    pub async fn get_session(&self, session_id: &str) -> ArbitrageResult<EnhancedUserSession> {
        console_log!(
            "SessionManagementService: Getting session by ID {}",
            session_id
        );
        let stmt = self.d1_service.prepare(
            "SELECT * FROM user_sessions WHERE session_id = ? AND session_state = 'active' ORDER BY created_at DESC LIMIT 1"
        );

        let result = stmt
            .bind(&[session_id.into()])
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to bind session_id: {}", e))
            })?
            .first::<serde_json::Value>(None)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to query session: {}", e))
            })?;

        match result {
            Some(row) => {
                console_log!(
                    "SessionManagementService: Session found by ID {}",
                    session_id
                );
                // Convert serde_json::Value to HashMap
                let row_map = if let serde_json::Value::Object(map) = row {
                    map.into_iter()
                        .collect::<std::collections::HashMap<String, serde_json::Value>>()
                } else {
                    console_log!(
                        "SessionManagementService: Invalid row format for session ID {}",
                        session_id
                    );
                    return Err(ArbitrageError::database_error("Invalid row format"));
                };
                let session = self.row_to_session(row_map)?;
                // Cache for future lookups
                console_log!(
                    "SessionManagementService: Caching session from DB for ID {}",
                    session_id
                );
                self.cache_session(&session).await?;
                Ok(session)
            }
            None => {
                console_log!(
                    "SessionManagementService: Session not found for ID {}",
                    session_id
                );
                Err(ArbitrageError::session_not_found(session_id.to_string()))
            }
        }
    }

    /// Get session by user ID
    pub async fn get_session_by_user_id(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<EnhancedUserSession> {
        console_log!(
            "SessionManagementService: Getting session by user ID {}",
            user_id
        );
        let stmt = self.d1_service.prepare(
            "SELECT * FROM user_sessions WHERE user_id = ? AND session_state = 'active' ORDER BY created_at DESC LIMIT 1"
        );

        let result = stmt
            .bind(&[user_id.into()])
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to bind parameters: {}", e))
            })?
            .first::<std::collections::HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to execute query: {}", e))
            })?;

        match result {
            Some(row) => {
                console_log!(
                    "SessionManagementService: Session found by user ID {}",
                    user_id
                );
                self.row_to_session(row)
            }
            None => {
                console_log!(
                    "SessionManagementService: Session not found for user ID {}",
                    user_id
                );
                Err(ArbitrageError::session_not_found(user_id))
            }
        }
    }

    /// Get session by telegram ID (with KV cache)
    pub async fn get_session_by_telegram_id(
        &self,
        telegram_id: i64,
    ) -> ArbitrageResult<EnhancedUserSession> {
        console_log!(
            "SessionManagementService: Getting session by Telegram ID {} (checking KV cache first)",
            telegram_id
        );
        // Try KV cache first
        let cache_key = format!("session_cache:{}", telegram_id);
        if let Some(cached_data) = self.kv_service.get(&cache_key).text().await? {
            if let Ok(session) = serde_json::from_str::<EnhancedUserSession>(&cached_data) {
                if session.is_active() {
                    console_log!(
                        "SessionManagementService: Session found in KV cache for Telegram ID {}",
                        telegram_id
                    );
                    return Ok(session);
                }
            }
        }
        console_log!("SessionManagementService: Session not found in KV cache for Telegram ID {}, falling back to DB", telegram_id);

        // Fallback to database
        let stmt = self.d1_service.prepare(
            "SELECT * FROM user_sessions WHERE telegram_id = ? AND session_state = 'active' ORDER BY created_at DESC LIMIT 1"
        );

        // Validate telegram_id is within JavaScript safe integer range
        Self::validate_telegram_id_for_js(telegram_id)?;

        let result = stmt
            .bind(&[JsValue::from_f64(telegram_id as f64)])
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to bind parameters: {}", e))
            })?
            .first::<std::collections::HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to execute query: {}", e))
            })?;

        match result {
            Some(row) => {
                console_log!(
                    "SessionManagementService: Session found in DB for Telegram ID {}",
                    telegram_id
                );
                let session = self.row_to_session(row)?;
                // Cache for future lookups
                console_log!(
                    "SessionManagementService: Caching session from DB for Telegram ID {}",
                    telegram_id
                );
                self.cache_session(&session).await?;
                Ok(session)
            }
            None => {
                console_log!(
                    "SessionManagementService: Session not found in DB for Telegram ID {}",
                    telegram_id
                );
                Err(ArbitrageError::session_not_found(telegram_id.to_string()))
            }
        }
    }

    /// Check if user is eligible for push notifications
    pub async fn is_eligible_for_push_notification(
        &self,
        user_id: &str,
        opportunity: &ArbitrageOpportunity,
        _chat_context: &ChatContext,
    ) -> ArbitrageResult<bool> {
        // Layer 1: Session validation
        let session = self.validate_session(user_id).await?;
        if session.is_none() {
            return Ok(false);
        }

        // Layer 2: Rate limiting - prevent spam
        if !self.check_notification_rate_limit(user_id).await? {
            return Ok(false);
        }

        // Layer 3: Basic notification permissions
        if !self
            .check_basic_notification_permissions(user_id, opportunity, _chat_context)
            .await?
        {
            return Ok(false);
        }

        // Layer 4: Notification preferences
        if !self
            .check_notification_preferences(user_id, opportunity, _chat_context)
            .await?
        {
            return Ok(false);
        }

        // Layer 5: Technical compatibility checks
        if !self
            .check_technical_compatibility(user_id, opportunity, _chat_context)
            .await?
        {
            return Ok(false);
        }

        // Layer 6: Context & compliance
        if !self
            .check_context_compliance(user_id, opportunity, _chat_context)
            .await?
        {
            return Ok(false);
        }

        Ok(true)
    }

    /// Check notification rate limiting to prevent spam
    async fn check_notification_rate_limit(&self, user_id: &str) -> ArbitrageResult<bool> {
        let now = chrono::Utc::now();
        let hour_key = format!(
            "notification_rate:{}:{}",
            user_id,
            now.format("%Y-%m-%d-%H")
        );
        let day_key = format!("notification_rate:{}:{}", user_id, now.format("%Y-%m-%d"));

        // Check hourly limit (max 5 notifications per hour)
        let hourly_count = match self.kv_service.get(&hour_key).text().await {
            Ok(Some(count_str)) => count_str.parse::<u32>().unwrap_or(0),
            _ => 0,
        };

        if hourly_count >= 5 {
            return Ok(false);
        }

        // Check daily limit (max 20 notifications per day)
        let daily_count = match self.kv_service.get(&day_key).text().await {
            Ok(Some(count_str)) => count_str.parse::<u32>().unwrap_or(0),
            _ => 0,
        };

        if daily_count >= 20 {
            return Ok(false);
        }

        // Update counters
        self.kv_service
            .put(&hour_key, (hourly_count + 1).to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to update hourly count: {}", e))
            })?
            .expiration_ttl(3600)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to execute hourly count update: {}",
                    e
                ))
            })?;

        self.kv_service
            .put(&day_key, (daily_count + 1).to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to update daily count: {}", e))
            })?
            .expiration_ttl(24 * 3600)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to execute daily count update: {}",
                    e
                ))
            })?;

        Ok(true)
    }

    /// Check basic notification permissions (placeholder for subscription integration)
    async fn check_basic_notification_permissions(
        &self,
        _user_id: &str,
        _opportunity: &ArbitrageOpportunity,
        _chat_context: &ChatContext,
    ) -> ArbitrageResult<bool> {
        // TODO: Integrate with UserProfileService to check subscription tier
        // For now, allow all users with valid sessions
        Ok(true)
    }

    /// Check user notification preferences (placeholder for preference service integration)
    async fn check_notification_preferences(
        &self,
        user_id: &str,
        opportunity: &ArbitrageOpportunity,
        _chat_context: &ChatContext,
    ) -> ArbitrageResult<bool> {
        // Check if user has disabled notifications for this opportunity type
        let pref_key = format!("notification_pref:{}:{:?}", user_id, opportunity.r#type);

        match self.kv_service.get(&pref_key).text().await {
            Ok(Some(pref_value)) => {
                // If preference exists, respect it (true = enabled, false = disabled)
                Ok(pref_value.parse::<bool>().unwrap_or(true))
            }
            _ => {
                // Default to enabled if no preference set
                Ok(true)
            }
        }
    }

    /// Check technical compatibility (placeholder for API validation integration)
    async fn check_technical_compatibility(
        &self,
        _user_id: &str,
        _opportunity: &ArbitrageOpportunity,
        _chat_context: &ChatContext,
    ) -> ArbitrageResult<bool> {
        // TODO: Integrate with UserProfileService to check:
        // - User has API keys for required exchanges
        // - API keys have necessary permissions
        // - User's trading setup is compatible
        // For now, assume basic compatibility
        Ok(true)
    }

    /// Check context compliance (groups, channels, private chats)
    async fn check_context_compliance(
        &self,
        _user_id: &str,
        _opportunity: &ArbitrageOpportunity,
        _chat_context: &ChatContext,
    ) -> ArbitrageResult<bool> {
        // Basic context validation - all contexts currently eligible
        // Future: Could implement context-specific rules
        // - Private chats: full notifications
        // - Groups: limited notifications
        // - Channels: broadcast only
        Ok(true)
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) -> ArbitrageResult<u32> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Find expired sessions
        let stmt = self.d1_service.prepare(
            "SELECT * FROM user_sessions WHERE session_state = 'active' AND expires_at < ?",
        );

        let result = stmt
            .bind(&[JsValue::from_f64(now as f64)])
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to bind parameters: {}", e))
            })?
            .all()
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to execute query: {}", e))
            })?;

        let results = result
            .results::<std::collections::HashMap<String, serde_json::Value>>()
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to parse results: {}", e))
            })?;

        let mut cleanup_count = 0;

        for row in results {
            if let Ok(mut session) = self.row_to_session(row) {
                session.current_state = EnhancedSessionState::Expired;
                session.update_activity();
                self.update_session(&session).await?;
                self.invalidate_session_cache(session.telegram_id).await?;

                // Record session analytics
                self.record_session_end(&session, SessionOutcome::Expired)
                    .await?;

                cleanup_count += 1;
            }
        }

        Ok(cleanup_count)
    }

    /// Get active session count
    pub async fn get_active_session_count(&self) -> ArbitrageResult<u32> {
        let stmt = self.d1_service.prepare(
            "SELECT COUNT(*) as count FROM user_sessions WHERE session_state = 'active' AND expires_at > datetime('now')"
        );

        let result = stmt
            .first::<std::collections::HashMap<String, serde_json::Value>>(None)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to get session count: {}", e))
            })?;

        match result {
            Some(row) => {
                let count = row.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                Ok(count)
            }
            None => Ok(0),
        }
    }

    /// Get all active sessions for admin monitoring
    pub async fn get_all_active_sessions(&self) -> ArbitrageResult<Vec<serde_json::Value>> {
        let query = r#"
            SELECT 
                user_id, telegram_id, session_state, created_at, 
                last_activity, expires_at, activity_count, context
            FROM user_sessions 
            WHERE session_state = 'active' 
            AND expires_at > datetime('now')
            ORDER BY last_activity DESC 
            LIMIT 500"#;

        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;
        let rows = result.results::<std::collections::HashMap<String, serde_json::Value>>()?;

        let sessions: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "user_id": row.get("user_id").and_then(|v| v.as_str()).unwrap_or(""),
                    "telegram_id": row.get("telegram_id").and_then(|v| v.as_str()).unwrap_or("0"),
                    "session_state": row.get("session_state").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    "created_at": row.get("created_at").and_then(|v| v.as_str()).unwrap_or(""),
                    "last_activity": row.get("last_activity").and_then(|v| v.as_str()).unwrap_or(""),
                    "expires_at": row.get("expires_at").and_then(|v| v.as_str()).unwrap_or(""),
                    "activity_count": row.get("activity_count").and_then(|v| v.as_str()).unwrap_or("0"),
                    "context": row.get("context").and_then(|v| v.as_str()).unwrap_or("{}")
                })
            })
            .collect();

        Ok(sessions)
    }

    /// Get session analytics for a user
    pub async fn get_session_analytics(
        &self,
        _user_id: &str,
    ) -> ArbitrageResult<Vec<SessionAnalytics>> {
        let mut analytics = Vec::new();

        // Get recent session analytics from KV store
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        // Get session count for the user for recent dates
        for date in [today, yesterday] {
            // Get session count for the date
            let count_key = format!("session_count:{}", date);
            if let Some(count_str) = self.kv_service.get(&count_key).text().await? {
                if let Ok(_count) = count_str.parse::<u32>() {
                    analytics.push(SessionAnalytics {
                        commands_executed: 0,
                        opportunities_viewed: 0,
                        trades_executed: 0,
                        session_duration_seconds: 0,
                        session_duration_ms: 0,
                        last_activity: chrono::Utc::now().timestamp() as u64,
                    });
                }
            }
        }

        // If no analytics found, return empty vector
        Ok(analytics)
    }

    // Private helper methods

    /// Validate telegram_id is within JavaScript safe integer range
    fn validate_telegram_id_for_js(telegram_id: i64) -> ArbitrageResult<()> {
        const JS_MAX_SAFE_INTEGER: i64 = 9007199254740991; // 2^53 - 1
        const JS_MIN_SAFE_INTEGER: i64 = -9007199254740991; // -(2^53 - 1)

        if !(JS_MIN_SAFE_INTEGER..=JS_MAX_SAFE_INTEGER).contains(&telegram_id) {
            return Err(ArbitrageError::validation_error(format!(
                "Telegram ID {} exceeds JavaScript safe integer range",
                telegram_id
            )));
        }
        Ok(())
    }

    /// Store session in database
    async fn store_session(&self, session: &EnhancedUserSession) -> ArbitrageResult<()> {
        console_log!(
            "SessionManagementService (store_session): Storing session {}",
            session.session_id
        );
        // Validate telegram_id before storing
        Self::validate_telegram_id_for_js(session.telegram_id)?;
        let stmt = self.d1_service.prepare(
            r#"
            INSERT OR REPLACE INTO user_sessions (
                session_id, user_id, telegram_id, session_state,
                started_at, last_activity_at, expires_at,
                onboarding_completed, preferences_set, metadata,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        );

        let metadata_json = serde_json::to_string(&session.metadata)?;

        stmt.bind(&[
            session.session_id.clone().into(),
            session.user_id.clone().into(),
            JsValue::from_f64(session.telegram_id as f64),
            session.session_state.to_db_string().into(),
            JsValue::from_f64(session.started_at as f64),
            JsValue::from_f64(session.last_activity_at as f64),
            JsValue::from_f64(session.expires_at as f64),
            session.onboarding_completed.into(),
            session.preferences_set.into(),
            metadata_json.into(),
            JsValue::from_f64(session.created_at as f64),
            JsValue::from_f64(session.updated_at as f64),
        ])
        .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Update existing session in database
    async fn update_session(&self, session: &EnhancedUserSession) -> ArbitrageResult<()> {
        console_log!(
            "SessionManagementService (update_session): Updating session {}",
            session.session_id
        );
        let stmt = self.d1_service.prepare(
            r#"
            UPDATE user_sessions SET
                session_state = ?, last_activity_at = ?, expires_at = ?,
                onboarding_completed = ?, preferences_set = ?, metadata = ?, updated_at = ?
            WHERE session_id = ?
        "#,
        );

        let metadata_json = serde_json::to_string(&session.metadata)?;

        stmt.bind(&[
            session.session_state.to_db_string().into(),
            JsValue::from_f64(session.last_activity_at as f64),
            JsValue::from_f64(session.expires_at as f64),
            session.onboarding_completed.into(),
            session.preferences_set.into(),
            metadata_json.into(),
            JsValue::from_f64(session.updated_at as f64),
            session.session_id.clone().into(),
        ])
        .map_err(|e| ArbitrageError::database_error(format!("Failed to bind parameters: {}", e)))?
        .run()
        .await
        .map_err(|e| ArbitrageError::database_error(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    /// Cache session in KV store for fast lookups
    async fn cache_session(&self, session: &EnhancedUserSession) -> ArbitrageResult<()> {
        console_log!(
            "SessionManagementService (cache_session): Caching session {}",
            session.session_id
        );
        let cache_key = format!("session_cache:{}", session.telegram_id);
        let session_json = serde_json::to_string(session).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize session: {}", e))
        })?;

        self.kv_service
            .put(&cache_key, &session_json)
            .map_err(|e| ArbitrageError::storage_error(format!("Failed to cache session: {}", e)))?
            .expiration_ttl(3600) // 1 hour cache
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to execute session cache: {}", e))
            })?;

        Ok(())
    }

    /// Invalidate session cache
    async fn invalidate_session_cache(&self, telegram_id: i64) -> ArbitrageResult<()> {
        console_log!("SessionManagementService (invalidate_session_cache): Invalidating cache for Telegram ID {}", telegram_id);
        let cache_key = format!("session_cache:{}", telegram_id);
        self.kv_service.delete(&cache_key).await.map_err(|e| {
            ArbitrageError::storage_error(format!("Failed to invalidate session cache: {}", e))
        })?;
        Ok(())
    }

    /// Convert database row to session object
    fn row_to_session(
        &self,
        row: std::collections::HashMap<String, serde_json::Value>,
    ) -> ArbitrageResult<EnhancedUserSession> {
        let session_id = row
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::database_error("Missing session_id"))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::database_error("Missing user_id"))?
            .to_string();

        let telegram_id = row
            .get("telegram_id")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::database_error("Missing telegram_id"))?
            as i64;

        let session_state_str = row
            .get("session_state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::database_error("Missing session_state"))?;

        let session_state = match session_state_str {
            "active" => EnhancedSessionState::Active,
            "expired" => EnhancedSessionState::Expired,
            "terminated" => EnhancedSessionState::Terminated,
            _ => EnhancedSessionState::Expired,
        };

        let started_at = row
            .get("started_at")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::database_error("Missing started_at"))?
            as u64;

        let last_activity_at = row
            .get("last_activity_at")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::database_error("Missing last_activity_at"))?
            as u64;

        let expires_at = row
            .get("expires_at")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::database_error("Missing expires_at"))?
            as u64;

        let onboarding_completed = row
            .get("onboarding_completed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let preferences_set = row
            .get("preferences_set")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let metadata = row
            .get("metadata")
            .and_then(|v| v.as_str())
            .and_then(|s| {
                if s == "null" {
                    Some(serde_json::Value::Null)
                } else {
                    serde_json::from_str(s).ok()
                }
            })
            .unwrap_or(serde_json::Value::Null);

        let created_at = row
            .get("created_at")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::database_error("Missing created_at"))?
            as u64;

        let updated_at = row
            .get("updated_at")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ArbitrageError::database_error("Missing updated_at"))?
            as u64;

        Ok(EnhancedUserSession {
            session_id,
            user_id,
            telegram_chat_id: telegram_id, // Assuming chat_id is same as telegram_id for now
            telegram_id,
            last_command: row
                .get("last_command")
                .and_then(|v| v.as_str())
                .map(String::from),
            current_state: session_state.clone(), // Assuming current_state is same as session_state from DB
            session_state,
            temporary_data: row
                .get("temporary_data")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            started_at,
            last_activity_at,
            expires_at,
            onboarding_completed,
            preferences_set,
            metadata,
            created_at,
            updated_at,
            session_analytics: row
                .get("session_analytics")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(SessionAnalytics {
                    commands_executed: 0,
                    opportunities_viewed: 0,
                    trades_executed: 0,
                    session_duration_seconds: 0,
                    session_duration_ms: 0,
                    last_activity: last_activity_at, // Use last_activity_at from row as a sensible default
                }),
            config: row
                .get("config")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
        })
    }

    /// Record session start analytics
    async fn record_session_start(&self, session: &EnhancedUserSession) -> ArbitrageResult<()> {
        // Record session start metrics in KV store for analytics
        let analytics_key = format!("session_analytics:start:{}", session.session_id);
        let analytics_data = serde_json::json!({
            "session_id": session.session_id,
            "user_id": session.user_id,
            "telegram_id": session.telegram_id,
            "started_at": session.started_at,
            "event_type": "session_start",
            "timestamp": chrono::Utc::now().timestamp() as u64
        });

        // Store analytics data with 30-day TTL
        self.kv_service
            .put(&analytics_key, analytics_data.to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create analytics put: {}", e))
            })?
            .expiration_ttl(30 * 24 * 3600)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to store analytics: {}", e))
            })?;

        // Update daily session count
        let date_key = format!("session_count:{}", chrono::Utc::now().format("%Y-%m-%d"));
        let current_count = match self.kv_service.get(&date_key).text().await {
            Ok(Some(count_str)) => count_str.parse::<u32>().unwrap_or(0),
            _ => 0,
        };

        self.kv_service
            .put(&date_key, (current_count + 1).to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create count put: {}", e))
            })?
            .expiration_ttl(24 * 3600)
            .execute()
            .await
            .map_err(|e| ArbitrageError::storage_error(format!("Failed to update count: {}", e)))?;

        Ok(())
    }

    /// Record session end analytics
    async fn record_session_end(
        &self,
        session: &EnhancedUserSession,
        outcome: SessionOutcome,
    ) -> ArbitrageResult<()> {
        // Record session end metrics in KV store for analytics
        let analytics_key = format!("session_analytics:end:{}", session.session_id);
        let session_duration = session.last_activity_at.saturating_sub(session.started_at);

        let analytics_data = serde_json::json!({
            "session_id": session.session_id,
            "user_id": session.user_id,
            "telegram_id": session.telegram_id,
            "started_at": session.started_at,
            "ended_at": session.last_activity_at,
            "duration_seconds": session_duration,
            "outcome": outcome.to_stable_string(),
            "onboarding_completed": session.onboarding_completed,
            "preferences_set": session.preferences_set,
            "event_type": "session_end",
            "timestamp": chrono::Utc::now().timestamp() as u64
        });

        // Store analytics data with 30-day TTL
        self.kv_service
            .put(&analytics_key, analytics_data.to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create analytics put: {}", e))
            })?
            .expiration_ttl(30 * 24 * 3600)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to store analytics: {}", e))
            })?;

        // Update outcome-specific counters
        let outcome_key = format!(
            "session_outcome:{}:{}",
            outcome.to_stable_string(),
            chrono::Utc::now().format("%Y-%m-%d")
        );
        let current_count = match self.kv_service.get(&outcome_key).text().await {
            Ok(Some(count_str)) => count_str.parse::<u32>().unwrap_or(0),
            _ => 0,
        };

        self.kv_service
            .put(&outcome_key, (current_count + 1).to_string())
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to create outcome put: {}", e))
            })?
            .expiration_ttl(24 * 3600)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to update outcome: {}", e))
            })?;

        Ok(())
    }
}

// Tests have been moved to packages/worker/tests/user/session_management_test.rs
