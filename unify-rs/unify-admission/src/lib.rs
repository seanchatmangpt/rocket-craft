//! `unify-admission` — Named-law gates bridging a static `StaticLaw` marker
//! system and a knhk-compatible `RuntimeLaw` trait.
//!
//! Design goals:
//! - Zero circular dependencies (does **not** import `unify-core` or `knhk`).
//! - Compile-time law identity via const `NAME` on `StaticLaw`.
//! - Runtime extensibility via the `RuntimeLaw` trait and `LawRegistry`.
//! - Composable gate chains via `GateChain`.

use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Core traits and types
// ─────────────────────────────────────────────────────────────────────────────

/// Marker trait for a static (compile-time) law.  Implemented by zero-sized
/// structs that give a law its identity (`NAME`).
pub trait StaticLaw {
    /// Unique human-readable name for this law.
    const NAME: &'static str;
}

/// A typed refusal produced when an artifact fails a gate for law `L`.
pub struct Refusal<L: StaticLaw> {
    /// Human-readable explanation of why the artifact was refused.
    pub message: String,
    /// Name of the law that was violated (equals `L::NAME`).
    pub law_name: &'static str,
    _phantom: PhantomData<L>,
}

impl<L: StaticLaw> Refusal<L> {
    /// Construct a new `Refusal` with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            law_name: L::NAME,
            _phantom: PhantomData,
        }
    }
}

impl<L: StaticLaw> fmt::Display for Refusal<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Law '{}' refused: {}", self.law_name, self.message)
    }
}

impl<L: StaticLaw> fmt::Debug for Refusal<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Refusal")
            .field("law_name", &self.law_name)
            .field("message", &self.message)
            .finish()
    }
}

/// An admission gate that checks whether `Self::Artifact` satisfies law `L`.
pub trait Admit<L: StaticLaw> {
    /// The type of value being checked.
    type Artifact;
    /// Return `Ok(())` if the artifact satisfies the law, or a typed `Refusal`.
    fn admit(&self, artifact: &Self::Artifact) -> Result<(), Refusal<L>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Built-in static laws
// ─────────────────────────────────────────────────────────────────────────────

/// Law: a name-like string must not be empty or whitespace-only.
pub struct NonEmptyNameLaw;
impl StaticLaw for NonEmptyNameLaw {
    const NAME: &'static str = "NonEmptyName";
}

/// Law: a collection must contain at least one element.
pub struct NonEmptyCollectionLaw;
impl StaticLaw for NonEmptyCollectionLaw {
    const NAME: &'static str = "NonEmptyCollection";
}

/// Law: a filesystem path must refer to an entry that exists.
pub struct ValidPathLaw;
impl StaticLaw for ValidPathLaw {
    const NAME: &'static str = "ValidPath";
}

/// Law: an Android project directory must contain a keystore file.
pub struct AndroidKeystorePresentLaw;
impl StaticLaw for AndroidKeystorePresentLaw {
    const NAME: &'static str = "AndroidKeystorePresent";
}

// ─────────────────────────────────────────────────────────────────────────────
// Built-in gates
// ─────────────────────────────────────────────────────────────────────────────

/// Gate that enforces `NonEmptyNameLaw` on `String` values.
pub struct NonEmptyNameGate;

impl Admit<NonEmptyNameLaw> for NonEmptyNameGate {
    type Artifact = String;

    fn admit(&self, name: &String) -> Result<(), Refusal<NonEmptyNameLaw>> {
        if name.trim().is_empty() {
            Err(Refusal::new("Name must not be empty or whitespace-only"))
        } else {
            Ok(())
        }
    }
}

/// Gate that enforces `NonEmptyCollectionLaw` on `Vec<T>` values.
pub struct NonEmptyVecGate<T>(PhantomData<T>);

impl<T> NonEmptyVecGate<T> {
    /// Create a new gate.  The phantom type `T` is inferred from context.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for NonEmptyVecGate<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Admit<NonEmptyCollectionLaw> for NonEmptyVecGate<T> {
    type Artifact = Vec<T>;

    fn admit(&self, v: &Vec<T>) -> Result<(), Refusal<NonEmptyCollectionLaw>> {
        if v.is_empty() {
            Err(Refusal::new("Collection must not be empty"))
        } else {
            Ok(())
        }
    }
}

/// Gate that enforces `ValidPathLaw`: the path must exist on the filesystem.
pub struct ValidPathGate;

impl Admit<ValidPathLaw> for ValidPathGate {
    type Artifact = PathBuf;

