/// Calculate final damage with all modifiers applied.
///
/// Formula:
/// ```text
/// raw        = base × combo_multiplier × (1 + equipment_bonus / 100)
/// with_bonus = raw × counter_bonus          (1.5× if perfect-parry counter)
/// final      = max(1.0, with_bonus − armor)
/// ```
///
/// `time_dilation` (0.5–1.3) affects reaction windows in the game loop;
/// it does **not** modify raw damage.
///
/// The damage floor is always **1.0** — a unit can never deal zero damage.
pub fn calculate_damage(
    base: f32,
    combo_multiplier: f32,
    equipment_bonus: f32,
    _time_dilation: f32, // affects reaction time, not damage
    is_perfect_parry_counter: bool,
    armor: f32,
) -> f32 {
    let raw = base * combo_multiplier * (1.0 + equipment_bonus / 100.0);
    let counter_bonus = if is_perfect_parry_counter { 1.5 } else { 1.0 };
    (raw * counter_bonus - armor).max(1.0)
}

// ---------------------------------------------------------------------------
// QIP Scar accumulation — GodKing Phase 2 mechanic
// ---------------------------------------------------------------------------

/// Tracks QIP (Quantum Intelligence Protocol) Scar stacks on the player.
///
/// At 3 stacks the game forces a rebirth (respawn at cost). The tracker
/// must be reset after a forced rebirth.
#[derive(Debug, Clone)]
pub struct QipScarTracker {
    pub stacks: u32,
    pub max_stacks: u32,
}

impl Default for QipScarTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl QipScarTracker {
    /// Create a new tracker with a cap of 3 stacks.
    pub fn new() -> Self {
        QipScarTracker {
            stacks: 0,
            max_stacks: 3,
        }
    }

    /// Apply one QIP Scar.
    ///
    /// Returns `true` when `max_stacks` has been reached, signalling a
    /// forced rebirth.
    pub fn apply_scar(&mut self) -> bool {
        self.stacks += 1;
        self.stacks >= self.max_stacks
    }

    /// Reset scar stacks (call after a forced rebirth).
    pub fn reset(&mut self) {
        self.stacks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── calculate_damage ──────────────────────────────────────────────────────

    #[test]
    fn floor_is_one_when_armor_exceeds_raw() {
        // high armor → floor kicks in
        let dmg = calculate_damage(10.0, 1.0, 0.0, 1.0, false, 999.0);
        assert_eq!(dmg, 1.0, "damage floor must be 1.0");
    }

    #[test]
    fn base_damage_no_modifiers() {
        // base=100, multiplier=1, no bonus, no armor → 100.0
        let dmg = calculate_damage(100.0, 1.0, 0.0, 1.0, false, 0.0);
        assert!((dmg - 100.0).abs() < 0.001);
    }

    #[test]
    fn equipment_bonus_scales_linearly() {
        // 50% equipment bonus → raw = 100 * 1 * 1.5 = 150
        let dmg = calculate_damage(100.0, 1.0, 50.0, 1.0, false, 0.0);
        assert!((dmg - 150.0).abs() < 0.001);
    }

    #[test]
    fn perfect_parry_counter_multiplies_by_1_5() {
        // base=100, perfect parry counter → 100 * 1.5 = 150
        let normal = calculate_damage(100.0, 1.0, 0.0, 1.0, false, 0.0);
        let counter = calculate_damage(100.0, 1.0, 0.0, 1.0, true, 0.0);
        assert!((counter / normal - 1.5).abs() < 0.001);
    }

    #[test]
    fn combo_multiplier_applied() {
        // combo_multiplier=2.0 doubles raw
        let dmg = calculate_damage(50.0, 2.0, 0.0, 1.0, false, 0.0);
        assert!((dmg - 100.0).abs() < 0.001);
    }

    #[test]
    fn armor_reduces_damage() {
        // raw=100 − armor=30 = 70
        let dmg = calculate_damage(100.0, 1.0, 0.0, 1.0, false, 30.0);
        assert!((dmg - 70.0).abs() < 0.001);
    }

    #[test]
    fn time_dilation_does_not_affect_damage() {
        let d1 = calculate_damage(100.0, 1.0, 0.0, 0.5, false, 0.0);
        let d2 = calculate_damage(100.0, 1.0, 0.0, 1.3, false, 0.0);
        assert!(
            (d1 - d2).abs() < 0.001,
            "time_dilation must not affect damage"
        );
    }

    // ── QipScarTracker ────────────────────────────────────────────────────────

    #[test]
    fn first_two_scars_do_not_trigger_rebirth() {
        let mut t = QipScarTracker::new();
        assert!(!t.apply_scar(), "1st scar must not trigger rebirth");
        assert!(!t.apply_scar(), "2nd scar must not trigger rebirth");
    }

    #[test]
    fn third_scar_triggers_rebirth() {
        let mut t = QipScarTracker::new();
        t.apply_scar();
        t.apply_scar();
        assert!(t.apply_scar(), "3rd scar must trigger rebirth");
    }

    #[test]
    fn reset_clears_stacks() {
        let mut t = QipScarTracker::new();
        t.apply_scar();
        t.apply_scar();
        t.reset();
        assert_eq!(t.stacks, 0);
        // After reset, two more scars must not trigger rebirth again
        assert!(!t.apply_scar());
        assert!(!t.apply_scar());
    }

    #[test]
    fn default_creates_fresh_tracker() {
        let t = QipScarTracker::default();
        assert_eq!(t.stacks, 0);
        assert_eq!(t.max_stacks, 3);
    }
}
