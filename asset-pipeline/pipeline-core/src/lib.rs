//! # Pipeline Core
//!
//! Provides the core pipeline processing features for asset pipeline processing,
//! including configurations, asset validation, conversion, staging, and event reporting.

pub mod config;
pub mod conversion;
pub mod discovery;
pub mod error;
pub mod reporting;
pub mod staging;
pub mod types;
pub mod validation;

// Re-export the most commonly used types at crate root
pub use config::{PipelineConfig, PipelineSection};
pub use error::PipelineError;
pub use types::{
    ConvertedAsset, DiscoveredAsset, Format, PipelineEvent, StagedAsset, ValidatedAsset,
};
