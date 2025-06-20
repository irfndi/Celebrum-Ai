//! Shared test utilities and common test infrastructure
//! This crate provides reusable test components for all packages in the monorepo

// Common test utilities and helpers
pub mod common;
pub mod test_utils;

// Core infrastructure tests (shared across services)
pub mod infrastructure;

// User and feature tests (shared across services)
pub mod features;
pub mod user;

// Re-export commonly used test utilities
pub use common::*;
pub use test_utils::*;
