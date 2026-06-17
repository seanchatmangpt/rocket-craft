pub mod capability;
pub mod diagnostic;
pub mod gate;
pub mod compositor;
pub mod snapshot;
pub mod conformance;
pub mod server;

pub use lsp_max::{Client, LanguageServer};
pub use lsp_max_protocol::{ConformanceVector, SnapshotId};

#[cfg(test)]
mod tests;
