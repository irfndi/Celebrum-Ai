//! Shared utilities and types for ArbEdge Arbitrage Platform
//!
//! This crate provides common types, utilities, and infrastructure
//! that are shared across all packages in the ArbEdge platform.

pub mod features;
pub mod infrastructure;
pub mod types;
pub mod user;
pub mod utils;

// Re-export commonly used types
pub use types::*;
