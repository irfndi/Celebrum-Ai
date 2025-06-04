use crate::log_info;
use crate::services::core::infrastructure::D1Service;
use crate::services::core::trading::exchange::{ExchangeInterface, ExchangeService};
use crate::services::core::user::UserProfileService;
use crate::types::{
    AccountStatus, ExchangeCredentials, ExchangeIdEnum, SubscriptionTier, Ticker, UserApiKey,
    UserProfile,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
use aes_gcm::aead::Aead; // Added Aead for encrypt/decrypt
use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use worker::kv::KvStore;

/// User Exchange API Management Service
/// Provides secure CRUD operations, validation, and compatibility checking for user exchange APIs
pub struct UserExchangeApiService {
    user_profile_service: Arc<UserProfileService>,
    exchange_service: Arc<ExchangeService>,
    d1_service: Arc<D1Service>,
    kv_store: KvStore,
    encryption_key: SecretString,
}

/// API Key Validation Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyValidationResult {
    pub is_valid: bool,
    pub can_read_market_data: bool,
    pub can_trade: bool,
    pub exchange_status: String,
    pub rate_limit_info: Option<RateLimitInfo>,
    pub error_message: Option<String>,
    pub last_validated: u64,
}

/// Rate Limit Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_per_minute: u32,
    pub requests_remaining: u32,
    pub reset_time: u64,
}

/// Exchange Compatibility Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCompatibilityResult {
    pub is_compatible: bool,
    pub supported_features: Vec<String>,
    pub missing_features: Vec<String>,
    pub arbitrage_compatible: bool,
    pub technical_compatible: bool,
    pub min_exchanges_met: bool,
    pub compatibility_score: f64,
}

/// API Key Addition Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddApiKeyRequest {
    pub exchange_id: String,
    pub api_key: String,
    pub secret: String,
    pub passphrase: Option<String>,
    pub exchange_type: Option<String>, // "spot", "futures", "margin"
    pub default_leverage: Option<u32>,
    pub is_testnet: Option<bool>,
}

/// API Key Update Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub is_active: Option<bool>,
    pub exchange_type: Option<String>,
    pub default_leverage: Option<u32>,
    pub permissions: Option<Vec<String>>,
}

impl UserExchangeApiService {
    const API_VALIDATION_CACHE_PREFIX: &'static str = "api_validation";
    const COMPATIBILITY_CACHE_PREFIX: &'static str = "exchange_compatibility";
    const CACHE_TTL_SECONDS: u64 = 3600; // 1 hour

    pub fn new(
        user_profile_service: Arc<UserProfileService>,
        exchange_service: Arc<ExchangeService>,
        d1_service: Arc<D1Service>,
        kv_store: KvStore,
        encryption_key: SecretString,
    ) -> Self {
        Self {
            user_profile_service,
            exchange_service,
            d1_service,
            kv_store,
            encryption_key,
        }
    }

