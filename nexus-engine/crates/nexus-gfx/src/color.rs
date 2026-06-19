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
