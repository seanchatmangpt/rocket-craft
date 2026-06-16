# Contributing to Rocket Craft

Welcome to the Rocket Craft project! This document provides guidelines and information for contributing to this multi-project workspace.

## Workspace Structure

The Rocket Craft workspace is organized as a multi-project repository containing several Unreal Engine 4 projects and supporting web assets, Rust tools, and APIs.

### Directory Overview

-   **/versions**: The core of the workspace. Contains multiple independent Unreal Engine projects.
-   **/pwa-staff**: Contains Progressive Web App (PWA) assets, including service workers and offline support.
-   **/chicago-tdd-tools**: Behavior-driven development suite.
-   **/tools/knhk**: Project constraint and semantic laws enforcement.
-   **/tools/rocket-cmd** & **/tools/rocket-sdk**: Core logic for the `./rocket` CLI.
-   **GEMINI.md**: Workspace-wide instructions and standards for AI agents and developers.
-   **README.md**: General project overview and setup instructions.

## The `./rocket` CLI Development Tool

The project is heavily driven by the `./rocket` CLI tool at the root of the workspace. Use it to manage the project environment and workflows:

-   `./rocket setup`: Setup the Unreal Engine environment.
-   `./rocket sync`: Synchronize project manifest with the filesystem.
-   `./rocket build`: Build project targets.
-   `./rocket test`: Run all tests (Rust, Asset validation, etc.).
-   `./rocket audit`: Audit project health and verify semantic law compliance.
-   `./rocket capabilities`: List integrated high-level features.

To see all available commands, run: `./rocket help`.

## Development Approaches

### Behavior-Driven Development (Chicago TDD Tools)

We utilize `chicago-tdd-tools` to practice Classicist (Chicago School) TDD in our Rust tooling.
-   **Behavior-focused**: Test state and behavior via public APIs rather than mocks and implementation details.
-   **Integration-friendly**: We encourage testing multiple components together as they would be used in production.
-   **Process**: Define the desired behavior in tests first, implement the minimal logic in the domain layer, then refactor while keeping tests green.

### Semantic Laws (`knhk`)

We enforce architectural standards, security requirements, and project-specific rules through the `knhk` Semantic Laws framework.
- Laws are represented by the `Law` trait (e.g., `AndroidKeystoreLaw`).
- Laws programmatically validate that the project satisfies rigid rules (e.g., ensuring an Android target always has an associated `.keystore`).
- Run `./rocket audit` to continuously validate the project against all registered semantic laws.

## Multi-Project Workflow

1.  **Context Awareness**: When making changes, ensure you are in the correct project directory within `versions/`.
2.  **Shared Standards**: Maintain consistent UI/UX and branding across all projects.
3.  **Cross-Project Testing**: If you modify shared logic, verify the impact across all relevant projects.

## Submitting Changes

1.  **Surgical Edits**: Keep changes focused on a single project or feature.
2.  **Commit Messages**: Use clear, concise commit messages.
3.  **Verification**: Ensure all projects still build, tests pass (`./rocket test`), and semantic laws are satisfied (`./rocket audit`).
