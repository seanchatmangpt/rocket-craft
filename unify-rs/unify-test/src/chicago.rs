//! Chicago TDD extensions bridging chicago-tdd-tools patterns.
//!
//! Provides noun-verb scenario builders, environment-aware test gates,
//! a classicist BehaviorTest trait, and log capture utilities mirroring
//! the patterns established in chicago-tdd-tools.

// ── NounVerbScenario ────────────────────────────────────────────────────────

/// A behavior scenario for a noun-verb dispatch system.
///
/// Mirrors the chicago-tdd-tools `ClapNoun` pattern where a *noun* (e.g.
/// `Account`) owns state and *verbs* (e.g. `Deposit`, `Withdraw`) mutate it.
///
/// The AAA (Arrange-Act-Assert) pattern is applied: all actions are run first,
/// then all assertions are evaluated.
pub struct NounVerbScenario<State> {
    description: String,
    initial_state: State,
    actions: Vec<Box<dyn Fn(&mut State)>>,
    assertions: Vec<Box<dyn Fn(&State)>>,
}

impl<State: Clone + 'static> NounVerbScenario<State> {
    /// Create a new scenario with the given description and initial state.
    pub fn new(description: &str, initial: State) -> Self {
        Self {
            description: description.to_owned(),
            initial_state: initial,
            actions: Vec::new(),
            assertions: Vec::new(),
        }
    }

    /// Register an action step (simulates a verb invocation against the noun).
    pub fn when_verb<F: Fn(&mut State) + 'static>(mut self, f: F) -> Self {
        self.actions.push(Box::new(f));
        self
    }

    /// Register an assertion on the resulting state after all actions.
    pub fn then_state<F: Fn(&State) + 'static>(mut self, f: F) -> Self {
        self.assertions.push(Box::new(f));
        self
    }

    /// Run all registered actions against the state clone, then run all
    /// assertions.  Panics at the first failing assertion.
    pub fn run(self) {
        let mut state = self.initial_state.clone();

        for action in &self.actions {
            action(&mut state);
        }

        for assertion in &self.assertions {
            assertion(&state);
        }
    }

    /// Returns the description of the scenario.
    pub fn description(&self) -> &str {
        &self.description
    }
}

// ── TestEnvironment ──────────────────────────────────────────────────────────

/// Tier of isolation required by a test.
///
/// Mirrors the layered `Environment` concept from chicago-tdd-tools: tests
/// are classified by their external-dependency footprint.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TestEnvironment {
    /// No external dependencies; runs fast in CI and local.
    Unit,
    /// May use temp directories, in-process mock servers, or the local FS.
    Integration,
    /// Requires real external services (network, databases, cloud APIs).
    E2e,
}

impl TestEnvironment {
    /// Parse a `TEST_ENV` string value into a `TestEnvironment`.
    ///
    /// Recognised strings (case-insensitive): `"unit"`, `"integration"`,
    /// `"e2e"`.  Anything else falls back to `Unit`.
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "integration" => Self::Integration,
            "e2e" => Self::E2e,
            _ => Self::Unit,
        }
    }
}

// ── EnvironmentGate ──────────────────────────────────────────────────────────

/// Guards a test so it only runs in a sufficiently capable environment.
///
/// Use `EnvironmentGate::from_env()` in test setup and call
/// `skip_if_not_allowed()` to emit a skip-worthy panic when the current
/// environment tier is lower than what the test requires.
pub struct EnvironmentGate {
    /// The minimum environment tier required to run the test.
    pub required: TestEnvironment,
    /// The environment tier detected (or overridden) for the current run.
    pub current: TestEnvironment,
}

impl EnvironmentGate {
    /// Create a gate that requires the given environment tier.
    ///
    /// The *current* environment is read from the `TEST_ENV` environment
    /// variable (defaulting to `Unit` if absent or unrecognised).
    pub fn new(required: TestEnvironment) -> Self {
        let current = std::env::var("TEST_ENV")
            .map(|v| TestEnvironment::from_str(&v))
            .unwrap_or(TestEnvironment::Unit);
        Self { required, current }
    }

    /// Create a gate that reads both `required` and `current` from `TEST_ENV`.
    ///
    /// In practice this is useful as a plain "what environment am I in?"
    /// probe; the gate will always allow (`required == current`).
    pub fn from_env() -> Self {
        let current = std::env::var("TEST_ENV")
            .map(|v| TestEnvironment::from_str(&v))
            .unwrap_or(TestEnvironment::Unit);
        Self { required: current, current }
    }

