# Chicago TDD Tools - AI Agent Instructions

This directory contains the behavioral testing suite for the `rocket-sdk` workspace. Future AI agents modifying or extending this codebase MUST strictly adhere to the following guidelines.

## TDD Principles: The Classicist (Chicago) School

When working in the Rocket Craft repository, you must adhere strictly to the **Classicist (Chicago) School of TDD**:

1. **Test Behaviors, Not Implementations**: Tests must verify the *what* (observable outcomes, state changes), not the *how* (internal method calls, private state). Interact exclusively with the public API of the system under test (SUT).
2. **State-Based Verification**: Assert on the final state of the system or the output of a function. Avoid mock objects and interaction verification unless dealing with slow, expensive, or non-deterministic external boundaries (like network I/O or file systems).
3. **No Internal Mocking**: Unit tests can and should span multiple internal modules or structs if they collectively represent a single logical behavior. Do not use mocks for internal collaborators.
4. **Refactoring Freedom**: Your tests should remain green during refactoring as long as the external behavior is preserved. If tests break when you change internal structures, your tests are too tightly coupled to the implementation.

## Unreal Environment Simulation

This suite leverages the `un-test-utils` crate to simulate Unreal Engine environments. Use these utilities to establish the necessary context for your tests without relying on a real engine instance.

## Running Tests

You must verify all changes by running the test suite:

```bash
cargo test
```

Always ensure tests pass before considering a task complete.
