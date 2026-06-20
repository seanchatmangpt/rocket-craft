//! blueprint-core — runtime types for building UE4 Blueprint T3D output.
//!
//! This crate exposes two layers:
//!
//! 1. **Low-level AST** (`types`, `ast`, `nodes`) — full node/pin graph that
//!    mirrors UE4's internal representation, suitable for round-trip T3D
//!    serialisation.
//!
//! 2. **High-level builder** ([`BlueprintBuilder`]) — ergonomic API consumed by
//!    the `blueprint_macros` proc-macro crate.

pub mod ast;
pub mod builder;
pub mod diff;
pub mod layout;
pub mod nodes;
pub mod parser;
pub mod registry;
pub mod render;
pub mod serializer;
pub mod types;
pub mod validator;

pub use ast::*;
pub use builder::{BlueprintBuilder, EventBodyBuilder, NodeHandle, Statement, VarType};
pub use layout::auto_layout_blueprint;
pub use serializer::{JsonSerializer, T3dSerializer};
pub use types::*;
