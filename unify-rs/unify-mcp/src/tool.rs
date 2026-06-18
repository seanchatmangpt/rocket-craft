/// Descriptor for a single MCP tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Type alias for a boxed tool handler function.
pub type ToolHandler =
    Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>;

/// Registry of MCP tools.
pub struct ToolRegistry {
    descriptors: Vec<ToolDescriptor>,
    handlers: std::collections::HashMap<String, ToolHandler>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register a tool with its descriptor and handler function.
    pub fn register(
        &mut self,
        desc: ToolDescriptor,
        handler: impl Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    ) {
        self.handlers.insert(desc.name.clone(), Box::new(handler));
        self.descriptors.push(desc);
    }

    /// Call a registered tool by name.
    pub fn call(&self, name: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        match self.handlers.get(name) {
            Some(handler) => handler(params),
            None => Err(format!("Unknown tool: {}", name)),
        }
    }

    /// List all registered tool descriptors.
    pub fn list(&self) -> &[ToolDescriptor] {
        &self.descriptors
    }

    /// Check if a tool is registered.
    pub fn has(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_desc(name: &str) -> ToolDescriptor {
        ToolDescriptor {
            name: name.to_string(),
            description: "Test tool".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        }
    }

    #[test]
    fn test_registry_register_has_list() {
        let mut registry = ToolRegistry::new();
        assert!(!registry.has("my_tool"));
        registry.register(make_desc("my_tool"), |_| Ok(json!("completed")));
        assert!(registry.has("my_tool"));
        assert_eq!(registry.list().len(), 1);
        assert_eq!(registry.list()[0].name, "my_tool");
    }

    #[test]
    fn test_registry_call_registered_tool_returns_ok() {
        let mut registry = ToolRegistry::new();
        registry.register(make_desc("echo"), |params| Ok(json!({"echo": params})));
        let result = registry.call("echo", json!({"input": "hello"}));
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["echo"]["input"], json!("hello"));
    }

    #[test]
    fn test_registry_call_unknown_tool_returns_err() {
        let registry = ToolRegistry::new();
        let result = registry.call("nonexistent", json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }
}
