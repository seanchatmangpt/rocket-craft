# Capability Manifest

This document outlines the integrated libraries, frameworks, and their purposes across the Rocket Craft project's environments.

## Rust Ecosystem (CLI & Tools)

### Core Framework, Concurrency & Utilities
- **tokio**: Asynchronous runtime for Rust, essential for building scalable network applications and handling I/O efficiently.
- **reqwest**: Ergonomic, async HTTP client for Rust, used for making API requests and integrating with web services.
- **anyhow**: Flexible error handling for applications.
- **thiserror**: Derive macros for defining custom error types.
- **serde / serde_json**: Serialization and deserialization framework for Rust, with specific support for JSON.
- **chrono**: Comprehensive date and time library.
- **walkdir**: Recursive directory traversal with efficient handling of large directory trees.
- **ignore**: Library for respecting `.gitignore` and other ignore files during directory traversal.
- **rayon**: Data parallelism library that makes it easy to convert sequential computations into parallel ones.
- **rust-ini**: Parser and writer for INI configuration files, commonly used in Unreal Engine projects.

### Configuration & State
- **config**: Layered configuration system for Rust, allowing structured and environment-aware settings management.
- **git2**: Rust bindings for libgit2, enabling native programmatic Git repository operations, versioning, and state tracking.

### Observability & Error Handling
- **tracing**: Application-level tracing and structured logging framework.
- **color-eyre**: Error report builder for panics and `eyre::Report`, providing colorful and informative crash reporting.

### CLI & User Interaction
- **clap**: Powerful Command Line Argument Parser for building robust CLI tools.
- **clap_complete**: Generation of shell completion scripts for `clap` based CLIs.
- **dialoguer**: Library for creating interactive command-line prompts and menus.
- **indicatif**: Progress bars, spinners, and other visual indicators for CLI tools, providing feedback for long-running operations.
- **colored**: Simple terminal coloring and formatting.

### Terminal UI (TUI)
- **ratatui**: A library for building rich terminal user interfaces.
- **crossterm**: A cross-platform terminal manipulation library that provides the backend for `ratatui`.

### Cryptography & Security
- **rcgen**: Rust X.509 certificate generation library.
- **p12-keystore**: Management of PKCS#12 keystores, used for signing and security.

### WebAssembly (Wasm) Integration
- **wasmer**: High-performance WebAssembly runtime for executing Wasm modules.
- **wasm4pm-compat**: (Internal/Local) Compatibility layer for WASM 4 PM integration.

### Internal Project Components
- **rocket-sdk**: The core software development kit for the Rocket Craft project.
- **rocket-cmd**: The primary command-line interface tool for Rocket Craft.
- **knhk**: Plugin or hook system integration.
- **unrdf**: Unified RDF (Resource Description Framework) processing library.
- **un-test-utils**: Shared testing utilities and mock objects.

## Web & PWA (Node.js & Browser)

### Core Technologies
- **TypeScript**: A strongly typed programming language that builds on JavaScript, giving you better tooling at any scale.

### Architecture & APIs
- **supabase-js**: Isomorphic JavaScript client for Supabase, facilitating direct integration with Supabase Auth, Realtime, and Postgres.
- **zod**: TypeScript-first schema declaration and validation library, ensuring robust data parsing and type safety across network boundaries.

### Testing & Quality Assurance
- **vitest**: A blazing fast unit test framework powered by Vite, providing excellent TypeScript support and a compatible API with Jest.
- **Playwright**: Reliable end-to-end testing for modern web apps, enabling cross-browser testing for the PWA interfaces.

### Development Tools
- **eslint**: Pluggable linting utility for JavaScript, TypeScript, and JSX.
- **prettier**: Opinionated code formatter for consistent styling.
- **local-web-server**: A lean, modular web server for local development and testing of PWA features.

## Generative Programming & Ontologies (ggen)
- **SPARQL**: Query language for RDF data, used by `ggen` to extract ontology concepts.
- **Tera**: A template engine for Rust, inspired by Jinja2, used by `ggen` for code generation.
- **RDF (Turtle)**: The primary format for defining project ontologies and schemas.

## Unreal Engine (UE 4.24.3-HTML5)

### Integrated Plugins
- **WebSocketNetworking**: Built-in Unreal Engine plugin for WebSocket-based networking.
- **VaRest**: REST API integration plugin for Unreal Engine, used for web services.
