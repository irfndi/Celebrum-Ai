//! Authentication Session Service
//! 
//! Session management integration for authentication workflows:
//! - Session creation and validation
//! - Integration with existing SessionManagementService
//! - User authentication session handling
//! - Session-based access control

use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::core::user::SessionManagementService;
use crate::types::UserProfile;
use crate::utils::{ArbitrageError, ArbitrageResult};
use worker::console_log;
use std::sync::Arc;

/// Authentication Session Service
/// 
/// Integrates with existing SessionManagementService for auth workflows
pub struct AuthSessionService {
    session_management_service: Arc<SessionManagementService>,
    service_container: Arc<ServiceContainer>,
}

impl AuthSessionService {
    /// Create new auth session service
    pub async fn new(service_container: &Arc<ServiceContainer>) -> ArbitrageResult<Self> {
        console_log!("🔐 Initializing Auth Session Service...");

        // Get session management service from container
        let session_management_service = service_container
            .get_session_management_service()
            .ok_or_else(|| ArbitrageError::service_unavailable("Session management service not available"))?;

        console_log!("✅ Auth Session Service initialized successfully");

        Ok(Self {
            session_management_service,
            service_container: service_container.clone(),
        })
    }

    /// Create or update session for user authentication
    pub async fn create_or_update_session(&self, telegram_id: i64, user_profile: &UserProfile) -> ArbitrageResult<super::SessionCreationResult> {
        console_log!("🔐 Creating/updating session for user {}", telegram_id);

        let user_id_str = telegram_id.to_string();

        // Check if session already exists
        let existing_session = self.session_management_service.validate_session(&user_id_str).await?;

        if existing_session {
            // Update existing session activity
            self.session_management_service.update_activity(&user_id_str).await?;
            console_log!("✅ Updated existing session for user {}", telegram_id);
            
            Ok(super::SessionCreationResult {
                session_id: user_id_str,
                is_new_session: false,
            })
        } else {
            // Create new session
            let new_session = self.session_management_service.start_session(telegram_id).await?;
            console_log!("✅ Created new session for user {}: {}", telegram_id, new_session.session_id);
            
            Ok(super::SessionCreationResult {
                session_id: new_session.session_id,
                is_new_session: true,
            })
        }
    }

    /// Validate session and return session info
    pub async fn validate_session(&self, session_id: &str) -> ArbitrageResult<super::SessionInfo> {
        console_log!("🔐 Validating session: {}", session_id);

        // Validate session using existing service
        let is_valid = self.session_management_service.validate_session(session_id).await?;

        if !is_valid {
            return Err(ArbitrageError::authentication_error("Invalid session"));
        }

        // Get session details (for now, create from session_id)
        // TODO: Enhance SessionManagementService to return full session details
        let session_info = super::SessionInfo {
            session_id: session_id.to_string(),
            user_id: session_id.to_string(), // Assuming session_id is user_id for now
            created_at: chrono::Utc::now(), // TODO: Get actual creation time
            expires_at: chrono::Utc::now() + chrono::Duration::days(7), // 7-day expiration
            last_activity: chrono::Utc::now(),
        };

        console_log!("✅ Session validated: {}", session_id);
        Ok(session_info)
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> ArbitrageResult<()> {
        console_log!("🔄 Updating session activity: {}", session_id);

        self.session_management_service.update_activity(session_id).await?;

        console_log!("✅ Session activity updated: {}", session_id);
        Ok(())
    }

    /// End session (logout)
    pub async fn end_session(&self, session_id: &str) -> ArbitrageResult<()> {
        console_log!("🚪 Ending session: {}", session_id);

        // TODO: Implement session termination in SessionManagementService
        // For now, we don't have a direct way to end sessions
        console_log!("⚠️ Session termination not implemented in SessionManagementService");

        Ok(())
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, user_id: &str) -> ArbitrageResult<SessionStats> {
        console_log!("📊 Getting session stats for user: {}", user_id);

        // Check if user has active session
        let has_active_session = self.session_management_service.validate_session(user_id).await?;

        // TODO: Get more detailed session statistics from SessionManagementService
        let stats = SessionStats {
            has_active_session,
            total_sessions: 1, // TODO: Get actual count
            last_login: chrono::Utc::now(), // TODO: Get actual last login
            session_duration: chrono::Duration::hours(1), // TODO: Get actual duration
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
    pub async fn validate_and_get_context(&self, session_id: &str) -> ArbitrageResult<SessionValidationResult> {
        console_log!("🔍 Validating session and getting context: {}", session_id);

        // Validate session
        let session_info = self.auth_session_service.validate_session(session_id).await?;

        // Update session activity
        self.auth_session_service.update_session_activity(session_id).await?;

        console_log!("✅ Session validation successful: {}", session_id);

        Ok(SessionValidationResult {
            is_valid: true,
            session_info: Some(session_info),
            user_id: session_id.to_string(), // TODO: Extract actual user_id
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
    pub async fn create_session(&self, telegram_id: i64, user_profile: &UserProfile) -> ArbitrageResult<super::SessionCreationResult> {
        console_log!("🆕 Creating session for user: {}", telegram_id);

        self.auth_session_service.create_or_update_session(telegram_id, user_profile).await
    }

    /// Terminate session
    pub async fn terminate_session(&self, session_id: &str) -> ArbitrageResult<()> {
        console_log!("🛑 Terminating session: {}", session_id);

        self.auth_session_service.end_session(session_id).await
    }

    /// Get active sessions for user
    pub async fn get_active_sessions(&self, user_id: &str) -> ArbitrageResult<Vec<super::SessionInfo>> {
        console_log!("📋 Getting active sessions for user: {}", user_id);

        // For now, return single session if active
        let has_session = self.auth_session_service.session_management_service.validate_session(user_id).await?;

        if has_session {
            let session_info = super::SessionInfo {
                session_id: user_id.to_string(),
                user_id: user_id.to_string(),
                created_at: chrono::Utc::now(), // TODO: Get actual creation time
                expires_at: chrono::Utc::now() + chrono::Duration::days(7),
                last_activity: chrono::Utc::now(),
            };
            Ok(vec![session_info])
        } else {
            Ok(vec![])
        }
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) -> ArbitrageResult<u32> {
        console_log!("🧹 Cleaning up expired sessions");

        // TODO: Implement session cleanup in SessionManagementService
        console_log!("⚠️ Session cleanup not implemented in SessionManagementService");

        Ok(0) // Return 0 for now
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