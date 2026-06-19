use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::ledger::{AccountType, JournalEntry, Ledger, LedgerError};

// ── Typestate markers ──────────────────────────────────────────────────────────

/// Auction is accepting bids.
pub struct OpenForBids;
/// The highest bid has been accepted and the auction is waiting to be closed.
pub struct BidAccepted;
/// Auction has been settled (winner paid, seller received funds, or no bids).
pub struct AuctionClosed;
/// Auction was cancelled before closing.
pub struct AuctionCancelled;

// ── Core types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub bidder_id: u64,
    pub amount: u32,
    pub placed_at: DateTime<Utc>,
}

/// An auction parameterised by its current lifecycle state `S`.
///
/// Only valid operations are representable at compile time.
///
/// # Examples
///
/// ```
/// use nexus_economy::auction::{Auction, OpenForBids};
///
/// let auction = Auction::new(
///     1,
///     101,
///     "Gundam Armor Fragment".to_string(),
///     150,
///     Some(200),
///     24
/// );
/// assert_eq!(auction.item_name, "Gundam Armor Fragment");
/// ```
pub struct Auction<S> {
    pub id: u64,
    pub seller_id: u64,
    pub item_name: String,
    pub starting_price: u32,
    pub reserve_price: Option<u32>,
    pub current_bid: Option<Bid>,
    pub ends_at: DateTime<Utc>,
    _state: PhantomData<S>,
}

/// Errors arising from invalid Auction construction.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AuctionBuildError {
    /// Item name cannot be empty.
    #[error("item_name is required and cannot be empty")]
    EmptyItemName,

    /// Starting price must be > 0.
    #[error("starting_price must be positive")]
    ZeroStartingPrice,

    /// Reserve price cannot be lower than the starting price.
    #[error("reserve_price ({reserve}) cannot be less than starting_price ({starting})")]
    ReserveBelowStarting {
        /// The starting price.
        starting: u32,
        /// The reserve price.
        reserve: u32,
    },

    /// The duration must be positive.
    #[error("duration_hours must be positive")]
    InvalidDuration,
}

/// A builder for [`Auction`] to simplify initialization and validate starting conditions.
///
/// # Examples
///
/// ```
/// use nexus_economy::auction::{AuctionBuilder, OpenForBids};
///
/// let auction = AuctionBuilder::new()
///     .id(1)
///     .seller_id(101)
///     .item_name("Luna Titanium".to_string())
///     .starting_price(100)
///     .duration_hours(12)
///     .build()
///     .unwrap();
///
/// assert_eq!(auction.starting_price, 100);
/// ```
#[derive(Debug, Clone)]
pub struct AuctionBuilder {
    id: Option<u64>,
    seller_id: Option<u64>,
    item_name: Option<String>,
    starting_price: Option<u32>,
    reserve_price: Option<u32>,
    duration_hours: Option<i64>,
}

impl AuctionBuilder {
    /// Create a new builder with default parameters.
    pub fn new() -> Self {
        Self {
            id: None,
            seller_id: None,
            item_name: None,
            starting_price: None,
            reserve_price: None,
            duration_hours: Some(24),
        }
    }

    /// Set the auction ID.
    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the seller player ID.
    pub fn seller_id(mut self, seller_id: u64) -> Self {
        self.seller_id = Some(seller_id);
        self
    }

    /// Set the name of the item being auctioned.
    pub fn item_name(mut self, name: String) -> Self {
        self.item_name = Some(name);
        self
    }

    /// Set the starting price in gold.
    pub fn starting_price(mut self, price: u32) -> Self {
        self.starting_price = Some(price);
        self
    }

    /// Set the reserve price in gold (optional).
    pub fn reserve_price(mut self, reserve: u32) -> Self {
        self.reserve_price = Some(reserve);
        self
    }

    /// Set the duration of the auction in hours. Defaults to 24 hours.
    pub fn duration_hours(mut self, hours: i64) -> Self {
        self.duration_hours = Some(hours);
        self
    }

