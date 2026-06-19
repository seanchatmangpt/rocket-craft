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

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_types::{AttackDir, ParryOutcome};

    #[test]
    fn any_direction_parry_is_normal() {
        let outcome = ParryResolver::resolve(AttackDir::Left, None);
        assert_eq!(outcome, ParryOutcome::Normal);
    }

    #[test]
    fn exact_direction_match_is_perfect() {
        let outcome = ParryResolver::resolve(AttackDir::Right, Some(AttackDir::Right));
        assert_eq!(outcome, ParryOutcome::Perfect);
    }

    #[test]
    fn wrong_direction_is_normal_not_perfect() {
        let outcome = ParryResolver::resolve(AttackDir::Left, Some(AttackDir::Right));
        assert_eq!(outcome, ParryOutcome::Normal);
    }

    #[test]
    fn shield_parry_not_broken_before_three_perfects() {
        let (outcome, broken) = ParryResolver::resolve_shield_parry(
            AttackDir::Overhead,
            Some(AttackDir::Overhead),
            1, // 2nd perfect parry total
        );
        assert_eq!(outcome, ParryOutcome::Perfect);
        assert!(!broken, "shield should not break until 3rd perfect");
    }

    #[test]
    fn shield_breaks_on_third_perfect_parry() {
        let (outcome, broken) = ParryResolver::resolve_shield_parry(
            AttackDir::Overhead,
            Some(AttackDir::Overhead),
            2, // this is the 3rd (parries_so_far=2 + 1 = 3)
        );
        assert_eq!(outcome, ParryOutcome::Perfect);
        assert!(broken, "shield must break on 3rd cumulative perfect parry");
    }

    #[test]
    fn shield_normal_parry_never_breaks_shield() {
        let (_, broken) = ParryResolver::resolve_shield_parry(AttackDir::Left, None, 99);
        assert!(!broken, "normal parry (any-dir) must never break shield");
    }
}