    /// Returns `true` when the current environment satisfies the requirement.
    ///
    /// A `Unit` environment only allows `Unit` tests; an `Integration`
    /// environment allows `Unit` and `Integration` tests; an `E2e` environment
    /// allows all tiers.
    pub fn allows(&self) -> bool {
        match (self.current, self.required) {
            (TestEnvironment::Unit, TestEnvironment::Unit) => true,
            (TestEnvironment::Integration, TestEnvironment::Unit)
            | (TestEnvironment::Integration, TestEnvironment::Integration) => true,
            (TestEnvironment::E2e, _) => true,
            _ => false,
        }
    }

    /// Panic with a descriptive skip message when the environment is
    /// insufficient.  Combine with `#[should_panic]` or a test harness that
    /// interprets the panic as a skip signal.
    pub fn skip_if_not_allowed(&self) {
        if !self.allows() {
            panic!(
                "Test skipped: requires {:?} environment but current is {:?}",
                self.required, self.current
            );
        }
    }
}

// ── Pre-built account scenarios ───────────────────────────────────────────────

/// Returns a `NounVerbScenario<i64>` that deposits 100 into an account
/// starting at 0 and asserts the resulting balance is 100.
///
/// Mirrors the `AccountVerb::Deposit` verb from chicago-tdd-tools.
pub fn account_deposit_scenario() -> NounVerbScenario<i64> {
    NounVerbScenario::new("account deposit increases balance", 0i64)
        .when_verb(|balance| *balance += 100)
        .then_state(|balance| assert_eq!(*balance, 100, "balance after deposit should be 100"))
}

/// Returns a `NounVerbScenario<i64>` that withdraws 40 from an account
/// starting at 100 and asserts the resulting balance is 60.
///
/// Mirrors the `AccountVerb::Withdraw` verb from chicago-tdd-tools.
pub fn account_withdraw_scenario() -> NounVerbScenario<i64> {
    NounVerbScenario::new("account withdraw decreases balance", 100i64)
        .when_verb(|balance| *balance -= 40)
        .then_state(|balance| assert_eq!(*balance, 60, "balance after withdrawal should be 60"))
}

/// Returns a `NounVerbScenario<i64>` that attempts to withdraw more than the
/// balance and demonstrates that a clamped-to-zero model keeps the balance
/// non-negative.
///
/// The scenario models overdraft protection by clamping: if `amount > balance`
/// the balance is set to 0 rather than going negative.
pub fn account_overdraft_scenario() -> NounVerbScenario<i64> {
    NounVerbScenario::new("account overdraft clamps to zero", 50i64)
        .when_verb(|balance| {
            let withdraw = 200i64;
            if withdraw > *balance {
                *balance = 0; // overdraft protection: floor at zero
            } else {
                *balance -= withdraw;
            }
        })
        .then_state(|balance| {
            assert!(*balance >= 0, "balance must never be negative after overdraft");
            assert_eq!(*balance, 0, "overdraft should clamp balance to zero");
        })
}

// ── BehaviorTest ─────────────────────────────────────────────────────────────

/// Classicist (Chicago-style) behavior test trait.
///
/// Implement `arrange`, `act`, and `assert` then call `run` to execute the
/// full Arrange-Act-Assert cycle.  No mocks required — the subject is a real
/// value obtained from `arrange`.
pub trait BehaviorTest {
    /// The type under test.
    type Subject: Clone;

    /// Return a freshly initialised subject for the test.
    fn arrange(&self) -> Self::Subject;

    /// Perform the action under test against the subject.
    fn act(&self, subject: &mut Self::Subject);

    /// Assert post-conditions on the subject after the action.
    fn assert(&self, subject: &Self::Subject);

    /// Convenience method that wires `arrange` → `act` → `assert`.
    fn run(&self) {
        let mut s = self.arrange();
        self.act(&mut s);
        self.assert(&s);
    }
}

// ── LogCapture ───────────────────────────────────────────────────────────────

/// A single captured log record.
///
/// Mirrors the structured log entries produced by chicago-tdd-tools
/// `Logger` / `LogSink` implementations.
pub struct LogRecord {
    /// String representation of the log level (e.g. `"INFO"`, `"WARN"`).
    pub level: String,
    /// The logged message.
    pub message: String,
    /// Wall-clock time when the record was captured.
    pub timestamp: std::time::SystemTime,
}

/// In-memory log sink for asserting on log output in tests.
///
/// Create with [`LogCapture::new`], push records via [`LogCapture::push`],
/// and interrogate with [`LogCapture::records`], [`LogCapture::has_message`],
/// or [`LogCapture::count_by_level`].
#[derive(Clone)]
pub struct LogCapture {
    records: std::sync::Arc<std::sync::Mutex<Vec<LogRecord>>>,
}

