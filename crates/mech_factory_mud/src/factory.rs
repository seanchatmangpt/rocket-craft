#[derive(Debug, Clone, Default)]
pub struct FactoryContext {
    pub id: String,
}

impl FactoryContext {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }
}
