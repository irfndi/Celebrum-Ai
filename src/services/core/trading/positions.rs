// src/services/positions.rs
use std::sync::Arc;

use crate::services::core::user::user_profile::UserProfileService;
use crate::types::{
    AccountInfo, ArbitragePosition, CommandPermission, ExchangeIdEnum, Position, PositionAction,
    PositionOptimizationResult, PositionSide, PositionStatus, RiskAssessment, RiskLevel,
    RiskManagementConfig,
};
use crate::utils::{ArbitrageError, ArbitrageResult};
// use std::collections::HashMap; // Removed unused import
// use worker::kv::KvStore; // Replaced with KvOperations trait
use crate::services::core::trading::{KvOperationError, KvOperations, KvResult};

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
    fn user_positions_key(user_id: &str) -> String {
        format!("user_positions:{}", user_id)
    }

    /// Set the UserProfile service for database-based RBAC
    pub fn set_user_profile_service(&mut self, user_profile_service: UserProfileService) {
        self.user_profile_service = Some(user_profile_service);
    }

    /// Check if user has required permission using database-based RBAC
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
            long_exchange: position_data.exchange, // This is ExchangeIdEnum, not Option<ExchangeIdEnum>
            size: Some(final_size_base_currency),
            pnl: Some(0.0),
            unrealized_pnl_percentage: Some(0.0),
            max_drawdown: None,
            created_at: now,
            holding_period_hours: None,
            trailing_stop_distance: None,
            stop_loss_price: None,
            closed_at: None,
            current_price_long: Some(45100.0),  // Example value
            current_price_short: Some(45050.0), // Example value
            updated_at: now_ms,
            short_exchange: ExchangeIdEnum::Kraken, // Corrected type
            current_price: None,                    // Should be updated by market data later
            max_loss_usd: None, // TODO: Calculate if stop_loss_price and size are known
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
        let key = Self::position_key(&id); // Use helper
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
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
        let mut position = match self.get_position(id).await? {
            Some(pos) => pos,
            None => return Ok(None),
        };

        // Update fields if provided
        if let Some(size) = update_data.size {
            position.size = Some(size);
            position.long_position.amount = size; // Update the actual position size
        }
        if let Some(current_price) = update_data.current_price {
            position.current_price = Some(current_price);
        }
        if let Some(pnl) = update_data.pnl {
            position.pnl = Some(pnl);
        }
        if let Some(status) = update_data.status {
            position.status = status;
        }

        position.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Store updated position
        let key = Self::position_key(id); // Use helper
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for position {}: {:?}",
                    position.id, e
                ))
            })?;

        Ok(Some(position))
    }

    pub async fn close_position(&self, id: &str) -> ArbitrageResult<bool> {
        let update_data = UpdatePositionData {
            size: None,
            current_price: None,
            pnl: None,
            status: Some(PositionStatus::Closed),
        };

        match self.update_position(id, update_data).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
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
        let mut position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(false),
        };

        // Validate stop-loss price based on position side
        match position.side {
            PositionSide::Long => {
                if stop_loss_price >= position.entry_price_long {
                    return Err(ArbitrageError::validation_error(
                        "Stop-loss price for long position must be below entry price".to_string(),
                    ));
                }
            }
            PositionSide::Short => {
                if stop_loss_price <= position.entry_price_long {
                    return Err(ArbitrageError::validation_error(
                        "Stop-loss price for short position must be above entry price".to_string(),
                    ));
                }
            }
            PositionSide::Both => {
                // For hedge positions, validate against both entry prices
                return Err(ArbitrageError::validation_error(
                    "Stop-loss validation for hedge positions not implemented".to_string(),
                ));
            }
        }

        position.stop_loss_price = Some(stop_loss_price);
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Calculate max loss in USD
        let price_diff = (position.entry_price_long - stop_loss_price).abs();
        position.max_loss_usd = Some(price_diff * position.size.unwrap_or(0.0));

        // Store updated position
        let key = Self::position_key(position_id); // Use helper
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for position {}: {:?}",
                    position.id, e
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
        let mut position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(false),
        };

        // Validate take-profit price based on position side
        match position.side {
            PositionSide::Long => {
                if take_profit_price <= position.entry_price_long {
                    return Err(ArbitrageError::validation_error(
                        "Take-profit price for long position must be above entry price".to_string(),
                    ));
                }
            }
            PositionSide::Short => {
                if take_profit_price >= position.entry_price_long {
                    return Err(ArbitrageError::validation_error(
                        "Take-profit price for short position must be below entry price"
                            .to_string(),
                    ));
                }
            }
            PositionSide::Both => {
                // For hedge positions, validate against both entry prices
                return Err(ArbitrageError::validation_error(
                    "Take-profit validation for hedge positions not implemented".to_string(),
                ));
            }
        }

        position.take_profit_price = Some(take_profit_price);
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Calculate risk/reward ratio if stop-loss is set
        if let Some(stop_loss) = position.stop_loss_price {
            let risk = (position.entry_price_long - stop_loss).abs();
            let reward = (take_profit_price - position.entry_price_long).abs();
            position.risk_reward_ratio = Some(reward / risk);
        }

        // Store updated position
        let key = Self::position_key(position_id); // Use helper
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for position {}: {:?}",
                    position.id, e
                ))
            })?;

        Ok(true)
    }

    /// Enable trailing stop for a position
    pub async fn enable_trailing_stop(
        &self,
        position_id: &str,
        trailing_distance: f64,
    ) -> ArbitrageResult<bool> {
        let mut position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(false),
        };

        if trailing_distance <= 0.0 {
            return Err(ArbitrageError::validation_error(
                "Trailing stop distance must be positive".to_string(),
            ));
        }

        position.trailing_stop_distance = Some(trailing_distance);
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Store updated position
        let key = Self::position_key(position_id); // Use helper
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for position {}: {:?}",
                    position.id, e
                ))
            })?;

        Ok(true)
    }

    /// Update position with current market price and calculate metrics
    pub async fn update_position_price(
        &self,
        position_id: &str,
        current_price: f64,
    ) -> ArbitrageResult<bool> {
        let mut position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(false),
        };

        let previous_price = position.current_price;
        position.current_price = Some(current_price);
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Calculate PnL
        let price_diff = match position.side {
            PositionSide::Long => current_price - position.entry_price_long,
            PositionSide::Short => position.entry_price_long - current_price,
            PositionSide::Both => {
                // For hedge positions, calculate combined PnL from both sides
                // This is a simplified implementation - in practice, you'd track both positions separately
                current_price - position.entry_price_long
            }
        };
        let pnl = price_diff * position.size.unwrap_or(0.0);
        position.pnl = Some(pnl);

        // Calculate unrealized PnL percentage
        let entry_value = position.entry_price_long * position.size.unwrap_or(0.0);
        position.unrealized_pnl_percentage = Some((pnl / entry_value) * 100.0);

        // Update max drawdown
        if let Some(prev_max_drawdown) = position.max_drawdown {
            if pnl < 0.0 && pnl < prev_max_drawdown {
                position.max_drawdown = Some(pnl);
            }
        } else if pnl < 0.0 {
            position.max_drawdown = Some(pnl);
        }

        // Calculate holding period
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let holding_period_ms = now - position.created_at;
        position.holding_period_hours = Some(holding_period_ms as f64 / (1000.0 * 60.0 * 60.0));

        // Update trailing stop if enabled
        if let Some(trailing_distance) = position.trailing_stop_distance {
            let new_stop_loss = match position.side {
                PositionSide::Long => current_price - trailing_distance,
                PositionSide::Short => current_price + trailing_distance,
                PositionSide::Both => {
                    // For hedge positions, use the long side logic as default
                    // TODO: Implement proper hedge position update logic
                    current_price - trailing_distance
                }
            };

            // Only update if the new stop-loss is more favorable
            if let Some(current_stop_loss) = position.stop_loss_price {
                let should_update = match position.side {
                    PositionSide::Long => new_stop_loss > current_stop_loss,
                    PositionSide::Short => new_stop_loss < current_stop_loss,
                    PositionSide::Both => {
                        // For hedge positions, use the long side logic as default
                        // TODO: Implement proper hedge position update logic
                        new_stop_loss > current_stop_loss
                    }
                };
                if should_update {
                    position.stop_loss_price = Some(new_stop_loss);
                }
            } else {
                position.stop_loss_price = Some(new_stop_loss);
            }
        }

        // Calculate volatility score based on price movements
        if let Some(prev_price) = previous_price {
            let price_change_pct = ((current_price - prev_price) / prev_price).abs() * 100.0;
            // Simple volatility score (in a real implementation, this would use historical data)
            position.volatility_score = Some(price_change_pct);
        }

        // Store updated position
        let key = format!("position:{}", position_id);
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for position {}: {:?}",
                    position.id, e
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

    /// Link related positions for multi-exchange arbitrage
    pub async fn link_positions(
        &self,
        position_id: &str,
        related_position_ids: Vec<String>,
    ) -> ArbitrageResult<bool> {
        let mut position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(false),
        };

        position.related_positions = related_position_ids;
        position.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        // Store updated position
        let key = format!("position:{}", position_id);
        let value = serde_json::to_string(&position).map_err(|e| {
            ArbitrageError::parse_error(format!("Failed to serialize position: {}", e))
        })?;

        self.kv_store
            .put(&key, &position) // Pass position directly, trait method handles serialization
            .await
            .map_err(|e| {
                ArbitrageError::storage_error(format!(
                    "KV store put operation failed for position {}: {:?}",
                    position.id, e
                ))
            })?;

        Ok(true)
    }

    /// Analyze position and provide optimization recommendations
    pub async fn analyze_position(
        &self,
        position_id: &str,
        config: &RiskManagementConfig,
    ) -> ArbitrageResult<Option<PositionOptimizationResult>> {
        let position = match self.get_position(position_id).await? {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let mut score = 50.0; // Base score
        let mut recommended_action = PositionAction::Hold;
        let mut reasoning = String::new();
        let mut suggested_stop_loss = None;
        let mut suggested_take_profit = None;

        // Analyze current PnL
        if let Some(pnl) = position.pnl {
            if pnl > 0.0 {
                score += 20.0;
                reasoning.push_str("Position is profitable. ");
            } else {
                score -= 20.0;
                reasoning.push_str("Position is at a loss. ");
            }

            // Check if position size is too large
            if let Some(size_usd) = position.calculated_size_usd {
                if size_usd > config.max_position_size_usd {
                    score -= 15.0;
                    recommended_action = PositionAction::DecreaseSize;
                    reasoning.push_str("Position size exceeds maximum limit. ");
                }
            }
        }

        // Analyze risk/reward ratio
        if let Some(rr_ratio) = position.risk_reward_ratio {
            if rr_ratio >= config.min_risk_reward_ratio {
                score += 15.0;
                reasoning.push_str("Good risk/reward ratio. ");
            } else {
                score -= 10.0;
                reasoning.push_str("Poor risk/reward ratio. ");
            }
        }

        // Analyze holding period
        if let Some(holding_hours) = position.holding_period_hours {
            if holding_hours > 24.0 {
                score -= 5.0;
                reasoning.push_str("Long holding period. ");
            }
        }

        // Suggest stop-loss if not set
        if position.stop_loss_price.is_none() {
            score -= 10.0;
            recommended_action = PositionAction::StopLoss;
            let stop_loss_distance = position.entry_price_long * config.stop_loss_percentage;
            suggested_stop_loss = Some(match position.side {
                PositionSide::Long => position.entry_price_long - stop_loss_distance,
                PositionSide::Short => position.entry_price_long + stop_loss_distance,
                PositionSide::Both => position.entry_price_long - stop_loss_distance, // Default to long side
            });
            reasoning.push_str("No stop-loss set. ");
        }

        // Suggest take-profit if not set
        if position.take_profit_price.is_none() {
            let take_profit_distance = position.entry_price_long * config.take_profit_percentage;
            suggested_take_profit = Some(match position.side {
                PositionSide::Long => position.entry_price_long + take_profit_distance,
                PositionSide::Short => position.entry_price_long - take_profit_distance,
                PositionSide::Both => position.entry_price_long + take_profit_distance, // Default to long side
            });
            reasoning.push_str("Consider setting take-profit. ");
        }

        // Assess overall risk
        let risk_level = if score >= 70.0 {
            RiskLevel::Low
        } else if score >= 50.0 {
            RiskLevel::Medium
        } else if score >= 30.0 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        };

        let risk_assessment = RiskAssessment {
            overall_risk_level: risk_level.clone(),
            risk_score: 100.0 - score,
            market_risk: 0.0,
            volatility_risk: position.volatility_score.unwrap_or(0.0),
            correlation_risk: 0.0,
            recommendations: vec![reasoning.clone()],
            max_position_size: position.calculated_size_usd.unwrap_or(0.0),
            stop_loss_recommendation: suggested_stop_loss.unwrap_or(0.0),
            take_profit_recommendation: suggested_take_profit.unwrap_or(0.0),
            risk_level,
            concentration_risk: 0.0,
        };

        let confidence_level = if score >= 70.0 || score <= 30.0 {
            0.8
        } else {
            0.6
        };

        let result = PositionOptimizationResult {
            position_id: position_id.to_string(),
            optimization_score: score,
            current_score: score,
            optimized_score: score + 10.0, // Placeholder improvement
            recommended_actions: vec![format!("{:?}", recommended_action)],
            risk_assessment,
            expected_improvement: 10.0, // Placeholder improvement percentage
            confidence: confidence_level,
            confidence_level,
            recommended_action,
            reasoning,
            suggested_stop_loss: suggested_stop_loss.unwrap_or(0.0),
            suggested_take_profit: suggested_take_profit.unwrap_or(0.0),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };

        Ok(Some(result))
    }

    /// Calculate total exposure across all positions
    pub async fn calculate_total_exposure(&self) -> ArbitrageResult<f64> {
        // Removed kv_store argument
        let positions = self.get_all_positions().await?; // Removed kv_store argument from call
        let total_exposure = positions
            .iter()
            .filter(|pos| pos.status == PositionStatus::Open)
            .map(|pos| pos.calculated_size_usd.unwrap_or(0.0))
            .sum();
        Ok(total_exposure)
    }

    /// Validate position against risk management rules
    pub async fn validate_position_risk(
        &self,
        position_data: &CreatePositionData,
        config: &RiskManagementConfig,
    ) -> ArbitrageResult<()> {
        // Check position size limits
        if let Some(size_usd) = position_data.size_usd {
            if size_usd > config.max_position_size_usd {
                return Err(ArbitrageError::validation_error(format!(
                    "Position size {} exceeds maximum allowed {}",
                    size_usd, config.max_position_size_usd
                )));
            }
        }

        // Check total exposure
        let current_exposure = self.calculate_total_exposure().await?;
        let new_position_size = position_data.size_usd.unwrap_or(0.0);
        if current_exposure + new_position_size > config.max_total_exposure_usd {
            return Err(ArbitrageError::validation_error(format!(
                "Total exposure would exceed maximum allowed {}",
                config.max_total_exposure_usd
            )));
        }

        // Check positions per exchange limit
        let exchange_positions = self
            .get_positions_by_exchange(&position_data.exchange) // Removed &self.kv_store argument
            .await?;
        let open_positions_count = exchange_positions
            .iter()
            .filter(|pos| pos.status == PositionStatus::Open)
            .count() as u32;

        if open_positions_count >= config.max_positions_per_exchange {
            return Err(ArbitrageError::validation_error(format!(
                "Maximum positions per exchange ({}) reached",
                config.max_positions_per_exchange
            )));
        }

        // Check positions per pair limit
        let pair_positions = self
            .get_positions_by_pair(&position_data.pair) // Removed &self.kv_store argument
            .await?;
        let open_pair_positions_count = pair_positions
            .iter()
            .filter(|pos| pos.status == PositionStatus::Open)
            .count() as u32;

        if open_pair_positions_count >= config.max_positions_per_pair {
            return Err(ArbitrageError::validation_error(format!(
                "Maximum positions per pair ({}) reached",
                config.max_positions_per_pair
            )));
        }

        Ok(())
    }

    /// RBAC-protected position creation with permission checking
    pub async fn create_position_with_permission(
        &self,
        user_id: &str,
        position_data: CreatePositionData,
        account_info: &AccountInfo,
    ) -> ArbitrageResult<ArbitragePosition> {
        // Check ManualTrading permission for position creation
        if !self
            .check_user_permission(user_id, &CommandPermission::ManualTrading)
            .await
        {
            return Err(ArbitrageError::validation_error(
                "Insufficient permissions: ManualTrading required for position creation"
                    .to_string(),
            ));
        }

        // Call the original create_position method
        self.create_position(position_data, account_info) // Removed kv_store argument
            .await
    }

    /// RBAC-protected position closure with permission checking
    pub async fn close_position_with_permission(
        &self,
        user_id: &str,
        id: &str,
    ) -> ArbitrageResult<bool> {
        // Check ManualTrading permission for position closure
        if !self
            .check_user_permission(user_id, &CommandPermission::ManualTrading)
            .await
        {
            return Err(ArbitrageError::validation_error(
                "Insufficient permissions: ManualTrading required for position closure".to_string(),
            ));
        }

        // Call the original close_position method
        self.close_position(id).await
    }

    /// RBAC-protected stop loss setting with permission checking
    pub async fn set_stop_loss_with_permission(
        &self,
        user_id: &str,
        position_id: &str,
        stop_loss_price: f64,
    ) -> ArbitrageResult<bool> {
        // Check ManualTrading permission for stop loss management
        if !self
            .check_user_permission(user_id, &CommandPermission::ManualTrading)
            .await
        {
            return Err(ArbitrageError::validation_error(
                "Insufficient permissions: ManualTrading required for stop loss management"
                    .to_string(),
            ));
        }

        // Call the original set_stop_loss method
        self.set_stop_loss(position_id, stop_loss_price).await
    }

    /// RBAC-protected take profit setting with permission checking
    pub async fn set_take_profit_with_permission(
        &self,
        user_id: &str,
        position_id: &str,
        take_profit_price: f64,
    ) -> ArbitrageResult<bool> {
        // Check ManualTrading permission for take profit management
        if !self
            .check_user_permission(user_id, &CommandPermission::ManualTrading)
            .await
        {
            return Err(ArbitrageError::validation_error(
                "Insufficient permissions: ManualTrading required for take profit management"
                    .to_string(),
            ));
        }

        // Call the original set_take_profit method
        self.set_take_profit(position_id, take_profit_price).await
    }

    /// RBAC-protected position analytics with permission checking
    pub async fn get_positions_analytics_with_permission(
        &self,
        user_id: &str,
    ) -> ArbitrageResult<Vec<ArbitragePosition>> {
        // Check AdvancedAnalytics permission for position analytics
        if !self
            .check_user_permission(user_id, &CommandPermission::AdvancedAnalytics)
            .await
        {
            return Err(ArbitrageError::validation_error(
                "Insufficient permissions: AdvancedAnalytics required for position analytics"
                    .to_string(),
            ));
        }

        // Call the original get_all_positions method
        self.get_all_positions().await // Removed kv_store argument from call
    }
}

