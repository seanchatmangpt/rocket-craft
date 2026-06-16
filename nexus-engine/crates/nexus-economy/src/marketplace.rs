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
    pub fn cancel(
        &mut self,
        listing_id: u64,
        seller_id: u64,
    ) -> Result<(), MarketplaceError> {
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
