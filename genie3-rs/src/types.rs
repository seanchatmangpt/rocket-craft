use serde::{Deserialize, Serialize};

/// A 3D coordinate, vector, or displacement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// Create a new Vector3.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculate the magnitude (length) of the vector.
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Add another vector.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Subtract another vector.
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Scale the vector by a factor.
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    /// Calculate the Euclidean distance to another Vector3.
    pub fn distance(&self, other: &Self) -> f32 {
        self.sub(other).magnitude()
    }

    /// Normalize the vector. Returns None if magnitude is zero.
    pub fn normalize(&self) -> Option<Self> {
        let mag = self.magnitude();
        if mag > 0.0 {
            Some(self.scale(1.0 / mag))
        } else {
            None
        }
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// 3D Euler angles representation for rotations (in degrees).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Rotation3D {
    /// Rotation around the Y-axis (in degrees).
    pub pitch: f32,
    /// Rotation around the Z-axis (in degrees).
    pub yaw: f32,
    /// Rotation around the X-axis (in degrees).
    pub roll: f32,
}

impl Rotation3D {
    /// Create a new Rotation3D.
    pub const fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }

    /// Add rotation deltas, wrapping angles to [-180, 180] degrees.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            pitch: Self::normalize_angle(self.pitch + other.pitch),
            yaw: Self::normalize_angle(self.yaw + other.yaw),
            roll: Self::normalize_angle(self.roll + other.roll),
        }
    }

    /// Normalize an angle in degrees to be within the [-180, 180] range.
    pub fn normalize_angle(angle: f32) -> f32 {
        let mut normalized = angle % 360.0;
        if normalized > 180.0 {
            normalized -= 360.0;
        } else if normalized < -180.0 {
            normalized += 360.0;
        }
        normalized
    }
}

impl Default for Rotation3D {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// Axis-aligned bounding box (AABB) in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Bounds3D {
    /// The center coordinates of the bounds.
    pub center: Vector3,
    /// The half-extents (half-sizes along each axis).
    pub half_extents: Vector3,
}

impl Bounds3D {
    /// Create a new Bounds3D.
    pub const fn new(center: Vector3, half_extents: Vector3) -> Self {
        Self {
            center,
            half_extents,
        }
    }

    /// Checks if a point is within this bounding box.
    pub fn contains_point(&self, point: &Vector3) -> bool {
        (point.x - self.center.x).abs() <= self.half_extents.x
            && (point.y - self.center.y).abs() <= self.half_extents.y
            && (point.z - self.center.z).abs() <= self.half_extents.z
    }

    /// Checks if this bounding box intersects with another bounding box.
    pub fn intersects(&self, other: &Self) -> bool {
        (self.center.x - other.center.x).abs() <= (self.half_extents.x + other.half_extents.x)
            && (self.center.y - other.center.y).abs()
                <= (self.half_extents.y + other.half_extents.y)
            && (self.center.z - other.center.z).abs()
                <= (self.half_extents.z + other.half_extents.z)
    }
}

impl Default for Bounds3D {
    fn default() -> Self {
        Self {
            center: Vector3::default(),
            half_extents: Vector3::new(100.0, 100.0, 100.0),
        }
    }
}

/// Complete placement definition in the 3D world (position, rotation, scale).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Rotation3D,
    pub scale: Vector3,
}

