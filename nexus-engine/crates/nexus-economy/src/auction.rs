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
            release_from_escrow(
                ledger,
                prev.bidder_id,
                prev.amount,
                "auction outbid refund",
            )
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
            let reserve_met = self
                .reserve_price
                .map(|r| bid.amount >= r)
                .unwrap_or(true);

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
            release_from_escrow(ledger, bid.bidder_id, bid.amount, "auction cancelled refund")
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
