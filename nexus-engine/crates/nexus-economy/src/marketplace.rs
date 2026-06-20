use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ledger::{Ledger, LedgerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: u64,
    pub seller_id: u64,
    pub item_name: String,
    pub item_rarity: String,
    pub price_gold: u32,
    pub listed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Fixed-price item marketplace.
pub struct Marketplace {
    listings: Vec<Listing>,
    next_listing_id: u64,
}

impl Default for Marketplace {
    fn default() -> Self {
        Self::new()
    }
}

impl Marketplace {
    pub fn new() -> Self {
        Self {
            listings: Vec::new(),
            next_listing_id: 0,
        }
    }

    /// Create a new listing.  Price must be > 0.
    pub fn list_item(
        &mut self,
        seller_id: u64,
        item_name: String,
        item_rarity: String,
        price: u32,
    ) -> Result<u64, MarketplaceError> {
        if price == 0 {
            return Err(MarketplaceError::InvalidPrice(0));
        }

        let id = self.next_listing_id;
        self.next_listing_id += 1;

        self.listings.push(Listing {
            id,
            seller_id,
            item_name,
            item_rarity,
            price_gold: price,
            listed_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
            is_active: true,
        });

        Ok(id)
    }

    /// Purchase a listing.  Gold is transferred atomically via the ledger.
    pub fn buy(
        &mut self,
        listing_id: u64,
        buyer_id: u64,
        ledger: &mut Ledger,
    ) -> Result<Listing, MarketplaceError> {
        let listing = self
            .listings
            .iter_mut()
            .find(|l| l.id == listing_id && l.is_active)
            .ok_or(MarketplaceError::ListingNotFound(listing_id))?;

        if listing.seller_id == buyer_id {
            return Err(MarketplaceError::CannotBuyOwnListing);
        }

        // Atomic: transfer gold then mark sold.
        ledger
            .transfer(
                buyer_id,
                listing.seller_id,
                listing.price_gold,
                &format!("marketplace purchase: {}", listing.item_name),
            )
            .map_err(|e| MarketplaceError::PaymentFailed(e.to_string()))?;

        listing.is_active = false;
        Ok(listing.clone())
    }

    /// Iterator over all currently active listings.
    pub fn active_listings(&self) -> impl Iterator<Item = &Listing> {
        self.listings.iter().filter(|l| l.is_active)
    }

    /// Iterator over active listings created by a specific seller.
    pub fn listings_by_seller(&self, seller_id: u64) -> impl Iterator<Item = &Listing> {
        self.listings
            .iter()
            .filter(move |l| l.seller_id == seller_id && l.is_active)
    }

    /// Cancel a listing (seller only).
    pub fn cancel(&mut self, listing_id: u64, seller_id: u64) -> Result<(), MarketplaceError> {
        let listing = self
            .listings
            .iter_mut()
            .find(|l| l.id == listing_id && l.seller_id == seller_id && l.is_active)
            .ok_or(MarketplaceError::ListingNotFound(listing_id))?;

        listing.is_active = false;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("listing {0} not found or expired")]
    ListingNotFound(u64),

    #[error("price must be > 0, got {0}")]
    InvalidPrice(u32),

    #[error("cannot buy your own listing")]
    CannotBuyOwnListing,

    #[error("payment failed: {0}")]
    PaymentFailed(String),
}

impl From<LedgerError> for MarketplaceError {
    fn from(e: LedgerError) -> Self {
        MarketplaceError::PaymentFailed(e.to_string())
    }
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

    // ── list_item ─────────────────────────────────────────────────────────────

    #[test]
    fn list_item_returns_sequential_ids() {
        let mut mp = Marketplace::new();
        let id0 = mp
            .list_item(1, "Beam Saber".into(), "Rare".into(), 100)
            .unwrap();
        let id1 = mp
            .list_item(1, "Shield".into(), "Common".into(), 50)
            .unwrap();
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn zero_price_listing_rejected() {
        let mut mp = Marketplace::new();
        let err = mp
            .list_item(1, "Free Item".into(), "Common".into(), 0)
            .unwrap_err();
        assert!(matches!(err, MarketplaceError::InvalidPrice(0)));
    }

    #[test]
    fn listed_item_appears_in_active_listings() {
        let mut mp = Marketplace::new();
        mp.list_item(1, "Newtype Crystal".into(), "Legendary".into(), 999)
            .unwrap();
        assert_eq!(mp.active_listings().count(), 1);
    }

    // ── buy ───────────────────────────────────────────────────────────────────

    #[test]
    fn buy_transfers_gold_from_buyer_to_seller() {
        let mut mp = Marketplace::new();
        let listing_id = mp
            .list_item(1, "Beam Rifle".into(), "SR".into(), 200)
            .unwrap();

        let mut ledger = funded_ledger(2, 500);
        ledger.award_gold(1, 0, "seller placeholder").ok(); // ensure account exists

        mp.buy(listing_id, 2, &mut ledger).unwrap();

        assert_eq!(
            ledger.balance_of(crate::ledger::AccountType::PlayerWallet(2)),
            300
        );
        assert_eq!(
            ledger.balance_of(crate::ledger::AccountType::PlayerWallet(1)),
            200
        );
    }

    #[test]
    fn buy_marks_listing_as_inactive() {
        let mut mp = Marketplace::new();
        let id = mp
            .list_item(1, "Z Gundam".into(), "SSR".into(), 100)
            .unwrap();
        let mut ledger = funded_ledger(2, 500);
        mp.buy(id, 2, &mut ledger).unwrap();
        assert_eq!(mp.active_listings().count(), 0);
    }

    #[test]
    fn cannot_buy_own_listing() {
        let mut mp = Marketplace::new();
        let id = mp
            .list_item(1, "Self-Sale".into(), "Common".into(), 50)
            .unwrap();
        let mut ledger = funded_ledger(1, 500);
        let err = mp.buy(id, 1, &mut ledger).unwrap_err();
        assert!(matches!(err, MarketplaceError::CannotBuyOwnListing));
    }

    #[test]
    fn buy_nonexistent_listing_returns_not_found() {
        let mut mp = Marketplace::new();
        let mut ledger = funded_ledger(2, 500);
        let err = mp.buy(99, 2, &mut ledger).unwrap_err();
        assert!(matches!(err, MarketplaceError::ListingNotFound(99)));
    }

    #[test]
    fn buy_with_insufficient_funds_fails() {
        let mut mp = Marketplace::new();
        let id = mp
            .list_item(1, "Expensive".into(), "SSR".into(), 1000)
            .unwrap();
        let mut ledger = funded_ledger(2, 50); // only 50 gold
        let err = mp.buy(id, 2, &mut ledger).unwrap_err();
        assert!(matches!(err, MarketplaceError::PaymentFailed(_)));
        // listing stays active after failed purchase
        assert_eq!(mp.active_listings().count(), 1);
    }

    #[test]
    fn cannot_buy_same_listing_twice() {
        let mut mp = Marketplace::new();
        let id = mp
            .list_item(1, "Unique".into(), "Legendary".into(), 100)
            .unwrap();
        let mut ledger = funded_ledger(2, 500);
        mp.buy(id, 2, &mut ledger).unwrap();
        let err = mp.buy(id, 2, &mut ledger).unwrap_err();
        assert!(matches!(err, MarketplaceError::ListingNotFound(0)));
    }

    // ── cancel_listing ────────────────────────────────────────────────────────

    #[test]
    fn seller_can_cancel_own_listing() {
        let mut mp = Marketplace::new();
        let id = mp
            .list_item(1, "Cancelled".into(), "Common".into(), 100)
            .unwrap();
        mp.cancel(id, 1).unwrap();
        assert_eq!(mp.active_listings().count(), 0);
    }

    #[test]
    fn cancel_listing_by_wrong_seller_fails() {
        let mut mp = Marketplace::new();
        let id = mp
            .list_item(1, "Mine".into(), "Common".into(), 100)
            .unwrap();
        let err = mp.cancel(id, 2).unwrap_err(); // player 2 is not the seller
        assert!(matches!(err, MarketplaceError::ListingNotFound(_)));
    }
}
