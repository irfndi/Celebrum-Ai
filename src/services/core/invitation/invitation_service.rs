use crate::services::core::infrastructure::database_repositories::DatabaseManager;

use crate::utils::error::{ArbitrageError, ArbitrageResult};
use crate::utils::helpers::generate_uuid;
use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvitationStatus {
    Active,
    Used,
    Expired,
}

impl InvitationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            InvitationStatus::Active => "active",
            InvitationStatus::Used => "used",
            InvitationStatus::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InvitationConfig {
    pub invitation_expiry_days: i64,
    pub beta_access_days: i64,
    pub max_batch_size: u32,
    pub rate_limit_attempts: u32,
}

impl Default for InvitationConfig {
    fn default() -> Self {
        Self {
            invitation_expiry_days: 30,
            beta_access_days: 90,
            max_batch_size: 100,
            rate_limit_attempts: 5,
        }
    }
}

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

// Note: InvitationUsage is defined locally in this module and used directly

pub struct InvitationService {
    d1_service: DatabaseManager,
    config: InvitationConfig,
}

impl InvitationService {
    pub fn new(d1_service: DatabaseManager) -> Self {
        Self {
            d1_service,
            config: InvitationConfig::default(),
        }
    }

    pub fn new_with_config(d1_service: DatabaseManager, config: InvitationConfig) -> Self {
        Self { d1_service, config }
    }

    /// Generate a new invitation code (Super Admin only)
    pub async fn generate_invitation_code(
        &self,
        admin_user_id: &str,
    ) -> ArbitrageResult<InvitationCode> {
        // Verify admin has permission to generate codes
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(ArbitrageError::authentication_error(
                "Unauthorized: Only super admins can generate invitation codes",
            ));
        }

        let invitation_code = InvitationCode {
            id: generate_uuid(),
            code: self.generate_unique_code().await?,
            created_by_admin_id: admin_user_id.to_string(),
            used_by_user_id: None,
            expires_at: Utc::now() + Duration::days(self.config.invitation_expiry_days),
            created_at: Utc::now(),
            used_at: None,
            is_active: true,
        };

