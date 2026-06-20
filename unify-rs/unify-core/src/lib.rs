//! `unify-core` — Core traits and types for the unify-rs workspace.
//!
//! This crate defines the five core abstractions that every other unify-rs
//! crate depends on:
//!
//! 1. [`StaticLaw`] — a compile-time (zero-cost, `const`) law gate.
//! 2. [`DynamicLaw`] — a runtime-dispatchable law compatible with `knhk::Law`.
//! 3. [`Admit`] — an admission gate that checks an artifact against a static law.
//! 4. [`Witness`] — a zero-sized compile-time marker carrying standard metadata.
//! 5. [`Classify`] — maps `self` to a `(namespace, noun, verb)` triple for CLI dispatch.
//! 6. [`Codegen`] — generates code/text from `self`.
//!
//! The `Evidence<T, State, W>` typestate container threads an artifact through
//! the `Raw → Parsed → Admitted → Exported` lifecycle in a way that is
//! statically verified by the type system.

use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

// ─── Law violation ────────────────────────────────────────────────────────────

/// A violation produced by either a [`DynamicLaw`] or a [`StaticLaw`] gate.
///
/// This mirrors `knhk::LawError` so that the two ecosystems can share
/// violation values without a direct dependency.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Law '{law_name}' violated: {message}")]
pub struct LawViolation {
    /// The name of the law that was violated.
    pub law_name: String,
    /// A human-readable explanation of the violation.
    pub message: String,
}

impl LawViolation {
    /// Construct a new `LawViolation`.
    pub fn new(law_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            law_name: law_name.into(),
            message: message.into(),
        }
    }
}

// ─── StaticLaw ────────────────────────────────────────────────────────────────

/// A **compile-time** law.
///
/// `StaticLaw` provides `const` metadata, enabling zero-cost type-level gates.
/// Every implementor is a marker struct; no allocation or vtable is needed.
///
/// # Example
/// ```
/// use unify_core::{StaticLaw, NonEmptyNameLaw};
///
/// fn check_law_name<L: StaticLaw>() -> &'static str {
///     L::NAME
/// }
///
/// assert_eq!(check_law_name::<NonEmptyNameLaw>(), "NonEmptyName");
/// ```
pub trait StaticLaw {
    /// Unique machine-readable name for the law.
    const NAME: &'static str;
    /// Human-readable explanation of what the law enforces.
    const DESCRIPTION: &'static str;
}

// ─── DynamicLaw ───────────────────────────────────────────────────────────────

/// A **runtime-dispatchable** law.
///
/// `DynamicLaw` mirrors `knhk::Law` so that laws implemented in `knhk` can be
/// wrapped and registered in a [`DynamicLawRegistry`] without a direct
/// dependency.
pub trait DynamicLaw: Send + Sync {
    /// Unique machine-readable name for this law instance.
    fn name(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// Validate the given path, returning a [`LawViolation`] on failure.
    fn validate_path(&self, path: &Path) -> Result<(), LawViolation>;
}

// ─── Admit ────────────────────────────────────────────────────────────────────

/// An **admission gate** that checks whether an `Artifact` satisfies law `L`.
///
/// Each implementor encodes the business rules for a specific `(Law, Artifact)`
/// combination.  The `Refusal` associated type carries typed rejection detail.
///
/// # Example
/// ```
/// use unify_core::{Admit, NonEmptyNameLaw, Refusal};
///
/// struct NamedThing { pub name: String }
///
/// struct NonEmptyNameGate;
///
/// impl Admit<NonEmptyNameLaw> for NonEmptyNameGate {
///     type Artifact = NamedThing;
///     type Refusal  = Refusal<NonEmptyNameLaw>;
///
///     fn admit(&self, artifact: &NamedThing) -> Result<(), Refusal<NonEmptyNameLaw>> {
///         if artifact.name.is_empty() {
///             Err(Refusal::new("name must not be empty"))
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Admit<L: StaticLaw> {
    /// The artifact type being checked.
    type Artifact;
    /// A displayable rejection reason.
    type Refusal: fmt::Display;

    /// Check `artifact` against law `L`.
    ///
    /// Returns `Ok(())` if the artifact satisfies the law, or `Err(Self::Refusal)`
    /// with a description of the violation.
    fn admit(&self, artifact: &Self::Artifact) -> Result<(), Self::Refusal>;
}

// ─── Witness ──────────────────────────────────────────────────────────────────

/// A **zero-sized compile-time marker** that attests to a standard or citation.
///
/// `Witness` implementors are unit structs that carry `const` metadata about
/// the standard they represent.  The `Default + Copy + 'static` bounds ensure
/// they can be used as pure type-level tokens with zero runtime cost.
///
/// # Example
/// ```
/// use unify_core::Witness;
///
/// #[derive(Default, Clone, Copy)]
/// struct Rfc3986;
///
/// impl Witness for Rfc3986 {
///     const STANDARD: &'static str = "RFC 3986";
///     const CITATION: &'static str = "https://www.rfc-editor.org/rfc/rfc3986";
/// }
///
/// assert_eq!(Rfc3986::STANDARD, "RFC 3986");
/// ```
pub trait Witness: Default + Copy + 'static {
    /// Name of the standard this witness represents.
    const STANDARD: &'static str;
    /// URL or identifier for the normative reference.
    const CITATION: &'static str;
}