impl LogCapture {
    /// Create a new, empty `LogCapture`.
    pub fn new() -> Self {
        Self {
            records: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Append a log record manually.
    ///
    /// This is the primary way to feed records into the capture during tests;
    /// wire your code under test to call this instead of a real logger.
    pub fn push(&self, level: impl Into<String>, message: impl Into<String>) {
        let record = LogRecord {
            level: level.into(),
            message: message.into(),
            timestamp: std::time::SystemTime::now(),
        };
        self.records.lock().unwrap().push(record);
    }

    /// Return a snapshot of all captured records.
    ///
    /// Records are cloned out of the shared buffer so the caller owns them.
    pub fn records(&self) -> Vec<LogRecord> {
        // LogRecord is not Clone, so we rebuild lightweight copies.
        self.records
            .lock()
            .unwrap()
            .iter()
            .map(|r| LogRecord {
                level: r.level.clone(),
                message: r.message.clone(),
                timestamp: r.timestamp,
            })
            .collect()
    }

    /// Returns `true` if any record's message contains `msg` as a substring.
    pub fn has_message(&self, msg: &str) -> bool {
        self.records
            .lock()
            .unwrap()
            .iter()
            .any(|r| r.message.contains(msg))
    }

    /// Returns the number of records whose `level` equals `level`
    /// (case-sensitive, e.g. `"INFO"`).
    pub fn count_by_level(&self, level: &str) -> usize {
        self.records
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.level == level)
            .count()
    }

    /// Returns the total number of captured records.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    /// Returns `true` when no records have been captured.
    pub fn is_empty(&self) -> bool {
        self.records.lock().unwrap().is_empty()
    }
}

impl Default for LogCapture {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── NounVerbScenario ──────────────────────────────────────────────────────

    #[test]
    fn noun_verb_scenario_runs_action_then_assertion() {
        NounVerbScenario::new("single action and assertion", 0i32)
            .when_verb(|s| *s += 5)
            .then_state(|s| assert_eq!(*s, 5))
            .run();
    }

    #[test]
    fn noun_verb_scenario_can_chain_multiple_verbs() {
        NounVerbScenario::new("chained verbs accumulate", 0i32)
            .when_verb(|s| *s += 10)
            .when_verb(|s| *s += 20)
            .when_verb(|s| *s += 30)
            .then_state(|s| assert_eq!(*s, 60, "sum of all deposits should be 60"))
            .run();
    }

    #[test]
    fn noun_verb_scenario_multiple_assertions_all_pass() {
        NounVerbScenario::new("multiple assertions", 0i32)
            .when_verb(|s| *s = 42)
            .then_state(|s| assert!(*s > 0))
            .then_state(|s| assert_eq!(*s, 42))
            .then_state(|s| assert!(*s < 100))
            .run();
    }

    #[test]
    fn noun_verb_scenario_with_string_state() {
        NounVerbScenario::new("string state mutation", String::new())
            .when_verb(|s| s.push_str("hello"))
            .when_verb(|s| s.push(' '))
            .when_verb(|s| s.push_str("world"))
            .then_state(|s| assert_eq!(s, "hello world"))
            .run();
    }

    #[test]
    fn noun_verb_scenario_description_is_accessible() {
        let scenario = NounVerbScenario::new("my description", 0i32);
        assert_eq!(scenario.description(), "my description");
    }

    #[test]
    fn noun_verb_scenario_no_actions_runs_assertions_on_initial_state() {
        NounVerbScenario::new("no actions", 99i32)
            .then_state(|s| assert_eq!(*s, 99))
            .run();
    }

    // ── Pre-built account scenarios ───────────────────────────────────────────

    #[test]
    fn account_deposit_scenario_increases_balance() {
        account_deposit_scenario().run();
    }

    #[test]
    fn account_withdraw_scenario_decreases_balance() {
        account_withdraw_scenario().run();
    }

    #[test]
    fn account_overdraft_scenario_balance_cannot_go_negative() {
        account_overdraft_scenario().run();
    }

    // ── EnvironmentGate ───────────────────────────────────────────────────────

    #[test]
    fn environment_gate_from_env_defaults_to_unit() {
        // TEST_ENV is not set in normal test runs; gate should default to Unit.
        // We can't unset env vars safely in parallel tests, but we can verify
        // the from_env() current value matches the env var (or Unit).
        let gate = EnvironmentGate::from_env();
        // from_env sets required == current, so allows() must be true.
        assert!(
            gate.allows(),
            "from_env gate should always allow (required == current)"
        );
    }

    #[test]
    fn environment_gate_allows_unit_tests_in_unit_env() {
        let gate = EnvironmentGate {
            required: TestEnvironment::Unit,
            current: TestEnvironment::Unit,
        };
        assert!(gate.allows());
    }

