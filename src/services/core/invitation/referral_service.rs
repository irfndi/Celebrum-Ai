use crate::services::core::infrastructure::d1_database::D1Service;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReferralCode {
    pub id: String,
    pub user_id: String,
    pub referral_code: String, // User's personal referral code (initially randomized, user can update)
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_uses: u32,
    pub total_bonuses_earned: f64,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralUsage {
    pub id: String,
    pub referrer_user_id: String,
    pub referred_user_id: String,
    pub referral_code: String,
    pub used_at: DateTime<Utc>,
    pub bonus_awarded: f64,
    pub bonus_type: ReferralBonusType,
    pub conversion_status: ConversionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferralBonusType {
    FeatureAccess,    // Limited feature access bonus
    RevenueKickback,  // Revenue sharing bonus
    Points,           // Points system bonus
    SubscriptionDiscount, // Discount on subscription
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionStatus {
    Registered,       // User registered with referral code
    FirstTrade,       // User made their first trade
    Subscribed,       // User upgraded to paid subscription
    ActiveUser,       // User is actively using the platform
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralStatistics {
    pub user_id: String,
    pub total_referrals: u32,
    pub successful_conversions: u32,
    pub total_bonuses_earned: f64,
    pub conversion_rate: f64,
    pub rank_position: Option<u32>,
    pub monthly_referrals: u32,
    pub monthly_bonuses: f64,
}

pub struct ReferralService {
    d1_service: D1Service,
}

impl ReferralService {
    pub fn new(d1_service: D1Service) -> Self {
        Self { d1_service }
    }

    /// Create a new referral code for a user (automatically called during user registration)
    pub async fn create_user_referral_code(&self, user_id: &str) -> Result<UserReferralCode> {
        // Check if user already has a referral code
        if let Ok(existing) = self.get_user_referral_code(user_id).await {
            return Ok(existing);
        }

        let referral_code = UserReferralCode {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            referral_code: self.generate_unique_referral_code().await?,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            total_uses: 0,
            total_bonuses_earned: 0.0,
            last_used_at: None,
        };

        self.store_referral_code(&referral_code).await?;
        Ok(referral_code)
    }

    /// Read/Get user's referral code
    pub async fn get_user_referral_code(&self, user_id: &str) -> Result<UserReferralCode> {
        let query = r#"
            SELECT id, user_id, referral_code, is_active, created_at, updated_at, 
                   total_uses, total_bonuses_earned, last_used_at
            FROM user_referral_codes 
            WHERE user_id = ?
        "#;
        
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(UserReferralCode {
                id: row.get("id").unwrap_or_default(),
                user_id: row.get("user_id").unwrap_or_default(),
                referral_code: row.get("referral_code").unwrap_or_default(),
                is_active: row.get("is_active").unwrap_or("false").parse().unwrap_or(false),
                created_at: DateTime::parse_from_rfc3339(row.get("created_at").unwrap_or_default())
                    .map_err(|e| anyhow!("Invalid created_at format: {}", e))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at").unwrap_or_default())
                    .map_err(|e| anyhow!("Invalid updated_at format: {}", e))?
                    .with_timezone(&Utc),
                total_uses: row.get("total_uses").unwrap_or("0").parse().unwrap_or(0),
                total_bonuses_earned: row.get("total_bonuses_earned").unwrap_or("0.0").parse().unwrap_or(0.0),
                last_used_at: row.get("last_used_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        } else {
            Err(anyhow!("Referral code not found for user"))
        }
    }

    /// Update user's referral code (user can customize their code)
    pub async fn update_user_referral_code(&self, user_id: &str, new_code: &str) -> Result<UserReferralCode> {
        // Validate new code format
        if !self.is_valid_referral_code_format(new_code) {
            return Err(anyhow!("Invalid referral code format. Must be 6-12 alphanumeric characters."));
        }

        // Check if code is already taken
        if self.referral_code_exists(new_code).await? {
            return Err(anyhow!("Referral code already taken. Please choose a different code."));
        }

        let query = r#"
            UPDATE user_referral_codes 
            SET referral_code = ?, updated_at = ?
            WHERE user_id = ?
        "#;
        
        self.d1_service.execute(query, &[
            new_code.into(),
            Utc::now().to_rfc3339().into(),
            user_id.into(),
        ]).await?;

        // Return updated referral code
        self.get_user_referral_code(user_id).await
    }

    /// Use a referral code (when a new user registers with someone's referral code)
    pub async fn use_referral_code(&self, referral_code: &str, new_user_id: &str) -> Result<ReferralUsage> {
        // Find the referrer
        let referrer = self.find_user_by_referral_code(referral_code).await?;
        
        // Create referral usage record
        let usage = ReferralUsage {
            id: Uuid::new_v4().to_string(),
            referrer_user_id: referrer.user_id.clone(),
            referred_user_id: new_user_id.to_string(),
            referral_code: referral_code.to_string(),
            used_at: Utc::now(),
            bonus_awarded: self.calculate_referral_bonus(&ReferralBonusType::FeatureAccess).await?,
            bonus_type: ReferralBonusType::FeatureAccess,
            conversion_status: ConversionStatus::Registered,
        };

        // Store usage record
        self.store_referral_usage(&usage).await?;
        
        // Update referrer's statistics
        self.update_referrer_statistics(&referrer.user_id).await?;

        Ok(usage)
    }

    /// Get referral statistics for a user
    pub async fn get_user_referral_statistics(&self, user_id: &str) -> Result<ReferralStatistics> {
        let total_referrals = self.count_user_referrals(user_id).await?;
        let successful_conversions = self.count_successful_conversions(user_id).await?;
        let total_bonuses_earned = self.calculate_total_bonuses_earned(user_id).await?;
        let conversion_rate = if total_referrals > 0 {
            (successful_conversions as f64 / total_referrals as f64) * 100.0
        } else {
            0.0
        };
        let rank_position = self.get_user_rank_position(user_id).await?;
        let monthly_referrals = self.count_monthly_referrals(user_id).await?;
        let monthly_bonuses = self.calculate_monthly_bonuses(user_id).await?;

        Ok(ReferralStatistics {
            user_id: user_id.to_string(),
            total_referrals,
            successful_conversions,
            total_bonuses_earned,
            conversion_rate,
            rank_position,
            monthly_referrals,
            monthly_bonuses,
        })
    }

    /// Get referral leaderboard
    pub async fn get_referral_leaderboard(&self, limit: u32) -> Result<Vec<ReferralStatistics>> {
        let query = r#"
            SELECT user_id, COUNT(*) as total_referrals, SUM(bonus_awarded) as total_bonuses
            FROM referral_usage 
            GROUP BY user_id 
            ORDER BY total_referrals DESC, total_bonuses DESC
            LIMIT ?
        "#;
        
        let result = self.d1_service.query(query, &[limit.to_string().into()]).await?;
        
        let mut leaderboard = Vec::new();
        for (index, row) in result.iter().enumerate() {
            let user_id = row.get("user_id").unwrap_or_default();
            let total_referrals: u32 = row.get("total_referrals").unwrap_or("0").parse().unwrap_or(0);
            let total_bonuses: f64 = row.get("total_bonuses").unwrap_or("0.0").parse().unwrap_or(0.0);
            
            leaderboard.push(ReferralStatistics {
                user_id,
                total_referrals,
                successful_conversions: 0, // Would need additional query
                total_bonuses_earned: total_bonuses,
                conversion_rate: 0.0, // Would need additional calculation
                rank_position: Some((index + 1) as u32),
                monthly_referrals: 0, // Would need additional query
                monthly_bonuses: 0.0, // Would need additional query
            });
        }
        
        Ok(leaderboard)
    }

    /// Award bonus for referral milestone (e.g., when referred user makes first trade)
    pub async fn award_referral_bonus(
        &self, 
        referrer_user_id: &str, 
        referred_user_id: &str, 
        bonus_type: ReferralBonusType,
        conversion_status: ConversionStatus
    ) -> Result<f64> {
        let bonus_amount = self.calculate_referral_bonus(&bonus_type).await?;
        
        // Update existing referral usage record or create new bonus record
        let query = r#"
            UPDATE referral_usage 
            SET bonus_awarded = bonus_awarded + ?, bonus_type = ?, conversion_status = ?
            WHERE referrer_user_id = ? AND referred_user_id = ?
        "#;
        
        self.d1_service.execute(query, &[
            bonus_amount.into(),
            format!("{:?}", bonus_type).into(),
            format!("{:?}", conversion_status).into(),
            referrer_user_id.into(),
            referred_user_id.into(),
        ]).await?;

        // Update referrer's total bonuses
        self.update_referrer_statistics(referrer_user_id).await?;

        Ok(bonus_amount)
    }

    // Private helper methods

    async fn generate_unique_referral_code(&self) -> Result<String> {
        loop {
            let code = self.generate_random_referral_code();
            if !self.referral_code_exists(&code).await? {
                return Ok(code);
            }
        }
    }

    fn generate_random_referral_code(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn is_valid_referral_code_format(&self, code: &str) -> bool {
        code.len() >= 6 && code.len() <= 12 && code.chars().all(|c| c.is_alphanumeric())
    }

    async fn referral_code_exists(&self, code: &str) -> Result<bool> {
        let query = "SELECT COUNT(*) as count FROM user_referral_codes WHERE referral_code = ?";
        let result = self.d1_service.query(query, &[code.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse::<i32>().unwrap_or(0) > 0)
        } else {
            Ok(false)
        }
    }

    async fn store_referral_code(&self, referral_code: &UserReferralCode) -> Result<()> {
        let query = r#"
            INSERT INTO user_referral_codes 
            (id, user_id, referral_code, is_active, created_at, updated_at, total_uses, total_bonuses_earned)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        self.d1_service.execute(query, &[
            referral_code.id.clone().into(),
            referral_code.user_id.clone().into(),
            referral_code.referral_code.clone().into(),
            referral_code.is_active.into(),
            referral_code.created_at.to_rfc3339().into(),
            referral_code.updated_at.to_rfc3339().into(),
            referral_code.total_uses.into(),
            referral_code.total_bonuses_earned.into(),
        ]).await?;
        
        Ok(())
    }

    async fn find_user_by_referral_code(&self, referral_code: &str) -> Result<UserReferralCode> {
        let query = r#"
            SELECT id, user_id, referral_code, is_active, created_at, updated_at, 
                   total_uses, total_bonuses_earned, last_used_at
            FROM user_referral_codes 
            WHERE referral_code = ? AND is_active = true
        "#;
        
        let result = self.d1_service.query(query, &[referral_code.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(UserReferralCode {
                id: row.get("id").unwrap_or_default(),
                user_id: row.get("user_id").unwrap_or_default(),
                referral_code: row.get("referral_code").unwrap_or_default(),
                is_active: row.get("is_active").unwrap_or("false").parse().unwrap_or(false),
                created_at: DateTime::parse_from_rfc3339(row.get("created_at").unwrap_or_default())
                    .map_err(|e| anyhow!("Invalid created_at format: {}", e))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at").unwrap_or_default())
                    .map_err(|e| anyhow!("Invalid updated_at format: {}", e))?
                    .with_timezone(&Utc),
                total_uses: row.get("total_uses").unwrap_or("0").parse().unwrap_or(0),
                total_bonuses_earned: row.get("total_bonuses_earned").unwrap_or("0.0").parse().unwrap_or(0.0),
                last_used_at: row.get("last_used_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        } else {
            Err(anyhow!("Referral code not found or inactive"))
        }
    }

    async fn store_referral_usage(&self, usage: &ReferralUsage) -> Result<()> {
        let query = r#"
            INSERT INTO referral_usage 
            (id, referrer_user_id, referred_user_id, referral_code, used_at, bonus_awarded, bonus_type, conversion_status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        self.d1_service.execute(query, &[
            usage.id.clone().into(),
            usage.referrer_user_id.clone().into(),
            usage.referred_user_id.clone().into(),
            usage.referral_code.clone().into(),
            usage.used_at.to_rfc3339().into(),
            usage.bonus_awarded.into(),
            format!("{:?}", usage.bonus_type).into(),
            format!("{:?}", usage.conversion_status).into(),
        ]).await?;
        
        Ok(())
    }

    async fn update_referrer_statistics(&self, user_id: &str) -> Result<()> {
        let query = r#"
            UPDATE user_referral_codes 
            SET total_uses = (
                SELECT COUNT(*) FROM referral_usage WHERE referrer_user_id = ?
            ),
            total_bonuses_earned = (
                SELECT COALESCE(SUM(bonus_awarded), 0) FROM referral_usage WHERE referrer_user_id = ?
            ),
            last_used_at = (
                SELECT MAX(used_at) FROM referral_usage WHERE referrer_user_id = ?
            ),
            updated_at = ?
            WHERE user_id = ?
        "#;
        
        self.d1_service.execute(query, &[
            user_id.into(),
            user_id.into(),
            user_id.into(),
            Utc::now().to_rfc3339().into(),
            user_id.into(),
        ]).await?;
        
        Ok(())
    }

    // Placeholder methods for statistics calculation
    async fn count_user_referrals(&self, user_id: &str) -> Result<u32> {
        let query = "SELECT COUNT(*) as count FROM referral_usage WHERE referrer_user_id = ?";
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn count_successful_conversions(&self, user_id: &str) -> Result<u32> {
        let query = r#"
            SELECT COUNT(*) as count FROM referral_usage 
            WHERE referrer_user_id = ? AND conversion_status IN ('Subscribed', 'ActiveUser')
        "#;
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn calculate_total_bonuses_earned(&self, user_id: &str) -> Result<f64> {
        let query = "SELECT COALESCE(SUM(bonus_awarded), 0) as total FROM referral_usage WHERE referrer_user_id = ?";
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("total").unwrap_or("0.0").parse().unwrap_or(0.0))
        } else {
            Ok(0.0)
        }
    }

    async fn get_user_rank_position(&self, user_id: &str) -> Result<Option<u32>> {
        // This would require a more complex query to rank users
        // For now, return None as placeholder
        Ok(None)
    }

    async fn count_monthly_referrals(&self, user_id: &str) -> Result<u32> {
        let query = r#"
            SELECT COUNT(*) as count FROM referral_usage 
            WHERE referrer_user_id = ? AND used_at >= datetime('now', '-1 month')
        "#;
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("count").unwrap_or("0").parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn calculate_monthly_bonuses(&self, user_id: &str) -> Result<f64> {
        let query = r#"
            SELECT COALESCE(SUM(bonus_awarded), 0) as total FROM referral_usage 
            WHERE referrer_user_id = ? AND used_at >= datetime('now', '-1 month')
        "#;
        let result = self.d1_service.query(query, &[user_id.into()]).await?;
        
        if let Some(row) = result.first() {
            Ok(row.get("total").unwrap_or("0.0").parse().unwrap_or(0.0))
        } else {
            Ok(0.0)
        }
    }

    async fn calculate_referral_bonus(&self, bonus_type: &ReferralBonusType) -> Result<f64> {
        match bonus_type {
            ReferralBonusType::FeatureAccess => Ok(0.0), // No monetary value, just feature access
            ReferralBonusType::RevenueKickback => Ok(5.0), // $5 revenue kickback
            ReferralBonusType::Points => Ok(100.0), // 100 points
            ReferralBonusType::SubscriptionDiscount => Ok(10.0), // $10 discount value
        }
    }
} 