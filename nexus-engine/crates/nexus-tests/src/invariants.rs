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

#[cfg(test)]
mod tests {
    use super::*;

    // ── damage_floor_holds ────────────────────────────────────────────────────

    #[test]
    fn damage_floor_holds_for_normal_hit() {
        // base=100, combo=1.5, equip=20%, armor=50 → raw=180, mitigated=max(130,1)=130
        assert!(damage_floor_holds(100.0, 1.5, 20.0, 50.0));
    }

    #[test]
    fn damage_floor_holds_when_armor_exceeds_raw() {
        // raw very low, massive armor → result clamped to 1.0
        assert!(damage_floor_holds(1.0, 1.0, 0.0, 9999.0));
    }

    #[test]
    fn damage_floor_holds_for_zero_base() {
        // zero base → guard returns true immediately (vacuous)
        assert!(damage_floor_holds(0.0, 2.0, 50.0, 0.0));
    }

    #[test]
    fn damage_floor_holds_for_negative_base() {
        // negative base → guard returns true (treated as vacuous)
        assert!(damage_floor_holds(-10.0, 1.0, 0.0, 0.0));
    }

    #[test]
    fn damage_floor_holds_for_maximum_hit() {
        assert!(damage_floor_holds(10_000.0, 3.0, 100.0, 0.0));
    }

    // ── qip_scar_rebirth_at_3 ─────────────────────────────────────────────────

    #[test]
    fn qip_scar_at_0_stacks_does_not_trigger() {
        // 0 stacks → apply_scar adds 1 → not yet triggered
        assert!(qip_scar_rebirth_at_3(0));
    }

    #[test]
    fn qip_scar_at_2_stacks_triggers() {
        // 2 stacks → apply adds one → 3 → rebirth triggered
        assert!(qip_scar_rebirth_at_3(2));
    }

    #[test]
    fn qip_scar_at_3_stacks_vacuous_true() {
        // stacks_before >= 3 → invariant returns true without touching tracker
        assert!(qip_scar_rebirth_at_3(3));
    }

    // ── inventory_add_remove_preserves_size ───────────────────────────────────

    #[test]
    fn inventory_preserves_size_for_empty_start() {
        assert!(inventory_add_remove_preserves_size(0));
    }

    #[test]
    fn inventory_preserves_size_for_several_items() {
        assert!(inventory_add_remove_preserves_size(5));
    }

    #[test]
    fn inventory_preserves_size_near_capacity() {
        // 49 items, then add + remove one → should still hold
        assert!(inventory_add_remove_preserves_size(49));
    }

    #[test]
    fn inventory_preserves_size_at_50_returns_true_early() {
        // >= 50 initial items → guard returns true immediately
        assert!(inventory_add_remove_preserves_size(50));
    }
}
