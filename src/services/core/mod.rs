// src/services/core/mod.rs

pub mod user;
pub mod trading;
pub mod opportunities;
pub mod analysis;
pub mod ai;
pub mod infrastructure;
pub mod invitation;
pub mod market_data;

// Re-export all services for convenience
pub use user::*;
pub use trading::*;
pub use opportunities::*;
pub use analysis::*;
pub use ai::*;
pub use infrastructure::*;
pub use invitation::*;
pub use market_data::*; 