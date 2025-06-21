use crate::types::TradingSettings;
use crate::utils::{ArbitrageError, ArbitrageResult};

/// Default configuration values used when user-specific trading settings are missing.
const DEFAULT_TRADE_SIZE_USDT: f64 = 100.0;
const DEFAULT_STOP_LOSS_PERCENT: f64 = 1.0; // 1%
const DEFAULT_TAKE_PROFIT_PERCENT: f64 = 2.0; // 2%

/// Calculated trade targets.
#[derive(Debug, Clone, Copy)]
pub struct TradeTargets {
    pub current_price: f64,
    pub stop_loss_price: f64,
    pub take_profit_price: f64,
    pub projected_pl_percent: f64,
    pub projected_pl_usd: f64,
}

/// Service responsible for calculating default or user-specific trade targets (SL/TP) plus projected P/L.
///
/// It is **stateless** and thus inexpensive to clone or share across threads.
pub struct TradeTargetCalculator;

impl TradeTargetCalculator {
    /// Calculate trade targets.
    ///
    /// * `current_price` – latest mid-price for the opportunity pair
    /// * `trade_size_usdt` – user's position size (in USDT). If `None`, service falls back to sensible default.
    /// * `settings` – optional user-level TradingSettings to honour personal SL/TP percentages & risk tolerance.
    pub fn calculate(
        current_price: f64,
        trade_size_usdt: Option<f64>,
        settings: Option<&TradingSettings>,
    ) -> ArbitrageResult<TradeTargets> {
        if current_price <= 0.0 {
            return Err(ArbitrageError::validation_error(
                "Current price must be positive for trade-target calculation".to_string(),
            ));
        }

        let (sl_pct, tp_pct, size) = match settings {
            Some(s) => (
                s.stop_loss_percentage.max(0.1), // clamp to avoid zero
                s.take_profit_percentage.max(0.2),
                trade_size_usdt.unwrap_or_else(|| DEFAULT_TRADE_SIZE_USDT.min(s.max_position_size)),
            ),
            None => (
                DEFAULT_STOP_LOSS_PERCENT,
                DEFAULT_TAKE_PROFIT_PERCENT,
                trade_size_usdt.unwrap_or(DEFAULT_TRADE_SIZE_USDT),
            ),
        };

        // Convert percentages to decimals
        let sl_dec = sl_pct / 100.0;
        let tp_dec = tp_pct / 100.0;

        let stop_loss_price = current_price * (1.0 - sl_dec);
        let take_profit_price = current_price * (1.0 + tp_dec);

        // Projected P/L in USD (assuming exit at TP)
        let projected_pl_usd = size * tp_dec;
        let projected_pl_percent = tp_pct;

        Ok(TradeTargets {
            current_price,
            stop_loss_price,
            take_profit_price,
            projected_pl_percent,
            projected_pl_usd,
        })
    }
}

// Tests have been moved to packages/worker/tests/trading/trade_target_calculator_test.rs
