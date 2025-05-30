//! User Onboarding Commands
//! 
//! Priority 1: User Onboarding & Session Management
//! - Session creation and validation
//! - Initial profile setup
//! - Beta access validation
//! - Welcome flow

use crate::services::interfaces::telegram::{UserInfo, UserPermissions};
use crate::services::core::infrastructure::service_container::ServiceContainer;
use crate::services::core::user::session_management::{SessionManagementService, EnhancedUserSession};
use crate::services::core::user::user_profile::UserProfileService;
use crate::types::{UserRole, UserProfile};
use crate::utils::{ArbitrageError, ArbitrageResult};
use worker::console_log;
use std::sync::Arc;

/// Handle /start command - Primary onboarding entry point
pub async fn handle_start_command(
    service_container: &Arc<ServiceContainer>,
    user_info: &UserInfo,
    permissions: &UserPermissions,
    args: &[&str],
) -> ArbitrageResult<String> {
    console_log!("🚀 Starting onboarding for user {}", user_info.user_id);

    // Step 1: Session Management (Critical Priority)
    let session_result = create_or_update_session(service_container, user_info).await?;
    
    // Step 2: Profile Management (High Priority)
    let profile_result = ensure_user_profile(service_container, user_info).await?;
    
    // Step 3: Beta Access Validation (High Priority)
    let beta_status = validate_beta_access(service_container, user_info).await?;
    
    // Step 4: Generate Welcome Message
    let welcome_message = generate_welcome_message(
        user_info,
        &session_result,
        &profile_result,
        &beta_status,
        args,
    ).await?;
    
    console_log!("✅ Onboarding completed for user {}", user_info.user_id);
    Ok(welcome_message)
}

/// Create or update user session
async fn create_or_update_session(
    service_container: &Arc<ServiceContainer>,
    user_info: &UserInfo,
) -> ArbitrageResult<SessionResult> {
    console_log!("🔐 Managing session for user {}", user_info.user_id);

    let session_service = service_container
        .get_session_management_service()
        .ok_or_else(|| ArbitrageError::service_unavailable("Session management service not available"))?;

    let user_id_str = user_info.user_id.to_string();
    let telegram_id = user_info.user_id;

    // Check if session already exists
    let existing_session = session_service.validate_session(&user_id_str).await?;

    if existing_session {
        // Update existing session activity
        session_service.update_activity(&user_id_str).await?;
        console_log!("✅ Updated existing session for user {}", user_info.user_id);
        
        Ok(SessionResult {
            is_new_session: false,
            session_id: user_id_str,
            message: "Welcome back! Your session has been refreshed.".to_string(),
        })
    } else {
        // Create new session
        let new_session = session_service.start_session(telegram_id).await?;
        console_log!("✅ Created new session for user {}: {}", user_info.user_id, new_session.session_id);
        
        Ok(SessionResult {
            is_new_session: true,
            session_id: new_session.session_id,
            message: "New session created! Welcome to ArbEdge.".to_string(),
        })
    }
}

/// Ensure user profile exists and is up to date
async fn ensure_user_profile(
    service_container: &Arc<ServiceContainer>,
    user_info: &UserInfo,
) -> ArbitrageResult<ProfileResult> {
    console_log!("👤 Managing profile for user {}", user_info.user_id);

    let user_profile_service = service_container
        .get_user_profile_service()
        .ok_or_else(|| ArbitrageError::service_unavailable("User profile service not available"))?;

    let user_id_str = user_info.user_id.to_string();

    // Try to get existing profile
    match user_profile_service.get_user_profile(&user_id_str).await {
        Ok(existing_profile) => {
            console_log!("✅ Found existing profile for user {}", user_info.user_id);
            
            // Update profile with latest Telegram info if needed
            let updated_profile = update_profile_from_telegram(existing_profile, user_info)?;
            
            Ok(ProfileResult {
                is_new_profile: false,
                profile: updated_profile,
                message: "Profile loaded successfully.".to_string(),
            })
        }
        Err(_) => {
            // Create new profile
            console_log!("🆕 Creating new profile for user {}", user_info.user_id);
            let new_profile = create_profile_from_telegram(user_info)?;
            
            // Save new profile
            user_profile_service.create_user_profile(&new_profile).await?;
            
            Ok(ProfileResult {
                is_new_profile: true,
                profile: new_profile,
                message: "New profile created! Welcome to ArbEdge.".to_string(),
            })
        }
    }
}

/// Validate beta access for user
async fn validate_beta_access(
    service_container: &Arc<ServiceContainer>,
    user_info: &UserInfo,
) -> ArbitrageResult<BetaAccessResult> {
    console_log!("🧪 Validating beta access for user {}", user_info.user_id);

    let user_profile_service = service_container
        .get_user_profile_service()
        .ok_or_else(|| ArbitrageError::service_unavailable("User profile service not available"))?;

    let user_id_str = user_info.user_id.to_string();
    let user_profile = user_profile_service.get_user_profile(&user_id_str).await?;

    let beta_access = user_profile.beta_access;
    let beta_expires_at = user_profile.beta_expires_at;

    // Check if beta access is still valid
    let is_beta_valid = if beta_access {
        if let Some(expires_at) = beta_expires_at {
            chrono::Utc::now() < expires_at
        } else {
            true // No expiration set
        }
    } else {
        false
    };

    console_log!("✅ Beta access validation for user {}: {}", user_info.user_id, is_beta_valid);

    Ok(BetaAccessResult {
        has_beta_access: is_beta_valid,
        expires_at: beta_expires_at,
        message: if is_beta_valid {
            "Beta access active.".to_string()
        } else {
            "Beta access not available.".to_string()
        },
    })
}

