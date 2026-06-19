use std::marker::PhantomData;

#[cfg(feature = "gpu")]
use std::sync::Arc;

// Pipeline state markers
pub struct Uninitialized;
pub struct Compiled;

/// Phantom-typed render pipeline to prevent using uncompiled pipelines.
///
/// The generic state parameter `S` is either `Uninitialized` or `Compiled`.
/// When the `gpu` feature is enabled, the `Compiled` variant stores the live
/// `wgpu::RenderPipeline` (or `None` when compiled via the no-device path).
pub struct RenderPipeline<S> {
    pub label: String,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    pub depth_write: bool,
    pub depth_compare: DepthCompare,
    _state: PhantomData<S>,

    /// Populated only in `Compiled` state when the `gpu` feature is enabled.
    #[cfg(feature = "gpu")]
    gpu_pipeline_inner: Option<Arc<wgpu::RenderPipeline>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Additive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthCompare {
    Less,
    LessEqual,
    Greater,
    Always,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors returned when compiling a GPU render pipeline.
#[cfg(feature = "gpu")]
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("shader compilation failed for '{label}': {message}")]
    ShaderCompilation { label: String, message: String },
}

// ---------------------------------------------------------------------------
// wgpu enum conversions (only compiled-in when the gpu feature is active)
// ---------------------------------------------------------------------------

#[cfg(feature = "gpu")]
impl From<CullMode> for Option<wgpu::Face> {
    fn from(m: CullMode) -> Self {
        match m {
            CullMode::None => None,
            CullMode::Front => Some(wgpu::Face::Front),
            CullMode::Back => Some(wgpu::Face::Back),
        }
    }
}

#[cfg(feature = "gpu")]
impl From<BlendMode> for Option<wgpu::BlendState> {
    fn from(m: BlendMode) -> Self {
        match m {
            BlendMode::Opaque => None,
            BlendMode::AlphaBlend => Some(wgpu::BlendState::ALPHA_BLENDING),
            BlendMode::Additive => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        }
    }
}

#[cfg(feature = "gpu")]
impl From<DepthCompare> for wgpu::CompareFunction {
    fn from(d: DepthCompare) -> Self {
        match d {
            DepthCompare::Less => wgpu::CompareFunction::Less,
            DepthCompare::LessEqual => wgpu::CompareFunction::LessEqual,
            DepthCompare::Greater => wgpu::CompareFunction::Greater,
            DepthCompare::Always => wgpu::CompareFunction::Always,
        }
    }
}

// ---------------------------------------------------------------------------
// Uninitialized state — builder methods + compile transitions
// ---------------------------------------------------------------------------

impl RenderPipeline<Uninitialized> {
    pub fn new(label: &str, vertex_shader: &str, fragment_shader: &str) -> Self {
        RenderPipeline {
            label: label.to_string(),
            vertex_shader: vertex_shader.to_string(),
            fragment_shader: fragment_shader.to_string(),
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::Opaque,
            depth_write: true,
            depth_compare: DepthCompare::Less,
            _state: PhantomData,
            #[cfg(feature = "gpu")]
            gpu_pipeline_inner: Option::None,
        }
    }

    pub fn with_blend(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }
    pub fn with_cull(mut self, mode: CullMode) -> Self {
        self.cull_mode = mode;
        self
    }

    /// Compile without a GPU device (type-state transition only).
    ///
    /// When the `gpu` feature is enabled the stored `gpu_pipeline` will be `None`.
    /// Use `compile_with_device` to get a real GPU pipeline.
    pub fn compile(self) -> RenderPipeline<Compiled> {
        RenderPipeline {
            label: self.label,
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            cull_mode: self.cull_mode,
            blend_mode: self.blend_mode,
            depth_write: self.depth_write,
            depth_compare: self.depth_compare,
            _state: PhantomData,
            #[cfg(feature = "gpu")]
            gpu_pipeline_inner: Option::None,
        }
    }

    /// Compile with a real wgpu device, creating a GPU-resident pipeline.
    ///
    /// Both `vertex_shader` and `fragment_shader` are treated as WGSL source.
    #[cfg(feature = "gpu")]
    pub fn compile_with_device(
        self,
        device: &wgpu::Device,
    ) -> Result<RenderPipeline<Compiled>, PipelineError> {
        let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{}_vs", self.label)),
            source: wgpu::ShaderSource::Wgsl(self.vertex_shader.as_str().into()),
        });

        let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{}_fs", self.label)),
            source: wgpu::ShaderSource::Wgsl(self.fragment_shader.as_str().into()),
        });

        let blend = Option::<wgpu::BlendState>::from(self.blend_mode);

        let color_targets = [Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            blend,
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let depth_stencil = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: self.depth_write,
            depth_compare: self.depth_compare.into(),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", self.label)),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let raw_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "fs_main",
                targets: &color_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: Option::None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: self.cull_mode.into(),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: Option::None,
            cache: Option::None,
        });

        Ok(RenderPipeline {
            label: self.label,
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            cull_mode: self.cull_mode,
            blend_mode: self.blend_mode,
            depth_write: self.depth_write,
            depth_compare: self.depth_compare,
            _state: PhantomData,
            gpu_pipeline_inner: Some(Arc::new(raw_pipeline)),
        })
    }
}

