# Implementation Plan: Refactor to Rust

## Background and Motivation

We want to rewrite the entire ArbEdge codebase in Rust to leverage Rust's performance, memory safety, and strong type guarantees. The application will continue to run on Cloudflare Workers using the Workers database (KV or Durable Objects) and maintain integrations with exchange APIs and the Telegram bot, preserving existing cloud infrastructure and workflows.

## Branch Name

`feature/refactor-to-rust`

## Key Challenges and Analysis

- **Cloudflare Workers Rust Support:** Compiling Rust to WASM with `wrangler` and ensuring compatibility.
- **Workers Database Integration:** Choosing between KV and Durable Objects and using Rust crates (`workers-rs` or `cloudflare`).
- **Exchange API Clients:** Selecting or building Rust bindings for exchange APIs (e.g., Binance, CCXT-like functionality).
- **Telegram Bot Integration:** Using a Rust crate (e.g., `teloxide`) to interface with the Telegram Bot API.
- **Async and Event-Driven Logic:** Mapping TypeScript async patterns and event handlers (webhook, scheduled) to Rust `async`/`await` and Futures.
- **Testing Strategy:** Unit tests (`cargo test`), integration tests with HTTP mocks, and ensuring coverage.
- **CI/CD Workflow:** Integrating Rust build and tests into GitHub Actions and `wrangler` deployments.

## High-level Task Breakdown

1. **Create feature branch `feature/refactor-to-rust`.**
   - Success Criteria: Branch created off `main`.
2. **Initialize Rust workspace and Worker configuration.**
   - Set up `Cargo.toml`, `wrangler.toml` for a Rust-based Worker.
   - Success Criteria: Worker compiles to WASM and `wrangler dev` starts.
3. **Integrate Workers database in Rust.**
   - Configure KV or Durable Objects using a Rust crate.
   - Success Criteria: Basic KV operations (get/put) succeed in a test Worker.
4. **Implement utility modules in Rust.**
   - Migrate `formatter`, `logger`, and `CustomError` logic.
   - Success Criteria: Unit tests pass for utilities.
5. **Develop exchange API client in Rust.**
   - Implement core exchange methods (e.g., fetch ticker, open positions).
   - Success Criteria: Exchange API client unit tests pass with mocks.
6. **Develop Telegram bot client in Rust.**
   - Integrate with the Telegram Bot API.
   - Success Criteria: Ability to send messages in a mocked environment.
7. **Migrate core business logic.**
   - Rewrite `opportunityService` and `positionsManager` in Rust.
   - Success Criteria: Business logic unit tests cover core functionality.
8. **Rewrite Worker entrypoints.**
   - Implement webhook and scheduled handlers in Rust.
   - Success Criteria: Handlers respond correctly in integration tests.
9. **Write end-to-end tests.**
   - Simulate full workflows (webhook, scheduled) with HTTP mocks.
   - Success Criteria: All end-to-end scenarios pass.
10. **Setup CI pipeline.**
    - Configure GitHub Actions to build, test, and lint the Rust project and deploy via `wrangler`.
    - Success Criteria: CI passes on pull requests.
11. **Deploy to Cloudflare Workers.**
    - Use `wrangler publish` for Rust Workers.
    - Success Criteria: Deployment succeeds and endpoints are live.

## Project Status Board

- [x] Task 1: Create feature branch `feature/refactor-to-rust`
- [x] Task 2: Initialize Rust workspace and Worker configuration
- [x] Task 3: Integrate Workers database in Rust
- [x] Task 4: Implement utility modules in Rust
- [x] Task 5: Develop exchange API client in Rust
- [ ] Task 6: Develop Telegram bot client in Rust
- [ ] Task 7: Migrate core business logic to Rust
- [ ] Task 8: Rewrite webhook and scheduled handlers to Rust
- [ ] Task 9: Write end-to-end tests
- [ ] Task 10: Remove all TypeScript code
- [ ] Task 11: Setup CI pipeline
- [ ] Task 12: Deploy to Cloudflare Workers

## Executor's Feedback or Assistance Requests

- ✅ RESOLVED: Build for wasm32-unknown-unknown was failing due to Homebrew Rust taking precedence. Fixed by setting `export PATH="$HOME/.cargo/bin:$PATH"` to prioritize rustup-managed toolchain.
- ✅ COMPLETED: Task 2 - Successfully set up Rust workspace with worker-build, updated to latest worker crate (0.5.0), and verified deployment dry-run succeeds. Basic KV operations are working.
- ✅ COMPLETED: Task 3 - Workers database integration complete with KV operations working in ExchangeService for credential storage.
- ✅ COMPLETED: Task 4 - All utility modules migrated: error handling (ArbitrageError), formatter, logger, helpers, and calculations.
- ✅ COMPLETED: Task 5 - Exchange API client fully implemented with CCXT-like interface. Features include:
  - HTTP client with authentication (HMAC-SHA256 for Binance/Bybit)
  - Market data fetching (markets, tickers, orderbooks, funding rates)
  - Credential management with KV storage
  - Comprehensive type system for orders, positions, and trading data
  - API endpoints for testing: /exchange/markets, /exchange/ticker, /exchange/funding
  - Successfully replaced CCXT dependency with native Rust implementation

## Lessons Learned

- [2025-01-22] CCXT replacement strategy: Since there's no direct CCXT equivalent for Rust, we built our own exchange client using reqwest HTTP client with exchange-specific authentication and API parsing. This provides better control and performance.
- [2025-01-22] Type system design: Used both enum (ExchangeIdEnum) and string alias (ExchangeId) types to maintain compatibility while providing type safety.
- [2025-01-22] Error handling: Implemented comprehensive error system with ArbitrageError that includes context, status codes, and conversion traits for seamless error propagation. 