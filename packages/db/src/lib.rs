//! Database layer for ArbEdge Arbitrage Platform
//!
//! This crate provides database abstractions, models, and migrations
//! for the ArbEdge platform.

pub mod connection;
pub mod migrations;
pub mod models;
pub mod repositories;

// Re-export commonly used types
pub use connection::*;
pub use models::*;
pub use repositories::*;
