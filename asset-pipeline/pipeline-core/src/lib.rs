//! # Pipeline Core
//!
//! Provides the core pipeline processing features for asset pipeline processing,
//! including configurations, asset validation, conversion, staging, and event reporting.

pub mod types;
pub mod error;
pub mod config;
pub mod discovery;
pub mod validation;
pub mod conversion;
pub mod staging;
pub mod reporting;

// Re-export the most commonly used types at crate root
pub use types::{
    DiscoveredAsset, ValidatedAsset, ConvertedAsset, StagedAsset,
    Format, PipelineEvent,
};
pub use error::PipelineError;
pub use config::{PipelineConfig, PipelineSection};
