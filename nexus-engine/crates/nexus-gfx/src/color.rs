/// Linear RGB color in [0, 1] range -- enforced via constructor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgb {
    r: f32,
    g: f32,
    b: f32,
}

impl LinearRgb {
    pub fn new(r: f32, g: f32, b: f32) -> Result<Self, GfxError> {
        if !((0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b)) {
            return Err(GfxError::ColorOutOfRange { r, g, b });
        }
        Ok(LinearRgb { r, g, b })
    }
    pub fn r(&self) -> f32 {
        self.r
    }
    pub fn g(&self) -> f32 {
        self.g
    }
    pub fn b(&self) -> f32 {
        self.b
    }
    pub fn to_array(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
    pub fn to_srgb(&self) -> [u8; 3] {
        fn to_srgb_channel(linear: f32) -> u8 {
            let srgb = if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            };
            (srgb * 255.0).round().clamp(0.0, 255.0) as u8
        }
        [
            to_srgb_channel(self.r),
            to_srgb_channel(self.g),
            to_srgb_channel(self.b),
        ]
    }
    // Beam saber colors per series
    pub const NU_GUNDAM_GREEN: LinearRgb = LinearRgb {
        r: 0.0,
        g: 1.0,
        b: 0.3,
    };
    pub const WING_ZERO_GOLD: LinearRgb = LinearRgb {
        r: 1.0,
        g: 0.84,
        b: 0.0,
    };
    pub const UNICORN_NT_D_RED: LinearRgb = LinearRgb {
        r: 1.0,
        g: 0.1,
        b: 0.0,
    };
    pub const FREEDOM_BLUE: LinearRgb = LinearRgb {
        r: 0.2,
        g: 0.4,
        b: 1.0,
    };
    pub const AERIAL_GREEN: LinearRgb = LinearRgb {
        r: 0.1,
        g: 1.0,
        b: 0.4,
    };
}

#[derive(Debug, thiserror::Error)]
pub enum GfxError {
    #[error("NDC coordinates out of range: ({x}, {y}) -- must be in [-1, 1]")]
    OutOfNdcRange { x: f32, y: f32 },
    #[error("invalid FOV: {0} degrees -- must be (0, 180)")]
    InvalidFov(f32),
    #[error("invalid depth range: near={near}, far={far} -- must have 0 < near < far")]
    InvalidDepthRange { near: f32, far: f32 },
    #[error("invalid AABB: min must be <= max on all axes")]
    InvalidAabb,
    #[error("color channel out of [0, 1] range: r={r}, g={g}, b={b}")]
    ColorOutOfRange { r: f32, g: f32, b: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── LinearRgb::new ────────────────────────────────────────────────────────

    #[test]
    fn valid_color_is_accepted() {
        let c = LinearRgb::new(0.5, 0.25, 1.0).unwrap();
        assert!((c.r() - 0.5).abs() < 1e-6);
        assert!((c.g() - 0.25).abs() < 1e-6);
        assert!((c.b() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn zero_color_is_valid() {
        LinearRgb::new(0.0, 0.0, 0.0).unwrap();
    }

    #[test]
    fn full_white_is_valid() {
        LinearRgb::new(1.0, 1.0, 1.0).unwrap();
    }

    #[test]
    fn channel_above_one_is_rejected() {
        assert!(LinearRgb::new(1.1, 0.5, 0.5).is_err());
        assert!(LinearRgb::new(0.5, 1.1, 0.5).is_err());
        assert!(LinearRgb::new(0.5, 0.5, 1.1).is_err());
    }

    #[test]
    fn negative_channel_is_rejected() {
        assert!(LinearRgb::new(-0.1, 0.5, 0.5).is_err());
    }

    // ── LinearRgb::to_array ───────────────────────────────────────────────────

    #[test]
    fn to_array_round_trips_values() {
        let c = LinearRgb::new(0.1, 0.2, 0.3).unwrap();
        let arr = c.to_array();
        assert!((arr[0] - 0.1).abs() < 1e-6);
        assert!((arr[1] - 0.2).abs() < 1e-6);
        assert!((arr[2] - 0.3).abs() < 1e-6);
    }

    // ── LinearRgb::to_srgb ────────────────────────────────────────────────────

    #[test]
    fn linear_black_maps_to_srgb_black() {
        let srgb = LinearRgb::new(0.0, 0.0, 0.0).unwrap().to_srgb();
        assert_eq!(srgb, [0, 0, 0]);
    }

    #[test]
    fn linear_white_maps_to_srgb_255() {
        let srgb = LinearRgb::new(1.0, 1.0, 1.0).unwrap().to_srgb();
        assert_eq!(srgb, [255, 255, 255]);
    }

    #[test]
    fn srgb_conversion_is_nonlinear() {
        // 50% linear is NOT 127 in sRGB — it's brighter (~188)
        let srgb = LinearRgb::new(0.5, 0.5, 0.5).unwrap().to_srgb();
        let mid = srgb[0];
        assert!(mid > 127 + 10, "linear 0.5 should map to sRGB > 137, got {mid}");
        assert!(mid < 200, "linear 0.5 should map to sRGB < 200, got {mid}");
    }

    #[test]
    fn srgb_conversion_below_threshold_uses_linear_segment() {
        // For linear values <= 0.0031308, sRGB = linear * 12.92
        let tiny = 0.001_f32;
        let expected = (tiny * 12.92 * 255.0).round() as u8;
        let srgb = LinearRgb::new(tiny, 0.0, 0.0).unwrap().to_srgb();
        assert_eq!(srgb[0], expected, "below-threshold channel should use linear segment");
    }

    // ── named constants ───────────────────────────────────────────────────────

    #[test]
    fn beam_saber_constants_are_valid_linear_colors() {
        let consts = [
            LinearRgb::NU_GUNDAM_GREEN,
            LinearRgb::WING_ZERO_GOLD,
            LinearRgb::UNICORN_NT_D_RED,
            LinearRgb::FREEDOM_BLUE,
            LinearRgb::AERIAL_GREEN,
        ];
        for c in consts {
            assert!((0.0..=1.0).contains(&c.r()), "r out of range: {}", c.r());
            assert!((0.0..=1.0).contains(&c.g()), "g out of range: {}", c.g());
            assert!((0.0..=1.0).contains(&c.b()), "b out of range: {}", c.b());
        }
    }
}
