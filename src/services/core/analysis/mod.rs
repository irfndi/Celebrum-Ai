// src/services/core/analysis/mod.rs

//! Analysis Services Module
//!
//! This module contains analysis services for market data processing,
//! technical analysis, and correlation analysis to support trading decisions.
//!
//! ## Services
//! - `MarketAnalysisService`: Market data analysis and opportunity detection
//! - `TechnicalAnalysisService`: Technical indicator analysis and signals
//! - `CorrelationAnalysisService`: Cross-market correlation analysis

pub mod correlation_analysis;
pub mod market_analysis;
pub mod technical_analysis;

pub use correlation_analysis::CorrelationAnalysisService;
pub use market_analysis::MarketAnalysisService;
pub use technical_analysis::TechnicalAnalysisService;