// ─── Classify ─────────────────────────────────────────────────────────────────

/// Maps `self` to a `(namespace, noun, verb)` triple for CLI dispatch.
///
/// Namespace → noun → verb mirrors the three-level hierarchy used in the
/// unify CLI: e.g. `unify schema validate` → `("schema", "validate", …)`.
///
/// # Example
/// ```
/// use unify_core::Classify;
///
/// struct SchemaValidateCommand;
///
/// impl Classify for SchemaValidateCommand {
///     fn namespace(&self) -> &'static str { "schema" }
///     fn noun(&self)      -> &'static str { "schema" }
///     fn verb(&self)      -> &'static str { "validate" }
/// }
/// ```
pub trait Classify {
    /// Top-level namespace (e.g. `"schema"`, `"project"`).
    fn namespace(&self) -> &'static str;
    /// Entity being acted on.
    fn noun(&self) -> &'static str;
    /// Action being performed.
    fn verb(&self) -> &'static str;
}

// ─── Codegen ──────────────────────────────────────────────────────────────────

/// Generate code or text output from `self`.
///
/// The contract is purely that `generate` returns a `String`; it is up to
/// implementors to decide what encoding that string uses.
///
/// # Example
/// ```
/// use unify_core::Codegen;
///
/// struct HelloTemplate { name: String }
///
/// impl Codegen for HelloTemplate {
///     fn generate(&self) -> String {
///         format!("Hello, {}!", self.name)
///     }
/// }
///
/// let t = HelloTemplate { name: "world".into() };
/// assert_eq!(t.generate(), "Hello, world!");
/// ```
pub trait Codegen {
    /// Produce a string representation of `self`.
    fn generate(&self) -> String;
}

// ─── Refusal<L> ───────────────────────────────────────────────────────────────

/// A typed refusal for static law `L`.
///
/// Carrying the law as a type parameter means you can never mix up refusals
/// from different laws at the type level.
#[derive(Debug)]
pub struct Refusal<L: StaticLaw> {
    /// Human-readable rejection message.
    pub message: String,
    _law: PhantomData<L>,
}

impl<L: StaticLaw> Refusal<L> {
    /// Construct a new `Refusal` for law `L`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            _law: PhantomData,
        }
    }

    /// The name of the violated law.
    pub fn law_name(&self) -> &'static str {
        L::NAME
    }

    /// Convert to a [`LawViolation`] for interop with `DynamicLaw` consumers.
    pub fn into_violation(self) -> LawViolation {
        LawViolation::new(L::NAME, self.message)
    }
}

impl<L: StaticLaw> fmt::Display for Refusal<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", L::NAME, self.message)
    }
}

impl<L: StaticLaw + fmt::Debug> std::error::Error for Refusal<L> {}

// ─── Lifecycle state markers ──────────────────────────────────────────────────

/// Artifact has just arrived — no structure has been imposed yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Raw;

/// Artifact has been syntactically parsed; structure is known.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Parsed;

/// Artifact has passed all admission gates; semantics are verified.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Admitted;