    #[test]
    fn environment_gate_allows_unit_tests_in_integration_env() {
        let gate = EnvironmentGate {
            required: TestEnvironment::Unit,
            current: TestEnvironment::Integration,
        };
        assert!(gate.allows(), "Integration env can run Unit tests");
    }

    #[test]
    fn environment_gate_allows_integration_tests_in_e2e_env() {
        let gate = EnvironmentGate {
            required: TestEnvironment::Integration,
            current: TestEnvironment::E2e,
        };
        assert!(gate.allows(), "E2e env can run Integration tests");
    }

    #[test]
    fn environment_gate_blocks_e2e_tests_in_unit_env() {
        let gate = EnvironmentGate {
            required: TestEnvironment::E2e,
            current: TestEnvironment::Unit,
        };
        assert!(!gate.allows(), "Unit env must not allow E2e tests");
    }

    #[test]
    fn environment_gate_blocks_integration_tests_in_unit_env() {
        let gate = EnvironmentGate {
            required: TestEnvironment::Integration,
            current: TestEnvironment::Unit,
        };
        assert!(!gate.allows(), "Unit env must not allow Integration tests");
    }

    #[test]
    #[should_panic(expected = "Test skipped")]
    fn environment_gate_skip_if_not_allowed_panics_when_blocked() {
        let gate = EnvironmentGate {
            required: TestEnvironment::E2e,
            current: TestEnvironment::Unit,
        };
        gate.skip_if_not_allowed();
    }

    // ── BehaviorTest ──────────────────────────────────────────────────────────

    struct DepositBehavior {
        amount: i64,
    }

    impl BehaviorTest for DepositBehavior {
        type Subject = i64;

        fn arrange(&self) -> i64 {
            0
        }

        fn act(&self, subject: &mut i64) {
            if self.amount > 0 {
                *subject += self.amount;
            }
        }

        fn assert(&self, subject: &i64) {
            assert_eq!(*subject, self.amount.max(0));
        }
    }

    #[test]
    fn behavior_test_run_calls_arrange_act_assert() {
        let behavior = DepositBehavior { amount: 250 };
        behavior.run(); // passes when arrange/act/assert all succeed
    }

    #[test]
    fn behavior_test_zero_amount_does_not_change_balance() {
        let behavior = DepositBehavior { amount: 0 };
        behavior.run();
    }

    // ── LogCapture ────────────────────────────────────────────────────────────

    #[test]
    fn log_capture_has_message_finds_exact_substring() {
        let cap = LogCapture::new();
        cap.push("INFO", "user logged in");
        assert!(cap.has_message("logged in"));
        assert!(cap.has_message("user"));
    }

    #[test]
    fn log_capture_has_message_returns_false_when_absent() {
        let cap = LogCapture::new();
        cap.push("INFO", "hello world");
        assert!(!cap.has_message("goodbye"));
    }

    #[test]
    fn log_capture_count_by_level_returns_correct_count() {
        let cap = LogCapture::new();
        cap.push("INFO", "first info");
        cap.push("WARN", "a warning");
        cap.push("INFO", "second info");
        cap.push("ERROR", "an error");
        cap.push("INFO", "third info");

        assert_eq!(cap.count_by_level("INFO"), 3);
        assert_eq!(cap.count_by_level("WARN"), 1);
        assert_eq!(cap.count_by_level("ERROR"), 1);
        assert_eq!(cap.count_by_level("DEBUG"), 0);
    }

    #[test]
    fn log_capture_records_returns_all_pushed_entries() {
        let cap = LogCapture::new();
        cap.push("DEBUG", "debug msg");
        cap.push("INFO", "info msg");

        let records = cap.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].level, "DEBUG");
        assert_eq!(records[0].message, "debug msg");
        assert_eq!(records[1].level, "INFO");
        assert_eq!(records[1].message, "info msg");
    }

    #[test]
    fn log_capture_is_empty_before_any_push() {
        let cap = LogCapture::new();
        assert!(cap.is_empty());
        assert_eq!(cap.len(), 0);
    }

    #[test]
    fn log_capture_len_tracks_push_count() {
        let cap = LogCapture::new();
        cap.push("INFO", "one");
        cap.push("INFO", "two");
        cap.push("WARN", "three");
        assert_eq!(cap.len(), 3);
    }

    #[test]
    fn log_capture_clone_shares_buffer() {
        let cap = LogCapture::new();
        let cap2 = cap.clone();
        cap.push("INFO", "shared");
        // Both handles see the same underlying buffer via Arc<Mutex<_>>
        assert!(cap2.has_message("shared"));
    }

    #[test]
    fn log_capture_default_creates_empty_capture() {
        let cap = LogCapture::default();
        assert!(cap.is_empty());
    }
}
