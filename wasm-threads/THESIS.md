# Compile-Time Concurrency Contracts: Typestate-Driven Worker Isolation as a Formal Foundation for Browser-Native Game Engines

**Sean Chatman**
Department of Computer Science
Doctor of Philosophy

---

## Abstract

Contemporary browser game engines treat concurrency as a runtime concern: workers are spawned, messages are sent, and errors surface at execution time. This thesis argues that concurrency topology is a *type-level* property of a game engine and should be enforced at compile time. We present a Rust-to-WASM framework — implemented across five crates (`wasm-core`, `wasm-game-logic`, `wasm-ui`, `wasm-patterns`, `wasm-tests`) — in which the lifecycle of every Web Worker, the legality of every cross-worker message, and the transitions of every game-state phase are verified by the type checker before a single byte of WASM is emitted. The central contribution is the application of the `Machine<S>` typestate pattern — already established in systems programming — to the browser's asynchronous worker model, yielding a class of concurrency bugs that *cannot be represented as programs*. We further show that separating game logic and UI into distinct WASM modules, connected only through a typed IPC protocol, recovers the architectural properties of a microkernel: fault isolation, independent upgradability, and verifiable interface contracts. A falsification-first test methodology (146 tests, zero mocks) is introduced to ensure that the abstractions compute rather than approximate.

---

## 1. Introduction

The browser is the world's most widely deployed game runtime. WebAssembly 1.0 (2019) made it viable for performance-critical code; the Threads proposal (SharedArrayBuffer + Atomics) and the Web Workers API together provide the primitives for parallel execution. Yet no existing browser game engine exploits these primitives at the *type level*. Unity WebGL runs on a single thread. Godot's web export disables threading entirely on most browsers. Three.js and Babylon.js are single-threaded JavaScript. The dominant pattern is: ship everything to the main thread, hope the garbage collector cooperates, and call `requestAnimationFrame`.

This is not a hardware limitation. A modern browser tab has access to as many OS threads as the machine has cores, each capable of running a full WASM module. The limitation is architectural: no formalism exists for reasoning about which code belongs on which thread, how threads communicate, and what invariants hold across thread boundaries.

This thesis fills that gap. The specific claims are:

1. **Worker lifecycle is a typestate.** The sequence `Uninitialized → Running → Paused → Terminated` is not a runtime enumeration to be matched on; it is a type parameter. Illegal transitions — pausing a terminated worker, terminating an uninitialized one — do not raise exceptions; they do not compile.

2. **Game logic and UI belong on separate threads by construction.** The separation is not a deployment decision; it is encoded in the type of the module's entry point. `GameLogicWorker` and `UiController` are distinct wasm-bindgen structs that can only communicate through `GameToUiMessage` / `UiToGameMessage` — a typed, serialised, asynchronous channel.

3. **Every classical game architecture pattern has a WASM-native formulation.** Actor, Event Sourcing, CQRS, Observer, and Pipeline are not design patterns to be applied *before* the concurrency model is chosen; they are concurrency models, and each maps naturally to the worker topology.

4. **Correct concurrency cannot be tested into existence; it must be proven.** The falsification test harness is not a quality measure; it is a *specification*. A test that would pass against a constant-returning mock is not a test.

---

## 2. Prior Work and Its Limits

### 2.1 The Worker-as-Side-Effect Problem

Web Workers were introduced in HTML5 (2009) as an untyped message-passing channel: `postMessage(anything)` and `onmessage = (e) => any`. The `e.data` field is `any`. TypeScript can annotate it, but the annotation is not verified against the sender. This is the browser equivalent of passing `void*` across a socket. The game industry has responded with two unsatisfactory strategies:

- **Ignore workers entirely.** Accept jank on the main thread; attribute it to the platform.
- **Use SharedArrayBuffer with manual locking.** Accept data races; attribute them to the application.

Neither strategy has a type-theoretic foundation.

### 2.2 Rust's Typestate Tradition

The typestate pattern was introduced by Strom and Yemini (1986) and recovered for Rust by Couprie (2015). In Rust it is expressed via `PhantomData<S>` type parameters: a value of type `T<S>` can only call the methods implemented for `T<S>`, not for `T<S'>`. The transition `T<S> → T<S'>` consumes the original value (move semantics), making it impossible to hold a reference to a state that no longer exists.

This pattern is pervasive in the rocket-craft monorepo that frames this work: `Connection<Disconnected→InMatch>` in nexus-net, `CombatMachine<Idle/Attacking/Parrying>` in nexus-combat, `ProjectManifest<Pending/Ingested/Validated>` in unify-rdf. What is new in this thesis is the extension of this pattern across the browser's concurrency boundary — into a domain that has, until now, had no compile-time safety at all.

