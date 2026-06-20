pub struct ComboTracker {
    pub depth: u32,
    turns_since_attack: u32,
    reset_threshold: u32,
}

impl ComboTracker {
    pub fn new(reset_threshold: u32) -> Self {
        ComboTracker {
            depth: 0,
            turns_since_attack: 0,
            reset_threshold,
        }
    }

    /// Call when player successfully attacks.
    pub fn on_hit(&mut self) {
        self.depth += 1;
        self.turns_since_attack = 0;
    }

    /// Call when player does anything other than attack.
    pub fn on_non_attack_turn(&mut self) {
        self.turns_since_attack += 1;
        if self.turns_since_attack >= self.reset_threshold {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.depth = 0;
        self.turns_since_attack = 0;
    }

    pub fn multiplier(&self) -> f32 {
        match self.depth {
            0 | 1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => 3.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.depth > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker() -> ComboTracker {
        ComboTracker::new(2) // reset after 2 non-attack turns
    }

    // ── multiplier table ──────────────────────────────────────────────────────

    #[test]
    fn multiplier_at_depth_0_is_1() {
        assert_eq!(tracker().multiplier(), 1.0);
    }

    #[test]
    fn multiplier_table_matches_spec() {
        let mut c = tracker();
        c.on_hit(); assert_eq!(c.multiplier(), 1.0); // depth 1
        c.on_hit(); assert_eq!(c.multiplier(), 1.5); // depth 2
        c.on_hit(); assert_eq!(c.multiplier(), 2.0); // depth 3
        c.on_hit(); assert_eq!(c.multiplier(), 3.0); // depth 4+
        c.on_hit(); assert_eq!(c.multiplier(), 3.0); // depth 5 — still 3.0
    }

    // ── on_hit / is_active ────────────────────────────────────────────────────

    #[test]
    fn is_active_after_first_hit() {
        let mut c = tracker();
        assert!(!c.is_active());
        c.on_hit();
        assert!(c.is_active());
    }

    #[test]
    fn on_hit_increments_depth() {
        let mut c = tracker();
        c.on_hit();
        c.on_hit();
        assert_eq!(c.depth, 2);
    }

    #[test]
    fn on_hit_resets_non_attack_counter() {
        let mut c = tracker();
        c.on_non_attack_turn(); // counter = 1
        c.on_hit();              // counter resets to 0
        c.on_non_attack_turn(); // counter = 1 — not yet at threshold 2
        assert!(c.is_active(), "combo should still be active after 1 non-attack post-hit");
    }

    // ── reset / on_non_attack_turn ────────────────────────────────────────────

    #[test]
    fn reset_clears_depth_and_counter() {
        let mut c = tracker();
        c.on_hit(); c.on_hit();
        c.reset();
        assert_eq!(c.depth, 0);
        assert!(!c.is_active());
    }

    #[test]
    fn two_non_attack_turns_resets_combo() {
        let mut c = tracker(); // threshold = 2
        c.on_hit();
        c.on_non_attack_turn();
        c.on_non_attack_turn(); // hits threshold → reset
        assert_eq!(c.depth, 0);
        assert!(!c.is_active());
    }

    #[test]
    fn one_non_attack_turn_does_not_reset() {
        let mut c = tracker();
        c.on_hit();
        c.on_non_attack_turn(); // counter = 1, threshold = 2 → no reset yet
        assert!(c.is_active());
    }
}
