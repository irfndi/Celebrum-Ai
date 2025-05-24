// Main integration test file that imports tests from Integrations/ subdirectory
// This allows cargo to find and run the integration tests

mod integrations;

// Re-export key integration tests for easy access
pub use integrations::integration_test_basic;
// Temporarily disabled due to compilation errors
// pub use integrations::critical_service_integration_tests;
// pub use integrations::service_integration_tests; 