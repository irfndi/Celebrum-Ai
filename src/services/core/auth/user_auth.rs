//! User Authentication Service
//!
//! Handles user authentication and profile creation:
//! - User onboarding and registration
//! - Profile creation with default settings
//! - Invitation code validation
//! - Beta access assignment

use crate::services::core::infrastructure::service_container::ServiceContainer;

use crate::types::{
    RiskProfile, Subscription, SubscriptionTier, UserAccessLevel, UserConfiguration,
    UserPreferences, UserProfile,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
use std::sync::Arc;
use worker::{console_log, D1Database};

/// User Authentication Service
pub struct UserAuthService {
    service_container: Arc<ServiceContainer>,
    d1_service: Arc<D1Database>,
}

impl UserAuthService {
    /// Create new user authentication service
    pub async fn new(service_container: &Arc<ServiceContainer>) -> ArbitrageResult<Self> {
        console_log!("👤 Initializing User Authentication Service...");

        let d1_service = service_container.database_manager.get_database();

        console_log!("✅ User Authentication Service initialized successfully");

        Ok(Self {
            service_container: service_container.clone(),
            d1_service,
        })
    }

    /// Create new user profile during authentication
    pub async fn create_new_user_profile(
        &self,
        telegram_id: i64,
        invitation_code: Option<String>,
    ) -> ArbitrageResult<UserProfile> {
        console_log!(
            "🆕 Creating new user profile for telegram_id: {}",
            telegram_id
        );

        // Get user profile service
        let user_profile_service =
            self.service_container
                .user_profile_service()
                .ok_or_else(|| {
                    ArbitrageError::service_unavailable("User profile service not available")
                })?;

        // Create profile with default settings
        let mut new_profile = self.create_default_profile(telegram_id)?;

        // Apply invitation code benefits if provided
        if let Some(ref code) = invitation_code {
            self.apply_invitation_benefits(&mut new_profile, code)
                .await?;
        }

        // Save the new profile
        let created_profile = user_profile_service
            .create_user_profile(telegram_id, invitation_code, None)
            .await?;

        console_log!(
            "✅ New user profile created for telegram_id: {} with role: {:?}",
            telegram_id,
            created_profile.get_user_role()
        );
        Ok(created_profile)
    }

    /// Authenticate existing user
    pub async fn authenticate_existing_user(
        &self,
        user_profile: &UserProfile,
    ) -> ArbitrageResult<LoginResult> {
        console_log!("🔐 Authenticating existing user: {}", user_profile.user_id);

        // Update last login time
        let user_profile_service =
            self.service_container
                .user_profile_service()
                .ok_or_else(|| {
                    ArbitrageError::service_unavailable("User profile service not available")
                })?;

        let mut updated_profile = user_profile.clone();
        let now_timestamp = chrono::Utc::now().timestamp() as u64;
        updated_profile.last_login = Some(now_timestamp);
        updated_profile.updated_at = now_timestamp;

        user_profile_service
            .update_user_profile(&updated_profile)
            .await?;

        console_log!(
            "✅ User authenticated successfully: {}",
            user_profile.user_id
        );

        Ok(LoginResult {
            user_profile: updated_profile,
            is_new_user: false,
            login_time: chrono::Utc::now(),
        })
    }

    /// Create default user profile
    fn create_default_profile(&self, telegram_id: i64) -> ArbitrageResult<UserProfile> {
        let now = chrono::Utc::now();

        Ok(UserProfile {
            user_id: telegram_id.to_string(),
            username: None, // Will be updated from Telegram info or other sources
            telegram_user_id: Some(telegram_id),
            telegram_username: None, // Will be updated from Telegram info
            email: None,
            access_level: UserAccessLevel::Free, // Default role for new users
            subscription_tier: SubscriptionTier::Free, // Default to Free tier
            subscription: Subscription::default(), // Default to free tier via Subscription struct
            is_beta_active: false,               // Default: no beta access
            beta_expires_at: None,
            created_at: now.timestamp_millis() as u64,
            updated_at: now.timestamp_millis() as u64,
            last_login: Some(now.timestamp_millis() as u64),
            is_active: true,
            preferences: UserPreferences::default(), // Use default UserPreferences
            // Initialize other UserProfile fields as per its definition in types.rs
            // Ensure all non-Option fields and fields without a default in UserProfile::default() are covered.
            // Example (check types.rs for actual fields and types):
            api_keys: Vec::new(),
            risk_profile: RiskProfile::default(),
            last_active: now.timestamp_millis() as u64,
            invitation_code_used: None,
            invitation_code: None,
            invited_by: None,
            total_invitations_sent: 0,
            successful_invitations: 0,
            total_trades: 0,
            total_pnl_usdt: 0.0,
            account_balance_usdt: 0.0,
            profile_metadata: None,
            group_admin_roles: Vec::new(),
            configuration: UserConfiguration::default(),
            // Fields like `can_trade` and `daily_opportunity_limit` are derived or part of Subscription
        })
    }

    /// Apply invitation code benefits to profile
    async fn apply_invitation_benefits(
        &self,
        profile: &mut UserProfile,
        invitation_code: &str,
    ) -> ArbitrageResult<()> {
        console_log!(
            "🎫 Applying invitation benefits for code: {}",
            invitation_code
        );

        if invitation_code.is_empty() {
            return Ok(());
        }

        // Validate invitation code and get benefits
        let validation_result = self.validate_invitation_code(invitation_code).await?;

        if !validation_result.is_valid {
            return Err(ArbitrageError::validation_error(
                validation_result
                    .error_message
                    .unwrap_or("Invalid invitation code".to_string()),
            ));
        }

        if let Some(benefits) = validation_result.benefits {
            // Apply beta access if included in benefits
            if benefits.beta_access {
                profile.is_beta_active = true;
                profile.beta_expires_at = Some(
                    (chrono::Utc::now()
                        + chrono::Duration::days(benefits.beta_duration_days as i64))
                    .timestamp_millis() as u64,
                );

                // Update subscription tier for beta users
                profile.subscription = Subscription::new(SubscriptionTier::Beta);

                // Override daily opportunity limit if specified in benefits
                if benefits.daily_opportunity_limit > 0 {
                    profile.subscription.daily_opportunity_limit =
                        Some(benefits.daily_opportunity_limit as u32);
                }
            }

            // Update preferences for invitation code users
            profile.preferences.has_beta_features_enabled = Some(benefits.beta_access);
            profile.preferences.applied_invitation_code = Some(invitation_code.to_string());

            // Store special features in profile metadata
            if !benefits.special_features.is_empty() {
                let metadata = serde_json::json!({
                    "invitation_features": benefits.special_features
                });
                profile.profile_metadata = Some(serde_json::to_string(&metadata)?);
            }

            console_log!(
                "✅ Invitation benefits applied to user {} via code {}: beta_access={}, duration={} days, daily_limit={}",
                profile.user_id,
                invitation_code,
                benefits.beta_access,
                benefits.beta_duration_days,
                benefits.daily_opportunity_limit
            );
        }

        Ok(())
    }

    /// Create a new invitation code (admin function)
    pub async fn create_invitation_code(
        &self,
        code: &str,
        benefits: &InvitationBenefits,
        max_uses: Option<u32>,
        expires_at: Option<u64>,
        created_by: &str,
    ) -> ArbitrageResult<()> {
        let benefits_json = serde_json::to_string(benefits)?;
        let now = chrono::Utc::now().timestamp_millis() as u64;

        let stmt = self.d1_service.prepare(
            "INSERT INTO invitation_codes 
             (code, is_active, max_uses, current_uses, benefits_json, expires_at, created_by, created_at)
             VALUES (?, 1, ?, 0, ?, ?, ?, ?)"
        );

        stmt.bind(&[
            code.into(),
            max_uses.unwrap_or(0).into(),
            benefits_json.into(),
            expires_at.unwrap_or(0).into(),
            created_by.into(),
            now.into(),
        ])?
        .run()
        .await
        .map_err(|e| {
            ArbitrageError::database_error(format!("Failed to create invitation code: {}", e))
        })?;

        console_log!("✅ Created invitation code: {}", code);
        Ok(())
    }

    /// Update user profile from Telegram information
    pub async fn update_profile_from_telegram(
        &self,
        user_profile: &mut UserProfile,
        telegram_info: &TelegramUserInfo,
    ) -> ArbitrageResult<()> {
        console_log!(
            "📱 Updating profile from Telegram info for user: {}",
            user_profile.user_id
        );

        // Update Telegram-specific fields
        user_profile.telegram_user_id = Some(telegram_info.telegram_id);
        user_profile.telegram_username = telegram_info.username.clone();
        // Note: first_name and last_name are not fields in UserProfile
        user_profile.updated_at = chrono::Utc::now().timestamp() as u64;

        // Save updated profile
        let user_profile_service =
            self.service_container
                .user_profile_service()
                .ok_or_else(|| {
                    ArbitrageError::service_unavailable("User profile service not available")
                })?;

        user_profile_service
            .update_user_profile(user_profile)
            .await?;

        console_log!(
            "✅ Profile updated from Telegram info for user: {}",
            user_profile.user_id
        );
        Ok(())
    }

    /// Validate invitation code
    pub async fn validate_invitation_code(
        &self,
        invitation_code: &str,
    ) -> ArbitrageResult<InvitationValidationResult> {
        console_log!("🎫 Validating invitation code: {}", invitation_code);

        if invitation_code.is_empty() {
            return Ok(InvitationValidationResult {
                is_valid: false,
                benefits: None,
                error_message: Some("Invitation code cannot be empty".to_string()),
            });
        }

        // Check invitation code in database
        let stmt = self.d1_service.prepare(
            "SELECT code, is_active, max_uses, current_uses, benefits_json, expires_at, created_by 
             FROM invitation_codes 
             WHERE code = ? AND is_active = 1",
        );

        let result = stmt
            .bind(&[invitation_code.into()])?
            .first::<serde_json::Value>(None)
            .await;

        match result {
            Ok(Some(row)) => {
                let max_uses = row["max_uses"].as_u64().unwrap_or(0);
                let current_uses = row["current_uses"].as_u64().unwrap_or(0);
                let expires_at = row["expires_at"].as_u64().unwrap_or(0);
                let now = chrono::Utc::now().timestamp_millis() as u64;

                // Check if code is expired
                if expires_at > 0 && expires_at < now {
                    return Ok(InvitationValidationResult {
                        is_valid: false,
                        benefits: None,
                        error_message: Some("Invitation code has expired".to_string()),
                    });
                }

                // Check if code has reached max uses
                if max_uses > 0 && current_uses >= max_uses {
                    return Ok(InvitationValidationResult {
                        is_valid: false,
                        benefits: None,
                        error_message: Some("Invitation code has reached maximum uses".to_string()),
                    });
                }

                // Parse benefits from JSON
                let benefits_json = row["benefits_json"].as_str().unwrap_or("{}");
                let benefits: InvitationBenefits = serde_json::from_str(benefits_json)
                    .unwrap_or_else(|_| InvitationBenefits {
                        beta_access: true,
                        beta_duration_days: 180,
                        daily_opportunity_limit: 10,
                        special_features: vec!["beta_access".to_string()],
                    });

                // Increment usage count
                let update_stmt = self.d1_service.prepare(
                    "UPDATE invitation_codes SET current_uses = current_uses + 1, last_used_at = ? WHERE code = ?"
                );
                let _ = update_stmt
                    .bind(&[now.into(), invitation_code.into()])?
                    .run()
                    .await;

                console_log!("✅ Invitation code validated: {}", invitation_code);

                Ok(InvitationValidationResult {
                    is_valid: true,
                    benefits: Some(benefits),
                    error_message: None,
                })
            }
            _ => {
                console_log!("❌ Invalid invitation code: {}", invitation_code);
                Ok(InvitationValidationResult {
                    is_valid: false,
                    benefits: None,
                    error_message: Some("Invalid invitation code".to_string()),
                })
            }
        }
    }

    /// Get user onboarding status
    pub async fn get_onboarding_status(&self, user_id: &str) -> ArbitrageResult<OnboardingStatus> {
        console_log!("📋 Getting onboarding status for user: {}", user_id);

        let user_profile_service =
            self.service_container
                .user_profile_service()
                .ok_or_else(|| {
                    ArbitrageError::service_unavailable("User profile service not available")
                })?;

        let user_profile = user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| {
                ArbitrageError::not_found(format!(
                    "User profile not found for user_id: {}",
                    user_id
                ))
            })?;

        // Check onboarding completion status
        let profile_complete =
            user_profile.telegram_username.is_some() && user_profile.username.is_some();
        let preferences_set = !user_profile.preferences.preferred_exchanges.is_empty()
            && user_profile.preferences.min_profit_threshold > 0.0;
        let api_keys_configured = user_profile.access_level.can_trade();

        let completion_percentage =
            calculate_onboarding_completion(profile_complete, preferences_set, api_keys_configured);

        console_log!(
            "✅ Onboarding status retrieved for user: {} ({}% complete)",
            user_id,
            completion_percentage
        );

        Ok(OnboardingStatus {
            is_complete: completion_percentage >= 100,
            completion_percentage,
            profile_complete,
            preferences_set,
            api_keys_configured,
            beta_access: user_profile.is_beta_active,
            next_steps: get_next_onboarding_steps(
                profile_complete,
                preferences_set,
                api_keys_configured,
            ),
        })
    }
}

