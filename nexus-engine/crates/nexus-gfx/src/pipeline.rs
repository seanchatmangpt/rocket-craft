use std::marker::PhantomData;

// Pipeline state markers
pub struct Uninitialized;
pub struct Compiled;

/// Phantom-typed render pipeline to prevent using uncompiled pipelines
pub struct RenderPipeline<S> {
    pub label: String,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    pub depth_write: bool,
    pub depth_compare: DepthCompare,
    _state: PhantomData<S>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode { None, Front, Back }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode { Opaque, AlphaBlend, Additive }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthCompare { Less, LessEqual, Greater, Always }

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
        }
    }

    pub fn with_blend(mut self, mode: BlendMode) -> Self { self.blend_mode = mode; self }
    pub fn with_cull(mut self, mode: CullMode) -> Self { self.cull_mode = mode; self }

    /// Compile (in production this would call wgpu -- here it's a type transition)
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
        }
    }
}

// Only compiled pipelines can be rendered with
impl RenderPipeline<Compiled> {
    pub fn label(&self) -> &str { &self.label }
    // In production: bind_to_render_pass(&mut wgpu::RenderPass<'_>) etc.
}

/// Standard Gundam Nexus pipeline set
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
                .with_cull(CullMode::Back).compile(),
            transparent: RenderPipeline::new("transparent", "suit.vert", "suit_transparent.frag")
                .with_blend(BlendMode::AlphaBlend).compile(),
            beam_effects: RenderPipeline::new("beam", "particle.vert", "beam.frag")
                .with_blend(BlendMode::Additive)
                .with_cull(CullMode::None).compile(),
            ui: RenderPipeline::new("ui", "ui.vert", "ui.frag")
                .with_blend(BlendMode::AlphaBlend)
                .with_cull(CullMode::None).compile(),
            shadow: RenderPipeline::new("shadow", "shadow.vert", "shadow.frag")
                .with_cull(CullMode::Front).compile(),
        }
    }
}
