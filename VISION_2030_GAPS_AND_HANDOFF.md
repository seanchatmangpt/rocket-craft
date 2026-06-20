# Vision 2030: Architectural Gaps & Implementation Handoff

**Target State:** The Rocket Craft 2030 Semantic AAA Ecosystem.
**Current State:** Stabilized local pipelines, but lacking the definitive `A = μ(O)` orchestration across all domains.
**Goal:** This document maps the entire architectural picture from first principles to provide clear, delineated directives for parallel coding agents to close the remaining gaps.

---

## 1. The Core Paradigm: `A = μ(O)`
The primary gap is that Rocket Craft's domain logic (game states, combinatorial parameters, validation rules) is currently hardcoded in Rust and TypeScript. It must be elevated into an RDF Ontology (`O`), queried via SPARQL (`μ`), and projected into source code (`A`) using `ggen`.

### The Actionable Gap:
- **Missing Specification Pipeline:** The project lacks the `.specify` directory structure, `ggen.toml`, and the foundational ontologies required to drive the generation of the `nexus-engine` typestates and the combinatorial test inputs.

---

## 2. Typestate Kernels (`Machine<L, P>`)
`VISION_2030.md` mandates that game systems (e.g., `nexus-combat`, `nexus-net`) transition from runtime state management to zero-cost compile-time verification using the `Machine<L: Law, P>` pattern.

### The Actionable Gap:
- **Legacy State Machines:** Existing Rust state machines in `nexus-engine` are not fully phantom-typed. They need to be refactored to align with the generated typestate templates that `ggen` will emit from the ontology.

---

## 3. Combinatorial Maximalism (The "Aimbot")
The 2030 vision requires an autonomous engine that discovers all game components and simulates every mathematical permutation of their state to prove balance and absence of infinite loops.

### The Actionable Gap:
- **Disparate Tools:** We have `chicago-tdd-tools` and the beginnings of `unify-automl`, but no single orchestration loop that reads the ontology, generates the massive coordinate matrix, feeds it into `nexus-engine`, and applies an AutoML filter to select the optimized configuration.

---

## 4. Semantic Web Orchestration (MCP / LSP)
The Model Context Protocol (MCP) must expose game-level resources (e.g., `nexus-mcp`, `economy-mcp`), allowing LLM agents and the `unify-lsp` IDE tooling to interact directly with game logic and SPARQL endpoints.

### The Actionable Gap:
- **Basic MCP Implementation:** Currently, `unify-mcp` only exposes basic project manifest tools. It needs to be expanded into game-level servers that bridge the generated Rust typestates and the RDF definitions.

---

## 5. Playwright Manufacturing & BLAKE3 Receipts
The pipeline's final step requires that optimized configurations are output as `map.t3d` files, packaged for WebGL2, driven by Playwright, and sealed with an unforgeable cryptographic receipt.

### The Actionable Gap:
- **Integration Disconnect:** The Playwright scripts and the Rust orchestration (`chicago-tdd-tools`) run in isolation. They need to be formally wired into the end of the Combinatorial Maximalism loop so that a verified receipt is only generated *after* a successful visual delta test.

---

## Agent Delegation Plan

To achieve the 80/20 of this vision immediately, specialized agents should be deployed in parallel across the following distinct tasks:

1. **The Ontologist Agent:** Responsible for defining the `ostar.ttl` ontology and writing the SPARQL `.rq` extraction queries that model the game state machines and combinatorial parameters.
2. **The Projection Agent:** Responsible for configuring `ggen.toml` and writing the `.tera` templates that consume the SPARQL output and generate the `Machine<L, P>` Rust kernels for `nexus-engine`.
3. **The Simulation Agent:** Responsible for finishing the `unify-automl` engine, enabling it to consume the generated typestates and run the brute-force Monte Carlo optimizations.
4. **The Manufacturing Agent:** Responsible for wiring the output of the AutoML engine into the `chicago-tdd-tools` WebGL2 Playwright test loop and ensuring the final BLAKE3 affidavit receipt is signed.