    /// Validate the parameters and build an [`Auction`] in [`OpenForBids`] state.
    pub fn build(self) -> Result<Auction<OpenForBids>, AuctionBuildError> {
        let id = self.id.unwrap_or(0);
        let seller_id = self.seller_id.unwrap_or(0);
        let item_name = self.item_name.ok_or(AuctionBuildError::EmptyItemName)?;
        if item_name.trim().is_empty() {
            return Err(AuctionBuildError::EmptyItemName);
        }
        let starting_price = self
            .starting_price
            .ok_or(AuctionBuildError::ZeroStartingPrice)?;
        if starting_price == 0 {
            return Err(AuctionBuildError::ZeroStartingPrice);
        }
        if let Some(reserve) = self.reserve_price {
            if reserve < starting_price {
                return Err(AuctionBuildError::ReserveBelowStarting {
                    starting: starting_price,
                    reserve,
                });
            }
        }
        let duration = self.duration_hours.unwrap_or(24);
        if duration <= 0 {
            return Err(AuctionBuildError::InvalidDuration);
        }

        Ok(Auction {
            id,
            seller_id,
            item_name,
            starting_price,
            reserve_price: self.reserve_price,
            current_bid: None,
            ends_at: Utc::now() + chrono::Duration::hours(duration),
            _state: PhantomData,
        })
    }
}

impl Default for AuctionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the dynamic runtime state of an auction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuctionState {
    OpenForBids,
    BidAccepted,
    AuctionClosed,
    AuctionCancelled,
}

/// Errors returned when an auction transition is invalid at runtime.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("Illegal auction state transition: cannot transition from {current:?} to {target:?}. Reason: {reason}")]
pub struct AuctionTransitionError {
    pub current: AuctionState,
    pub target: AuctionState,
    pub reason: String,
}

// ── Escrow helpers ─────────────────────────────────────────────────────────────
//
// Player wallet 0 is used as the auction-house escrow account.  Because the
// ledger's `transfer` method checks the *sender's* balance, we need raw
// `record` calls for escrow movements where the escrow account is the sender.

const ESCROW_ID: u64 = 0;

/// Move `amount` gold from a player into escrow (locks funds for a bid).
fn lock_to_escrow(
    ledger: &mut Ledger,
    bidder_id: u64,
    amount: u32,
    description: &str,
) -> Result<u64, LedgerError> {
    // Standard player→escrow transfer.
    ledger.transfer(bidder_id, ESCROW_ID, amount, description)
}

/// Release `amount` gold from escrow to a player (refund or payout).
fn release_from_escrow(
    ledger: &mut Ledger,
    recipient_id: u64,
    amount: u32,
    description: &str,
) -> Result<u64, LedgerError> {
    // Escrow → player: we use a raw record so the balance check on wallet 0
    // doesn't block us.  The escrow account will have a matching negative
    // balance from the original lock, so total_balance stays zero.
    let tx_id = ledger.next_tx_id();
    let entry_id = ledger.next_entry_id();
    ledger.record(JournalEntry {
        id: entry_id,
        timestamp: Utc::now(),
        description: description.to_string(),
        debits: vec![(AccountType::PlayerWallet(ESCROW_ID), amount)],
        credits: vec![(AccountType::PlayerWallet(recipient_id), amount)],
        transaction_id: tx_id,
    })
}

// ── Auction<OpenForBids> ───────────────────────────────────────────────────────

impl Auction<OpenForBids> {
    pub fn new(
        id: u64,
        seller_id: u64,
        item_name: String,
        starting_price: u32,
        reserve: Option<u32>,
        duration_hours: i64,
    ) -> Self {
        Self {
            id,
            seller_id,
            item_name,
            starting_price,
            reserve_price: reserve,
            current_bid: None,
            ends_at: Utc::now() + chrono::Duration::hours(duration_hours),
            _state: PhantomData,
        }
    }

    /// Place a bid.  The new bid must be at least 5 % above the current bid
    /// (minimum 1 gold increment).  The previous bidder's funds are refunded
    /// from escrow; the new bidder's funds are locked into escrow.
    pub fn place_bid(
        &mut self,
        bidder_id: u64,
        amount: u32,
        ledger: &mut Ledger,
    ) -> Result<(), AuctionError> {
        // Minimum bid = current bid + 5 % (at least +1).
        let min_bid = self
            .current_bid
            .as_ref()
            .map(|b| b.amount + (b.amount / 20).max(1))
            .unwrap_or(self.starting_price);

        if amount < min_bid {
            return Err(AuctionError::BidTooLow {
                minimum: min_bid,
                offered: amount,
            });
        }

        if bidder_id == self.seller_id {
            return Err(AuctionError::SellerCannotBid);
        }

        // Refund the previous bidder from escrow.
        if let Some(prev) = &self.current_bid {
            release_from_escrow(ledger, prev.bidder_id, prev.amount, "auction outbid refund")
                .map_err(|e| AuctionError::LedgerError(e.to_string()))?;
        }

        // Lock new bid in escrow.
        lock_to_escrow(
            ledger,
            bidder_id,
            amount,
            &format!("auction bid escrow: {}", self.item_name),
        )
        .map_err(|e| AuctionError::LedgerError(e.to_string()))?;

        self.current_bid = Some(Bid {
            bidder_id,
            amount,
            placed_at: Utc::now(),
        });

        Ok(())
    }

