//! `unify-ffi` — N-API Node.js FFI bindings for the unify-rs ecosystem.
//!
//! On `cfg(target_os = "linux")` with the `napi` feature, real N-API bindings
//! would be emitted. Without that feature / on other targets, this is a plain
//! Rust library exposing the same types and functions.

pub mod types;
pub mod convert;
pub mod registry;
pub mod napi_shim;

pub use types::{FfiError, FfiResult, FfiValue};
pub use convert::{FromFfi, ToFfi, ffi_to_json, json_to_ffi};
pub use registry::FfiCommandRegistry;
