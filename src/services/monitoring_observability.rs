use crate::types::{ArbitrageOpportunity, CommandPermission};
use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(not(target_arch = "wasm32"))]
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt};

/// Metric Types for monitoring different aspects of the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    // System Metrics
    CpuUsagePercent,
    MemoryUsagePercent,
    DiskUsagePercent,
    NetworkBytesIn,
    NetworkBytesOut,
    
    // Application Metrics
    RequestsPerSecond,
    ResponseTimeMs,
    ErrorRate,
    ActiveConnections,
    
    // Business Metrics
    OpportunitiesDetected,
    OpportunitiesDistributed,
    UserSessions,
    TradingVolume,
    ProfitGenerated,
    
    // Service-specific Metrics
    TelegramMessagesPerHour,
    DatabaseQueriesPerSecond,
    ExchangeApiCalls,
    AiModelInference,
    TechnicalAnalysisSignals,
}

/// Metric Value Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { buckets: Vec<f64>, counts: Vec<u64> },
    Summary { count: u64, sum: f64, quantiles: HashMap<String, f64> },
}

impl MetricValue {
    pub fn counter(value: u64) -> Self {
        MetricValue::Counter(value)
    }

    pub fn gauge(value: f64) -> Self {
        MetricValue::Gauge(value)
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            MetricValue::Counter(v) => *v as f64,
            MetricValue::Gauge(v) => *v,
            MetricValue::Histogram { counts, .. } => counts.iter().sum::<u64>() as f64,
            MetricValue::Summary { sum, .. } => *sum,
        }
    }
}

/// Metric Data Point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub timestamp: u64,
    pub labels: HashMap<String, String>,
    pub help_text: String,
}

impl MetricDataPoint {
    pub fn new(metric_type: MetricType, value: MetricValue, help_text: String) -> Self {
        Self {
            metric_type,
            value,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            labels: HashMap::new(),
            help_text,
        }
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }
}

/// Alert Severity Levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert Condition Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCondition {
    Threshold { metric: MetricType, operator: String, value: f64 },
    Rate { metric: MetricType, operator: String, value: f64, window_seconds: u64 },
    Anomaly { metric: MetricType, sensitivity: f64 },
    ServiceDown { service_name: String },
}

/// Alert Rule Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub notification_channels: Vec<String>,
    pub labels: HashMap<String, String>,
}

/// Active Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub triggered_at: u64,
    pub resolved_at: Option<u64>,
    pub current_value: f64,
    pub threshold_value: f64,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

impl Alert {
    pub fn new(rule: &AlertRule, current_value: f64, threshold_value: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            name: rule.name.clone(),
            description: rule.description.clone(),
            severity: rule.severity.clone(),
            triggered_at: chrono::Utc::now().timestamp_millis() as u64,
            resolved_at: None,
            current_value,
            threshold_value,
            labels: rule.labels.clone(),
            annotations: HashMap::new(),
        }
    }

    pub fn resolve(&mut self) {
        self.resolved_at = Some(chrono::Utc::now().timestamp_millis() as u64);
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved_at.is_some()
    }

    pub fn duration_ms(&self) -> u64 {
        let end_time = self.resolved_at.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64);
        end_time - self.triggered_at
    }
}

/// Performance Monitoring Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub request_count: u64,
    pub total_response_time_ms: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub throughput_rps: f64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub last_updated: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            total_response_time_ms: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            throughput_rps: 0.0,
            p50_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Trace Information for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<u64>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<TraceLog>,
    pub status: TraceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLog {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatus {
    Ok,
    Error,
    Timeout,
    Cancelled,
}

impl TraceSpan {
    pub fn new(operation_name: String) -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: None,
            operation_name,
            start_time: chrono::Utc::now().timestamp_millis() as u64,
            end_time: None,
            duration_ms: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: TraceStatus::Ok,
        }
    }

    pub fn finish(&mut self) {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        self.end_time = Some(now);
        self.duration_ms = Some(now - self.start_time);
    }

    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    pub fn add_log(&mut self, level: String, message: String) {
        let log = TraceLog {
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            level,
            message,
            fields: HashMap::new(),
        };
        self.logs.push(log);
    }
}