    /// Close the auction and settle funds.
    ///
    /// * If there is a bid that meets the reserve, the seller receives the bid
    ///   amount minus a 5 % house fee (fee goes to `AuctionHouse`).
    /// * If the reserve is not met, the highest bidder is refunded.
    /// * If there were no bids, the auction closes with no fund movements.
    pub fn close(self, ledger: &mut Ledger) -> Result<Auction<AuctionClosed>, AuctionError> {
        if let Some(ref bid) = self.current_bid {
            let reserve_met = self.reserve_price.map(|r| bid.amount >= r).unwrap_or(true);

            if reserve_met {
                let fee = bid.amount / 20;
                let seller_gets = bid.amount - fee;

                // Escrow → seller
                release_from_escrow(ledger, self.seller_id, seller_gets, "auction sale proceeds")
                    .map_err(|e| AuctionError::LedgerError(e.to_string()))?;

                // Escrow → AuctionHouse (fee)
                if fee > 0 {
                    let tx_id = ledger.next_tx_id();
                    let entry_id = ledger.next_entry_id();
                    ledger
                        .record(JournalEntry {
                            id: entry_id,
                            timestamp: Utc::now(),
                            description: "auction house fee".to_string(),
                            debits: vec![(AccountType::PlayerWallet(ESCROW_ID), fee)],
                            credits: vec![(AccountType::AuctionHouse, fee)],
                            transaction_id: tx_id,
                        })
                        .map_err(|e| AuctionError::LedgerError(e.to_string()))?;
                }
            } else {
                // Reserve not met — refund the bidder.
                release_from_escrow(
                    ledger,
                    bid.bidder_id,
                    bid.amount,
                    "auction reserve not met refund",
                )
                .map_err(|e| AuctionError::LedgerError(e.to_string()))?;
            }
        }

        Ok(Auction {
            id: self.id,
            seller_id: self.seller_id,
            item_name: self.item_name,
            starting_price: self.starting_price,
            reserve_price: self.reserve_price,
            current_bid: self.current_bid,
            ends_at: self.ends_at,
            _state: PhantomData,
        })
    }

    /// Cancel the auction.  If there is a current bid, the bidder is refunded.
    pub fn cancel(self, ledger: &mut Ledger) -> Result<Auction<AuctionCancelled>, AuctionError> {
        if let Some(ref bid) = self.current_bid {
            release_from_escrow(
                ledger,
                bid.bidder_id,
                bid.amount,
                "auction cancelled refund",
            )
            .map_err(|e| AuctionError::LedgerError(e.to_string()))?;
        }

        Ok(Auction {
            id: self.id,
            seller_id: self.seller_id,
            item_name: self.item_name,
            starting_price: self.starting_price,
            reserve_price: self.reserve_price,
            current_bid: self.current_bid,
            ends_at: self.ends_at,
            _state: PhantomData,
        })
    }
}

