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
        let translation = self.translation
            + self.rotation * child.translation.component_mul(&self.scale);
        let rotation = self.rotation * child.rotation;
        let scale = self.scale.component_mul(&child.scale);
        Transform { translation, rotation, scale }
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
    pub use glam::{Vec3A, Vec4, Mat4 as GlamMat4, Quat as GlamQuat};

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
