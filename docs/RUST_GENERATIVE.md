# Rust Generative Infrastructure & Architectural Closure

This document describes the Rust generative infrastructure, the integration of the **Ostar** semantic ontology, the **ggen** code manufacturer, and how **rocket-cmd** leverages them to enforce strict architectural closure across the Rocket Craft ecosystem.

## 1. Overview

The infrastructure relies on a decoupled, declarative architecture governed by the **Chatman Equation: A = μ(O)**. 
- **O (Ontology)**: The semantic laws defined in Ostar.
- **μ (Projection)**: The deterministic code generation engine, `ggen`.
- **A (Agent/System Behavior)**: The resulting implemented Rust code.

This pipeline guarantees that the executable code physically enforces the semantic constraints defined in the ontology, eliminating drift between system design and physical implementation.

---

## 2. The Ostar Semantic Ontology (The Brain)

**Ostar** represents the semantic layer and the single source of truth for the system's capabilities. 

### Semantic Laws
Capabilities are defined in ontology files (e.g., using the IES 4D pattern via RDF/Turtle) mapping states, events, and consequences. A law strictly defines:
- **State**: Valid configurations of a domain object or system.
- **Event**: Recognized external signals that can trigger state transitions.
- **Consequence**: The ensured deterministic outcome of admitting a lawful event.

### Governance
The **ostar-governor** ensures that any proposed capability is formally defined as a law in the ontology before any implementation code is generated. This creates a hard gate for architectural consistency and prevents undocumented "shadow features."

---

## 3. The `ggen` Integration (The Muscle)

**`ggen`** is the code manufacturer that reads the Ostar semantic laws and translates them into high-performance Rust abstractions.

### Zero-Cost Typestates
`ggen` manufactures a **Zero-Cost Typestate Kernel** for each capability. It generates Rust structs to represent operational phases and traits to represent the laws, strictly consuming `self` on transition.
- **Validation at Compile-Time:** Moving state validation from runtime to compile-time ensures illegal states are unrepresentable.
- **No State Aliasing:** Linear consumption of ownership prevents state aliasing.
- **Generated Scaffolding:** `ggen` scaffolds the entire structural foundation (the `Machine<Law, Phase>` pattern), allowing developers to focus solely on implementing the specific boundary logic.

---

## 4. `rocket-cmd` and Architectural Closure

The **`rocket-cmd`** CLI tool orchestrates the pipeline to enforce **architectural closure**, meaning that the final compiled code represents exactly what is defined in the ontology—no more, no less.

### The Generative Pipeline Execution
When a developer runs `rocket-cmd` commands (such as a generative sync):
1.  **Read Ontology:** The pipeline extracts the latest semantic definitions from Ostar.
2.  **Manufacture Code:** `rocket-cmd` drives `ggen` to manufacture the corresponding Rust typestates, interfaces, and validation traits directly into the target crates (like `rocket-sdk`).
3.  **Boundary Enforcement:** The CLI ensures that the newly generated abstractions are seamlessly integrated and that any manual boundary implementations (the "bounds") correctly fulfill the generated generic traits.

### Diagnostic Verification
To prevent partial implementations, `rocket-cmd` works with the **ostar-doctor** diagnostic layer to verify "law closure." It proves that for every semantic state transition defined by the governor, `ggen` has scaffolded the required Rust mechanism, and the target codebase correctly uses those typestates without bypassing the generated logic.

### Cryptographic Auditing
As part of the closure process, `rocket-cmd` oversees the integration of **ostar-auditor** patterns. This ensures that the generated state transitions emit standards-compliant, unforgeable cryptographic receipts (e.g., using BLAKE3). This provides runtime proof that the compiled typestate kernel executed exactly as the semantic laws mandated.