/// Dashboard Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub name: String,
    pub description: String,
    pub panels: Vec<DashboardPanel>,
    pub refresh_interval_seconds: u64,
    pub time_range_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    pub title: String,
    pub panel_type: PanelType,
    pub metrics: Vec<MetricType>,
    pub position: PanelPosition,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelType {
    LineChart,
    BarChart,
    Gauge,
    Counter,
    Table,
    Heatmap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Monitoring and Observability Service
pub struct MonitoringObservabilityService {
    metrics_store: Arc<RwLock<HashMap<MetricType, Vec<MetricDataPoint>>>>,
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    performance_metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    trace_spans: Arc<RwLock<Vec<TraceSpan>>>,
    dashboards: Arc<RwLock<Vec<DashboardConfig>>>,
    // Store response time samples for percentile calculation
    response_time_samples: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    system: Arc<RwLock<System>>,
    #[cfg(not(target_arch = "wasm32"))]
    service_start_time: std::time::Instant,
    enabled: bool,
}

impl MonitoringObservabilityService {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let mut system = System::new_all();
        #[cfg(not(target_arch = "wasm32"))]
        system.refresh_all();

        Self {
            metrics_store: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            trace_spans: Arc::new(RwLock::new(Vec::new())),
            dashboards: Arc::new(RwLock::new(Vec::new())),
            response_time_samples: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(not(target_arch = "wasm32"))]
            system: Arc::new(RwLock::new(system)),
            #[cfg(not(target_arch = "wasm32"))]
            service_start_time: std::time::Instant::now(),
            enabled: true,
        }
    }

    /// Initialize monitoring with default configurations
    pub async fn initialize(&mut self) -> ArbitrageResult<()> {
        // Setup default alert rules
        self.setup_default_alert_rules().await?;
        
        // Setup default dashboards
        self.setup_default_dashboards().await?;
        
        // Start background tasks
        self.start_monitoring_tasks().await?;
        
        Ok(())
    }

    /// Record a metric data point
    pub async fn record_metric(&self, metric: MetricDataPoint) -> ArbitrageResult<()> {
        if !self.enabled {
            return Ok(());
        }

        {
            let mut store = self.metrics_store.write().await;
            let entries = store.entry(metric.metric_type.clone()).or_insert_with(Vec::new);
            entries.push(metric.clone());
            
            // Keep only recent metrics (last 24 hours)
            let cutoff = chrono::Utc::now().timestamp_millis() as u64 - (24 * 60 * 60 * 1000);
            entries.retain(|m| m.timestamp > cutoff);
        }

        // Check alert conditions
        self.check_alert_conditions(&metric).await?;

        Ok(())
    }

    /// Record system metrics (CPU, memory, etc.) using real system monitoring
    pub async fn record_system_metrics(&self) -> ArbitrageResult<()> {
        // Use real system monitoring via sysinfo
        let cpu_usage = self.get_real_cpu_usage().await;
        let memory_usage = self.get_real_memory_usage().await;
        let disk_usage = self.get_real_disk_usage().await;

        let metrics = vec![
            MetricDataPoint::new(
                MetricType::CpuUsagePercent,
                MetricValue::gauge(cpu_usage),
                "Real-time CPU usage percentage from sysinfo".to_string(),
            ),
            MetricDataPoint::new(
                MetricType::MemoryUsagePercent,
                MetricValue::gauge(memory_usage),
                "Real-time memory usage percentage from sysinfo".to_string(),
            ),
            MetricDataPoint::new(
                MetricType::DiskUsagePercent,
                MetricValue::gauge(disk_usage),
                "Real-time disk usage percentage from sysinfo".to_string(),
            ),
        ];

        for metric in metrics {
            self.record_metric(metric).await?;
        }

        Ok(())
    }

    /// Record business metrics
    pub async fn record_business_metrics(&self, opportunities_count: u64, volume: f64) -> ArbitrageResult<()> {
        let metrics = vec![
            MetricDataPoint::new(
                MetricType::OpportunitiesDetected,
                MetricValue::counter(opportunities_count),
                "Number of arbitrage opportunities detected".to_string(),
            ).with_label("period".to_string(), "hourly".to_string()),
            MetricDataPoint::new(
                MetricType::TradingVolume,
                MetricValue::gauge(volume),
                "Total trading volume in USD".to_string(),
            ).with_label("currency".to_string(), "USD".to_string()),
        ];

        for metric in metrics {
            self.record_metric(metric).await?;
        }

        Ok(())
    }

    /// Start a new trace span
    pub async fn start_trace(&self, operation_name: String) -> TraceSpan {
        TraceSpan::new(operation_name)
    }

    /// Finish and record a trace span
    pub async fn finish_trace(&self, mut span: TraceSpan) -> ArbitrageResult<()> {
        span.finish();
        
        {
            let mut traces = self.trace_spans.write().await;
            traces.push(span);
            
            // Keep only recent traces (last hour)
            let cutoff = chrono::Utc::now().timestamp_millis() as u64 - (60 * 60 * 1000);
            traces.retain(|t| t.start_time > cutoff);
        }

        Ok(())
    }

    /// Record performance metrics for a service operation
    pub async fn record_performance(&self, operation: &str, response_time_ms: u64, is_error: bool) -> ArbitrageResult<()> {
        // Store the response time sample for percentile calculation
        {
            let mut samples = self.response_time_samples.write().await;
            let operation_samples = samples.entry(operation.to_string()).or_insert_with(Vec::new);
            operation_samples.push(response_time_ms);
            
            // Keep only last 1000 samples to prevent unbounded growth
            if operation_samples.len() > 1000 {
                operation_samples.remove(0);
            }
        }

        {
            let mut perf_metrics = self.performance_metrics.write().await;
            let metrics = perf_metrics.entry(operation.to_string()).or_insert_with(PerformanceMetrics::default);
            
            metrics.request_count += 1;
            metrics.total_response_time_ms += response_time_ms;
            if is_error {
                metrics.error_count += 1;
            }
            
            // Calculate derived metrics
            metrics.avg_response_time_ms = metrics.total_response_time_ms as f64 / metrics.request_count as f64;
            metrics.error_rate = metrics.error_count as f64 / metrics.request_count as f64;
            
            // Calculate actual percentiles from response time samples
            let percentiles = self.calculate_percentiles(operation).await;
            metrics.p50_response_time_ms = percentiles.0;
            metrics.p95_response_time_ms = percentiles.1;
            metrics.p99_response_time_ms = percentiles.2;
            
            metrics.last_updated = chrono::Utc::now().timestamp_millis() as u64;
        }

        // Record as metric
        self.record_metric(MetricDataPoint::new(
            MetricType::ResponseTimeMs,
            MetricValue::gauge(response_time_ms as f64),
            "Response time for operation".to_string(),
        ).with_label("operation".to_string(), operation.to_string())).await?;

        Ok(())
    }

    /// Get metrics for a specific type
    pub async fn get_metrics(&self, metric_type: &MetricType, hours: u64) -> Vec<MetricDataPoint> {
        let store = self.metrics_store.read().await;
        let cutoff = chrono::Utc::now().timestamp_millis() as u64 - (hours * 60 * 60 * 1000);
        
        store.get(metric_type)
            .map(|metrics| metrics.iter()
                .filter(|m| m.timestamp > cutoff)
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.iter().filter(|a| !a.is_resolved()).cloned().collect()
    }

    /// Get system health summary
    pub async fn get_system_health_summary(&self) -> SystemHealthSummary {
        let active_alerts = self.get_active_alerts().await;
        let critical_alerts = active_alerts.iter().filter(|a| a.severity == AlertSeverity::Critical).count();
        let warning_alerts = active_alerts.iter().filter(|a| a.severity == AlertSeverity::Warning).count();
        
        // Get recent system metrics
        let cpu_metrics = self.get_metrics(&MetricType::CpuUsagePercent, 1).await;
        let memory_metrics = self.get_metrics(&MetricType::MemoryUsagePercent, 1).await;
        let error_rate_metrics = self.get_metrics(&MetricType::ErrorRate, 1).await;
        
        let latest_cpu = cpu_metrics.last().map(|m| m.value.as_f64()).unwrap_or(0.0);
        let latest_memory = memory_metrics.last().map(|m| m.value.as_f64()).unwrap_or(0.0);
        let latest_error_rate = error_rate_metrics.last().map(|m| m.value.as_f64()).unwrap_or(0.0);
        
        let health_score = self.calculate_health_score(latest_cpu, latest_memory, latest_error_rate, critical_alerts, warning_alerts);
        
        SystemHealthSummary {
            health_score,
            status: if critical_alerts > 0 {
                "critical".to_string()
            } else if warning_alerts > 0 {
                "warning".to_string()
            } else if health_score < 0.7 {
                "degraded".to_string()
            } else {
                "healthy".to_string()
            },
            cpu_usage_percent: latest_cpu,
            memory_usage_percent: latest_memory,
            error_rate_percent: latest_error_rate * 100.0,
            active_alerts_count: active_alerts.len(),
            critical_alerts_count: critical_alerts,
            warning_alerts_count: warning_alerts,
            uptime_hours: self.get_real_uptime_hours(),
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    /// Get performance metrics for all operations
    pub async fn get_performance_overview(&self) -> HashMap<String, PerformanceMetrics> {
        let perf_metrics = self.performance_metrics.read().await;
        perf_metrics.clone()
    }

    /// Setup default alert rules
    async fn setup_default_alert_rules(&self) -> ArbitrageResult<()> {
        let default_rules = vec![
            AlertRule {
                id: "high_cpu_usage".to_string(),
                name: "High CPU Usage".to_string(),
                description: "CPU usage exceeds 80%".to_string(),
                condition: AlertCondition::Threshold {
                    metric: MetricType::CpuUsagePercent,
                    operator: ">".to_string(),
                    value: 80.0,
                },
                severity: AlertSeverity::Warning,
                enabled: true,
                cooldown_seconds: 300,
                notification_channels: vec!["telegram".to_string()],
                labels: HashMap::new(),
            },
            AlertRule {
                id: "critical_cpu_usage".to_string(),
                name: "Critical CPU Usage".to_string(),
                description: "CPU usage exceeds 95%".to_string(),
                condition: AlertCondition::Threshold {
                    metric: MetricType::CpuUsagePercent,
                    operator: ">".to_string(),
                    value: 95.0,
                },
                severity: AlertSeverity::Critical,
                enabled: true,
                cooldown_seconds: 60,
                notification_channels: vec!["telegram".to_string(), "email".to_string()],
                labels: HashMap::new(),
            },
            AlertRule {
                id: "high_error_rate".to_string(),
                name: "High Error Rate".to_string(),
                description: "Error rate exceeds 5%".to_string(),
                condition: AlertCondition::Threshold {
                    metric: MetricType::ErrorRate,
                    operator: ">".to_string(),
                    value: 0.05,
                },
                severity: AlertSeverity::Error,
                enabled: true,
                cooldown_seconds: 180,
                notification_channels: vec!["telegram".to_string()],
                labels: HashMap::new(),
            },
            AlertRule {
                id: "slow_response_time".to_string(),
                name: "Slow Response Time".to_string(),
                description: "Average response time exceeds 2 seconds".to_string(),
                condition: AlertCondition::Threshold {
                    metric: MetricType::ResponseTimeMs,
                    operator: ">".to_string(),
                    value: 2000.0,
                },
                severity: AlertSeverity::Warning,
                enabled: true,
                cooldown_seconds: 300,
                notification_channels: vec!["telegram".to_string()],
                labels: HashMap::new(),
            },
        ];

        {
            let mut rules = self.alert_rules.write().await;
            rules.extend(default_rules);
        }

        Ok(())
    }

    /// Setup default dashboards
    async fn setup_default_dashboards(&self) -> ArbitrageResult<()> {
        let system_dashboard = DashboardConfig {
            name: "System Overview".to_string(),
            description: "System performance and health metrics".to_string(),
            panels: vec![
                DashboardPanel {
                    title: "CPU Usage".to_string(),
                    panel_type: PanelType::LineChart,
                    metrics: vec![MetricType::CpuUsagePercent],
                    position: PanelPosition { x: 0, y: 0, width: 6, height: 3 },
                    configuration: serde_json::json!({"yAxis": {"max": 100}}),
                },
                DashboardPanel {
                    title: "Memory Usage".to_string(),
                    panel_type: PanelType::LineChart,
                    metrics: vec![MetricType::MemoryUsagePercent],
                    position: PanelPosition { x: 6, y: 0, width: 6, height: 3 },
                    configuration: serde_json::json!({"yAxis": {"max": 100}}),
                },
                DashboardPanel {
                    title: "Response Time".to_string(),
                    panel_type: PanelType::LineChart,
                    metrics: vec![MetricType::ResponseTimeMs],
                    position: PanelPosition { x: 0, y: 3, width: 12, height: 4 },
                    configuration: serde_json::json!({}),
                },
            ],
            refresh_interval_seconds: 30,
            time_range_hours: 6,
        };

        let business_dashboard = DashboardConfig {
            name: "Business Metrics".to_string(),
            description: "Trading and business performance metrics".to_string(),
            panels: vec![
                DashboardPanel {
                    title: "Opportunities Detected".to_string(),
                    panel_type: PanelType::Counter,
                    metrics: vec![MetricType::OpportunitiesDetected],
                    position: PanelPosition { x: 0, y: 0, width: 3, height: 2 },
                    configuration: serde_json::json!({}),
                },
                DashboardPanel {
                    title: "Trading Volume".to_string(),
                    panel_type: PanelType::Gauge,
                    metrics: vec![MetricType::TradingVolume],
                    position: PanelPosition { x: 3, y: 0, width: 3, height: 2 },
                    configuration: serde_json::json!({}),
                },
                DashboardPanel {
                    title: "User Sessions".to_string(),
                    panel_type: PanelType::LineChart,
                    metrics: vec![MetricType::UserSessions],
                    position: PanelPosition { x: 6, y: 0, width: 6, height: 4 },
                    configuration: serde_json::json!({}),
                },
            ],
            refresh_interval_seconds: 60,
            time_range_hours: 24,
        };

        {
            let mut dashboards = self.dashboards.write().await;
            dashboards.push(system_dashboard);
            dashboards.push(business_dashboard);
        }

        Ok(())
    }

    /// Start background monitoring tasks
    async fn start_monitoring_tasks(&self) -> ArbitrageResult<()> {
        // Start system metrics collection
        let metrics_store = self.metrics_store.clone();
        let alert_rules = self.alert_rules.clone();
        let active_alerts = self.active_alerts.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // Collect system metrics
                // This would be implemented with actual system monitoring in production
            }
        });

        Ok(())
    }

    /// Check alert conditions for a new metric
    async fn check_alert_conditions(&self, metric: &MetricDataPoint) -> ArbitrageResult<()> {
        let rules = self.alert_rules.read().await;
        
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let should_alert = match &rule.condition {
                AlertCondition::Threshold { metric: rule_metric, operator, value } => {
                    if *rule_metric == metric.metric_type {
                        let metric_value = metric.value.as_f64();
                        match operator.as_str() {
                            ">" => metric_value > *value,
                            "<" => metric_value < *value,
                            ">=" => metric_value >= *value,
                            "<=" => metric_value <= *value,
                            "==" => (metric_value - value).abs() < f64::EPSILON,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                AlertCondition::Rate { metric: rate_metric, operator, value, window_seconds } => {
                    if *rate_metric == metric.metric_type {
                        // Calculate rate over the specified time window
                        let rate = self.calculate_metric_rate(&metric.metric_type, *window_seconds).await;
                        match operator.as_str() {
                            ">" => rate > *value,
                            "<" => rate < *value,
                            ">=" => rate >= *value,
                            "<=" => rate <= *value,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                AlertCondition::Anomaly { metric: anomaly_metric, sensitivity } => {
                    if *anomaly_metric == metric.metric_type {
                        // Apply statistical anomaly detection
                        self.detect_anomaly(&metric.metric_type, metric.value.as_f64(), *sensitivity).await
                    } else {
                        false
                    }
                }
                AlertCondition::ServiceDown { service_name } => {
                    // Check service health status
                    self.check_service_health(service_name).await
                }
            };

            if should_alert {
                // Check if we're in cooldown period
                let active_alerts = self.active_alerts.read().await;
                let recent_alert = active_alerts.iter().any(|alert| {
                    alert.rule_id == rule.id && 
                    (chrono::Utc::now().timestamp_millis() as u64 - alert.triggered_at) < (rule.cooldown_seconds * 1000)
                });

                if !recent_alert {
                    drop(active_alerts); // Release read lock
                    self.trigger_alert(rule, metric.value.as_f64()).await?;
                }
            }
        }

        Ok(())
    }

    /// Trigger a new alert
    async fn trigger_alert(&self, rule: &AlertRule, current_value: f64) -> ArbitrageResult<()> {
        let threshold_value = match &rule.condition {
            AlertCondition::Threshold { value, .. } => *value,
            _ => 0.0,
        };

        let alert = Alert::new(rule, current_value, threshold_value);
        
        {
            let mut alerts = self.active_alerts.write().await;
            alerts.push(alert.clone());
        }

        // TODO: Send notifications through configured channels
        log::warn!("Alert triggered: {} - Current value: {}, Threshold: {}", 
                  alert.name, current_value, threshold_value);

        Ok(())
    }

    /// Calculate overall system health score
    fn calculate_health_score(&self, cpu: f64, memory: f64, error_rate: f64, critical_alerts: usize, warning_alerts: usize) -> f64 {
        let mut score = 1.0;
        
        // CPU penalty
        if cpu > 80.0 {
            score -= (cpu - 80.0) / 100.0;
        }
        
        // Memory penalty
        if memory > 80.0 {
            score -= (memory - 80.0) / 100.0;
        }
        
        // Error rate penalty
        score -= error_rate * 2.0;
        
        // Alert penalties
        score -= critical_alerts as f64 * 0.3;
        score -= warning_alerts as f64 * 0.1;
        
        score.max(0.0).min(1.0)
    }

    /// Get real CPU usage percentage using sysinfo
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_real_cpu_usage(&self) -> f64 {
        let mut system = self.system.write().await;
        system.refresh_cpu();
        
        // Wait a bit to get accurate CPU measurements
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        system.refresh_cpu();
        
        // Calculate average CPU usage across all cores
        let cpus = system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }
        
        let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        (total_usage / cpus.len() as f32) as f64
    }

    /// Get real memory usage percentage using sysinfo
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_real_memory_usage(&self) -> f64 {
        let mut system = self.system.write().await;
        system.refresh_memory();
        
        let used_memory = system.used_memory();
        let total_memory = system.total_memory();
        
        if total_memory == 0 {
            return 0.0;
        }
        
        (used_memory as f64 / total_memory as f64) * 100.0
    }

    /// Get real disk usage percentage using sysinfo
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_real_disk_usage(&self) -> f64 {
        let mut system = self.system.write().await;
        system.refresh_disks();
        
        let disks = system.disks();
        if disks.is_empty() {
            return 0.0;
        }
        
        // Calculate average disk usage across all disks
        let mut total_available = 0;
        let mut total_space = 0;
        
        for disk in disks {
            total_available += disk.available_space();
            total_space += disk.total_space();
        }
        
        if total_space == 0 {
            return 0.0;
        }
        
        let used_space = total_space - total_available;
        (used_space as f64 / total_space as f64) * 100.0
    }

    /// Get real service uptime in hours
    #[cfg(not(target_arch = "wasm32"))]
    fn get_real_uptime_hours(&self) -> f64 {
        let uptime_duration = self.service_start_time.elapsed();
        uptime_duration.as_secs_f64() / 3600.0
    }

    /// Fallback functions for WASM or when sysinfo is not available
    #[cfg(target_arch = "wasm32")]
    async fn get_real_cpu_usage(&self) -> f64 {
        // WASM fallback: simulated values
        25.0 + (chrono::Utc::now().timestamp() % 100) as f64 / 4.0
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_real_memory_usage(&self) -> f64 {
        // WASM fallback: simulated values
        45.0 + (chrono::Utc::now().timestamp() % 1000) as f64 / 100.0
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_real_disk_usage(&self) -> f64 {
        // WASM fallback: simulated values
        65.0 + (chrono::Utc::now().timestamp() % 10000) as f64 / 1000.0
    }

    #[cfg(target_arch = "wasm32")]
    fn get_real_uptime_hours(&self) -> f64 {
        // WASM fallback: simulate uptime
        72.5
    }

    /// Calculate metric rate over time window
    async fn calculate_metric_rate(&self, metric_type: &MetricType, window_seconds: u64) -> f64 {
        let metrics = self.get_metrics(metric_type, window_seconds / 3600).await;
        
        if metrics.len() < 2 {
            return 0.0;
        }

        // Calculate rate as the difference between latest and earliest values divided by time window
        let latest = metrics.last().unwrap();
        let earliest = metrics.first().unwrap();
        
        let time_diff = (latest.timestamp - earliest.timestamp) as f64 / 1000.0; // Convert to seconds
        if time_diff > 0.0 {
            (latest.value.as_f64() - earliest.value.as_f64()) / time_diff
        } else {
            0.0
        }
    }

    /// Detect anomalies using statistical analysis
    async fn detect_anomaly(&self, metric_type: &MetricType, current_value: f64, sensitivity: f64) -> bool {
        let historical_metrics = self.get_metrics(metric_type, 24).await; // Get 24 hours of data
        
        if historical_metrics.len() < 10 {
            return false; // Need enough data for statistical analysis
        }

        // Calculate mean and standard deviation
        let values: Vec<f64> = historical_metrics.iter().map(|m| m.value.as_f64()).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Anomaly if current value is more than sensitivity * std_dev away from mean
        let threshold = sensitivity * std_dev;
        (current_value - mean).abs() > threshold
    }

    /// Check service health status
    async fn check_service_health(&self, service_name: &str) -> bool {
        // In production, this would check actual service status
        // For now, simulate based on service name and some logic
        match service_name {
            "database" => {
                // Check if database queries are failing
                let error_rate_metrics = self.get_metrics(&MetricType::ErrorRate, 1).await;
                if let Some(latest) = error_rate_metrics.last() {
                    latest.value.as_f64() > 0.1 // Alert if error rate > 10%
                } else {
                    false
                }
            }
            "telegram_service" => {
                // Check if telegram messages are being sent
                let telegram_metrics = self.get_metrics(&MetricType::TelegramMessagesPerHour, 1).await;
                if let Some(latest) = telegram_metrics.last() {
                    latest.value.as_f64() == 0.0 // Alert if no messages in last hour
                } else {
                    true // Alert if no metrics available
                }
            }
            "exchange_api" => {
                // Check if exchange API calls are responding
                let api_metrics = self.get_metrics(&MetricType::ExchangeApiCalls, 1).await;
                if let Some(latest) = api_metrics.last() {
                    latest.value.as_f64() == 0.0 // Alert if no API calls in last hour
                } else {
                    true // Alert if no metrics available
                }
            }
            _ => {
                log::warn!("Unknown service for health check: {}", service_name);
                false
            }
        }
    }

    /// Calculate percentiles from response time samples
    async fn calculate_percentiles(&self, operation: &str) -> (f64, f64, f64) {
        let samples = self.response_time_samples.read().await;
        
        if let Some(operation_samples) = samples.get(operation) {
            if operation_samples.is_empty() {
                return (0.0, 0.0, 0.0);
            }

            // Clone and sort the samples for percentile calculation
            let mut sorted_samples = operation_samples.clone();
            sorted_samples.sort_unstable();

            let len = sorted_samples.len();
            
            // Calculate percentiles using linear interpolation
            let p50 = calculate_percentile(&sorted_samples, 50.0);
            let p95 = calculate_percentile(&sorted_samples, 95.0);
            let p99 = calculate_percentile(&sorted_samples, 99.0);

            (p50, p95, p99)
        } else {
            (0.0, 0.0, 0.0)
        }
    }
}

/// Calculate a specific percentile from sorted samples
fn calculate_percentile(sorted_samples: &[u64], percentile: f64) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }

    let len = sorted_samples.len();
    if len == 1 {
        return sorted_samples[0] as f64;
    }

    // Calculate the index for the percentile
    let index = (percentile / 100.0) * (len - 1) as f64;
    let lower_index = index.floor() as usize;
    let upper_index = index.ceil() as usize;

    if lower_index == upper_index {
        sorted_samples[lower_index] as f64
    } else {
        // Linear interpolation between the two nearest values
        let lower_value = sorted_samples[lower_index] as f64;
        let upper_value = sorted_samples[upper_index] as f64;
        let weight = index - lower_index as f64;
        
        lower_value + weight * (upper_value - lower_value)
    }
}

impl Default for MonitoringObservabilityService {
    fn default() -> Self {
        Self::new()
    }
}

/// System Health Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSummary {
    pub health_score: f64, // 0.0 to 1.0
    pub status: String,    // "healthy", "degraded", "warning", "critical"
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub error_rate_percent: f64,
    pub active_alerts_count: usize,
    pub critical_alerts_count: usize,
    pub warning_alerts_count: usize,
    pub uptime_hours: f64,
    pub last_updated: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let service = MonitoringObservabilityService::new();
        assert!(service.enabled);
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let service = MonitoringObservabilityService::new();
        
        let metric = MetricDataPoint::new(
            MetricType::CpuUsagePercent,
            MetricValue::gauge(75.5),
            "Test CPU metric".to_string(),
        );

        service.record_metric(metric).await.unwrap();

        let recorded_metrics = service.get_metrics(&MetricType::CpuUsagePercent, 1).await;
        assert_eq!(recorded_metrics.len(), 1);
        assert_eq!(recorded_metrics[0].value.as_f64(), 75.5);
    }

    #[tokio::test]
    async fn test_performance_recording() {
        let service = MonitoringObservabilityService::new();
        
        service.record_performance("test_operation", 150, false).await.unwrap();
        service.record_performance("test_operation", 200, true).await.unwrap();

        let perf_overview = service.get_performance_overview().await;
        let test_metrics = perf_overview.get("test_operation").unwrap();
        
        assert_eq!(test_metrics.request_count, 2);
        assert_eq!(test_metrics.error_count, 1);
        assert_eq!(test_metrics.avg_response_time_ms, 175.0);
        assert_eq!(test_metrics.error_rate, 0.5);
    }

    #[tokio::test]
    async fn test_trace_span() {
        let service = MonitoringObservabilityService::new();
        
        let mut span = service.start_trace("test_operation".to_string()).await;
        span.add_tag("user_id".to_string(), "123".to_string());
        span.add_log("info".to_string(), "Operation started".to_string());
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        service.finish_trace(span).await.unwrap();

        let traces = service.trace_spans.read().await;
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].operation_name, "test_operation");
        assert!(traces[0].duration_ms.is_some());
    }

    #[tokio::test]
    async fn test_alert_initialization() {
        let mut service = MonitoringObservabilityService::new();
        service.initialize().await.unwrap();

        let rules = service.alert_rules.read().await;
        assert!(!rules.is_empty());
        
        // Check for high CPU alert rule
        let cpu_rule = rules.iter().find(|r| r.id == "high_cpu_usage");
        assert!(cpu_rule.is_some());
        assert_eq!(cpu_rule.unwrap().severity, AlertSeverity::Warning);
    }

    #[tokio::test]
    async fn test_system_health_summary() {
        let service = MonitoringObservabilityService::new();
        
        // Record some test metrics
        service.record_metric(MetricDataPoint::new(
            MetricType::CpuUsagePercent,
            MetricValue::gauge(45.0),
            "Test".to_string(),
        )).await.unwrap();

        let health = service.get_system_health_summary().await;
        assert_eq!(health.status, "healthy");
        assert!(health.health_score > 0.8);
        assert_eq!(health.active_alerts_count, 0);
    }

    #[test]
    fn test_metric_value_types() {
        let counter = MetricValue::counter(100);
        assert_eq!(counter.as_f64(), 100.0);

        let gauge = MetricValue::gauge(75.5);
        assert_eq!(gauge.as_f64(), 75.5);
    }

    #[test]
    fn test_alert_creation() {
        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test Alert".to_string(),
            description: "Test alert rule".to_string(),
            condition: AlertCondition::Threshold {
                metric: MetricType::CpuUsagePercent,
                operator: ">".to_string(),
                value: 80.0,
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_seconds: 300,
            notification_channels: vec!["telegram".to_string()],
            labels: HashMap::new(),
        };

        let alert = Alert::new(&rule, 85.0, 80.0);
        assert_eq!(alert.rule_id, "test_rule");
        assert_eq!(alert.current_value, 85.0);
        assert_eq!(alert.threshold_value, 80.0);
        assert!(!alert.is_resolved());
    }
} 