    /// Add a new API key for a user
    pub async fn add_api_key(
        &self,
        user_id: &str,
        request: AddApiKeyRequest,
    ) -> ArbitrageResult<UserApiKey> {
        // Validate the API key first
        let validation_result = self
            .validate_api_key_connectivity(&request.exchange_id, &request.api_key, &request.secret)
            .await?;

        if !validation_result.is_valid {
            return Err(ArbitrageError::validation_error(format!(
                "API key validation failed: {}",
                validation_result
                    .error_message
                    .unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        // Get user profile
        let mut user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("User not found: {}", user_id)))?;

        // Check if user already has this exchange
        if user_profile
            .api_keys
            .iter()
            .any(|key| key.exchange_id == request.exchange_id && key.is_active)
        {
            return Err(ArbitrageError::validation_error(format!(
                "User already has an active API key for exchange: {}",
                request.exchange_id
            )));
        }

        // Encrypt the API credentials
        let encrypted_api_key = self.encrypt_string(&request.api_key)?;
        let encrypted_secret = self.encrypt_string(&request.secret)?;
        let encrypted_passphrase = if let Some(passphrase) = &request.passphrase {
            Some(self.encrypt_string(passphrase)?)
        } else {
            None
        };

        // Validate default_leverage if provided
        if let Some(default_leverage) = request.default_leverage {
            if default_leverage < 1 || default_leverage > 100 {
                return Err(ArbitrageError::validation_error(format!(
                    "Default leverage must be between 1 and 100, got: {}",
                    default_leverage
                )));
            }
        }

        // Create new API key
        let new_api_key = UserApiKey {
            exchange_id: request.exchange_id.clone(),
            api_key: encrypted_api_key,
            secret: encrypted_secret,
            passphrase: encrypted_passphrase,
            is_active: true,
            permissions: validation_result
                .can_trade
                .then(|| vec!["trade".to_string()])
                .unwrap_or_else(|| vec!["read".to_string()]),
            default_leverage: request.default_leverage,
            exchange_type: request.exchange_type,
            is_testnet: request.is_testnet.unwrap_or(false),
            created_at: Utc::now().timestamp() as u64,
            last_used: None,
        };

        // Add to user profile
        user_profile.api_keys.push(new_api_key.clone());
        user_profile.updated_at = Utc::now().timestamp() as u64;

        // Update user profile in database
        self.user_profile_service
            .update_user_profile(&user_profile)
            .await?;

        // Cache validation result
        self.cache_validation_result(user_id, &request.exchange_id, &validation_result)
            .await?;

        log_info!(
            "Added new API key for user",
            serde_json::json!({
                "user_id": user_id,
                "exchange_id": request.exchange_id,
                "can_trade": validation_result.can_trade,
                "exchange_type": request.exchange_type
            })
        );

        Ok(new_api_key)
    }

    /// Update an existing API key
    pub async fn update_api_key(
        &self,
        user_id: &str,
        exchange_id: &str,
        request: UpdateApiKeyRequest,
    ) -> ArbitrageResult<UserApiKey> {
        // Get user profile
        let mut user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("User not found: {}", user_id)))?;

        // Find the API key to update
        let api_key_index = user_profile
            .api_keys
            .iter()
            .position(|key| key.exchange_id == exchange_id)
            .ok_or_else(|| {
                ArbitrageError::not_found(format!(
                    "API key not found for exchange: {}",
                    exchange_id
                ))
            })?;

        // Update the API key
        let api_key = &mut user_profile.api_keys[api_key_index];

        if let Some(is_active) = request.is_active {
            api_key.is_active = is_active;
        }

        if let Some(exchange_type) = request.exchange_type {
            api_key.exchange_type = Some(exchange_type);
        }

        if let Some(default_leverage) = request.default_leverage {
            // Validate default_leverage is within reasonable range (1-100)
            if default_leverage < 1 || default_leverage > 100 {
                return Err(ArbitrageError::validation_error(format!(
                    "Default leverage must be between 1 and 100, got: {}",
                    default_leverage
                )));
            }
            api_key.default_leverage = Some(default_leverage);
        }

        if let Some(permissions) = request.permissions {
            api_key.permissions = permissions;
        }

        user_profile.updated_at = Utc::now().timestamp() as u64;

        // Update user profile in database
        self.user_profile_service
            .update_user_profile(&user_profile)
            .await?;

        log_info!(
            "Updated API key for user",
            serde_json::json!({
                "user_id": user_id,
                "exchange_id": exchange_id,
                "is_active": api_key.is_active
            })
        );

        Ok(api_key.clone())
    }

