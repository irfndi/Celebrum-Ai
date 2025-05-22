# Scratchpad

## Current Task

- **Task Name:** Refactor to Rust
- **Implementation Plan:** [./implementation-plan/refactor-to-rust.md](./implementation-plan/refactor-to-rust.md)
- **Status:** In Progress

## Lessons Learned
- [YYYY-MM-DD] Example lesson.
- [2025-05-22] Completed tests for `formatter.ts`, `logger.ts`, and `CustomError.ts`, achieving 100% coverage for these modules.
- [2025-05-22] Identified TypeScript type mismatch errors in `tests/__mocks__/ccxt.ts` related to missing `ExchangeId` literals, blocking `exchangeService.ts` tests and requiring updates to type definitions.
- [2025-05-22] Encountered Rust build error E0463 (`can't find crate for core`) when compiling for `wasm32-unknown-unknown`; resolved by installing the target for the stable toolchain with `rustup target add wasm32-unknown-unknown --toolchain stable`. 