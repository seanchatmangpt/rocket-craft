use nalgebra as na;
use serde::{Serialize, Deserialize};

use crate::color::GfxError;

/// World-space 3D position
pub type Vec3 = na::Vector3<f32>;
/// 4D vector (homogeneous coordinates / RGBA color)
pub type Vec4 = na::Vector4<f32>;
/// Column-major 4x4 transformation matrix
pub type Mat4 = na::Matrix4<f32>;
/// Normalized unit quaternion for rotation (nalgebra enforces unit constraint)
pub type UnitQuat = na::UnitQuaternion<f32>;
/// 2D vector for UV coordinates and gesture input
pub type Vec2 = na::Vector2<f32>;

/// Screen-space pixel coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelCoord { pub x: u32, pub y: u32 }

/// Normalized device coordinate: both axes in [-1, 1]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ndc { x: f32, y: f32 }

impl Ndc {
    pub fn new(x: f32, y: f32) -> Result<Self, GfxError> {
        if !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y) {
            Err(GfxError::OutOfNdcRange { x, y })
        } else {
            Ok(Ndc { x, y })
        }
    }
    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
}

/// Game-space transform: position + rotation + uniform scale
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: UnitQuat,
    pub scale: f32,  // uniform scale (mobile suit proportions preserved)
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            translation: Vec3::zeros(),
            rotation: UnitQuat::identity(),
            scale: 1.0,
        }
    }

    /// Compute model-to-world matrix M = T * R * S
    pub fn to_matrix(&self) -> Mat4 {
        let t = na::Translation3::from(self.translation);
        let r = self.rotation.to_homogeneous();
        let s = Mat4::new_scaling(self.scale);
        t.to_homogeneous() * r * s
    }

    /// Parent * child composition (child is in parent space)
    pub fn mul_transform(&self, child: &Transform) -> Transform {
        Transform {
            translation: self.rotation * (child.translation * self.scale) + self.translation,
            rotation: self.rotation * child.rotation,
            scale: self.scale * child.scale,
        }
    }

    /// Linear interpolation between transforms (for animation)
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        let t = t.clamp(0.0, 1.0);
        Transform {
            translation: self.translation.lerp(&other.translation, t),
            rotation: self.rotation.slerp(&other.rotation, t),
            scale: self.scale + (other.scale - self.scale) * t,
        }
    }
}

impl Default for Transform {
    fn default() -> Self { Self::identity() }
}
