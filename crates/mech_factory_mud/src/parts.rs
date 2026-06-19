#[derive(Debug, Clone, Default)]
pub struct MechPart {
    pub id: String,
}

impl MechPart {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }
}
