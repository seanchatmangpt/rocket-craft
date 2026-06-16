use nexus_session::{
    inventory::{Inventory, InventoryError, Item},
    player::PlayerProfile,
    session::{Connecting, InLobby, InMatch, PlayerSession},
};

use proptest::prelude::*;

// ────────────────────────────────────────────────────────────────────────────
// Session state machine — unit tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn connecting_to_authenticated_on_valid_token() {
    let session = PlayerSession::<Connecting>::new(1, "Siris".to_string());
    let auth = session.authenticate(true);
    assert!(auth.is_ok());
}

#[test]
fn connecting_to_disconnected_on_invalid_token() {
    let session = PlayerSession::<Connecting>::new(1, "Siris".to_string());
    let auth = session.authenticate(false);
    assert!(auth.is_err());
}

#[test]
fn reject_goes_to_disconnected() {
    let session = PlayerSession::<Connecting>::new(2, "Banned".to_string());
    let _disconnected = session.reject();
    // The fact that this compiles and doesn't panic is the assertion.
}

#[test]
fn full_session_lifecycle() {
    let conn = PlayerSession::<Connecting>::new(42, "Aran".to_string());
    let auth = conn.authenticate(true).unwrap();
    let lobby: PlayerSession<InLobby> = auth.enter_lobby();
    let (in_match, match_id): (PlayerSession<InMatch>, u64) = lobby.enter_match(999);
    assert_eq!(match_id, 999);
    let back_in_lobby: PlayerSession<InLobby> = in_match.match_complete();
    let _disconnected = back_in_lobby.disconnect();
}

#[test]
fn lobby_to_spectate_lifecycle() {
    let lobby = PlayerSession::<Connecting>::new(7, "Observer".to_string())
        .authenticate(true)
        .unwrap()
        .enter_lobby();
    let (spec, mid) = lobby.spectate(12);
    assert_eq!(mid, 12);
    let _back = spec.leave_spectate();
}

#[test]
fn authenticated_disconnect() {
    let auth = PlayerSession::<Connecting>::new(3, "Phantom".to_string())
        .authenticate(true)
        .unwrap();
    let _d = auth.disconnect();
}

// ────────────────────────────────────────────────────────────────────────────
// Inventory — unit tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn inventory_add_and_retrieve() {
    let mut inv = Inventory::<5>::new();
    let item = Item {
        id: 1,
        name: "Beam Saber".to_string(),
        attack_bonus: 10,
        ..Item::default()
    };
    let idx = inv.add(item).unwrap();
    assert_eq!(inv.get(idx).unwrap().name, "Beam Saber");
    assert_eq!(inv.len(), 1);
}

#[test]
fn inventory_at_capacity_returns_error() {
    let mut inv = Inventory::<3>::new();
    for i in 0..3 {
        inv.add(Item {
            id: i,
            name: format!("Item {}", i),
            ..Item::default()
        })
        .unwrap();
    }
    let result = inv.add(Item {
        id: 99,
        name: "overflow".to_string(),
        ..Item::default()
    });
    assert!(matches!(result, Err(InventoryError::Full { capacity: 3 })));
}

#[test]
fn inventory_remove_out_of_range() {
    let mut inv = Inventory::<5>::new();
    let err = inv.remove(0);
    assert!(matches!(err, Err(InventoryError::InvalidSlot(0))));
}

#[test]
fn inventory_find_by_name() {
    let mut inv = Inventory::<10>::new();
    inv.add(Item {
        id: 1,
        name: "Shield".to_string(),
        defense_bonus: 5,
        ..Item::default()
    })
    .unwrap();
    let found = inv.find_by_name("Shield");
    assert!(found.is_some());
    assert_eq!(found.unwrap().1.defense_bonus, 5);
}

#[test]
fn inventory_total_bonuses() {
    let mut inv = Inventory::<10>::new();
    inv.add(Item {
        id: 1,
        name: "A".to_string(),
        attack_bonus: 3,
        defense_bonus: 1,
        ..Item::default()
    })
    .unwrap();
    inv.add(Item {
        id: 2,
        name: "B".to_string(),
        attack_bonus: 7,
        defense_bonus: 4,
        ..Item::default()
    })
    .unwrap();
    assert_eq!(inv.total_attack_bonus(), 10);
    assert_eq!(inv.total_defense_bonus(), 5);
}

