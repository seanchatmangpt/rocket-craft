pub use nexus_types::{AttackDir, ParryOutcome};

/// Stateless resolver for parry interactions.
pub struct ParryResolver;

impl ParryResolver {
    /// Resolve a parry attempt.
    ///
    /// - `announced`: the direction the enemy telegraphed.
    /// - `player_dir`: the direction the player inputted (`None` = any-direction parry).
    ///
    /// Perfect parry requires an exact direction match.
    /// Normal parry uses any-direction input or a wrong-direction input.
    pub fn resolve(announced: AttackDir, player_dir: Option<AttackDir>) -> ParryOutcome {
        match player_dir {
            None => ParryOutcome::Normal,
            Some(d) if d == announced => ParryOutcome::Perfect,
            Some(_) => ParryOutcome::Normal, // wrong direction = blocked but not perfect
        }
    }

    /// GodKing shield parry — the shield breaks after 3 cumulative perfect parries.
    ///
    /// Returns `(outcome, shield_broken)`.
    pub fn resolve_shield_parry(
        announced: AttackDir,
        player_dir: Option<AttackDir>,
        parries_so_far: u32,
    ) -> (ParryOutcome, bool) {
        let outcome = Self::resolve(announced, player_dir);
        let shield_broken = outcome == ParryOutcome::Perfect && parries_so_far + 1 >= 3;
        (outcome, shield_broken)
    }
}