    fn admit(&self, p: &PathBuf) -> Result<(), Refusal<ValidPathLaw>> {
        if p.exists() {
            Ok(())
        } else {
            Err(Refusal::new(format!(
                "Path '{}' does not exist",
                p.display()
            )))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Runtime law system (mirrors knhk without importing it)
// ─────────────────────────────────────────────────────────────────────────────

/// A violation produced by a `RuntimeLaw`.  Mirrors `knhk::LawError`.
#[derive(Debug, Clone)]
pub struct LawViolation {
    /// Name of the law that was violated.
    pub law_name: String,
    /// Human-readable explanation.
    pub message: String,
}

impl fmt::Display for LawViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Law '{}' violated: {}", self.law_name, self.message)
    }
}

/// A runtime-dispatched law that can validate a filesystem path.
/// Mirrors `knhk::Law` but lives in this crate so we stay dependency-free.
pub trait RuntimeLaw: Send + Sync {
    /// Unique name of the law.
    fn name(&self) -> &str;
    /// Human-readable description of what the law enforces.
    fn description(&self) -> &str;
    /// Validate the given path, returning a `LawViolation` if the law is broken.
    fn validate_path(&self, path: &Path) -> Result<(), LawViolation>;
}

/// A registry of `RuntimeLaw` implementations.  Mirrors `knhk::Validator`.
pub struct LawRegistry {
    laws: Vec<Box<dyn RuntimeLaw>>,
}

impl LawRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { laws: Vec::new() }
    }

    /// Register a new law.
    pub fn register(&mut self, law: Box<dyn RuntimeLaw>) {
        self.laws.push(law);
    }

    /// Run every registered law against `path` and collect all violations.
    pub fn validate_all(&self, path: &Path) -> Vec<LawViolation> {
        let mut violations = Vec::new();
        for law in &self.laws {
            if let Err(v) = law.validate_path(path) {
                violations.push(v);
            }
        }
        violations
    }

    /// Return `true` iff no registered law is violated for `path`.
    pub fn is_compliant(&self, path: &Path) -> bool {
        self.validate_all(path).is_empty()
    }
}

impl Default for LawRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AndroidKeystoreGate
// ─────────────────────────────────────────────────────────────────────────────

/// Gate that enforces `AndroidKeystorePresentLaw`.
///
/// Admission logic (mirrors `knhk::AndroidKeystoreLaw`):
/// 1. Walk the project directory looking for a sub-directory whose name
///    contains `"Android"`.
/// 2. If found, also check for a `.keystore` or `.jks` file anywhere under
///    the project root.
/// 3. If an Android directory exists but no keystore is found, refuse.
/// 4. If no Android directory is found, the project is not an Android target
///    and admission is granted immediately.
pub struct AndroidKeystoreGate;

impl Admit<AndroidKeystorePresentLaw> for AndroidKeystoreGate {
    type Artifact = PathBuf;

    fn admit(&self, project_dir: &PathBuf) -> Result<(), Refusal<AndroidKeystorePresentLaw>> {
        let has_android = has_android_dir(project_dir);

        if has_android {
            if has_keystore_file(project_dir) {
                Ok(())
            } else {
                Err(Refusal::new(
                    "Android target detected but no .keystore or .jks file found",
                ))
            }
        } else {
            // Not an Android project — law does not apply.
            Ok(())
        }
    }
}

/// Return `true` if any immediate or nested sub-directory under `root` has
/// a name that contains `"Android"`.
fn has_android_dir(root: &Path) -> bool {
    walk_dirs(root, &|p: &Path| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.contains("Android"))
            .unwrap_or(false)
    })
}

/// Return `true` if any file under `root` has a `.keystore` or `.jks`
/// extension.
fn has_keystore_file(root: &Path) -> bool {
    walk_files(root, &|p: &Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "keystore" || e == "jks")
            .unwrap_or(false)
    })
}

/// Walk `root` recursively; return `true` as soon as `predicate` matches a
/// directory entry.
fn walk_dirs<F: Fn(&Path) -> bool>(root: &Path, predicate: &F) -> bool {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if predicate(&path) || walk_dirs(&path, predicate) {
                return true;
            }
        }
    }
    false
}

