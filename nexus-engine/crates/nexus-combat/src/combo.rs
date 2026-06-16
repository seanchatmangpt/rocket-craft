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
