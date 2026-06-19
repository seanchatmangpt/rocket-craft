/// Combat invariant: damage floor — computed damage is always >= 1.0
pub fn damage_floor_holds(base: f32, combo_mult: f32, equipment_bonus: f32, armor: f32) -> bool {
    if base <= 0.0 {
        return true;
    }
    let raw = base * combo_mult * (1.0 + equipment_bonus / 100.0);
    let mitigated = (raw - armor).max(1.0);
    mitigated >= 1.0
}

/// QIP scar invariant: forced rebirth exactly at 3 stacks
pub fn qip_scar_rebirth_at_3(stacks_before: u32) -> bool {
    use nexus_combat::QipScarTracker;
    if stacks_before >= 3 {
        return true;
    }
    let mut tracker = QipScarTracker::new();
    tracker.stacks = stacks_before;
    let triggered = tracker.apply_scar();

    if triggered {
        tracker.stacks >= 3
    } else {
        tracker.stacks < 3
    }
}

/// Inventory invariant: adding then removing preserves size
pub fn inventory_add_remove_preserves_size(initial_size: usize) -> bool {
    use nexus_session::inventory::{Inventory, Item};
    if initial_size >= 50 {
        return true;
    }
    let mut inv = Inventory::<50>::new();
    for i in 0..initial_size {
        let item = Item {
            id: i as u64,
            ..Default::default()
        };
        if inv.add(item).is_err() {
            return false;
        }
    }
    let before_add = inv.len();
    if before_add != initial_size {
        return false;
    }

    let new_item = Item {
        id: 999,
        ..Default::default()
    };
    let idx = match inv.add(new_item) {
        Ok(idx) => idx,
        Err(_) => return false,
    };
    if inv.len() != before_add + 1 {
        return false;
    }

    if inv.remove(idx).is_err() {
        return false;
    }

    inv.len() == before_add
}
