//! Tests for cache utilities
//! Extracted from src/services/core/infrastructure/cache.rs

use crate::infrastructure::cache::*;
use std::time::Duration;

#[tokio::test]
async fn test_memory_cache_basic_operations() {
    let cache = MemoryCache::new(100); // 100 item capacity
    
    // Test set and get
    cache.set("key1", "value1", None).await;
    let result = cache.get("key1").await;
    assert_eq!(result, Some("value1".to_string()));
    
    // Test non-existent key
    let result = cache.get("nonexistent").await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_memory_cache_expiration() {
    let cache = MemoryCache::new(100);
    
    // Set with short TTL
    cache.set("expiring_key", "value", Some(Duration::from_millis(50))).await;
    
    // Should be available immediately
    let result = cache.get("expiring_key").await;
    assert_eq!(result, Some("value".to_string()));
    
    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Should be expired
    let result = cache.get("expiring_key").await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_memory_cache_capacity_limit() {
    let cache = MemoryCache::new(2); // Small capacity for testing
    
    // Fill cache to capacity
    cache.set("key1", "value1", None).await;
    cache.set("key2", "value2", None).await;
    
    // Both should be present
    assert_eq!(cache.get("key1").await, Some("value1".to_string()));
    assert_eq!(cache.get("key2").await, Some("value2".to_string()));
    
    // Add third item (should evict oldest)
    cache.set("key3", "value3", None).await;
    
    // key1 should be evicted, key2 and key3 should remain
    assert_eq!(cache.get("key1").await, None);
    assert_eq!(cache.get("key2").await, Some("value2".to_string()));
    assert_eq!(cache.get("key3").await, Some("value3".to_string()));
}

#[tokio::test]
async fn test_redis_cache_operations() {
    // Note: This test assumes Redis is available for testing
    // In a real environment, you might want to use a Redis test container
    let cache_result = RedisCache::new("redis://localhost:6379").await;
    
    if let Ok(cache) = cache_result {
        // Test basic operations
        cache.set("test_key", "test_value", None).await.unwrap();
        let result = cache.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));
        
        // Test deletion
        cache.delete("test_key").await.unwrap();
        let result = cache.get("test_key").await.unwrap();
        assert_eq!(result, None);
    }
    // If Redis is not available, test passes silently
}

#[tokio::test]
async fn test_cache_with_serialization() {
    let cache = MemoryCache::new(100);
    
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestStruct {
        id: u32,
        name: String,
    }
    
    let test_data = TestStruct {
        id: 123,
        name: "test".to_string(),
    };
    
    // Serialize and store
    let serialized = serde_json::to_string(&test_data).unwrap();
    cache.set("struct_key", &serialized, None).await;
    
    // Retrieve and deserialize
    let retrieved = cache.get("struct_key").await.unwrap();
    let deserialized: TestStruct = serde_json::from_str(&retrieved).unwrap();
    
    assert_eq!(deserialized, test_data);
}

#[tokio::test]
async fn test_cache_statistics() {
    let cache = MemoryCache::new(100);
    
    // Perform some operations
    cache.set("key1", "value1", None).await;
    cache.set("key2", "value2", None).await;
    
    // Some hits
    cache.get("key1").await;
    cache.get("key1").await;
    
    // Some misses
    cache.get("nonexistent1").await;
    cache.get("nonexistent2").await;
    
    let stats = cache.get_statistics().await;
    assert_eq!(stats.total_items, 2);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 2);
    assert!((stats.hit_rate - 0.5).abs() < 0.001); // 50% hit rate
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = MemoryCache::new(100);
    
    // Add some items
    cache.set("key1", "value1", None).await;
    cache.set("key2", "value2", None).await;
    cache.set("key3", "value3", None).await;
    
    // Verify they exist
    assert_eq!(cache.get("key1").await, Some("value1".to_string()));
    assert_eq!(cache.get("key2").await, Some("value2".to_string()));
    
    // Clear cache
    cache.clear().await;
    
    // Verify all items are gone
    assert_eq!(cache.get("key1").await, None);
    assert_eq!(cache.get("key2").await, None);
    assert_eq!(cache.get("key3").await, None);
    
    let stats = cache.get_statistics().await;
    assert_eq!(stats.total_items, 0);
}

