// ArbEdge Test Suite
// Comprehensive testing framework for all components

// Common test utilities and helpers
pub mod common;

// Unit tests for individual services and components
pub mod unit;

// Integration tests for service communication and coordination
pub mod integration;

// End-to-end tests for complete user workflows
pub mod e2e;

// Integration tests are now individual files in tests/ directory
// - integration_test_basic.rs
// - telegram_advanced_commands_test.rs
// - telegram_bot_commands_test.rs
// - market_data_pipeline_test.rs
// - comprehensive_service_integration_test.rs
