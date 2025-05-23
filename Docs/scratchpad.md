# Scratchpad

## Current Task

- **Task Name:** Refactor to Rust
- **Implementation Plan:** [./implementation-plan/refactor-to-rust.md](./implementation-plan/refactor-to-rust.md)
- **Status:** In Progress - Major milestone reached

## Lessons Learned
- [YYYY-MM-DD] Example lesson.
- [2025-05-22] Completed tests for `formatter.ts`, `logger.ts`, and `CustomError.ts`, achieving 100% coverage for these modules.
- [2025-05-22] Identified TypeScript type mismatch errors in `tests/__mocks__/ccxt.ts` related to missing `ExchangeId` literals, blocking `exchangeService.ts` tests and requiring updates to type definitions.
- [2025-05-22] Encountered Rust build error E0463 (`can't find crate for core`) when compiling for `wasm32-unknown-unknown`; resolved by installing the target for the stable toolchain with `rustup target add wasm32-unknown-unknown --toolchain stable`.
- [2025-01-22] CCXT replacement strategy: Since there's no direct CCXT equivalent for Rust, we built our own exchange client using reqwest HTTP client with exchange-specific authentication and API parsing. This provides better control and performance.
- [2025-01-22] Type system design: Used both enum (ExchangeIdEnum) and string alias (ExchangeId) types to maintain compatibility while providing type safety.
- [2025-01-22] Error handling: Implemented comprehensive error system with ArbitrageError that includes context, status codes, and conversion traits for seamless error propagation.
- [2025-01-22] Successfully implemented core exchange service with CCXT-like interface in Rust, including HTTP authentication, market data fetching, and KV storage integration. 