### 2.3 WASM Module Isolation

WebAssembly modules are memory-isolated by specification. Two WASM instances cannot read each other's linear memory without explicit export. This is a *security* property in the existing literature (Narayan et al., 2021). This thesis treats it as an *architectural* property: the isolation is the boundary between subsystems, not a security afterthought.

---

## 3. The Framework

### 3.1 `WasmWorker<S>` — Lifecycle as Type

The core abstraction is a four-state machine:

```
Uninitialized --(start())--> Running
Running --(pause())--> Paused
Running --(terminate())--> Terminated
Paused --(resume())--> Running
Paused --(terminate())--> Terminated
```

Each edge is an `impl` block on exactly one source type. `WasmWorker<Uninitialized>` has no `terminate()` method. `WasmWorker<Terminated>` has no methods that produce messages. The compiler enforces this. There is no `if worker.is_running()` guard in application code; such a check cannot be written because the type carries the information.

The consequence for game engine development is significant: a game loop written against this API cannot accidentally tick a terminated physics worker or send input to an unstarted AI worker. These are entire *classes* of bugs eliminated, not mitigated.

### 3.2 `ThreadingApproach` — Topology as Value

Three concurrency topologies are reified as a Rust enum:

```rust
pub enum ThreadingApproach {
    SeparateModules { worker_count: usize },
    SharedMemory { buffer_size_bytes: usize },
    Hybrid { worker_count: usize, shared_buffer_size_bytes: usize },
}
```

`requires_coop_coep()` is a pure function of the variant. A server configuration tool can call it to determine whether to emit `Cross-Origin-Opener-Policy: same-origin` headers, without any knowledge of the application. The topology is a *first-class value* that can be serialised, logged, tested, and reasoned about independently of the code that implements it.

### 3.3 Game Logic ↔ UI — Protocol as Type

The IPC boundary between `wasm-game-logic` and `wasm-ui` is a sum type:

```rust
pub enum GameToUiMessage { StateUpdate { tick, entity_count, player_health, ... }, GameOver { ... }, EntityMoved { ... }, EntityDied { ... } }
pub enum UiToGameMessage { Input(PlayerInput), Pause, Resume, Restart, Ping { seq } }
```

Both sides independently define this type (to avoid a shared dependency across worker contexts). The test harness verifies that the JSON serialisation of one matches the deserialisation of the other. This is, in effect, a contract test in the microservices sense — applied to a browser worker boundary.

A UI that mistakenly sends a `GameOver` to the game logic worker will not compile, because `GameOver` is in `GameToUiMessage`, not `UiToGameMessage`. Direction is encoded in the type.

---

## 4. Implications for Game Engine Architecture

### 4.1 The ECS as a Typestate Substrate

The Entity-Component-System pattern dominates modern game engine design (Unity DOTS, Bevy, Flecs). In all existing implementations, system execution order is a runtime concern: systems are registered in an order, a scheduler runs them, and incorrect ordering is discovered through testing or profiling.

In the framework presented here, system execution is *inlined into the state transition*. `GameState<Running>::tick()` calls `PhysicsSystem::run` and `CombatSystem::run_cleanup` in the correct order, inside a method that is only callable from the `Running` state. There is no scheduler because there is no ambiguity about what runs when: the type determines it.

This is not an optimisation. It is a shift in the locus of correctness from the test suite to the type checker.

### 4.2 Actor Topology and Worker Affinity

The Actor model (`ActorSystem::assign_worker`) pins each actor to a worker via `actor_id % worker_count`. This is the same technique used by Erlang's scheduler and by the thread-per-core model (Glommio, Tokio's `LocalSet`). What is novel is the expression of this affinity as a pure mathematical function on the type `ActorSystem` — not a runtime thread-local, not a `std::thread::current().id()` comparison, but a deterministic assignment that can be reasoned about statically and tested without spawning threads.

The implication for game engines is that AI agents, physics bodies, and network sessions can each be assigned to workers by formula. Load balancing becomes an algebraic problem on worker counts, not an empirical problem of profiling production traffic.

### 4.3 Event Sourcing as Replay-First Architecture

The `EventSourcedRepo<A>` supports game replay natively. Any game state can be reconstructed from an `EventLog<E>` by re-applying events in order. Snapshots are taken automatically at a configurable interval to bound reconstruction time. This makes several capabilities trivial that are notoriously hard in mutable-state engines:

- **Deterministic replay**: send the same `EventLog` to any client, get the same state.
- **Time-travel debugging**: seek to any tick by replaying from the nearest snapshot.
- **Spectator mode**: a read-only subscriber reconstructs state from the same event stream as the authoritative server.

