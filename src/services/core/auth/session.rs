//! Authentication Session Service
//!
//! Session management integration for authentication workflows:
//! - Session creation and validation
//! - Integration with existing SessionManagementService
//! - User authentication session handling
//! - Session-based access control

use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::core::user::SessionManagementService;
use crate::types::EnhancedUserSession;
use crate::types::UserProfile;
use crate::utils::{ArbitrageError, ArbitrageResult};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use worker::console_log;

/// Authentication Session Service
///
/// Integrates with existing SessionManagementService for auth workflows
pub struct AuthSessionService {
    session_management_service: Arc<SessionManagementService>,
    #[allow(dead_code)] // Will be used for service discovery and DI
    service_container: Arc<ServiceContainer>,
}

impl AuthSessionService {
    /// Create new auth session service
    pub async fn new(service_container: &Arc<ServiceContainer>) -> ArbitrageResult<Self> {
        console_log!("🔐 Initializing Auth Session Service...");

        // Get session management service from container
        let session_management_service = service_container.session_service().clone();

        console_log!("✅ Auth Session Service initialized successfully");

        Ok(Self {
            session_management_service,
            service_container: service_container.clone(),
        })
    }

    /// Create or update session for user authentication
    pub async fn create_or_update_session(
        &self,
        telegram_id: i64,
        user_profile: &UserProfile,
    ) -> ArbitrageResult<super::SessionCreationResult> {
        console_log!("🔐 Creating/updating session for user {}", telegram_id);

        let user_id_str = telegram_id.to_string(); // This is used as user_id for validate_session

        // Check if session already exists
        let existing_session_option = self
            .session_management_service
            .validate_session(&user_id_str) // user_id_str is the user_id here
            .await?;

        if let Some(existing_session_details) = existing_session_option {
            // Update existing session activity using the user_id
            self.session_management_service
                .update_activity(&existing_session_details.user_id)
                .await?;
            console_log!("✅ Updated existing session for user {}", telegram_id);

            Ok(super::SessionCreationResult {
                session_id: existing_session_details.session_id, // Use actual session_id
                is_new_session: false,
            })
        } else {
            // Create new session, start_session takes telegram_id and user_id
            let new_session = self
                .session_management_service
                .start_session(telegram_id, user_profile.user_id.clone())
                .await?;
            console_log!(
                "✅ Created new session for user {}: {}",
                telegram_id,
                new_session.session_id
            );

            Ok(super::SessionCreationResult {
                session_id: new_session.session_id,
                is_new_session: true,
            })
        }
    }

    /// Validate session and return session info
    pub async fn validate_session(&self, session_id: &str) -> ArbitrageResult<super::SessionInfo> {
        console_log!("🔐 Validating session: {}", session_id);

        // Validate session using existing service.
        // Note: session_id is used as user_id for SessionManagementService.validate_session
        let session_details_option = self
            .session_management_service
            .validate_session(session_id)
            .await?;

        if let Some(session_details) = session_details_option {
            let (created_at, expires_at, last_activity) =
                convert_session_timestamps(&session_details)?;

            let session_info = super::SessionInfo {
                session_id: session_details.session_id, // Use the actual session_id from details
                user_id: session_details.user_id,       // Use the actual user_id from details
                created_at,
                expires_at,
                last_activity,
            };
            console_log!("✅ Session validated: {}", session_info.session_id);
            Ok(session_info)
        } else {
            Err(ArbitrageError::authentication_error(
                "Invalid or expired session",
            ))
        }
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> ArbitrageResult<()> {
        console_log!("🔄 Updating session activity: {}", session_id);

        self.session_management_service
            .update_activity(session_id)
            .await?;

        console_log!("✅ Session activity updated: {}", session_id);
        Ok(())
    }

    /// End session (logout)
    pub async fn end_session(&self, session_id: &str) -> ArbitrageResult<()> {
        console_log!("🚪 Ending session: {}", session_id);

        self.session_management_service
            .end_session(session_id)
            .await?;
        console_log!("✅ Session ended: {}", session_id);
        Ok(())
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, user_id: &str) -> ArbitrageResult<SessionStats> {
        console_log!("📊 Getting session stats for user: {}", user_id);

        // Check if user has active session
        let active_session_option = self
            .session_management_service
            .validate_session(user_id)
            .await?;
        let has_active_session = active_session_option.is_some();

        // Get detailed session statistics from SessionManagementService
        let stats = if let Some(session) = &active_session_option {
            let session_duration_ms = session.last_activity_at - session.started_at;
            let session_duration = chrono::Duration::milliseconds(session_duration_ms as i64);

            SessionStats {
                has_active_session,
                total_sessions: 1, // Could be enhanced to count all user sessions
                last_login: chrono::DateTime::from_timestamp_millis(session.started_at as i64)
                    .unwrap_or_else(chrono::Utc::now),
                session_duration,
            }
        } else {
            SessionStats {
                has_active_session: false,
                total_sessions: 0,
                last_login: chrono::Utc::now(),
                session_duration: chrono::Duration::zero(),
            }
        };

        console_log!("✅ Session stats retrieved for user: {}", user_id);
        Ok(stats)
    }
}

/// Session Validator for validating sessions
pub struct SessionValidator {
    auth_session_service: AuthSessionService,
}

impl SessionValidator {
    /// Create new session validator
    pub async fn new(service_container: &Arc<ServiceContainer>) -> ArbitrageResult<Self> {
        let auth_session_service = AuthSessionService::new(service_container).await?;

        Ok(Self {
            auth_session_service,
        })
    }

