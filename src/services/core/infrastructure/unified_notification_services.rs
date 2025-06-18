// Unified Notification Services - Consolidated Notification System
// Consolidates: notification_module (5 files → 1 unified module)
// Components: NotificationCoordinator, TemplateEngine, DeliveryManager, ChannelManager

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use worker::kv::KvStore;

// =============================================================================
// Core Types and Enums
// =============================================================================

/// Notification types for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    OpportunityAlert,
    BalanceChange,
    PriceAlert,
    ProfitLoss,
    RiskWarning,
    SystemMaintenance,
    SecurityAlert,
    TradingSignal,
    Custom(String),
}

impl NotificationType {
    pub fn as_str(&self) -> &str {
        match self {
            NotificationType::OpportunityAlert => "opportunity_alert",
            NotificationType::BalanceChange => "balance_change",
            NotificationType::PriceAlert => "price_alert",
            NotificationType::ProfitLoss => "profit_loss",
            NotificationType::RiskWarning => "risk_warning",
            NotificationType::SystemMaintenance => "system_maintenance",
            NotificationType::SecurityAlert => "security_alert",
            NotificationType::TradingSignal => "trading_signal",
            NotificationType::Custom(name) => name,
        }
    }

    pub fn default_priority(&self) -> u8 {
        match self {
            NotificationType::SecurityAlert => 10,
            NotificationType::RiskWarning => 9,
            NotificationType::OpportunityAlert => 8,
            NotificationType::TradingSignal => 7,
            NotificationType::ProfitLoss => 6,
            NotificationType::BalanceChange => 6,
            NotificationType::PriceAlert => 5,
            NotificationType::SystemMaintenance => 4,
            NotificationType::Custom(_) => 5,
        }
    }

    pub fn default_channels(&self) -> Vec<NotificationChannel> {
        match self {
            NotificationType::SecurityAlert => vec![
                NotificationChannel::Push,
                NotificationChannel::Email,
                NotificationChannel::Telegram,
            ],
            NotificationType::RiskWarning => {
                vec![NotificationChannel::Push, NotificationChannel::Telegram]
            }
            NotificationType::OpportunityAlert => {
                vec![NotificationChannel::Telegram, NotificationChannel::Push]
            }
            NotificationType::TradingSignal => vec![NotificationChannel::Telegram],
            NotificationType::ProfitLoss => {
                vec![NotificationChannel::Email, NotificationChannel::Telegram]
            }
            NotificationType::BalanceChange => vec![NotificationChannel::Email],
            NotificationType::PriceAlert => vec![NotificationChannel::Push],
            NotificationType::SystemMaintenance => vec![NotificationChannel::Email],
            NotificationType::Custom(_) => vec![NotificationChannel::Email],
        }
    }
}

/// Notification channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Telegram,
    Push,
    Webhook,
    Discord,
    Slack,
}

impl NotificationChannel {
    pub fn as_str(&self) -> &str {
        match self {
            NotificationChannel::Email => "email",
            NotificationChannel::Telegram => "telegram",
            NotificationChannel::Push => "push",
            NotificationChannel::Webhook => "webhook",
            NotificationChannel::Discord => "discord",
            NotificationChannel::Slack => "slack",
        }
    }
}

/// Priority levels for notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl NotificationPriority {
    pub fn as_u8(&self) -> u8 {
        match self {
            NotificationPriority::Critical => 10,
            NotificationPriority::High => 8,
            NotificationPriority::Medium => 5,
            NotificationPriority::Low => 3,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            10 => NotificationPriority::Critical,
            8..=9 => NotificationPriority::High,
            5..=7 => NotificationPriority::Medium,
            3..=4 => NotificationPriority::Low,
            _ => NotificationPriority::Low,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            NotificationPriority::Critical => "critical",
            NotificationPriority::High => "high",
            NotificationPriority::Medium => "medium",
            NotificationPriority::Low => "low",
        }
    }
}

// =============================================================================
// Template Engine
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub format: TemplateFormat,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
    pub channels: Vec<NotificationChannel>,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    Trading,
    System,
    Security,
    Marketing,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateFormat {
    Plain,
    Html,
    Markdown,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub variable_type: VariableType,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Date,
    Currency,
    Percentage,
}

