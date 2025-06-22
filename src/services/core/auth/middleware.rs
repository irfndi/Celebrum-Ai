//! Authentication Middleware
//!
//! Middleware for request authentication and authorization:
//! - Request authentication validation
//! - User context extraction
//! - Permission checking for endpoints
//! - Session validation

use crate::services::core::auth::permissions::PermissionChecker;
use crate::services::core::auth::{SessionInfo, SessionValidator, UserContext};
use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::types::{
    RiskProfile, Subscription, SubscriptionTier, UserAccessLevel, UserConfiguration,
    UserPreferences, UserProfile,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
use chrono;
use md5;
use std::sync::Arc;
use worker::{console_log, Request};

/// Authentication Middleware
pub struct AuthMiddleware {
    service_container: Arc<ServiceContainer>,
}

impl AuthMiddleware {
    /// Create new authentication middleware
    pub async fn new(service_container: Arc<ServiceContainer>) -> ArbitrageResult<Self> {
        console_log!("🔐 Initializing Authentication Middleware...");

        console_log!("✅ Authentication Middleware initialized successfully");

        Ok(Self { service_container })
    }

    /// Authenticate request and extract user context
    pub async fn authenticate_request(
        &self,
        req: &Request,
    ) -> ArbitrageResult<AuthenticationResult> {
        console_log!("🔐 Authenticating request");

        // Extract authentication information from request
        let auth_info = self.extract_auth_info(req)?;

        match auth_info {
            AuthInfo::SessionId(session_id) => self.authenticate_with_session(&session_id).await,
            AuthInfo::TelegramId(telegram_id) => {
                self.authenticate_with_telegram_id(telegram_id).await
            }
            AuthInfo::ApiKey(api_key) => self.authenticate_with_api_key(&api_key).await,
            AuthInfo::None => {
                console_log!("❌ No authentication information found in request");
                Err(ArbitrageError::authentication_error(
                    "No authentication information provided",
                ))
            }
        }
    }

    /// Authorize request for specific action
    pub async fn authorize_request(
        &self,
        user_profile: &UserProfile,
        action: &str,
    ) -> ArbitrageResult<AuthorizationResult> {
        console_log!(
            "🔍 Authorizing request for user {} action '{}'",
            user_profile.user_id,
            action
        );

        // Check if user has the required permission for this action
        let user_context = UserContext {
            user_profile: user_profile.clone(),
            session_info: SessionInfo {
                session_id: "temp".to_string(), // This will be properly set in authenticate_request
                user_id: user_profile.user_id.clone(),
                created_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(24), // 24 hour session
                last_activity: chrono::Utc::now(),
            },
            permissions: self.load_user_permissions(user_profile).await?,
        };

        let permission_checker = PermissionChecker::new(&self.service_container).await?;
        let has_permission = permission_checker
            .check_permission(&user_context.user_profile, action)
            .await?;

        if has_permission {
            console_log!(
                "✅ Authorization granted for user {} action '{}'",
                user_profile.user_id,
                action
            );
            Ok(AuthorizationResult {
                is_authorized: true,
                user_profile: user_profile.clone(),
                action: action.to_string(),
                reason: None,
            })
        } else {
            console_log!(
                "❌ Authorization denied for user {} action '{}'",
                user_profile.user_id,
                action
            );
            Ok(AuthorizationResult {
                is_authorized: false,
                user_profile: user_profile.clone(),
                action: action.to_string(),
                reason: Some("Insufficient permissions".to_string()),
            })
        }
    }

    /// Extract authentication information from request
    fn extract_auth_info(&self, req: &Request) -> ArbitrageResult<AuthInfo> {
        let headers = req.headers();

        // Check for session ID in headers
        if let Ok(Some(session_id)) = headers.get("X-Session-ID") {
            return Ok(AuthInfo::SessionId(session_id));
        }

        // Check for user ID in headers (legacy support)
        if let Ok(Some(user_id)) = headers.get("X-User-ID") {
            if let Ok(telegram_id) = user_id.parse::<i64>() {
                return Ok(AuthInfo::TelegramId(telegram_id));
            }
        }

        // Check for API key in Authorization header
        if let Ok(Some(auth_header)) = headers.get("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let api_key = auth_header
                    .strip_prefix("Bearer ")
                    .unwrap_or("")
                    .to_string();
                return Ok(AuthInfo::ApiKey(api_key));
            }
        }

        // Check for Telegram ID in custom header
        if let Ok(Some(telegram_id_str)) = headers.get("X-Telegram-ID") {
            if let Ok(telegram_id) = telegram_id_str.parse::<i64>() {
                return Ok(AuthInfo::TelegramId(telegram_id));
            }
        }

        Ok(AuthInfo::None)
    }

    /// Authenticate with session ID
    async fn authenticate_with_session(
        &self,
        session_id: &str,
    ) -> ArbitrageResult<AuthenticationResult> {
        console_log!("🔐 Authenticating with session ID: {}", session_id);

        // Extract user_id from session validation
        let session_validator = SessionValidator::new(&self.service_container)
            .await
            .map_err(|e| {
                ArbitrageError::from(format!("Failed to create session validator: {}", e))
            })?;
        let validation_result = session_validator
            .validate_and_get_context(session_id)
            .await
            .map_err(|e| ArbitrageError::from(format!("Session validation failed: {}", e)))?;

        if !validation_result.is_valid {
            return Err(ArbitrageError::from("Invalid session".to_string()));
        }

        let session_service = &self.service_container.session_service;

        // Get session and validate
        let session = session_service
            .get_session(session_id)
            .await
            .map_err(|_| ArbitrageError::unauthorized("Invalid session"))?;

        if !session.is_active() {
            return Err(ArbitrageError::unauthorized("Session expired or inactive"));
        }

        // Get user profile from session
        let user_profile_service =
            self.service_container
                .user_profile_service()
                .ok_or_else(|| {
                    ArbitrageError::service_unavailable("User profile service not available")
                })?;

        let user_profile = user_profile_service
            .get_user_profile(&session.user_id)
            .await
            .map_err(|_| ArbitrageError::unauthorized("Failed to get user profile"))?
            .ok_or_else(|| ArbitrageError::unauthorized("User not found"))?;

        // Create temporary user context
        let session_info = SessionInfo {
            session_id: session.session_id.clone(),
            user_id: session.user_id.clone(),
            created_at: chrono::DateTime::from_timestamp_millis(session.started_at as i64)
                .unwrap_or_else(chrono::Utc::now),
            expires_at: chrono::DateTime::from_timestamp_millis(session.expires_at as i64)
                .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(24)),
            last_activity: chrono::DateTime::from_timestamp_millis(session.last_activity_at as i64)
                .unwrap_or_else(chrono::Utc::now),
        };

        // Load user permissions from database
        let permissions = self.load_user_permissions(&user_profile).await?;

        let user_context = UserContext {
            user_profile,
            session_info,
            permissions,
        };

        console_log!(
            "✅ Session authentication successful for user: {}",
            user_context.user_profile.user_id
        );

        Ok(AuthenticationResult {
            is_authenticated: true,
            user_profile: Some(user_context.user_profile),
            auth_method: AuthMethod::Session,
            session_id: Some(session_id.to_string()),
            error_message: None,
        })
    }

    /// Authenticate with Telegram ID (legacy support)
    async fn authenticate_with_telegram_id(
        &self,
        telegram_id: i64,
    ) -> ArbitrageResult<AuthenticationResult> {
        console_log!("🔐 Authenticating with Telegram ID: {}", telegram_id);

        // Get user profile service
        let user_profile_service =
            self.service_container
                .user_profile_service()
                .ok_or_else(|| {
                    ArbitrageError::service_unavailable("User profile service not available")
                })?;

        // Get user profile
        let user_id = telegram_id.to_string();
        match user_profile_service.get_user_profile(&user_id).await {
            Ok(Some(user_profile)) => {
                console_log!(
                    "✅ Telegram ID authentication successful for user: {}",
                    user_profile.user_id
                );
                Ok(AuthenticationResult {
                    is_authenticated: true,
                    user_profile: Some(user_profile),
                    auth_method: AuthMethod::TelegramId,
                    session_id: None,
                    error_message: None,
                })
            }
            Ok(None) => {
                console_log!("❌ Telegram ID authentication failed: user not found");
                Ok(AuthenticationResult {
                    is_authenticated: false,
                    user_profile: None,
                    auth_method: AuthMethod::TelegramId,
                    session_id: None,
                    error_message: Some("User not found".to_string()),
                })
            }
            Err(_) => {
                console_log!("❌ Telegram ID authentication failed: user not found");
                Ok(AuthenticationResult {
                    is_authenticated: false,
                    user_profile: None,
                    auth_method: AuthMethod::TelegramId,
                    session_id: None,
                    error_message: Some("User not found".to_string()),
                })
            }
        }
    }

    /// Authenticate with API key
    async fn authenticate_with_api_key(
        &self,
        api_key: &str,
    ) -> ArbitrageResult<AuthenticationResult> {
        console_log!(
            "🔐 Authenticating with API key: {}...",
            &api_key[..8.min(api_key.len())]
        );

        if api_key.is_empty() {
            return Ok(AuthenticationResult {
                is_authenticated: false,
                user_profile: None,
                auth_method: AuthMethod::ApiKey,
                session_id: None,
                error_message: Some("API key cannot be empty".to_string()),
            });
        }

        // Get D1 service for API key validation
        let d1_service = self.service_container.database_manager.get_database();

        // Check API key in database
        let stmt = d1_service.prepare(
            "SELECT ak.key_id, ak.user_id, ak.is_active, ak.expires_at, ak.permissions, ak.last_used_at,
                    up.user_id, up.username, up.role, up.access_level, up.subscription_tier
             FROM api_keys ak
             JOIN user_profiles up ON ak.user_id = up.user_id
             WHERE ak.key_hash = ? AND ak.is_active = 1"
        );

        // Hash the API key for comparison (in production, use proper hashing)
        let key_hash = format!("{:x}", md5::compute(api_key.as_bytes()));

        let result = stmt
            .bind(&[key_hash.clone().into()])?
            .first::<serde_json::Value>(None)
            .await;

        match result {
            Ok(Some(row)) => {
                let expires_at = row["expires_at"].as_u64().unwrap_or(0);
                let now = chrono::Utc::now().timestamp_millis() as u64;

                // Check if API key is expired
                if expires_at > 0 && expires_at < now {
                    console_log!(
                        "❌ API key expired: {}...",
                        &api_key[..8.min(api_key.len())]
                    );
                    return Ok(AuthenticationResult {
                        is_authenticated: false,
                        user_profile: None,
                        auth_method: AuthMethod::ApiKey,
                        session_id: None,
                        error_message: Some("API key has expired".to_string()),
                    });
                }

                // Update last used timestamp
                let update_stmt = d1_service.prepare(
                    "UPDATE api_keys SET last_used_at = ?, usage_count = usage_count + 1 WHERE key_hash = ?"
                );
                let _ = update_stmt
                    .bind(&[now.into(), key_hash.into()])?
                    .run()
                    .await;

                // Create user profile from database row
                let user_profile = UserProfile {
                    user_id: row["user_id"].as_str().unwrap_or("").to_string(),
                    telegram_user_id: None,
                    telegram_username: None,
                    username: row["username"].as_str().map(|s| s.to_string()),
                    email: None,
                    access_level: serde_json::from_str(
                        row["access_level"].as_str().unwrap_or("\"Basic\""),
                    )
                    .unwrap_or(UserAccessLevel::Basic),
                    subscription_tier: serde_json::from_str(
                        row["subscription_tier"].as_str().unwrap_or("\"Free\""),
                    )
                    .unwrap_or(SubscriptionTier::Free),
                    api_keys: Vec::new(),
                    preferences: UserPreferences::default(),
                    risk_profile: RiskProfile::default(),
                    created_at: 0,
                    updated_at: 0,
                    last_active: 0,
                    last_login: None,
                    is_active: true,
                    is_beta_active: false,
                    invitation_code_used: None,
                    invitation_code: None,
                    invited_by: None,
                    total_invitations_sent: 0,
                    successful_invitations: 0,
                    beta_expires_at: None,
                    total_trades: 0,
                    total_pnl_usdt: 0.0,
                    account_balance_usdt: 0.0,
                    profile_metadata: None,
                    subscription: Subscription::new(
                        serde_json::from_str(
                            row["subscription_tier"].as_str().unwrap_or("\"Free\""),
                        )
                        .unwrap_or(SubscriptionTier::Free),
                    ),
                    group_admin_roles: Vec::new(),
                    configuration: UserConfiguration::default(),
                };

                console_log!(
                    "✅ API key authenticated for user: {}",
                    user_profile.user_id
                );

                Ok(AuthenticationResult {
                    is_authenticated: true,
                    user_profile: Some(user_profile),
                    auth_method: AuthMethod::ApiKey,
                    session_id: None,
                    error_message: None,
                })
            }
            _ => {
                console_log!(
                    "❌ Invalid API key: {}...",
                    &api_key[..8.min(api_key.len())]
                );
                Ok(AuthenticationResult {
                    is_authenticated: false,
                    user_profile: None,
                    auth_method: AuthMethod::ApiKey,
                    session_id: None,
                    error_message: Some("Invalid API key".to_string()),
                })
            }
        }
    }

    /// Check if request requires authentication
    pub fn requires_authentication(&self, path: &str, _method: &str) -> bool {
        // Public endpoints that don't require authentication
        let public_endpoints = [
            "/health",
            "/health/detailed",
            "/telegram/webhook", // Telegram webhook is authenticated differently
        ];

        // Check if path is public
        for public_path in &public_endpoints {
            if path.starts_with(public_path) {
                return false;
            }
        }

        // All other endpoints require authentication
        true
    }

    /// Load user permissions from database
    async fn load_user_permissions(
        &self,
        user_profile: &UserProfile,
    ) -> ArbitrageResult<Vec<String>> {
        let d1_service = self.service_container.database_manager.get_database();

        // Query user-specific permissions
        let stmt = d1_service.prepare(
            "SELECT DISTINCT p.permission_name 
             FROM user_permissions up
             JOIN permissions p ON up.permission_id = p.permission_id
             WHERE up.user_id = ? AND up.is_active = 1
             UNION
             SELECT DISTINCT p.permission_name
             FROM role_permissions rp
             JOIN permissions p ON rp.permission_id = p.permission_id
             WHERE rp.role = ? AND rp.is_active = 1
             UNION
             SELECT DISTINCT p.permission_name
             FROM access_level_permissions alp
             JOIN permissions p ON alp.permission_id = p.permission_id
             WHERE alp.access_level = ? AND alp.is_active = 1",
        );

        let role_str = serde_json::to_string(&user_profile.get_user_role()).unwrap_or_default();
        let access_level_str =
            serde_json::to_string(&user_profile.access_level).unwrap_or_default();

        let results = stmt
            .bind(&[
                user_profile.user_id.clone().into(),
                role_str.into(),
                access_level_str.into(),
            ])?
            .all()
            .await?;

        let mut permissions = Vec::new();
        for result in results.results::<serde_json::Value>()? {
            if let Some(permission_name) = result.get("permission_name") {
                if let Some(name_str) = permission_name.as_str() {
                    permissions.push(name_str.to_string());
                }
            }
        }

        console_log!(
            "📋 Loaded {} permissions for user: {}",
            permissions.len(),
            user_profile.user_id
        );
        Ok(permissions)
    }

    /// Check if request requires specific permission
    pub fn get_required_permission(&self, path: &str, method: &str) -> Option<String> {
        match (method, path) {
            // User management endpoints
            ("GET", "/api/v1/user/profile") => Some("users.read".to_string()),
            ("PUT", "/api/v1/user/profile") => Some("users.update".to_string()),

            // Admin endpoints
            ("GET", "/api/v1/admin/users") => Some("users.read".to_string()),
            ("GET", "/api/v1/admin/system") => Some("system.admin".to_string()),
            ("GET", "/api/v1/admin/config") => Some("system.config".to_string()),
            ("PUT", "/api/v1/admin/config") => Some("system.config".to_string()),

            // Trading endpoints
            ("GET", "/api/v1/trading/balance") => Some("trading.manual".to_string()),
            ("POST", "/api/v1/trading/order") => Some("trading.manual".to_string()),

            // AI endpoints
            ("POST", "/api/v1/ai/analyze") => Some("ai.basic".to_string()),

            // Analytics endpoints
            ("GET", "/api/v1/analytics/dashboard") => Some("analytics.basic".to_string()),

            _ => None, // No specific permission required
        }
    }
}

/// Authentication information extracted from request
#[derive(Debug, Clone)]
enum AuthInfo {
    SessionId(String),
    TelegramId(i64),
    ApiKey(String),
    None,
}

/// Authentication method used
#[derive(Debug, Clone)]
pub enum AuthMethod {
    Session,
    TelegramId,
    ApiKey,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    pub is_authenticated: bool,
    pub user_profile: Option<UserProfile>,
    pub auth_method: AuthMethod,
    pub session_id: Option<String>,
    pub error_message: Option<String>,
}

/// Authorization result
#[derive(Debug, Clone)]
pub struct AuthorizationResult {
    pub is_authorized: bool,
    pub user_profile: UserProfile,
    pub action: String,
    pub reason: Option<String>,
}
