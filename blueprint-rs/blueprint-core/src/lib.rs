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

pub mod types;
pub mod ast;
pub mod nodes;
pub mod builder;
pub mod serializer;
pub mod render;
pub mod layout;
pub mod patterns;

pub use types::*;
pub use ast::*;
pub use builder::{BlueprintBuilder, NodeHandle, VarType, EventBodyBuilder, Statement};
pub use serializer::{T3dSerializer, JsonSerializer};
pub use layout::auto_layout_blueprint;