/// Artifact has been rendered to an output string.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Exported;

// ─── Evidence<T, State, W> ────────────────────────────────────────────────────

/// A typestate container that carries an artifact `T` through the
/// `Raw → Parsed → Admitted → Exported` lifecycle.
///
/// The phantom type parameters `S` (state) and `W` (witness) make illegal
/// transitions unrepresentable at compile time.  You cannot call `.admit()`
/// on a `Raw` artifact, and you cannot call `.export()` on a `Parsed` one.
///
/// # Example
/// ```
/// use unify_core::{Evidence, Raw, Witness, Admit, NonEmptyNameLaw, Refusal};
///
/// #[derive(Default, Clone, Copy)]
/// struct MyWitness;
/// impl Witness for MyWitness {
///     const STANDARD: &'static str = "my-std";
///     const CITATION: &'static str = "https://example.com";
/// }
///
/// struct NamedArtifact { name: String }
/// struct NameGate;
///
/// impl Admit<NonEmptyNameLaw> for NameGate {
///     type Artifact = NamedArtifact;
///     type Refusal  = Refusal<NonEmptyNameLaw>;
///     fn admit(&self, a: &NamedArtifact) -> Result<(), Refusal<NonEmptyNameLaw>> {
///         if a.name.is_empty() { Err(Refusal::new("empty")) } else { Ok(()) }
///     }
/// }
///
/// let ev = Evidence::<_, Raw, MyWitness>::new(NamedArtifact { name: "foo".into() });
/// let ev = ev.map_parsed(|a| a);
/// let ev = ev.admit::<NonEmptyNameLaw, _>(&NameGate).unwrap();
/// let (output, _ev) = ev.export(|a| a.name.clone());
/// assert_eq!(output, "foo");
/// ```
#[derive(Debug)]
pub struct Evidence<T, S, W: Witness> {
    /// The wrapped artifact.
    pub inner: T,
    _state: PhantomData<S>,
    _witness: PhantomData<W>,
}

impl<T, W: Witness> Evidence<T, Raw, W> {
    /// Wrap a raw artifact in the Evidence container.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _state: PhantomData,
            _witness: PhantomData,
        }
    }

    /// Transform the inner artifact and advance state to [`Parsed`].
    pub fn map_parsed<U, F>(self, f: F) -> Evidence<U, Parsed, W>
    where
        F: FnOnce(T) -> U,
    {
        Evidence {
            inner: f(self.inner),
            _state: PhantomData,
            _witness: PhantomData,
        }
    }
}

impl<T, W: Witness> Evidence<T, Parsed, W> {
    /// Run the admission gate `G` for law `L`.
    ///
    /// Returns `Evidence<T, Admitted, W>` if the gate accepts, otherwise returns
    /// `Err(G::Refusal)`.
    pub fn admit<L, G>(self, gate: &G) -> Result<Evidence<T, Admitted, W>, G::Refusal>
    where
        L: StaticLaw,
        G: Admit<L, Artifact = T>,
    {
        gate.admit(&self.inner)?;
        Ok(Evidence {
            inner: self.inner,
            _state: PhantomData,
            _witness: PhantomData,
        })
    }

    /// Access the inner artifact immutably.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T, W: Witness> Evidence<T, Admitted, W> {
    /// Render the artifact to a string, advancing state to [`Exported`].
    ///
    /// Returns the rendered string *and* the artifact in its exported state so
    /// callers can keep a handle on both.
    pub fn export<F>(self, f: F) -> (String, Evidence<T, Exported, W>)
    where
        F: FnOnce(&T) -> String,
    {
        let output = f(&self.inner);
        let exported = Evidence {
            inner: self.inner,
            _state: PhantomData,
            _witness: PhantomData,
        };
        (output, exported)
    }

