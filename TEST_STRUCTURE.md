# Test Structure Documentation

This document describes the reorganized test structure for the ArbEdge monorepo.

## Overview

Tests have been moved from the centralized `/tests` directory to individual package directories for better maintainability and organization in the monorepo architecture.

## Test Organization

### Package-Specific Tests

#### Telegram Bot (`packages/telegram-bot/tests/`)
- `telegram/` - Telegram bot command tests and functionality
- Contains tests for bot commands, message handling, and Telegram-specific features

#### Discord Bot (`packages/discord-bot/tests/`)
- `discord/` - Discord bot tests and functionality
- Contains tests for Discord-specific features and integrations

#### Web (`packages/web/tests/`)
- `api/` - Web API interface tests
- Contains tests for REST endpoints, web interface, and API functionality

#### Worker (`packages/worker/tests/`)
- `analysis/` - Market analysis and trading algorithm tests
- `trading/` - Trading engine and execution tests
- `opportunities/` - Opportunity detection and distribution tests
- `integration/` - Cross-service integration tests
- `e2e/` - End-to-end workflow tests
- `ai/` - AI and machine learning component tests
- `disabled/` - Temporarily disabled tests

#### Shared (`packages/shared/tests/`)
- `infrastructure/` - Database, queue, and infrastructure tests
- `user/` - User management and profile tests
- `features/` - Feature flag and configuration tests
- `common/` - Common test utilities and helpers
- `test_utils/` - Reusable test utilities

## Running Tests

### Run all tests in the workspace:
```bash
cargo test
```

### Run tests for a specific package:
```bash
# Telegram bot tests
cargo test -p telegram-bot-tests

# Worker tests
cargo test -p worker-tests

# Shared tests
cargo test -p shared-tests

# Discord bot tests
cargo test -p discord-bot-tests

# Web tests
cargo test -p web-tests
```

### Run specific test categories:
```bash
# Integration tests only
cargo test -p worker-tests integration

# E2E tests only
cargo test -p worker-tests e2e

# Infrastructure tests only
cargo test -p shared-tests infrastructure
```

## Benefits of This Structure

1. **Better Organization**: Tests are co-located with the code they test
2. **Faster CI/CD**: Can run tests for only changed packages
3. **Clearer Dependencies**: Each test package has explicit dependencies
4. **Easier Maintenance**: Developers working on a package can focus on its tests
5. **Scalability**: Easy to add new packages with their own test suites

## Migration Notes

- All import paths in tests have been updated to use the new package structure
- Shared test utilities are available through the `shared-tests` crate
- Common test patterns and mocks are centralized in `packages/shared/tests/`
- Integration and E2E tests remain in the worker package as they test cross-service functionality