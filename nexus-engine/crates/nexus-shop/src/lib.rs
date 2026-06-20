//! # Nexus Shop, Gacha, and Progression Monetization Systems
//!
//! The `nexus-shop` crate manages in-game progression monetization loops, prize gacha systems,
//! battle pass mechanics, and secure cryptographic action-receipt bridging.
//!
//! ## Key Modules
//! - **`gacha`**: Simulates drop pools, banners, probability rates (SSR, SR, R), pull sessions, and soft/hard pity counters.
//! - **`battle_pass`**: Defines battle pass tier structures, experience milestones, rewards unlocking, and premium activations.
//! - **`ar_bridge`**: The Action Receipt (AR) Bridge, providing cryptographic verification for transactions and receipts mapping.
//!
//! ## System Integration
//! This crate integrates with the currency systems in `nexus-economy` (validating balances before pulls or unlocks)
//! and records receipt logs into `nexus-integration` for visual compliance audits.

pub mod ar_bridge;
pub mod battle_pass;
pub mod gacha;