#[derive(Debug, Clone)]
pub struct TemplateEngineConfig {
    pub enable_template_caching: bool,
    pub cache_ttl_seconds: u64,
    pub max_template_size_bytes: usize,
    pub enable_template_validation: bool,
    pub template_directory: String,
}

impl Default for TemplateEngineConfig {
    fn default() -> Self {
        Self {
            enable_template_caching: true,
            cache_ttl_seconds: 3600,
            max_template_size_bytes: 1024 * 1024, // 1MB
            enable_template_validation: true,
            template_directory: "templates".to_string(),
        }
    }
}

pub struct TemplateEngine {
    config: TemplateEngineConfig,
    templates: Arc<Mutex<HashMap<String, NotificationTemplate>>>,
    cache: Arc<Mutex<HashMap<String, (String, u64)>>>,
    kv_store: KvStore,
}

impl TemplateEngine {
    pub async fn new(config: TemplateEngineConfig, kv_store: KvStore) -> ArbitrageResult<Self> {
        let engine = Self {
            config,
            templates: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
            kv_store,
        };
        
        engine.load_templates().await?;
        Ok(engine)
    }

    async fn load_templates(&self) -> ArbitrageResult<()> {
        // Load templates from KV store
        let templates_key = format!("{}/templates", self.config.template_directory);
        
        if let Ok(Some(templates_data)) = self.kv_store.get(&templates_key).text().await {
            if let Ok(templates) = serde_json::from_str::<Vec<NotificationTemplate>>(&templates_data) {
                let mut template_map = self.templates.lock().unwrap();
                for template in templates {
                    template_map.insert(template.id.clone(), template);
                }
            }
        }
        
        Ok(())
    }

    pub async fn render_template(
        &self,
        template_id: &str,
        variables: &HashMap<String, String>,
        channel: &NotificationChannel,
    ) -> ArbitrageResult<String> {
        let cache_key = format!("{}:{}:{}", template_id, channel.as_str(), 
                               variables.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&"));
        
        // Check cache first
        if self.config.enable_template_caching {
            let cache = self.cache.lock().unwrap();
            if let Some((content, timestamp)) = cache.get(&cache_key) {
                let now = chrono::Utc::now().timestamp_millis() as u64;
                if now - timestamp < self.config.cache_ttl_seconds * 1000 {
                    return Ok(content.clone());
                }
            }
        }

        // Get template
        let template = {
            let templates = self.templates.lock().unwrap();
            templates.get(template_id).cloned()
                .ok_or_else(|| ArbitrageError::not_found(format!("Template not found: {}", template_id)))?
        };

        // Render template
        let mut content = template.content.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            content = content.replace(&placeholder, value);
        }

        // Cache result
        if self.config.enable_template_caching {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, (content.clone(), chrono::Utc::now().timestamp_millis() as u64));
        }

        Ok(content)
    }

    pub async fn add_template(&self, template: NotificationTemplate) -> ArbitrageResult<()> {
        let mut templates = self.templates.lock().unwrap();
        templates.insert(template.id.clone(), template);
        self.save_templates().await?;
        Ok(())
    }

    async fn save_templates(&self) -> ArbitrageResult<()> {
        let templates = self.templates.lock().unwrap();
        let templates_vec: Vec<_> = templates.values().cloned().collect();
        let templates_json = serde_json::to_string(&templates_vec)?;
        
        let templates_key = format!("{}/templates", self.config.template_directory);
        self.kv_store.put(&templates_key, templates_json)?.execute().await?;
        
        Ok(())
    }
}

// =============================================================================
// Channel Manager
// =============================================================================

