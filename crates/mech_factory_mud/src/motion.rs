#[derive(Debug, Clone, Default)]
pub struct MotionSurrogate {
    pub id: String,
}

impl MotionSurrogate {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }
}
