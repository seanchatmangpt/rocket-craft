/// A combo chain whose maximum depth is a compile-time constant `N`.
///
/// Multiplier table:
/// - depth 0 or 1 → 1.0 ×
/// - depth 2      → 1.5 ×
/// - depth 3      → 2.0 ×
/// - depth 4+     → 3.0 × (Trans-Am / overdrive zone; triggers at exactly depth 4)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboChain<const MAX_DEPTH: usize> {
    depth: usize,
    idle_turns: u32,
    reset_after_turns: u32,
}

impl<const N: usize> ComboChain<N> {
    /// Create a new combo chain.
    ///
    /// `reset_after_turns`: how many consecutive non-attack turns before the
    /// combo depth resets to 0.
    pub fn new(reset_after_turns: u32) -> Self {
        ComboChain {
            depth: 0,
            idle_turns: 0,
            reset_after_turns,
        }
    }

    /// Current combo depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Damage multiplier for the current depth.
    pub fn multiplier(&self) -> f32 {
        match self.depth {
            0 | 1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => 3.0, // Trans-Am zone
        }
    }

    /// `true` when depth >= 4 (Trans-Am / overdrive zone active).
    pub fn is_trans_am_zone(&self) -> bool {
        self.depth >= 4
    }

    /// Record a successful hit — advances depth (capped at `N`) and resets idle counter.
    pub fn on_hit(&mut self) {
        self.depth = (self.depth + 1).min(N);
        self.idle_turns = 0;
    }

    /// Record a non-attack turn — advances idle counter and resets combo if
    /// the idle threshold is reached.
    pub fn on_non_attack_turn(&mut self) {
        self.idle_turns += 1;
        if self.idle_turns >= self.reset_after_turns {
            self.reset();
        }
    }

    /// Unconditionally reset depth and idle counter.
    pub fn reset(&mut self) {
        self.depth = 0;
        self.idle_turns = 0;
    }
}

/// Standard 5-hit combo chain (most mobile suits).
pub type StandardCombo = ComboChain<5>;

/// Trans-Am 7-hit combo chain (Gundam 00 special; Trans-Am zone activates at depth 4).
pub type TransAmCombo = ComboChain<7>;

#[cfg(test)]
mod tests {
    use super::*;

    type Combo = StandardCombo; // 5-hit cap

    #[test]
    fn new_combo_starts_at_depth_zero() {
        let c = Combo::new(3);
        assert_eq!(c.depth(), 0);
        assert_eq!(c.multiplier(), 1.0);
    }

    #[test]
    fn multiplier_table_matches_spec() {
        let mut c = Combo::new(10);
        // depth 0 → 1.0
        assert_eq!(c.multiplier(), 1.0);
        c.on_hit(); // depth 1
        assert_eq!(c.multiplier(), 1.0);
        c.on_hit(); // depth 2
        assert_eq!(c.multiplier(), 1.5);
        c.on_hit(); // depth 3
        assert_eq!(c.multiplier(), 2.0);
        c.on_hit(); // depth 4 — Trans-Am zone
        assert_eq!(c.multiplier(), 3.0);
    }

    #[test]
    fn depth_capped_at_max_depth() {
        let mut c = Combo::new(10);
        for _ in 0..20 {
            c.on_hit();
        }
        assert_eq!(c.depth(), 5, "StandardCombo must cap at 5");
    }

    #[test]
    fn trans_am_zone_activates_at_depth_4() {
        let mut c: TransAmCombo = ComboChain::new(10);
        for _ in 0..3 {
            c.on_hit();
            assert!(!c.is_trans_am_zone());
        }
        c.on_hit(); // depth 4
        assert!(c.is_trans_am_zone(), "Trans-Am must activate at depth >= 4");
    }

    #[test]
    fn idle_turns_reset_combo_after_threshold() {
        let mut c = Combo::new(2); // reset after 2 idle turns
        c.on_hit();
        c.on_hit(); // depth 2
        assert_eq!(c.depth(), 2);
        c.on_non_attack_turn(); // idle 1
        assert_eq!(c.depth(), 2, "combo must not reset before threshold");
        c.on_non_attack_turn(); // idle 2 — should reset
        assert_eq!(c.depth(), 0, "combo must reset at idle threshold");
    }

    #[test]
    fn hit_resets_idle_counter() {
        let mut c = Combo::new(2);
        c.on_hit();
        c.on_non_attack_turn(); // idle 1
        c.on_hit(); // hit resets idle counter
        c.on_non_attack_turn(); // idle 1 again — not 2
        assert_ne!(c.depth(), 0, "idle counter must reset on hit");
    }

    #[test]
    fn explicit_reset_clears_depth_and_idle() {
        let mut c = Combo::new(5);
        c.on_hit();
        c.on_hit();
        c.reset();
        assert_eq!(c.depth(), 0);
        assert_eq!(c.multiplier(), 1.0);
    }
}