    /// Access the inner artifact immutably.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T, W: Witness> Evidence<T, Exported, W> {
    /// Access the inner artifact after export (read-only).
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

// ─── DynamicLawRegistry ───────────────────────────────────────────────────────

/// A registry for runtime laws that mirrors the `knhk::Validator` pattern.
///
/// Laws implementing [`DynamicLaw`] (or wrapping `knhk::Law`) can be registered
/// here.  The registry then validates a path against all registered laws and
/// collects any [`LawViolation`]s.
#[derive(Default)]
pub struct DynamicLawRegistry {
    laws: Vec<Box<dyn DynamicLaw>>,
}

impl DynamicLawRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { laws: Vec::new() }
    }

    /// Register a new law.
    pub fn register(&mut self, law: Box<dyn DynamicLaw>) {
        self.laws.push(law);
    }

    /// Validate `path` against every registered law and collect all violations.
    pub fn validate_all(&self, path: &Path) -> Vec<LawViolation> {
        self.laws
            .iter()
            .filter_map(|law| law.validate_path(path).err())
            .collect()
    }

    /// Return `true` iff `path` satisfies all registered laws (empty violations list).
    pub fn is_compliant(&self, path: &Path) -> bool {
        self.validate_all(path).is_empty()
    }

    /// Return the number of registered laws.
    pub fn len(&self) -> usize {
        self.laws.len()
    }

    /// Return `true` if no laws are registered.
    pub fn is_empty(&self) -> bool {
        self.laws.is_empty()
    }
}

// ─── KnhkAdapter ──────────────────────────────────────────────────────────────

/// Adapter that lifts a [`DynamicLaw`] into the shape expected by consumers
/// of the `knhk` crate's `Law` trait, **without** importing `knhk` as a
/// dependency here.
///
/// Use this when you have a `Box<dyn DynamicLaw>` and need to pass it to a
/// `knhk::Validator` that expects `Box<dyn knhk::Law>`.  The adapter is a
/// thin newtype that delegates every call.
pub struct KnhkAdapter<D: DynamicLaw + ?Sized>(pub Box<D>);

impl<D: DynamicLaw> KnhkAdapter<D> {
    /// Wrap `law` in the adapter.
    pub fn new(law: D) -> Self {
        Self(Box::new(law))
    }
}

// ─── Built-in StaticLaws ──────────────────────────────────────────────────────

/// Law: an artifact name must not be empty.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NonEmptyNameLaw;

impl StaticLaw for NonEmptyNameLaw {
    const NAME: &'static str = "NonEmptyName";
    const DESCRIPTION: &'static str = "Artifact name must not be empty";
}

/// Law: a collection must contain at least one element.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NonEmptyCollectionLaw;

impl StaticLaw for NonEmptyCollectionLaw {
    const NAME: &'static str = "NonEmptyCollection";
    const DESCRIPTION: &'static str = "Collection must have at least one element";
}

// ─── Built-in DynamicLaw wrappers for the built-in StaticLaws ────────────────

/// Runtime law wrapping [`NonEmptyNameLaw`] — validates a file's stem.
pub struct NonEmptyNameDynamicLaw;

impl DynamicLaw for NonEmptyNameDynamicLaw {
    fn name(&self) -> &str {
        NonEmptyNameLaw::NAME
    }

    fn description(&self) -> &str {
        NonEmptyNameLaw::DESCRIPTION
    }

    fn validate_path(&self, path: &Path) -> Result<(), LawViolation> {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem.is_empty() {
            Err(LawViolation::new(
                NonEmptyNameLaw::NAME,
                "file stem is empty",
            ))
        } else {
            Ok(())
        }
    }
}

// ─── Utility: static-to-dynamic adapter ──────────────────────────────────────

/// Type alias for path-validation closure.
pub type LawValidationFn = Box<dyn Fn(&Path) -> Result<(), LawViolation> + Send + Sync>;

/// Wraps a [`StaticLaw`] value together with a path-validation closure,
/// exposing it as a [`DynamicLaw`].  Useful for quick registrations in tests.
pub struct StaticLawAdapter<L: StaticLaw> {
    _law: PhantomData<L>,
    validate_fn: LawValidationFn,
}

impl<L: StaticLaw> StaticLawAdapter<L> {
    /// Construct an adapter with the given validation closure.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Path) -> Result<(), LawViolation> + Send + Sync + 'static,
    {
        Self {
            _law: PhantomData,
            validate_fn: Box::new(f),
        }
    }
}

impl<L: StaticLaw + Send + Sync> DynamicLaw for StaticLawAdapter<L> {
    fn name(&self) -> &str {
        L::NAME
    }

