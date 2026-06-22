# Cross-Crate Innovation Synthesis: Iteration 2

**Loop Focus:** `lsp-max`, `lsp-types-max`, `cargo-cicd`, `affidavit`
**Date:** June 2026

## Iteration 2 Objective
Draft the exact Rust typestate prototypes to enforce SHACL bounds within `lsp-types-max`, and update the `lsp-max` RulePackServer trait to strictly require an `affidavit` Cryptographic Signer for all semantic mutations.

---

### 1. `lsp-types-max`: SHACL-Bound ZSTs
To prevent an agent from creating a "Private Ontology" or submitting morphology outside the target envelope, `lsp-types-max` must map its structs natively to the source SHACL shapes.

**Implementation Prototype:**
```rust
use std::marker::PhantomData;

// Zero-Sized Types representing the verification phase of an LSP payload.
pub struct RawPayload;
pub struct VerifiedPayload;

// The core LSP type wrapped in an Evidence bound.
pub struct Evidence<T, State> {
    pub inner: T,
    _state: PhantomData<State>,
}

// A specific SHACL-bound payload type for a Bipedal Torso dimension edit.
pub struct TorsoDimensionPayload {
    pub scale_y: f64,
}

// The Admittance Gate
impl Evidence<TorsoDimensionPayload, RawPayload> {
    pub fn admit(self, validator: &crate::ShaclValidator) -> Result<Evidence<TorsoDimensionPayload, VerifiedPayload>, &'static str> {
        // Enforcing the 110_bipedal_metric_envelope_law.ttl bounds
        if self.inner.scale_y > 2.0 || self.inner.scale_y < 1.0 {
            return Err("VIS200: Morphology Convergence Violation");
        }
        Ok(Evidence {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}
```

---

### 2. `lsp-max`: Forcing Affidavit Signatures
`lsp-max` must prevent the LSP server from silently mutating the workspace. Any `textDocument/didChange` or `workspace/executeCommand` that alters semantic meaning must be cryptographically sealed.

**Implementation Prototype:**
```rust
use affidavit::AffidavitSigner;
use lsp_types_max::{Evidence, VerifiedPayload};

pub trait RulePackServer {
    /// Every semantic mutation request must supply an `AffidavitSigner`.
    /// The return type requires `Evidence` of a `VerifiedPayload`.
    fn apply_semantic_mutation(
        &self, 
        payload: Evidence<TargetPayload, VerifiedPayload>, 
        signer: &mut AffidavitSigner
    ) -> Result<affidavit::Blake3Receipt, lsp_server::ResponseError>;
}
```

### 3. Implications for `cargo-cicd`
When `cargo-cicd` runs its headless integration tests, it will dynamically construct a `ShaclValidator` and an `AffidavitSigner`. If an AI agent attempts to bypass the `admit()` function and manually constructs a `VerifiedPayload`, the Rust compiler will throw an E0308 type mismatch. The compiler natively becomes the Jidoka line.

## Next Steps for Iteration 3
1. Implement the `OCEL 2.0` process miner interface inside `affidavit`.
2. Map the `cargo-cicd` exit codes to strictly emit `BLOCKED`, `CLAIM_HOLD`, or `ADMITTED` string tokens for direct CLI output ingestion.
