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

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64, name: &str, atk: u32, def: u32) -> Item {
        Item {
            id,
            name: name.into(),
            attack_bonus: atk,
            defense_bonus: def,
            ..Item::default()
        }
    }

    // ── capacity / len / is_full / is_empty ───────────────────────────────────

    #[test]
    fn new_inventory_is_empty() {
        let inv = Inventory::<5>::new();
        assert!(inv.is_empty());
        assert_eq!(inv.len(), 0);
        assert_eq!(inv.capacity(), 5);
    }

    #[test]
    fn capacity_is_the_type_constant() {
        assert_eq!(Inventory::<10>::new().capacity(), 10);
        assert_eq!(Inventory::<200>::new().capacity(), 200);
    }

    #[test]
    fn is_full_only_after_cap_is_reached() {
        let mut inv = Inventory::<2>::new();
        assert!(!inv.is_full());
        inv.add(item(1, "sword", 10, 0)).unwrap();
        assert!(!inv.is_full());
        inv.add(item(2, "shield", 0, 5)).unwrap();
        assert!(inv.is_full());
    }

    // ── add ───────────────────────────────────────────────────────────────────

    #[test]
    fn add_returns_slot_index() {
        let mut inv = Inventory::<5>::new();
        assert_eq!(inv.add(item(1, "a", 0, 0)).unwrap(), 0);
        assert_eq!(inv.add(item(2, "b", 0, 0)).unwrap(), 1);
    }

    #[test]
    fn add_over_capacity_returns_full_error() {
        let mut inv = Inventory::<1>::new();
        inv.add(item(1, "a", 0, 0)).unwrap();
        let err = inv.add(item(2, "b", 0, 0)).unwrap_err();
        assert!(matches!(err, InventoryError::Full { capacity: 1 }));
    }

    // ── remove ────────────────────────────────────────────────────────────────

    #[test]
    fn remove_returns_item_and_shifts_remaining() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(1, "first", 1, 0)).unwrap();
        inv.add(item(2, "second", 2, 0)).unwrap();
        let removed = inv.remove(0).unwrap();
        assert_eq!(removed.name, "first");
        assert_eq!(inv.len(), 1);
        assert_eq!(inv.get(0).unwrap().name, "second");
    }

    #[test]
    fn remove_out_of_bounds_returns_invalid_slot() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(1, "x", 0, 0)).unwrap();
        let err = inv.remove(5).unwrap_err();
        assert!(matches!(err, InventoryError::InvalidSlot(5)));
    }

    // ── get / iter ────────────────────────────────────────────────────────────

    #[test]
    fn get_returns_correct_item_or_none() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(42, "ring", 3, 0)).unwrap();
        assert_eq!(inv.get(0).unwrap().id, 42);
        assert!(inv.get(1).is_none());
    }

    #[test]
    fn iter_yields_all_items_in_order() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(1, "a", 0, 0)).unwrap();
        inv.add(item(2, "b", 0, 0)).unwrap();
        let ids: Vec<u64> = inv.iter().map(|i| i.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    // ── find_by_name ──────────────────────────────────────────────────────────

    #[test]
    fn find_by_name_returns_index_and_ref() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(99, "Trans-Am Saber", 50, 0)).unwrap();
        let (idx, found) = inv.find_by_name("Trans-Am Saber").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(found.id, 99);
    }

    #[test]
    fn find_by_name_missing_returns_none() {
        let inv = Inventory::<5>::new();
        assert!(inv.find_by_name("Phantom Gundam").is_none());
    }

    // ── bonus aggregation ─────────────────────────────────────────────────────

    #[test]
    fn total_attack_bonus_sums_all_items() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(1, "a", 10, 0)).unwrap();
        inv.add(item(2, "b", 25, 0)).unwrap();
        assert_eq!(inv.total_attack_bonus(), 35);
    }

    #[test]
    fn total_defense_bonus_sums_all_items() {
        let mut inv = Inventory::<5>::new();
        inv.add(item(1, "a", 0, 7)).unwrap();
        inv.add(item(2, "b", 0, 13)).unwrap();
        assert_eq!(inv.total_defense_bonus(), 20);
    }

    #[test]
    fn empty_inventory_bonuses_are_zero() {
        let inv = Inventory::<5>::new();
        assert_eq!(inv.total_attack_bonus(), 0);
        assert_eq!(inv.total_defense_bonus(), 0);
    }
}