    fn description(&self) -> &str {
        L::DESCRIPTION
    }

    fn validate_path(&self, path: &Path) -> Result<(), LawViolation> {
        (self.validate_fn)(path)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── Helper witness ────────────────────────────────────────────────────────

    #[derive(Debug, Default, Clone, Copy)]
    struct TestWitness;

    impl Witness for TestWitness {
        const STANDARD: &'static str = "test-standard-1.0";
        const CITATION: &'static str = "https://example.com/test-standard";
    }

    // ── Helper artifact ───────────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct NamedArtifact {
        name: String,
        items: Vec<String>,
    }

    // ── Helper gate: NonEmptyName ─────────────────────────────────────────────

    struct NameGate;

    impl Admit<NonEmptyNameLaw> for NameGate {
        type Artifact = NamedArtifact;
        type Refusal = Refusal<NonEmptyNameLaw>;

        fn admit(&self, artifact: &NamedArtifact) -> Result<(), Refusal<NonEmptyNameLaw>> {
            if artifact.name.is_empty() {
                Err(Refusal::new("name must not be empty"))
            } else {
                Ok(())
            }
        }
    }

    // ── Helper gate: NonEmptyCollection ──────────────────────────────────────

    struct CollectionGate;

    impl Admit<NonEmptyCollectionLaw> for CollectionGate {
        type Artifact = NamedArtifact;
        type Refusal = Refusal<NonEmptyCollectionLaw>;

        fn admit(&self, artifact: &NamedArtifact) -> Result<(), Refusal<NonEmptyCollectionLaw>> {
            if artifact.items.is_empty() {
                Err(Refusal::new("collection must not be empty"))
            } else {
                Ok(())
            }
        }
    }

    // ── StaticLaw tests ───────────────────────────────────────────────────────

    #[test]
    fn static_law_non_empty_name_constants() {
        assert_eq!(NonEmptyNameLaw::NAME, "NonEmptyName");
        assert_eq!(
            NonEmptyNameLaw::DESCRIPTION,
            "Artifact name must not be empty"
        );
    }

    #[test]
    fn static_law_non_empty_collection_constants() {
        assert_eq!(NonEmptyCollectionLaw::NAME, "NonEmptyCollection");
        assert_eq!(
            NonEmptyCollectionLaw::DESCRIPTION,
            "Collection must have at least one element"
        );
    }

    // ── Witness tests ─────────────────────────────────────────────────────────

    #[test]
    fn witness_const_values() {
        assert_eq!(TestWitness::STANDARD, "test-standard-1.0");
        assert_eq!(TestWitness::CITATION, "https://example.com/test-standard");
    }

    #[test]
    fn witness_is_copy_and_default() {
        let w = TestWitness::default();
        let _w2 = w; // Copy — original still usable
        let _w3 = w;
    }

    // ── LawViolation tests ────────────────────────────────────────────────────

    #[test]
    fn law_violation_display() {
        let v = LawViolation::new("MyLaw", "something went wrong");
        assert_eq!(format!("{v}"), "Law 'MyLaw' violated: something went wrong");
    }

    #[test]
    fn law_violation_equality() {
        let a = LawViolation::new("L", "m");
        let b = LawViolation::new("L", "m");
        assert_eq!(a, b);
    }

    // ── Refusal tests ─────────────────────────────────────────────────────────

    #[test]
    fn refusal_carries_law_name() {
        let r: Refusal<NonEmptyNameLaw> = Refusal::new("empty name");
        assert_eq!(r.law_name(), "NonEmptyName");
    }

    #[test]
    fn refusal_display_includes_law_name() {
        let r: Refusal<NonEmptyNameLaw> = Refusal::new("oops");
        assert!(format!("{r}").contains("NonEmptyName"));
        assert!(format!("{r}").contains("oops"));
    }

    #[test]
    fn refusal_into_violation_round_trips() {
        let r: Refusal<NonEmptyCollectionLaw> = Refusal::new("empty!");
        let v = r.into_violation();
        assert_eq!(v.law_name, NonEmptyCollectionLaw::NAME);
        assert_eq!(v.message, "empty!");
    }

