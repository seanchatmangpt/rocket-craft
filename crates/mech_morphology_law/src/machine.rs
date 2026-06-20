//! `Machine<L: MorphologyLaw, P>` typestate. Phase `P` is a zero-sized
//! `PhantomData<P>` marker; illegal transitions are *absent impl blocks* and thus
//! compile errors.
//!
//! `Measured -> Validated -> Admitted`:
//! - only `Machine<L, Measured>::validate()` exists;
//! - only `Machine<L, Validated>::admit()` exists, and it is a runtime CLAIM_HOLD
//!   unless `Standing::Admitted` (no admission by exclusion).

use crate::law::{LawOutcome, MorphologyLaw};
use crate::mech::MeasuredMech;
use crate::refusal::{RefusalCode, Standing};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Phase marker: a measured-but-unvalidated mech.
#[derive(Debug)]
pub struct Measured;
/// Phase marker: validated, carrying a computed `Standing`.
#[derive(Debug)]
pub struct Validated;
/// Phase marker: terminal admitted proof phase.
#[derive(Debug)]
pub struct Admitted;

/// The typestate admission machine.
#[derive(Debug)]
pub struct Machine<L: MorphologyLaw, P> {
    law: L,
    mech: MeasuredMech,
    outcome: Option<LawOutcome>,
    _phase: PhantomData<P>,
}

/// A runtime claim-hold: admission was attempted but `Standing` was not
/// `Admitted`. This is the honest CONTINUE_REPAIR verdict.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClaimHold {
    /// The standing that blocked admission.
    pub standing: Standing,
    /// The live refusals.
    pub reason: Vec<RefusalCode>,
}

impl<L: MorphologyLaw + Default> Machine<L, Measured> {
    /// Construct a measured machine using the law's default configuration.
    pub fn new(mech: MeasuredMech) -> Self {
        Machine {
            law: L::default(),
            mech,
            outcome: None,
            _phase: PhantomData,
        }
    }
}

impl<L: MorphologyLaw> Machine<L, Measured> {
    /// Construct a measured machine with an explicit law instance.
    pub fn with_law(law: L, mech: MeasuredMech) -> Self {
        Machine {
            law,
            mech,
            outcome: None,
            _phase: PhantomData,
        }
    }

    /// The only legal transition from `Measured`: validate against the law.
    /// Consumes self and yields a `Validated` machine carrying the outcome.
    pub fn validate(self) -> Machine<L, Validated> {
        let outcome = self.law.validate(&self.mech);
        Machine {
            law: self.law,
            mech: self.mech,
            outcome: Some(outcome),
            _phase: PhantomData,
        }
    }
}

impl<L: MorphologyLaw> Machine<L, Validated> {
    /// Borrow the computed law outcome.
    pub fn outcome(&self) -> &LawOutcome {
        // Always `Some` in the Validated phase by construction.
        self.outcome.as_ref().unwrap_or(&MISSING_OUTCOME)
    }

    /// Borrow the computed standing.
    pub fn standing(&self) -> &Standing {
        &self.outcome().standing
    }

    /// Borrow the underlying mech (for receipt freshness checks).
    pub fn mech(&self) -> &MeasuredMech {
        &self.mech
    }

    /// The gated admission transition. Available *only* from `Validated`.
    /// Returns `Ok(Machine<L, Admitted>)` iff `Standing::Admitted`; otherwise a
    /// runtime [`ClaimHold`] — never admission by exclusion.
    pub fn admit(self) -> Result<Machine<L, Admitted>, ClaimHold> {
        let outcome = self.outcome.clone().unwrap_or(MISSING_OUTCOME.clone());
        match &outcome.standing {
            Standing::Admitted => Ok(Machine {
                law: self.law,
                mech: self.mech,
                outcome: self.outcome,
                _phase: PhantomData,
            }),
            other => Err(ClaimHold {
                standing: other.clone(),
                reason: outcome.refusals.clone(),
            }),
        }
    }
}

impl<L: MorphologyLaw> Machine<L, Admitted> {
    /// Borrow the proof outcome of an admitted mech.
    pub fn proof(&self) -> &LawOutcome {
        self.outcome.as_ref().unwrap_or(&MISSING_OUTCOME)
    }

    /// Borrow the admitted mech.
    pub fn mech(&self) -> &MeasuredMech {
        &self.mech
    }
}

// A total fallback outcome, never observed in practice (Validated/Admitted phases
// always carry `Some`). Keeps the API panic-free.
static MISSING_OUTCOME: LawOutcome = LawOutcome {
    standing: Standing::Unknown,
    refusals: Vec::new(),
    orientation: None,
};
