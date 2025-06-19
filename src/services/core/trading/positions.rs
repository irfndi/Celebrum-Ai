// src/services/positions.rs
use std::sync::Arc;

use crate::services::core::user::user_profile::UserProfileService;
use crate::types::{
    AccountInfo, ArbitragePosition, CommandPermission, ExchangeIdEnum, Position, PositionAction,
    PositionSide, PositionStatus,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
// use std::collections::HashMap; // Removed unused import
// use worker::kv::KvStore; // Replaced with KvOperations trait
use crate::services::core::trading::{KvOperationError, KvOperations};

/// Data structure for creating a new position
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePositionData {
    pub pair: String, // Renamed from symbol
    pub side: PositionSide,
    pub size: Option<f64>,
    pub size_usd: Option<f64>, // Added field
    pub entry_price_long: f64,
    pub entry_price_short: f64,
    pub risk_percentage: Option<f64>,
    pub max_size_usd: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub long_exchange: ExchangeIdEnum,
    pub short_exchange: ExchangeIdEnum,
    pub exchange: ExchangeIdEnum, // Added field
}

/// Data structure for updating an existing position
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePositionData {
    pub take_profit_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub status: Option<PositionStatus>,
    pub size: Option<f64>,
    pub current_price: Option<f64>, // Added field
    pub pnl: Option<f64>,           // Added field
}

/// Production positions service type alias
// pub type ProductionPositionsService =
//     PositionsService<crate::services::core::infrastructure::kv::KVService>;

// PositionsService now uses a generic KV store
#[derive(Clone)]
pub struct PositionsService<T: KvOperations + Send + Sync + 'static> {
    kv_store: Arc<T>,
    user_profile_service: Option<UserProfileService>,
}

