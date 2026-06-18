pub mod capability;
pub mod compositor;
pub mod conformance;
pub mod diagnostic;
pub mod gate;
pub mod server;
pub mod snapshot;

pub use lsp_max::{Client, LanguageServer};
pub use lsp_max_protocol::{ConformanceVector, SnapshotId};

#[cfg(test)]
mod tests;
