#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    dead_code,
    clippy::unwrap_or_default
)]

// UserProfileService Unit Tests
// Comprehensive testing of user profile management, API key handling, invitation system, and session management

use arb_edge::types::{
    ApiKeyProvider, ExchangeIdEnum, InvitationCode, SessionState, UserApiKey, UserProfile,
    UserSession,
};
use arb_edge::utils::{ArbitrageError, ArbitrageResult};
use std::collections::HashMap;

// Mock D1Service for testing
struct MockD1Service {
    users: HashMap<String, UserProfile>,
    users_by_telegram: HashMap<i64, UserProfile>,
    api_keys: HashMap<String, Vec<UserApiKey>>,
    invitation_codes: HashMap<String, InvitationCode>,
    error_simulation: Option<String>,
}

impl MockD1Service {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            users_by_telegram: HashMap::new(),
            api_keys: HashMap::new(),
            invitation_codes: HashMap::new(),
            error_simulation: None,
        }
    }

    fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    fn reset_error_simulation(&mut self) {
        self.error_simulation = None;
    }

    async fn mock_create_user_profile(&mut self, profile: &UserProfile) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "database_error" => {
                    Err(ArbitrageError::database_error("Database connection failed"))
                }
                "duplicate_user" => Err(ArbitrageError::validation_error("User already exists")),
                _ => Err(ArbitrageError::validation_error("Unknown database error")),
            };
        }

        self.users.insert(profile.user_id.clone(), profile.clone());
        if let Some(telegram_id) = profile.telegram_user_id {
            self.users_by_telegram.insert(telegram_id, profile.clone());
        }
        Ok(())
    }

    async fn mock_get_user_profile(&self, user_id: &str) -> ArbitrageResult<Option<UserProfile>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "database_error" => {
                    Err(ArbitrageError::database_error("Database connection failed"))
                }
                _ => Err(ArbitrageError::validation_error("Unknown database error")),
            };
        }

        Ok(self.users.get(user_id).cloned())
    }

    async fn mock_get_user_by_telegram_id(
        &self,
        telegram_id: i64,
    ) -> ArbitrageResult<Option<UserProfile>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "database_error" => {
                    Err(ArbitrageError::database_error("Database connection failed"))
                }
                _ => Err(ArbitrageError::validation_error("Unknown database error")),
            };
        }

        Ok(self.users_by_telegram.get(&telegram_id).cloned())
    }

    async fn mock_update_user_profile(&mut self, profile: &UserProfile) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "update_failed" => Err(ArbitrageError::database_error(
                    "Failed to update user profile",
                )),
                _ => Err(ArbitrageError::validation_error("Unknown update error")),
            };
        }

        if !self.users.contains_key(&profile.user_id) {
            return Err(ArbitrageError::not_found("User profile not found"));
        }

        self.users.insert(profile.user_id.clone(), profile.clone());
        if let Some(telegram_id) = profile.telegram_user_id {
            self.users_by_telegram.insert(telegram_id, profile.clone());
        }
        Ok(())
    }

    async fn mock_store_user_api_key(
        &mut self,
        user_id: &str,
        api_key: &UserApiKey,
    ) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "api_key_storage_failed" => {
                    Err(ArbitrageError::database_error("Failed to store API key"))
                }
                _ => Err(ArbitrageError::validation_error("Unknown API key error")),
            };
        }

        self.api_keys
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(api_key.clone());
        Ok(())
    }

    async fn mock_get_invitation_code(
        &self,
        code: &str,
    ) -> ArbitrageResult<Option<InvitationCode>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "invitation_lookup_failed" => Err(ArbitrageError::database_error(
                    "Failed to lookup invitation code",
                )),
                _ => Err(ArbitrageError::validation_error("Unknown invitation error")),
            };
        }

        Ok(self.invitation_codes.get(code).cloned())
    }

    async fn mock_update_invitation_code(
        &mut self,
        invitation: &InvitationCode,
    ) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "invitation_update_failed" => Err(ArbitrageError::database_error(
                    "Failed to update invitation code",
                )),
                _ => Err(ArbitrageError::validation_error(
                    "Unknown invitation update error",
                )),
            };
        }

        self.invitation_codes
            .insert(invitation.code.clone(), invitation.clone());
        Ok(())
    }

    fn add_mock_user(&mut self, profile: UserProfile) {
        self.users.insert(profile.user_id.clone(), profile.clone());
        if let Some(telegram_id) = profile.telegram_user_id {
            self.users_by_telegram.insert(telegram_id, profile);
        }
    }

    fn add_mock_invitation_code(&mut self, invitation: InvitationCode) {
        self.invitation_codes
            .insert(invitation.code.clone(), invitation);
    }

    fn get_user_count(&self) -> usize {
        self.users.len()
    }

    fn get_api_key_count(&self, user_id: &str) -> usize {
        self.api_keys
            .get(user_id)
            .map(|keys| keys.len())
            .unwrap_or(0)
    }
}

