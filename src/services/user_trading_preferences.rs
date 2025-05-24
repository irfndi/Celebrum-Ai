// User Trading Preferences Service
// Task 1.5: Trading focus selection and automation preferences management

use crate::utils::{ArbitrageResult, ArbitrageError, logger::{Logger, LogLevel}};
use crate::services::D1Service;
use serde::{Deserialize, Serialize};
use worker::*;

// ============= TRADING PREFERENCE TYPES =============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingFocus {
    #[serde(rename = "arbitrage")]
    Arbitrage,      // Default - focus on arbitrage opportunities (low risk)
    #[serde(rename = "technical")]
    Technical,      // Focus on technical analysis trading (higher risk)
    #[serde(rename = "hybrid")]
    Hybrid,         // Both arbitrage and technical (experienced users)
}

impl Default for TradingFocus {
    fn default() -> Self {
        TradingFocus::Arbitrage // Safe default for new users
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutomationLevel {
    #[serde(rename = "manual")]
    Manual,         // Alerts only, user executes manually (default)
    #[serde(rename = "semi_auto")]
    SemiAuto,       // Pre-approval required for each trade
    #[serde(rename = "full_auto")]
    FullAuto,       // Automated execution based on rules
}

impl Default for AutomationLevel {
    fn default() -> Self {
        AutomationLevel::Manual // Safe default for new users
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutomationScope {
    #[serde(rename = "none")]
    None,           // No automation (manual only)
    #[serde(rename = "arbitrage_only")]
    ArbitrageOnly,  // Automate arbitrage trades only
    #[serde(rename = "technical_only")]
    TechnicalOnly,  // Automate technical trades only  
    #[serde(rename = "both")]
    Both,           // Automate both types
}

impl Default for AutomationScope {
    fn default() -> Self {
        AutomationScope::None // Safe default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperienceLevel {
    #[serde(rename = "beginner")]
    Beginner,       // New to trading, conservative approach
    #[serde(rename = "intermediate")]
    Intermediate,   // Some trading experience
    #[serde(rename = "advanced")]
    Advanced,       // Experienced trader
}

impl Default for ExperienceLevel {
    fn default() -> Self {
        ExperienceLevel::Beginner // Safe default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskTolerance {
    #[serde(rename = "conservative")]
    Conservative,   // Low risk, capital preservation focused
    #[serde(rename = "balanced")]
    Balanced,       // Moderate risk tolerance
    #[serde(rename = "aggressive")]
    Aggressive,     // Higher risk tolerance
}

impl Default for RiskTolerance {
    fn default() -> Self {
        RiskTolerance::Conservative // Safe default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradingPreferences {
    pub preference_id: String,
    pub user_id: String,
    
    // Trading Focus Selection
    pub trading_focus: TradingFocus,
    pub experience_level: ExperienceLevel,
    pub risk_tolerance: RiskTolerance,
    
    // Automation Preferences  
    pub automation_level: AutomationLevel,
    pub automation_scope: AutomationScope,
    
    // Feature Access Control
    pub arbitrage_enabled: bool,
    pub technical_enabled: bool,
    pub advanced_analytics_enabled: bool,
    
    // User Preferences
    pub preferred_notification_channels: Vec<String>, // ["telegram", "email", "push"]
    pub trading_hours_timezone: String,
    pub trading_hours_start: String, // "HH:MM" format
    pub trading_hours_end: String,   // "HH:MM" format
    
    // Onboarding Progress
    pub onboarding_completed: bool,
    pub tutorial_steps_completed: Vec<String>,
    
    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
}

impl UserTradingPreferences {
    pub fn new_default(user_id: String) -> Self {
        let preference_id = format!("pref_{}", user_id);
        
        // Use different timestamp generation for WASM vs native
        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            preference_id,
            user_id,
            trading_focus: TradingFocus::default(),
            experience_level: ExperienceLevel::default(),
            risk_tolerance: RiskTolerance::default(),
            automation_level: AutomationLevel::default(),
            automation_scope: AutomationScope::default(),
            arbitrage_enabled: true,  // Default access to arbitrage
            technical_enabled: false, // Opt-in to technical trading
            advanced_analytics_enabled: false, // Opt-in to advanced features
            preferred_notification_channels: vec!["telegram".to_string()],
            trading_hours_timezone: "UTC".to_string(),
            trading_hours_start: "00:00".to_string(),
            trading_hours_end: "23:59".to_string(),
            onboarding_completed: false,
            tutorial_steps_completed: vec![],
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn enable_technical_trading(&mut self) -> ArbitrageResult<()> {
        // Validate that user is not beginner for technical trading
        if self.experience_level == ExperienceLevel::Beginner {
            return Err(ArbitrageError::validation_error(
                "Technical trading requires intermediate or advanced experience level"
            ));
        }
        
        self.technical_enabled = true;
        
        // Update timestamp
        #[cfg(target_arch = "wasm32")]
        {
            self.updated_at = js_sys::Date::now() as u64;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
        
        Ok(())
    }
    
    pub fn set_automation_level(&mut self, level: AutomationLevel, scope: AutomationScope) -> ArbitrageResult<()> {
        // Validate automation access based on experience
        match level {
            AutomationLevel::Manual => {
                // Always allowed
            }
            AutomationLevel::SemiAuto => {
                if self.experience_level == ExperienceLevel::Beginner {
                    return Err(ArbitrageError::validation_error(
                        "Semi-automated trading requires intermediate or advanced experience"
                    ));
                }
            }
            AutomationLevel::FullAuto => {
                if self.experience_level != ExperienceLevel::Advanced {
                    return Err(ArbitrageError::validation_error(
                        "Full automation requires advanced experience level"
                    ));
                }
            }
        }
        
        self.automation_level = level;
        self.automation_scope = scope;
        
        // Update timestamp
        #[cfg(target_arch = "wasm32")]
        {
            self.updated_at = js_sys::Date::now() as u64;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
        
        Ok(())
    }
}

// ============= FEATURE ACCESS CONTROL =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAccess {
    pub arbitrage_alerts: bool,
    pub technical_alerts: bool,
    pub arbitrage_automation: bool,
    pub technical_automation: bool,
    pub advanced_analytics: bool,
    pub priority_notifications: bool,
    pub ai_integration: bool,
    pub custom_indicators: bool,
}

impl FeatureAccess {
    pub fn from_preferences(preferences: &UserTradingPreferences) -> Self {
        Self {
            arbitrage_alerts: preferences.arbitrage_enabled,
            technical_alerts: preferences.technical_enabled,
            arbitrage_automation: preferences.automation_level != AutomationLevel::Manual 
                && (preferences.automation_scope == AutomationScope::ArbitrageOnly 
                    || preferences.automation_scope == AutomationScope::Both),
            technical_automation: preferences.automation_level != AutomationLevel::Manual
                && (preferences.automation_scope == AutomationScope::TechnicalOnly 
                    || preferences.automation_scope == AutomationScope::Both)
                && preferences.technical_enabled,
            advanced_analytics: preferences.advanced_analytics_enabled,
            priority_notifications: preferences.experience_level != ExperienceLevel::Beginner,
            ai_integration: preferences.experience_level != ExperienceLevel::Beginner,
            custom_indicators: preferences.technical_enabled && preferences.experience_level != ExperienceLevel::Beginner,
        }
    }
}

// ============= PREFERENCE VALIDATION =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommendations: Vec<String>,
}

impl PreferenceValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            warnings: vec![],
            errors: vec![],
            recommendations: vec![],
        }
    }
    
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            warnings: vec![],
            errors,
            recommendations: vec![],
        }
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

// ============= USER TRADING PREFERENCES SERVICE =============

pub struct UserTradingPreferencesService {
    d1_service: D1Service,
    logger: Logger,
}

impl UserTradingPreferencesService {
    pub fn new(d1_service: D1Service, logger: Logger) -> Self {
        Self {
            d1_service,
            logger,
        }
    }
    
    /// Create default trading preferences for a new user
    pub async fn create_default_preferences(&self, user_id: &str) -> ArbitrageResult<UserTradingPreferences> {
        let preferences = UserTradingPreferences::new_default(user_id.to_string());
        
        self.logger.info(
            &format!("Creating default trading preferences for user: {}", user_id),
        );
        
        self.d1_service.store_trading_preferences(&preferences).await?;
        
        Ok(preferences)
    }
    
    /// Get user trading preferences
    pub async fn get_preferences(&self, user_id: &str) -> ArbitrageResult<Option<UserTradingPreferences>> {
        self.d1_service.get_trading_preferences(user_id).await
    }
    
    /// Get preferences or create defaults if none exist
    pub async fn get_or_create_preferences(&self, user_id: &str) -> ArbitrageResult<UserTradingPreferences> {
        match self.get_preferences(user_id).await? {
            Some(preferences) => Ok(preferences),
            None => self.create_default_preferences(user_id).await,
        }
    }
    
    /// Update user trading preferences
    pub async fn update_preferences(&self, preferences: &UserTradingPreferences) -> ArbitrageResult<()> {
        // Validate preferences before updating
        let validation = self.validate_preferences(preferences);
        if !validation.is_valid {
            return Err(ArbitrageError::validation_error(&format!(
                "Invalid preferences: {:?}", validation.errors
            )));
        }
        
        self.logger.info(
            &format!("Updating trading preferences for user: {}", preferences.user_id),
        );
        
        self.d1_service.update_trading_preferences(preferences).await
    }

    /// Delete user's trading preferences
    pub async fn delete_preferences(&self, user_id: &str) -> ArbitrageResult<bool> {
        self.logger.info(
            &format!("Deleting trading preferences for user: {}", user_id),
        );
        
        // Use the existing delete method from D1Service
        self.d1_service.delete_trading_preferences(user_id).await
    }
    
    /// Update trading focus
    pub async fn update_trading_focus(&self, user_id: &str, focus: TradingFocus) -> ArbitrageResult<UserTradingPreferences> {
        let mut preferences = self.get_or_create_preferences(user_id).await?;
        
        preferences.trading_focus = focus.clone();
        
        // Auto-enable technical trading if choosing technical or hybrid focus
        match focus {
            TradingFocus::Technical | TradingFocus::Hybrid => {
                if let Err(_) = preferences.enable_technical_trading() {
                    // If they can't enable technical trading, suggest upgrading experience level
                    self.logger.warn(
                        &format!("User {} attempted to enable technical trading but is beginner level", user_id),
                    );
                    return Err(ArbitrageError::validation_error(
                        "Technical trading requires intermediate or advanced experience level. Please update your experience level first."
                    ));
                }
            }
            TradingFocus::Arbitrage => {
                // Keep current technical_enabled setting
            }
        }
        
        // Update timestamp
        #[cfg(target_arch = "wasm32")]
        {
            preferences.updated_at = js_sys::Date::now() as u64;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            preferences.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
        
        self.update_preferences(&preferences).await?;
        
        Ok(preferences)
    }
    
    /// Update automation preferences
    pub async fn update_automation(&self, user_id: &str, level: AutomationLevel, scope: AutomationScope) -> ArbitrageResult<UserTradingPreferences> {
        let mut preferences = self.get_or_create_preferences(user_id).await?;
        
        preferences.set_automation_level(level, scope)?;
        
        // Update timestamp
        #[cfg(target_arch = "wasm32")]
        {
            preferences.updated_at = js_sys::Date::now() as u64;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            preferences.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
        
        self.update_preferences(&preferences).await?;
        
        Ok(preferences)
    }
    
    /// Get feature access for user
    pub async fn get_feature_access(&self, user_id: &str) -> ArbitrageResult<FeatureAccess> {
        let preferences = self.get_or_create_preferences(user_id).await?;
        Ok(FeatureAccess::from_preferences(&preferences))
    }
    
    /// Validate preferences
    pub fn validate_preferences(&self, preferences: &UserTradingPreferences) -> PreferenceValidationResult {
        let mut result = PreferenceValidationResult::valid();
        
        // Validate automation level vs experience
        match preferences.automation_level {
            AutomationLevel::SemiAuto if preferences.experience_level == ExperienceLevel::Beginner => {
                result.errors.push("Semi-automated trading requires intermediate or advanced experience".to_string());
                result.is_valid = false;
            }
            AutomationLevel::FullAuto if preferences.experience_level != ExperienceLevel::Advanced => {
                result.errors.push("Full automation requires advanced experience level".to_string());
                result.is_valid = false;
            }
            _ => {}
        }
        
        // Validate technical trading vs experience
        if preferences.technical_enabled && preferences.experience_level == ExperienceLevel::Beginner {
            result.errors.push("Technical trading requires intermediate or advanced experience".to_string());
            result.is_valid = false;
        }
        
        // Add recommendations
        if preferences.trading_focus == TradingFocus::Arbitrage && preferences.experience_level == ExperienceLevel::Advanced {
            result.add_recommendation("Consider exploring technical trading or hybrid approach for additional opportunities".to_string());
        }
        
        if preferences.automation_level == AutomationLevel::Manual && preferences.experience_level == ExperienceLevel::Advanced {
            result.add_recommendation("Consider semi-automated or automated execution to optimize trading efficiency".to_string());
        }
        
        result
    }
    
    /// Complete onboarding step
    pub async fn complete_onboarding_step(&self, user_id: &str, step: &str) -> ArbitrageResult<UserTradingPreferences> {
        let mut preferences = self.get_or_create_preferences(user_id).await?;
        
        if !preferences.tutorial_steps_completed.contains(&step.to_string()) {
            preferences.tutorial_steps_completed.push(step.to_string());
        }
        
        // Check if onboarding is complete
        let required_steps = vec!["trading_focus_selected", "automation_configured", "exchanges_connected"];
        let completed_required = required_steps.iter()
            .all(|step| preferences.tutorial_steps_completed.contains(&step.to_string()));
        
        if completed_required && !preferences.onboarding_completed {
            preferences.onboarding_completed = true;
            self.logger.info(
                &format!("User {} completed onboarding", user_id),
            );
        }
        
        // Update timestamp
        #[cfg(target_arch = "wasm32")]
        {
            preferences.updated_at = js_sys::Date::now() as u64;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            preferences.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
        
        self.update_preferences(&preferences).await?;
        
        Ok(preferences)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let preferences = UserTradingPreferences::new_default("test_user".to_string());
        
        assert_eq!(preferences.trading_focus, TradingFocus::Arbitrage);
        assert_eq!(preferences.automation_level, AutomationLevel::Manual);
        assert_eq!(preferences.automation_scope, AutomationScope::None);
        assert_eq!(preferences.experience_level, ExperienceLevel::Beginner);
        assert_eq!(preferences.risk_tolerance, RiskTolerance::Conservative);
        assert!(preferences.arbitrage_enabled);
        assert!(!preferences.technical_enabled);
        assert!(!preferences.advanced_analytics_enabled);
        assert!(!preferences.onboarding_completed);
    }
    
    #[test]
    fn test_enable_technical_trading() {
        let mut preferences = UserTradingPreferences::new_default("test_user".to_string());
        
        // Should fail for beginners
        assert!(preferences.enable_technical_trading().is_err());
        
        // Should work for intermediate users
        preferences.experience_level = ExperienceLevel::Intermediate;
        assert!(preferences.enable_technical_trading().is_ok());
        assert!(preferences.technical_enabled);
    }
    
    #[test]
    fn test_automation_level_validation() {
        let mut preferences = UserTradingPreferences::new_default("test_user".to_string());
        
        // Manual should always work
        assert!(preferences.set_automation_level(AutomationLevel::Manual, AutomationScope::None).is_ok());
        
        // Semi-auto should fail for beginners
        assert!(preferences.set_automation_level(AutomationLevel::SemiAuto, AutomationScope::ArbitrageOnly).is_err());
        
        // Should work for intermediate
        preferences.experience_level = ExperienceLevel::Intermediate;
        assert!(preferences.set_automation_level(AutomationLevel::SemiAuto, AutomationScope::ArbitrageOnly).is_ok());
        
        // Full auto should fail for intermediate
        assert!(preferences.set_automation_level(AutomationLevel::FullAuto, AutomationScope::Both).is_err());
        
        // Should work for advanced
        preferences.experience_level = ExperienceLevel::Advanced;
        assert!(preferences.set_automation_level(AutomationLevel::FullAuto, AutomationScope::Both).is_ok());
    }
    
    #[test]
    fn test_feature_access() {
        let mut preferences = UserTradingPreferences::new_default("test_user".to_string());
        preferences.experience_level = ExperienceLevel::Intermediate;
        preferences.technical_enabled = true;
        preferences.automation_level = AutomationLevel::SemiAuto;
        preferences.automation_scope = AutomationScope::ArbitrageOnly;
        
        let access = FeatureAccess::from_preferences(&preferences);
        
        assert!(access.arbitrage_alerts);
        assert!(access.technical_alerts);
        assert!(access.arbitrage_automation);
        assert!(!access.technical_automation); // Technical not in scope
        assert!(access.ai_integration); // Intermediate level
        assert!(access.custom_indicators); // Technical enabled + not beginner
    }
    
    #[test]
    #[ignore] // Requires proper environment setup 
    fn test_preference_validation() {
        // This test requires proper environment setup
        // TODO: Implement proper test environment setup
        /*
        let service = UserTradingPreferencesService::new(
            D1Service::new(Env::default()),
            Logger::new("test".to_string(), LogLevel::Info),
        );
        
        let mut preferences = UserTradingPreferences::new_default("test_user".to_string());
        
        // Valid default preferences
        let validation = service.validate_preferences(&preferences);
        assert!(validation.is_valid);
        
        // Invalid: beginner with technical trading
        preferences.technical_enabled = true;
        let validation = service.validate_preferences(&preferences);
        assert!(!validation.is_valid);
        assert!(!validation.errors.is_empty());
        
        // Fix by upgrading experience
        preferences.experience_level = ExperienceLevel::Intermediate;
        let validation = service.validate_preferences(&preferences);
        assert!(validation.is_valid);
        */
    }
} 