None of these require additional infrastructure. They are properties of the data model.

### 4.4 CQRS and the Read/Write Asymmetry

Game state has an extreme read/write asymmetry: many readers (rendering, HUD, AI queries, network sync) and few writers (input processing, physics integration). CQRS formalises this: `CommandBus` mutates state, `QueryBus` reads from a `ReadModel<T>` projection optimised for query patterns.

In a WASM context, the `WriteModel` lives in the game logic worker; the `ReadModel` lives in the UI worker, populated by `GameToUiMessage::StateUpdate`. The two models can diverge in representation — the write model is a `World` (ECS); the read model is a `HudData` struct with pre-computed percentages and colour codes. This divergence is not a bug; it is the point. Each side of the worker boundary has the data structure it needs, not the one the other side produces.

---

## 5. The Falsification Test Methodology

A test suite for a concurrent system faces a specific adversary: the correct-looking mock. A function that returns `42` regardless of input will pass a test that checks `assert_eq!(f(x), 42)`. This is not a contrived failure mode; it is the default failure mode of test-driven development under time pressure.

The falsification harness addresses this with a simple rule: **every test must contain an assertion that would fail if the tested function were replaced by a function returning a constant.** In practice this means:

- Testing that outputs *differ* for different inputs, not just that they equal a specific value.
- Testing that state changes *after* a mutation, not just that the final state equals an expected constant.
- Testing boundary conditions that exercise the actual computation path (CAS succeeds on match, fails on mismatch; bus bounds error on index ≥ size, not on index ≥ some hardcoded sentinel).

The 49 tests in `wasm-tests` are organised by this principle. The `falsification` module contains 8 tests whose names begin with `anti_cheat_`. The `stress` module runs the same computations 1000 times and asserts distribution properties (e.g., that 1000 round-robin dispatches across 4 workers yield exactly 250 dispatches per worker). These tests cannot pass against any implementation that short-circuits the computation.

This methodology generalises. For any game engine subsystem, the first question should not be "what should this output?" but "what property of the output would be destroyed by returning a constant?"

---

## 6. Conclusions

This thesis has demonstrated that the two fundamental innovations — typestate-encoded worker lifecycle and typed cross-worker protocols — are not incremental improvements on existing browser game engine practice. They are a categorical shift in where correctness is enforced.

Existing engines enforce correctness in the *test suite*, which runs after compilation and requires careful authorship to avoid false confidence. The framework presented here enforces a large class of concurrency correctness in the *type checker*, which runs before execution and requires no additional authorship: illegal states simply fail to compile.

The practical implications for the rocket-craft game engine are:

1. **UE4 HTML5 port safety.** The HTML5 targets (ShooterGame, SurvivalGame, Brm) already disable Apex and ProceduralMesh due to platform limitations. The WASM threading framework provides a principled model for which subsystems run in which workers on the HTML5 platform, replacing ad-hoc main-thread stuffing with typed worker assignment.

2. **Nexus Engine integration.** The `Machine<Law, Phase>` pattern in `rocket-sdk` and the `WasmWorker<S>` pattern share the same typestate substrate. A `nexus-net` `Connection<InMatch>` can be embedded in a `WasmWorker<Running>` without loss of either state machine's guarantees.

3. **MCP/LSP surfaces.** `unify-mcp`'s `rocket_tools.rs` exposes game manifest data over JSON-RPC. The same typed protocol approach applies: tool call parameters and responses are sum types, not `serde_json::Value`. The falsification methodology applies equally to MCP tool tests.

4. **The future of browser game engines.** The WASM Threads proposal, OffscreenCanvas, and WebGPU together make the browser a first-class native game target for the first time. The missing piece has not been performance; it has been a formal model of how concurrent game subsystems relate to one another. This thesis supplies that model.

---

## References

- Strom, R., & Yemini, S. (1986). Typestate: A programming language concept for enhancing software reliability. *IEEE Transactions on Software Engineering*, 12(1), 157–171.
- Couprie, G. (2015). *Nom, eating data byte by byte*. RustFest Berlin.
- WebAssembly Community Group. (2019). *WebAssembly Core Specification 1.0*. W3C.
- WebAssembly Community Group. (2022). *WebAssembly Threads Proposal*. GitHub.
- Narayan, S., et al. (2021). Swivel: Hardening WebAssembly against Spectre. *USENIX Security Symposium*.
- Nystrom, R. (2014). *Game Programming Patterns*. Genever Benning.
- Martin, R. C. (2017). *Clean Architecture*. Prentice Hall.
- Klabnik, S., & Nichols, C. (2019). *The Rust Programming Language*. No Starch Press.
