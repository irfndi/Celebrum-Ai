// src/services/interfaces/telegram/core/mod.rs

//! Core Telegram functionality
//!
//! This module contains the core Telegram bot functionality including:
//! - Bot client for API communication
//! - Message handling and processing
//! - Webhook processing
//! - Command routing and handling
//! - Basic bot operations

pub mod bot_client;
pub mod command_router;
pub mod message_handler;
pub mod webhook_handler;
