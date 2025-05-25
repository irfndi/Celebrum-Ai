// src/services/core/analysis/mod.rs

pub mod market_analysis;
pub mod technical_analysis;
pub mod correlation_analysis;

pub use market_analysis::MarketAnalysisService;
pub use technical_analysis::TechnicalAnalysisService;
pub use correlation_analysis::CorrelationAnalysisService; 