// Type alias for production use with KvStore
pub type ProductionPositionsService = PositionsService<worker::kv::KvStore>;

// Helper structs for position operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePositionData {
    pub exchange: ExchangeIdEnum,
    pub pair: String,
    pub side: PositionSide,
    pub size_usd: Option<f64>,
    pub entry_price_long: f64,
    pub risk_percentage: Option<f64>,
    pub max_size_usd: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePositionData {
    pub size: Option<f64>,
    pub current_price: Option<f64>,
    pub pnl: Option<f64>,
    pub status: Option<PositionStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::core::trading::{KvOperationError, KvOperations, KvResult};
    use crate::test_utils::mock_kv_store::MockKvStore;
    use crate::types::ExchangeIdEnum;
    use std::sync::Arc;

    // Helper to create a test position
    fn create_test_position(id: &str, pair: &str) -> ArbitragePosition {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        ArbitragePosition {
            entry_price_short: 0.0,  // Added: Initialize with a default value
            risk_reward_ratio: None, // Added based on struct definition
            id: id.to_string(),
            user_id: "test_user".to_string(),       // Added
            opportunity_id: "test_opp".to_string(), // Added
            long_position: Position {
                // Updated based on Position struct definition
                info: serde_json::Value::Null,
                id: Some("long_pos".to_string()),
                symbol: pair.to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                datetime: chrono::Utc::now().to_rfc3339(),
                isolated: Some(true),
                hedged: Some(false),
                side: "long".to_string(), // PositionSide::Long.to_string() or format!("{:?}", PositionSide::Long).to_lowercase(),
                amount: 1.0,
                contracts: Some(1.0),
                contract_size: Some(1.0),
                entry_price: Some(45000.0),
                mark_price: Some(45100.0),
                notional: Some(45000.0),
                leverage: Some(1.0),
                collateral: Some(45000.0),
                initial_margin: Some(1000.0),
                initial_margin_percentage: Some(0.1),
                maintenance_margin: Some(500.0),
                maintenance_margin_percentage: Some(0.05),
                unrealized_pnl: Some(100.0),
                realized_pnl: Some(0.0),
                percentage: Some(1.0),
            },
            short_position: Position {
                // Updated based on Position struct definition
                info: serde_json::Value::Null,
                id: Some("short_pos".to_string()),
                symbol: pair.to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                datetime: chrono::Utc::now().to_rfc3339(),
                isolated: Some(true),
                hedged: Some(false),
                side: "short".to_string(), // PositionSide::Short.to_string() or format!("{:?}", PositionSide::Short).to_lowercase(),
                amount: 1.0,
                contracts: Some(1.0),
                contract_size: Some(1.0),
                entry_price: Some(45000.0),
                mark_price: Some(44900.0),
                notional: Some(45000.0),
                leverage: Some(1.0),
                collateral: Some(45000.0),
                initial_margin: Some(1000.0),
                initial_margin_percentage: Some(0.1),
                maintenance_margin: Some(500.0),
                maintenance_margin_percentage: Some(0.05),
                unrealized_pnl: Some(-100.0),
                realized_pnl: Some(0.0),
                percentage: Some(-1.0),
            },
            status: PositionStatus::Open,
            entry_time: chrono::Utc::now().timestamp_millis() as u64, // Added
            exit_time: None,                                          // Added
            realized_pnl: 0.0,                                        // Added
            unrealized_pnl: 0.0,                                      // Added
            total_fees: 0.0,                                          // Added
            risk_score: 0.5,                                          // Added
            margin_used: 2000.0,                                      // Added
            symbol: pair.to_string(),                                 // Added
            side: PositionSide::Long, // This seems to be for the overall arbitrage position
            entry_price_long: 45000.0,
            take_profit_price: None,
            volatility_score: None,
            calculated_size_usd: Some(1000.0 * 45000.0),
            long_exchange: ExchangeIdEnum::Binance, // Added
            size: Some(1000.0),                     // Base currency amount - WRAPPED IN SOME
            pnl: Some(15.5),
            unrealized_pnl_percentage: None,
            max_drawdown: None,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            holding_period_hours: None,
            trailing_stop_distance: None,
            stop_loss_price: None,
            closed_at: None,
            current_price_long: Some(45100.0),  // Example value
            current_price_short: Some(45050.0), // Example value
            updated_at: now_ms,
            short_exchange: ExchangeIdEnum::Kraken, // Corrected type
            current_price: Some(45100.0),
            max_loss_usd: None,
            exchange: ExchangeIdEnum::Binance, // This might be redundant or represent the primary exchange
            pair: pair.to_string(),
            related_positions: Vec::new(),
            last_optimization_check: None,
            hedge_position_id: None,
            position_group_id: None,
            current_state: Some("monitoring".to_string()), // Added
            recommended_action: None,
            risk_percentage_applied: None,
            optimization_score: None,
        }
    }

    // Helper to create CreatePositionData for tests
    fn create_test_create_position_data(
        size_usd: Option<f64>,
        risk_percentage: Option<f64>,
        max_size_usd: Option<f64>,
        entry_price: f64,
    ) -> CreatePositionData {
        CreatePositionData {
            exchange: ExchangeIdEnum::Binance,
            pair: "BTCUSDT".to_string(),
            side: PositionSide::Long,
            size_usd,
            entry_price_long: entry_price,
            risk_percentage,
            max_size_usd,
        }
    }

    #[tokio::test]
    async fn test_create_position_risk_percentage_sizing() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(None, Some(0.01), None, 50000.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_ok());
        let position = result.unwrap();
        assert!(position.size > Some(0.0));
        assert_eq!(position.risk_percentage_applied, Some(0.01));
    }

    #[tokio::test]
    async fn test_create_position_risk_percentage_with_max_cap_limiting() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(None, Some(0.01), Some(50.0), 50000.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_ok());
        let position = result.unwrap();
        assert!(position.size > Some(0.0));
        // Should be limited by max_size_usd, not the risk percentage
        assert_eq!(position.calculated_size_usd, Some(50.0));
    }

    #[tokio::test]
    async fn test_create_position_risk_percentage_with_max_cap_not_limiting() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(None, Some(0.01), None, 50000.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_ok());
        let position = result.unwrap();
        assert!(position.size > Some(0.0));
        assert_eq!(position.calculated_size_usd, Some(100.0)); // 10000 * 0.01
    }

    #[tokio::test]
    async fn test_create_position_fixed_usd_sizing() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_ok());
        let position = result.unwrap();
        assert!(position.size > Some(0.0));
        assert_eq!(position.calculated_size_usd, Some(1000.0));
    }

    #[tokio::test]
    async fn test_create_position_error_no_size_specified() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(None, None, None, 50000.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_position_error_zero_entry_price_risk_sizing() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(None, Some(0.01), None, 0.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_position_error_zero_entry_price_fixed_sizing() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 0.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_position_error_calculated_size_non_positive() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store);
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(None, Some(0.01), None, -10.0);
        let result = service.create_position(position_data, &account_info).await;
        assert!(result.is_err());
    }

    // Task 6: Advanced Position Management Tests

    #[tokio::test]
    async fn test_set_stop_loss_long_position() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone()); // Clone Arc for multiple uses
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);

        // Create position
        let position = service
            .create_position(position_data, &account_info)
            .await
            .unwrap();

        // Set stop-loss
        let result = service.set_stop_loss(&position.id, 48000.0).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify stop-loss was set
        let updated_position = service.get_position(&position.id).await.unwrap().unwrap();
        assert_eq!(updated_position.stop_loss_price, Some(48000.0));
        assert!(updated_position.max_loss_usd.is_some());
    }

    #[tokio::test]
    async fn test_set_stop_loss_validation_error() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);

        // Create position
        let position = service
            .create_position(position_data, &account_info)
            .await
            .unwrap();

        // Try to set invalid stop-loss (above entry price for long position)
        let result = service.set_stop_loss(&position.id, 52000.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_take_profit_long_position() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);

        // Create position and set stop-loss first
        let position = service
            .create_position(position_data, &account_info)
            .await
            .unwrap();
        service.set_stop_loss(&position.id, 48000.0).await.unwrap();

        // Set take-profit
        let result = service.set_take_profit(&position.id, 55000.0).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify take-profit and risk/reward ratio were set
        let updated_position = service.get_position(&position.id).await.unwrap().unwrap();
        assert_eq!(updated_position.take_profit_price, Some(55000.0));
        assert!(updated_position.risk_reward_ratio.is_some());

        // Risk = 50000 - 48000 = 2000, Reward = 55000 - 50000 = 5000, Ratio = 5000/2000 = 2.5
        assert!((updated_position.risk_reward_ratio.unwrap() - 2.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_enable_trailing_stop() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);

        // Create position
        let position = service
            .create_position(position_data, &account_info)
            .await
            .unwrap();

        // Enable trailing stop
        let result = service.enable_trailing_stop(&position.id, 1000.0).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify trailing stop was set
        let updated_position = service.get_position(&position.id).await.unwrap().unwrap();
        assert_eq!(updated_position.trailing_stop_distance, Some(1000.0));
    }

    #[tokio::test]
    async fn test_update_position_price_and_metrics() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone()); // Pass Arc<MockKvStore>
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);

        // Create position
        let position = service
            .create_position(position_data, &account_info) // Removed &kv_store
            .await
            .unwrap();

        // Update price
        let result = service
            .update_position_price(&position.id, 52000.0) // Removed &kv_store
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify metrics were calculated
        let updated_position = service
            .get_position(&position.id) // Removed &kv_store
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_position.current_price, Some(52000.0));
        assert!(updated_position.pnl.is_some());
        assert!(updated_position.unrealized_pnl_percentage.is_some());
        assert!(updated_position.holding_period_hours.is_some());

        // PnL should be positive for long position with price increase
        assert!(updated_position.pnl.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_check_risk_triggers_stop_loss() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);

        // Create position and set stop-loss
        let position = service
            .create_position(position_data, &account_info)
            .await
            .unwrap();
        service.set_stop_loss(&position.id, 48000.0).await.unwrap();

        // Update price below stop-loss
        service
            .update_position_price(&position.id, 47000.0)
            .await
            .unwrap();

        // Check risk triggers
        let result = service.check_risk_triggers(&position.id).await;
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, Some(PositionAction::Close));
    }

    #[tokio::test]
    async fn test_get_positions_by_exchange() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };

        // Create positions on different exchanges
        let mut position_data1 =
            create_test_create_position_data(Some(1000.0), None, None, 50000.0);
        position_data1.exchange = ExchangeIdEnum::Binance;

        let mut position_data2 =
            create_test_create_position_data(Some(1000.0), None, None, 50000.0);
        position_data2.exchange = ExchangeIdEnum::Bybit;

        service
            .create_position(position_data1, &account_info)
            .await
            .unwrap();
        service
            .create_position(position_data2, &account_info)
            .await
            .unwrap();

        // Get positions by exchange
        let binance_positions = service
            .get_positions_by_exchange(&ExchangeIdEnum::Binance)
            .await;
        assert!(binance_positions.is_ok());
        assert_eq!(binance_positions.unwrap().len(), 1);

        let bybit_positions = service
            .get_positions_by_exchange(&ExchangeIdEnum::Bybit)
            .await;
        assert!(bybit_positions.is_ok());
        assert_eq!(bybit_positions.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_link_positions() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };

        // Create two positions
        let position1 = service
            .create_position(
                create_test_create_position_data(Some(1000.0), None, None, 50000.0),
                &account_info,
            )
            .await
            .unwrap();

        let position2 = service
            .create_position(
                create_test_create_position_data(Some(1000.0), None, None, 50000.0),
                &account_info,
            )
            .await
            .unwrap();

        // Link positions
        let result = service
            .link_positions(&position1.id, vec![position2.id.clone()]) // Removed &kv_store
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify positions were linked
        let updated_position = service
            .get_position(&position1.id) // Removed &kv_store
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_position.related_positions, vec![position2.id]);
    }

    #[tokio::test]
    async fn test_analyze_position() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };

        // Create position
        let position = service
            .create_position(
                create_test_create_position_data(Some(1000.0), None, None, 50000.0),
                &account_info,
            )
            .await
            .unwrap();

        // Update price to create PnL
        service
            .update_position_price(&position.id, 52000.0)
            .await
            .unwrap();

        // Create risk management config
        let config = RiskManagementConfig {
            max_position_size_percent: 0.1,  // Added
            max_correlation_threshold: 0.8,  // Added
            stop_loss_percentage: 0.02,      // Added
            take_profit_percentage: 0.04,    // Added
            max_drawdown_percentage: 0.15,   // Added
            risk_per_trade_percentage: 0.01, // Added
            min_risk_reward_ratio: 1.5,
            max_positions_per_exchange: 5,
            max_positions_per_pair: 2,
            max_position_size_usd: 10000.0,
            max_total_exposure_usd: 50000.0,
            volatility_threshold: 0.05,
            default_stop_loss_percentage: 2.0,
            default_take_profit_percentage: 4.0,
            max_portfolio_risk_percentage: 5.0,
            max_single_position_risk_percentage: 2.0,
            enable_stop_loss: true,
            enable_take_profit: true,
            enable_trailing_stop: false,
            correlation_limit: 0.7,
        };

        // Analyze position
        let result = service
            .analyze_position(&position.id, &config) // Removed &kv_store
            .await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.is_some());

        let analysis = analysis.unwrap();
        assert_eq!(analysis.position_id, position.id);
        assert!(analysis.current_score > 0.0);
        assert!(analysis.confidence_level > 0.0);
        assert!(!analysis.reasoning.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_total_exposure() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());
        let account_info = AccountInfo {
            account_id: "test_account".to_string(),
            exchange: ExchangeIdEnum::Binance,
            balances: vec![],
            total_balance_usd: 10000.0,
            available_balance_usd: 10000.0,
            used_balance_usd: 0.0,
            last_updated: 0,
        };

        // Create multiple positions
        service
            .create_position(
                create_test_create_position_data(Some(1000.0), None, None, 50000.0),
                &account_info,
            )
            .await
            .unwrap();

        service
            .create_position(
                create_test_create_position_data(Some(2000.0), None, None, 50000.0),
                &account_info,
            )
            .await
            .unwrap();

        // Calculate total exposure
        let result = service.calculate_total_exposure().await; // Already correct, no kv_store argument
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3000.0);
    }

    #[tokio::test]
    async fn test_validate_position_risk() {
        let mock_kv_store = Arc::new(MockKvStore::new());
        let service = PositionsService::new(mock_kv_store.clone());

        let config = RiskManagementConfig {
            max_position_size_percent: 0.1,  // Added
            max_correlation_threshold: 0.8,  // Added
            stop_loss_percentage: 0.02,      // Added
            take_profit_percentage: 0.04,    // Added
            max_drawdown_percentage: 0.15,   // Added
            risk_per_trade_percentage: 0.01, // Added
            min_risk_reward_ratio: 1.5,
            max_positions_per_exchange: 5,
            max_positions_per_pair: 2,
            max_position_size_usd: 10000.0,
            max_total_exposure_usd: 50000.0,
            volatility_threshold: 0.05,
            default_stop_loss_percentage: 2.0,
            default_take_profit_percentage: 4.0,
            max_portfolio_risk_percentage: 5.0,
            max_single_position_risk_percentage: 2.0,
            enable_stop_loss: true,
            enable_take_profit: true,
            enable_trailing_stop: false,
            correlation_limit: 0.7,
        };

        // Test valid position
        let position_data = create_test_create_position_data(Some(1000.0), None, None, 50000.0);
        let result = service
            .validate_position_risk(&position_data, &config)
            .await;
        assert!(result.is_ok());

        // Test position size too large
        let large_position_data =
            create_test_create_position_data(Some(10000.0), None, None, 50000.0);
        let result = service
            .validate_position_risk(&large_position_data, &config)
            .await;
        assert!(result.is_err());
    }

    // New module for advanced position management and risk logic (Task 6)
    #[cfg(test)]
    mod advanced_position_management_tests {
        use super::*;
        use crate::services::core::trading::{KvOperationError, KvOperations, KvResult};

        #[test]
        fn test_position_sizing_with_stop_loss_and_risk_reward() {
            // Simulate a position with a stop-loss and risk-reward ratio
            let entry_price: f64 = 100.0;
            let stop_loss_price: f64 = 95.0;
            let take_profit_price: f64 = 110.0;
            let risk_per_trade: f64 = 0.02; // 2% of account
            let account_balance: f64 = 10000.0;
            let risk_amount: f64 = account_balance * risk_per_trade; // $200
            let risk_per_unit: f64 = (entry_price - stop_loss_price).abs();
            let expected_size: f64 = risk_amount / risk_per_unit;

            assert_eq!(risk_per_unit, 5.0_f64);
            assert_eq!(expected_size, 40.0_f64);

            // Risk-reward ratio
            let reward_per_unit: f64 = (take_profit_price - entry_price).abs();
            let risk_reward_ratio: f64 = reward_per_unit / risk_per_unit;
            assert_eq!(risk_reward_ratio, 2.0_f64);
        }

        #[test]
        fn test_risk_management_stop_loss_take_profit_trailing() {
            let mut position = create_test_position("risk_mgmt_test", "BTCUSDT");
            position.stop_loss_price = Some(44000.0);
            position.take_profit_price = Some(46000.0);
            position.trailing_stop_distance = Some(500.0);
            position.max_loss_usd = Some(200.0);
            position.risk_reward_ratio = Some(2.0);

            assert_eq!(position.stop_loss_price, Some(44000.0));
            assert_eq!(position.take_profit_price, Some(46000.0));
            assert_eq!(position.trailing_stop_distance, Some(500.0));
            assert_eq!(position.max_loss_usd, Some(200.0));
            assert_eq!(position.risk_reward_ratio, Some(2.0));
        }

        #[test]
        fn test_multi_exchange_position_linking() {
            let mut position1 = create_test_position("pos1", "BTCUSDT");
            let mut position2 = create_test_position("pos2", "BTCUSDT");
            position1.related_positions.push(position2.id.clone());
            position2.hedge_position_id = Some(position1.id.clone());
            position1.position_group_id = Some("group1".to_string());
            position2.position_group_id = Some("group1".to_string());

            assert_eq!(position1.related_positions, vec!["pos2".to_string()]);
            assert_eq!(position2.hedge_position_id, Some("pos1".to_string()));
            assert_eq!(position1.position_group_id, Some("group1".to_string()));
            assert_eq!(position2.position_group_id, Some("group1".to_string()));
        }

        #[test]
        fn test_position_optimization_recommendation() {
            let mut position = create_test_position("opt_test", "BTCUSDT");
            position.optimization_score = Some(0.85);
            position.recommended_action = Some("hold".to_string()); // Changed to String
            position.last_optimization_check = Some(chrono::Utc::now().timestamp_millis() as u64);

            assert_eq!(position.optimization_score, Some(0.85));
            assert_eq!(position.recommended_action, Some("hold".to_string())); // Changed to String
            assert!(position.last_optimization_check.is_some());
        }
    }

    fn create_test_risk_config() -> RiskManagementConfig {
        RiskManagementConfig {
            max_position_size_percent: 0.1,  // Added
            max_correlation_threshold: 0.8,  // Added
            stop_loss_percentage: 0.02,      // Added
            take_profit_percentage: 0.04,    // Added
            max_drawdown_percentage: 0.15,   // Added
            risk_per_trade_percentage: 0.01, // Added
            min_risk_reward_ratio: 1.5,
            max_positions_per_exchange: 5, // Added
            max_positions_per_pair: 2,     // Added
            max_position_size_usd: 10000.0,
            max_total_exposure_usd: 50000.0,
            volatility_threshold: 0.05,
            default_stop_loss_percentage: 2.0,
            default_take_profit_percentage: 4.0,
            max_portfolio_risk_percentage: 5.0,
            max_single_position_risk_percentage: 2.0,
            enable_stop_loss: true,
            enable_take_profit: true,
            enable_trailing_stop: false,
            correlation_limit: 0.7,
        }
    }
}