/// Create user profile from Telegram information
fn create_profile_from_telegram(user_info: &UserInfo) -> ArbitrageResult<UserProfile> {
    let now = chrono::Utc::now();
    
    Ok(UserProfile {
        user_id: user_info.user_id.to_string(),
        telegram_id: Some(user_info.user_id),
        username: user_info.username.clone(),
        first_name: user_info.first_name.clone(),
        last_name: None,
        email: None,
        role: UserRole::Basic, // Default role for new users
        subscription_tier: "free".to_string(),
        beta_access: false, // Default: no beta access (requires invitation)
        beta_expires_at: None,
        can_trade: false, // Default: no trading until API keys added
        daily_opportunity_limit: 3, // Free tier limit
        created_at: now,
        updated_at: now,
        last_login: Some(now),
        is_active: true,
        preferences: serde_json::json!({
            "notifications": true,
            "language": "en",
            "timezone": "UTC"
        }),
    })
}

/// Update existing profile with latest Telegram information
fn update_profile_from_telegram(
    mut profile: UserProfile,
    user_info: &UserInfo,
) -> ArbitrageResult<UserProfile> {
    let now = chrono::Utc::now();
    
    // Update Telegram-specific fields
    profile.telegram_id = Some(user_info.user_id);
    profile.username = user_info.username.clone();
    profile.first_name = user_info.first_name.clone();
    profile.last_login = Some(now);
    profile.updated_at = now;
    profile.is_active = true;
    
    Ok(profile)
}

/// Generate comprehensive welcome message
async fn generate_welcome_message(
    user_info: &UserInfo,
    session_result: &SessionResult,
    profile_result: &ProfileResult,
    beta_status: &BetaAccessResult,
    args: &[&str],
) -> ArbitrageResult<String> {
    let user_name = user_info.first_name
        .as_ref()
        .or(user_info.username.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("Trader");

    let mut message = String::new();
    
    // Welcome header
    if session_result.is_new_session || profile_result.is_new_profile {
        message.push_str(&format!("🚀 *Welcome to ArbEdge, {}!*\n\n", user_name));
        message.push_str("Your advanced cryptocurrency arbitrage assistant is ready.\n\n");
    } else {
        message.push_str(&format!("👋 *Welcome back, {}!*\n\n", user_name));
    }
    
    // Session status
    message.push_str("🔐 *Session Status*\n");
    message.push_str(&format!("✅ {}\n\n", session_result.message));
    
    // Profile status
    message.push_str("👤 *Profile Status*\n");
    message.push_str(&format!("✅ {}\n", profile_result.message));
    message.push_str(&format!("Role: {:?}\n", profile_result.profile.role));
    message.push_str(&format!("Subscription: {}\n\n", profile_result.profile.subscription_tier));
    
    // Beta access status
    message.push_str("🧪 *Beta Access*\n");
    if beta_status.has_beta_access {
        message.push_str("✅ Beta access active\n");
        if let Some(expires_at) = beta_status.expires_at {
            message.push_str(&format!("Expires: {}\n", expires_at.format("%Y-%m-%d")));
        }
    } else {
        message.push_str("❌ Beta access not available\n");
        message.push_str("Contact admin for beta invitation\n");
    }
    message.push_str("\n");
    
    // Quick start guide
    message.push_str("🎯 *Quick Start*\n");
    message.push_str("• `/opportunities` - View trading opportunities\n");
    message.push_str("• `/profile` - Manage your profile\n");
    message.push_str("• `/settings` - Configure preferences\n");
    message.push_str("• `/help` - Get help and commands\n\n");
    
    // Beta features (if available)
    if beta_status.has_beta_access {
        message.push_str("🧪 *Beta Features Available*\n");
        message.push_str("• `/beta` - Access beta features\n");
        message.push_str("• Advanced AI analysis\n");
        message.push_str("• Priority opportunity access\n\n");
    }
    
    // Next steps for new users
    if profile_result.is_new_profile {
        message.push_str("📋 *Next Steps*\n");
        message.push_str("1. Complete your profile with `/profile`\n");
        message.push_str("2. Configure notifications with `/settings`\n");
        message.push_str("3. Start exploring opportunities!\n\n");
    }
    
    message.push_str("Ready to start trading? Let's find some opportunities! 🚀");
    
    Ok(message)
}

// Result structures
#[derive(Debug)]
struct SessionResult {
    is_new_session: bool,
    session_id: String,
    message: String,
}

#[derive(Debug)]
struct ProfileResult {
    is_new_profile: bool,
    profile: UserProfile,
    message: String,
}

#[derive(Debug)]
struct BetaAccessResult {
    has_beta_access: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    message: String,
} 