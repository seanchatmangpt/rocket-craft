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