// ---------------------------------------------------------------------------
// Compiled state — query methods
// ---------------------------------------------------------------------------

impl RenderPipeline<Compiled> {
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns a reference to the wgpu pipeline, if one was compiled via
    /// `compile_with_device`.
    #[cfg(feature = "gpu")]
    pub fn gpu_pipeline(&self) -> Option<&wgpu::RenderPipeline> {
        self.gpu_pipeline_inner.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Standard Gundam Nexus pipeline set
// ---------------------------------------------------------------------------

/// Standard Gundam Nexus pipeline set.
pub struct PipelineSet {
    pub opaque: RenderPipeline<Compiled>,
    pub transparent: RenderPipeline<Compiled>,
    pub beam_effects: RenderPipeline<Compiled>,
    pub ui: RenderPipeline<Compiled>,
    pub shadow: RenderPipeline<Compiled>,
}

impl PipelineSet {
    pub fn build() -> Self {
        PipelineSet {
            opaque: RenderPipeline::new("opaque", "suit.vert", "suit.frag")
                .with_cull(CullMode::Back)
                .compile(),
            transparent: RenderPipeline::new("transparent", "suit.vert", "suit_transparent.frag")
                .with_blend(BlendMode::AlphaBlend)
                .compile(),
            beam_effects: RenderPipeline::new("beam", "particle.vert", "beam.frag")
                .with_blend(BlendMode::Additive)
                .with_cull(CullMode::None)
                .compile(),
            ui: RenderPipeline::new("ui", "ui.vert", "ui.frag")
                .with_blend(BlendMode::AlphaBlend)
                .with_cull(CullMode::None)
                .compile(),
            shadow: RenderPipeline::new("shadow", "shadow.vert", "shadow.frag")
                .with_cull(CullMode::Front)
                .compile(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── RenderPipeline typestate: Uninitialized ────────────────────────────────

    #[test]
    fn new_pipeline_stores_labels_and_defaults() {
        let p = RenderPipeline::<Uninitialized>::new("main", "vs.wgsl", "fs.wgsl");
        assert_eq!(p.label, "main");
        assert_eq!(p.vertex_shader, "vs.wgsl");
        assert_eq!(p.fragment_shader, "fs.wgsl");
        assert_eq!(p.cull_mode, CullMode::Back);
        assert_eq!(p.blend_mode, BlendMode::Opaque);
        assert!(p.depth_write);
        assert_eq!(p.depth_compare, DepthCompare::Less);
    }

    #[test]
    fn with_blend_alpha_sets_blend_mode() {
        let p = RenderPipeline::<Uninitialized>::new("ui", "vs.wgsl", "fs.wgsl")
            .with_blend(BlendMode::AlphaBlend);
        assert_eq!(p.blend_mode, BlendMode::AlphaBlend);
    }

    #[test]
    fn with_cull_none_sets_cull_mode() {
        let p = RenderPipeline::<Uninitialized>::new("sky", "vs.wgsl", "fs.wgsl")
            .with_cull(CullMode::None);
        assert_eq!(p.cull_mode, CullMode::None);
    }

    // ── compile() — no-gpu typestate transition ────────────────────────────────

    #[test]
    fn compile_transitions_and_preserves_settings() {
        let compiled = RenderPipeline::<Uninitialized>::new("beam", "vs.wgsl", "fs.wgsl")
            .with_blend(BlendMode::Additive)
            .with_cull(CullMode::Front)
            .compile();
        assert_eq!(compiled.label, "beam");
        assert_eq!(compiled.blend_mode, BlendMode::Additive);
        assert_eq!(compiled.cull_mode, CullMode::Front);
    }

    // Cannot call compile on an already-compiled pipeline (type system prevents it)
    // — this test proves the transition is one-way by verifying the Compiled variant
    // exposes the same fields correctly.
    #[test]
    fn compiled_pipeline_fields_match_configured_values() {
        let c = RenderPipeline::<Uninitialized>::new("depth", "vs.wgsl", "fs.wgsl")
            .compile();
        assert_eq!(c.depth_compare, DepthCompare::Less);
        assert!(c.depth_write);
    }

    // ── Enum distinctness ─────────────────────────────────────────────────────

    #[test]
    fn cull_mode_variants_are_distinct() {
        assert_ne!(CullMode::None, CullMode::Front);
        assert_ne!(CullMode::Front, CullMode::Back);
        assert_ne!(CullMode::None, CullMode::Back);
    }

    #[test]
    fn blend_mode_variants_are_distinct() {
        assert_ne!(BlendMode::Opaque, BlendMode::AlphaBlend);
        assert_ne!(BlendMode::AlphaBlend, BlendMode::Additive);
    }

    #[test]
    fn depth_compare_all_four_variants_distinct() {
        let variants = [
            DepthCompare::Less,
            DepthCompare::LessEqual,
            DepthCompare::Greater,
            DepthCompare::Always,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i == j {
                    assert_eq!(variants[i], variants[j]);
                } else {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // ── Clone / Copy for enums ────────────────────────────────────────────────

    #[test]
    fn enum_variants_clone_and_copy() {
        let m = CullMode::Back;
        let m2 = m; // Copy
        assert_eq!(m, m2);
        let b = BlendMode::AlphaBlend;
        let b2 = b.clone(); // Clone
        assert_eq!(b, b2);
    }
}