    /// Validate session and return user context
    pub async fn validate_and_get_context(
        &self,
        session_id: &str,
    ) -> ArbitrageResult<SessionValidationResult> {
        console_log!("🔍 Validating session and getting context: {}", session_id);

        // Validate session
        let session_info = self
            .auth_session_service
            .validate_session(session_id)
            .await?;

        // Update session activity
        self.auth_session_service
            .update_session_activity(session_id)
            .await?;

        console_log!("✅ Session validation successful: {}", session_id);

        Ok(SessionValidationResult {
            is_valid: true,
            session_info: Some(session_info.clone()),
            user_id: session_info.user_id,
        })
    }

    /// Quick session validation (no activity update)
    pub async fn quick_validate(&self, session_id: &str) -> ArbitrageResult<bool> {
        console_log!("⚡ Quick session validation: {}", session_id);

        match self.auth_session_service.validate_session(session_id).await {
            Ok(_) => {
                console_log!("✅ Quick validation successful: {}", session_id);
                Ok(true)
            }
            Err(_) => {
                console_log!("❌ Quick validation failed: {}", session_id);
                Ok(false)
            }
        }
    }
}

/// Session Manager for managing multiple sessions
pub struct SessionManager {
    auth_session_service: AuthSessionService,
}

impl SessionManager {
    /// Create new session manager
    pub async fn new(service_container: &Arc<ServiceContainer>) -> ArbitrageResult<Self> {
        let auth_session_service = AuthSessionService::new(service_container).await?;

        Ok(Self {
            auth_session_service,
        })
    }

    /// Create session for user
    pub async fn create_session(
        &self,
        telegram_id: i64,
        user_profile: &UserProfile,
    ) -> ArbitrageResult<super::SessionCreationResult> {
        console_log!("🆕 Creating session for user: {}", telegram_id);

        self.auth_session_service
            .create_or_update_session(telegram_id, user_profile)
            .await
    }

    /// Terminate session
    pub async fn terminate_session(&self, session_id: &str) -> ArbitrageResult<()> {
        console_log!("🛑 Terminating session: {}", session_id);

        self.auth_session_service.end_session(session_id).await
    }

    /// Get active sessions for user
    pub async fn get_active_sessions(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Vec<super::SessionInfo>> {
        console_log!("📋 Getting active sessions for user: {}", user_id);

        // user_id here is the user_id for SessionManagementService.validate_session
        let active_session_option = self
            .auth_session_service
            .session_management_service
            .validate_session(user_id) // user_id is the user_id
            .await?;

        if let Some(session_details) = active_session_option {
            let (created_at, expires_at, last_activity) =
                convert_session_timestamps(&session_details)?;

            let session_info = super::SessionInfo {
                session_id: session_details.session_id,
                user_id: session_details.user_id,
                created_at,
                expires_at,
                last_activity,
            };
            Ok(vec![session_info])
        } else {
            Ok(vec![])
        }
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) -> ArbitrageResult<u32> {
        console_log!("🧹 Cleaning up expired sessions");

        let count = self
            .auth_session_service
            .session_management_service
            .cleanup_expired_sessions()
            .await?;
        console_log!("✅ Expired sessions cleanup complete. Count: {}", count);
        Ok(count)
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub has_active_session: bool,
    pub total_sessions: u32,
    pub last_login: chrono::DateTime<chrono::Utc>,
    pub session_duration: chrono::Duration,
}

/// Session validation result
#[derive(Debug, Clone)]
pub struct SessionValidationResult {
    pub is_valid: bool,
    pub session_info: Option<super::SessionInfo>,
    pub user_id: String,
}

// Helper function to convert u64 timestamps from EnhancedUserSession to DateTime<Utc>
fn convert_session_timestamps(
    session: &EnhancedUserSession,
) -> ArbitrageResult<(DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)> {
    let created_at = DateTime::<Utc>::from_timestamp_millis(session.created_at as i64)
        .ok_or_else(|| ArbitrageError::internal_error("Invalid created_at timestamp"))?;
    let expires_at = DateTime::<Utc>::from_timestamp_millis(session.expires_at as i64)
        .ok_or_else(|| ArbitrageError::internal_error("Invalid expires_at timestamp"))?;
    let last_activity = DateTime::<Utc>::from_timestamp_millis(session.last_activity_at as i64)
        .ok_or_else(|| ArbitrageError::internal_error("Invalid last_activity timestamp"))?;
    Ok((created_at, expires_at, last_activity))
}

// Tests have been moved to packages/worker/tests/auth/session_test.rs
