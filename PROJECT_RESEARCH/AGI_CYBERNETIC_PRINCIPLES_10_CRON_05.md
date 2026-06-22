# The BIG BANG $\mu$-Pipeline: Andon Cords & Agent Jidoka

**Iteration:** 10 (Cron Iteration 05)
**Target Ecosystem:** `~/ggen` (Core Manufacturing Pipeline)
**Source Files Analyzed:** 
- `crates/ggen-core/src/poka_yoke/andon.rs`
- `crates/ggen-core/src/graph/cycle_fixer.rs`

## Prologue
The 5-minute schedule loop has executed. In previous iterations, we saw how `ggen` evaluates the AGI using Six Sigma metrics (DPMO) and TPS Waste. This iteration explores what happens when the AGI *does* produce a defect. How does the factory handle a critical error?

## First Principles Extracted: Jidoka (Autonomation)

### 1. The Andon Cord (`andon.rs`)
In the Toyota Production System, the "Andon Cord" is a physical rope that any worker can pull to stop the entire assembly line if a defect is detected. `ggen` implements this literally via `AndonSignal`.
- **RED SIGNAL (Stop The Line):** If the AGI generates an invalid manifest, creates a circular dependency, or violates strict boundaries, the system throws an `AndonSignal::Red`. It does not attempt to "guess" what the AGI meant. It completely halts the generation pipeline.
- **YELLOW SIGNAL (Caution):** Issued for inefficiencies, like generating unused ontology files or slow SPARQL execution times.

**The Principle:** Never pass a defect downstream. If the AGI generates a paradox, the cybernetic loop pulls the Andon cord, halts execution, and forces a repair.

### 2. Autonomous Repair: The Cycle Fixer (`cycle_fixer.rs`)
While the Andon cord stops the line, the system also implements true *Jidoka* (automation with a human touch) via autonomous repair mechanisms. 
When the AGI hallucinates a circular dependency in the RDF graph (e.g., File A imports B imports C imports A), this creates an infinite loop for the SPARQL engine.
The `CycleFixer` detects these graphs and provides explicit strategies to fix the AGI's mistake without human intervention:
- **`remove_import`:** Automatically severs the cycle.
- **`merge_files`:** Automatically combines the cyclic files into a single, valid Directed Acyclic Graph (DAG) ontology.
- **`create_interface`:** Intelligently extracts shared RDF class/property definitions into a new `shared_definitions.ttl` interface file, updating all imports.

## Conclusion
The BIG BANG pipeline does not trust the AGI. It treats the AGI as a highly capable but error-prone engine. To counteract this, it employs strict TPS mechanisms: the Andon Cord to immediately halt execution upon detecting a paradox, and Jidoka (Cycle Fixer) to automatically restructure the AGI's ontological mistakes back into mathematical validity. 

The cybernetic loop is fully supervised by these manufacturing constraints.
