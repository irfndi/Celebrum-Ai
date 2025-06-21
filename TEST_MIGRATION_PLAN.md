# Test Migration Plan: Moving Tests to Monorepo Packages

## Overview
This document outlines the plan to move embedded test modules from the main `src/` directory to appropriate packages within the monorepo structure.

## Current State Analysis

### TypeScript Tests
- ✅ Already properly organized in packages
- ✅ `packages/worker/src/*.test.ts` - Worker-specific tests
- ✅ `packages/shared/src/*.test.ts` - Shared functionality tests
- ✅ Using Vitest framework with proper configuration

### Rust Tests
- ✅ Package-specific test directories exist: `packages/*/tests/`
- ✅ Each has proper `Cargo.toml` configuration
- ❌ Main `src/` directory contains 20+ embedded test modules that need relocation

## Test Modules to Relocate

### 1. Core Infrastructure Tests → `packages/worker/tests/infrastructure/`
- `src/services/core/infrastructure/unified_analytics_and_cleanup.rs` (lines 837-890)
- `src/services/core/infrastructure/unified_core_services.rs` (lines 577-690)
- `src/services/core/infrastructure/unified_cloudflare_services.rs` (lines 806-860)
- `src/services/core/infrastructure/infrastructure_engine.rs` (lines 613-680)
- `src/services/core/infrastructure/mod.rs` (lines 675-720)
- `src/services/core/infrastructure/shared_types.rs` (lines 445-520)
- `src/services/core/infrastructure/unified_notification_services.rs` (lines 1269-1330)
- `src/services/core/infrastructure/unified_financial_services.rs` (lines 1221-1270)
- `src/services/core/infrastructure/unified_ai_services.rs` (lines 1059-1100)

### 2. Persistence Layer Tests → `packages/worker/tests/persistence/`
- `src/services/core/infrastructure/persistence_layer/database_manager.rs` (lines 2020-2080)
- `src/services/core/infrastructure/persistence_layer/mod.rs` (lines 563-600)
- `src/services/core/infrastructure/persistence_layer/unified_repository_layer.rs`
- `src/services/core/infrastructure/persistence_layer/storage_layer.rs`

### 3. Data Ingestion Tests → `packages/worker/tests/ingestion/`
- `src/services/core/infrastructure/data_ingestion_module/mod.rs` (lines 672-760)
- `src/services/core/infrastructure/data_ingestion_module/simple_ingestion.rs`

### 4. User & AI Tests → `packages/shared/tests/user/`
- `src/services/core/user/ai_access.rs` (lines 940-1030)
- `src/services/core/user/group_management.rs` (lines 498-580)
- `src/services/core/ai/ai_integration.rs` (lines 1012-1220)

### 5. Trading Tests → `packages/worker/tests/trading/`
- `src/services/core/trading/ai_exchange_router.rs` (lines 754-1410)
- `src/services/core/trading/exchange_availability.rs` (lines 430-530)
- `src/services/core/trading/exchange.rs`

### 6. Utility Tests → `packages/shared/tests/utils/`
- `src/utils/formatter.rs` (lines 524-596)
- `src/utils/helpers.rs` (lines 199-390)

## Implementation Steps

### Phase 1: Extract and Create Test Files
1. Extract test modules from source files
2. Create corresponding test files in target packages
3. Update imports and dependencies
4. Ensure proper module structure

### Phase 2: Verify and Clean
1. Run tests in new locations to ensure they pass
2. Update `Cargo.toml` files if needed
3. Remove test modules from original source files
4. Update any cross-references

### Phase 3: Integration
1. Update CI pipeline to run tests from new locations
2. Verify `make ci` still passes
3. Update documentation

## Benefits
- ✅ Better separation of concerns
- ✅ Package-specific test isolation
- ✅ Easier maintenance and development
- ✅ Cleaner source code without embedded tests
- ✅ Improved monorepo organization

## Status
- [x] Analysis completed
- [ ] Phase 1: Extract and create test files
- [ ] Phase 2: Verify and clean
- [ ] Phase 3: Integration