#[tokio::test]
async fn test_cache_update_operation() {
    let cache = MemoryCache::new(100);
    
    // Set initial value
    cache.set("update_key", "initial_value", None).await;
    assert_eq!(cache.get("update_key").await, Some("initial_value".to_string()));
    
    // Update value
    cache.set("update_key", "updated_value", None).await;
    assert_eq!(cache.get("update_key").await, Some("updated_value".to_string()));
}

#[tokio::test]
async fn test_cache_concurrent_access() {
    let cache = std::sync::Arc::new(MemoryCache::new(100));
    let mut handles = vec![];
    
    // Spawn multiple tasks that access the cache concurrently
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = format!("value_{}", i);
            
            cache_clone.set(&key, &value, None).await;
            let retrieved = cache_clone.get(&key).await;
            assert_eq!(retrieved, Some(value));
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all items are in cache
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        let expected_value = format!("value_{}", i);
        assert_eq!(cache.get(&key).await, Some(expected_value));
    }
}

#[tokio::test]
async fn test_cache_pattern_matching() {
    let cache = MemoryCache::new(100);
    
    // Set up test data with patterns
    cache.set("user:123:profile", "profile_data", None).await;
    cache.set("user:123:settings", "settings_data", None).await;
    cache.set("user:456:profile", "other_profile", None).await;
    cache.set("session:abc123", "session_data", None).await;
    
    // Test pattern matching (if implemented)
    let user_keys = cache.get_keys_matching("user:123:*").await;
    assert_eq!(user_keys.len(), 2);
    assert!(user_keys.contains(&"user:123:profile".to_string()));
    assert!(user_keys.contains(&"user:123:settings".to_string()));
    
    let all_user_keys = cache.get_keys_matching("user:*").await;
    assert_eq!(all_user_keys.len(), 3);
}

#[tokio::test]
async fn test_cache_batch_operations() {
    let cache = MemoryCache::new(100);
    
    // Test batch set
    let batch_data = vec![
        ("batch_key1", "batch_value1"),
        ("batch_key2", "batch_value2"),
        ("batch_key3", "batch_value3"),
    ];
    
    cache.set_batch(batch_data.clone(), None).await;
    
    // Test batch get
    let keys: Vec<String> = batch_data.iter().map(|(k, _)| k.to_string()).collect();
    let results = cache.get_batch(&keys).await;
    
    assert_eq!(results.len(), 3);
    assert_eq!(results["batch_key1"], Some("batch_value1".to_string()));
    assert_eq!(results["batch_key2"], Some("batch_value2".to_string()));
    assert_eq!(results["batch_key3"], Some("batch_value3".to_string()));
}

#[tokio::test]
async fn test_cache_memory_pressure_handling() {
    let cache = MemoryCache::new(5); // Very small cache
    
    // Fill cache beyond capacity
    for i in 0..10 {
        let key = format!("pressure_key_{}", i);
        let value = format!("value_{}", i);
        cache.set(&key, &value, None).await;
    }
    
    // Cache should only contain the last 5 items
    let stats = cache.get_statistics().await;
    assert_eq!(stats.total_items, 5);
    
    // Verify that the most recent items are still there
    for i in 5..10 {
        let key = format!("pressure_key_{}", i);
        let expected_value = format!("value_{}", i);
        assert_eq!(cache.get(&key).await, Some(expected_value));
    }
    
    // Verify that the oldest items were evicted
    for i in 0..5 {
        let key = format!("pressure_key_{}", i);
        assert_eq!(cache.get(&key).await, None);
    }
}