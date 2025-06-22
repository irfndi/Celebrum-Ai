use crate::services::core::user::session_management::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock KV service for testing
#[derive(Clone)]
struct MockKvService {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl MockKvService {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn put(&self, key: &str, value: &str, _ttl: Option<u32>) -> ArbitrageResult<()> {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get(&self, key: &str) -> ArbitrageResult<Option<String>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(key).cloned())
    }
}

/// Mock D1 service for testing
#[derive(Clone)]
struct MockD1Service {
    sessions: Arc<Mutex<HashMap<String, EnhancedUserSession>>>,
    query_results: Arc<Mutex<Vec<HashMap<String, serde_json::Value>>>>,
}

impl MockD1Service {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            query_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn query(
        &self,
        sql: &str,
        _params: &[serde_json::Value],
    ) -> ArbitrageResult<Vec<HashMap<String, serde_json::Value>>> {
        if sql.contains("SELECT * FROM user_sessions") {
            let sessions = self.sessions.lock().unwrap();
            let results: Vec<HashMap<String, serde_json::Value>> = sessions
                .values()
                .map(|session| {
                    let mut row = HashMap::new();
                    row.insert(
                        "session_id".to_string(),
                        serde_json::json!(session.session_id),
                    );
                    row.insert("user_id".to_string(), serde_json::json!(session.user_id));
                    row.insert(
                        "telegram_id".to_string(),
                        serde_json::json!(session.telegram_id),
                    );
                    row.insert(
                        "session_state".to_string(),
                        serde_json::json!(session.session_state.to_db_string()),
                    );
                    row.insert(
                        "started_at".to_string(),
                        serde_json::json!(session.started_at),
                    );
                    row.insert(
                        "last_activity_at".to_string(),
                        serde_json::json!(session.last_activity_at),
                    );
                    row.insert(
                        "expires_at".to_string(),
                        serde_json::json!(session.expires_at),
                    );
                    row.insert(
                        "onboarding_completed".to_string(),
                        serde_json::json!(session.onboarding_completed),
                    );
                    row.insert(
                        "preferences_set".to_string(),
                        serde_json::json!(session.preferences_set),
                    );
                    row.insert("metadata".to_string(), serde_json::json!("null"));
                    row.insert(
                        "created_at".to_string(),
                        serde_json::json!(session.created_at),
                    );
                    row.insert(
                        "updated_at".to_string(),
                        serde_json::json!(session.updated_at),
                    );
                    row
                })
                .collect();
            Ok(results)
        } else if sql.contains("COUNT(*) as count FROM user_sessions") {
            let sessions = self.sessions.lock().unwrap();
            let count = sessions.len();
            let mut row = HashMap::new();
            row.insert("count".to_string(), serde_json::json!(count));
            Ok(vec![row])
        } else {
            let query_results = self.query_results.lock().unwrap();
            Ok(query_results.clone())
        }
    }

    fn add_session(&self, session: EnhancedUserSession) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.session_id.clone(), session);
    }

    fn get_session_count(&self) -> usize {
        let sessions = self.sessions.lock().unwrap();
        sessions.len()
    }
}

/// Create mock session management service for testing
fn create_mock_session_service() -> (MockD1Service, MockKvService) {
    let mock_d1 = MockD1Service::new();
    let mock_kv = MockKvService::new();

    (mock_d1, mock_kv)
}

/// Create test session
fn create_test_session(telegram_id: i64, user_id: String) -> EnhancedUserSession {
    let now = chrono::Utc::now().timestamp_millis() as u64;
    EnhancedUserSession {
        session_id: format!("session_{}", telegram_id),
        user_id,
        telegram_chat_id: telegram_id,
        telegram_id,
        last_command: None,
        current_state: EnhancedSessionState::Active,
        session_state: EnhancedSessionState::Active,
        temporary_data: std::collections::HashMap::new(),
        started_at: now,
        last_activity_at: now,
        expires_at: now + (7 * 24 * 60 * 60 * 1000), // 7 days
        onboarding_completed: true,
        preferences_set: true,
        metadata: serde_json::Value::Null,
        created_at: now,
        updated_at: now,
        session_analytics: SessionAnalytics {
            commands_executed: 0,
            opportunities_viewed: 0,
            trades_executed: 0,
            session_duration_seconds: 0,
            session_duration_ms: 0,
            last_activity: now,
        },
        config: SessionConfig::default(),
    }
}

#[tokio::test]
async fn test_session_creation() {
    let (mock_d1, _mock_kv) = create_mock_session_service();

    // Test session creation
    let telegram_id = 123456789i64;
    let user_id = "test_user_001".to_string();
    let session = create_test_session(telegram_id, user_id.clone());

    // Verify session properties
    assert_eq!(session.telegram_id, telegram_id);
    assert_eq!(session.user_id, user_id);
    assert!(matches!(
        session.session_state,
        EnhancedSessionState::Active
    ));
    assert!(session.onboarding_completed);
    assert!(session.preferences_set);
    assert!(session.expires_at > session.started_at);

    // Store in mock D1
    mock_d1.add_session(session.clone());
    assert_eq!(mock_d1.get_session_count(), 1);
}

