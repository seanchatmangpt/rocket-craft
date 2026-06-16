# Chicago TDD Tools

The official behavioral testing suite for the `rocket-sdk` workspace.

## Overview

This toolset is designed for practicing Classicist (Chicago School) Test-Driven Development (TDD) in Rust. It serves as the primary behavioral testing environment for the `rocket-sdk`, ensuring that features are tested based on their observable behaviors rather than their internal implementations.

## Features

- **Behavior-Focused Testing**: Examples and utilities for testing state and behavior rather than relying on mocks and exposing implementation details.
- **Unreal Environment Simulation**: Leverages `un-test-utils` to simulate Unreal Engine environments, allowing for robust testing of SDK integrations without requiring a full engine instance.
- **Domain-Driven Structure**: Logic is organized into domain modules and tested via a public API.
- **Integration-Friendly**: Encourages testing multiple components together as they would be used in production.

## Running Tests

To run the behavioral testing suite:

```bash
cargo test
```
