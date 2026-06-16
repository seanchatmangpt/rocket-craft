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
