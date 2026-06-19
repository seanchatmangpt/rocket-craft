use bytemuck::{Pod, Zeroable};

/// Standard 3D mesh vertex with position, normal, UV
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4], // xyz = tangent, w = handedness
}

/// Skinned mesh vertex with joint indices and weights for animation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub joint_indices: [u32; 4], // up to 4 bone influences
    pub joint_weights: [f32; 4], // must sum to 1.0
}

impl SkinnedVertex {
    pub fn weights_are_normalized(&self) -> bool {
        let sum: f32 = self.joint_weights.iter().sum();
        (sum - 1.0).abs() < 1e-4
    }
}

/// Particle vertex (position + color + size for beam effects)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ParticleVertex {
    pub position: [f32; 3],
    pub color: [f32; 4], // RGBA
    pub size: f32,
    pub rotation: f32,
}
