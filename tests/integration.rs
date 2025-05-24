// Main integration test file that imports tests from Integrations/ subdirectory
// This allows cargo to find and run the integration tests

mod integrations;

// Re-export key integration tests for easy access
pub use integrations::integration_test_basic;
// TODO: Re-enable critical_service_integration_tests once service mocking infrastructure is complete
// Issue: Service dependency injection needed for proper testing
// Target: After PR #24 Comment 40 resolution
// pub use integrations::critical_service_integration_tests;

// TODO: Re-enable service_integration_tests once MockD1Service trait implementation is stable
// Issue: Requires consistent trait-based mocking for all services
// Target: After integration test framework refactoring
// pub use integrations::service_integration_tests;