impl<T: KvOperations + Send + Sync + 'static> PositionsService<T> {
    pub fn new(kv_store: Arc<T>) -> Self {
        Self {
            kv_store,
            user_profile_service: None,
        }
    }

    // Helper to create a KvStore key for a position
    fn position_key(id: &str) -> String {
        format!("position:{}", id)
    }

    // Helper to create a KvStore key for user positions
    #[allow(dead_code)] // Will be used for position management
    fn user_positions_key(user_id: &str) -> String {
        format!("user_positions:{}", user_id)
    }

    /// Set the UserProfile service for database-based RBAC
    pub fn set_user_profile_service(&mut self, user_profile_service: UserProfileService) {
        self.user_profile_service = Some(user_profile_service);
    }

    /// Check if user has required permission using database-based RBAC
    #[allow(dead_code)] // Will be used for permission checking
    async fn check_user_permission(&self, user_id: &str, permission: &CommandPermission) -> bool {
        // If UserProfile service is not available, deny access for security
        let Some(ref user_profile_service) = self.user_profile_service else {
            // For critical position operations, always deny if RBAC is not configured
            return false;
        };

        // Get user profile from database to check their role
        let user_profile = match user_profile_service
            .get_user_by_telegram_id(user_id.parse::<i64>().unwrap_or(0))
            .await
        {
            Ok(Some(profile)) => profile,
            _ => {
                // If user not found in database or error occurred, no permissions
                return false;
            }
        };

        // Use the existing UserProfile permission checking method
        user_profile.has_permission(permission.clone())
    }

    pub async fn create_position(
        &self,
        position_data: CreatePositionData,
        account_info: &AccountInfo,
    ) -> ArbitrageResult<ArbitragePosition> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis() as u64;

        let final_size_base_currency: f64;
        let calculated_size_usd_for_audit: Option<f64>;
        let mut _risk_percentage_applied_for_audit: Option<f64> = None;

        if let Some(risk_perc) = position_data.risk_percentage {
            if position_data.entry_price_long <= 0.0 {
                return Err(ArbitrageError::validation_error(
                    "Entry price must be positive for risk-based sizing.".to_string(),
                ));
            }
            _risk_percentage_applied_for_audit = Some(risk_perc);
            let mut amount_to_risk_usd = account_info.total_balance_usd * risk_perc;

            if let Some(max_usd) = position_data.max_size_usd {
                amount_to_risk_usd = amount_to_risk_usd.min(max_usd);
            }

            final_size_base_currency = amount_to_risk_usd / position_data.entry_price_long;
            calculated_size_usd_for_audit = Some(amount_to_risk_usd);
        } else if let Some(fixed_usd_size) = position_data.size_usd {
            if position_data.entry_price_long <= 0.0 {
                return Err(ArbitrageError::validation_error(
                    "Entry price must be positive for fixed USD sizing.".to_string(),
                ));
            }
            final_size_base_currency = fixed_usd_size / position_data.entry_price_long;
            calculated_size_usd_for_audit = Some(fixed_usd_size);
        } else {
            return Err(ArbitrageError::validation_error(
                "Position size not specified: either risk_percentage or size_usd must be provided."
                    .to_string(),
            ));
        }

        if final_size_base_currency <= 0.0 {
            return Err(ArbitrageError::validation_error(
                format!("Calculated position size in base currency is not positive: {}. Check inputs: entry_price, balance, risk_percentage, or size_usd.", final_size_base_currency)
            ));
        }

        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let position = ArbitragePosition {
            id: id.clone(),
            user_id: "system".to_string(), // TODO: Use actual user_id
            opportunity_id: "manual".to_string(), // TODO: Use actual opportunity_id if available
            long_position: Position {
                info: serde_json::json!({}),
                id: Some(id.clone()),
                symbol: position_data.pair.clone(),
                timestamp: now,
                datetime: chrono::DateTime::from_timestamp(now as i64 / 1000, 0)
                    .unwrap()
                    .to_rfc3339(),
                isolated: Some(false),
                hedged: Some(false),
                side: "long".to_string(),
                amount: final_size_base_currency,
                contracts: None,
                contract_size: None,
                entry_price: Some(position_data.entry_price_long),
                mark_price: None,
                notional: None,
                leverage: Some(1.0),
                collateral: None,
                initial_margin: None,
                initial_margin_percentage: None,
                maintenance_margin: None,
                maintenance_margin_percentage: None,
                unrealized_pnl: Some(0.0),
                realized_pnl: Some(0.0),
                percentage: None,
            },
            short_position: Position {
                info: serde_json::json!({}),
                id: None, // Short position might not have an ID initially
                symbol: position_data.pair.clone(),
                timestamp: now,
                datetime: chrono::DateTime::from_timestamp(now as i64 / 1000, 0)
                    .unwrap()
                    .to_rfc3339(),
                isolated: Some(false),
                hedged: Some(false),
                side: "short".to_string(),
                amount: 0.0, // Assuming short side is not opened yet or handled elsewhere
                contracts: None,
                contract_size: None,
                entry_price: None, // No entry for short side yet
                mark_price: None,
                notional: None,
                leverage: Some(1.0),
                collateral: None,
                initial_margin: None,
                initial_margin_percentage: None,
                maintenance_margin: None,
                maintenance_margin_percentage: None,
                unrealized_pnl: Some(0.0),
                realized_pnl: Some(0.0),
                percentage: None,
            },
            status: PositionStatus::Open,
            entry_time: now,
            exit_time: None,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            total_fees: 0.0,
            risk_score: 0.5,  // This is f64, not Option<f64>
            margin_used: 0.0, // This is f64, not Option<f64>
            symbol: position_data.pair.clone(),
            side: position_data.side,
            entry_price_long: position_data.entry_price_long,
            take_profit_price: None,
            volatility_score: None,
            calculated_size_usd: calculated_size_usd_for_audit,
            long_exchange: position_data.long_exchange,
            size: Some(final_size_base_currency),
            pnl: Some(0.0),
            unrealized_pnl_percentage: Some(0.0),
            max_drawdown: None,
            created_at: now,
            holding_period_hours: None,
            trailing_stop_distance: None,
            stop_loss_price: None,
            closed_at: None,
            current_price_long: None, // Should be updated by market data later
            current_price_short: None, // Should be updated by market data later
            updated_at: now_ms,
            short_exchange: position_data.short_exchange,
            current_price: None, // Should be updated by market data later
            max_loss_usd: None,  // TODO: Calculate if stop_loss_price and size are known
            exchange: position_data.exchange,
            pair: position_data.pair,
            related_positions: Vec::new(),
            entry_price_short: 0.0, // Added: Initialize with a default value
            risk_reward_ratio: None,
            last_optimization_check: None,
            hedge_position_id: None,
            position_group_id: None, // TODO: Consider if this needs a value
            current_state: Some("created".to_string()), // Initial state
            recommended_action: None,
            risk_percentage_applied: _risk_percentage_applied_for_audit,
            optimization_score: None,
        };

        // Store position
        let key = Self::position_key(&id);
        // let value = serde_json::to_string(&position).map_err(|e| { // Confirmed: This line remains commented out as 'value' is not used with the direct put below.
        //     ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        // })?;

        self.kv_store.put(&key, &position).await.map_err(|e| {
            ArbitrageError::storage_error(format!(
                "KV store put operation failed for position {}: {:?}",
                position.id, e
            ))
        })?;

        // Update position index
        self.add_to_position_index(&id).await?;

        Ok(position)
    }

    pub async fn get_position(&self, id: &str) -> ArbitrageResult<Option<ArbitragePosition>> {
        let key = Self::position_key(id); // Use helper

        match self.kv_store.get::<ArbitragePosition>(&key).await {
            Ok(Some(position)) => Ok(Some(position)),
            Ok(None) => Ok(None),
            Err(KvOperationError::NotFound(_)) => Ok(None), // Explicitly handle NotFound from KvOperations
            Err(e) => Err(ArbitrageError::database_error(format!(
                "Failed to get position {}: {:?}",
                id, e
            ))),
        }
    }

    pub async fn update_position(
        &self,
        id: &str,
        update_data: UpdatePositionData,
    ) -> ArbitrageResult<Option<ArbitragePosition>> {
        let position_key = Self::position_key(id);
        let mut position: ArbitragePosition = match self.kv_store.get(&position_key).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        if let Some(size) = update_data.size {
            position.size = Some(size);
            // TODO: Recalculate notional, margin based on new size if applicable
        }
        if let Some(price) = update_data.current_price {
            // Assuming this update is for the long position's price for now
            position.current_price_long = Some(price);
            // TODO: Need to clarify if short_position price also needs update or if this is generic current price
        }
        if let Some(pnl) = update_data.pnl {
            position.pnl = Some(pnl);
            // TODO: Differentiate between realized and unrealized PNL updates if necessary
        }
        if let Some(status) = update_data.status {
            position.status = status;
            if position.status == PositionStatus::Closed
                || position.status == PositionStatus::Liquidated
            {
                position.exit_time = Some(chrono::Utc::now().timestamp_millis() as u64);
                position.closed_at = Some(chrono::Utc::now().timestamp_millis() as u64);
            }
        }

        // Update timestamp
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64; // Corrected: Direct u64 assignment

        self.kv_store
            .put(&position_key, &position)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to update position {}: {}", id, e))
            })?;

        Ok(Some(position))
    }

    pub async fn close_position(&self, id: &str) -> ArbitrageResult<bool> {
        let position_key = Self::position_key(id);
        let mut position: ArbitragePosition = match self.kv_store.get(&position_key).await? {
            Some(p) => p,
            None => return Ok(false), // Position not found
        };

        if position.status == PositionStatus::Closed
            || position.status == PositionStatus::Liquidated
        {
            return Ok(true); // Already closed
        }

        position.status = PositionStatus::Closed;
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        position.exit_time = Some(now_ms);
        position.closed_at = Some(now_ms);
        position.updated_at = now_ms; // Ensure this is a direct u64 assignment

        self.kv_store
            .put(&position_key, &position)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!("Failed to close position {}: {}", id, e))
            })?;
        self.remove_from_position_index(id).await.map_err(|e| {
            ArbitrageError::storage_error(format!(
                "Failed to update position index for closed position {}: {}",
                id, e
            ))
        })?;

        Ok(true)
    }

    pub async fn get_all_positions(&self) -> ArbitrageResult<Vec<ArbitragePosition>> {
        // Get position IDs from index
        let position_ids = self.get_position_index().await?;
        let mut positions = Vec::new();

        for id in position_ids {
            if let Some(position) = self.get_position(&id).await? {
                positions.push(position);
            }
        }

        Ok(positions)
    }

    pub async fn get_open_positions(&self) -> ArbitrageResult<Vec<ArbitragePosition>> {
        let all_positions = self.get_all_positions().await?;
        Ok(all_positions
            .into_iter()
            .filter(|pos| pos.status == PositionStatus::Open)
            .collect())
    }

    pub async fn calculate_total_pnl(&self) -> ArbitrageResult<f64> {
        let positions = self.get_open_positions().await?;
        let total_pnl = positions.iter().filter_map(|pos| pos.pnl).sum();
        Ok(total_pnl)
    }

    // Helper methods for position index management
    async fn get_position_index(&self) -> ArbitrageResult<Vec<String>> {
        match self.kv_store.get::<Vec<String>>("positions:index").await {
            Ok(Some(ids)) => Ok(ids),
            Ok(None) => Ok(Vec::new()), // If no index exists, return an empty Vec
            Err(KvOperationError::NotFound(_)) => Ok(Vec::new()), // Explicitly handle NotFound
            Err(e) => Err(ArbitrageError::database_error(format!(
                "Failed to get position index: {:?}",
                e
            ))),
        }
    }

    async fn add_to_position_index(&self, position_id: &str) -> ArbitrageResult<()> {
        let mut index = self.get_position_index().await?;
        if !index.contains(&position_id.to_string()) {
            index.push(position_id.to_string());
            self.save_position_index(&index).await?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    async fn remove_from_position_index(&self, position_id: &str) -> ArbitrageResult<()> {
        let mut index = self.get_position_index().await?;
        index.retain(|id| id != position_id);
        self.save_position_index(&index).await?;
        Ok(())
    }

    async fn save_position_index(&self, index: &[String]) -> ArbitrageResult<()> {
        self.kv_store
            .put("positions:index", index)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for index: {:?}",
                    e
                ))
            })?; // Removed .execute().await as it's handled by the trait

        Ok(())
    }

    // Advanced Position Management Methods (Task 6)

    /// Set stop-loss for a position
    pub async fn set_stop_loss(
        &self,
        position_id: &str,
        stop_loss_price: f64,
    ) -> ArbitrageResult<bool> {
        let position_key = Self::position_key(position_id);
        let mut position: ArbitragePosition = match self.kv_store.get(&position_key).await? {
            Some(p) => p,
            None => return Ok(false), // Position not found
        };

        if position.status != PositionStatus::Open {
            return Err(ArbitrageError::validation_error(
                "Stop loss can only be set for open positions.".to_string(),
            ));
        }

        // Basic validation: for long, stop_loss < entry; for short, stop_loss > entry
        // More complex validation might be needed depending on strategy
        match position.side {
            PositionSide::Long => {
                if stop_loss_price >= position.entry_price_long {
                    return Err(ArbitrageError::validation_error(
                        "Stop loss price must be below entry price for a long position."
                            .to_string(),
                    ));
                }
            }
            PositionSide::Short => {
                // Check if entry_price_short is meaningfully set (e.g., > 0.0)
                if position.entry_price_short > 0.0 {
                    if stop_loss_price <= position.entry_price_short {
                        return Err(ArbitrageError::validation_error(
                            "Stop loss price must be above entry price for a short position."
                                .to_string(),
                        ));
                    }
                } else {
                    // Handle cases where entry_price_short is not set (e.g., log, error, or specific logic)
                    return Err(ArbitrageError::validation_error(
                        "Cannot set stop loss for short side without a valid entry price for the short leg.".to_string(),
                    ));
                }
            }
            PositionSide::Both => {
                // For hedge positions, this logic might need to be more complex.
                // Assuming for now it follows similar rules to Long or needs specific handling.
                // Placeholder: if stop_loss is outside a combined range or similar.
                // For simplicity, let's ensure it's not equal to entry price for now.
                if stop_loss_price == position.entry_price_long
                    || (position.entry_price_short > 0.0 // Corrected: Check if > 0.0 instead of is_some()
                        && stop_loss_price == position.entry_price_short)
                // Corrected: Direct access instead of unwrap()
                {
                    return Err(ArbitrageError::validation_error(
                        "Stop loss price for a 'Both' position requires specific validation."
                            .to_string(),
                    ));
                }
            }
        }

        position.stop_loss_price = Some(stop_loss_price);
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64; // Ensure u64 assignment

        self.kv_store
            .put(&position_key, &position)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to set stop loss for position {}: {}",
                    position_id, e
                ))
            })?;

        Ok(true)
    }

    /// Set take-profit for a position
    pub async fn set_take_profit(
        &self,
        position_id: &str,
        take_profit_price: f64,
    ) -> ArbitrageResult<bool> {
        let position_key = Self::position_key(position_id);
        let mut position: ArbitragePosition = match self.kv_store.get(&position_key).await? {
            Some(p) => p,
            None => return Ok(false), // Position not found
        };

        if position.status != PositionStatus::Open {
            return Err(ArbitrageError::validation_error(
                "Take profit can only be set for open positions.".to_string(),
            ));
        }

        // Basic validation: for long, take_profit > entry; for short, take_profit < entry
        match position.side {
            PositionSide::Long => {
                if take_profit_price <= position.entry_price_long {
                    return Err(ArbitrageError::validation_error(
                        "Take profit price must be above entry price for a long position."
                            .to_string(),
                    ));
                }
            }
            PositionSide::Short => {
                // Check if entry_price_short is meaningfully set
                if position.entry_price_short > 0.0 {
                    if take_profit_price >= position.entry_price_short {
                        return Err(ArbitrageError::validation_error(
                            "Take profit price must be below entry price for a short position."
                                .to_string(),
                        ));
                    }
                } else {
                    return Err(ArbitrageError::validation_error(
                        "Cannot set take profit for short side without a valid entry price for the short leg.".to_string(),
                    ));
                }
            }
            PositionSide::Both => {
                // For hedge positions, this logic might need to be more complex.
                // Placeholder: Similar validation as for stop_loss.
                if take_profit_price == position.entry_price_long
                    || (position.entry_price_short > 0.0 // Corrected: Check if > 0.0 instead of is_some()
                        && take_profit_price == position.entry_price_short)
                // Corrected: Direct access instead of unwrap()
                {
                    return Err(ArbitrageError::validation_error(
                        "Take profit price for a 'Both' position requires specific validation."
                            .to_string(),
                    ));
                }
            }
        }

        position.take_profit_price = Some(take_profit_price);
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64; // Ensure u64 assignment

        self.kv_store
            .put(&position_key, &position)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to set take profit for position {}: {}",
                    position_id, e
                ))
            })?;

        Ok(true)
    }

    /// Enable trailing stop for a position
    pub async fn enable_trailing_stop(
        &self,
        position_id: &str,
        trailing_distance: f64, // Can be percentage or absolute price offset
    ) -> ArbitrageResult<bool> {
        let position_key = Self::position_key(position_id);
        let mut position: ArbitragePosition = match self.kv_store.get(&position_key).await? {
            Some(p) => p,
            None => return Ok(false), // Position not found
        };

        if position.status != PositionStatus::Open {
            return Err(ArbitrageError::validation_error(
                "Trailing stop can only be enabled for open positions.".to_string(),
            ));
        }

        if trailing_distance <= 0.0 {
            return Err(ArbitrageError::validation_error(
                "Trailing stop distance must be positive.".to_string(),
            ));
        }

        // Assuming trailing_distance is a price offset for now
        // Logic for percentage-based would be more complex
        position.trailing_stop_distance = Some(trailing_distance);
        // Initial activation of trailing stop might set a concrete stop_loss_price here
        // based on current price and distance, or this is handled by a separate process.
        // For simplicity, just enabling the parameter.
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64; // Ensure u64 assignment

        self.kv_store
            .put(&position_key, &position)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to enable trailing stop for position {}: {}",
                    position_id, e
                ))
            })?;

        Ok(true)
    }

    /// Update position with current market price and calculate metrics
    pub async fn update_position_price(
        &self,
        position_id: &str,
        current_price: f64, // This is the new mark price for the main leg (e.g., long)
    ) -> ArbitrageResult<bool> {
        let position_key = Self::position_key(position_id);
        let mut position: ArbitragePosition = match self.kv_store.get(&position_key).await? {
            Some(p) => p,
            None => return Ok(false), // Position not found
        };

        if position.status != PositionStatus::Open {
            return Ok(false); // No updates for closed positions
        }

        // Update the relevant current price based on position side
        // This is a simplified model; a real system might get bid/ask for long/short legs separately
        let entry_price_for_pnl_calc = match position.side {
            PositionSide::Long => {
                position.current_price_long = Some(current_price);
                position.long_position.mark_price = Some(current_price);
                position.entry_price_long
            }
            PositionSide::Short => {
                position.current_price_short = Some(current_price);
                position.short_position.mark_price = Some(current_price);
                // Use entry_price_short directly since it's f64
                // If PnL calculation needs a specific fallback, that logic should be here.
                if position.entry_price_short > 0.0 {
                    position.entry_price_short
                } else {
                    current_price // Or handle error / specific logic if entry_price_short is required
                } // Corrected: Ensure this is the expression returned by the block
            }
            PositionSide::Both => {
                // For 'Both', we might need to update both long and short current prices
                // or have a primary leg. Assuming long leg is primary for now.
                position.current_price_long = Some(current_price);
                position.long_position.mark_price = Some(current_price);
                // If there's a distinct short leg price update mechanism, it should be handled.
                // For PnL calculation, 'Both' might be complex (e.g. net PnL of two legs).
                // Sticking to long leg for now for pnl_calc.
                position.entry_price_long
            }
        };

        let size = position.size.unwrap_or(0.0);
        let pnl_factor = if position.side == PositionSide::Long {
            1.0
        } else {
            -1.0
        };

        if size > 0.0 && entry_price_for_pnl_calc > 0.0 {
            let pnl = (current_price - entry_price_for_pnl_calc) * size * pnl_factor;
            position.unrealized_pnl = pnl;
            position.pnl = Some(pnl); // Assuming pnl field tracks unrealized_pnl for open positions

            let pnl_percentage = (pnl / (entry_price_for_pnl_calc * size)) * 100.0;
            position.unrealized_pnl_percentage = Some(pnl_percentage);

            // Update individual leg PNL if structure supports it
            if position.side == PositionSide::Long {
                position.long_position.unrealized_pnl = Some(pnl);
                position.long_position.percentage = Some(pnl_percentage);
            } else {
                position.short_position.unrealized_pnl = Some(pnl);
                position.short_position.percentage = Some(pnl_percentage);
            }
        }

        // Trailing stop logic
        if let Some(trailing_distance) = position.trailing_stop_distance {
            if position.side == PositionSide::Long {
                let new_stop_loss = current_price - trailing_distance;
                if new_stop_loss > position.stop_loss_price.unwrap_or(0.0) {
                    position.stop_loss_price = Some(new_stop_loss);
                }
            } else {
                // PositionSide::Short
                let new_stop_loss = current_price + trailing_distance;
                if new_stop_loss < position.stop_loss_price.unwrap_or(f64::MAX) {
                    position.stop_loss_price = Some(new_stop_loss);
                }
            }
        }

        position.updated_at = chrono::Utc::now().timestamp_millis() as u64; // Ensure u64 assignment

        self.kv_store
            .put(&position_key, &position)
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "Failed to update price for position {}: {}",
                    position_id, e
                ))
            })?;

        Ok(true)
    }

    /// Check if position should be closed based on risk management rules
    pub async fn check_risk_triggers(
        &self,
        position_id: &str,
    ) -> ArbitrageResult<Option<PositionAction>> {
        let position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(None),
        };

        if let Some(current_price) = position.current_price {
            // Check stop-loss trigger
            if let Some(stop_loss) = position.stop_loss_price {
                let should_close = match position.side {
                    PositionSide::Long => current_price <= stop_loss,
                    PositionSide::Short => current_price >= stop_loss,
                    PositionSide::Both => {
                        // For hedge positions, check both sides
                        // This is a simplified implementation
                        current_price <= stop_loss || current_price >= stop_loss
                    }
                };
                if should_close {
                    return Ok(Some(PositionAction::Close));
                }
            }

            // Check take-profit trigger
            if let Some(take_profit) = position.take_profit_price {
                let should_close = match position.side {
                    PositionSide::Long => current_price >= take_profit,
                    PositionSide::Short => current_price <= take_profit,
                    PositionSide::Both => {
                        // For hedge positions, check both sides
                        // This is a simplified implementation
                        current_price >= take_profit || current_price <= take_profit
                    }
                };
                if should_close {
                    return Ok(Some(PositionAction::Close));
                }
            }

            // Check max loss trigger
            if let Some(max_loss) = position.max_loss_usd {
                if let Some(pnl) = position.pnl {
                    if pnl <= -max_loss {
                        return Ok(Some(PositionAction::Close));
                    }
                }
            }
        }

        Ok(Some(PositionAction::Hold))
    }

    /// Get positions by exchange for multi-exchange tracking
    pub async fn get_positions_by_exchange(
        &self,
        exchange: &ExchangeIdEnum,
    ) -> ArbitrageResult<Vec<ArbitragePosition>> {
        let all_positions = self.get_all_positions().await?;
        let filtered_positions = all_positions
            .into_iter()
            .filter(|pos| pos.exchange == *exchange)
            .collect();
        Ok(filtered_positions)
    }

    /// Get positions by trading pair
    pub async fn get_positions_by_pair(
        &self,
        pair: &str,
    ) -> ArbitrageResult<Vec<ArbitragePosition>> {
        let all_positions = self.get_all_positions().await?;
        let filtered_positions = all_positions
            .into_iter()
            .filter(|pos| pos.pair == pair)
            .collect();
        Ok(filtered_positions)
    }
}