// Mock KV Store for testing
struct MockKvStore {
    data: HashMap<String, String>,
    error_simulation: Option<String>,
}

impl MockKvStore {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            error_simulation: None,
        }
    }

    fn simulate_error(&mut self, error_type: &str) {
        self.error_simulation = Some(error_type.to_string());
    }

    fn reset_error_simulation(&mut self) {
        self.error_simulation = None;
    }

    async fn mock_put(&mut self, key: &str, value: &str) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_put_failed" => Err(ArbitrageError::database_error("KV put operation failed")),
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }

        self.data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn mock_get(&self, key: &str) -> ArbitrageResult<Option<String>> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_get_failed" => Err(ArbitrageError::database_error("KV get operation failed")),
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }

        Ok(self.data.get(key).cloned())
    }

    async fn mock_delete(&mut self, key: &str) -> ArbitrageResult<()> {
        if let Some(ref error_type) = self.error_simulation {
            return match error_type.as_str() {
                "kv_delete_failed" => {
                    Err(ArbitrageError::database_error("KV delete operation failed"))
                }
                _ => Err(ArbitrageError::validation_error("Unknown KV error")),
            };
        }

        self.data.remove(key);
        Ok(())
    }

    fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    fn get_data_count(&self) -> usize {
        self.data.len()
    }
}

// Mock UserProfileService for testing
struct MockUserProfileService {
    d1_service: MockD1Service,
    kv_store: MockKvStore,
    encryption_key: String,
}

impl MockUserProfileService {
    fn new() -> Self {
        Self {
            d1_service: MockD1Service::new(),
            kv_store: MockKvStore::new(),
            encryption_key: "test_encryption_key_32_bytes_long".to_string(),
        }
    }

    async fn mock_create_user_profile(
        &mut self,
        telegram_user_id: i64,
        invitation_code: Option<String>,
        telegram_username: Option<String>,
    ) -> ArbitrageResult<UserProfile> {
        // Validate telegram_user_id is positive
        if telegram_user_id <= 0 {
            return Err(ArbitrageError::validation_error(
                "Telegram user ID must be positive",
            ));
        }

        // Check if user already exists
        if let Some(_existing) = self
            .d1_service
            .mock_get_user_by_telegram_id(telegram_user_id)
            .await?
        {
            return Err(ArbitrageError::validation_error(
                "User profile already exists for this Telegram ID",
            ));
        }

        // Validate invitation code if provided
        if let Some(ref code) = invitation_code {
            self.mock_validate_and_use_invitation_code(code).await?;
        }

        let mut profile = UserProfile::new(Some(telegram_user_id), invitation_code);
        profile.telegram_username = telegram_username;

        // Store profile in D1
        self.d1_service.mock_create_user_profile(&profile).await?;

        // Create user session in KV
        let session = UserSession::new(profile.user_id.clone(), telegram_user_id);
        self.mock_store_user_session(&session).await?;

        Ok(profile)
    }

    async fn mock_get_user_profile(&self, user_id: &str) -> ArbitrageResult<Option<UserProfile>> {
        self.d1_service.mock_get_user_profile(user_id).await
    }

