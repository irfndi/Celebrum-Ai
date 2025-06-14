// src/services/core/ai/mod.rs

//! AI Services Module
//!
//! This module contains AI-powered services for enhancing trading opportunities,
//! providing intelligent analysis, and supporting decision-making processes.
//!
//! ## Services
//! - `AiIntegrationService`: Core AI integration and coordination
//! - `AiIntelligenceService`: Advanced AI analysis and insights
//! - `AiAnalysisService`: Simplified AI analysis for API endpoints

pub mod ai_analysis_service;
pub mod ai_beta_integration;
pub mod ai_integration;
pub mod ai_intelligence;

pub use ai_analysis_service::AiAnalysisService;
pub use ai_beta_integration::AiBetaIntegrationService;
pub use ai_integration::AiIntegrationService;
pub use ai_intelligence::AiIntelligenceService;
