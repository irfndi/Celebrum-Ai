//! Core Worker for ArbEdge Arbitrage Platform
//!
//! This crate contains the main business logic for the ArbEdge platform,
//! including trading algorithms, opportunity detection, and market analysis.

pub mod analysis;
pub mod infrastructure;
pub mod opportunities;
pub mod trading;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use types::*;