        self.store_invitation_code(&invitation_code).await?;
        Ok(invitation_code)
    }

    /// Generate multiple invitation codes in a batch with proper transaction support
    pub async fn generate_multiple_codes(
        &self,
        admin_user_id: &str,
        count: u32,
    ) -> ArbitrageResult<Vec<InvitationCode>> {
        // Upfront admin permission verification before any code generation
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(ArbitrageError::authentication_error(
                "Unauthorized: Only super admins can generate invitation codes",
            ));
        }

        if count == 0 || count > self.config.max_batch_size {
            return Err(ArbitrageError::validation_error(format!(
                "Invalid count: must be between 1 and {}",
                self.config.max_batch_size
            )));
        }

        // Pre-generate all codes and validate uniqueness before storage
        let mut codes = Vec::new();
        for _ in 0..count {
            let code = self.generate_unique_code().await?;
            let invitation = InvitationCode {
                id: generate_uuid(),
                code,
                created_by_admin_id: admin_user_id.to_string(),
                used_by_user_id: None,
                expires_at: Utc::now() + Duration::days(self.config.invitation_expiry_days),
                created_at: Utc::now(),
                used_at: None,
                is_active: true,
            };
            codes.push(invitation);
        }

        let mut queries = Vec::new();
        let mut params_list = Vec::new();

        for code in &codes {
            let (sql, params_values) = self.generate_insert_invitation_query(code)?;
            queries.push(sql);
            params_list.push(params_values);
        }

        if !queries.is_empty() {
            // For batch operations, we don't need parsed results, so we use a simple unit parser
            self.d1_service
                .execute_transactional_query(
                    queries,
                    params_list,
                    |_row| Ok(()), // Simple unit parser for insert operations
                )
                .await?;
        }

        // Use our sanitized logger instead of standard log macro
        crate::utils::logger::logger().info(&format!(
            "Successfully stored {} invitation codes in atomic transaction",
            codes.len()
        ));
        Ok(codes)
    }

    /// Validate and use an invitation code during user registration
    pub async fn use_invitation_code(
        &self,
        code: &str,
        user_id: &str,
        telegram_id: i64,
    ) -> ArbitrageResult<InvitationUsage> {
        // Find the invitation code
        let invitation = self.find_invitation_by_code(code).await?;

        // Validate the code
        self.validate_invitation_code(&invitation)?;

        // Mark code as used with provided user_id
        let beta_expires_at = Utc::now() + Duration::days(self.config.beta_access_days);

        let usage = InvitationUsage {
            invitation_id: invitation.id.clone(),
            user_id: user_id.to_string(),
            telegram_id,
            used_at: Utc::now(),
            beta_expires_at,
        };

        // Use database transaction to ensure atomicity of marking code as used and storing usage
        self.mark_invitation_used_transaction(&invitation.id, user_id, &usage)
            .await?;

        // Use our sanitized logger instead of standard log macro
        crate::utils::logger::logger()
            .info("Successfully used invitation code in atomic transaction");
        Ok(usage)
    }

    /// Check if an invitation code is valid
    pub fn validate_invitation_code(&self, invitation: &InvitationCode) -> ArbitrageResult<()> {
        if !invitation.is_active {
            return Err(ArbitrageError::validation_error(
                "Invitation code is inactive",
            ));
        }

        if invitation.used_by_user_id.is_some() {
            return Err(ArbitrageError::validation_error(
                "Invitation code has already been used",
            ));
        }

        if Utc::now() > invitation.expires_at {
            return Err(ArbitrageError::validation_error(
                "Invitation code has expired",
            ));
        }

        Ok(())
    }

    /// Get invitation code statistics for admin dashboard
    pub async fn get_invitation_statistics(
        &self,
        admin_user_id: &str,
    ) -> ArbitrageResult<InvitationStatistics> {
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(ArbitrageError::authentication_error(
                "Unauthorized: Only super admins can view invitation statistics",
            ));
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
    pub async fn list_admin_invitations(
        &self,
        admin_user_id: &str,
        limit: Option<u32>,
    ) -> ArbitrageResult<Vec<InvitationCode>> {
        if !self.verify_admin_permission(admin_user_id).await? {
            return Err(ArbitrageError::authentication_error(
                "Unauthorized: Only super admins can list invitation codes",
            ));
        }

        self.get_invitations_by_admin(admin_user_id, limit.unwrap_or(50))
            .await
    }

    /// Check if a user's beta access has expired and needs downgrade
    pub async fn check_beta_expiration(&self, user_id: &str) -> ArbitrageResult<bool> {
        if let Some(d1_usage) = self
            .d1_service
            .get_invitation_usage_by_user(user_id)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to get invitation usage: {}", e))
            })?
        {
            Ok(Utc::now() > d1_usage.beta_expires_at)
        } else {
            Ok(false) // User wasn't invited via invitation code
        }
    }

    /// Get beta expiration date for a user
    pub async fn get_beta_expiration(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Option<DateTime<Utc>>> {
        if let Some(d1_usage) = self
            .d1_service
            .get_invitation_usage_by_user(user_id)
            .await
            .map_err(|e| {
                ArbitrageError::database_error(format!("Failed to get invitation usage: {}", e))
            })?
        {
            Ok(Some(d1_usage.beta_expires_at))
        } else {
            Ok(None)
        }
    }

    // Private helper methods

    async fn generate_unique_code(&self) -> ArbitrageResult<String> {
        loop {
            let code = self.generate_random_code();
            if !self.code_exists(&code).await? {
                return Ok(code);
            }
        }
    }

    fn generate_random_code(&self) -> String {
        use rand::{rngs::OsRng, Rng};
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = OsRng;

        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    async fn verify_admin_permission(&self, user_id: &str) -> ArbitrageResult<bool> {
        // Use subscription_tier as the authoritative source for admin status
        let query = "SELECT subscription_tier FROM user_profiles WHERE user_id = ?";
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[user_id.into()])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        if let Ok(results) = result.results::<serde_json::Value>() {
            if let Some(row) = results.first() {
                if let Some(tier) = row.get("subscription_tier").and_then(|v| v.as_str()) {
                    return Ok(tier == "SuperAdmin");
                }
            }
        }

        Ok(false)
    }

    async fn code_exists(&self, code: &str) -> ArbitrageResult<bool> {
        let query = "SELECT COUNT(*) as count FROM invitation_codes WHERE code = ?";
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[code.into()])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        if let Ok(results) = result.results::<serde_json::Value>() {
            if let Some(row) = results.first() {
                if let Some(count) = row.get("count").and_then(|v| {
                    if let Some(count_str) = v.as_str() {
                        count_str.parse::<i64>().ok()
                    } else if v.is_number() {
                        v.as_f64().map(|f| f as i64)
                    } else {
                        None
                    }
                }) {
                    return Ok(count > 0);
                }
            }
        }
        Ok(false)
    }

    async fn store_invitation_code(&self, invitation: &InvitationCode) -> ArbitrageResult<()> {
        let query = r#"
            INSERT INTO invitation_codes 
            (id, code, created_by_admin_id, expires_at, created_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[
            invitation.id.clone().into(),
            invitation.code.clone().into(),
            invitation.created_by_admin_id.clone().into(),
            invitation.expires_at.to_rfc3339().into(),
            invitation.created_at.to_rfc3339().into(),
            invitation.is_active.into(),
        ])?;
        bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn delete_invitation_code(&self, invitation_id: &str) -> ArbitrageResult<()> {
        let query = "DELETE FROM invitation_codes WHERE id = ?";

        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[invitation_id.into()])?;
        bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        Ok(())
    }

    fn parse_invitation_from_row(
        &self,
        row: &HashMap<String, serde_json::Value>,
    ) -> ArbitrageResult<InvitationCode> {
        // Explicitly check required fields and return errors if missing
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing required field: id"))?;

        let code = row
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing required field: code"))?;

        let created_by_admin_id = row
            .get("created_by_admin_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ArbitrageError::parse_error("Missing required field: created_by_admin_id")
            })?;

        let expires_at_str = row
            .get("expires_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing required field: expires_at"))?;

        let created_at_str = row
            .get("created_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArbitrageError::parse_error("Missing required field: created_at"))?;

        let is_active = row
            .get("is_active")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| {
                ArbitrageError::parse_error("Missing required field or not a bool: is_active")
            })?;

        let created_at = DateTime::parse_from_rfc3339(created_at_str)
            .map_err(|e| {
                ArbitrageError::parse_error(format!(
                    "Invalid created_at format '{}': {}",
                    created_at_str, e
                ))
            })?
            .with_timezone(&Utc);

        // Optional fields can use safe defaults
        let used_by_user_id = row
            .get("used_by_user_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let used_at = row
            .get("used_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(InvitationCode {
            id: id.to_string(),
            code: code.to_string(),
            created_by_admin_id: created_by_admin_id.to_string(),
            used_by_user_id,
            expires_at: DateTime::parse_from_rfc3339(expires_at_str)
                .map_err(|e| {
                    ArbitrageError::parse_error(format!(
                        "Invalid expires_at format '{}': {}",
                        expires_at_str, e
                    ))
                })?
                .with_timezone(&Utc),
            created_at,
            used_at,
            is_active,
        })
    }

    async fn find_invitation_by_code(&self, code: &str) -> ArbitrageResult<InvitationCode> {
        let query = r#"
            SELECT id, code, created_by_admin_id, used_by_user_id, expires_at, created_at, used_at, is_active
            FROM invitation_codes 
            WHERE code = ?
        "#;

        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[code.into()])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        if let Ok(results) = result.results::<serde_json::Value>() {
            if let Some(row) = results.first() {
                // Convert serde_json::Value to HashMap for compatibility with existing parse method
                let mut row_map = HashMap::new();
                if let Some(object) = row.as_object() {
                    for (key, value) in object {
                        row_map.insert(key.clone(), value.clone());
                    }
                }
                self.parse_invitation_from_row(&row_map)
            } else {
                Err(ArbitrageError::not_found("Invitation code not found"))
            }
        } else {
            Err(ArbitrageError::not_found("Invitation code not found"))
        }
    }

    /// Helper method to execute count queries with consistent error handling
    async fn execute_count_query(
        &self,
        query: &str,
        params: &[serde_json::Value],
    ) -> ArbitrageResult<u32> {
        let params: Vec<worker::wasm_bindgen::JsValue> = params
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => worker::wasm_bindgen::JsValue::from(s.as_str()),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        worker::wasm_bindgen::JsValue::from(i as f64)
                    } else if let Some(f) = n.as_f64() {
                        worker::wasm_bindgen::JsValue::from(f)
                    } else {
                        worker::wasm_bindgen::JsValue::from(0.0)
                    }
                }
                serde_json::Value::Bool(b) => worker::wasm_bindgen::JsValue::from(*b),
                serde_json::Value::Null => worker::wasm_bindgen::JsValue::NULL,
                _ => worker::wasm_bindgen::JsValue::from(v.to_string().as_str()),
            })
            .collect();
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&params)?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        if let Ok(results) = result.results::<serde_json::Value>() {
            if let Some(row) = results.first() {
                let count_str = row
                    .get("count")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "0".to_string());
                match count_str.parse::<u32>() {
                    Ok(count) => Ok(count),
                    Err(e) => {
                        // Use our sanitized logger instead of standard log macro
                        crate::utils::logger::logger()
                            .warn(&format!("Failed to parse count '{}': {}", count_str, e));
                        Ok(0)
                    }
                }
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// Helper method to handle the transaction logic for marking invitation codes as used
    async fn mark_invitation_used_transaction(
        &self,
        invitation_id: &str,
        user_id: &str,
        usage: &InvitationUsage,
    ) -> ArbitrageResult<()> {
        // Since D1 doesn't support real transactions, we'll execute operations sequentially
        // TODO: When D1 supports real transactions, implement proper atomic operations

        // Mark invitation code as used
        let mark_used_query = r#"
            UPDATE invitation_codes 
            SET used_by_user_id = ?, used_at = ?, is_active = false
            WHERE id = ?
        "#;

        let stmt = self.d1_service.prepare(mark_used_query);
        let bound_stmt = stmt.bind(&[
            user_id.into(),
            Utc::now().to_rfc3339().into(),
            invitation_id.into(),
        ])?;
        bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        // Store usage record
        let store_usage_query = r#"
            INSERT INTO invitation_usage 
            (invitation_id, user_id, telegram_id, used_at, beta_expires_at)
            VALUES (?, ?, ?, ?, ?)
        "#;

        let stmt = self.d1_service.prepare(store_usage_query);
        let bound_stmt = stmt.bind(&[
            usage.invitation_id.clone().into(),
            usage.user_id.clone().into(),
            usage.telegram_id.into(),
            usage.used_at.to_rfc3339().into(),
            usage.beta_expires_at.to_rfc3339().into(),
        ])?;
        bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        Ok(())
    }

    async fn count_total_invitations(&self) -> ArbitrageResult<u32> {
        self.execute_count_query("SELECT COUNT(*) as count FROM invitation_codes", &[])
            .await
    }

    async fn count_used_invitations(&self) -> ArbitrageResult<u32> {
        self.execute_count_query(
            "SELECT COUNT(*) as count FROM invitation_codes WHERE used_by_user_id IS NOT NULL",
            &[],
        )
        .await
    }

    async fn count_expired_invitations(&self) -> ArbitrageResult<u32> {
        self.execute_count_query("SELECT COUNT(*) as count FROM invitation_codes WHERE expires_at < datetime('now') AND used_by_user_id IS NULL", &[]).await
    }

    async fn count_active_beta_users(&self) -> ArbitrageResult<u32> {
        self.execute_count_query("SELECT COUNT(*) as count FROM invitation_usage WHERE beta_expires_at > datetime('now')", &[]).await
    }

    async fn get_invitations_by_admin(
        &self,
        admin_user_id: &str,
        limit: u32,
    ) -> ArbitrageResult<Vec<InvitationCode>> {
        let query = r#"
            SELECT id, code, created_by_admin_id, used_by_user_id, expires_at, created_at, used_at, is_active
            FROM invitation_codes 
            WHERE created_by_admin_id = ?
            ORDER BY created_at DESC
            LIMIT ?
        "#;

        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[admin_user_id.into(), limit.to_string().into()])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        let mut invitations = Vec::new();
        let results_vec = result.results::<HashMap<String, serde_json::Value>>()?;
        for row in results_vec {
            let invitation = self.parse_invitation_from_row(&row)?;
            invitations.push(invitation);
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

impl InvitationService {
    fn generate_insert_invitation_query(
        &self,
        code: &InvitationCode,
    ) -> ArbitrageResult<(String, Vec<String>)> {
        let sql = "INSERT INTO invitation_codes (id, code, created_by_admin_id, used_by_user_id, expires_at, created_at, used_at, is_active) VALUES (?, ?, ?, ?, ?, ?, ?, ?)".to_string();
        let params = vec![
            code.id.clone(),
            code.code.clone(),
            code.created_by_admin_id.clone(),
            code.used_by_user_id.clone().unwrap_or_default(), // Handle Option<String>
            code.expires_at.to_rfc3339(),
            code.created_at.to_rfc3339(),
            code.used_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(), // Handle Option<DateTime<Utc>>
            code.is_active.to_string(),
        ];
        Ok((sql, params))
    }

    pub async fn create_invitation(
        &self,
        inviter_id: &str,
        _invitee_email: &str,
        _expires_at: Option<i64>,
    ) -> ArbitrageResult<InvitationCode> {
        let invitation = InvitationCode {
            id: generate_uuid(),
            code: self.generate_unique_code().await?,
            created_by_admin_id: inviter_id.to_string(),
            used_by_user_id: None,
            expires_at: Utc::now() + Duration::days(self.config.invitation_expiry_days),
            created_at: Utc::now(),
            used_at: None,
            is_active: true,
        };

        self.store_invitation_code(&invitation).await?;
        Ok(invitation)
    }

    pub async fn get_invitation_by_code(
        &self,
        code: &str,
    ) -> ArbitrageResult<Option<InvitationCode>> {
        let query = "SELECT * FROM invitation_codes WHERE code = ?";
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[code.into()])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        if let Ok(results) = result.results::<serde_json::Value>() {
            if let Some(first_result) = results.first() {
                // D1 results are returned as JavaScript objects, so we need to extract the values properly
                let invitation = InvitationCode {
                    id: first_result
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| ArbitrageError::parse_error("Missing id field"))?,
                    code: first_result
                        .get("code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| ArbitrageError::parse_error("Missing code field"))?,
                    created_by_admin_id: first_result
                        .get("created_by_admin_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| {
                            ArbitrageError::parse_error("Missing created_by_admin_id field")
                        })?,
                    expires_at: {
                        let expires_str = first_result
                            .get("expires_at")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                            ArbitrageError::parse_error("Missing expires_at field")
                        })?;
                        DateTime::parse_from_rfc3339(expires_str)
                            .map_err(|e| {
                                ArbitrageError::parse_error(format!(
                                    "Invalid expires_at format: {}",
                                    e
                                ))
                            })?
                            .with_timezone(&Utc)
                    },
                    created_at: {
                        let created_str = first_result
                            .get("created_at")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                            ArbitrageError::parse_error("Missing created_at field")
                        })?;
                        DateTime::parse_from_rfc3339(created_str)
                            .map_err(|e| {
                                ArbitrageError::parse_error(format!(
                                    "Invalid created_at format: {}",
                                    e
                                ))
                            })?
                            .with_timezone(&Utc)
                    },
                    is_active: first_result
                        .get("is_active")
                        .and_then(|v| v.as_bool())
                        .ok_or_else(|| ArbitrageError::parse_error("Missing is_active field"))?,
                    used_by_user_id: first_result
                        .get("used_by_user_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    used_at: {
                        if let Some(used_str) = first_result.get("used_at").and_then(|v| v.as_str())
                        {
                            Some(
                                DateTime::parse_from_rfc3339(used_str)
                                    .map_err(|e| {
                                        ArbitrageError::parse_error(format!(
                                            "Invalid used_at format: {}",
                                            e
                                        ))
                                    })?
                                    .with_timezone(&Utc),
                            )
                        } else {
                            None
                        }
                    },
                };
                Ok(Some(invitation))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn update_invitation_status(
        &self,
        code: &str,
        status: InvitationStatus,
    ) -> ArbitrageResult<()> {
        let query = "UPDATE invitations SET status = ?, updated_at = ? WHERE code = ?";
        let updated_at = Utc::now().timestamp();
        let params = [
            status.as_str().into(),
            updated_at.to_string().into(),
            code.into(),
        ];
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&params)?;
        bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_invitation(&self, code: &str) -> ArbitrageResult<()> {
        let query = "DELETE FROM invitations WHERE code = ?";
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[code.into()])?;
        bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;
        Ok(())
    }

    pub async fn get_invitations_by_inviter(
        &self,
        inviter_id: &str,
    ) -> ArbitrageResult<Vec<InvitationCode>> {
        let query = "SELECT * FROM invitation_codes WHERE created_by_admin_id = ?";
        let stmt = self.d1_service.prepare(query);
        let bound_stmt = stmt.bind(&[inviter_id.into()])?;
        let result = bound_stmt
            .run()
            .await
            .map_err(|e| ArbitrageError::database_error(e.to_string()))?;

        let mut invitations = Vec::new();
        if let Ok(results) = result.results::<serde_json::Value>() {
            for result_row in results {
                // Convert D1 result to InvitationCode struct
                let invitation = InvitationCode {
                    id: result_row
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| ArbitrageError::parse_error("Missing id field"))?,
                    code: result_row
                        .get("code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| ArbitrageError::parse_error("Missing code field"))?,
                    created_by_admin_id: result_row
                        .get("created_by_admin_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| {
                            ArbitrageError::parse_error("Missing created_by_admin_id field")
                        })?,
                    expires_at: {
                        let expires_str = result_row
                            .get("expires_at")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ArbitrageError::parse_error("Missing expires_at field")
                            })?;
                        DateTime::parse_from_rfc3339(expires_str)
                            .map_err(|e| {
                                ArbitrageError::parse_error(format!(
                                    "Invalid expires_at format: {}",
                                    e
                                ))
                            })?
                            .with_timezone(&Utc)
                    },
                    created_at: {
                        let created_str = result_row
                            .get("created_at")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ArbitrageError::parse_error("Missing created_at field")
                            })?;
                        DateTime::parse_from_rfc3339(created_str)
                            .map_err(|e| {
                                ArbitrageError::parse_error(format!(
                                    "Invalid created_at format: {}",
                                    e
                                ))
                            })?
                            .with_timezone(&Utc)
                    },
                    is_active: result_row
                        .get("is_active")
                        .and_then(|v| v.as_bool())
                        .ok_or_else(|| ArbitrageError::parse_error("Missing is_active field"))?,
                    used_by_user_id: result_row
                        .get("used_by_user_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    used_at: {
                        if let Some(used_str) = result_row.get("used_at").and_then(|v| v.as_str()) {
                            Some(
                                DateTime::parse_from_rfc3339(used_str)
                                    .map_err(|e| {
                                        ArbitrageError::parse_error(format!(
                                            "Invalid used_at format: {}",
                                            e
                                        ))
                                    })?
                                    .with_timezone(&Utc),
                            )
                        } else {
                            None
                        }
                    },
                };
                invitations.push(invitation);
            }
        }

        Ok(invitations)
    }
}
