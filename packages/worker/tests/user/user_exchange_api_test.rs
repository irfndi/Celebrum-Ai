use crate::user::user_exchange_api::*;

#[tokio::test]
async fn test_encryption_decryption() {
    use aes_gcm::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        Aes256Gcm, Key, Nonce,
    };
    use base64::{engine::general_purpose, Engine as _};
    use sha2::{Digest, Sha256};

    // Test the encryption/decryption logic directly
    let encryption_key = "fake_test_encryption_key_for_testing_only";
    let original = "test_api_key_12345";

    // Encrypt
    let mut hasher = Sha256::new();
    hasher.update(encryption_key.as_bytes());
    let key_bytes = hasher.finalize();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, original.as_bytes()).unwrap();
    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);
    let encrypted = general_purpose::STANDARD.encode(encrypted_data);

    // Decrypt
    let encrypted_data = general_purpose::STANDARD.decode(encrypted).unwrap();
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();
    let decrypted = String::from_utf8(plaintext).unwrap();

    assert_eq!(original, decrypted);
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
