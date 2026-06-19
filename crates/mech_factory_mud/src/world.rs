#[derive(Debug, Clone, Default)]
pub struct FactoryWorld {
    pub id: String,
}

impl FactoryWorld {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }
}
