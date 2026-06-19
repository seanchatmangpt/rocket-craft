/// 3-D math type aliases and a game-space `Transform` type.
///
/// nalgebra types are used for the general API; a SIMD-optimised
/// glam sub-module is provided for hot paths such as beam-saber hit detection.
use nalgebra as na;

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Column-major 4×4 homogeneous transform matrix.
pub type Mat4 = na::Matrix4<f32>;

/// General 3-D column vector.
pub type Vec3 = na::Vector3<f32>;

/// Unit quaternion for rotation (compile-time enforced normalisation).
pub type Quat = na::UnitQuaternion<f32>;

/// 2-D column vector for UI and gesture-input coordinates.
pub type Vec2 = na::Vector2<f32>;

/// Axis-aligned bounding box represented as (min, max) corners.
pub type Aabb = (Vec3, Vec3);

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

/// Game-space transform: position, rotation and non-uniform scale.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    /// World-space translation.
    pub translation: Vec3,
    /// Orientation as a unit quaternion.
    pub rotation: Quat,
    /// Per-axis scale factors.
    pub scale: Vec3,
}

impl Transform {
    /// The identity transform: no translation, no rotation, unit scale.
    pub fn identity() -> Self {
        Self {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Convert this transform to a 4×4 column-major matrix suitable for GPU upload.
    pub fn to_matrix(&self) -> Mat4 {
        let t = na::Translation3::from(self.translation);
        let r = self.rotation.to_homogeneous();
        let s = na::Matrix4::new_nonuniform_scaling(&self.scale);
        t.to_homogeneous() * r * s
    }

    /// Linearly interpolate translation and scale; spherically interpolate rotation.
    ///
    /// `t = 0.0` returns `*self`; `t = 1.0` returns `*other`.
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            translation: self.translation.lerp(&other.translation, t),
            rotation: self.rotation.slerp(&other.rotation, t),
            scale: self.scale.lerp(&other.scale, t),
        }
    }

    /// Apply a child transform in this transform's local space (parent * child).
    pub fn mul_transform(&self, child: &Transform) -> Transform {
        let translation =
            self.translation + self.rotation * child.translation.component_mul(&self.scale);
        let rotation = self.rotation * child.rotation;
        let scale = self.scale.component_mul(&child.scale);
        Transform {
            translation,
            rotation,
            scale,
        }
    }
}

// ---------------------------------------------------------------------------
// SIMD-optimised sub-module (glam)
// ---------------------------------------------------------------------------

/// SIMD-width math types via glam for hot-path batch operations.
///
/// Use these types for beam-saber hit detection and any loop that processes
/// hundreds of transforms per frame.
pub mod simd {
    pub use glam::{Mat4 as GlamMat4, Quat as GlamQuat, Vec3A, Vec4};

    /// Convert a nalgebra `Vector3<f32>` to a glam `Vec3A` (16-byte aligned).
    #[inline]
    pub fn to_simd(v: nalgebra::Vector3<f32>) -> Vec3A {
        Vec3A::new(v.x, v.y, v.z)
    }

    /// Convert a glam `Vec3A` back to a nalgebra `Vector3<f32>`.
    #[inline]
    pub fn from_simd(v: Vec3A) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(v.x, v.y, v.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    // ── identity transform ────────────────────────────────────────────────────

    #[test]
    fn identity_has_zero_translation() {
        let t = Transform::identity();
        assert!(t.translation.norm() < EPSILON);
    }

    #[test]
    fn identity_has_unit_scale() {
        let t = Transform::identity();
        assert!(approx_eq(t.scale.x, 1.0));
        assert!(approx_eq(t.scale.y, 1.0));
        assert!(approx_eq(t.scale.z, 1.0));
    }

    #[test]
    fn identity_matrix_is_4x4_identity() {
        let m = Transform::identity().to_matrix();
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(approx_eq(m[(i, j)], expected), "m[{i},{j}] = {}", m[(i, j)]);
            }
        }
    }

    // ── lerp ─────────────────────────────────────────────────────────────────

    #[test]
    fn lerp_at_t0_returns_self() {
        let a = Transform::identity();
        let mut b = Transform::identity();
        b.translation = Vec3::new(10.0, 0.0, 0.0);
        let r = a.lerp(&b, 0.0);
        assert!(r.translation.norm() < EPSILON);
    }

    #[test]
    fn lerp_at_t1_returns_other() {
        let a = Transform::identity();
        let mut b = Transform::identity();
        b.translation = Vec3::new(10.0, 0.0, 0.0);
        let r = a.lerp(&b, 1.0);
        assert!(approx_eq(r.translation.x, 10.0));
    }

    #[test]
    fn lerp_at_t_half_is_midpoint() {
        let a = Transform::identity();
        let mut b = Transform::identity();
        b.translation = Vec3::new(20.0, 0.0, 0.0);
        let r = a.lerp(&b, 0.5);
        assert!(approx_eq(r.translation.x, 10.0));
    }

    // ── mul_transform (parent * child) ────────────────────────────────────────

    #[test]
    fn identity_parent_does_not_change_child() {
        let parent = Transform::identity();
        let mut child = Transform::identity();
        child.translation = Vec3::new(5.0, 3.0, 0.0);
        let combined = parent.mul_transform(&child);
        assert!(approx_eq(combined.translation.x, 5.0));
        assert!(approx_eq(combined.translation.y, 3.0));
    }

    #[test]
    fn parent_translation_offsets_child() {
        let mut parent = Transform::identity();
        parent.translation = Vec3::new(10.0, 0.0, 0.0);
        let child = Transform::identity(); // child at local origin
        let combined = parent.mul_transform(&child);
        assert!(approx_eq(combined.translation.x, 10.0));
    }

    // ── simd round-trip ───────────────────────────────────────────────────────

    #[test]
    fn simd_to_from_round_trips() {
        use simd::{from_simd, to_simd};
        let v = Vec3::new(1.0, 2.0, 3.0);
        let back = from_simd(to_simd(v));
        assert!(approx_eq(back.x, 1.0));
        assert!(approx_eq(back.y, 2.0));
        assert!(approx_eq(back.z, 3.0));
    }
}