    // ── Evidence lifecycle tests ──────────────────────────────────────────────

    fn make_raw(name: &str, items: Vec<&str>) -> Evidence<NamedArtifact, Raw, TestWitness> {
        Evidence::new(NamedArtifact {
            name: name.to_string(),
            items: items.iter().map(|s| s.to_string()).collect(),
        })
    }

    #[test]
    fn evidence_raw_to_parsed() {
        let raw = make_raw("hello", vec!["a"]);
        let parsed = raw.map_parsed(|a| a);
        assert_eq!(parsed.inner().name, "hello");
    }

    #[test]
    fn evidence_map_parsed_transforms_inner() {
        let raw = make_raw("hello", vec!["a"]);
        let parsed = raw.map_parsed(|mut a| {
            a.name = a.name.to_uppercase();
            a
        });
        assert_eq!(parsed.inner().name, "HELLO");
    }

    #[test]
    fn evidence_admit_succeeds_for_valid_artifact() {
        let parsed = make_raw("my-artifact", vec!["item"]).map_parsed(|a| a);
        let admitted = parsed.admit::<NonEmptyNameLaw, _>(&NameGate);
        assert!(admitted.is_ok());
    }

    #[test]
    fn evidence_admit_fails_for_empty_name() {
        let parsed = make_raw("", vec!["item"]).map_parsed(|a| a);
        let result = parsed.admit::<NonEmptyNameLaw, _>(&NameGate);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{err}").contains("NonEmptyName"));
    }

    #[test]
    fn evidence_admit_collection_fails_for_empty_vec() {
        let parsed = make_raw("ok-name", vec![]).map_parsed(|a| a);
        let result = parsed.admit::<NonEmptyCollectionLaw, _>(&CollectionGate);
        assert!(result.is_err());
    }

    #[test]
    fn evidence_export_produces_string() {
        let admitted = make_raw("foo", vec!["x"])
            .map_parsed(|a| a)
            .admit::<NonEmptyNameLaw, _>(&NameGate)
            .unwrap();
        let (output, _ev) = admitted.export(|a| format!("name={}", a.name));
        assert_eq!(output, "name=foo");
    }

    #[test]
    fn evidence_export_returns_exported_state_with_inner() {
        let admitted = make_raw("bar", vec!["y"])
            .map_parsed(|a| a)
            .admit::<NonEmptyNameLaw, _>(&NameGate)
            .unwrap();
        let (_, exported) = admitted.export(|a| a.name.clone());
        assert_eq!(exported.inner().name, "bar");
    }

    #[test]
    fn evidence_full_lifecycle() {
        // Raw → Parsed → Admitted → Exported
        let raw: Evidence<_, Raw, TestWitness> = Evidence::new(NamedArtifact {
            name: "lifecycle".into(),
            items: vec!["one".into(), "two".into()],
        });

        let parsed = raw.map_parsed(|mut a| {
            a.name = a.name.to_ascii_uppercase();
            a
        });
        assert_eq!(parsed.inner().name, "LIFECYCLE");

        let admitted = parsed
            .admit::<NonEmptyNameLaw, _>(&NameGate)
            .expect("admission should succeed");
        assert_eq!(admitted.inner().items.len(), 2);

        let (output, exported) = admitted.export(|a| format!("{}: {}", a.name, a.items.join(", ")));
        assert_eq!(output, "LIFECYCLE: one, two");
        assert_eq!(exported.inner().name, "LIFECYCLE");
    }

    // ── DynamicLawRegistry tests ──────────────────────────────────────────────

    fn always_pass() -> Box<dyn DynamicLaw> {
        Box::new(StaticLawAdapter::<NonEmptyNameLaw>::new(|_path| Ok(())))
    }

    fn always_fail(msg: &'static str) -> Box<dyn DynamicLaw> {
        Box::new(StaticLawAdapter::<NonEmptyCollectionLaw>::new(
            move |_path| Err(LawViolation::new(NonEmptyCollectionLaw::NAME, msg)),
        ))
    }

