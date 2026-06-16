//! Scenario DSL — given/when/then + mock state (Chicago-style classicist TDD).

/// A single named step in a [`Scenario`].
pub struct ScenarioStep<State> {
    pub name: String,
    pub f: Box<dyn Fn(&mut State)>,
}

/// Result of running a [`Scenario`].
#[derive(Debug)]
pub struct ScenarioResult {
    pub name: String,
    pub steps_run: usize,
    pub passed: bool,
    pub error: Option<String>,
}

/// A Scenario is an ordered list of given/when/then steps executed against a
/// shared mutable state value.  The state is cloned from `initial_state` at
/// construction time; each step mutates the live copy in sequence.
pub struct Scenario<State: Clone> {
    name: String,
    state: State,
    steps: Vec<ScenarioStep<State>>,
}

impl<State: Clone + std::fmt::Debug> Scenario<State> {
    /// Create a new scenario with the given name and initial state.
    pub fn new(name: impl Into<String>, initial_state: State) -> Self {
        Self {
            name: name.into(),
            state: initial_state,
            steps: Vec::new(),
        }
    }

    /// Append a *given* (setup) step.
    pub fn given(mut self, name: impl Into<String>, f: impl Fn(&mut State) + 'static) -> Self {
        self.steps.push(ScenarioStep { name: name.into(), f: Box::new(f) });
        self
    }

    /// Append a *when* (action) step.
    pub fn when(mut self, name: impl Into<String>, f: impl Fn(&mut State) + 'static) -> Self {
        self.steps.push(ScenarioStep { name: name.into(), f: Box::new(f) });
        self
    }

    /// Append a *then* (assertion) step.
    pub fn then(mut self, name: impl Into<String>, f: impl Fn(&mut State) + 'static) -> Self {
        self.steps.push(ScenarioStep { name: name.into(), f: Box::new(f) });
        self
    }

    /// Execute all steps in order, catching any panics.
    /// Returns a [`ScenarioResult`] describing success or failure.
    pub fn run(mut self) -> ScenarioResult {
        let name = self.name.clone();
        let total = self.steps.len();
        let mut steps_run = 0usize;

        for step in self.steps.drain(..) {
            let step_name = step.name.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (step.f)(&mut self.state);
            }));
            steps_run += 1;
            if let Err(e) = result {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    format!("panic in step '{step_name}'")
                };
                return ScenarioResult {
                    name,
                    steps_run,
                    passed: false,
                    error: Some(msg),
                };
            }
        }

        ScenarioResult { name, steps_run: total, passed: true, error: None }
    }

    /// Like [`run`], but panics with an informative message on failure.
    pub fn run_and_unwrap(self) {
        let result = self.run();
        if !result.passed {
            panic!(
                "Scenario '{}' failed after {}/{} steps: {}",
                result.name,
                result.steps_run,
                result.steps_run,
                result.error.unwrap_or_default()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scenario_has_no_steps() {
        let s: Scenario<i32> = Scenario::new("empty", 0);
        assert_eq!(s.steps.len(), 0);
        assert_eq!(s.name, "empty");
    }

    #[test]
    fn given_when_then_steps_run_in_order() {
        let result = Scenario::new("counter", 0i32)
            .given("start at zero", |state| assert_eq!(*state, 0))
            .when("increment", |state| *state += 1)
            .then("is one", |state| assert_eq!(*state, 1))
            .run();

        assert!(result.passed, "scenario should pass: {:?}", result.error);
        assert_eq!(result.steps_run, 3);
    }

    #[test]
    fn run_returns_passed_true_when_no_panics() {
        let result = Scenario::new("trivial", ())
            .given("noop", |_| {})
            .run();
        assert!(result.passed);
        assert!(result.error.is_none());
    }

    #[test]
    fn run_captures_failed_step() {
        let result = Scenario::new("bad", 0i32)
            .then("expect 99", |state| assert_eq!(*state, 99))
            .run();
        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    #[test]
    #[should_panic]
    fn run_and_unwrap_panics_on_failure() {
        Scenario::new("failing", 0i32)
            .then("fail", |state| assert_eq!(*state, 42))
            .run_and_unwrap();
    }
}
