use crate::types::FairnessConfig;

/// Configuration for opportunity distribution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistributionConfig {
    pub max_opportunities_per_user_per_hour: u32,
    pub max_opportunities_per_user_per_day: u32,
    pub cooldown_period_minutes: u32,
    pub batch_size: u32,
    pub distribution_interval_seconds: u32,
    pub max_participants_per_opportunity: Option<u32>,
    pub fairness_config: FairnessConfig,
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            max_opportunities_per_user_per_hour: 2,
            max_opportunities_per_user_per_day: 10,
            cooldown_period_minutes: 240, // 4 hours
            batch_size: 50,
            distribution_interval_seconds: 30,
            max_participants_per_opportunity: Some(100), // Default to 100 participants
            fairness_config: FairnessConfig::default(),
        }
    }
}