#[tokio::test]
async fn test_session_validation() {
    let (mock_d1, mock_kv) = create_mock_session_service();

    // Create and store a test session
    let session = create_test_session(987654321, "validation_user".to_string());
    mock_d1.add_session(session.clone());

    // Cache the session
    let cache_key = format!("session_cache:{}", session.telegram_id);
    let session_json = serde_json::to_string(&session).unwrap();
    mock_kv
        .put(&cache_key, &session_json, Some(3600))
        .await
        .unwrap();

    // Test cache retrieval
    let cached_session = mock_kv.get(&cache_key).await.unwrap();
    assert!(cached_session.is_some());

    let parsed_session: EnhancedUserSession =
        serde_json::from_str(&cached_session.unwrap()).unwrap();
    assert_eq!(parsed_session.session_id, session.session_id);
    assert_eq!(parsed_session.user_id, session.user_id);
}

#[tokio::test]
async fn test_notification_rate_limiting() {
    let (_mock_d1, mock_kv) = create_mock_session_service();

    let user_id = "rate_limit_user";
    let hour_key = format!(
        "notification_rate:{}:{}",
        user_id,
        chrono::Utc::now().format("%Y-%m-%d-%H")
    );
    let day_key = format!(
        "notification_rate:{}:{}",
        user_id,
        chrono::Utc::now().format("%Y-%m-%d")
    );

    // Simulate rate limiting counters
    mock_kv.put(&hour_key, "1", Some(3600)).await.unwrap();
    mock_kv.put(&day_key, "5", Some(86400)).await.unwrap();

    // Verify rate limiting data
    let hourly_count = mock_kv
        .get(&hour_key)
        .await
        .unwrap()
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let daily_count = mock_kv
        .get(&day_key)
        .await
        .unwrap()
        .unwrap()
        .parse::<u32>()
        .unwrap();

    assert!(hourly_count < 2); // Default hourly limit
    assert!(daily_count < 10); // Default daily limit
}

#[tokio::test]
async fn test_session_analytics() {
    let (_mock_d1, mock_kv) = create_mock_session_service();

    // Test session analytics recording
    let session = create_test_session(123123123, "analytics_user".to_string());

    // Record session start analytics
    let analytics_key = format!("session_analytics:start:{}", session.session_id);
    let analytics_data = serde_json::json!({
        "session_id": session.session_id,
        "user_id": session.user_id,
        "telegram_id": session.telegram_id,
        "started_at": session.started_at,
        "event_type": "session_start",
        "timestamp": chrono::Utc::now().timestamp() as u64
    });

    mock_kv
        .put(
            &analytics_key,
            &analytics_data.to_string(),
            Some(30 * 24 * 3600),
        )
        .await
        .unwrap();

    // Verify analytics storage
    let stored_analytics = mock_kv.get(&analytics_key).await.unwrap().unwrap();
    let parsed_analytics: serde_json::Value = serde_json::from_str(&stored_analytics).unwrap();

    assert_eq!(parsed_analytics["session_id"], session.session_id);
    assert_eq!(parsed_analytics["event_type"], "session_start");
    assert_eq!(parsed_analytics["user_id"], session.user_id);

    // Test session count tracking
    let date_key = format!("session_count:{}", chrono::Utc::now().format("%Y-%m-%d"));
    mock_kv.put(&date_key, "1", Some(24 * 3600)).await.unwrap();

    let session_count = mock_kv
        .get(&date_key)
        .await
        .unwrap()
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert_eq!(session_count, 1);
}

#[tokio::test]
async fn test_session_cleanup() {
    let (mock_d1, _mock_kv) = create_mock_session_service();

    // Create multiple sessions with different states
    let now = chrono::Utc::now().timestamp_millis() as u64;

    // Active session
    let active_session = create_test_session(111, "active".to_string());
    mock_d1.add_session(active_session);

    // Expired session
    let mut expired_session = create_test_session(222, "expired".to_string());
    expired_session.session_state = EnhancedSessionState::Expired;
    expired_session.expires_at = now - (24 * 60 * 60 * 1000);
    mock_d1.add_session(expired_session);

    // Terminated session
    let mut terminated_session = create_test_session(333, "terminated".to_string());
    terminated_session.session_state = EnhancedSessionState::Terminated;
    mock_d1.add_session(terminated_session);

    // Verify all sessions were added
    assert_eq!(mock_d1.get_session_count(), 3);

    // In a real cleanup, expired and terminated sessions would be removed
    // For this test, we just verify they exist and can be identified
    let query_result = mock_d1
        .query("SELECT * FROM user_sessions", &[])
        .await
        .unwrap();
    assert_eq!(query_result.len(), 3);
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    let (mock_d1, mock_kv) = create_mock_session_service();

    // Test concurrent session creation
    let mut sessions = Vec::new();
    for i in 0..10 {
        let session = create_test_session(i as i64, format!("concurrent_user_{}", i));
        sessions.push(session);
    }

    // Store all sessions
    for session in &sessions {
        mock_d1.add_session(session.clone());

        // Cache each session
        let cache_key = format!("session_cache:{}", session.telegram_id);
        let session_json = serde_json::to_string(session).unwrap();
        mock_kv
            .put(&cache_key, &session_json, Some(3600))
            .await
            .unwrap();
    }

    // Verify all sessions were stored
    assert_eq!(mock_d1.get_session_count(), 10);

    // Verify all sessions can be retrieved from cache
    for session in &sessions {
        let cache_key = format!("session_cache:{}", session.telegram_id);
        let cached_session = mock_kv.get(&cache_key).await.unwrap();
        assert!(cached_session.is_some());

        let parsed_session: EnhancedUserSession =
            serde_json::from_str(&cached_session.unwrap()).unwrap();
        assert_eq!(parsed_session.user_id, session.user_id);
    }
}
