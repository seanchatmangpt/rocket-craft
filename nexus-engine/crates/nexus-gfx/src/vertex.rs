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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Vertex ────────────────────────────────────────────────────────────────

    #[test]
    fn vertex_bytemuck_zeroed_is_all_zeros() {
        let v: Vertex = bytemuck::Zeroable::zeroed();
        assert_eq!(v.position, [0.0, 0.0, 0.0]);
        assert_eq!(v.normal, [0.0, 0.0, 0.0]);
        assert_eq!(v.uv, [0.0, 0.0]);
    }

    #[test]
    fn vertex_is_pod_cast_roundtrip() {
        let v = Vertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
            tangent: [1.0, 0.0, 0.0, 1.0],
        };
        let bytes: &[u8] = bytemuck::bytes_of(&v);
        assert_eq!(bytes.len(), std::mem::size_of::<Vertex>());
        let back: &Vertex = bytemuck::from_bytes(bytes);
        assert_eq!(back.position, v.position);
        assert_eq!(back.uv, v.uv);
    }

    // ── SkinnedVertex::weights_are_normalized ─────────────────────────────────

    #[test]
    fn skinned_vertex_unit_weight_is_normalized() {
        let sv = SkinnedVertex {
            position: [0.0; 3], normal: [0.0, 1.0, 0.0], uv: [0.0; 2],
            joint_indices: [0, 0, 0, 0],
            joint_weights: [1.0, 0.0, 0.0, 0.0],
        };
        assert!(sv.weights_are_normalized());
    }

    #[test]
    fn skinned_vertex_four_equal_weights_are_normalized() {
        let sv = SkinnedVertex {
            position: [0.0; 3], normal: [0.0, 1.0, 0.0], uv: [0.0; 2],
            joint_indices: [0, 1, 2, 3],
            joint_weights: [0.25, 0.25, 0.25, 0.25],
        };
        assert!(sv.weights_are_normalized());
    }

    #[test]
    fn skinned_vertex_wrong_weight_sum_is_not_normalized() {
        let sv = SkinnedVertex {
            position: [0.0; 3], normal: [0.0, 1.0, 0.0], uv: [0.0; 2],
            joint_indices: [0, 1, 0, 0],
            joint_weights: [0.6, 0.6, 0.0, 0.0], // sum = 1.2
        };
        assert!(!sv.weights_are_normalized());
    }

    // ── ParticleVertex ────────────────────────────────────────────────────────

    #[test]
    fn particle_vertex_zeroed_is_valid_pod() {
        let p: ParticleVertex = bytemuck::Zeroable::zeroed();
        assert_eq!(p.position, [0.0; 3]);
        assert_eq!(p.color, [0.0; 4]);
        assert_eq!(p.size, 0.0);
    }
}
