use crate::color::GfxError;
use crate::math::*;
use nalgebra as na;

/// Perspective camera with physically-based parameters.
/// FovY is constrained: (0, pi).
pub struct Camera {
    pub transform: Transform,
    fov_y_radians: f32, // private -- enforced via constructor
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(fov_y_degrees: f32, aspect: f32, near: f32, far: f32) -> Result<Self, GfxError> {
        if fov_y_degrees <= 0.0 || fov_y_degrees >= 180.0 {
            return Err(GfxError::InvalidFov(fov_y_degrees));
        }
        if near <= 0.0 || far <= near {
            return Err(GfxError::InvalidDepthRange { near, far });
        }
        Ok(Camera {
            transform: Transform::identity(),
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio: aspect,
            near,
            far,
        })
    }

    /// View matrix: inverse of camera's world-space transform
    pub fn view_matrix(&self) -> Mat4 {
        self.transform
            .to_matrix()
            .try_inverse()
            .unwrap_or(Mat4::identity()) // degenerate fallback
    }

    /// Projection matrix (RH, depth [0,1] for wgpu)
    pub fn projection_matrix(&self) -> Mat4 {
        na::Perspective3::new(self.aspect_ratio, self.fov_y_radians, self.near, self.far)
            .to_homogeneous()
    }

    /// Combined view-projection matrix (VP = P * V)
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn fov_y_degrees(&self) -> f32 {
        self.fov_y_radians.to_degrees()
    }
}

/// Axis-aligned bounding box for frustum culling
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Result<Self, GfxError> {
        if min.x > max.x || min.y > max.y || min.z > max.z {
            return Err(GfxError::InvalidAabb);
        }
        Ok(Aabb { min, max })
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) / 2.0
    }

    pub fn contains_point(&self, p: Vec3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    pub fn transform(&self, t: &Transform) -> Aabb {
        let m = t.to_matrix();
        let corners = self.corners();
        let transformed: Vec<Vec3> = corners
            .iter()
            .map(|c| {
                let h = na::Vector4::new(c.x, c.y, c.z, 1.0);
                let r = m * h;
                Vec3::new(r.x / r.w, r.y / r.w, r.z / r.w)
            })
            .collect();
        let min = Vec3::new(
            transformed
                .iter()
                .map(|v| v.x)
                .fold(f32::INFINITY, f32::min),
            transformed
                .iter()
                .map(|v| v.y)
                .fold(f32::INFINITY, f32::min),
            transformed
                .iter()
                .map(|v| v.z)
                .fold(f32::INFINITY, f32::min),
        );
        let max = Vec3::new(
            transformed
                .iter()
                .map(|v| v.x)
                .fold(f32::NEG_INFINITY, f32::max),
            transformed
                .iter()
                .map(|v| v.y)
                .fold(f32::NEG_INFINITY, f32::max),
            transformed
                .iter()
                .map(|v| v.z)
                .fold(f32::NEG_INFINITY, f32::max),
        );
        Aabb { min, max } // always valid after transform
    }

    fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }
}

/// View frustum for culling (6 planes)
pub struct Frustum {
    planes: [Vec4; 6], // ax + by + cz + d = 0
}

impl Frustum {
    pub fn from_view_projection(vp: &Mat4) -> Self {
        // Gribb-Hartmann method: extract planes from VP matrix
        // nalgebra uses (row, col) indexing
        let m = vp;
        Frustum {
            planes: [
                // Left plane
                Vec4::new(
                    m[(3, 0)] + m[(0, 0)],
                    m[(3, 1)] + m[(0, 1)],
                    m[(3, 2)] + m[(0, 2)],
                    m[(3, 3)] + m[(0, 3)],
                ),
                // Right plane
                Vec4::new(
                    m[(3, 0)] - m[(0, 0)],
                    m[(3, 1)] - m[(0, 1)],
                    m[(3, 2)] - m[(0, 2)],
                    m[(3, 3)] - m[(0, 3)],
                ),
                // Bottom
                Vec4::new(
                    m[(3, 0)] + m[(1, 0)],
                    m[(3, 1)] + m[(1, 1)],
                    m[(3, 2)] + m[(1, 2)],
                    m[(3, 3)] + m[(1, 3)],
                ),
                // Top
                Vec4::new(
                    m[(3, 0)] - m[(1, 0)],
                    m[(3, 1)] - m[(1, 1)],
                    m[(3, 2)] - m[(1, 2)],
                    m[(3, 3)] - m[(1, 3)],
                ),
                // Near
                Vec4::new(
                    m[(3, 0)] + m[(2, 0)],
                    m[(3, 1)] + m[(2, 1)],
                    m[(3, 2)] + m[(2, 2)],
                    m[(3, 3)] + m[(2, 3)],
                ),
                // Far
                Vec4::new(
                    m[(3, 0)] - m[(2, 0)],
                    m[(3, 1)] - m[(2, 1)],
                    m[(3, 2)] - m[(2, 2)],
                    m[(3, 3)] - m[(2, 3)],
                ),
            ],
        }
    }

    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        let center = aabb.center();
        let half = aabb.half_extents();
        for plane in &self.planes {
            let n = Vec3::new(plane.x, plane.y, plane.z);
            let d = plane.w;
            let effective_radius = half.x * n.x.abs() + half.y * n.y.abs() + half.z * n.z.abs();
            if n.dot(&center) + d < -effective_radius {
                return false; // AABB completely outside this plane
            }
        }
        true
    }
}
