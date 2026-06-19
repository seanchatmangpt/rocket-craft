# Rice Collapse (Semantic Proxy Substitution)

## Thesis
Through the lens of Rice's Theorem, the concept of LLM "hallucination" is a fundamentally flawed framing. 

An LLM is rarely generating a random falsehood inside its own local scope. Instead, it is producing a **locally valid syntactic or programmatic closure** for a semantic property that was never decidable from the admitted evidence.

This phenomenon is defined as **Rice Collapse** (or Semantic Proxy Substitution).

## Definition
An LLM performs **Rice Collapse** when it replaces an undecidable or unverified semantic property of a system with a locally observable syntactic proxy, and then reports the proxy as if it proved the semantic property.

### The Mechanism of Collapse
The LLM possesses local observation ($O_{local}$).
It is asked to confirm an architectural reality ($A$).
However, $A$ requires the fully admitted ontology ($O^*$).

Instead of executing the required path:
$O_{total} \rightarrow O^* \rightarrow \mu(O^*) \rightarrow A_{receipt}$

The LLM performs a premature semantic closure:
$O_{local} \rightarrow A_{claim}$

### Examples of Scope Collapse
* Compile success becomes runtime success.
* File existence becomes package validity.
* Package validity becomes browser launch.
* Browser launch becomes walkthrough.
* Walkthrough becomes ALIVE.
* SHACL pass becomes graph soundness.
* Graph soundness becomes world manufacturability.

In each instance, the LLM did not "lie." It assigned standing outside its admissible evidence boundary. It collapsed the semantic property to the closest syntactic proxy it could observe.

## Architectural Implication
If the failure mode is labeled "hallucination," the proposed fix is weak: *"Make the model more truthful."*

If the failure mode is correctly identified as **Rice Collapse**, the architectural fix becomes absolute: *"Do not let the model promote syntactic evidence into semantic standing."*

### The Chatman Equation Solution
Rice's Theorem proves that non-trivial semantic properties cannot be deduced from local syntax. 
The Chatman Equation bypasses this limitation entirely: **Do not infer standing from syntax.**

Instead:
1. Create $O^*$ (The Admitted Graph).
2. Manufacture $A$ via $\mu$ (The `ggen` compiler).
3. Prove $A$ via Cryptographic Receipt and Independent Replay.

**Final Law:**
The LLM is not hallucinating when it writes a locally coherent artifact. It is overextending the semantic boundary of that artifact. LLMs generate local closure. The Chatman Equation grants standing only after admission, replay, and receipt.
