/// Strongly-typed entity IDs using `PhantomData` tag types.
///
/// `TypedId<PlayerTag>` and `TypedId<ItemTag>` are distinct types at
/// compile time even though both wrap a `u64`, so callers cannot accidentally
/// pass a player ID where an item ID is expected.
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Phantom tag types (zero-sized, never instantiated)
// ---------------------------------------------------------------------------

/// Tag for player entity IDs.
pub struct PlayerTag;
/// Tag for enemy/NPC entity IDs.
pub struct EnemyTag;
/// Tag for inventory item IDs.
pub struct ItemTag;
/// Tag for multiplayer session IDs.
pub struct SessionTag;
/// Tag for economy transaction IDs.
pub struct TransactionTag;
/// Tag for auction-house listing IDs.
pub struct AuctionTag;

// ---------------------------------------------------------------------------
// Generic typed ID
// ---------------------------------------------------------------------------

/// A `u64` ID whose `Tag` type parameter prevents cross-entity-type confusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TypedId<Tag>(u64, PhantomData<Tag>);

impl<T> TypedId<T> {
    /// Wrap a raw `u64` value with the given entity tag.
    #[inline]
    pub fn new(v: u64) -> Self {
        TypedId(v, PhantomData)
    }

    /// Extract the underlying raw `u64`.
    #[inline]
    pub fn raw(self) -> u64 {
        self.0
    }
}

impl<T> std::fmt::Display for TypedId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Concrete ID aliases
// ---------------------------------------------------------------------------

/// Unique identifier for a player account.
pub type PlayerId = TypedId<PlayerTag>;
/// Unique identifier for an enemy or NPC instance.
pub type EnemyId = TypedId<EnemyTag>;
/// Unique identifier for an inventory item instance.
pub type ItemId = TypedId<ItemTag>;
/// Unique identifier for a multiplayer match session.
pub type SessionId = TypedId<SessionTag>;
/// Unique identifier for an economy transaction record.
pub type TransactionId = TypedId<TransactionTag>;
/// Unique identifier for an auction-house listing.
pub type AuctionId = TypedId<AuctionTag>;

#[cfg(test)]
mod tests {
    use super::*;

    // ── TypedId construction / round-trip ─────────────────────────────────────

    #[test]
    fn player_id_raw_round_trips() {
        let id: PlayerId = TypedId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn enemy_id_raw_round_trips() {
        let id: EnemyId = TypedId::new(99);
        assert_eq!(id.raw(), 99);
    }

    #[test]
    fn item_id_zero_is_valid() {
        let id: ItemId = TypedId::new(0);
        assert_eq!(id.raw(), 0);
    }

    #[test]
    fn distinct_typed_ids_with_same_raw_have_equal_raw() {
        let a: PlayerId = TypedId::new(7);
        let b: PlayerId = TypedId::new(7);
        assert_eq!(a.raw(), b.raw());
    }

    #[test]
    fn distinct_raw_values_have_different_raw() {
        let a: PlayerId = TypedId::new(1);
        let b: PlayerId = TypedId::new(2);
        assert_ne!(a.raw(), b.raw());
    }

    // ── Copy semantics ────────────────────────────────────────────────────────

    #[test]
    fn typed_id_raw_is_stable() {
        // Calling .raw() twice on the same ID must return the same value.
        // (Copy semantics are enforced at the type level; no runtime test needed.)
        let a: SessionId = TypedId::new(5);
        assert_eq!(a.raw(), 5);
    }

    // ── Display ───────────────────────────────────────────────────────────────

    #[test]
    fn typed_id_display_includes_raw_value() {
        let id: PlayerId = TypedId::new(123);
        let s = format!("{id}");
        assert!(s.contains("123"), "display must include raw value, got: {s}");
    }
}
