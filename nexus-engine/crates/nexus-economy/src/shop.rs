use serde::{Deserialize, Serialize};

use crate::ledger::{AccountType, JournalEntry, Ledger, LedgerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub item_name: String,
    pub rarity: String,
    pub base_price: u32,
    /// `None` means infinite stock.
    pub stock: Option<u32>,
    pub required_bloodline: u32,
}

/// An NPC shop with markup pricing and optional bloodline requirements.
pub struct Shop {
    pub name: String,
    /// Multiplier applied to base_price (e.g. 1.2 = 20 % above base).
    pub markup: f32,
    pub items: Vec<ShopItem>,
}

impl Shop {
    /// Create a new shop with the given name and markup.
    pub fn new(name: String, markup: f32) -> Self {
        Self {
            name,
            markup,
            items: Vec::new(),
        }
    }

    /// Add an item to the shop's inventory.
    pub fn add_item(&mut self, item: ShopItem) {
        self.items.push(item);
    }

    /// Effective sell price of an item (base_price * markup, rounded up).
    pub fn price_of(&self, item_name: &str) -> Option<u32> {
        self.items
            .iter()
            .find(|i| i.item_name == item_name)
            .map(|i| (i.base_price as f32 * self.markup).ceil() as u32)
    }

    /// Purchase an item from the shop.
    ///
    /// Checks bloodline requirement, stock, and deducts gold from the buyer via
    /// the ledger (buyer wallet is debited; ShopRevenue account is credited).
    /// Returns the purchased item on success.
    pub fn buy(
        &mut self,
        item_name: &str,
        buyer_id: u64,
        bloodline: u32,
        ledger: &mut Ledger,
    ) -> Result<ShopItem, ShopError> {
        let item_idx = self
            .items
            .iter()
            .position(|i| i.item_name == item_name)
            .ok_or_else(|| ShopError::ItemNotFound(item_name.to_string()))?;

        // Validate bloodline and stock before touching the ledger.
        {
            let item = &self.items[item_idx];
            if bloodline < item.required_bloodline {
                return Err(ShopError::BloodlineRequired {
                    need: item.required_bloodline,
                    have: bloodline,
                });
            }
            if let Some(stock) = item.stock {
                if stock == 0 {
                    return Err(ShopError::OutOfStock(item_name.to_string()));
                }
            }
        }

        let price = (self.items[item_idx].base_price as f32 * self.markup).ceil() as u32;

        // Check buyer funds.
        if ledger.balance_of(AccountType::PlayerWallet(buyer_id)) < price as i64 {
            return Err(ShopError::PaymentFailed(format!(
                "insufficient funds: player {buyer_id} needs {price} gold"
            )));
        }

        // Record the purchase: debit buyer wallet, credit ShopRevenue.
        let tx_id = ledger.next_tx_id();
        let entry_id = ledger.next_entry_id();
        ledger
            .record(JournalEntry {
                id: entry_id,
                timestamp: chrono::Utc::now(),
                description: format!("shop purchase: {}", item_name),
                debits: vec![(AccountType::PlayerWallet(buyer_id), price)],
                credits: vec![(AccountType::ShopRevenue, price)],
                transaction_id: tx_id,
            })
            .map_err(|e| ShopError::PaymentFailed(e.to_string()))?;

        // Reduce stock.
        if let Some(stock) = &mut self.items[item_idx].stock {
            *stock -= 1;
        }

        Ok(self.items[item_idx].clone())
    }

    /// Sell an item to the shop.  The shop pays 50 % of `base_value` (gold
    /// is awarded from the system source account).
    pub fn sell_item(
        &mut self,
        item_name: &str,
        seller_id: u64,
        base_value: u32,
        ledger: &mut Ledger,
    ) -> Result<u32, ShopError> {
        let sell_price = base_value / 2;
        ledger
            .award_gold(seller_id, sell_price, &format!("sold: {}", item_name))
            .map_err(|e| ShopError::PaymentFailed(e.to_string()))?;
        Ok(sell_price)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShopError {
    #[error("item not found: {0}")]
    ItemNotFound(String),

    #[error("bloodline {need} required, player has {have}")]
    BloodlineRequired { need: u32, have: u32 },

    #[error("out of stock: {0}")]
    OutOfStock(String),

    #[error("payment failed: {0}")]
    PaymentFailed(String),
}

impl From<LedgerError> for ShopError {
    fn from(e: LedgerError) -> Self {
        ShopError::PaymentFailed(e.to_string())
    }
}
