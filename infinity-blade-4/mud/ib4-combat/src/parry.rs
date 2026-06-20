use ib4_core::types::AttackDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParryOutcome {
    Miss,
    NormalParry,
    PerfectParry,
}

pub enum ParryIntent {
    AnyParry,
    DirectionalParry(AttackDir),
}

pub struct ParryResolver;

impl ParryResolver {
    pub fn resolve(announced: AttackDir, intent: ParryIntent) -> ParryOutcome {
        match intent {
            ParryIntent::AnyParry => ParryOutcome::NormalParry,
            ParryIntent::DirectionalParry(d) => {
                if d == announced {
                    ParryOutcome::PerfectParry
                } else {
                    ParryOutcome::Miss
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::types::AttackDir;

    #[test]
    fn any_parry_intent_always_normal() {
        for dir in [AttackDir::Overhead, AttackDir::Left, AttackDir::Right] {
            let outcome = ParryResolver::resolve(dir.clone(), ParryIntent::AnyParry);
            assert_eq!(outcome, ParryOutcome::NormalParry);
        }
    }

    #[test]
    fn directional_exact_match_is_perfect() {
        let outcome = ParryResolver::resolve(
            AttackDir::Left,
            ParryIntent::DirectionalParry(AttackDir::Left),
        );
        assert_eq!(outcome, ParryOutcome::PerfectParry);
    }

    #[test]
    fn directional_mismatch_is_miss() {
        let outcome = ParryResolver::resolve(
            AttackDir::Overhead,
            ParryIntent::DirectionalParry(AttackDir::Left),
        );
        assert_eq!(outcome, ParryOutcome::Miss);
    }

    #[test]
    fn all_directions_match_themselves_perfectly() {
        for dir in [AttackDir::Overhead, AttackDir::Left, AttackDir::Right] {
            let o = ParryResolver::resolve(dir.clone(), ParryIntent::DirectionalParry(dir));
            assert_eq!(o, ParryOutcome::PerfectParry);
        }
    }

    #[test]
    fn no_direction_matches_a_different_one() {
        let cases = [
            (AttackDir::Overhead, AttackDir::Left),
            (AttackDir::Left, AttackDir::Right),
            (AttackDir::Right, AttackDir::Overhead),
        ];
        for (announced, guessed) in cases {
            let o = ParryResolver::resolve(announced, ParryIntent::DirectionalParry(guessed));
            assert_eq!(o, ParryOutcome::Miss);
        }
    }
}
