/// Descriptor for a single MCP resource.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceDescriptor {
    pub uri: String,
    pub name: String,
    pub mime_type: String,
    pub description: String,
}

/// Type alias for a boxed resource handler function.
pub type ResourceHandler = Box<dyn Fn(&str) -> Result<serde_json::Value, String> + Send + Sync>;

/// Registry of MCP resources.
pub struct ResourceRegistry {
    descriptors: Vec<ResourceDescriptor>,
    handlers: std::collections::HashMap<String, ResourceHandler>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register a resource with its descriptor and handler function.
    pub fn register(
        &mut self,
        desc: ResourceDescriptor,
        handler: impl Fn(&str) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    ) {
        self.handlers.insert(desc.uri.clone(), Box::new(handler));
        self.descriptors.push(desc);
    }

    /// Read a resource by URI.
    pub fn read(&self, uri: &str) -> Result<serde_json::Value, String> {
        match self.handlers.get(uri) {
            Some(handler) => handler(uri),
            None => Err(format!("Unknown resource: {}", uri)),
        }
    }

    /// List all registered resource descriptors.
    pub fn list(&self) -> &[ResourceDescriptor] {
        &self.descriptors
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_desc(uri: &str) -> ResourceDescriptor {
        ResourceDescriptor {
            uri: uri.to_string(),
            name: "Test Resource".to_string(),
            mime_type: "application/json".to_string(),
            description: "A test resource".to_string(),
        }
    }

    #[test]
    fn test_resource_registry_register_list_read() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_desc("unify://test/resource"), |_uri| {
            Ok(json!({"content": "hello"}))
        });
        assert_eq!(registry.list().len(), 1);
        assert_eq!(registry.list()[0].uri, "unify://test/resource");
        let result = registry.read("unify://test/resource");
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["content"], json!("hello"));
    }

    #[test]
    fn read_unknown_uri_returns_error() {
        let registry = ResourceRegistry::new();
        let result = registry.read("unify://nonexistent");
        assert!(result.is_err(), "reading unknown URI must fail");
    }

    #[test]
    fn list_empty_registry_returns_empty_slice() {
        let registry = ResourceRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn register_two_resources_both_listed() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_desc("unify://a"), |_| Ok(json!({"id": "a"})));
        registry.register(make_desc("unify://b"), |_| Ok(json!({"id": "b"})));
        assert_eq!(registry.list().len(), 2);
        let uris: Vec<&str> = registry.list().iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"unify://a"));
        assert!(uris.contains(&"unify://b"));
    }

    #[test]
    fn handler_error_propagates_to_read_caller() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_desc("unify://broken"), |_| Err("broken".into()));
        let result = registry.read("unify://broken");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("broken"));
    }
}