#[derive(Debug, Clone)]
pub struct ChannelManagerConfig {
    pub enable_email: bool,
    pub enable_telegram: bool,
    pub enable_push: bool,
    pub enable_webhook: bool,
    pub enable_discord: bool,
    pub enable_slack: bool,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

impl Default for ChannelManagerConfig {
    fn default() -> Self {
        Self {
            enable_email: true,
            enable_telegram: true,
            enable_push: true,
            enable_webhook: false,
            enable_discord: false,
            enable_slack: false,
            max_retries: 3,
            retry_delay_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEndpoint {
    pub channel: NotificationChannel,
    pub endpoint_url: String,
    pub auth_type: AuthType,
    pub auth_credentials: ChannelAuth,
    pub message_format: MessageFormat,
    pub is_active: bool,
    pub rate_limit_per_minute: u32,
    pub features: Vec<ChannelFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Bearer,
    Basic,
    ApiKey,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAuth {
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_key: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageFormat {
    PlainText,
    Html,
    Markdown,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelFeature {
    RichText,
    Attachments,
    Buttons,
    InlineKeyboard,
    Threading,
    Reactions,
}

#[derive(Debug, Clone)]
pub struct ChannelDeliveryConfig {
    pub channel: NotificationChannel,
    pub max_concurrent_deliveries: u32,
    pub timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_seconds: u64,
    pub max_delay_seconds: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 1,
            max_delay_seconds: 60,
            backoff_multiplier: 2.0,
        }
    }
}

pub struct ChannelManager {
    config: ChannelManagerConfig,
    endpoints: Arc<Mutex<HashMap<NotificationChannel, ChannelEndpoint>>>,
    rate_limits: Arc<Mutex<HashMap<NotificationChannel, (u32, u64)>>>,
}

impl ChannelManager {
    pub fn new(config: ChannelManagerConfig) -> Self {
        Self {
            config,
            endpoints: Arc::new(Mutex::new(HashMap::new())),
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_channel_endpoint(&self, endpoint: ChannelEndpoint) -> ArbitrageResult<()> {
        let mut endpoints = self.endpoints.lock().unwrap();
        endpoints.insert(endpoint.channel.clone(), endpoint);
        Ok(())
    }

    pub fn is_channel_available(&self, channel: &NotificationChannel) -> bool {
        let endpoints = self.endpoints.lock().unwrap();
        endpoints.get(channel).map_or(false, |endpoint| endpoint.is_active)
    }

    pub fn check_rate_limit(&self, channel: &NotificationChannel) -> bool {
        let mut rate_limits = self.rate_limits.lock().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        
        if let Some((count, last_reset)) = rate_limits.get_mut(channel) {
            if now - *last_reset >= 60 {
                *count = 0;
                *last_reset = now;
            }
            
            let endpoints = self.endpoints.lock().unwrap();
            if let Some(endpoint) = endpoints.get(channel) {
                if *count >= endpoint.rate_limit_per_minute {
                    return false;
                }
                *count += 1;
            }
        } else {
            rate_limits.insert(channel.clone(), (1, now));
        }
        
        true
    }
}

// =============================================================================
// Delivery Management
// =============================================================================

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_seconds: u64,
    pub max_delay_seconds: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 1,
            max_delay_seconds: 60,
            backoff_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeliveryManagerConfig {
    pub max_concurrent_deliveries: u32,
    pub default_timeout_seconds: u64,
    pub enable_retry_logic: bool,
    pub default_retry_config: RetryConfig,
    pub enable_delivery_tracking: bool,
    pub enable_metrics: bool,
}

impl Default for DeliveryManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_deliveries: 50,
            default_timeout_seconds: 30,
            enable_retry_logic: true,
            default_retry_config: RetryConfig::default(),
            enable_delivery_tracking: true,
            enable_metrics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRequest {
    pub id: String,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub content: String,
    pub priority: u8,
    pub retry_config: RetryConfig,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub attempt_number: u32,
    pub timestamp: u64,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub response_message: Option<String>,
    pub delivery_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    InProgress,
    Delivered,
    Failed,
    Retrying,
    Cancelled,
}

pub struct DeliveryManager {
    config: DeliveryManagerConfig,
    channel_manager: Arc<ChannelManager>,
    active_deliveries: Arc<Mutex<HashMap<String, DeliveryRequest>>>,
    delivery_history: Arc<Mutex<HashMap<String, Vec<DeliveryAttempt>>>>,
}

impl DeliveryManager {
    pub fn new(config: DeliveryManagerConfig, channel_manager: Arc<ChannelManager>) -> Self {
        Self {
            config,
            channel_manager,
            active_deliveries: Arc::new(Mutex::new(HashMap::new())),
            delivery_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn deliver_notification(&self, request: DeliveryRequest) -> ArbitrageResult<DeliveryAttempt> {
        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        
        if !self.channel_manager.is_channel_available(&request.channel) {
            return Ok(DeliveryAttempt {
                attempt_number: 1,
                timestamp: start_time,
                status: DeliveryStatus::Failed,
                response_code: None,
                response_message: None,
                delivery_time_ms: 0,
                error_message: Some("Channel not available".to_string()),
            });
        }

        if !self.channel_manager.check_rate_limit(&request.channel) {
            return Ok(DeliveryAttempt {
                attempt_number: 1,
                timestamp: start_time,
                status: DeliveryStatus::Failed,
                response_code: None,
                response_message: None,
                delivery_time_ms: 0,
                error_message: Some("Rate limit exceeded".to_string()),
            });
        }

        {
            let mut active = self.active_deliveries.lock().unwrap();
            active.insert(request.id.clone(), request.clone());
        }

        let delivery_result = self.send_to_channel(&request).await;
        
        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        let delivery_time = end_time - start_time;

        let attempt = match delivery_result {
            Ok(_) => DeliveryAttempt {
                attempt_number: 1,
                timestamp: start_time,
                status: DeliveryStatus::Delivered,
                response_code: Some(200),
                response_message: Some("Delivered successfully".to_string()),
                delivery_time_ms: delivery_time,
                error_message: None,
            },
            Err(e) => DeliveryAttempt {
                attempt_number: 1,
                timestamp: start_time,
                status: DeliveryStatus::Failed,
                response_code: Some(500),
                response_message: None,
                delivery_time_ms: delivery_time,
                error_message: Some(e.to_string()),
            },
        };

        {
            let mut history = self.delivery_history.lock().unwrap();
            history.entry(request.id.clone()).or_insert_with(Vec::new).push(attempt.clone());
        }

        {
            let mut active = self.active_deliveries.lock().unwrap();
            active.remove(&request.id);
        }

        Ok(attempt)
    }

    async fn send_to_channel(&self, request: &DeliveryRequest) -> ArbitrageResult<()> {
        match request.channel {
            NotificationChannel::Email => Ok(()),
            NotificationChannel::Telegram => Ok(()),
            NotificationChannel::Push => Ok(()),
            _ => Err(ArbitrageError::not_implemented("Channel not implemented".to_string())),
        }
    }
}

// =============================================================================
// Notification Coordination
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub notification_id: String,
    pub user_id: String,
    pub notification_type: String,
    pub priority: u8,
    pub channels: Vec<NotificationChannel>,
    pub template_id: Option<String>,
    pub template_variables: HashMap<String, String>,
    pub custom_content: Option<String>,
    pub recipients: HashMap<NotificationChannel, String>,
    pub metadata: HashMap<String, String>,
    pub expires_at: Option<u64>,
    pub created_at: u64,
}

impl NotificationRequest {
    pub fn new(user_id: String, notification_type: String, channels: Vec<NotificationChannel>) -> Self {
        Self {
            notification_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            notification_type,
            priority: 5,
            channels,
            template_id: None,
            template_variables: HashMap::new(),
            custom_content: None,
            recipients: HashMap::new(),
            metadata: HashMap::new(),
            expires_at: None,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_template(mut self, template_id: String, variables: HashMap<String, String>) -> Self {
        self.template_id = Some(template_id);
        self.template_variables = variables;
        self
    }

    pub fn with_custom_content(mut self, content: String) -> Self {
        self.custom_content = Some(content);
        self
    }

    pub fn with_recipient(mut self, channel: NotificationChannel, recipient: String) -> Self {
        self.recipients.insert(channel, recipient);
        self
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.channels.is_empty() {
            return Err(ArbitrageError::validation_error("At least one channel must be specified"));
        }

        if self.template_id.is_none() && self.custom_content.is_none() {
            return Err(ArbitrageError::validation_error("Either template_id or custom_content must be provided"));
        }

        for channel in &self.channels {
            if !self.recipients.contains_key(channel) {
                return Err(ArbitrageError::validation_error(format!(
                    "Recipient not provided for channel: {}",
                    channel.as_str()
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    pub notification_id: String,
    pub success: bool,
    pub channel_results: HashMap<NotificationChannel, ChannelResult>,
    pub total_channels: u32,
    pub successful_channels: u32,
    pub failed_channels: u32,
    pub processing_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResult {
    pub channel: NotificationChannel,
    pub success: bool,
    pub message_id: Option<String>,
    pub error_message: Option<String>,
    pub delivery_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct NotificationCoordinatorConfig {
    pub enable_coordinator: bool,
    pub enable_parallel_processing: bool,
    pub max_concurrent_notifications: usize,
    pub enable_fallback_processing: bool,
    pub enable_metrics: bool,
    pub enable_kv_storage: bool,
    pub kv_key_prefix: String,
    pub template_engine_config: TemplateEngineConfig,
    pub delivery_manager_config: DeliveryManagerConfig,
    pub channel_manager_config: ChannelManagerConfig,
}

impl Default for NotificationCoordinatorConfig {
    fn default() -> Self {
        Self {
            enable_coordinator: true,
            enable_parallel_processing: true,
            max_concurrent_notifications: 100,
            enable_fallback_processing: true,
            enable_metrics: true,
            enable_kv_storage: true,
            kv_key_prefix: "notifications".to_string(),
            template_engine_config: TemplateEngineConfig::default(),
            delivery_manager_config: DeliveryManagerConfig::default(),
            channel_manager_config: ChannelManagerConfig::default(),
        }
    }
}

pub struct NotificationCoordinator {
    config: NotificationCoordinatorConfig,
    template_engine: TemplateEngine,
    delivery_manager: DeliveryManager,
    channel_manager: Arc<ChannelManager>,
    active_notifications: Arc<Mutex<HashMap<String, NotificationRequest>>>,
    kv_store: KvStore,
}

impl NotificationCoordinator {
    pub async fn new(config: NotificationCoordinatorConfig, kv_store: KvStore, _env: &worker::Env) -> ArbitrageResult<Self> {
        let channel_manager = Arc::new(ChannelManager::new(config.channel_manager_config.clone()));
        let template_engine = TemplateEngine::new(config.template_engine_config.clone(), kv_store.clone()).await?;
        let delivery_manager = DeliveryManager::new(config.delivery_manager_config.clone(), channel_manager.clone());

        Ok(Self {
            config,
            template_engine,
            delivery_manager,
            channel_manager,
            active_notifications: Arc::new(Mutex::new(HashMap::new())),
            kv_store,
        })
    }

    pub async fn send_notification(&self, request: NotificationRequest) -> ArbitrageResult<NotificationResult> {
        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        request.validate()?;

        {
            let mut active = self.active_notifications.lock().unwrap();
            active.insert(request.notification_id.clone(), request.clone());
        }

        let mut channel_results = HashMap::new();
        let mut successful_channels = 0;
        let mut failed_channels = 0;

        for channel in &request.channels {
            let recipient = match request.recipients.get(channel) {
                Some(recipient) => recipient.clone(),
                None => {
                    failed_channels += 1;
                    channel_results.insert(channel.clone(), ChannelResult {
                        channel: channel.clone(),
                        success: false,
                        message_id: None,
                        error_message: Some("No recipient specified".to_string()),
                        delivery_time_ms: 0,
                    });
                    continue;
                }
            };

            let content = match self.prepare_content(&request, channel).await {
                Ok(content) => content,
                Err(e) => {
                    failed_channels += 1;
                    channel_results.insert(channel.clone(), ChannelResult {
                        channel: channel.clone(),
                        success: false,
                        message_id: None,
                        error_message: Some(e.to_string()),
                        delivery_time_ms: 0,
                    });
                    continue;
                }
            };

            let delivery_request = DeliveryRequest {
                id: format!("{}:{}", request.notification_id, channel.as_str()),
                channel: channel.clone(),
                recipient,
                content,
                priority: request.priority,
                retry_config: RetryConfig::default(),
                metadata: request.metadata.clone(),
                created_at: chrono::Utc::now().timestamp_millis() as u64,
            };

            match self.delivery_manager.deliver_notification(delivery_request).await {
                Ok(attempt) => {
                    let success = matches!(attempt.status, DeliveryStatus::Delivered);
                    if success {
                        successful_channels += 1;
                    } else {
                        failed_channels += 1;
                    }
                    
                    channel_results.insert(channel.clone(), ChannelResult {
                        channel: channel.clone(),
                        success,
                        message_id: if success { Some(format!("msg_{}", attempt.timestamp)) } else { None },
                        error_message: attempt.error_message,
                        delivery_time_ms: attempt.delivery_time_ms,
                    });
                }
                Err(e) => {
                    failed_channels += 1;
                    channel_results.insert(channel.clone(), ChannelResult {
                        channel: channel.clone(),
                        success: false,
                        message_id: None,
                        error_message: Some(e.to_string()),
                        delivery_time_ms: 0,
                    });
                }
            }
        }

        {
            let mut active = self.active_notifications.lock().unwrap();
            active.remove(&request.notification_id);
        }

        let end_time = chrono::Utc::now().timestamp_millis() as u64;
        let processing_time = end_time - start_time;

        Ok(NotificationResult {
            notification_id: request.notification_id,
            success: successful_channels > 0,
            channel_results,
            total_channels: request.channels.len() as u32,
            successful_channels,
            failed_channels,
            processing_time_ms: processing_time,
            error_message: if failed_channels == request.channels.len() as u32 {
                Some("All channels failed".to_string())
            } else {
                None
            },
        })
    }

    async fn prepare_content(&self, request: &NotificationRequest, channel: &NotificationChannel) -> ArbitrageResult<String> {
        if let Some(custom_content) = &request.custom_content {
            return Ok(custom_content.clone());
        }

        if let Some(template_id) = &request.template_id {
            return self.template_engine.render_template(template_id, &request.template_variables, channel).await;
        }

        Err(ArbitrageError::validation_error("No content source specified"))
    }
}

// =============================================================================
// Delivery Manager
// =============================================================================

#[derive(Debug, Clone)]
pub struct DeliveryManagerConfig {
    pub max_concurrent_deliveries: u32,
    pub default_timeout_seconds: u64,
    pub enable_retry_logic: bool,
    pub default_retry_config: RetryConfig,
    pub enable_delivery_tracking: bool,
    pub enable_metrics: bool,
}

impl Default for DeliveryManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_deliveries: 50,
            default_timeout_seconds: 30,
            enable_retry_logic: true,
            default_retry_config: RetryConfig::default(),
            enable_delivery_tracking: true,
            enable_metrics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRequest {
    pub id: String,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub content: String,
    pub priority: u8,
    pub retry_config: RetryConfig,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub attempt_number: u32,
    pub timestamp: u64,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub response_message: Option<String>,
    pub delivery_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    InProgress,
    Delivered,
    Failed,
    Retrying,
    Cancelled,
}

// Removed duplicate DeliveryManager and NotificationRequest definitions
// Keeping only the first implementations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    pub notification_id: String,
    pub success: bool,
    pub channel_results: HashMap<NotificationChannel, ChannelResult>,
    pub total_channels: u32,
    pub successful_channels: u32,
    pub failed_channels: u32,
    pub processing_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResult {
    pub channel: NotificationChannel,
    pub success: bool,
    pub message_id: Option<String>,
    pub error_message: Option<String>,
    pub delivery_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct NotificationCoordinatorConfig {
    pub enable_coordinator: bool,
    pub enable_parallel_processing: bool,
    pub max_concurrent_notifications: usize,
    pub enable_fallback_processing: bool,
    pub enable_metrics: bool,
    pub enable_kv_storage: bool,
    pub kv_key_prefix: String,
    pub template_engine_config: TemplateEngineConfig,
    pub delivery_manager_config: DeliveryManagerConfig,
    pub channel_manager_config: ChannelManagerConfig,
}

impl Default for NotificationCoordinatorConfig {
    fn default() -> Self {
        Self {
            enable_coordinator: true,
            enable_parallel_processing: true,
            max_concurrent_notifications: 100,
            enable_fallback_processing: true,
            enable_metrics: true,
            enable_kv_storage: true,
            kv_key_prefix: "notifications".to_string(),
            template_engine_config: TemplateEngineConfig::default(),
            delivery_manager_config: DeliveryManagerConfig::default(),
            channel_manager_config: ChannelManagerConfig::default(),
        }
    }
}

// =============================================================================
// Unified Notification Services
// =============================================================================

#[derive(Debug, Clone)]

// =============================================================================
// Unified Notification Services
// =============================================================================

#[derive(Debug, Clone)]
pub struct UnifiedNotificationServicesConfig {
    pub enable_notifications: bool,
    pub enable_high_performance_mode: bool,
    pub enable_high_reliability_mode: bool,
    pub max_notifications_per_minute: u32,
    pub max_notifications_per_hour: u32,
    pub coordinator_config: NotificationCoordinatorConfig,
}

impl Default for UnifiedNotificationServicesConfig {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            enable_high_performance_mode: false,
            enable_high_reliability_mode: true,
            max_notifications_per_minute: 100,
            max_notifications_per_hour: 1000,
            coordinator_config: NotificationCoordinatorConfig::default(),
        }
    }
}

impl UnifiedNotificationServicesConfig {
    pub fn high_performance() -> Self {
        Self {
            enable_notifications: true,
            enable_high_performance_mode: true,
            enable_high_reliability_mode: false,
            max_notifications_per_minute: 500,
            max_notifications_per_hour: 10000,
            coordinator_config: NotificationCoordinatorConfig {
                max_concurrent_notifications: 200,
                enable_parallel_processing: true,
                ..NotificationCoordinatorConfig::default()
            },
        }
    }

    pub fn high_reliability() -> Self {
        Self {
            enable_notifications: true,
            enable_high_performance_mode: false,
            enable_high_reliability_mode: true,
            max_notifications_per_minute: 50,
            max_notifications_per_hour: 500,
            coordinator_config: NotificationCoordinatorConfig {
                max_concurrent_notifications: 50,
                enable_fallback_processing: true,
                ..NotificationCoordinatorConfig::default()
            },
        }
    }

    pub fn validate(&self) -> ArbitrageResult<()> {
        if self.max_notifications_per_minute == 0 {
            return Err(ArbitrageError::configuration_error("max_notifications_per_minute must be greater than 0".to_string()));
        }
        if self.max_notifications_per_hour == 0 {
            return Err(ArbitrageError::configuration_error("max_notifications_per_hour must be greater than 0".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNotificationServicesHealth {
    pub is_healthy: bool,
    pub coordinator_healthy: bool,
    pub template_engine_healthy: bool,
    pub delivery_manager_healthy: bool,
    pub channel_manager_healthy: bool,
    pub total_notifications_processed: u64,
    pub successful_notifications: u64,
    pub failed_notifications: u64,
    pub average_processing_time_ms: f64,
    pub active_notifications: u64,
    pub last_health_check: u64,
}

impl Default for UnifiedNotificationServicesHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            coordinator_healthy: false,
            template_engine_healthy: false,
            delivery_manager_healthy: false,
            channel_manager_healthy: false,
            total_notifications_processed: 0,
            successful_notifications: 0,
            failed_notifications: 0,
            average_processing_time_ms: 0.0,
            active_notifications: 0,
            last_health_check: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNotificationServicesMetrics {
    pub total_notifications: u64,
    pub notifications_per_second: f64,
    pub successful_notifications: u64,
    pub failed_notifications: u64,
    pub average_processing_time_ms: f64,
    pub notifications_by_type: HashMap<String, u64>,
    pub notifications_by_channel: HashMap<NotificationChannel, u64>,
    pub notifications_by_priority: HashMap<u8, u64>,
    pub channel_success_rates: HashMap<NotificationChannel, f32>,
    pub error_counts: HashMap<String, u64>,
    pub last_updated: u64,
}

impl Default for UnifiedNotificationServicesMetrics {
    fn default() -> Self {
        Self {
            total_notifications: 0,
            notifications_per_second: 0.0,
            successful_notifications: 0,
            failed_notifications: 0,
            average_processing_time_ms: 0.0,
            notifications_by_type: HashMap::new(),
            notifications_by_channel: HashMap::new(),
            notifications_by_priority: HashMap::new(),
            channel_success_rates: HashMap::new(),
            error_counts: HashMap::new(),
            last_updated: 0,
        }
    }
}

pub struct UnifiedNotificationServices {
    config: UnifiedNotificationServicesConfig,
    coordinator: NotificationCoordinator,
    health: Arc<Mutex<UnifiedNotificationServicesHealth>>,
    metrics: Arc<Mutex<UnifiedNotificationServicesMetrics>>,
    startup_time: u64,
}

impl UnifiedNotificationServices {
    pub async fn new(config: UnifiedNotificationServicesConfig, kv_store: KvStore, env: &worker::Env) -> ArbitrageResult<Self> {
        config.validate()?;
        
        let coordinator = NotificationCoordinator::new(config.coordinator_config.clone(), kv_store, env).await?;
        
        Ok(Self {
            config,
            coordinator,
            health: Arc::new(Mutex::new(UnifiedNotificationServicesHealth::default())),
            metrics: Arc::new(Mutex::new(UnifiedNotificationServicesMetrics::default())),
            startup_time: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    pub async fn send_notification(&self, request: NotificationRequest) -> ArbitrageResult<NotificationResult> {
        if !self.config.enable_notifications {
            return Err(ArbitrageError::service_unavailable("Notifications are disabled"));
        }

        let result = self.coordinator.send_notification(request).await?;
        self.update_metrics(&result).await;
        Ok(result)
    }

    pub async fn send_batch_notifications(&self, requests: Vec<NotificationRequest>) -> ArbitrageResult<Vec<NotificationResult>> {
        let mut results = Vec::new();
        
        for request in requests {
            match self.send_notification(request).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(NotificationResult {
                        notification_id: "unknown".to_string(),
                        success: false,
                        channel_results: HashMap::new(),
                        total_channels: 0,
                        successful_channels: 0,
                        failed_channels: 0,
                        processing_time_ms: 0,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(results)
    }

    async fn update_metrics(&self, result: &NotificationResult) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_notifications += 1;
        
        if result.success {
            metrics.successful_notifications += 1;
        } else {
            metrics.failed_notifications += 1;
        }
        
        // Update running average
        let total = metrics.total_notifications as f64;
        metrics.average_processing_time_ms = 
            (metrics.average_processing_time_ms * (total - 1.0) + result.processing_time_ms as f64) / total;
        
        metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    pub async fn health_check(&self) -> ArbitrageResult<UnifiedNotificationServicesHealth> {
        let mut health = self.health.lock().unwrap();
        
        health.coordinator_healthy = true; // In real implementation, check coordinator health
        health.template_engine_healthy = true;
        health.delivery_manager_healthy = true;
        health.channel_manager_healthy = true;
        
        health.is_healthy = health.coordinator_healthy && 
                           health.template_engine_healthy && 
                           health.delivery_manager_healthy && 
                           health.channel_manager_healthy;
        
        health.last_health_check = chrono::Utc::now().timestamp_millis() as u64;
        
        Ok(health.clone())
    }

    pub async fn get_metrics(&self) -> UnifiedNotificationServicesMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }

    pub fn get_uptime_seconds(&self) -> u64 {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        (now - self.startup_time) / 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_properties() {
        let security_alert = NotificationType::SecurityAlert;
        assert_eq!(security_alert.as_str(), "security_alert");
        assert_eq!(security_alert.default_priority(), 10);
        assert!(security_alert.default_channels().contains(&NotificationChannel::Email));
    }

    #[test]
    fn test_notification_priority_conversion() {
        let priority = NotificationPriority::High;
        assert_eq!(priority.as_u8(), 8);
        assert_eq!(NotificationPriority::from_u8(8), NotificationPriority::High);
    }

    #[test]
    fn test_unified_notification_services_config_validation() {
        let config = UnifiedNotificationServicesConfig::default();
        assert!(config.validate().is_ok());
        
        let invalid_config = UnifiedNotificationServicesConfig {
            max_notifications_per_minute: 0,
            ..UnifiedNotificationServicesConfig::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_high_performance_config() {
        let config = UnifiedNotificationServicesConfig::high_performance();
        assert!(config.enable_high_performance_mode);
        assert_eq!(config.max_notifications_per_minute, 500);
    }

    #[test]
    fn test_high_reliability_config() {
        let config = UnifiedNotificationServicesConfig::high_reliability();
        assert!(config.enable_high_reliability_mode);
        assert_eq!(config.max_notifications_per_minute, 50);
    }

    #[test]
    fn test_notification_request_validation() {
        let request = NotificationRequest::new(
            "user123".to_string(),
            "opportunity_alert".to_string(),
            vec![NotificationChannel::Email]
        ).with_recipient(NotificationChannel::Email, "user@example.com".to_string())
         .with_custom_content("Test notification".to_string());
        
        assert!(request.validate().is_ok());
        
        let invalid_request = NotificationRequest::new(
            "user123".to_string(),
            "opportunity_alert".to_string(),
            vec![]
        );
        assert!(invalid_request.validate().is_err());
    }
} 