    #[test]
    fn registry_new_is_empty() {
        let reg = DynamicLawRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn registry_register_increments_len() {
        let mut reg = DynamicLawRegistry::new();
        reg.register(always_pass());
        assert_eq!(reg.len(), 1);
        reg.register(always_fail("x"));
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn registry_is_compliant_with_no_laws() {
        let reg = DynamicLawRegistry::new();
        assert!(reg.is_compliant(Path::new("/")));
    }

    #[test]
    fn registry_is_compliant_when_all_pass() {
        let mut reg = DynamicLawRegistry::new();
        reg.register(always_pass());
        reg.register(always_pass());
        assert!(reg.is_compliant(Path::new("/")));
    }

    #[test]
    fn registry_validate_all_returns_violations() {
        let mut reg = DynamicLawRegistry::new();
        reg.register(always_pass());
        reg.register(always_fail("first failure"));
        reg.register(always_fail("second failure"));

        let violations = reg.validate_all(Path::new("/"));
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn registry_is_not_compliant_when_any_fail() {
        let mut reg = DynamicLawRegistry::new();
        reg.register(always_pass());
        reg.register(always_fail("oops"));
        assert!(!reg.is_compliant(Path::new("/")));
    }

    #[test]
    fn registry_violation_messages_are_preserved() {
        let mut reg = DynamicLawRegistry::new();
        reg.register(always_fail("my specific message"));
        let violations = reg.validate_all(Path::new("/tmp"));
        assert_eq!(violations[0].message, "my specific message");
    }

    // ── DynamicLaw built-in wrapper test ─────────────────────────────────────

    #[test]
    fn non_empty_name_dynamic_law_passes_for_named_file() {
        let law = NonEmptyNameDynamicLaw;
        assert_eq!(law.name(), NonEmptyNameLaw::NAME);
        let result = law.validate_path(Path::new("/some/path/file.txt"));
        assert!(result.is_ok());
    }

    #[test]
    fn non_empty_name_dynamic_law_fails_for_dotfile() {
        let law = NonEmptyNameDynamicLaw;
        // A path like "/dir/.hidden" has stem ".hidden" — non-empty; use "/" which has no stem.
        let result = law.validate_path(Path::new("/"));
        // "/" has no file stem → violation
        assert!(result.is_err());
    }

    // ── Classify trait test ───────────────────────────────────────────────────

    #[test]
    fn classify_dispatch_returns_correct_triple() {
        struct ValidateCmd;
        impl Classify for ValidateCmd {
            fn namespace(&self) -> &'static str {
                let ns = "schema";
                ns
            }
            fn noun(&self) -> &'static str {
                let n = "schema";
                n
            }
            fn verb(&self) -> &'static str {
                let v = "validate";
                v
            }
        }

        let cmd = ValidateCmd;
        assert_eq!(cmd.namespace(), "schema");
        assert_eq!(cmd.noun(), "schema");
        assert_eq!(cmd.verb(), "validate");
    }

    // ── Codegen trait test ────────────────────────────────────────────────────

    #[test]
    fn codegen_generates_output() {
        struct Template {
            greeting: String,
        }
        impl Codegen for Template {
            fn generate(&self) -> String {
                format!(
                    "// Generated\nconst GREETING: &str = \"{}\";",
                    self.greeting
                )
            }
        }

        let t = Template {
            greeting: "hello".into(),
        };
        let code = t.generate();
        assert!(code.contains("Generated"));
        assert!(code.contains("hello"));
    }

    // ── StaticLawAdapter / KnhkAdapter tests ─────────────────────────────────

    #[test]
    fn static_law_adapter_delegates_name_and_description() {
        let adapter = StaticLawAdapter::<NonEmptyNameLaw>::new(|_| Ok(()));
        assert_eq!(adapter.name(), NonEmptyNameLaw::NAME);
        assert_eq!(adapter.description(), NonEmptyNameLaw::DESCRIPTION);
    }

    #[test]
    fn static_law_adapter_closure_called_on_validate() {
        let path = PathBuf::from("/some/path");
        let adapter = StaticLawAdapter::<NonEmptyNameLaw>::new(|p| {
            if p == Path::new("/some/path") {
                Err(LawViolation::new("NonEmptyName", "matched"))
            } else {
                Ok(())
            }
        });
        let result = adapter.validate_path(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "matched");
    }
}
