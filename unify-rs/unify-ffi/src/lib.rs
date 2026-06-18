//! `unify-ffi` — N-API Node.js FFI bindings for the unify-rs ecosystem.
//!
//! On `cfg(target_os = "linux")` with the `napi` feature, real N-API bindings
//! would be emitted. Without that feature / on other targets, this is a plain
//! Rust library exposing the same types and functions.
//!
//! # Feature flags
//!
//! * `napi` — adds real [`napi-rs`](https://napi.rs/) 2.x exports (`dispatch`,
//!   `version`, `list_commands`) that are compiled into the cdylib `.node` binary
//!   loadable by Node.js.  Without this flag the crate is a pure Rust library.

pub mod convert;
pub mod napi_bindings;
pub mod napi_shim;
pub mod registry;
pub mod types;

pub use convert::{ffi_to_json, json_to_ffi, FromFfi, ToFfi};
pub use napi_bindings::{dispatch_raw, list_commands_raw, version_raw};
pub use registry::FfiCommandRegistry;
pub use types::{FfiError, FfiResult, FfiValue};
