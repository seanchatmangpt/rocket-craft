#[derive(Debug, Clone, Default)]
pub struct VerifierGate {
    pub id: String,
}

impl VerifierGate {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }
}