impl Transform {
    /// Create a new Transform.
    pub const fn new(position: Vector3, rotation: Rotation3D, scale: Vector3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::default(),
            rotation: Rotation3D::default(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;
    fn approx(a: f32, b: f32) -> bool { (a - b).abs() < EPS }

    // ── Vector3 ───────────────────────────────────────────────────────────────

    #[test]
    fn vector3_magnitude_unit_x() {
        let v = Vector3::new(1.0, 0.0, 0.0);
        assert!(approx(v.magnitude(), 1.0));
    }

    #[test]
    fn vector3_magnitude_3_4_0() {
        // 3-4-5 triangle: sqrt(9+16) = 5
        let v = Vector3::new(3.0, 4.0, 0.0);
        assert!(approx(v.magnitude(), 5.0));
    }

    #[test]
    fn vector3_add_is_componentwise() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);
        let c = a.add(&b);
        assert_eq!(c, Vector3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn vector3_sub_is_componentwise() {
        let a = Vector3::new(5.0, 7.0, 9.0);
        let b = Vector3::new(1.0, 2.0, 3.0);
        let c = a.sub(&b);
        assert_eq!(c, Vector3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn vector3_scale() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let s = v.scale(2.0);
        assert_eq!(s, Vector3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn vector3_distance_is_symmetric() {
        let a = Vector3::new(0.0, 0.0, 0.0);
        let b = Vector3::new(1.0, 0.0, 0.0);
        assert!(approx(a.distance(&b), 1.0));
        assert!(approx(b.distance(&a), 1.0));
    }

    #[test]
    fn vector3_normalize_unit_vector() {
        let v = Vector3::new(3.0, 4.0, 0.0); // mag = 5
        let n = v.normalize().unwrap();
        assert!(approx(n.magnitude(), 1.0));
    }

    #[test]
    fn vector3_normalize_zero_returns_none() {
        let v = Vector3::new(0.0, 0.0, 0.0);
        assert!(v.normalize().is_none());
    }

    #[test]
    fn vector3_default_is_origin() {
        assert_eq!(Vector3::default(), Vector3::new(0.0, 0.0, 0.0));
    }

    // ── Rotation3D ────────────────────────────────────────────────────────────

    #[test]
    fn rotation3d_normalize_angle_no_wrap_needed() {
        assert!(approx(Rotation3D::normalize_angle(90.0), 90.0));
        assert!(approx(Rotation3D::normalize_angle(-90.0), -90.0));
    }

    #[test]
    fn rotation3d_normalize_angle_wraps_above_180() {
        // 270 degrees should wrap to -90
        assert!(approx(Rotation3D::normalize_angle(270.0), -90.0));
    }

    #[test]
    fn rotation3d_normalize_angle_wraps_below_neg_180() {
        // -270 degrees should wrap to 90
        assert!(approx(Rotation3D::normalize_angle(-270.0), 90.0));
    }

    #[test]
    fn rotation3d_add_wraps_angles() {
        let a = Rotation3D::new(170.0, 0.0, 0.0);
        let b = Rotation3D::new(20.0, 0.0, 0.0); // total 190 → wraps to -170
        let c = a.add(&b);
        assert!(approx(c.pitch, -170.0));
    }

    #[test]
    fn rotation3d_default_is_zero() {
        let r = Rotation3D::default();
        assert!(approx(r.pitch, 0.0) && approx(r.yaw, 0.0) && approx(r.roll, 0.0));
    }

    // ── Bounds3D ──────────────────────────────────────────────────────────────

    #[test]
    fn bounds3d_contains_center() {
        let b = Bounds3D::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        assert!(b.contains_point(&Vector3::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn bounds3d_excludes_outside_point() {
        let b = Bounds3D::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        assert!(!b.contains_point(&Vector3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn bounds3d_intersects_overlapping() {
        let a = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 2.0));
        let b = Bounds3D::new(Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn bounds3d_no_intersect_when_apart() {
        let a = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let b = Bounds3D::new(Vector3::new(10.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        assert!(!a.intersects(&b));
    }

    // ── Transform ─────────────────────────────────────────────────────────────

    #[test]
    fn transform_default_has_unit_scale() {
        let t = Transform::default();
        assert_eq!(t.scale, Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn transform_default_has_zero_position_and_rotation() {
        let t = Transform::default();
        assert_eq!(t.position, Vector3::default());
        assert_eq!(t.rotation, Rotation3D::default());
    }
}
