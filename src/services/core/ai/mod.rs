// src/services/core/ai/mod.rs

//! AI Services Module
//! 
//! This module contains AI-powered services for enhancing trading opportunities,
//! providing intelligent analysis, and supporting decision-making processes.
//! 
//! ## Services
//! - `AiIntegrationService`: Core AI integration and coordination
//! - `AiIntelligenceService`: Advanced AI analysis and insights

pub mod ai_beta_integration;
pub mod ai_intelligence;
pub mod ai_integration;

pub use ai_beta_integration::AiBetaIntegrationService;
pub use ai_intelligence::AiIntelligenceService;
pub use ai_integration::AiIntegrationService; 