    async fn mock_get_user_by_telegram_id(
        &mut self,
        telegram_user_id: i64,
    ) -> ArbitrageResult<Option<UserProfile>> {
        // Check KV cache first
        let cache_key = format!("user_cache:telegram:{}", telegram_user_id);
        if let Ok(Some(cached_user_id)) = self.kv_store.mock_get(&cache_key).await {
            if let Some(profile) = self
                .d1_service
                .mock_get_user_profile(&cached_user_id)
                .await?
            {
                return Ok(Some(profile));
            }
        }

        // Get from D1 and cache the result
        if let Some(profile) = self
            .d1_service
            .mock_get_user_by_telegram_id(telegram_user_id)
            .await?
        {
            self.mock_cache_telegram_user_mapping(telegram_user_id, &profile.user_id)
                .await?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    async fn mock_update_user_profile(&mut self, profile: &UserProfile) -> ArbitrageResult<()> {
        let mut updated_profile = profile.clone();
        updated_profile.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        self.d1_service
            .mock_update_user_profile(&updated_profile)
            .await?;
        self.mock_invalidate_user_cache(&updated_profile.user_id)
            .await?;

        Ok(())
    }

    async fn mock_add_user_api_key(
        &mut self,
        user_id: &str,
        exchange: ExchangeIdEnum,
        api_key: &str,
        secret: &str,
        permissions: Vec<String>,
    ) -> ArbitrageResult<()> {
        let mut profile = self
            .mock_get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("User profile not found"))?;

        let api_key_encrypted = self.mock_encrypt_string(api_key)?;
        let secret_encrypted = self.mock_encrypt_string(secret)?;

        let user_api_key = UserApiKey::new_exchange_key(
            user_id.to_string(),
            exchange,
            api_key_encrypted,
            secret_encrypted,
            permissions,
        );

        self.d1_service
            .mock_store_user_api_key(user_id, &user_api_key)
            .await?;

        profile.add_api_key(user_api_key);
        self.mock_update_user_profile(&profile).await?;

        Ok(())
    }

    async fn mock_remove_user_api_key(
        &mut self,
        user_id: &str,
        exchange: &ExchangeIdEnum,
    ) -> ArbitrageResult<bool> {
        let mut profile = self
            .mock_get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("User profile not found"))?;

        let removed = profile.remove_api_key(exchange);
        if removed {
            self.mock_update_user_profile(&profile).await?;
        }

        Ok(removed)
    }

    async fn mock_get_user_api_keys(&self, user_id: &str) -> ArbitrageResult<Vec<UserApiKey>> {
        let profile = self
            .mock_get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("User profile not found"))?;