// ────────────────────────────────────────────────────────────────────────────
// PlayerProfile — unit tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn player_level_up_on_xp_threshold() {
    let mut player = PlayerProfile::new(1, "Siris".to_string());
    // Level 2 requires 100 * 4 = 400 XP.
    assert!(!player.apply_xp_gain(399));
    assert_eq!(player.level, 1);
    assert!(player.apply_xp_gain(1));
    assert_eq!(player.level, 2);
}

#[test]
fn player_spend_gold_success() {
    let mut player = PlayerProfile::new(1, "Rich".to_string());
    player.gold = 500;
    player.spend_gold(200).unwrap();
    assert_eq!(player.gold, 300);
}

#[test]
fn player_spend_gold_insufficient() {
    let mut player = PlayerProfile::new(1, "Poor".to_string());
    player.gold = 10;
    let err = player.spend_gold(100);
    assert!(err.is_err());
    // Gold must not have changed.
    assert_eq!(player.gold, 10);
}

#[test]
fn player_is_alive() {
    let mut player = PlayerProfile::new(1, "Alive".to_string());
    assert!(player.is_alive());
    player.hp = 0.0;
    assert!(!player.is_alive());
}

#[test]
fn bloodline_labels_are_correct() {
    let mut player = PlayerProfile::new(1, "t".to_string());

    player.bloodline = 0;
    assert_eq!(player.bloodline_label(), "First Blood");
    player.bloodline = 3;
    assert_eq!(player.bloodline_label(), "Bloodline Awakened");
    player.bloodline = 7;
    assert_eq!(player.bloodline_label(), "Veteran");
    player.bloodline = 12;
    assert_eq!(player.bloodline_label(), "Deathless");
    player.bloodline = 17;
    assert_eq!(player.bloodline_label(), "Ausar's Echo");
    player.bloodline = 20;
    assert_eq!(player.bloodline_label(), "Negative Bloodline");
    player.bloodline = 99;
    assert_eq!(player.bloodline_label(), "Beyond Reckoning");
}

// ────────────────────────────────────────────────────────────────────────────
// Property-based tests
// ────────────────────────────────────────────────────────────────────────────

proptest! {
    /// add-then-remove is an identity operation on both item content and length.
    #[test]
    fn add_remove_identity(name in "[a-z]{3,20}") {
        let mut inv = Inventory::<10>::new();
        let item = Item { id: 1, name: name.clone(), ..Item::default() };
        let idx = inv.add(item).unwrap();
        let removed = inv.remove(idx).unwrap();
        prop_assert_eq!(removed.name, name);
        prop_assert_eq!(inv.len(), 0);
    }

    /// XP gain never decreases the player's level.
    #[test]
    fn xp_gain_level_monotone(xp_batches in prop::collection::vec(1u64..10000, 1..20)) {
        let mut player = PlayerProfile::new(1, "test".to_string());
        let mut prev_level = player.level;
        for xp in xp_batches {
            player.apply_xp_gain(xp);
            prop_assert!(player.level >= prev_level);
            prev_level = player.level;
        }
    }

    /// Gold can never underflow — it stays between 0 and the starting amount.
    #[test]
    fn gold_spend_no_underflow(starting in 0u32..10000, spend in 0u32..20000) {
        let mut player = PlayerProfile::new(1, "test".to_string());
        player.gold = starting;
        let _ = player.spend_gold(spend); // may fail legitimately
        prop_assert!(player.gold <= starting);
    }

    /// Inventory length never exceeds its compile-time capacity.
    #[test]
    fn inventory_len_never_exceeds_cap(additions in 0usize..15) {
        let mut inv = Inventory::<10>::new();
        for i in 0..additions {
            let _ = inv.add(Item { id: i as u64, name: format!("i{}", i), ..Item::default() });
        }
        prop_assert!(inv.len() <= inv.capacity());
    }
}
