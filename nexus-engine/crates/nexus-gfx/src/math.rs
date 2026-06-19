use nalgebra as na;

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
pub struct PixelCoord {
    pub x: u32,
    pub y: u32,
}

/// Normalized device coordinate: both axes in [-1, 1]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ndc {
    x: f32,
    y: f32,
}

impl Ndc {
    pub fn new(x: f32, y: f32) -> Result<Self, GfxError> {
        if !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y) {
            Err(GfxError::OutOfNdcRange { x, y })
        } else {
            Ok(Ndc { x, y })
        }
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
}

pub use nexus_types::Transform;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Ndc::new ──────────────────────────────────────────────────────────────

    #[test]
    fn ndc_valid_origin() {
        let n = Ndc::new(0.0, 0.0).unwrap();
        assert_eq!(n.x(), 0.0);
        assert_eq!(n.y(), 0.0);
    }

    #[test]
    fn ndc_boundary_values_accepted() {
        Ndc::new(-1.0, -1.0).unwrap();
        Ndc::new(1.0, 1.0).unwrap();
        Ndc::new(-1.0, 1.0).unwrap();
    }

    #[test]
    fn ndc_out_of_range_rejected() {
        assert!(Ndc::new(1.1, 0.0).is_err());
        assert!(Ndc::new(-1.1, 0.0).is_err());
        assert!(Ndc::new(0.0, 1.1).is_err());
        assert!(Ndc::new(0.0, -1.1).is_err());
    }

    #[test]
    fn pixel_coord_stores_x_and_y() {
        let p = PixelCoord { x: 1920, y: 1080 };
        assert_eq!(p.x, 1920);
        assert_eq!(p.y, 1080);
    }

    #[test]
    fn pixel_coord_zero_is_valid() {
        let p = PixelCoord { x: 0, y: 0 };
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }

    #[test]
    fn vec3_zero_is_additive_identity() {
        let v = Vec3::zeros();
        let u = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v + u, u);
    }

    #[test]
    fn mat4_identity_preserves_vec4() {
        let m = Mat4::identity();
        let v = Vec4::new(1.0, 2.0, 3.0, 1.0);
        assert_eq!(m * v, v);
    }
}
