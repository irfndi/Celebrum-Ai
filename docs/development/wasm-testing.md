# WASM Target Testing

## Overview

The ArbEdge project now includes comprehensive WASM (WebAssembly) target testing in the CI pipeline to catch compatibility and lifetime issues early in the development cycle.

## Why WASM Testing Matters

Cloudflare Workers run on a WASM runtime, which has stricter lifetime and borrowing rules compared to native Rust compilation. Issues that compile fine for native targets may fail on WASM targets due to:

- **Stricter lifetime requirements**: WASM compiler is more conservative about temporary value lifetimes
- **Different async runtime**: Uses `gloo-timers` instead of `tokio` for timeout handling
- **Memory management**: Different garbage collection and memory allocation patterns
- **API differences**: Some standard library features behave differently in WASM

## CI Pipeline Integration

The enhanced CI pipeline now includes WASM checks at multiple stages:

### Step 3: WASM Target Compilation Check
```bash
cargo check --target wasm32-unknown-unknown --lib
```
- **Purpose**: Early detection of WASM-specific compilation issues
- **Timing**: After linting, before running tests
- **Benefits**: Fails fast if WASM compatibility is broken

### Step 8: Final WASM Build Verification
```bash
cargo build --target wasm32-unknown-unknown --lib --quiet
```
- **Purpose**: Full WASM build verification after all tests pass
- **Timing**: Final step before CI completion
- **Benefits**: Ensures deployment-ready WASM binary can be built

## Available Commands

### Quick WASM Check
```bash
make check-wasm
```
Fast compilation check for WASM target during development.

### Development Cycle with WASM
```bash
make dev
```
Now includes WASM checking in the standard development cycle (format, lint, test, WASM check).

### Comprehensive Checks
```bash
make check-all
```
Runs all basic checks including native and WASM builds.

### Full CI Pipeline
```bash
make ci
```
Complete CI pipeline with WASM verification at multiple stages.

## Common WASM Issues and Solutions

### 1. Lifetime Issues with Fetch Requests

**Problem**: Temporary values dropped while borrowed
```rust
// ❌ This fails on WASM
let request_future = Fetch::Request(request).send();
```

**Solution**: Use async move closure
```rust
// ✅ This works on WASM
let fetch_request = Fetch::Request(request);
let request_future = async move { fetch_request.send().await };
```

### 2. Timeout Handling Differences

**WASM**: Uses `gloo-timers::future::TimeoutFuture`
```rust
#[cfg(target_arch = "wasm32")]
{
    use gloo_timers::future::TimeoutFuture;
    let timeout_future = TimeoutFuture::new(timeout_ms as u32);
}
```

**Native**: Uses `tokio::time::timeout`
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    let timeout_duration = std::time::Duration::from_secs(timeout_seconds);
    tokio::time::timeout(timeout_duration, future).await
}
```

### 3. Dependency Management

WASM-specific dependencies in `Cargo.toml`:
```toml
[dependencies]
gloo-timers = { version = "0.3", features = ["futures"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
gloo-timers = "0.3"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { version = "1.42", features = ["time"] }
```

## Best Practices

1. **Test Early**: Run `make check-wasm` during development
2. **Use Conditional Compilation**: Separate WASM and native code paths with `#[cfg]`
3. **Avoid Temporary Borrows**: Be explicit about variable lifetimes in async contexts
4. **Platform-Specific Dependencies**: Use target-specific dependencies when needed
5. **CI Integration**: Always include WASM checks in CI pipeline

## Troubleshooting

### Build Fails on WASM but Not Native

1. Check for lifetime issues with temporary values
2. Verify async/await usage in WASM context
3. Ensure all dependencies support WASM target
4. Review conditional compilation directives

### Performance Differences

WASM builds may have different performance characteristics:
- Smaller binary size with optimized dependencies
- Different memory allocation patterns
- Async runtime differences

## Future Enhancements

- **WASM-specific tests**: Add tests that only run on WASM target
- **Performance benchmarks**: Compare WASM vs native performance
- **Size optimization**: Monitor and optimize WASM binary size
- **Runtime testing**: Test actual WASM execution in Cloudflare Workers environment

## Related Documentation

- [Cloudflare Workers Rust Documentation](https://developers.cloudflare.com/workers/languages/rust/)
- [wasm-bindgen Book](https://rustwasm.github.io/wasm-bindgen/)
- [Rust WASM Working Group](https://rustwasm.github.io/) 