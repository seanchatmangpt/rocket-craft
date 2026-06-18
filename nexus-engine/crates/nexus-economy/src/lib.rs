//! # Nexus Economy Subsystem
//!
//! The `nexus-economy` crate manages the in-game financial transaction systems, auction houses,
//! and merchant shops. It features a robust, double-entry accounting ledger designed to guarantee
//! transaction integrity and maintain strict balance invariants across the game economy.
//!
//! ## Key Modules
//! - **`ledger`**: A double-entry accounting journal tracking debits, credits, and account types (e.g., player wallets, shop revenue, escrow accounts).
//! - **`shop`**: Implements vendor inventory, purchasing flow, and item price calculations.
//! - **`auction`**: Handles bidding, buyouts, time-limits, and escrow management for player-to-player trade.
//! - **`marketplace`**: Orchestrates global listings, query matching, and marketplace state updates.
//!
//! ## System Integration
//! The economy subsystem connects directly to player profiles in `nexus-session` and coordinates with the
//! `nexus-ecs` crate when player entities carry gold components. Transaction failures are handled via structured
//! error types to prevent state corruption.

pub mod auction;
pub mod ledger;
pub mod marketplace;
pub mod shop;

pub use auction::{Auction, AuctionBuilder, AuctionBuildError, AuctionState, AuctionTransitionError};
pub use ledger::{Ledger, JournalEntry, AccountType};
pub use marketplace::Marketplace;
pub use shop::{Shop, ShopItem, ShopError};