/// Authentication Provider trait for different auth methods
pub trait AuthenticationProvider {
    #[allow(async_fn_in_trait)] // Required for WASM compatibility in Cloudflare Workers
    async fn authenticate(&self, credentials: &AuthCredentials) -> ArbitrageResult<LoginResult>;
    #[allow(async_fn_in_trait)] // Required for WASM compatibility in Cloudflare Workers
    async fn validate_credentials(&self, credentials: &AuthCredentials) -> ArbitrageResult<bool>;
}

/// Calculate onboarding completion percentage
fn calculate_onboarding_completion(
    profile_complete: bool,
    preferences_set: bool,
    api_keys_configured: bool,
) -> u8 {
    let mut completion = 0u8;

    if profile_complete {
        completion += 40;
    }
    if preferences_set {
        completion += 30;
    }
    if api_keys_configured {
        completion += 30;
    }

    completion
}

/// Get next onboarding steps
fn get_next_onboarding_steps(
    profile_complete: bool,
    preferences_set: bool,
    api_keys_configured: bool,
) -> Vec<String> {
    let mut steps = Vec::new();

    if !profile_complete {
        steps.push("Complete your profile information".to_string());
    }
    if !preferences_set {
        steps.push("Set your trading preferences".to_string());
    }
    if !api_keys_configured {
        steps.push("Configure exchange API keys for trading".to_string());
    }

    if steps.is_empty() {
        steps.push("Onboarding complete! Start exploring opportunities".to_string());
    }

    steps
}

/// Login result
#[derive(Debug, Clone)]
pub struct LoginResult {
    pub user_profile: UserProfile,
    pub is_new_user: bool,
    pub login_time: chrono::DateTime<chrono::Utc>,
}

/// Authentication credentials
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub telegram_id: i64,
    pub invitation_code: Option<String>,
    pub telegram_info: Option<TelegramUserInfo>,
}

/// Telegram user information
#[derive(Debug, Clone)]
pub struct TelegramUserInfo {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Invitation validation result
#[derive(Debug, Clone)]
pub struct InvitationValidationResult {
    pub is_valid: bool,
    pub benefits: Option<InvitationBenefits>,
    pub error_message: Option<String>,
}

/// Invitation benefits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InvitationBenefits {
    pub beta_access: bool,
    pub beta_duration_days: i32,
    pub daily_opportunity_limit: i32,
    pub special_features: Vec<String>,
}

/// Onboarding status
#[derive(Debug, Clone)]
pub struct OnboardingStatus {
    pub is_complete: bool,
    pub completion_percentage: u8,
    pub profile_complete: bool,
    pub preferences_set: bool,
    pub api_keys_configured: bool,
    pub beta_access: bool,
    pub next_steps: Vec<String>,
}
