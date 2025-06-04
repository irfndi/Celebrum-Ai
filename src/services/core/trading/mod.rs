// src/services/core/trading/mod.rs

pub mod ai_exchange_router;
pub mod exchange;
pub mod kv_operations;
pub mod positions;

pub use ai_exchange_router::AiExchangeRouterService;
pub use exchange::ExchangeService;
pub use positions::PositionsService;

// Re-export items from kv_operations to make them directly accessible under the trading module
pub use kv_operations::{KvOperationError, KvOperations, KvResult};
