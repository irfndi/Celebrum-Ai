use crate::services::core::infrastructure::d1_database::D1Service;
use crate::types::{UserProfile, SubscriptionTier, UserRole};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationCode {
    pub id: String,
    pub code: String,
    pub created_by_admin_id: String,
    pub used_by_user_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationUsage {
    pub invitation_id: String,
    pub user_id: String,
    pub telegram_id: i64,
    pub used_at: DateTime<Utc>,
    pub beta_expires_at: DateTime<Utc>,
}

pub struct InvitationService {
    d1_service: D1Service,
}

impl InvitationService {
    pub fn new(d1_service: D1Service) -> Self {
        Self { d1_service }
    }

    /// Generate a new invitation code (Super Admin only)
    pub async fn generate_invitation_code(&self, admin_user_id: &str) -> Result<InvitationCode> {
        // Verify admin has permission to generate codes
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(anyhow!("Unauthorized: Only super admins can generate invitation codes"));
        }

        let invitation_code = InvitationCode {
            id: Uuid::new_v4().to_string(),
            code: self.generate_unique_code().await?,
            created_by_admin_id: admin_user_id.to_string(),
            used_by_user_id: None,
            expires_at: Utc::now() + Duration::days(30), // Invitation codes expire in 30 days
            created_at: Utc::now(),
            used_at: None,
            is_active: true,
        };

        self.store_invitation_code(&invitation_code).await?;
        Ok(invitation_code)
    }

    /// Generate multiple invitation codes at once
    pub async fn generate_multiple_codes(&self, admin_user_id: &str, count: u32) -> Result<Vec<InvitationCode>> {
        if count > 100 {
            return Err(anyhow!("Cannot generate more than 100 codes at once"));
        }

        let mut codes = Vec::new();
        for _ in 0..count {
            let code = self.generate_invitation_code(admin_user_id).await?;
            codes.push(code);
        }
        Ok(codes)
    }

    /// Validate and use an invitation code during user registration
    pub async fn use_invitation_code(&self, code: &str, user_id: &str, telegram_id: i64) -> Result<InvitationUsage> {
        // Find the invitation code
        let invitation = self.find_invitation_by_code(code).await?;
        
        // Validate the code
        self.validate_invitation_code(&invitation)?;

        // Mark code as used with provided user_id
        let beta_expires_at = Utc::now() + Duration::days(90); // 90-day beta period
        
        let usage = InvitationUsage {
            invitation_id: invitation.id.clone(),
            user_id: user_id.to_string(),
            telegram_id,
            used_at: Utc::now(),
            beta_expires_at,
        };

        self.mark_invitation_used(&invitation.id, user_id).await?;
        
        // Convert to D1Service InvitationUsage struct
        let d1_usage = crate::services::core::infrastructure::d1_database::InvitationUsage {
            invitation_id: usage.invitation_id.clone(),
            user_id: usage.user_id.clone(),
            telegram_id: usage.telegram_id,
            used_at: usage.used_at,
            beta_expires_at: usage.beta_expires_at,
        };
        
        self.d1_service.store_invitation_usage(&d1_usage).await
            .map_err(|e| anyhow!("Failed to store invitation usage: {}", e))?;

        Ok(usage)
    }

    /// Check if an invitation code is valid
    pub async fn validate_invitation_code(&self, invitation: &InvitationCode) -> Result<()> {
        if !invitation.is_active {
            return Err(anyhow!("Invitation code is inactive"));
        }

        if invitation.used_by_user_id.is_some() {
            return Err(anyhow!("Invitation code has already been used"));
        }

        if Utc::now() > invitation.expires_at {
            return Err(anyhow!("Invitation code has expired"));
        }

        Ok(())
    }

    /// Get invitation code statistics for admin dashboard
    pub async fn get_invitation_statistics(&self, admin_user_id: &str) -> Result<InvitationStatistics> {
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(anyhow!("Unauthorized: Only super admins can view invitation statistics"));
        }

        let total_generated = self.count_total_invitations().await?;
        let total_used = self.count_used_invitations().await?;
        let total_expired = self.count_expired_invitations().await?;
        let active_beta_users = self.count_active_beta_users().await?;

        Ok(InvitationStatistics {
            total_generated,
            total_used,
            total_expired,
            active_beta_users,
            conversion_rate: if total_generated > 0 { 
                (total_used as f64 / total_generated as f64) * 100.0 
            } else { 
                0.0 
            },
        })
    }

    /// List all invitation codes created by an admin
    pub async fn list_admin_invitations(&self, admin_user_id: &str, limit: Option<u32>) -> Result<Vec<InvitationCode>> {
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(anyhow!("Unauthorized: Only super admins can list invitation codes"));
        }

        self.get_invitations_by_admin(admin_user_id, limit.unwrap_or(50)).await
    }

    /// Check if a user's beta access has expired and needs downgrade
    pub async fn check_beta_expiration(&self, user_id: &str) -> Result<bool> {
        if let Some(d1_usage) = self.d1_service.get_invitation_usage_by_user(user_id).await
            .map_err(|e| anyhow!("Failed to get invitation usage: {}", e))? {
            Ok(Utc::now() > d1_usage.beta_expires_at)
        } else {
            Ok(false) // User wasn't invited via invitation code
        }
    }

    /// Get beta expiration date for a user
    pub async fn get_beta_expiration(&self, user_id: &str) -> Result<Option<DateTime<Utc>>> {
        if let Some(d1_usage) = self.d1_service.get_invitation_usage_by_user(user_id).await
            .map_err(|e| anyhow!("Failed to get invitation usage: {}", e))? {
            Ok(Some(d1_usage.beta_expires_at))
        } else {
            Ok(None)
        }
    }

    // Private helper methods

    async fn generate_unique_code(&self) -> Result<String> {
        loop {
            let code = self.generate_random_code();
            if !self.code_exists(&code).await? {
                return Ok(code);
            }
        }
    }

    fn generate_random_code(&self) -> String {
        use rand::{Rng, rngs::OsRng};
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = OsRng;
        
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    async fn verify_admin_permission(&self, user_id: &str) -> Result<bool> {
        // Use subscription_tier as the single source of truth for admin status
        let query = "SELECT subscription_tier, profile_metadata FROM user_profiles WHERE user_id = ?";
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            // Primary check: subscription tier (single source of truth)
            if let Some(tier_str) = row.get("subscription_tier") {
                if tier_str == "SuperAdmin" {
                    return Ok(true);
                }
            }
            
            // Fallback check: role in metadata (with proper error logging)
            if let Some(metadata_str) = row.get("profile_metadata") {
                match serde_json::from_str::<serde_json::Value>(metadata_str) {
                    Ok(metadata) => {
                        if let Some(role) = metadata.get("role") {
                            if role == "SuperAdmin" {
                                // Log inconsistency between subscription_tier and metadata
                                eprintln!("⚠️ Admin permission inconsistency for user {}: metadata role is SuperAdmin but subscription_tier is not", user_id);
                                return Ok(true);
                            }
                        }
                    }
                    Err(e) => {
                        // Log JSON parsing error for debugging
                        eprintln!("❌ Failed to parse profile_metadata JSON for user {}: {}", user_id, e);
                    }
                }
            }
        }
        
        Ok(false)
    }

    async fn code_exists(&self, code: &str) -> Result<bool> {
        let query = "SELECT COUNT(*) as count FROM invitation_codes WHERE code = ?";
        let result = self.d1_service.query(query, &[code.into()]).await?;
        
        if let Some(row) = result.first() {
            if let Some(count_str) = row.get("count") {
                return Ok(count_str.parse::<i32>().unwrap_or(0) > 0);
            }
        }
        
        Ok(false)
    }

    async fn store_invitation_code(&self, invitation: &InvitationCode) -> Result<()> {
        let query = r#"
            INSERT INTO invitation_codes 
            (id, code, created_by_admin_id, expires_at, created_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;
        
        self.d1_service.execute(query, &[
            invitation.id.clone().into(),
            invitation.code.clone().into(),
            invitation.created_by_admin_id.clone().into(),
            invitation.expires_at.to_rfc3339().into(),
            invitation.created_at.to_rfc3339().into(),
            invitation.is_active.into(),
        ]).await?;
        
        Ok(())
    }

    async fn find_invitation_by_code(&self, code: &str) -> Result<InvitationCode> {
        let query = r#"
            SELECT id, code, created_by_admin_id, used_by_user_id, expires_at, created_at, used_at, is_active
            FROM invitation_codes 
            WHERE code = ?
        "#;
        
        let result = self.d1_service.query(query, &[code.into()]).await?;
        
        if let Some(row) = result.first() {
            // Explicitly check required fields and return errors if missing
            let id = row.get("id")
                .ok_or_else(|| anyhow!("Missing required field: id"))?;
            
            let code = row.get("code")
                .ok_or_else(|| anyhow!("Missing required field: code"))?;
            
            let created_by_admin_id = row.get("created_by_admin_id")
                .ok_or_else(|| anyhow!("Missing required field: created_by_admin_id"))?;
            
            let expires_at_str = row.get("expires_at")
                .ok_or_else(|| anyhow!("Missing required field: expires_at"))?;
            
            let created_at_str = row.get("created_at")
                .ok_or_else(|| anyhow!("Missing required field: created_at"))?;
            
            let is_active_str = row.get("is_active")
                .ok_or_else(|| anyhow!("Missing required field: is_active"))?;
            
            // Parse required fields with proper error handling
            let expires_at = DateTime::parse_from_rfc3339(expires_at_str)
                .map_err(|e| anyhow!("Invalid expires_at format '{}': {}", expires_at_str, e))?
                .with_timezone(&Utc);
            
            let created_at = DateTime::parse_from_rfc3339(created_at_str)
                .map_err(|e| anyhow!("Invalid created_at format '{}': {}", created_at_str, e))?
                .with_timezone(&Utc);
            
            let is_active = is_active_str.parse::<bool>()
                .map_err(|e| anyhow!("Invalid is_active format '{}': {}", is_active_str, e))?;
            
            // Optional fields can use safe defaults
            let used_by_user_id = row.get("used_by_user_id");
            let used_at = row.get("used_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            
            Ok(InvitationCode {
                id,
                code,
                created_by_admin_id,
                used_by_user_id,
                expires_at,
                created_at,
                used_at,
                is_active,
            })
        } else {
            Err(anyhow!("Invitation code not found"))
        }
    }

    async fn mark_invitation_used(&self, invitation_id: &str, user_id: &str) -> Result<()> {
        let query = r#"
            UPDATE invitation_codes 
            SET used_by_user_id = ?, used_at = ?, is_active = false
            WHERE id = ?
        "#;
        
        self.d1_service.execute(query, &[
            user_id.into(),
            Utc::now().to_rfc3339().into(),
            invitation_id.into(),
        ]).await?;
        
        Ok(())
    }

    async fn count_total_invitations(&self) -> Result<u32> {
        let query = "SELECT COUNT(*) as count FROM invitation_codes";
        let result = self.d1_service.query(query, &[]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn count_used_invitations(&self) -> Result<u32> {
        let query = "SELECT COUNT(*) as count FROM invitation_codes WHERE used_by_user_id IS NOT NULL";
        let result = self.d1_service.query(query, &[]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn count_expired_invitations(&self) -> Result<u32> {
        let query = "SELECT COUNT(*) as count FROM invitation_codes WHERE expires_at < datetime('now') AND used_by_user_id IS NULL";
        let result = self.d1_service.query(query, &[]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn count_active_beta_users(&self) -> Result<u32> {
        let query = "SELECT COUNT(*) as count FROM invitation_usage WHERE beta_expires_at > datetime('now')";
        let result = self.d1_service.query(query, &[]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn get_invitations_by_admin(&self, admin_user_id: &str, limit: u32) -> Result<Vec<InvitationCode>> {
        let query = r#"
            SELECT id, code, created_by_admin_id, used_by_user_id, expires_at, created_at, used_at, is_active
            FROM invitation_codes 
            WHERE created_by_admin_id = ?
            ORDER BY created_at DESC
            LIMIT ?
        "#;
        
        let result = self.d1_service.query(query, &[admin_user_id.into(), limit.to_string().into()]).await?;
        
        let mut invitations = Vec::new();
        for row in result {
            invitations.push(InvitationCode {
                id: row.get("id").unwrap_or_default(),
                code: row.get("code").unwrap_or_default(),
                created_by_admin_id: row.get("created_by_admin_id").unwrap_or_default(),
                used_by_user_id: row.get("used_by_user_id"),
                expires_at: DateTime::parse_from_rfc3339(row.get("expires_at").unwrap_or_default())
                    .map_err(|e| anyhow!("Invalid expires_at format: {}", e))?
                    .with_timezone(&Utc),
                created_at: DateTime::parse_from_rfc3339(row.get("created_at").unwrap_or_default())
                    .map_err(|e| anyhow!("Invalid created_at format: {}", e))?
                    .with_timezone(&Utc),
                used_at: row.get("used_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                is_active: row.get("is_active").unwrap_or("false").parse().unwrap_or(false),
            });
        }
        
        Ok(invitations)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationStatistics {
    pub total_generated: u32,
    pub total_used: u32,
    pub total_expired: u32,
    pub active_beta_users: u32,
    pub conversion_rate: f64,
} 