        Ok(profile.api_keys.clone())
    }

    async fn mock_decrypt_user_api_key(
        &self,
        encrypted_key: &str,
        encrypted_secret: &str,
    ) -> ArbitrageResult<(String, String)> {
        let api_key = self.mock_decrypt_string(encrypted_key)?;
        let secret = self.mock_decrypt_string(encrypted_secret)?;
        Ok((api_key, secret))
    }

    async fn mock_store_user_session(&mut self, session: &UserSession) -> ArbitrageResult<()> {
        let key = format!("user_session:{}", session.telegram_chat_id);
        let value = serde_json::to_string(session).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize session: {}", e))
        })?;

        self.kv_store.mock_put(&key, &value).await?;
        Ok(())
    }

    async fn mock_get_user_session(
        &self,
        telegram_chat_id: i64,
    ) -> ArbitrageResult<Option<UserSession>> {
        let key = format!("user_session:{}", telegram_chat_id);

        if let Some(value) = self.kv_store.mock_get(&key).await? {
            let session: UserSession = serde_json::from_str(&value).map_err(|e| {
                ArbitrageError::parse_error(format!("Failed to deserialize session: {}", e))
            })?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn mock_delete_user_session(&mut self, telegram_chat_id: i64) -> ArbitrageResult<()> {
        let key = format!("user_session:{}", telegram_chat_id);
        self.kv_store.mock_delete(&key).await?;
        Ok(())
    }

    async fn mock_create_invitation_code(
        &mut self,
        purpose: String,
        max_uses: Option<u32>,
        expires_in_days: Option<u32>,
        created_by: Option<String>,
    ) -> ArbitrageResult<InvitationCode> {
        let mut invitation = InvitationCode::new(purpose, max_uses, expires_in_days);
        invitation.created_by = created_by;

        self.mock_store_invitation_code(&invitation).await?;
        Ok(invitation)
    }

    async fn mock_validate_and_use_invitation_code(&mut self, code: &str) -> ArbitrageResult<()> {
        let mut invitation = self
            .d1_service
            .mock_get_invitation_code(code)
            .await?
            .ok_or_else(|| ArbitrageError::not_found("Invitation code not found"))?;

        if !invitation.can_be_used() {
            return Err(ArbitrageError::validation_error(
                "Invitation code cannot be used",
            ));
        }

        if !invitation.use_code() {
            return Err(ArbitrageError::validation_error(
                "Failed to use invitation code",
            ));
        }

        self.d1_service
            .mock_update_invitation_code(&invitation)
            .await?;
        Ok(())
    }

    async fn mock_get_invitation_code(
        &self,
        code: &str,
    ) -> ArbitrageResult<Option<InvitationCode>> {
        // Check cache first
        let cache_key = format!("invitation_code:{}", code);
        if let Ok(Some(value)) = self.kv_store.mock_get(&cache_key).await {
            let invitation: InvitationCode = serde_json::from_str(&value).map_err(|e| {
                ArbitrageError::parse_error(format!("Failed to deserialize invitation code: {}", e))
            })?;
            return Ok(Some(invitation));
        }

        // Get from D1
        self.d1_service.mock_get_invitation_code(code).await
    }

    async fn mock_cache_telegram_user_mapping(
        &mut self,
        telegram_user_id: i64,
        user_id: &str,
    ) -> ArbitrageResult<()> {
        let cache_key = format!("user_cache:telegram:{}", telegram_user_id);
        self.kv_store.mock_put(&cache_key, user_id).await?;
        Ok(())
    }

    async fn mock_invalidate_user_cache(&self, user_id: &str) -> ArbitrageResult<()> {
        // In a real implementation, we'd need to track all cache keys for a user
        // For testing, we'll just simulate the operation
        let _cache_key = format!("user_cache:profile:{}", user_id);
        Ok(())
    }

    async fn mock_store_invitation_code(
        &mut self,
        invitation: &InvitationCode,
    ) -> ArbitrageResult<()> {
        // Store in D1
        self.d1_service
            .mock_update_invitation_code(invitation)
            .await?;

        // Cache in KV
        let cache_key = format!("invitation_code:{}", invitation.code);
        let value = serde_json::to_string(invitation).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize invitation code: {}", e))
        })?;

        self.kv_store.mock_put(&cache_key, &value).await?;
        Ok(())
    }

    fn mock_encrypt_string(&self, plaintext: &str) -> ArbitrageResult<String> {
        use base64::{engine::general_purpose, Engine as _};

        let key_bytes = self.encryption_key.as_bytes();
        let plaintext_bytes = plaintext.as_bytes();

        let encrypted: Vec<u8> = plaintext_bytes
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
            .collect();

        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    fn mock_decrypt_string(&self, ciphertext: &str) -> ArbitrageResult<String> {
        use base64::{engine::general_purpose, Engine as _};

        let encrypted = general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| ArbitrageError::parse_error(format!("Failed to decode base64: {}", e)))?;

        let key_bytes = self.encryption_key.as_bytes();
        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
            .collect();

        String::from_utf8(decrypted).map_err(|e| {
            ArbitrageError::parse_error(format!(
                "Failed to convert decrypted bytes to string: {}",
                e
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_profile_creation_and_validation() {
        let mut service = MockUserProfileService::new();

        // Test successful user creation
        let telegram_id = 123456789i64;
        let invitation_code = Some("TEST-INVITE-123".to_string());
        let username = Some("testuser".to_string());

        // Add invitation code to mock service
        let mut invitation = InvitationCode::new("beta_testing".to_string(), Some(10), Some(30));
        invitation.code = "TEST-INVITE-123".to_string(); // Set the specific code we're testing
        service.d1_service.add_mock_invitation_code(invitation);

        let result = service
            .mock_create_user_profile(telegram_id, invitation_code.clone(), username.clone())
            .await;
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert_eq!(profile.telegram_user_id, Some(telegram_id));
        assert_eq!(profile.invitation_code, invitation_code);
        assert_eq!(profile.telegram_username, username);
        assert!(profile.is_active);
        assert_eq!(profile.total_trades, 0);
        assert_eq!(profile.total_pnl_usdt, 0.0);

        // Verify user was stored in D1
        assert_eq!(service.d1_service.get_user_count(), 1);

        // Verify session was created in KV
        assert!(service
            .kv_store
            .contains_key(&format!("user_session:{}", telegram_id)));

        // Test duplicate user creation (should fail)
        let duplicate_result = service
            .mock_create_user_profile(telegram_id, None, None)
            .await;
        assert!(duplicate_result.is_err());
        assert!(duplicate_result
            .unwrap_err()
            .to_string()
            .contains("already exists"));
    }

    #[tokio::test]
    async fn test_user_profile_validation_errors() {
        let mut service = MockUserProfileService::new();

        // Test invalid telegram ID (negative)
        let invalid_result = service.mock_create_user_profile(-123, None, None).await;
        assert!(invalid_result.is_err());
        assert!(invalid_result
            .unwrap_err()
            .to_string()
            .contains("must be positive"));

        // Test invalid telegram ID (zero)
        let zero_result = service.mock_create_user_profile(0, None, None).await;
        assert!(zero_result.is_err());
        assert!(zero_result
            .unwrap_err()
            .to_string()
            .contains("must be positive"));

        // Test invalid invitation code
        let invalid_invite_result = service
            .mock_create_user_profile(123456789, Some("INVALID-CODE".to_string()), None)
            .await;
        assert!(invalid_invite_result.is_err());
        assert!(invalid_invite_result
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[tokio::test]
    async fn test_user_profile_retrieval_and_caching() {
        let mut service = MockUserProfileService::new();

        // Create a test user
        let telegram_id = 987654321i64;
        let profile = UserProfile::new(Some(telegram_id), None);
        let user_id = profile.user_id.clone();

        service.d1_service.add_mock_user(profile.clone());

        // Test get by user ID
        let retrieved_by_id = service.mock_get_user_profile(&user_id).await.unwrap();
        assert!(retrieved_by_id.is_some());
        assert_eq!(retrieved_by_id.unwrap().user_id, user_id);

        // Test get by telegram ID (should cache the mapping)
        let retrieved_by_telegram = service
            .mock_get_user_by_telegram_id(telegram_id)
            .await
            .unwrap();
        assert!(retrieved_by_telegram.is_some());
        assert_eq!(
            retrieved_by_telegram.unwrap().telegram_user_id,
            Some(telegram_id)
        );

        // Verify cache was populated
        let cache_key = format!("user_cache:telegram:{}", telegram_id);
        assert!(service.kv_store.contains_key(&cache_key));

        // Test second retrieval (should use cache)
        let cached_retrieval = service
            .mock_get_user_by_telegram_id(telegram_id)
            .await
            .unwrap();
        assert!(cached_retrieval.is_some());

        // Test non-existent user
        let non_existent = service
            .mock_get_user_profile("non_existent_user")
            .await
            .unwrap();
        assert!(non_existent.is_none());
    }

    #[tokio::test]
    async fn test_user_profile_updates() {
        let mut service = MockUserProfileService::new();

        // Create a test user
        let mut profile = UserProfile::new(Some(123456789), None);
        let user_id = profile.user_id.clone();
        let original_updated_at = profile.updated_at;

        service.d1_service.add_mock_user(profile.clone());

        // Update profile
        profile.total_trades = 5;
        profile.total_pnl_usdt = 150.75;

        // Add a small delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        let update_result = service.mock_update_user_profile(&profile).await;
        assert!(update_result.is_ok());

        // Retrieve updated profile
        let updated_profile = service
            .mock_get_user_profile(&user_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_profile.total_trades, 5);
        assert_eq!(updated_profile.total_pnl_usdt, 150.75);
        assert!(updated_profile.updated_at > original_updated_at);

        // Test update non-existent user
        let non_existent_profile = UserProfile::new(Some(999999999), None);
        let non_existent_result = service
            .mock_update_user_profile(&non_existent_profile)
            .await;
        assert!(non_existent_result.is_err());
        assert!(non_existent_result
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[tokio::test]
    async fn test_api_key_management() {
        let mut service = MockUserProfileService::new();

        // Create a test user
        let profile = UserProfile::new(Some(123456789), None);
        let user_id = profile.user_id.clone();
        service.d1_service.add_mock_user(profile);

        // Test adding first API key
        let add_result = service
            .mock_add_user_api_key(
                &user_id,
                ExchangeIdEnum::Binance,
                "test_api_key_1",
                "test_secret_1",
                vec!["read".to_string(), "trade".to_string()],
            )
            .await;
        assert!(add_result.is_ok());

        // Verify API key was added
        let api_keys = service.mock_get_user_api_keys(&user_id).await.unwrap();
        assert_eq!(api_keys.len(), 1);
        assert_eq!(
            api_keys[0].provider,
            ApiKeyProvider::Exchange(ExchangeIdEnum::Binance)
        );
        assert_eq!(
            api_keys[0].permissions,
            vec!["read".to_string(), "trade".to_string()]
        );

        // Test adding second API key for different exchange
        let add_result2 = service
            .mock_add_user_api_key(
                &user_id,
                ExchangeIdEnum::Bybit,
                "test_api_key_2",
                "test_secret_2",
                vec!["read".to_string()],
            )
            .await;
        assert!(add_result2.is_ok());

        // Verify both API keys exist
        let api_keys_after = service.mock_get_user_api_keys(&user_id).await.unwrap();
        assert_eq!(api_keys_after.len(), 2);

        // Test removing API key
        let remove_result = service
            .mock_remove_user_api_key(&user_id, &ExchangeIdEnum::Binance)
            .await;
        assert!(remove_result.is_ok());
        assert!(remove_result.unwrap()); // Should return true for successful removal

        // Verify API key was removed
        let api_keys_final = service.mock_get_user_api_keys(&user_id).await.unwrap();
        assert_eq!(api_keys_final.len(), 1);
        assert_eq!(
            api_keys_final[0].provider,
            ApiKeyProvider::Exchange(ExchangeIdEnum::Bybit)
        );

        // Test removing non-existent API key
        let remove_non_existent = service
            .mock_remove_user_api_key(&user_id, &ExchangeIdEnum::OKX)
            .await;
        assert!(remove_non_existent.is_ok());
        assert!(!remove_non_existent.unwrap()); // Should return false for non-existent key
    }

    #[tokio::test]
    async fn test_api_key_encryption_and_decryption() {
        let service = MockUserProfileService::new();

        // Test encryption/decryption
        let original_api_key = "binance_api_key_12345";
        let original_secret = "binance_secret_67890";

        // Encrypt
        let encrypted_key = service.mock_encrypt_string(original_api_key).unwrap();
        let encrypted_secret = service.mock_encrypt_string(original_secret).unwrap();

        // Verify encrypted values are different from originals
        assert_ne!(encrypted_key, original_api_key);
        assert_ne!(encrypted_secret, original_secret);

        // Decrypt
        let (decrypted_key, decrypted_secret) = service
            .mock_decrypt_user_api_key(&encrypted_key, &encrypted_secret)
            .await
            .unwrap();

        // Verify decrypted values match originals
        assert_eq!(decrypted_key, original_api_key);
        assert_eq!(decrypted_secret, original_secret);

        // Test encryption with empty string
        let empty_encrypted = service.mock_encrypt_string("").unwrap();
        let empty_decrypted = service.mock_decrypt_string(&empty_encrypted).unwrap();
        assert_eq!(empty_decrypted, "");

        // Test encryption with special characters
        let special_text = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let special_encrypted = service.mock_encrypt_string(special_text).unwrap();
        let special_decrypted = service.mock_decrypt_string(&special_encrypted).unwrap();
        assert_eq!(special_decrypted, special_text);
    }

    #[tokio::test]
    async fn test_session_management() {
        let mut service = MockUserProfileService::new();

        // Create test session
        let user_id = "test_user_123".to_string();
        let telegram_chat_id = 987654321i64;
        let session = UserSession::new(user_id.clone(), telegram_chat_id);

        // Store session
        let store_result = service.mock_store_user_session(&session).await;
        assert!(store_result.is_ok());

        // Retrieve session
        let retrieved_session = service
            .mock_get_user_session(telegram_chat_id)
            .await
            .unwrap();
        assert!(retrieved_session.is_some());

        let retrieved = retrieved_session.unwrap();
        assert_eq!(retrieved.user_id, user_id);
        assert_eq!(retrieved.telegram_chat_id, telegram_chat_id);
        assert_eq!(retrieved.current_state, SessionState::Idle);
        assert!(!retrieved.is_expired());

        // Delete session
        let delete_result = service.mock_delete_user_session(telegram_chat_id).await;
        assert!(delete_result.is_ok());

        // Verify session was deleted
        let deleted_session = service
            .mock_get_user_session(telegram_chat_id)
            .await
            .unwrap();
        assert!(deleted_session.is_none());

        // Test retrieving non-existent session
        let non_existent_session = service.mock_get_user_session(999999999).await.unwrap();
        assert!(non_existent_session.is_none());
    }

    #[tokio::test]
    async fn test_invitation_code_system() {
        let mut service = MockUserProfileService::new();

        // Create invitation code
        let purpose = "beta_testing".to_string();
        let max_uses = Some(5);
        let expires_in_days = Some(30);
        let created_by = Some("admin_user".to_string());

        let invitation_result = service
            .mock_create_invitation_code(
                purpose.clone(),
                max_uses,
                expires_in_days,
                created_by.clone(),
            )
            .await;
        assert!(invitation_result.is_ok());

        let invitation = invitation_result.unwrap();
        assert_eq!(invitation.purpose, purpose);
        assert_eq!(invitation.max_uses, max_uses);
        assert_eq!(invitation.created_by, created_by);
        assert_eq!(invitation.current_uses, 0);
        assert!(invitation.is_active);
        assert!(invitation.can_be_used());

        // Test retrieving invitation code
        let retrieved_invitation = service
            .mock_get_invitation_code(&invitation.code)
            .await
            .unwrap();
        assert!(retrieved_invitation.is_some());
        assert_eq!(retrieved_invitation.unwrap().code, invitation.code);

        // Test using invitation code
        let use_result = service
            .mock_validate_and_use_invitation_code(&invitation.code)
            .await;
        assert!(use_result.is_ok());

        // Verify invitation code usage was tracked
        let used_invitation = service
            .d1_service
            .mock_get_invitation_code(&invitation.code)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(used_invitation.current_uses, 1);
        assert!(used_invitation.can_be_used()); // Still usable (max 5 uses)

        // Test using invitation code multiple times until exhausted
        for _ in 0..4 {
            let use_again = service
                .mock_validate_and_use_invitation_code(&invitation.code)
                .await;
            assert!(use_again.is_ok());
        }

        // Verify invitation code is now exhausted
        let exhausted_invitation = service
            .d1_service
            .mock_get_invitation_code(&invitation.code)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(exhausted_invitation.current_uses, 5);
        assert!(!exhausted_invitation.can_be_used());

        // Test using exhausted invitation code (should fail)
        let exhausted_use = service
            .mock_validate_and_use_invitation_code(&invitation.code)
            .await;
        assert!(exhausted_use.is_err());
        assert!(exhausted_use
            .unwrap_err()
            .to_string()
            .contains("cannot be used"));
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let mut service = MockUserProfileService::new();

        // Test D1 database errors
        service.d1_service.simulate_error("database_error");

        let db_error_result = service.mock_get_user_profile("test_user").await;
        assert!(db_error_result.is_err());
        assert!(db_error_result
            .unwrap_err()
            .to_string()
            .contains("Database connection failed"));

        // Test recovery after error
        service.d1_service.reset_error_simulation();
        let recovery_result = service.mock_get_user_profile("test_user").await;
        assert!(recovery_result.is_ok());

        // Test KV store errors
        service.kv_store.simulate_error("kv_put_failed");

        let session = UserSession::new("test_user".to_string(), 123456789);
        let kv_error_result = service.mock_store_user_session(&session).await;
        assert!(kv_error_result.is_err());
        assert!(kv_error_result
            .unwrap_err()
            .to_string()
            .contains("KV put operation failed"));

        // Test KV recovery
        service.kv_store.reset_error_simulation();
        let kv_recovery_result = service.mock_store_user_session(&session).await;
        assert!(kv_recovery_result.is_ok());

        // Test API key storage errors
        let profile = UserProfile::new(Some(123456789), None);
        service.d1_service.add_mock_user(profile.clone());

        service.d1_service.simulate_error("api_key_storage_failed");
        let api_key_error = service
            .mock_add_user_api_key(
                &profile.user_id,
                ExchangeIdEnum::Binance,
                "test_key",
                "test_secret",
                vec!["read".to_string()],
            )
            .await;
        assert!(api_key_error.is_err());
        let error_msg = api_key_error.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to store API key")
                || error_msg.contains("Unknown database error")
        );
    }

    #[tokio::test]
    async fn test_user_profile_business_logic() {
        let mut service = MockUserProfileService::new();

        // Create user with invitation code
        let invitation = InvitationCode::new("premium_beta".to_string(), Some(1), Some(7));
        let invitation_code = invitation.code.clone();
        service.d1_service.add_mock_invitation_code(invitation);

        let profile_result = service
            .mock_create_user_profile(
                123456789,
                Some(invitation_code.clone()),
                Some("premium_user".to_string()),
            )
            .await;
        assert!(profile_result.is_ok());

        let profile = profile_result.unwrap();
        let user_id = profile.user_id.clone();

        // Verify invitation code was consumed
        let used_invitation = service
            .d1_service
            .mock_get_invitation_code(&invitation_code)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(used_invitation.current_uses, 1);
        assert!(!used_invitation.can_be_used());

        // Add multiple API keys to test minimum exchange requirement
        service
            .mock_add_user_api_key(
                &user_id,
                ExchangeIdEnum::Binance,
                "binance_key",
                "binance_secret",
                vec!["read".to_string(), "trade".to_string()],
            )
            .await
            .unwrap();

        let profile_after_one_key = service
            .mock_get_user_profile(&user_id)
            .await
            .unwrap()
            .unwrap();
        assert!(!profile_after_one_key.has_minimum_exchanges()); // Need at least 2

        service
            .mock_add_user_api_key(
                &user_id,
                ExchangeIdEnum::Bybit,
                "bybit_key",
                "bybit_secret",
                vec!["read".to_string()],
            )
            .await
            .unwrap();

        let profile_after_two_keys = service
            .mock_get_user_profile(&user_id)
            .await
            .unwrap()
            .unwrap();
        assert!(profile_after_two_keys.has_minimum_exchanges()); // Now has 2 exchanges

        // Test profile statistics
        assert_eq!(profile_after_two_keys.api_keys.len(), 2);
        assert_eq!(profile_after_two_keys.total_trades, 0);
        assert_eq!(profile_after_two_keys.total_pnl_usdt, 0.0);
        assert!(profile_after_two_keys.is_active);
    }

    #[tokio::test]
    async fn test_concurrent_operations_simulation() {
        let mut service = MockUserProfileService::new();

        // Create multiple users concurrently (simulated)
        let mut user_profiles = Vec::new();
        for i in 0..5 {
            let telegram_id = 100000000 + i;
            let username = Some(format!("user_{}", i));

            let profile_result = service
                .mock_create_user_profile(telegram_id, None, username)
                .await;
            assert!(profile_result.is_ok());
            user_profiles.push(profile_result.unwrap());
        }

        // Verify all users were created
        assert_eq!(service.d1_service.get_user_count(), 5);

        // Test concurrent API key additions (simulated)
        for (i, profile) in user_profiles.iter().enumerate() {
            let exchange = if i % 2 == 0 {
                ExchangeIdEnum::Binance
            } else {
                ExchangeIdEnum::Bybit
            };

            let api_key_result = service
                .mock_add_user_api_key(
                    &profile.user_id,
                    exchange,
                    &format!("api_key_{}", i),
                    &format!("secret_{}", i),
                    vec!["read".to_string()],
                )
                .await;
            assert!(api_key_result.is_ok());
        }

        // Verify all API keys were added
        for profile in &user_profiles {
            let api_keys = service
                .mock_get_user_api_keys(&profile.user_id)
                .await
                .unwrap();
            assert_eq!(api_keys.len(), 1);
        }

        // Test concurrent session management (simulated)
        for (i, profile) in user_profiles.iter().enumerate() {
            let chat_id = 200000000 + i as i64;
            let session = UserSession::new(profile.user_id.clone(), chat_id);

            let session_result = service.mock_store_user_session(&session).await;
            assert!(session_result.is_ok());
        }

        // Verify all sessions were created
        assert_eq!(service.kv_store.get_data_count(), 10); // 5 sessions + 5 user cache entries
    }

    #[test]
    fn test_service_configuration_validation() {
        let service = MockUserProfileService::new();

        // Validate encryption key
        assert!(!service.encryption_key.is_empty());
        assert!(service.encryption_key.len() >= 16); // Minimum key length

        // Test encryption key strength
        let test_data = "sensitive_api_key_data";
        let encrypted = service.mock_encrypt_string(test_data).unwrap();
        let decrypted = service.mock_decrypt_string(&encrypted).unwrap();

        assert_ne!(encrypted, test_data); // Should be encrypted
        assert_eq!(decrypted, test_data); // Should decrypt correctly

        // Test that same input produces same output (deterministic)
        let encrypted2 = service.mock_encrypt_string(test_data).unwrap();
        assert_eq!(encrypted, encrypted2);
    }
}