    /// Delete an API key
    pub async fn delete_api_key(&self, user_id: &str, exchange_id: &str) -> ArbitrageResult<()> {
        // Get user profile
        let mut user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("User not found: {}", user_id)))?;

        // Remove the API key
        let initial_count = user_profile.api_keys.len();
        user_profile
            .api_keys
            .retain(|key| key.exchange_id != exchange_id);

        if user_profile.api_keys.len() == initial_count {
            return Err(ArbitrageError::not_found(format!(
                "API key not found for exchange: {}",
                exchange_id
            )));
        }

        user_profile.updated_at = Utc::now().timestamp() as u64;

        // Update user profile in database
        self.user_profile_service
            .update_user_profile(&user_profile)
            .await?;

        // Clear cached validation
        self.clear_validation_cache(user_id, exchange_id).await?;

        log_info!(
            "Deleted API key for user",
            serde_json::json!({
                "user_id": user_id,
                "exchange_id": exchange_id
            })
        );

        Ok(())
    }

    /// Get all API keys for a user (with decrypted credentials)
    pub async fn get_user_api_keys(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Vec<(ExchangeIdEnum, ExchangeCredentials)>> {
        let user_profile = self
            .user_profile_service
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| ArbitrageError::not_found(format!("User not found: {}", user_id)))?;

        let mut exchange_credentials = Vec::new();

        for api_key in &user_profile.api_keys {
            if api_key.is_active {
                if let Ok(exchange_id) = api_key.exchange_id.parse::<ExchangeIdEnum>() {
                    // Decrypt credentials and use immediately to minimize memory exposure
                    let decrypted_secret = self.decrypt_string(&api_key.secret)?;
                    let credentials = ExchangeCredentials {
                        exchange: exchange_id,
                        api_key: self.decrypt_string(&api_key.api_key)?,
                        api_secret: decrypted_secret.clone(),
                        secret: decrypted_secret,
                        passphrase: api_key.passphrase.clone(),
                        sandbox: false,
                        is_testnet: api_key.is_testnet.unwrap_or(false),
                        default_leverage: api_key.default_leverage.unwrap_or(1),
                        exchange_type: api_key
                            .exchange_type
                            .clone()
                            .unwrap_or_else(|| "spot".to_string()),
                    };

                    exchange_credentials.push((exchange_id, credentials));
                }
            }
        }

        Ok(exchange_credentials)
    }

    /// Validate API key connectivity and permissions
    pub async fn validate_api_key_connectivity(
        &self,
        exchange_id: &str,
        api_key: &str,
        secret: &str,
    ) -> ArbitrageResult<ApiKeyValidationResult> {
        // Try to make a simple API call to test connectivity
        match self
            .test_api_connectivity(exchange_id, api_key, secret)
            .await
        {
            Ok((can_read, can_trade, rate_limit)) => Ok(ApiKeyValidationResult {
                is_valid: true,
                can_read_market_data: can_read,
                can_trade,
                exchange_status: "connected".to_string(),
                rate_limit_info: rate_limit,
                error_message: None,
                last_validated: Utc::now().timestamp() as u64,
            }),
            Err(e) => Ok(ApiKeyValidationResult {
                is_valid: false,
                can_read_market_data: false,
                can_trade: false,
                exchange_status: "error".to_string(),
                rate_limit_info: None,
                error_message: Some(e.to_string()),
                last_validated: Utc::now().timestamp() as u64,
            }),
        }
    }

    /// Check exchange compatibility for opportunities
    pub async fn check_exchange_compatibility(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<ExchangeCompatibilityResult> {
        // Try cache first
        let cache_key = format!("{}:{}", Self::COMPATIBILITY_CACHE_PREFIX, user_id);
        if let Ok(Some(cached)) = self.get_cached_compatibility(&cache_key).await {
            return Ok(cached);
        }

        let user_exchanges = self.get_user_api_keys(user_id).await?;

        // Check basic requirements
        let arbitrage_compatible = user_exchanges.len() >= 2;
        let technical_compatible = !user_exchanges.is_empty();

        // Check supported features
        let mut supported_features = Vec::new();
        let mut missing_features = Vec::new();

        if arbitrage_compatible {
            supported_features.push("arbitrage".to_string());
        } else {
            missing_features.push("arbitrage (requires 2+ exchanges)".to_string());
        }

        if technical_compatible {
            supported_features.push("technical_analysis".to_string());
        } else {
            missing_features.push("technical_analysis (requires 1+ exchange)".to_string());
        }

        // Calculate compatibility score
        let compatibility_score = if user_exchanges.is_empty() {
            0.0
        } else if arbitrage_compatible {
            1.0
        } else {
            0.5
        };

        let result = ExchangeCompatibilityResult {
            is_compatible: !user_exchanges.is_empty(),
            supported_features,
            missing_features,
            arbitrage_compatible,
            technical_compatible,
            min_exchanges_met: arbitrage_compatible,
            compatibility_score,
        };

        // Cache the result
        self.cache_compatibility_result(&cache_key, &result).await?;

        Ok(result)
    }

    /// Test API connectivity with actual exchange call
    async fn test_api_connectivity(
        &self,
        exchange_id: &str,
        api_key: &str,
        secret: &str,
    ) -> ArbitrageResult<(bool, bool, Option<RateLimitInfo>)> {
        // Try to get account info or balance to test API
        match self
            .exchange_service
            .test_api_connection(exchange_id, api_key, secret)
            .await
        {
            Ok(test_result) => {
                // Parse test result to determine capabilities
                let can_read = test_result
                    .get("can_read")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let can_trade = test_result
                    .get("can_trade")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let rate_limit = test_result.get("rate_limit").and_then(|rl| {
                    Some(RateLimitInfo {
                        requests_per_minute: rl.get("requests_per_minute")?.as_u64()? as u32,
                        requests_remaining: rl.get("requests_remaining")?.as_u64()? as u32,
                        reset_time: rl.get("reset_time")?.as_u64()?,
                    })
                });

                Ok((can_read, can_trade, rate_limit))
            }
            Err(e) => Err(e),
        }
    }

    /// AES-GCM encryption for API keys with secure key derivation
    fn encrypt_string(&self, plaintext: &str) -> ArbitrageResult<String> {
        use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
        use base64::{engine::general_purpose, Engine as _};
        use rand::rngs::OsRng;
        use sha2::{Digest, Sha256};

        // Derive a 256-bit key from the encryption key using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(self.encryption_key.expose_secret().as_bytes());
        let key_bytes = hasher.finalize();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

        // Create cipher instance
        let cipher = Aes256Gcm::new(key);

        // Generate a random 96-bit nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the plaintext
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| ArbitrageError::parse_error(format!("Encryption failed: {}", e)))?;

        // Combine nonce + ciphertext and encode as base64
        let mut encrypted_data = nonce.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(encrypted_data))
    }

    /// AES-GCM decryption for API keys
    fn decrypt_string(&self, encrypted: &str) -> ArbitrageResult<String> {
        use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
        use base64::{engine::general_purpose, Engine as _};
        use sha2::{Digest, Sha256};

        // Decode the base64 encrypted data
        let encrypted_data = general_purpose::STANDARD.decode(encrypted).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to decode encrypted string: {}", e))
        })?;

        // Ensure we have at least nonce (12 bytes) + some ciphertext
        if encrypted_data.len() < 12 {
            return Err(ArbitrageError::parse_error(
                "Invalid encrypted data length".to_string(),
            ));
        }

        // Derive the same 256-bit key from the encryption key using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(self.encryption_key.expose_secret().as_bytes());
        let key_bytes = hasher.finalize();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

        // Create cipher instance
        let cipher = Aes256Gcm::new(key);

        // Extract nonce (first 12 bytes) and ciphertext (remaining bytes)
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the ciphertext
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ArbitrageError::parse_error(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext).map_err(|e| {
            ArbitrageError::parse_error(format!(
                "Failed to convert decrypted data to string: {}",
                e
            ))
        })
    }

    /// Cache validation result
    async fn cache_validation_result(
        &self,
        user_id: &str,
        exchange_id: &str,
        result: &ApiKeyValidationResult,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:{}:{}",
            Self::API_VALIDATION_CACHE_PREFIX,
            user_id,
            exchange_id
        );
        let serialized = serde_json::to_string(result).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize validation result: {}", e))
        })?;

        self.kv_store
            .put(&cache_key, serialized)
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to cache validation result: {:?}", e))
            })?
            .expiration_ttl(Self::CACHE_TTL_SECONDS)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to execute cache put: {:?}", e))
            })?;

        Ok(())
    }

    /// Cache compatibility result
    async fn cache_compatibility_result(
        &self,
        cache_key: &str,
        result: &ExchangeCompatibilityResult,
    ) -> ArbitrageResult<()> {
        let serialized = serde_json::to_string(result).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize compatibility result: {}", e))
        })?;

        self.kv_store
            .put(cache_key, serialized)
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to cache compatibility result: {:?}",
                    e
                ))
            })?
            .expiration_ttl(Self::CACHE_TTL_SECONDS)
            .execute()
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to execute cache put: {:?}", e))
            })?;

        Ok(())
    }

    /// Get cached compatibility result
    async fn get_cached_compatibility(
        &self,
        cache_key: &str,
    ) -> ArbitrageResult<Option<ExchangeCompatibilityResult>> {
        match self.kv_store.get(cache_key).text().await {
            // Already correct
            Ok(Some(cached)) => {
                match serde_json::from_str(&cached) {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => {
                        eprintln!("Warning: Failed to deserialize cached compatibility result for key '{}': {}", cache_key, e);
                        Ok(None) // Invalid cache data
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Clear validation cache
    async fn clear_validation_cache(
        &self,
        user_id: &str,
        exchange_id: &str,
    ) -> ArbitrageResult<()> {
        let cache_key = format!(
            "{}:{}:{}",
            Self::API_VALIDATION_CACHE_PREFIX,
            user_id,
            exchange_id
        );
        let _ = self.kv_store.delete(&cache_key).await; // Already correct
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let service = UserExchangeApiService::new(
            Arc::new(UserProfileService::new(
                D1Service::new(&Env::default()).unwrap(),
            )),
            Arc::new(ExchangeService::new(&Env::default()).unwrap()),
            Arc::new(D1Service::new(&Env::default()).unwrap()),
            KvStore::default(),
            SecretString::new("fake_test_encryption_key_for_testing_only".to_string()),
        );

        let original = "test_api_key_12345";
        let encrypted = service.encrypt_string(original).unwrap();
        let decrypted = service.decrypt_string(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted); // Should be different when encrypted
    }

    #[test]
    fn test_compatibility_scoring() {
        // Test that compatibility scoring works correctly
        let result = ExchangeCompatibilityResult {
            is_compatible: true,
            supported_features: vec!["arbitrage".to_string(), "technical_analysis".to_string()],
            missing_features: vec![],
            arbitrage_compatible: true,
            technical_compatible: true,
            min_exchanges_met: true,
            compatibility_score: 1.0,
        };

        assert_eq!(result.compatibility_score, 1.0);
        assert!(result.arbitrage_compatible);
        assert!(result.technical_compatible);
    }
}