/// Walk `root` recursively; return `true` as soon as `predicate` matches a
/// file entry.
fn walk_files<F: Fn(&Path) -> bool>(root: &Path, predicate: &F) -> bool {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if predicate(&path) {
                return true;
            }
        } else if path.is_dir() && walk_files(&path, predicate) {
            return true;
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// GateChain — compose two gates sequentially over the same artifact type
// ─────────────────────────────────────────────────────────────────────────────

/// Compose two `Admit` gates that share the same `Artifact` type.
///
/// `admit` runs the first gate; if it passes, runs the second gate.  The first
/// failure short-circuits (the second gate is never called).
///
/// Errors from either gate are converted to `String` so that the two distinct
/// `Refusal<L1>` / `Refusal<L2>` types can be unified in the return value.
pub struct GateChain<A, B, L1, L2, T>
where
    L1: StaticLaw,
    L2: StaticLaw,
    A: Admit<L1, Artifact = T>,
    B: Admit<L2, Artifact = T>,
{
    first: A,
    second: B,
    _l1: PhantomData<L1>,
    _l2: PhantomData<L2>,
    _t: PhantomData<T>,
}

impl<A, B, L1, L2, T> GateChain<A, B, L1, L2, T>
where
    L1: StaticLaw,
    L2: StaticLaw,
    A: Admit<L1, Artifact = T>,
    B: Admit<L2, Artifact = T>,
{
    /// Construct a new chain from two gates.
    pub fn new(first: A, second: B) -> Self {
        Self {
            first,
            second,
            _l1: PhantomData,
            _l2: PhantomData,
            _t: PhantomData,
        }
    }

    /// Run both gates in order.  The first failure short-circuits the chain.
    pub fn admit(&self, artifact: &T) -> Result<(), String> {
        self.first.admit(artifact).map_err(|r| r.to_string())?;
        self.second.admit(artifact).map_err(|r| r.to_string())?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── Refusal ───────────────────────────────────────────────────────────────

    #[test]
    fn refusal_display_includes_law_name_and_message() {
        let r: Refusal<NonEmptyNameLaw> = Refusal::new("Name must not be empty or whitespace-only");
        let text = r.to_string();
        assert!(text.contains("NonEmptyName"), "missing law name in: {text}");
        assert!(
            text.contains("Name must not be empty"),
            "missing message in: {text}"
        );
    }

    #[test]
    fn refusal_law_name_field_matches_const() {
        let r: Refusal<ValidPathLaw> = Refusal::new("no such path");
        assert_eq!(r.law_name, ValidPathLaw::NAME);
    }

    #[test]
    fn refusal_debug_is_available() {
        let r: Refusal<NonEmptyCollectionLaw> = Refusal::new("empty");
        let _ = format!("{:?}", r);
    }

    // ── NonEmptyNameGate ──────────────────────────────────────────────────────

    #[test]
    fn non_empty_name_gate_admits_normal_name() {
        let gate = NonEmptyNameGate;
        assert!(gate.admit(&"Rocket".to_string()).is_ok());
    }

    #[test]
    fn non_empty_name_gate_admits_name_with_spaces() {
        let gate = NonEmptyNameGate;
        assert!(gate.admit(&"My Cool Project".to_string()).is_ok());
    }

    #[test]
    fn non_empty_name_gate_rejects_empty_string() {
        let gate = NonEmptyNameGate;
        let result = gate.admit(&String::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.law_name, NonEmptyNameLaw::NAME);
    }

    #[test]
    fn non_empty_name_gate_rejects_whitespace_only() {
        let gate = NonEmptyNameGate;
        let result = gate.admit(&"   ".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn non_empty_name_gate_rejects_tab_only() {
        let gate = NonEmptyNameGate;
        assert!(gate.admit(&"\t".to_string()).is_err());
    }

    // ── NonEmptyVecGate ───────────────────────────────────────────────────────

    #[test]
    fn non_empty_vec_gate_admits_non_empty_vec() {
        let gate: NonEmptyVecGate<i32> = NonEmptyVecGate::new();
        assert!(gate.admit(&vec![1, 2, 3]).is_ok());
    }

    #[test]
    fn non_empty_vec_gate_admits_single_element_vec() {
        let gate: NonEmptyVecGate<&str> = NonEmptyVecGate::new();
        assert!(gate.admit(&vec!["hello"]).is_ok());
    }

    #[test]
    fn non_empty_vec_gate_rejects_empty_vec() {
        let gate: NonEmptyVecGate<u8> = NonEmptyVecGate::new();
        let result = gate.admit(&vec![]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.law_name, NonEmptyCollectionLaw::NAME);
    }

    // ── ValidPathGate ─────────────────────────────────────────────────────────

    #[test]
    fn valid_path_gate_admits_existing_directory() {
        let dir = TempDir::new().unwrap();
        let gate = ValidPathGate;
        assert!(gate.admit(&dir.path().to_path_buf()).is_ok());
    }

    #[test]
    fn valid_path_gate_admits_existing_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, b"hello").unwrap();
        let gate = ValidPathGate;
        assert!(gate.admit(&file).is_ok());
    }

    #[test]
    fn valid_path_gate_rejects_nonexistent_path() {
        let gate = ValidPathGate;
        let bogus = PathBuf::from("/this/path/does/not/exist/ever");
        let result = gate.admit(&bogus);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.law_name, ValidPathLaw::NAME);
        assert!(err.message.contains("does not exist"));
    }

    // ── AndroidKeystoreGate ───────────────────────────────────────────────────

    #[test]
    fn android_keystore_gate_admits_non_android_project() {
        let dir = TempDir::new().unwrap();
        // No "Android" sub-dir — admission should pass.
        let gate = AndroidKeystoreGate;
        assert!(gate.admit(&dir.path().to_path_buf()).is_ok());
    }

    #[test]
    fn android_keystore_gate_admits_android_project_with_keystore() {
        let dir = TempDir::new().unwrap();
        let android_dir = dir.path().join("Android");
        fs::create_dir(&android_dir).unwrap();
        let keystore = dir.path().join("release.keystore");
        fs::write(&keystore, b"fake keystore").unwrap();

        let gate = AndroidKeystoreGate;
        assert!(gate.admit(&dir.path().to_path_buf()).is_ok());
    }

    #[test]
    fn android_keystore_gate_admits_android_project_with_jks() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("Android")).unwrap();
        fs::write(dir.path().join("app.jks"), b"fake jks").unwrap();

        let gate = AndroidKeystoreGate;
        assert!(gate.admit(&dir.path().to_path_buf()).is_ok());
    }

    #[test]
    fn android_keystore_gate_rejects_android_project_without_keystore() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("Android")).unwrap();
        // No keystore file added.

        let gate = AndroidKeystoreGate;
        let result = gate.admit(&dir.path().to_path_buf());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.law_name, AndroidKeystorePresentLaw::NAME);
        assert!(err.message.contains("keystore"));
    }

    // ── GateChain ─────────────────────────────────────────────────────────────

    /// Helper: a gate that always passes for `String`.
    struct AlwaysPassGate;
    struct AlwaysPassLaw;
    impl StaticLaw for AlwaysPassLaw {
        const NAME: &'static str = "AlwaysPass";
    }
    impl Admit<AlwaysPassLaw> for AlwaysPassGate {
        type Artifact = String;
        fn admit(&self, _: &String) -> Result<(), Refusal<AlwaysPassLaw>> {
            Ok(())
        }
    }

    #[test]
    fn gate_chain_passes_when_both_gates_pass() {
        // First gate: NonEmptyNameGate, Second gate: AlwaysPassGate
        let chain = GateChain::new(NonEmptyNameGate, AlwaysPassGate);
        assert!(chain.admit(&"Hello".to_string()).is_ok());
    }

    #[test]
    fn gate_chain_fails_when_first_gate_fails() {
        let chain = GateChain::new(NonEmptyNameGate, AlwaysPassGate);
        let result = chain.admit(&String::new());
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("NonEmptyName"),
            "error should name first law: {msg}"
        );
    }

    #[test]
    fn gate_chain_fails_when_second_gate_fails() {
        /// A gate that always refuses.
        struct AlwaysRefuseGate;
        struct AlwaysRefuseLaw;
        impl StaticLaw for AlwaysRefuseLaw {
            const NAME: &'static str = "AlwaysRefuse";
        }
        impl Admit<AlwaysRefuseLaw> for AlwaysRefuseGate {
            type Artifact = String;
            fn admit(&self, _: &String) -> Result<(), Refusal<AlwaysRefuseLaw>> {
                Err(Refusal::new("always refused"))
            }
        }

        let chain = GateChain::new(AlwaysPassGate, AlwaysRefuseGate);
        let result = chain.admit(&"valid".to_string());
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("AlwaysRefuse"),
            "error should name second law: {msg}"
        );
    }

    #[test]
    fn gate_chain_short_circuits_on_first_failure() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        // Count how many times the second gate is called.
        struct CountingGate {
            calls: Arc<AtomicUsize>,
        }
        struct CountingLaw;
        impl StaticLaw for CountingLaw {
            const NAME: &'static str = "Counting";
        }
        impl Admit<CountingLaw> for CountingGate {
            type Artifact = String;
            fn admit(&self, _: &String) -> Result<(), Refusal<CountingLaw>> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let calls = Arc::new(AtomicUsize::new(0));
        let chain = GateChain::new(
            NonEmptyNameGate,
            CountingGate {
                calls: calls.clone(),
            },
        );

        // Empty string triggers the first gate; second gate must NOT be called.
        let _ = chain.admit(&String::new());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "second gate was called despite first failure"
        );
    }

    // ── LawRegistry ───────────────────────────────────────────────────────────

    /// A `RuntimeLaw` that always passes.
    struct PassingRuntimeLaw;
    impl RuntimeLaw for PassingRuntimeLaw {
        fn name(&self) -> &str {
            "PassingLaw"
        }
        fn description(&self) -> &str {
            "Always passes"
        }
        fn validate_path(&self, _: &Path) -> Result<(), LawViolation> {
            Ok(())
        }
    }

    /// A `RuntimeLaw` that always fails.
    struct FailingRuntimeLaw {
        message: &'static str,
    }
    impl RuntimeLaw for FailingRuntimeLaw {
        fn name(&self) -> &str {
            "FailingLaw"
        }
        fn description(&self) -> &str {
            "Always fails"
        }
        fn validate_path(&self, _: &Path) -> Result<(), LawViolation> {
            Err(LawViolation {
                law_name: self.name().to_string(),
                message: self.message.to_string(),
            })
        }
    }

    #[test]
    fn law_registry_empty_is_compliant() {
        let registry = LawRegistry::new();
        let dir = TempDir::new().unwrap();
        assert!(registry.is_compliant(dir.path()));
    }

    #[test]
    fn law_registry_compliant_when_all_laws_pass() {
        let mut registry = LawRegistry::new();
        registry.register(Box::new(PassingRuntimeLaw));
        registry.register(Box::new(PassingRuntimeLaw));
        let dir = TempDir::new().unwrap();
        assert!(registry.is_compliant(dir.path()));
    }

    #[test]
    fn law_registry_not_compliant_when_any_law_fails() {
        let mut registry = LawRegistry::new();
        registry.register(Box::new(PassingRuntimeLaw));
        registry.register(Box::new(FailingRuntimeLaw { message: "bad" }));
        let dir = TempDir::new().unwrap();
        assert!(!registry.is_compliant(dir.path()));
    }

    #[test]
    fn law_registry_validate_all_returns_all_violations() {
        let mut registry = LawRegistry::new();
        registry.register(Box::new(FailingRuntimeLaw { message: "first" }));
        registry.register(Box::new(FailingRuntimeLaw { message: "second" }));
        let dir = TempDir::new().unwrap();
        let violations = registry.validate_all(dir.path());
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn law_registry_validate_all_empty_when_compliant() {
        let mut registry = LawRegistry::new();
        registry.register(Box::new(PassingRuntimeLaw));
        let dir = TempDir::new().unwrap();
        let violations = registry.validate_all(dir.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn law_violation_display_includes_law_name_and_message() {
        let v = LawViolation {
            law_name: "SomeLaw".to_string(),
            message: "something went wrong".to_string(),
        };
        let text = v.to_string();
        assert!(text.contains("SomeLaw"), "missing law name in: {text}");
        assert!(
            text.contains("something went wrong"),
            "missing message in: {text}"
        );
    }

    // ── StaticLaw NAME constants ───────────────────────────────────────────────

    #[test]
    fn static_law_names_are_correct() {
        assert_eq!(NonEmptyNameLaw::NAME, "NonEmptyName");
        assert_eq!(NonEmptyCollectionLaw::NAME, "NonEmptyCollection");
        assert_eq!(ValidPathLaw::NAME, "ValidPath");
        assert_eq!(AndroidKeystorePresentLaw::NAME, "AndroidKeystorePresent");
    }
}
