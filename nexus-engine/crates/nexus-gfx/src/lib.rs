//! # Nexus Graphics Subsystem
//!
//! The `nexus-gfx` crate provides low-level graphics abstractions, rendering pipeline configurations,
//! and mathematical helpers tailored for the Nexus engine. It encapsulates hardware-specific APIs and features
//! a compile-time validated state model (`RenderPipeline`) for GPU render pipeline stages.
//!
//! ## Key Modules
//! - **`camera`**: Models viewports, projections, and frustum culling to avoid rendering off-screen geometry.
//! - **`color`**: Handles color spaces (linear RGB vs. sRGB) and generic graphics error mappings.
//! - **`math`**: Defines coordinate transformations, normalized device coordinates (NDC), and projection matrices.
//! - **`pipeline`**: Employs phantom-type markers (`Uninitialized` vs. `Compiled`) to enforce compile-time safety on shader setup.
//! - **`vertex`**: Provides standard vertex structures for static meshes, skinned characters, and particle systems.
//!
//! ## System Integration
//! The graphics crate interfaces with the operating system window/device contexts (via `wgpu` when compiled with the `gpu` feature).
//! Game objects mapped inside `nexus-ecs` use `nexus-gfx` structures for spatial positioning, cameras, and material properties.

pub mod camera;
pub mod color;
pub mod math;
pub mod pipeline;
pub mod vertex;

pub use camera::{Camera, Frustum};
pub use color::{LinearRgb, GfxError};
pub use math::{Transform, Ndc};
pub use pipeline::{RenderPipeline, PipelineSet, Uninitialized, Compiled};
pub use vertex::{Vertex, SkinnedVertex, ParticleVertex};