// ── Error type ─────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum AuctionError {
    #[error("bid too low: minimum {minimum}, offered {offered}")]
    BidTooLow { minimum: u32, offered: u32 },

    #[error("seller cannot bid on own auction")]
    SellerCannotBid,

    #[error("auction is not open")]
    NotOpen,

    #[error("ledger error: {0}")]
    LedgerError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Ledger;

    fn funded_ledger(player_id: u64, amount: u32) -> Ledger {
        let mut ledger = Ledger::new();
        ledger.award_gold(player_id, amount, "test setup").unwrap();
        ledger
    }

    fn open_auction(seller_id: u64, start: u32, reserve: Option<u32>) -> Auction<OpenForBids> {
        AuctionBuilder::new()
            .id(1)
            .seller_id(seller_id)
            .item_name("Luna Titanium Armor".into())
            .starting_price(start)
            .reserve_price(reserve.unwrap_or(start))
            .duration_hours(24)
            .build()
            .unwrap()
    }

    // ── AuctionBuilder validation ─────────────────────────────────────────────

    #[test]
    fn builder_requires_item_name() {
        let result = AuctionBuilder::new()
            .seller_id(1)
            .starting_price(100)
            .build();
        assert!(matches!(result, Err(AuctionBuildError::EmptyItemName)));
    }

    #[test]
    fn builder_requires_positive_starting_price() {
        let result = AuctionBuilder::new()
            .item_name("x".into())
            .starting_price(0)
            .build();
        assert!(matches!(result, Err(AuctionBuildError::ZeroStartingPrice)));
    }

    #[test]
    fn builder_rejects_reserve_below_starting() {
        let result = AuctionBuilder::new()
            .item_name("x".into())
            .starting_price(200)
            .reserve_price(100)
            .build();
        assert!(matches!(
            result,
            Err(AuctionBuildError::ReserveBelowStarting { .. })
        ));
    }

    #[test]
    fn builder_valid_auction_has_no_current_bid() {
        let a = AuctionBuilder::new()
            .id(1)
            .seller_id(10)
            .item_name("Gundam".into())
            .starting_price(50)
            .duration_hours(24)
            .build()
            .unwrap();
        assert!(a.current_bid.is_none());
        assert_eq!(a.starting_price, 50);
    }

    // ── place_bid ─────────────────────────────────────────────────────────────

    #[test]
    fn seller_cannot_bid_on_own_auction() {
        let mut a = open_auction(1, 100, None);
        let mut ledger = funded_ledger(1, 500);
        let err = a.place_bid(1, 100, &mut ledger).unwrap_err();
        assert!(matches!(err, AuctionError::SellerCannotBid));
    }

    #[test]
    fn bid_below_starting_price_rejected() {
        let mut a = open_auction(1, 100, None);
        let mut ledger = funded_ledger(2, 500);
        let err = a.place_bid(2, 50, &mut ledger).unwrap_err();
        assert!(matches!(err, AuctionError::BidTooLow { minimum: 100, .. }));
    }

    #[test]
    fn valid_first_bid_locks_funds_in_escrow() {
        let mut a = open_auction(1, 100, None);
        let mut ledger = funded_ledger(2, 500);
        a.place_bid(2, 100, &mut ledger).unwrap();
        // Bidder's wallet reduced by bid amount
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 400);
        // Escrow account credited
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(ESCROW_ID)), 100);
    }

    #[test]
    fn outbid_refunds_previous_bidder() {
        let mut a = open_auction(1, 100, None);
        let mut ledger = funded_ledger(2, 500);
        ledger.award_gold(3, 500, "test").unwrap();

        a.place_bid(2, 100, &mut ledger).unwrap();
        a.place_bid(3, 106, &mut ledger).unwrap(); // 5% above 100 = min 105

        // Player 2 refunded to original balance
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 500);
        // Player 3's bid in escrow
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(ESCROW_ID)), 106);
    }

    // ── close ─────────────────────────────────────────────────────────────────

    #[test]
    fn close_with_no_bids_leaves_ledger_unchanged() {
        let a = open_auction(1, 100, None);
        let mut ledger = Ledger::new();
        a.close(&mut ledger).unwrap();
        assert_eq!(ledger.total_balance(), 0);
    }

    #[test]
    fn close_with_winning_bid_pays_seller_minus_fee() {
        let mut a = open_auction(1, 100, Some(100));
        let mut ledger = funded_ledger(2, 500);
        a.place_bid(2, 100, &mut ledger).unwrap();
        a.close(&mut ledger).unwrap();

        // 5% fee = 5; seller gets 95
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 95);
        assert_eq!(ledger.balance_of(AccountType::AuctionHouse), 5);
        assert_eq!(ledger.total_balance(), 0);
    }

    #[test]
    fn close_when_reserve_not_met_refunds_bidder() {
        let mut a = open_auction(1, 100, Some(500)); // reserve = 500, bid = 100
        let mut ledger = funded_ledger(2, 500);
        a.place_bid(2, 100, &mut ledger).unwrap();
        a.close(&mut ledger).unwrap();

        // Bidder refunded
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 500);
        assert_eq!(ledger.total_balance(), 0);
    }

    // ── cancel ────────────────────────────────────────────────────────────────

    #[test]
    fn cancel_with_active_bid_refunds_bidder() {
        let mut a = open_auction(1, 100, None);
        let mut ledger = funded_ledger(2, 500);
        a.place_bid(2, 100, &mut ledger).unwrap();
        a.cancel(&mut ledger).unwrap();

        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 500);
        assert_eq!(ledger.total_balance(), 0);
    }

    #[test]
    fn cancel_with_no_bids_is_noop_on_ledger() {
        let a = open_auction(1, 100, None);
        let mut ledger = Ledger::new();
        a.cancel(&mut ledger).unwrap();
        assert_eq!(ledger.total_balance(), 0);
    }
}
