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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{AccountType, Ledger};

    fn funded_ledger(player_id: u64, gold: u32) -> Ledger {
        let mut ledger = Ledger::new();
        ledger.award_gold(player_id, gold, "test setup").unwrap();
        ledger
    }

    fn basic_item(name: &str, price: u32, stock: Option<u32>) -> ShopItem {
        ShopItem {
            item_name: name.to_string(),
            rarity: "SR".to_string(),
            base_price: price,
            stock,
            required_bloodline: 0,
        }
    }

    fn gated_item(name: &str, price: u32, bloodline: u32) -> ShopItem {
        ShopItem {
            item_name: name.to_string(),
            rarity: "SSR".to_string(),
            base_price: price,
            stock: None,
            required_bloodline: bloodline,
        }
    }

    // ── Shop::new / add_item / price_of ──────────────────────────────────────

    #[test]
    fn price_of_applies_markup() {
        let mut shop = Shop::new("Test".to_string(), 1.5);
        shop.add_item(basic_item("Sword", 100, None));
        // 100 * 1.5 = 150
        assert_eq!(shop.price_of("Sword"), Some(150));
    }

    #[test]
    fn price_of_rounds_fractional_markup_up() {
        let mut shop = Shop::new("Test".to_string(), 1.1);
        shop.add_item(basic_item("Potion", 3, None)); // 3 * 1.1 = 3.3 → ceil → 4
        assert_eq!(shop.price_of("Potion"), Some(4));
    }

    #[test]
    fn price_of_unknown_item_returns_none() {
        let shop = Shop::new("Test".to_string(), 1.0);
        assert!(shop.price_of("Ghost").is_none());
    }

    // ── Shop::buy — happy path ────────────────────────────────────────────────

    #[test]
    fn buy_deducts_gold_and_returns_item() {
        let mut shop = Shop::new("Anaheim".to_string(), 1.0);
        shop.add_item(basic_item("Beam Saber", 200, None));
        let mut ledger = funded_ledger(1, 500);

        let item = shop.buy("Beam Saber", 1, 0, &mut ledger).unwrap();

        assert_eq!(item.item_name, "Beam Saber");
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 300);
        assert_eq!(ledger.balance_of(AccountType::ShopRevenue), 200);
        assert_eq!(ledger.total_balance(), 0, "double-entry invariant");
    }

    #[test]
    fn buy_reduces_finite_stock_by_one() {
        let mut shop = Shop::new("Limited".to_string(), 1.0);
        shop.add_item(basic_item("Frame", 100, Some(3)));
        let mut ledger = funded_ledger(1, 1000);

        shop.buy("Frame", 1, 0, &mut ledger).unwrap();
        shop.buy("Frame", 1, 0, &mut ledger).unwrap();

        assert_eq!(shop.items[0].stock, Some(1));
    }

    #[test]
    fn buy_infinite_stock_item_does_not_decrement() {
        let mut shop = Shop::new("Infinite".to_string(), 1.0);
        shop.add_item(basic_item("Ammo", 10, None));
        let mut ledger = funded_ledger(1, 1000);

        for _ in 0..5 {
            shop.buy("Ammo", 1, 0, &mut ledger).unwrap();
        }
        assert_eq!(shop.items[0].stock, None);
    }

    // ── Shop::buy — error paths ───────────────────────────────────────────────

    #[test]
    fn buy_unknown_item_returns_not_found() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        let mut ledger = funded_ledger(1, 500);
        let err = shop.buy("Ghost", 1, 0, &mut ledger).unwrap_err();
        assert!(matches!(err, ShopError::ItemNotFound(_)));
    }

    #[test]
    fn buy_with_insufficient_funds_returns_payment_failed() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        shop.add_item(basic_item("Gundam", 1000, None));
        let mut ledger = funded_ledger(1, 50);
        let err = shop.buy("Gundam", 1, 0, &mut ledger).unwrap_err();
        assert!(matches!(err, ShopError::PaymentFailed(_)));
        // Ledger unchanged
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 50);
    }

    #[test]
    fn buy_out_of_stock_returns_error() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        shop.add_item(basic_item("Rare Frame", 100, Some(0)));
        let mut ledger = funded_ledger(1, 500);
        let err = shop.buy("Rare Frame", 1, 0, &mut ledger).unwrap_err();
        assert!(matches!(err, ShopError::OutOfStock(_)));
    }

    #[test]
    fn buy_insufficient_bloodline_returns_error() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        shop.add_item(gated_item("NT-D Unicorn", 500, 5));
        let mut ledger = funded_ledger(1, 1000);
        let err = shop.buy("NT-D Unicorn", 1, 3, &mut ledger).unwrap_err();
        assert!(matches!(err, ShopError::BloodlineRequired { need: 5, have: 3 }));
        // Ledger unchanged
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 1000);
    }

    #[test]
    fn buy_exact_bloodline_threshold_succeeds() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        shop.add_item(gated_item("Delta Plus", 100, 4));
        let mut ledger = funded_ledger(1, 500);
        // bloodline == required → allowed
        assert!(shop.buy("Delta Plus", 1, 4, &mut ledger).is_ok());
    }

    #[test]
    fn buy_out_of_stock_does_not_touch_ledger() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        shop.add_item(basic_item("Fin Funnel", 200, Some(0)));
        let mut ledger = funded_ledger(1, 1000);
        let _ = shop.buy("Fin Funnel", 1, 0, &mut ledger);
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 1000);
        assert_eq!(ledger.balance_of(AccountType::ShopRevenue), 0);
    }

    // ── Shop::sell_item ───────────────────────────────────────────────────────

    #[test]
    fn sell_item_awards_fifty_percent_of_base_value() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        let mut ledger = Ledger::new();
        let payout = shop.sell_item("Old Sword", 1, 200, &mut ledger).unwrap();
        assert_eq!(payout, 100);
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 100);
    }

    #[test]
    fn sell_item_truncates_odd_base_value() {
        // integer division: 101 / 2 = 50 (shop rounds down in its favor)
        let mut shop = Shop::new("Test".to_string(), 1.0);
        let mut ledger = Ledger::new();
        let payout = shop.sell_item("Chip", 1, 101, &mut ledger).unwrap();
        assert_eq!(payout, 50);
    }

    #[test]
    fn sell_item_total_balance_invariant() {
        let mut shop = Shop::new("Test".to_string(), 1.0);
        let mut ledger = Ledger::new();
        shop.sell_item("Item", 1, 100, &mut ledger).unwrap();
        assert_eq!(ledger.total_balance(), 0, "double-entry invariant");
    }
}
