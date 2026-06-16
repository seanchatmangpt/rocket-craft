use serde::{Deserialize, Serialize};

use crate::player::GundamSeries;

// ────────────────────────────────────────────────────────────────────────────
// Item domain types
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Infinity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Shield,
    Helmet,
    Ring,
    GunplaHead,
    GunplaArm,
    GunplaLeg,
    GunplaBackpack,
    PilotSuit,
}

/// A single item that can live in any inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub rarity: ItemRarity,
    pub item_type: ItemType,
    pub attack_bonus: u32,
    pub defense_bonus: u32,
    pub magic_bonus: u32,
    pub health_bonus: u32,
    pub value_gold: u32,
    pub is_equipped: bool,
    pub series: Option<GundamSeries>,
    pub special_ability: Option<String>,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            id: 0,
            name: String::new(),
            description: String::new(),
            rarity: ItemRarity::Common,
            item_type: ItemType::Weapon,
            attack_bonus: 0,
            defense_bonus: 0,
            magic_bonus: 0,
            health_bonus: 0,
            value_gold: 0,
            is_equipped: false,
            series: None,
            special_ability: None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Const-generic Inventory<N>
// ────────────────────────────────────────────────────────────────────────────

/// An inventory whose **maximum capacity** `CAP` is encoded in the type.
///
/// The backing storage is a `Vec` so that memory is only allocated for items
/// actually held, but insertion is rejected (at runtime, with a typed error)
/// once `CAP` slots are filled.
#[derive(Debug, Clone)]
pub struct Inventory<const CAP: usize> {
    slots: Vec<Item>,
}

impl<const CAP: usize> Default for Inventory<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> Inventory<CAP> {
    /// Create an empty inventory pre-allocated for `CAP` items.
    pub fn new() -> Self {
        Inventory {
            slots: Vec::with_capacity(CAP),
        }
    }

    /// The compile-time capacity of this inventory.
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// How many items are currently stored.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// `true` when `len() == CAP`.
    pub fn is_full(&self) -> bool {
        self.slots.len() >= CAP
    }

    /// `true` when no items are stored.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Add `item` and return its slot index.
    ///
    /// Returns [`InventoryError::Full`] when the inventory has reached
    /// capacity.
    pub fn add(&mut self, item: Item) -> Result<usize, InventoryError> {
        if self.is_full() {
            return Err(InventoryError::Full { capacity: CAP });
        }
        self.slots.push(item);
        Ok(self.slots.len() - 1)
    }

    /// Remove and return the item at `index`, shifting subsequent items left.
    ///
    /// Returns [`InventoryError::InvalidSlot`] when the index is out of range.
    pub fn remove(&mut self, index: usize) -> Result<Item, InventoryError> {
        if index >= self.slots.len() {
            return Err(InventoryError::InvalidSlot(index));
        }
        Ok(self.slots.remove(index))
    }

    /// Borrow the item at `index`, or `None` if out of range.
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.slots.get(index)
    }

    /// Iterate over all stored items in slot order.
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.slots.iter()
    }

    /// Find the first item whose `name` matches, returning `(slot_index, &item)`.
    pub fn find_by_name(&self, name: &str) -> Option<(usize, &Item)> {
        self.slots
            .iter()
            .enumerate()
            .find(|(_, item)| item.name == name)
    }

    /// Sum of all `attack_bonus` values across stored items.
    pub fn total_attack_bonus(&self) -> u32 {
        self.slots.iter().map(|i| i.attack_bonus).sum()
    }

    /// Sum of all `defense_bonus` values across stored items.
    pub fn total_defense_bonus(&self) -> u32 {
        self.slots.iter().map(|i| i.defense_bonus).sum()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Canonical size aliases
// ────────────────────────────────────────────────────────────────────────────

/// Standard player inventory — up to 50 items.
pub type PlayerInventory = Inventory<50>;

/// NPC inventory — smaller, up to 20 items.
pub type NpcInventory = Inventory<20>;

/// Shop catalog — large, up to 200 items.
pub type ShopInventory = Inventory<200>;

// ────────────────────────────────────────────────────────────────────────────
// Errors
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("inventory full: capacity is {capacity}")]
    Full { capacity: usize },

    #[error("invalid slot index: {0}")]
    InvalidSlot(usize),

    #[error("item not found: {0}")]
    ItemNotFound(String),
}
